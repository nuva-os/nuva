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
//! Provides verified boot chain, measured boot, and attestation.

use core::sync::atomic::{AtomicU32, AtomicU64, AtomicU8, AtomicBool, Ordering};
use alloc::vec::Vec;
use crate::{pr_info, pr_debug, pr_warn};

use super::tpm_abi::{TpmAbi, TpmProviderType, TpmError, TpmResult, PcrIndex, PCR_DIGEST_SIZE};
use super::tpm_hw::HardwareTpm;
use super::tpm_ftpm::FirmwareTpm;
use super::measurement::{MeasurementEngine, MeasurementState, BootComponent as MeasBootComponent};
use super::aik::{AikManager, AikState};
use super::quote::{QuoteEngine, QuoteToken, AttestationResponse, NONCE_SIZE};
use super::event_log::EventLogManager;
use super::sha256::sha256_digest;

/// Capability: boot attestation
pub const CAP_BOOT_ATTEST: u32 = 100;
/// Capability: measurement read
pub const CAP_MEASUREMENT_READ: u32 = 101;
/// Capability: AIK administration
pub const CAP_AIK_ADMIN: u32 = 102;

/// Secure boot state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecureBootState {
    Disabled = 0,
    Setup = 1,
    Enforced = 2,
}

/// Boot component in the verification chain
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootComponent {
    Firmware = 0,
    BootLoader = 1,
    BootConfig = 2,
    Kernel = 3,
    InitRd = 4,
    DeviceTree = 5,
}

pub const MAX_BOOT_COMPONENTS: usize = 6;
pub const BOOT_HASH_SIZE: usize = 32;
pub const MAX_MEASUREMENT_ENTRIES: usize = 32;

/// Boot verification result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootVerifyResult {
    Success = 0,
    SignatureFailed = 1,
    HashMismatch = 2,
    KeyNotAuthorized = 3,
    NotSigned = 4,
    Incomplete = 5,
    ConfigLocked = 6,
    InternalError = 7,
    CapabilityDenied = 8,
}

/// Measurement log entry
#[derive(Debug, Clone, Copy)]
pub struct MeasurementEntry {
    pub component: BootComponent,
    pub hash: [u8; BOOT_HASH_SIZE],
    pub size: u64,
    pub pcr_index: u32,
}

/// Boot configuration
pub struct BootConfig {
    pub state: AtomicU32,
    pub locked: AtomicBool,
    pub verified: AtomicU32,
    pub required: AtomicU32,
    pub measurement_count: AtomicU32,
    pub pcr_values: [[u8; BOOT_HASH_SIZE]; MAX_BOOT_COMPONENTS],
    pub measurements: [Option<MeasurementEntry>; MAX_MEASUREMENT_ENTRIES],
    pub fail_count: AtomicU32,
    pub debug_mode: AtomicBool,
}

impl BootConfig {
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
    pub fn get_state(&self) -> SecureBootState {
        match self.state.load(Ordering::Acquire) {
            0 => SecureBootState::Disabled, 1 => SecureBootState::Setup, 2 => SecureBootState::Enforced, _ => SecureBootState::Disabled,
        }
    }
    pub fn enter_setup(&self) -> Result<(), BootVerifyResult> {
        if self.locked.load(Ordering::Acquire) { return Err(BootVerifyResult::ConfigLocked); }
        if self.get_state() != SecureBootState::Disabled { return Err(BootVerifyResult::InternalError); }
        self.state.store(SecureBootState::Setup as u32, Ordering::Release);
        Ok(())
    }
    pub fn enforce(&self) -> Result<(), BootVerifyResult> {
        if self.locked.load(Ordering::Acquire) { return Err(BootVerifyResult::ConfigLocked); }
        if self.get_state() != SecureBootState::Setup { return Err(BootVerifyResult::InternalError); }
        self.state.store(SecureBootState::Enforced as u32, Ordering::Release);
        Ok(())
    }
}

pub fn verify_boot_chain(config: &BootConfig) -> BootVerifyResult {
    let state = config.get_state();
    if state == SecureBootState::Disabled { return BootVerifyResult::Success; }
    let required = config.required.load(Ordering::Acquire);
    let verified = config.verified.load(Ordering::Acquire);
    if (required & !verified) != 0 { return BootVerifyResult::Incomplete; }
    for i in 0..MAX_BOOT_COMPONENTS {
        if (required & (1 << i)) == 0 { continue; }
        if config.pcr_values[i].iter().all(|&b| b == 0) && state == SecureBootState::Enforced { return BootVerifyResult::Incomplete; }
    }
    BootVerifyResult::Success
}

pub fn lock_boot_config(config: &BootConfig) -> Result<(), BootVerifyResult> {
    if config.locked.load(Ordering::Acquire) { return Ok(()); }
    if config.get_state() == SecureBootState::Disabled { return Err(BootVerifyResult::InternalError); }
    config.locked.store(true, Ordering::Release);
    Ok(())
}

pub fn measured_boot(config: &mut BootConfig, component: BootComponent, data: &[u8]) -> BootVerifyResult {
    if config.get_state() == SecureBootState::Disabled { return BootVerifyResult::Success; }
    let hash = compute_boot_hash(data);
    let idx = component as usize;
    if idx >= MAX_BOOT_COMPONENTS { return BootVerifyResult::InternalError; }
    config.pcr_values[idx] = extend_pcr(&config.pcr_values[idx], &hash);
    let count = config.measurement_count.load(Ordering::Acquire);
    if (count as usize) < MAX_MEASUREMENT_ENTRIES {
        config.measurements[count as usize] = Some(MeasurementEntry { component, hash, size: data.len() as u64, pcr_index: idx as u32 });
        config.measurement_count.fetch_add(1, Ordering::Release);
    }
    config.verified.fetch_or(1 << idx, Ordering::AcqRel);
    BootVerifyResult::Success
}

