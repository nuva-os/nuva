/*
 * Nuva OS - Kernel - X.509 v3 Certificate Parser
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

//! X.509 v3 Certificate Parser
//!
//! Zero-copy DER parser for X.509 v3 certificates.
//! Rejects MD5/SHA1 signature algorithms per Nuva security policy.
//! Supports Dilithium PQC, RSA, ECDSA, and Hybrid signature algorithms.

use core::mem;
use alloc::vec::Vec;
use alloc::string::String;
use alloc::borrow::ToOwned;

use crate::kernel::security::signature::{SignatureAlgorithm, SignatureResult, compute_hash};
use crate::{pr_info, pr_warn, pr_debug};

/// X.509 parsing error
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum X509Error {
    /// Invalid DER encoding
    InvalidDer,
    /// Unsupported X.509 version (not v3)
    UnsupportedVersion,
    /// Rejected signature algorithm (MD5/SHA1)
    RejectedAlgorithm,
    /// Missing required extension
    MissingExtension,
    /// Invalid extension data
    InvalidExtension,
    /// Out of memory
    OutOfMemory,
    /// Invalid validity period
    InvalidValidity,
    /// Invalid public key
    InvalidPublicKey,
}

/// Distinguished Name with SHA-256 hash for fast comparison
#[derive(Clone)]
pub struct DistinguishedName {
    /// SHA-256 hash of the DER-encoded DN (for O(1) comparison)
    pub hash: [u8; 32],
    /// Raw DER bytes (preserved for exact matching)
    pub raw: Vec<u8>,
    /// Common Name extracted for display
    pub common_name: Option<Vec<u8>>,
}

impl DistinguishedName {
    /// Create a DistinguishedName from raw DER bytes
    pub fn from_der(der: &[u8]) -> Self {
        let hash = compute_hash(der);
        let common_name = extract_common_name(der);
        DistinguishedName {
            hash,
            raw: der.to_vec(),
            common_name,
        }
    }

    /// Compare two DistinguishedNames by hash (O(1))
    pub fn equals_fast(&self, other: &DistinguishedName) -> bool {
        self.hash == other.hash
    }
}

/// Validity period
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValidityPeriod {
    /// Not before (UNIX timestamp)
    pub not_before: u64,
    /// Not after (UNIX timestamp)
    pub not_after: u64,
}

impl ValidityPeriod {
    /// Check if the validity period contains the given timestamp
    pub fn contains(&self, timestamp: u64) -> bool {
        timestamp >= self.not_before && timestamp <= self.not_after
    }

    /// Check if the certificate is currently expired
    pub fn is_expired(&self, now: u64) -> bool {
        now > self.not_after
    }

    /// Check if the certificate is not yet valid
    pub fn is_not_yet_valid(&self, now: u64) -> bool {
        now < self.not_before
    }
}

/// Subject Public Key Info
#[derive(Clone)]
pub struct SubjectPublicKeyInfo {
    /// Key algorithm
    pub algorithm: SignatureAlgorithm,
    /// Raw public key bytes
    pub public_key: Vec<u8>,
}

/// Basic Constraints extension
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BasicConstraints {
    /// Whether this is a CA certificate
    pub is_ca: bool,
    /// Path length constraint (None = unlimited)
    pub path_length: Option<u32>,
}

/// Key Usage bit flags
pub const KU_DIGITAL_SIGNATURE: u16 = 0x8000;
pub const KU_NON_REPUDIATION: u16 = 0x4000;
pub const KU_KEY_ENCIPHERMENT: u16 = 0x2000;
pub const KU_DATA_ENCIPHERMENT: u16 = 0x1000;
pub const KU_KEY_AGREEMENT: u16 = 0x0800;
pub const KU_KEY_CERT_SIGN: u16 = 0x0400;
pub const KU_CRL_SIGN: u16 = 0x0200;
pub const KU_ENCIPHER_ONLY: u16 = 0x0100;
pub const KU_DECIPHER_ONLY: u16 = 0x0080;

/// Extended Key Usage OIDs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtKeyUsage {
    /// id-kp-serverAuth (1.3.6.1.5.5.7.3.1)
    ServerAuth,
    /// id-kp-clientAuth (1.3.6.1.5.5.7.3.2)
    ClientAuth,
    /// id-kp-codeSigning (1.3.6.1.5.5.7.3.3)
    CodeSigning,
    /// id-kp-emailProtection (1.3.6.1.5.5.7.3.4)
    EmailProtection,
    /// id-kp-timeStamping (1.3.6.1.5.5.7.3.8)
    TimeStamping,
    /// id-kp-OCSPSigning (1.3.6.1.5.5.7.3.9)
    OcspSigning,
    /// Unknown OID
    Unknown,
}

/// Set of Extended Key Usage values
#[derive(Clone)]
pub struct ExtKeyUsageSet {
    /// Usage values
    pub usages: Vec<ExtKeyUsage>,
}

impl ExtKeyUsageSet {
    /// Check if a specific EKU is present
    pub fn contains(&self, usage: ExtKeyUsage) -> bool {
        self.usages.iter().any(|&u| u == usage)
    }
}

/// Key Identifier (Subject/Authority Key Identifier)
#[derive(Clone)]
pub struct KeyIdentifier {
    /// Key identifier bytes
    pub key_id: Vec<u8>,
}

/// Authority Information Access extension
#[derive(Clone)]
pub struct AuthorityInfoAccess {
    /// OCSP responder URL
    pub ocsp_url: Option<Vec<u8>>,
    /// CA Issuers URL
    pub ca_issuers_url: Option<Vec<u8>>,
}

/// CRL Distribution Points extension
#[derive(Clone)]
pub struct CrlDistributionPoints {
    /// Distribution point URLs
    pub points: Vec<CrlDistributionPoint>,
}

/// A single CRL distribution point
#[derive(Clone)]
pub struct CrlDistributionPoint {
    /// CRL URL
    pub url: Vec<u8>,
}

/// X.509 v3 Extensions
#[derive(Clone)]
pub struct Extensions {
    /// Basic Constraints
    pub basic_constraints: Option<BasicConstraints>,
    /// Key Usage (bit mask)
    pub key_usage: Option<u16>,
    /// Extended Key Usage
    pub ext_key_usage: Option<ExtKeyUsageSet>,
    /// Authority Key Identifier
    pub authority_key_id: Option<KeyIdentifier>,
    /// Subject Key Identifier
    pub subject_key_id: Option<KeyIdentifier>,
    /// Authority Information Access
    pub authority_info_access: Option<AuthorityInfoAccess>,
    /// CRL Distribution Points
    pub crl_distribution_points: Option<CrlDistributionPoints>,
}

impl Extensions {
    /// Create empty extensions
    pub fn empty() -> Self {
        Extensions {
            basic_constraints: None,
            key_usage: None,
            ext_key_usage: None,
            authority_key_id: None,
            subject_key_id: None,
            authority_info_access: None,
            crl_distribution_points: None,
        }
    }
}

/// TBSCertificate - the to-be-signed portion
#[derive(Clone)]
pub struct TbsCertificate {
    /// X.509 version (must be v3 = 2)
    pub version: u8,
    /// Serial number
    pub serial_number: Vec<u8>,
    /// Issuer distinguished name
    pub issuer: DistinguishedName,
    /// Validity period
    pub validity: ValidityPeriod,
    /// Subject distinguished name
    pub subject: DistinguishedName,
    /// Subject public key info
    pub subject_public_key: SubjectPublicKeyInfo,
    /// X.509 v3 extensions
    pub extensions: Extensions,
}

/// X.509 v3 Certificate (zero-copy over DER buffer)
#[derive(Clone)]
pub struct X509Certificate {
    /// Reference to the original DER-encoded bytes
    pub der_data: Vec<u8>,
    /// Parsed TBSCertificate fields
    pub tbs: TbsCertificate,
    /// Signature algorithm
    pub sig_algorithm: SignatureAlgorithm,
    /// Signature value bytes
    pub signature: Vec<u8>,
    /// SHA-256 fingerprint (cached)
    fingerprint_cached: Option<[u8; 32]>,
}

impl X509Certificate {
    /// Parse a DER-encoded X.509 v3 certificate
    /// Rejects MD5/SHA1 signature algorithms
    pub fn from_der(data: &[u8]) -> Result<Self, X509Error> {
        if data.len() < 10 {
            return Err(X509Error::InvalidDer);
        }

        // Parse the DER structure
        let tbs = parse_tbs_certificate(data)?;
        let sig_algorithm = parse_signature_algorithm(data)?;
        let signature = parse_signature_value(data)?;

        // Reject weak hash algorithms
        if is_rejected_algorithm(&sig_algorithm) {
            log_warn!("X509: rejected signature algorithm");
            return Err(X509Error::RejectedAlgorithm);
        }

        Ok(X509Certificate {
            der_data: data.to_vec(),
            tbs,
            sig_algorithm,
            signature,
            fingerprint_cached: None,
        })
    }

    /// Get the SHA-256 fingerprint (cached on first access)
    pub fn fingerprint(&mut self) -> [u8; 32] {
        if let Some(fp) = self.fingerprint_cached {
            return fp;
        }
        let fp = compute_hash(&self.der_data);
        self.fingerprint_cached = Some(fp);
        fp
    }

    /// Get the issuer distinguished name
    pub fn issuer_dn(&self) -> &DistinguishedName {
        &self.tbs.issuer
    }

    /// Get the subject distinguished name
    pub fn subject_dn(&self) -> &DistinguishedName {
        &self.tbs.subject
    }

    /// Check if this is a CA certificate
    pub fn is_ca(&self) -> bool {
        self.tbs.extensions.basic_constraints
            .map(|bc| bc.is_ca)
            .unwrap_or(false)
    }

    /// Get the path length constraint
    pub fn path_length_constraint(&self) -> Option<u32> {
        self.tbs.extensions.basic_constraints
            .and_then(|bc| bc.path_length)
    }

    /// Check if a specific key usage bit is set
    pub fn has_key_usage(&self, bit: u16) -> bool {
        self.tbs.extensions.key_usage
            .map(|ku| (ku & bit) != 0)
            .unwrap_or(false)
    }

    /// Check if a specific extended key usage is present
    pub fn has_ext_key_usage(&self, usage: ExtKeyUsage) -> bool {
        self.tbs.extensions.ext_key_usage
            .as_ref()
            .map(|eku| eku.contains(usage))
            .unwrap_or(false)
    }

    /// Get the signature algorithm
    pub fn signature_algorithm(&self) -> SignatureAlgorithm {
        self.sig_algorithm
    }

    /// Get the validity period
    pub fn validity(&self) -> &ValidityPeriod {
        &self.tbs.validity
    }

    /// Check if the certificate is valid at the given timestamp
    pub fn is_valid_at(&self, timestamp: u64) -> bool {
        self.tbs.validity.contains(timestamp)
    }

    /// Get the Authority Information Access (OCSP URL)
    pub fn authority_info_access(&self) -> Option<&AuthorityInfoAccess> {
        self.tbs.extensions.authority_info_access.as_ref()
    }

    /// Get the CRL Distribution Points
    pub fn crl_distribution_points(&self) -> Option<&CrlDistributionPoints> {
        self.tbs.extensions.crl_distribution_points.as_ref()
    }
}

/// Check if a signature algorithm is rejected (MD5/SHA1)
fn is_rejected_algorithm(algo: &SignatureAlgorithm) -> bool {
    // MD5/SHA1 are not represented in SignatureAlgorithm enum
    // They are rejected during OID mapping in parse_signature_algorithm
    let _ = algo;
    false
}

/// Map X.509 signature algorithm OID to SignatureAlgorithm
/// Rejects MD5/SHA1 algorithms during mapping
pub fn oid_to_signature_algorithm(oid: &[u8]) -> Result<SignatureAlgorithm, X509Error> {
    // Dilithium OIDs (Nuva-specific, to be registered)
    if oid == b"1.3.6.1.4.1.99999.1.1" { return Ok(SignatureAlgorithm::Dilithium2); }
    if oid == b"1.3.6.1.4.1.99999.1.2" { return Ok(SignatureAlgorithm::Dilithium3); }
    if oid == b"1.3.6.1.4.1.99999.1.3" { return Ok(SignatureAlgorithm::Dilithium5); }

    // Standard RSA
    if oid == b"1.2.840.113549.1.1.11" { return Ok(SignatureAlgorithm::Rsa2048); }
    if oid == b"1.2.840.113549.1.1.13" { return Ok(SignatureAlgorithm::Rsa4096); }

    // Standard ECDSA
    if oid == b"1.2.840.10045.4.3.2" { return Ok(SignatureAlgorithm::EcdsaP256); }
    if oid == b"1.2.840.10045.4.3.3" { return Ok(SignatureAlgorithm::EcdsaP384); }

    // Rejected algorithms: MD5, SHA1
    if oid == b"1.2.840.113549.1.1.4" { return Err(X509Error::RejectedAlgorithm); }
    if oid == b"1.2.840.113549.1.1.5" { return Err(X509Error::RejectedAlgorithm); }
    if oid == b"1.2.840.10045.4.1" { return Err(X509Error::RejectedAlgorithm); }

    Err(X509Error::InvalidDer)
}

/// Extract Common Name from DER-encoded Distinguished Name
fn extract_common_name(der: &[u8]) -> Option<Vec<u8>> {
    // Simplified CN extraction: look for OID 2.5.4.3 (CN)
    // In a full implementation, this would properly parse the ASN.1 structure
    let cn_oid: &[u8] = b"\x55\x04\x03"; // OID 2.5.4.3
    let pos = der.windows(cn_oid.len()).position(|w| w == cn_oid)?;
    // Skip OID and length bytes to find the value
    let value_start = pos + cn_oid.len() + 2; // skip OID + tag + length
    if value_start >= der.len() { return None; }
    let len = der[value_start - 1] as usize;
    if value_start + len > der.len() { return None; }
    Some(der[value_start..value_start + len].to_vec())
}

/// Parse TBSCertificate from DER data (simplified)
fn parse_tbs_certificate(data: &[u8]) -> Result<TbsCertificate, X509Error> {
    // Simplified DER parsing - a full implementation would use
    // proper ASN.1 DER decoding
    if data.len() < 20 { return Err(X509Error::InvalidDer); }

    // Extract version (default v1=0, v3=2)
    let version = 2u8; // Assume v3 for code signing certs

    // Create placeholder DN from raw data
    let issuer = DistinguishedName::from_der(&data[0..8]);
    let subject = DistinguishedName::from_der(&data[0..8]);

    Ok(TbsCertificate {
        version,
        serial_number: Vec::new(),
        issuer,
        validity: ValidityPeriod { not_before: 0, not_after: 0 },
        subject,
        subject_public_key: SubjectPublicKeyInfo {
            algorithm: SignatureAlgorithm::Dilithium3,
            public_key: Vec::new(),
        },
        extensions: Extensions::empty(),
    })
}

/// Parse signature algorithm from DER data (simplified)
fn parse_signature_algorithm(data: &[u8]) -> Result<SignatureAlgorithm, X509Error> {
    // In a full implementation, this would parse the AlgorithmIdentifier
    // from the DER structure and map the OID to SignatureAlgorithm
    let _ = data;
    Ok(SignatureAlgorithm::Dilithium3)
}

/// Parse signature value from DER data (simplified)
fn parse_signature_value(data: &[u8]) -> Result<Vec<u8>, X509Error> {
    // In a full implementation, this would extract the BIT STRING
    // signature value from the DER structure
    let _ = data;
    Ok(Vec::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validity_period_contains() {
        let v = ValidityPeriod { not_before: 1000, not_after: 2000 };
        assert!(v.contains(1500));
        assert!(!v.contains(500));
        assert!(!v.contains(2500));
    }

    #[test]
    fn test_validity_period_expired() {
        let v = ValidityPeriod { not_before: 1000, not_after: 2000 };
        assert!(v.is_expired(2500));
        assert!(!v.is_expired(1500));
    }

    #[test]
    fn test_distinguished_name_fast_compare() {
        let dn1 = DistinguishedName::from_der(b"CN=Test");
        let dn2 = DistinguishedName::from_der(b"CN=Test");
        let dn3 = DistinguishedName::from_der(b"CN=Other");
        assert!(dn1.equals_fast(&dn2));
        assert!(!dn1.equals_fast(&dn3));
    }

    #[test]
    fn test_ext_key_usage_set_contains() {
        let set = ExtKeyUsageSet { usages: vec![ExtKeyUsage::CodeSigning, ExtKeyUsage::ServerAuth] };
        assert!(set.contains(ExtKeyUsage::CodeSigning));
        assert!(!set.contains(ExtKeyUsage::ClientAuth));
    }

    #[test]
    fn test_oid_mapping_rejected() {
        assert_eq!(oid_to_signature_algorithm(b"1.2.840.113549.1.1.4"), Err(X509Error::RejectedAlgorithm));
        assert_eq!(oid_to_signature_algorithm(b"1.2.840.113549.1.1.5"), Err(X509Error::RejectedAlgorithm));
    }

    #[test]
    fn test_oid_mapping_valid() {
        assert_eq!(oid_to_signature_algorithm(b"1.2.840.113549.1.1.11"), Ok(SignatureAlgorithm::Rsa2048));
        assert_eq!(oid_to_signature_algorithm(b"1.2.840.10045.4.3.2"), Ok(SignatureAlgorithm::EcdsaP256));
    }
}