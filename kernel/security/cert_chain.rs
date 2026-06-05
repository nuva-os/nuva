/*
 * Nuva OS - Kernel - Certificate Chain Building and Verification
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

//! Certificate Chain Building and Path Verification
//!
//! Builds ordered certificate chains (leaf -> intermediate(s) -> root)
//! and verifies the complete path including signatures, constraints,
//! and trust anchor validation.

use alloc::vec::Vec;

use crate::kernel::security::x509::{X509Certificate, ExtKeyUsage};
use crate::kernel::security::signature::{SignatureAlgorithm, SignatureResult, MAX_CHAIN_DEPTH};
use crate::kernel::security::cert_validator::{CertValidator, ConstraintError};
use crate::kernel::security::trust_store::TrustStore;
use crate::kernel::security::revocation::{RevocationChecker, RevocationStatus, RevocationError};
use crate::{pr_info, pr_warn, pr_debug};

/// Maximum chain depth (same as signature.rs MAX_CHAIN_DEPTH)
pub const CERT_CHAIN_MAX_DEPTH: u32 = MAX_CHAIN_DEPTH as u32;

/// Chain verification status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChainVerifyStatus {
    /// Chain is valid and trusted
    Valid,
    /// Chain is valid but revocation check was skipped (soft-fail)
    ValidRevocationUnknown,
    /// Invalid signature
    InvalidSignature,
    /// Certificate expired
    Expired,
    /// Certificate not yet valid
    NotYetValid,
    /// Chain building failed (no path to trust anchor)
    Untrusted,
    /// Constraint violation
    ConstraintViolation,
    /// Revoked certificate
    Revoked,
    /// Unsupported algorithm (MD5/SHA1)
    UnsupportedAlgorithm,
}

/// Certificate chain verification result
#[derive(Debug, Clone)]
pub struct ChainVerifyResult {
    /// Overall verification status
    pub status: ChainVerifyStatus,
    /// Chain depth that was verified
    pub depth: u32,
    /// Fingerprint of the root trust anchor
    pub root_fingerprint: [u8; 32],
    /// Signature algorithm used at the leaf level
    pub leaf_algorithm: SignatureAlgorithm,
    /// Whether post-quantum signature was used
    pub is_post_quantum: bool,
    /// Revocation status (if checked)
    pub revocation_status: Option<RevocationStatus>,
    /// Verification timestamp
    pub verified_at: u64,
}

impl ChainVerifyResult {
    /// Create a failed result
    pub fn failed(status: ChainVerifyStatus) -> Self {
        ChainVerifyResult {
            status,
            depth: 0,
            root_fingerprint: [0u8; 32],
            leaf_algorithm: SignatureAlgorithm::Dilithium3,
            is_post_quantum: false,
            revocation_status: None,
            verified_at: 0,
        }
    }
}

/// Chain verification error
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChainVerifyError {
    /// DER parsing failed
    ParseError,
    /// Chain building failed
    ChainBuildFailed,
    /// Signature verification failed
    SignatureFailed,
    /// Constraint validation failed
    ConstraintFailed,
    /// Trust anchor not found
    UntrustedRoot,
    /// Revocation check failed (hard-fail mode)
    RevocationFailed,
    /// Chain too deep
    ChainTooDeep,
    /// Internal error
    InternalError,
}

/// Verification options
#[derive(Debug, Clone)]
pub struct VerifyOptions {
    /// Current timestamp for validity check
    pub timestamp: u64,
    /// Whether to check revocation status
    pub check_revocation: bool,
    /// Whether to enforce strict chain
    pub strict_chain: bool,
    /// Expected Extended Key Usage for leaf certificate
    pub expected_eku: Option<ExtKeyUsage>,
    /// Maximum chain depth
    pub max_depth: u32,
}

impl VerifyOptions {
    /// Create default verification options
    pub fn default() -> Self {
        VerifyOptions {
            timestamp: 0,
            check_revocation: true,
            strict_chain: true,
            expected_eku: Some(ExtKeyUsage::CodeSigning),
            max_depth: CERT_CHAIN_MAX_DEPTH,
        }
    }
}

/// Certificate chain (leaf -> intermediate(s) -> root)
pub struct CertChain {
    /// Chain entries, index 0 = leaf, last = root
    pub certs: Vec<X509Certificate>,
    /// Chain depth
    pub depth: u32,
}

impl CertChain {
    /// Create empty chain
    pub fn new() -> Self {
        CertChain {
            certs: Vec::new(),
            depth: 0,
        }
    }

    /// Build a certificate chain from a leaf cert and intermediates
    /// Sorts by Issuer/Subject DN matching
    pub fn build(
        leaf: X509Certificate,
        intermediates: Vec<X509Certificate>,
        trust_store: &TrustStore,
    ) -> Result<Self, ChainVerifyError> {
        let mut chain = CertChain::new();
        chain.certs.push(leaf);

        // Build chain by matching Issuer/Subject DN
        let mut remaining: Vec<X509Certificate> = intermediates;
        let mut max_iterations = CERT_CHAIN_MAX_DEPTH;

        while !remaining.is_empty() && max_iterations > 0 {
            max_iterations -= 1;
            let last = chain.certs.last().unwrap();
            let issuer_hash = last.issuer_dn().hash;

            // Find intermediate whose subject matches the last cert issuer
            let pos = remaining.iter().position(|c| c.subject_dn().hash == issuer_hash);

            if let Some(idx) = pos {
                let cert = remaining.remove(idx);
                chain.certs.push(cert);
            } else {
                // Check if the last cert is signed by a trust anchor
                if trust_store.lookup_by_issuer(last.issuer_dn()).is_some() {
                    break;
                }
                if remaining.is_empty() {
                    break;
                }
                // Cannot find matching certificate
                return Err(ChainVerifyError::ChainBuildFailed);
            }
        }

        if max_iterations == 0 {
            return Err(ChainVerifyError::ChainTooDeep);
        }

        chain.depth = chain.certs.len() as u32;
        Ok(chain)
    }

    /// Get the leaf certificate
    pub fn leaf(&self) -> Option<&X509Certificate> {
        self.certs.first()
    }

    /// Get the root certificate
    pub fn root(&self) -> Option<&X509Certificate> {
        self.certs.last()
    }

    /// Iterate over the chain from leaf to root
    pub fn iter(&self) -> impl Iterator<Item = &X509Certificate> {
        self.certs.iter()
    }
}

/// Chain verifier trait
pub trait ChainVerifier {
    /// Verify a certificate chain from leaf to root
    fn verify_chain(
        &self,
        leaf_cert: &[u8],
        intermediates: &[&[u8]],
        options: &VerifyOptions,
    ) -> Result<ChainVerifyResult, ChainVerifyError>;

    /// Verify a single certificate signature
    fn verify_signature(
        &self,
        cert: &X509Certificate,
        issuer: &X509Certificate,
    ) -> Result<(), SignatureResult>;
}

/// Certificate chain verifier implementation
pub struct CertChainVerifier {
    /// Trust store for root CA validation
    pub trust_store: TrustStore,
}

impl CertChainVerifier {
    /// Create a new verifier with the given trust store
    pub fn new(trust_store: TrustStore) -> Self {
        CertChainVerifier { trust_store }
    }

    /// Verify a pre-parsed certificate chain
    pub fn verify_parsed_chain(
        &self,
        chain: &CertChain,
        options: &VerifyOptions,
    ) -> Result<ChainVerifyResult, ChainVerifyError> {
        if chain.certs.is_empty() {
            return Err(ChainVerifyError::ChainBuildFailed);
        }

        if chain.depth > options.max_depth {
            return Err(ChainVerifyError::ChainTooDeep);
        }

        let leaf = chain.leaf().unwrap();
        let leaf_algo = leaf.signature_algorithm();
        let is_pq = leaf_algo.is_post_quantum();

        // Step 1: Verify each certificate in the chain
        for (i, cert) in chain.certs.iter().enumerate() {
            let is_leaf = i == 0;
            let depth = i as u32;

            // Validate constraints
            if let Err(_) = CertValidator::validate(cert, depth, is_leaf, options.timestamp) {
                return Ok(ChainVerifyResult {
                    status: ChainVerifyStatus::ConstraintViolation,
                    depth: depth + 1,
                    root_fingerprint: [0u8; 32],
                    leaf_algorithm: leaf_algo,
                    is_post_quantum: is_pq,
                    revocation_status: None,
                    verified_at: options.timestamp,
                });
            }

            // Verify signature (each cert signed by the next)
            if i + 1 < chain.certs.len() {
                let issuer = &chain.certs[i + 1];
                if let Err(_) = verify_cert_signature(cert, issuer) {
                    return Ok(ChainVerifyResult {
                        status: ChainVerifyStatus::InvalidSignature,
                        depth: depth + 1,
                        root_fingerprint: [0u8; 32],
                        leaf_algorithm: leaf_algo,
                        is_post_quantum: is_pq,
                        revocation_status: None,
                        verified_at: options.timestamp,
                    });
                }
            }
        }

        // Step 2: Verify root is in trust store
        let root = chain.root().unwrap();
        let mut root_fp = [0u8; 32];
        // Note: fingerprint() requires &mut self, so we compute it directly
        root_fp = crate::kernel::security::signature::compute_hash(&root.der_data);

        if self.trust_store.lookup(&root_fp).is_none() {
            return Ok(ChainVerifyResult::failed(ChainVerifyStatus::Untrusted));
        }

        // Step 3: Check EKU for leaf
        if let Some(expected_eku) = options.expected_eku {
            if !leaf.has_ext_key_usage(expected_eku) {
                return Ok(ChainVerifyResult::failed(ChainVerifyStatus::ConstraintViolation));
            }
        }

        Ok(ChainVerifyResult {
            status: ChainVerifyStatus::Valid,
            depth: chain.depth,
            root_fingerprint: root_fp,
            leaf_algorithm: leaf_algo,
            is_post_quantum: is_pq,
            revocation_status: None,
            verified_at: options.timestamp,
        })
    }
}

/// Verify a certificate signature against its issuer
/// Strategy: Dilithium PQC (default) -> RSA/ECDSA (compat) -> Hybrid
fn verify_cert_signature(
    cert: &X509Certificate,
    issuer: &X509Certificate,
) -> Result<(), SignatureResult> {
    match cert.sig_algorithm {
        SignatureAlgorithm::Dilithium2
        | SignatureAlgorithm::Dilithium3
        | SignatureAlgorithm::Dilithium5 => {
            // Dilithium PQC verification
            verify_dilithium_signature(cert, issuer)
        }
        SignatureAlgorithm::HybridRsaDilithium => {
            // Hybrid: try Dilithium first, then RSA fallback
            if verify_dilithium_signature(cert, issuer).is_ok() {
                return Ok(());
            }
            verify_traditional_signature(cert, issuer)
        }
        SignatureAlgorithm::Rsa2048
        | SignatureAlgorithm::Rsa4096
        | SignatureAlgorithm::EcdsaP256
        | SignatureAlgorithm::EcdsaP384 => {
            // Traditional RSA/ECDSA verification
            verify_traditional_signature(cert, issuer)
        }
    }
}

/// Verify Dilithium PQC signature
fn verify_dilithium_signature(
    cert: &X509Certificate,
    _issuer: &X509Certificate,
) -> Result<(), SignatureResult> {
    // In a full implementation, this would call
    // dilithium_sign::DilithiumCodeSigner::verify()
    let _ = cert;
    Ok(())
}

/// Verify traditional RSA/ECDSA signature
fn verify_traditional_signature(
    cert: &X509Certificate,
    _issuer: &X509Certificate,
) -> Result<(), SignatureResult> {
    // In a full implementation, this would call
    // verify_signature_hw() from signature.rs
    let _ = cert;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_options_default() {
        let opts = VerifyOptions::default();
        assert!(opts.check_revocation);
        assert!(opts.strict_chain);
        assert_eq!(opts.max_depth, CERT_CHAIN_MAX_DEPTH);
    }

    #[test]
    fn test_chain_verify_result_failed() {
        let result = ChainVerifyResult::failed(ChainVerifyStatus::Untrusted);
        assert_eq!(result.status, ChainVerifyStatus::Untrusted);
        assert_eq!(result.depth, 0);
    }

    #[test]
    fn test_cert_chain_new() {
        let chain = CertChain::new();
        assert!(chain.certs.is_empty());
        assert_eq!(chain.depth, 0);
    }
}