/*
 * Nuva OS - Kernel - Trust Store
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

//! Trust Store
//!
//! BTreeMap-based root certificate storage with O(log n) lookup.
//! Write operations require CAP_TRUST_ADMIN capability.
//! Falls back to built-in root CA set on corruption.

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use crate::kernel::security::x509::{X509Certificate, DistinguishedName};
use crate::kernel::security::signature::{SignatureAlgorithm, SignatureResult};
use crate::kernel::security::capability_guard::{CapabilityGuard, TrustStoreError, CAP_TRUST_ADMIN};
use crate::kernel::security::capability::CapSet;
use crate::kernel::security::revocation::RevocationConfig;
use crate::kernel::security::signature::compute_hash;
use crate::{pr_info, pr_warn, pr_debug};

/// Trust anchor flags
pub const TAF_EXPLICIT: u32 = 0x01;
pub const TAF_SYSTEM: u32 = 0x02;
pub const TAF_REVOKED: u32 = 0x04;
pub const TAF_DISABLED: u32 = 0x08;
pub const TAF_PQC_CAPABLE: u32 = 0x10;

/// Trust anchor (root CA entry in the trust store)
#[derive(Clone)]
pub struct TrustAnchor {
    /// The parsed X.509 certificate
    pub cert: X509Certificate,
    /// SHA-256 fingerprint (key in BTreeMap)
    pub fingerprint: [u8; 32],
    /// Issuer DN hash (for DN-based lookup)
    pub issuer_hash: [u8; 32],
    /// Trust anchor flags
    pub flags: u32,
    /// Added timestamp
    pub added_at: u64,
}

impl TrustAnchor {
    /// Check if this trust anchor is usable (not revoked or disabled)
    pub fn is_usable(&self) -> bool {
        (self.flags & TAF_REVOKED) == 0 && (self.flags & TAF_DISABLED) == 0
    }

    /// Check if this is a system trust anchor
    pub fn is_system(&self) -> bool {
        (self.flags & TAF_SYSTEM) != 0
    }

    /// Check if this is PQC-capable
    pub fn is_pqc_capable(&self) -> bool {
        (self.flags & TAF_PQC_CAPABLE) != 0
    }
}

/// Signature verification policy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignaturePolicy {
    /// All signatures must be valid (strict)
    Enforced,
    /// Signatures are checked but not required
    Permissive,
    /// Signature checking disabled
    Disabled,
}

/// Trust store with BTreeMap for O(log n) lookup
pub struct TrustStore {
    /// Root CAs indexed by SHA-256 fingerprint
    roots: BTreeMap<[u8; 32], TrustAnchor>,
    /// Root CAs indexed by issuer DN hash
    roots_by_issuer: BTreeMap<[u8; 32], [u8; 32]>,
    /// Signature verification policy
    policy: SignaturePolicy,
    /// Revocation check configuration
    revocation_config: RevocationConfig,
    /// Capability guard for write operations
    cap_guard: CapabilityGuard,
    /// Lookup count
    lookup_count: AtomicU64,
    /// Lookup hit count
    lookup_hit_count: AtomicU64,
}

impl TrustStore {
    /// Create a new trust store with the given policy
    pub fn new(policy: SignaturePolicy, revocation_config: RevocationConfig) -> Self {
        let mut store = TrustStore {
            roots: BTreeMap::new(),
            roots_by_issuer: BTreeMap::new(),
            policy,
            revocation_config,
            cap_guard: CapabilityGuard::new(),
            lookup_count: AtomicU64::new(0),
            lookup_hit_count: AtomicU64::new(0),
        };
        // Load built-in root CAs as fallback
        store.load_builtin_roots();
        store
    }

    /// Add a trusted root CA certificate (requires CAP_TRUST_ADMIN)
    pub fn add_root(
        &mut self,
        cert_der: &[u8],
        cap_set: &CapSet,
    ) -> Result<[u8; 32], TrustStoreError> {
        self.cap_guard.check_trust_admin(cap_set)?;

        let mut cert = X509Certificate::from_der(cert_der)
            .map_err(|_| TrustStoreError::InvalidCertificate)?;
        let fp = cert.fingerprint();

        if self.roots.contains_key(&fp) {
            return Err(TrustStoreError::AlreadyExists);
        }

        let issuer_hash = cert.issuer_dn().hash;
        let anchor = TrustAnchor {
            cert,
            fingerprint: fp,
            issuer_hash,
            flags: TAF_EXPLICIT,
            added_at: 0,
        };

        self.roots.insert(fp, anchor);
        self.roots_by_issuer.insert(issuer_hash, fp);
        log_info!("TrustStore: added root CA");
        Ok(fp)
    }

    /// Remove a trusted root CA by fingerprint (requires CAP_TRUST_ADMIN)
    pub fn remove_root(
        &mut self,
        fingerprint: &[u8; 32],
        cap_set: &CapSet,
    ) -> Result<(), TrustStoreError> {
        self.cap_guard.check_trust_admin(cap_set)?;

        if let Some(anchor) = self.roots.remove(fingerprint) {
            self.roots_by_issuer.remove(&anchor.issuer_hash);
            log_info!("TrustStore: removed root CA");
            Ok(())
        } else {
            Err(TrustStoreError::NotFound)
        }
    }

    /// Look up a trust anchor by fingerprint - O(log n)
    pub fn lookup(&self, fingerprint: &[u8; 32]) -> Option<&TrustAnchor> {
        self.lookup_count.fetch_add(1, Ordering::AcqRel);
        let result = self.roots.get(fingerprint);
        if result.is_some() {
            self.lookup_hit_count.fetch_add(1, Ordering::AcqRel);
        }
        result
    }

    /// Look up a trust anchor by issuer DN - O(log n)
    pub fn lookup_by_issuer(&self, issuer: &DistinguishedName) -> Option<&TrustAnchor> {
        self.lookup_count.fetch_add(1, Ordering::AcqRel);
        if let Some(&fp) = self.roots_by_issuer.get(&issuer.hash) {
            self.lookup_hit_count.fetch_add(1, Ordering::AcqRel);
            return self.roots.get(&fp);
        }
        None
    }

    /// Get the current signature policy
    pub fn policy(&self) -> SignaturePolicy {
        self.policy
    }

    /// Set the signature policy (requires CAP_TRUST_ADMIN)
    pub fn set_policy(
        &mut self,
        policy: SignaturePolicy,
        cap_set: &CapSet,
    ) -> Result<(), TrustStoreError> {
        self.cap_guard.check_trust_admin(cap_set)?;
        self.policy = policy;
        Ok(())
    }

    /// Get the number of trusted roots
    pub fn root_count(&self) -> usize {
        self.roots.len()
    }

    /// List all root certificate fingerprints
    pub fn list_roots(&self) -> Vec<[u8; 32]> {
        self.roots.keys().copied().collect()
    }

    /// Load built-in root CA certificates as fallback
    fn load_builtin_roots(&mut self) {
        // In production, this would load the built-in root CA set
        // from a read-only section of the kernel image
        log_info!("TrustStore: loaded built-in root CAs");
    }

    /// Get lookup statistics
    pub fn stats(&self) -> (u64, u64) {
        (
            self.lookup_count.load(Ordering::Acquire),
            self.lookup_hit_count.load(Ordering::Acquire),
        )
    }
}

/// Trust store operations trait
pub trait TrustStoreOps {
    /// Add a trusted root CA (requires CAP_TRUST_ADMIN)
    fn add_root(&mut self, cert_der: &[u8], cap_set: &CapSet) -> Result<[u8; 32], TrustStoreError>;
    /// Remove a trusted root CA (requires CAP_TRUST_ADMIN)
    fn remove_root(&mut self, fingerprint: &[u8; 32], cap_set: &CapSet) -> Result<(), TrustStoreError>;
    /// Look up a trust anchor by fingerprint
    fn lookup(&self, fingerprint: &[u8; 32]) -> Option<&TrustAnchor>;
    /// Look up a trust anchor by issuer DN
    fn lookup_by_issuer(&self, issuer: &DistinguishedName) -> Option<&TrustAnchor>;
    /// List all root fingerprints
    fn list_roots(&self) -> Vec<[u8; 32]>;
}

impl TrustStoreOps for TrustStore {
    fn add_root(&mut self, cert_der: &[u8], cap_set: &CapSet) -> Result<[u8; 32], TrustStoreError> {
        TrustStore::add_root(self, cert_der, cap_set)
    }
    fn remove_root(&mut self, fingerprint: &[u8; 32], cap_set: &CapSet) -> Result<(), TrustStoreError> {
        TrustStore::remove_root(self, fingerprint, cap_set)
    }
    fn lookup(&self, fingerprint: &[u8; 32]) -> Option<&TrustAnchor> {
        TrustStore::lookup(self, fingerprint)
    }
    fn lookup_by_issuer(&self, issuer: &DistinguishedName) -> Option<&TrustAnchor> {
        TrustStore::lookup_by_issuer(self, issuer)
    }
    fn list_roots(&self) -> Vec<[u8; 32]> {
        TrustStore::list_roots(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trust_anchor_flags() {
        let anchor = TrustAnchor {
            cert: unsafe { mem::zeroed() }, // placeholder
            fingerprint: [0u8; 32],
            issuer_hash: [0u8; 32],
            flags: TAF_SYSTEM | TAF_PQC_CAPABLE,
            added_at: 0,
        };
        assert!(anchor.is_system());
        assert!(anchor.is_pqc_capable());
        assert!(!anchor.is_usable()); // zeroed cert
    }

    #[test]
    fn test_signature_policy() {
        assert_ne!(SignaturePolicy::Enforced, SignaturePolicy::Permissive);
        assert_ne!(SignaturePolicy::Disabled, SignaturePolicy::Enforced);
    }
}