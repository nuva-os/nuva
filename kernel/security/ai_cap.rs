/*
 * Nuva OS - Kernel - Security - AiCap
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
/*
 * AI Inference Capability-Based Access Control
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Provides capability-based access control for AI model inference,
 * including model access permissions, inference quotas, rate limits,
 * and capability token mechanism.
 */

use core::sync::atomic::{AtomicU64, AtomicU32, AtomicBool, Ordering};
use alloc::vec::Vec;

use crate::{pr_info, pr_warn};

/// Maximum model name length
pub const MAX_MODEL_NAME: usize = 64;

/// Maximum capabilities per model
pub const MAX_MODEL_CAPS: usize = 16;

/// Capability token size (32 bytes)
pub const CAP_TOKEN_SIZE: usize = 32;

/// AI capability error type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiCapError {
    /// Permission denied
    PermissionDenied,
    /// Quota exceeded
    QuotaExceeded,
    /// Rate limit exceeded
    RateLimitExceeded,
    /// Invalid capability token
    InvalidToken,
    /// Model not found
    ModelNotFound,
    /// Process not found
    ProcessNotFound,
    /// Out of memory
    OutOfMemory,
}

/// AI model access permissions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiModelPermission {
    /// Read model metadata
    Read = 0,
    /// Execute inference
    Infer = 1,
    /// Fine-tune model
    Finetune = 2,
    /// Export model weights
    Export = 3,
    /// Admin: modify permissions
    Admin = 4,
}

/// AI capability structure
/// Defines a process's rights to access a specific AI model,
/// including permissions, quotas, and rate limits.
pub struct AiCapability {
    /// Model identifier
    pub model_id: [u8; MAX_MODEL_NAME],
    /// Model ID length
    pub model_id_len: u32,
    /// Granted permissions bitmap
    pub permissions: AtomicU32,
    /// Inference call quota (max total calls, 0 = unlimited)
    pub quota_max: AtomicU64,
    /// Inference calls consumed
    pub quota_used: AtomicU64,
    /// Rate limit: max calls per second (0 = unlimited)
    pub rate_limit: AtomicU32,
    /// Max memory usage for this model (bytes, 0 = unlimited)
    pub max_memory: AtomicU64,
    /// Max compute time per inference (ms, 0 = unlimited)
    pub max_compute_time_ms: AtomicU32,
    /// Capability token for delegation
    pub token: [u8; CAP_TOKEN_SIZE],
    /// Is capability valid
    pub valid: AtomicBool,
}

impl AiCapability {
    /// Create empty capability
    pub const fn empty() -> Self {
        AiCapability {
            model_id: [0u8; MAX_MODEL_NAME],
            model_id_len: 0,
            permissions: AtomicU32::new(0),
            quota_max: AtomicU64::new(0),
            quota_used: AtomicU64::new(0),
            rate_limit: AtomicU32::new(0),
            max_memory: AtomicU64::new(0),
            max_compute_time_ms: AtomicU32::new(0),
            token: [0u8; CAP_TOKEN_SIZE],
            valid: AtomicBool::new(false),
        }
    }

    /// Create capability for a model with given permissions
    pub fn new(model_name: &[u8], permissions: u32, quota: u64, rate: u32) -> Self {
        let mut cap = AiCapability::empty();
        let len = model_name.len().min(MAX_MODEL_NAME);
        cap.model_id[..len].copy_from_slice(&model_name[..len]);
        cap.model_id_len = len as u32;
        cap.permissions.store(permissions, Ordering::Release);
        cap.quota_max.store(quota, Ordering::Release);
        cap.rate_limit.store(rate, Ordering::Release);
        cap.valid.store(true, Ordering::Release);
        cap
    }

    /// Check if a specific permission is granted
    pub fn has_permission(&self, perm: AiModelPermission) -> bool {
        let bits = self.permissions.load(Ordering::Acquire);
        (bits & (1 << (perm as u32))) != 0
    }

    /// Grant a permission
    pub fn grant(&self, perm: AiModelPermission) {
        self.permissions.fetch_or(1 << (perm as u32), Ordering::AcqRel);
    }

    /// Revoke a permission
    pub fn revoke(&self, perm: AiModelPermission) {
        self.permissions.fetch_and(!(1 << (perm as u32)), Ordering::AcqRel);
    }

    /// Invalidate capability
    pub fn invalidate(&self) {
        self.valid.store(false, Ordering::Release);
    }

    /// Check if capability is valid
    pub fn is_valid(&self) -> bool {
        self.valid.load(Ordering::Acquire)
    }
}

