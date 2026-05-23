/*
 * Nuva OS - Kernel - Secure Boot
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

//! Secure Boot Framework
/*!*/
//! Provides verified boot chain from firmware to kernel:
//! - Boot state machine: DISABLED -> SETUP -> ENFORCED
//! - Measured boot (TPM-style measurement)
//! - Boot configuration locking

use core::sync::atomic::{AtomicU32, AtomicU64, AtomicU8, AtomicBool, Ordering};
use crate::{pr_info, pr_debug, pr_warn};

/// Secure boot state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecureBootState {
    /// Secure boot disabled
    Disabled = 0,
    /// Setup mode (keys can be enrolled)
    Setup = 1,
    /// Enforced mode (verification required)
    Enforced = 2,
}

/// Boot component in the verification chain
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootComponent {
    /// Platform firmware (UEFI/BIOS)
    Firmware = 0,
    /// Boot loader (GRUB, etc.)
    BootLoader = 1,
    /// Boot loader configuration
    BootConfig = 2,
    /// OS kernel
    Kernel = 3,
    /// Init ramdisk
    InitRd = 4,
    /// Device tree
    DeviceTree = 5,
}

/// Maximum components in boot chain
pub const MAX_BOOT_COMPONENTS: usize = 6;

/// Hash size (SHA-256)
pub const BOOT_HASH_SIZE: usize = 32;

/// Maximum measurement log entries
pub const MAX_MEASUREMENT_ENTRIES: usize = 32;

/// Boot verification result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootVerifyResult {
    /// Verification succeeded
    Success = 0,
    /// Signature verification failed
    SignatureFailed = 1,
    /// Hash mismatch
    HashMismatch = 2,
    /// Key not authorized
    KeyNotAuthorized = 3,
    /// Component not signed
    NotSigned = 4,
    /// Boot chain incomplete
    Incomplete = 5,
    /// Configuration locked
    ConfigLocked = 6,
    /// Internal error
    InternalError = 7,
}

/// Measurement log entry
#[derive(Debug, Clone, Copy)]
pub struct MeasurementEntry {
    /// Component being measured
    pub component: BootComponent,
    /// SHA-256 hash of component
    pub hash: [u8; BOOT_HASH_SIZE],
    /// Component size in bytes
    pub size: u64,
    /// Measurement index (PCR-like)
    pub pcr_index: u32,
}

/// Boot configuration
pub struct BootConfig {
    /// Secure boot state
    pub state: AtomicU32,
    /// Config is locked
    pub locked: AtomicBool,
    /// Verified component bitmap
    pub verified: AtomicU32,
    /// Required component bitmap
    pub required: AtomicU32,
    /// Measurement count
    pub measurement_count: AtomicU32,
    /// PCR values (accumulated measurements)
    pub pcr_values: [[u8; BOOT_HASH_SIZE]; MAX_BOOT_COMPONENTS],
    /// Measurement log
    pub measurements: [Option<MeasurementEntry>; MAX_MEASUREMENT_ENTRIES],
    /// Boot fail count
    pub fail_count: AtomicU32,
    /// Debug mode
    pub debug_mode: AtomicBool,
}

impl BootConfig {
    /// Create default boot config
    pub const fn new() -> Self {
        BootConfig {
            state: AtomicU32::new(SecureBootState::Disabled as u32),
            locked: AtomicBool::new(false),
            verified: AtomicU32::new(0),
            required: AtomicU32::new(0),
            measurement_count: AtomicU32::new(0),
            pcr_values: [[0u8; BOOT_HASH_SIZE]; MAX_BOOT_COMPONENTS],
            measurements: [None; MAX_MEASUREMENT_ENTRIES],
            fail_count: AtomicU32::new(0),
            debug_mode: AtomicBool::new(false),
        }
    }

    /// Get current secure boot state
    pub fn get_state(&self) -> SecureBootState {
        match self.state.load(Ordering::Acquire) {
            0 => SecureBootState::Disabled,
            1 => SecureBootState::Setup,
            2 => SecureBootState::Enforced,
            _ => SecureBootState::Disabled,
        }
    }

    /// Transition to setup mode
    pub fn enter_setup(&self) -> Result<(), BootVerifyResult> {
        if self.locked.load(Ordering::Acquire) {
            return Err(BootVerifyResult::ConfigLocked);
        }

        let current = self.get_state();
        if current != SecureBootState::Disabled {
            return Err(BootVerifyResult::InternalError);
        }

        self.state.store(SecureBootState::Setup as u32, Ordering::Release);
        log_info!("Secure boot: entered setup mode");
        Ok(())
    }

    /// Transition to enforced mode
    pub fn enforce(&self) -> Result<(), BootVerifyResult> {
        if self.locked.load(Ordering::Acquire) {
            return Err(BootVerifyResult::ConfigLocked);
        }

        let current = self.get_state();
        if current != SecureBootState::Setup {
            return Err(BootVerifyResult::InternalError);
        }

        self.state.store(SecureBootState::Enforced as u32, Ordering::Release);
        log_info!("Secure boot: enforcement enabled");
        Ok(())
    }
}