fn extend_pcr(current: &[u8; BOOT_HASH_SIZE], measurement: &[u8; BOOT_HASH_SIZE]) -> [u8; BOOT_HASH_SIZE] {
    // TPM-standard PCR extend: SHA256(PCR_old || measurement)
    let mut concat = [0u8; BOOT_HASH_SIZE * 2];
    concat[..BOOT_HASH_SIZE].copy_from_slice(current);
    concat[BOOT_HASH_SIZE..].copy_from_slice(measurement);
    compute_boot_hash(&concat)
}

fn compute_boot_hash(data: &[u8]) -> [u8; BOOT_HASH_SIZE] {
    crate::kernel::security::signature::compute_hash(data)
}

/// TPM provider selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TpmSelection {
    None,
    Hardware,
    Firmware,
}

/// Boot Attestation Manager
/// Selects TPM provider (hardware first, firmware fallback).
pub struct BootAttestationManager {
    tpm_selection: TpmSelection,
    hw_tpm: HardwareTpm,
    fw_tpm: FirmwareTpm,
    aik: AikManager,
    quote_engine: QuoteEngine,
    capabilities: AtomicU32,
}

impl BootAttestationManager {
    pub fn new() -> Self {
        let mut mgr = BootAttestationManager {
            tpm_selection: TpmSelection::None,
            hw_tpm: HardwareTpm::new(),
            fw_tpm: FirmwareTpm::new(),
            aik: AikManager::new(),
            quote_engine: QuoteEngine::new(),
            capabilities: AtomicU32::new(0),
        };
        if mgr.hw_tpm.probe().is_ok() && mgr.hw_tpm.init().is_ok() {
            mgr.tpm_selection = TpmSelection::Hardware;
        } else if mgr.fw_tpm.init().is_ok() {
            mgr.tpm_selection = TpmSelection::Firmware;
        }
        if mgr.tpm_selection != TpmSelection::None {
            mgr.capabilities.store((1 << CAP_BOOT_ATTEST) | (1 << CAP_MEASUREMENT_READ) | (1 << CAP_AIK_ADMIN), Ordering::Release);
        }
        mgr
    }

    /// Measure a boot component (main entry point for boot flow)
    pub fn measure(&mut self, component: BootComponent, data: &[u8], pcr_index: PcrIndex) -> MeasurementState {
        let mc = match component {
            BootComponent::Firmware => MeasBootComponent::Firmware,
            BootComponent::BootLoader => MeasBootComponent::BootLoader,
            BootComponent::BootConfig => MeasBootComponent::BootConfig,
            BootComponent::Kernel => MeasBootComponent::Kernel,
            BootComponent::InitRd => MeasBootComponent::InitRd,
            BootComponent::DeviceTree => MeasBootComponent::DeviceTree,
        };
        match self.tpm_selection {
            TpmSelection::Hardware => { let mut e = MeasurementEngine::new(&mut self.hw_tpm); e.measure_component(mc, data, pcr_index) }
            TpmSelection::Firmware => { let mut e = MeasurementEngine::new(&mut self.fw_tpm); e.measure_component(mc, data, pcr_index) }
            TpmSelection::None => MeasurementState::Failed,
        }
    }

    /// Generate attestation quote (requires CAP_BOOT_ATTEST)
    pub fn quote(&self, nonce: &[u8; NONCE_SIZE]) -> Result<AttestationResponse, BootVerifyResult> {
        if (self.capabilities.load(Ordering::Acquire) & (1 << CAP_BOOT_ATTEST)) == 0 {
            return Err(BootVerifyResult::CapabilityDenied);
        }
        let event_log = EventLogManager::new();
        match self.tpm_selection {
            TpmSelection::Hardware => self.quote_engine.attest(&self.hw_tpm, &self.aik, &event_log, nonce).map_err(|_| BootVerifyResult::InternalError),
            TpmSelection::Firmware => self.quote_engine.attest(&self.fw_tpm, &self.aik, &event_log, nonce).map_err(|_| BootVerifyResult::InternalError),
            TpmSelection::None => Err(BootVerifyResult::InternalError),
        }
    }

    pub fn tpm_selection(&self) -> TpmSelection { self.tpm_selection }
    pub fn has_capability(&self, cap: u32) -> bool { (self.capabilities.load(Ordering::Acquire) & (1 << cap)) != 0 }
}

static BOOT_CONFIG: core::sync::OnceLock<BootConfig> = core::sync::OnceLock::new();
pub fn boot_config() -> &'static BootConfig { BOOT_CONFIG.get_or_init(BootConfig::new) }
pub fn get_boot_config() -> &'static BootConfig { boot_config() }

pub fn init_secure_boot() {
    let config = get_boot_config();
    config.required.store((1 << BootComponent::Firmware as u32) | (1 << BootComponent::BootLoader as u32) | (1 << BootComponent::Kernel as u32), Ordering::Release);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_boot_state_transitions() {
        let config = BootConfig::new();
        assert_eq!(config.get_state(), SecureBootState::Disabled);
        assert!(config.enter_setup().is_ok());
        assert!(config.enforce().is_ok());
    }
    #[test]
    fn test_capability_constants() {
        assert_eq!(CAP_BOOT_ATTEST, 100);
        assert_eq!(CAP_MEASUREMENT_READ, 101);
        assert_eq!(CAP_AIK_ADMIN, 102);
    }
}