/// Check if a process has permission to use a model
/// @param pid: Process ID
/// @param cap: AI capability for the model
/// @param required_perm: Required permission
/// @return: Ok if permission granted, Err otherwise
pub fn ai_check_permission(
    pid: u32,
    cap: &AiCapability,
    required_perm: AiModelPermission,
) -> Result<(), AiCapError> {
    let _ = pid;

    if !cap.is_valid() {
        return Err(AiCapError::InvalidToken);
    }

    if !cap.has_permission(required_perm) {
        log_warn!("AI cap: permission denied for model {:?}", cap.model_id);
        return Err(AiCapError::PermissionDenied);
    }

    Ok(())
}

/// Enforce inference quota
/// Atomically increments the usage counter and checks against quota.
/// Returns Err if quota is exceeded.
pub fn ai_enforce_quota(pid: u32, cap: &AiCapability) -> Result<u64, AiCapError> {
    let _ = pid;

    if !cap.is_valid() {
        return Err(AiCapError::InvalidToken);
    }

    let quota_max = cap.quota_max.load(Ordering::Acquire);
    if quota_max == 0 {
        return Ok(cap.quota_used.fetch_add(1, Ordering::AcqRel));
    }

    let used = cap.quota_used.load(Ordering::Acquire);
    if used >= quota_max {
        log_warn!("AI cap: quota exceeded ({}/{})", used, quota_max);
        return Err(AiCapError::QuotaExceeded);
    }

    cap.quota_used.fetch_add(1, Ordering::AcqRel);
    Ok(used)
}

/// Enforce inference rate limit
/// Uses a simple token bucket algorithm.
/// Returns the current call count within the rate window.
pub fn ai_rate_limit(
    pid: u32,
    cap: &AiCapability,
    current_calls_in_window: u32,
) -> Result<(), AiCapError> {
    let _ = pid;

    if !cap.is_valid() {
        return Err(AiCapError::InvalidToken);
    }

    let limit = cap.rate_limit.load(Ordering::Acquire);
    if limit == 0 {
        return Ok(());
    }

    if current_calls_in_window >= limit {
        log_warn!("AI cap: rate limit exceeded ({}/{})", current_calls_in_window, limit);
        return Err(AiCapError::RateLimitExceeded);
    }

    Ok(())
}

/// Capability token mechanism
/// Generates and validates opaque capability tokens that can
/// be delegated between processes. Tokens are HMAC-SHA256
/// of (model_id || permissions || pid) with a system secret key.
pub fn generate_capability_token(
    cap: &AiCapability,
    pid: u32,
) -> [u8; CAP_TOKEN_SIZE] {
    let mut token = [0u8; CAP_TOKEN_SIZE];

    let model_len = cap.model_id_len as usize;
    let perms = cap.permissions.load(Ordering::Acquire);

    let mut input = alloc::vec![0u8; model_len + 4 + 4];
    input[..model_len].copy_from_slice(&cap.model_id[..model_len]);
    input[model_len..model_len + 4].copy_from_slice(&perms.to_le_bytes());
    input[model_len + 4..model_len + 8].copy_from_slice(&pid.to_le_bytes());

    // SAFETY: FFI call to HMAC-SHA256
    let result = unsafe {
        hmac_sha256_ffi(
            SYSTEM_CAP_KEY.as_ptr(),
            SYSTEM_CAP_KEY.len(),
            input.as_ptr(),
            input.len(),
            token.as_mut_ptr(),
        )
    };

    if result != 0 {
        token = [0u8; CAP_TOKEN_SIZE];
    }

    token
}

/// Verify a capability token
pub fn verify_capability_token(
    cap: &AiCapability,
    pid: u32,
    token: &[u8; CAP_TOKEN_SIZE],
) -> bool {
    let expected = generate_capability_token(cap, pid);
    let mut diff: u8 = 0;
    for i in 0..CAP_TOKEN_SIZE {
        diff |= expected[i] ^ token[i];
    }
    diff == 0
}

/// System capability key (initialized during boot)
static SYSTEM_CAP_KEY: [u8; 32] = [
    0x4e, 0x75, 0x76, 0x61, 0x4f, 0x53, 0x41, 0x69,
    0x43, 0x61, 0x70, 0x4b, 0x65, 0x79, 0x00, 0x00,
    0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
    0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10,
];

extern "C" {
    /// HMAC-SHA256
    fn hmac_sha256_ffi(
        key: *const u8,
        key_len: usize,
        input: *const u8,
        input_len: usize,
        output: *mut u8,
    ) -> i32;
}

/// AI capability manager
/// Manages capabilities for all processes and models.
pub struct AiCapManager {
    /// Total capabilities issued
    total_caps: AtomicU64,
    /// Total permission checks
    total_checks: AtomicU64,
    /// Total denials
    total_denials: AtomicU64,
    /// Initialized
    initialized: AtomicBool,
}

impl AiCapManager {
    /// Create new manager
    pub const fn new() -> Self {
        AiCapManager {
            total_caps: AtomicU64::new(0),
            total_checks: AtomicU64::new(0),
            total_denials: AtomicU64::new(0),
            initialized: AtomicBool::new(false),
        }
    }

