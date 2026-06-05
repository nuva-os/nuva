/*
 * Nuva OS - Kernel - Capability Guard for Trust Store
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

//! Capability Guard for Trust Store Write Operations
//!
//! Enforces CAP_TRUST_ADMIN capability requirement for modifying
//! the trust store (adding/removing root CAs, changing policy).
//! This is a Nuva capability-based security model, not UID-based.

use core::sync::atomic::{AtomicU64, Ordering};

use crate::kernel::security::capability::CapSet;

/// CAP_TRUST_ADMIN capability constant
/// Extends the existing POSIX capability namespace (next after AUDIT_READ = 37)
pub const CAP_TRUST_ADMIN: u32 = 38;

/// Trust store error type for capability guard
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrustStoreError {
    /// CAP_TRUST_ADMIN not held
    PermissionDenied,
    /// Certificate already exists in trust store
    AlreadyExists,
    /// Certificate not found in trust store
    NotFound,
    /// Invalid certificate data
    InvalidCertificate,
    /// Parse error when processing certificate
    ParseError,
    /// Trust store is corrupted
    Corrupted,
    /// Internal error
    InternalError,
}

/// Capability guard for trust store write operations
///
/// Checks that the caller holds CAP_TRUST_ADMIN before allowing
/// modifications to the trust store. This implements the Nuva
/// capability-based security model for trust management.
pub struct CapabilityGuard {
    /// Number of permission checks performed
    check_count: AtomicU64,
    /// Number of permission denials
    deny_count: AtomicU64,
}

impl CapabilityGuard {
    /// Create a new capability guard
    pub const fn new() -> Self {
        CapabilityGuard {
            check_count: AtomicU64::new(0),
            deny_count: AtomicU64::new(0),
        }
    }

    /// Check if the capability set holds CAP_TRUST_ADMIN
    ///
    /// Returns Ok(()) if the capability is present, Err otherwise.
    pub fn check_trust_admin(&self, cap_set: &CapSet) -> Result<(), TrustStoreError> {
        self.check_count.fetch_add(1, Ordering::AcqRel);
        if cap_set.has(CAP_TRUST_ADMIN) {
            Ok(())
        } else {
            self.deny_count.fetch_add(1, Ordering::AcqRel);
            Err(TrustStoreError::PermissionDenied)
        }
    }

    /// Check if the capability set holds CAP_TRUST_ADMIN or SYS_ADMIN
    ///
    /// SYS_ADMIN (cap 21) is a superset capability that also grants
    /// trust store write access for backward compatibility.
    pub fn check_trust_admin_or_sysadmin(&self, cap_set: &CapSet) -> Result<(), TrustStoreError> {
        self.check_count.fetch_add(1, Ordering::AcqRel);
        if cap_set.has(CAP_TRUST_ADMIN) || cap_set.has(21) {
            Ok(())
        } else {
            self.deny_count.fetch_add(1, Ordering::AcqRel);
            Err(TrustStoreError::PermissionDenied)
        }
    }

    /// Get the number of permission checks performed
    pub fn check_count(&self) -> u64 {
        self.check_count.load(Ordering::Acquire)
    }

    /// Get the number of permission denials
    pub fn deny_count(&self) -> u64 {
        self.deny_count.load(Ordering::Acquire)
    }
}

/// Security context for passing capability tokens
///
/// Wraps a reference to a CapSet so that trust store operations
/// can verify capabilities without needing the full SecurityContext.
pub struct SecurityToken<'a> {
    /// Reference to the capability set
    pub caps: &'a CapSet,
}

impl<'a> SecurityToken<'a> {
    /// Create a new security token from a capability set
    pub fn new(caps: &'a CapSet) -> Self {
        SecurityToken { caps }
    }

    /// Check if CAP_TRUST_ADMIN is held
    pub fn has_trust_admin(&self) -> bool {
        self.caps.has(CAP_TRUST_ADMIN)
    }

    /// Check if SYS_ADMIN is held
    pub fn has_sysadmin(&self) -> bool {
        self.caps.has(21)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cap_trust_admin_constant() {
        assert_eq!(CAP_TRUST_ADMIN, 38);
    }

    #[test]
    fn test_capability_guard_check_allowed() {
        let guard = CapabilityGuard::new();
        let mut caps = CapSet::new();
        caps.set(CAP_TRUST_ADMIN);
        assert!(guard.check_trust_admin(&caps).is_ok());
    }

    #[test]
    fn test_capability_guard_check_denied() {
        let guard = CapabilityGuard::new();
        let caps = CapSet::new();
        assert_eq!(guard.check_trust_admin(&caps), Err(TrustStoreError::PermissionDenied));
    }

    #[test]
    fn test_capability_guard_sysadmin_fallback() {
        let guard = CapabilityGuard::new();
        let mut caps = CapSet::new();
        caps.set(21);
        assert!(guard.check_trust_admin_or_sysadmin(&caps).is_ok());
    }

    #[test]
    fn test_capability_guard_stats() {
        let guard = CapabilityGuard::new();
        let caps = CapSet::new();
        let _ = guard.check_trust_admin(&caps);
        let _ = guard.check_trust_admin(&caps);
        assert_eq!(guard.check_count(), 2);
        assert_eq!(guard.deny_count(), 2);
    }

    #[test]
    fn test_security_token() {
        let mut caps = CapSet::new();
        caps.set(CAP_TRUST_ADMIN);
        let token = SecurityToken::new(&caps);
        assert!(token.has_trust_admin());
        assert!(!token.has_sysadmin());
    }
}