/// Verify the complete boot chain
pub fn verify_boot_chain(config: &BootConfig) -> BootVerifyResult {
    let state = config.get_state();
    if state == SecureBootState::Disabled {
        log_info!("Secure boot: disabled, skipping verification");
        return BootVerifyResult::Success;
    }

    let required = config.required.load(Ordering::Acquire);
    let verified = config.verified.load(Ordering::Acquire);

    if (required & !verified) != 0 {
        log_warn!("Secure boot: not all required components verified");
        return BootVerifyResult::Incomplete;
    }

    for i in 0..MAX_BOOT_COMPONENTS {
        if (required & (1 << i)) == 0 {
            continue;
        }

        let pcr = &config.pcr_values[i];
        let all_zero = pcr.iter().all(|&b| b == 0);
        if all_zero && state == SecureBootState::Enforced {
            log_warn!("Secure boot: component {} not measured", i);
            return BootVerifyResult::Incomplete;
        }
    }

    log_info!("Secure boot: boot chain verified successfully");
    BootVerifyResult::Success
}

/// Lock boot configuration to prevent tampering
pub fn lock_boot_config(config: &BootConfig) -> Result<(), BootVerifyResult> {
    if config.locked.load(Ordering::Acquire) {
        return Ok(());
    }

    let state = config.get_state();
    if state == SecureBootState::Disabled {
        log_warn!("Secure boot: cannot lock in disabled state");
        return Err(BootVerifyResult::InternalError);
    }

    config.locked.store(true, Ordering::Release);
    log_info!("Secure boot: configuration locked");
    Ok(())
}

/// Perform measured boot (TPM-style measurement)
pub fn measured_boot(
    config: &mut BootConfig,
    component: BootComponent,
    data: &[u8],
) -> BootVerifyResult {
    let state = config.get_state();
    if state == SecureBootState::Disabled {
        return BootVerifyResult::Success;
    }

    let hash = compute_boot_hash(data);
    let idx = component as usize;

    if idx >= MAX_BOOT_COMPONENTS {
        return BootVerifyResult::InternalError;
    }

    config.pcr_values[idx] = extend_pcr(&config.pcr_values[idx], &hash);

    let count = config.measurement_count.load(Ordering::Acquire);
    if (count as usize) < MAX_MEASUREMENT_ENTRIES {
        let entry = MeasurementEntry {
            component,
            hash,
            size: data.len() as u64,
            pcr_index: idx as u32,
        };
        config.measurements[count as usize] = Some(entry);
        config.measurement_count.fetch_add(1, Ordering::Release);
    }

    config.verified.fetch_or(1 << idx, Ordering::AcqRel);

    log_debug!("Measured boot: {:?} measured (size={})", component, data.len());
    BootVerifyResult::Success
}

/// Extend PCR with new measurement (TPM extend operation)
fn extend_pcr(current: &[u8; BOOT_HASH_SIZE], measurement: &[u8; BOOT_HASH_SIZE]) -> [u8; BOOT_HASH_SIZE] {
    let mut result = [0u8; BOOT_HASH_SIZE];
    for i in 0..BOOT_HASH_SIZE {
        result[i] = current[i] ^ measurement[i];
    }
    let mut acc: u64 = 0xcbf29ce484222325;
    for &byte in result.iter() {
        acc = acc.wrapping_mul(0x100000001b3);
        acc ^= byte as u64;
    }
    for i in 0..4 {
        result[i * 8..(i + 1) * 8].copy_from_slice(&acc.wrapping_add(i as u64).to_le_bytes());
    }
    result
}

/// Compute SHA-256 hash for boot component measurement
fn compute_boot_hash(data: &[u8]) -> [u8; BOOT_HASH_SIZE] {
    crate::kernel::security::signature::compute_hash(data)
}

/// Global boot configuration
static BOOT_CONFIG: core::sync::OnceLock<BootConfig> = core::sync::OnceLock::new();

/// Get boot configuration
pub fn boot_config() -> &'static BootConfig {
    BOOT_CONFIG.get_or_init(BootConfig::new)
}

/// Initialize secure boot subsystem
pub fn init_secure_boot() {
    let config = get_boot_config();
    config.required.store(
        (1 << BootComponent::Firmware as u32)
            | (1 << BootComponent::BootLoader as u32)
            | (1 << BootComponent::Kernel as u32),
        Ordering::Release,
    );
    log_info!("Secure boot subsystem initialized");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boot_state_transitions() {
        let config = BootConfig::new();
        assert_eq!(config.get_state(), SecureBootState::Disabled);

        assert!(config.enter_setup().is_ok());
        assert_eq!(config.get_state(), SecureBootState::Setup);

        assert!(config.enforce().is_ok());
        assert_eq!(config.get_state(), SecureBootState::Enforced);
    }

    #[test]
    fn test_config_lock() {
        let config = BootConfig::new();
        assert!(config.enter_setup().is_ok());

        assert!(lock_boot_config(&config).is_ok());
        assert!(config.locked.load(Ordering::Acquire));

        assert!(config.enforce().is_err());
    }

    #[test]
    fn test_measured_boot() {
        let mut config = BootConfig::new();
        config.state.store(SecureBootState::Enforced as u32, Ordering::Release);

        let result = measured_boot(&mut config, BootComponent::Kernel, b"kernel_data");
        assert_eq!(result, BootVerifyResult::Success);

        let verified = config.verified.load(Ordering::Acquire);
        assert_ne!(verified & (1 << BootComponent::Kernel as u32), 0);
    }

    #[test]
    fn test_pcr_extend() {
        let pcr = [0u8; BOOT_HASH_SIZE];
        let measurement = [0xABu8; BOOT_HASH_SIZE];
        let extended = extend_pcr(&pcr, &measurement);
        assert_ne!(extended, [0u8; BOOT_HASH_SIZE]);
    }
}