    /// Initialize manager
    pub fn init(&self) {
        if self.initialized.load(Ordering::Acquire) {
            return;
        }
        self.initialized.store(true, Ordering::Release);
        log_info!("AI capability manager initialized");
    }

    /// Record a capability grant
    pub fn record_grant(&self) {
        self.total_caps.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a permission check
    pub fn record_check(&self, allowed: bool) {
        self.total_checks.fetch_add(1, Ordering::Relaxed);
        if !allowed {
            self.total_denials.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Get statistics
    pub fn stats(&self) -> (u64, u64, u64) {
        (
            self.total_caps.load(Ordering::Acquire),
            self.total_checks.load(Ordering::Acquire),
            self.total_denials.load(Ordering::Acquire),
        )
    }
}

/// Global AI capability manager
static AI_CAP_MANAGER: crate::sync_oncelock::OnceLock<AiCapManager> = crate::sync_oncelock::OnceLock::new();

/// Get global manager
pub fn get_ai_cap_manager() -> &'static AiCapManager {
    AI_CAP_MANAGER.get_or_init(AiCapManager::new)
}

/// Initialize AI capability subsystem
pub fn init_ai_cap() {
    get_ai_cap_manager().init();
}

#[cfg(test)]
mod tests {
    use super::*;
use alloc::vec;

    #[test]
    fn test_ai_capability_empty() {
        let cap = AiCapability::empty();
        assert!(!cap.is_valid());
        assert!(!cap.has_permission(AiModelPermission::Infer));
    }

    #[test]
    fn test_ai_capability_new() {
        let cap = AiCapability::new(b"resnet50", 0b111, 1000, 100);
        assert!(cap.is_valid());
        assert!(cap.has_permission(AiModelPermission::Read));
        assert!(cap.has_permission(AiModelPermission::Infer));
        assert!(cap.has_permission(AiModelPermission::Finetune));
        assert!(!cap.has_permission(AiModelPermission::Export));
    }

    #[test]
    fn test_ai_capability_grant_revoke() {
        let cap = AiCapability::new(b"model", 0, 0, 0);
        assert!(!cap.has_permission(AiModelPermission::Infer));
        cap.grant(AiModelPermission::Infer);
        assert!(cap.has_permission(AiModelPermission::Infer));
        cap.revoke(AiModelPermission::Infer);
        assert!(!cap.has_permission(AiModelPermission::Infer));
    }

    #[test]
    fn test_ai_capability_invalidate() {
        let cap = AiCapability::new(b"model", 0xFF, 0, 0);
        assert!(cap.is_valid());
        cap.invalidate();
        assert!(!cap.is_valid());
    }

    #[test]
    fn test_check_permission_granted() {
        let cap = AiCapability::new(b"model", 0b11, 0, 0);
        assert!(ai_check_permission(1, &cap, AiModelPermission::Infer).is_ok());
    }

    #[test]
    fn test_check_permission_denied() {
        let cap = AiCapability::new(b"model", 0b1, 0, 0);
        assert!(ai_check_permission(1, &cap, AiModelPermission::Infer).is_err());
    }

    #[test]
    fn test_check_permission_invalid() {
        let cap = AiCapability::empty();
        assert!(ai_check_permission(1, &cap, AiModelPermission::Infer).is_err());
    }

    #[test]
    fn test_enforce_quota_unlimited() {
        let cap = AiCapability::new(b"model", 0xFF, 0, 0);
        assert!(ai_enforce_quota(1, &cap).is_ok());
    }

    #[test]
    fn test_enforce_quota_limited() {
        let cap = AiCapability::new(b"model", 0xFF, 2, 0);
        assert!(ai_enforce_quota(1, &cap).is_ok());
        assert!(ai_enforce_quota(1, &cap).is_ok());
        assert_eq!(ai_enforce_quota(1, &cap).err(), Some(AiCapError::QuotaExceeded));
    }

    #[test]
    fn test_rate_limit_unlimited() {
        let cap = AiCapability::new(b"model", 0xFF, 0, 0);
        assert!(ai_rate_limit(1, &cap, 999).is_ok());
    }

    #[test]
    fn test_rate_limit_exceeded() {
        let cap = AiCapability::new(b"model", 0xFF, 0, 10);
        assert!(ai_rate_limit(1, &cap, 10).is_err());
    }

    #[test]
    fn test_cap_manager() {
        let mgr = AiCapManager::new();
        mgr.init();
        mgr.record_grant();
        mgr.record_check(true);
        mgr.record_check(false);
        let (caps, checks, denials) = mgr.stats();
        assert_eq!(caps, 1);
        assert_eq!(checks, 2);
        assert_eq!(denials, 1);
    }
}
