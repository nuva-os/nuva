/*
 * Plugin Audit - Security Review and Signing Workflow
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

//! Plugin audit: security review, approval, and release signing workflow

use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;

use crate::hal::quantum::pqc::dilithium::{
    Dilithium, DilithiumVariant, DilithiumError,
    PublicKey, SecretKey, Signature,
};
use super::core::{PluginId, PluginError};
use super::signature::{PluginSignature, SignaturePolicy, TrustStore};

// ============================================================================
// Audit Status
// ============================================================================

/// Plugin audit status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuditStatus {
    /// Awaiting review
    Pending,
    /// Review passed, plugin approved
    Approved,
    /// Review failed, plugin rejected
    Rejected,
    /// Previously approved plugin revoked
    Revoked,
}

// ============================================================================
// Audit Record
// ============================================================================

/// Plugin audit record
#[derive(Debug, Clone)]
pub struct AuditRecord {
    /// Plugin ID
    pub plugin_id: PluginId,
    /// Plugin name
    pub plugin_name: String,
    /// Plugin version string
    pub plugin_version: String,
    /// Current audit status
    pub status: AuditStatus,
    /// Reviewer identity
    pub reviewer: String,
    /// Review timestamp (epoch ms)
    pub timestamp: u64,
    /// Review notes / comments
    pub notes: String,
    /// Security findings
    pub findings: Vec<SecurityFinding>,
    /// Audit trail (previous status changes)
    pub history: Vec<AuditHistoryEntry>,
}

/// Security finding during audit
#[derive(Debug, Clone)]
pub struct SecurityFinding {
    /// Finding severity
    pub severity: FindingSeverity,
    /// Finding category
    pub category: FindingCategory,
    /// Finding description
    pub description: String,
    /// Whether finding is resolved
    pub resolved: bool,
}

/// Finding severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FindingSeverity {
    /// Informational
    Info,
    /// Low severity
    Low,
    /// Medium severity
    Medium,
    /// High severity
    High,
    /// Critical severity
    Critical,
}

/// Finding category
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FindingCategory {
    /// Memory safety issue
    MemorySafety,
    /// Permission issue
    Permission,
    /// Resource leak
    ResourceLeak,
    /// Data exposure
    DataExposure,
    /// Code quality
    CodeQuality,
    /// API misuse
    ApiMisuse,
    /// Other
    Other,
}

/// Audit history entry
#[derive(Debug, Clone)]
pub struct AuditHistoryEntry {
    /// Previous status
    pub from: AuditStatus,
    /// New status
    pub to: AuditStatus,
    /// Timestamp
    pub timestamp: u64,
    /// Who made the change
    pub actor: String,
    /// Reason for the change
    pub reason: String,
}

// ============================================================================
// Audit Manager
// ============================================================================

/// Plugin audit manager
pub struct AuditManager {
    /// Audit records (plugin_name -> AuditRecord)
    records: BTreeMap<String, AuditRecord>,
    /// Signing key (for release signing)
    signing_key: Option<SecretKey>,
    /// Signing public key
    signing_pk: Option<PublicKey>,
    /// Dilithium variant for signing
    variant: DilithiumVariant,
    /// Trust store
    trust_store: TrustStore,
    /// Whether signing key is initialized
    key_initialized: bool,
}

/// Audit error type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuditError {
    /// Plugin not found in audit records
    NotFound,
    /// Plugin already pending review
    AlreadyPending,
    /// Plugin already approved
    AlreadyApproved,
    /// Plugin already rejected
    AlreadyRejected,
    /// Plugin already revoked
    AlreadyRevoked,
    /// Signing key not initialized
    SigningKeyNotReady,
    /// Dilithium signing error
    SigningError,
    /// Invalid state transition
    InvalidTransition,
    /// Permission denied
    PermissionDenied,
}

impl AuditManager {
    /// Create a new audit manager
    pub fn new() -> Self {
        AuditManager {
            records: BTreeMap::new(),
            signing_key: None,
            signing_pk: None,
            variant: DilithiumVariant::Dilithium3,
            trust_store: TrustStore::new(SignaturePolicy::Enforced),
            key_initialized: false,
        }
    }

    /// Initialize signing keys
    /// Generates a new Dilithium key pair for release signing.
    pub fn init_signing_keys(&mut self) -> Result<(), AuditError> {
        let dilithium = Dilithium::new(self.variant);
        match dilithium.keygen() {
            Ok((pk, sk)) => {
                self.signing_pk = Some(pk);
                self.signing_key = Some(sk);
                self.key_initialized = true;
                Ok(())
            }
            Err(_) => Err(AuditError::SigningError),
        }
    }

    /// Set signing keys from existing keys
    pub fn set_signing_keys(&mut self, pk: PublicKey, sk: SecretKey) {
        self.signing_pk = Some(pk);
        self.signing_key = Some(sk);
        self.key_initialized = true;
    }

    /// Submit a plugin for audit
    /// Creates a new audit record with Pending status.
    /// @param plugin_id: Plugin ID
    /// @param name: Plugin name
    /// @param version: Plugin version string
    /// @param submitter: Who submitted the plugin
    pub fn submit_for_audit(
        &mut self,
        plugin_id: PluginId,
        name: &str,
        version: &str,
        submitter: &str,
    ) -> Result<(), AuditError> {
        if let Some(record) = self.records.get(name) {
            if record.status == AuditStatus::Pending {
                return Err(AuditError::AlreadyPending);
            }
        }

        let record = AuditRecord {
            plugin_id,
            plugin_name: String::from(name),
            plugin_version: String::from(version),
            status: AuditStatus::Pending,
            reviewer: String::new(),
            timestamp: current_timestamp(),
            notes: String::new(),
            findings: Vec::new(),
            history: Vec::new(),
        };

        self.records.insert(String::from(name), record);
        Ok(())
    }

    /// Approve a plugin after audit
    /// Transitions the plugin from Pending to Approved.
    /// @param name: Plugin name
    /// @param reviewer: Reviewer identity
    /// @param notes: Review notes
    /// @param findings: Security findings
    pub fn approve_plugin(
        &mut self,
        name: &str,
        reviewer: &str,
        notes: &str,
        findings: Vec<SecurityFinding>,
    ) -> Result<(), AuditError> {
        let record = self.records.get_mut(name)
            .ok_or(AuditError::NotFound)?;

        if record.status != AuditStatus::Pending {
            return Err(AuditError::InvalidTransition);
        }

        let has_critical = findings.iter().any(|f| f.severity == FindingSeverity::Critical);
        if has_critical {
            return Err(AuditError::InvalidTransition);
        }

        let old_status = record.status;
        record.status = AuditStatus::Approved;
        record.reviewer = String::from(reviewer);
        record.timestamp = current_timestamp();
        record.notes = String::from(notes);
        record.findings = findings;

        record.history.push(AuditHistoryEntry {
            from: old_status,
            to: AuditStatus::Approved,
            timestamp: record.timestamp,
            actor: String::from(reviewer),
            reason: String::from(notes),
        });

        Ok(())
    }

    /// Reject a plugin after audit
    /// Transitions the plugin from Pending to Rejected.
    /// @param name: Plugin name
    /// @param reviewer: Reviewer identity
    /// @param reason: Rejection reason
    pub fn reject_plugin(
        &mut self,
        name: &str,
        reviewer: &str,
        reason: &str,
    ) -> Result<(), AuditError> {
        let record = self.records.get_mut(name)
            .ok_or(AuditError::NotFound)?;

        if record.status != AuditStatus::Pending {
            return Err(AuditError::InvalidTransition);
        }

        let old_status = record.status;
        record.status = AuditStatus::Rejected;
        record.reviewer = String::from(reviewer);
        record.timestamp = current_timestamp();
        record.notes = String::from(reason);

        record.history.push(AuditHistoryEntry {
            from: old_status,
            to: AuditStatus::Rejected,
            timestamp: record.timestamp,
            actor: String::from(reviewer),
            reason: String::from(reason),
        });

        Ok(())
    }

    /// Revoke a previously approved plugin
    /// Transitions from Approved to Revoked.
    /// @param name: Plugin name
    /// @param actor: Who revoked the plugin
    /// @param reason: Revocation reason
    pub fn revoke_plugin(
        &mut self,
        name: &str,
        actor: &str,
        reason: &str,
    ) -> Result<(), AuditError> {
        let record = self.records.get_mut(name)
            .ok_or(AuditError::NotFound)?;

        if record.status != AuditStatus::Approved {
            return Err(AuditError::InvalidTransition);
        }

        let old_status = record.status;
        record.status = AuditStatus::Revoked;
        record.timestamp = current_timestamp();
        record.notes = String::from(reason);

        record.history.push(AuditHistoryEntry {
            from: old_status,
            to: AuditStatus::Revoked,
            timestamp: record.timestamp,
            actor: String::from(actor),
            reason: String::from(reason),
        });

        Ok(())
    }

    /// Sign a release version with Dilithium
    /// Signs the plugin binary with the audit manager's signing key.
    /// The plugin must be in Approved status.
    /// @param name: Plugin name
    /// @param plugin_data: Raw plugin binary data
    pub fn sign_release(
        &mut self,
        name: &str,
        plugin_data: &[u8],
    ) -> Result<PluginSignature, AuditError> {
        if !self.key_initialized {
            return Err(AuditError::SigningKeyNotReady);
        }

        let record = self.records.get(name)
            .ok_or(AuditError::NotFound)?;

        if record.status != AuditStatus::Approved {
            return Err(AuditError::InvalidTransition);
        }

        let sk = self.signing_key.as_ref()
            .ok_or(AuditError::SigningKeyNotReady)?;

        let dilithium = Dilithium::new(self.variant);

        let sig = match dilithium.sign(sk, plugin_data) {
            Ok(s) => s,
            Err(_) => return Err(AuditError::SigningError),
        };

        let fingerprint = super::signature::compute_plugin_hash(plugin_data);

        Ok(PluginSignature::from_dilithium(&sig, fingerprint))
    }

    /// Get audit record for a plugin
    pub fn get_record(&self, name: &str) -> Option<&AuditRecord> {
        self.records.get(name)
    }

    /// Get audit status for a plugin
    pub fn get_status(&self, name: &str) -> Option<AuditStatus> {
        self.records.get(name).map(|r| r.status)
    }

    /// List all plugins with a given status
    pub fn list_by_status(&self, status: AuditStatus) -> Vec<&AuditRecord> {
        self.records.values()
            .filter(|r| r.status == status)
            .collect()
    }

    /// Check if a plugin is approved
    pub fn is_approved(&self, name: &str) -> bool {
        self.records.get(name)
            .map(|r| r.status == AuditStatus::Approved)
            .unwrap_or(false)
    }
}

impl Default for AuditManager {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Utility
// ============================================================================

/// Get current timestamp from the system timer.
/// Reads the architectural cycle counter and converts to milliseconds.
fn current_timestamp() -> u64 {
    crate::hal::cpu::read_cycle_counter() / 1000
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_submit() {
        let mut mgr = AuditManager::new();
        let result = mgr.submit_for_audit(
            PluginId(1), "test_plugin", "1.0.0", "submitter"
        );
        assert!(result.is_ok());
        assert_eq!(mgr.get_status("test_plugin"), Some(AuditStatus::Pending));
    }

    #[test]
    fn test_audit_approve() {
        let mut mgr = AuditManager::new();
        mgr.submit_for_audit(PluginId(1), "test_plugin", "1.0.0", "submitter").ok();
        let result = mgr.approve_plugin("test_plugin", "reviewer", "looks good", Vec::new());
        assert!(result.is_ok());
        assert_eq!(mgr.get_status("test_plugin"), Some(AuditStatus::Approved));
    }

    #[test]
    fn test_audit_reject() {
        let mut mgr = AuditManager::new();
        mgr.submit_for_audit(PluginId(1), "bad_plugin", "1.0.0", "submitter").ok();
        let result = mgr.reject_plugin("bad_plugin", "reviewer", "security issues");
        assert!(result.is_ok());
        assert_eq!(mgr.get_status("bad_plugin"), Some(AuditStatus::Rejected));
    }

    #[test]
    fn test_audit_revoke() {
        let mut mgr = AuditManager::new();
        mgr.submit_for_audit(PluginId(1), "test_plugin", "1.0.0", "submitter").ok();
        mgr.approve_plugin("test_plugin", "reviewer", "ok", Vec::new()).ok();
        let result = mgr.revoke_plugin("test_plugin", "admin", "vulnerability found");
        assert!(result.is_ok());
        assert_eq!(mgr.get_status("test_plugin"), Some(AuditStatus::Revoked));
    }

    #[test]
    fn test_audit_invalid_transition() {
        let mut mgr = AuditManager::new();
        mgr.submit_for_audit(PluginId(1), "test_plugin", "1.0.0", "submitter").ok();
        let result = mgr.revoke_plugin("test_plugin", "admin", "not approved yet");
        assert!(result.is_err());
    }

    #[test]
    fn test_finding_severity_ordering() {
        assert!(FindingSeverity::Info < FindingSeverity::Low);
        assert!(FindingSeverity::Low < FindingSeverity::Medium);
        assert!(FindingSeverity::Medium < FindingSeverity::High);
        assert!(FindingSeverity::High < FindingSeverity::Critical);
    }
}
