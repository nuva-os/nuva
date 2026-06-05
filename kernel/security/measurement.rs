/*
 * Nuva OS - Kernel - Measurement Engine
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

//! Measurement Engine
//!
//! Performs component measurement, PCR extension, and event logging.
//! Measurement failures result in graceful degradation (incomplete state)
//! rather than blocking the boot process.

use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, AtomicU8, Ordering};

use super::tpm_abi::{TpmAbi, TpmError, PcrIndex, PCR_DIGEST_SIZE};
use super::sha256::sha256_digest;
use super::event_log::{EventLogManager, MeasurementEvent};
use crate::{pr_info, pr_debug, pr_warn};

/// Boot component in the verification chain
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootComponent {
    /// Platform firmware (UEFI/BIOS)
    Firmware = 0,
    /// Boot loader
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

/// Measurement state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeasurementState {
    /// All measurements completed successfully
    Complete = 0,
    /// Some measurements failed but boot continues
    Incomplete = 1,
    /// Critical measurement failure
    Failed = 2,
}

impl MeasurementState {
    fn from_u8(v: u8) -> Self {
        match v { 0 => MeasurementState::Complete, 1 => MeasurementState::Incomplete, _ => MeasurementState::Failed }
    }
}

/// Measurement engine
///
/// Orchestrates the three-phase measurement pipeline:
/// 1. Compute hash (target: <=10ms)
/// 2. PCR extend (target: <=1ms)
/// 3. Event record (target: <=5ms)
pub struct MeasurementEngine<T: TpmAbi> {
    /// TPM backend for PCR operations
    tpm: T,
    /// Event log manager
    event_log: EventLogManager,
    /// Overall measurement state
    state: AtomicU8,
    /// Total measurements performed
    measure_count: AtomicU64,
    /// Failed measurement count
    fail_count: AtomicU64,
}

impl<T: TpmAbi> MeasurementEngine<T> {
    /// Create a new measurement engine with the given TPM backend
    pub fn new(tpm: T) -> Self {
        MeasurementEngine {
            tpm,
            event_log: EventLogManager::new(),
            state: AtomicU8::new(MeasurementState::Complete as u8),
            measure_count: AtomicU64::new(0),
            fail_count: AtomicU64::new(0),
        }
    }

    /// Measure a boot component.
    ///
    /// Three-phase pipeline:
    /// 1. Compute SHA-256 hash of component data (<=10ms)
    /// 2. Extend PCR register with the hash (<=1ms)
    /// 3. Record measurement event in the log (<=5ms)
    ///
    /// On failure, marks state as Incomplete but does NOT block boot.
    pub fn measure_component(&mut self, component: BootComponent, data: &[u8], pcr_index: PcrIndex) -> MeasurementState {
        self.measure_count.fetch_add(1, Ordering::Relaxed);
        // Phase 1: Compute hash
        let hash = sha256_digest(data);
        // Phase 2: PCR extend
        let pcr_result = self.tpm.pcr_extend(pcr_index, &hash);
        if let Err(e) = pcr_result {
            self.fail_count.fetch_add(1, Ordering::Relaxed);
            self.degrade_state(MeasurementState::Incomplete);
            // Still log the event even on PCR failure
            self.event_log.append(MeasurementEvent {
                component: component as u32,
                pcr_index,
                digest: hash,
                data_size: data.len() as u64,
                timestamp: self.measure_count.load(Ordering::Relaxed),
                pcr_extend_ok: false,
            });
            return self.get_state();
        }
        // Phase 3: Record event
        self.event_log.append(MeasurementEvent {
            component: component as u32,
            pcr_index,
            digest: hash,
            data_size: data.len() as u64,
            timestamp: self.measure_count.load(Ordering::Relaxed),
            pcr_extend_ok: true,
        });
        self.get_state()
    }

    /// Get current measurement state
    pub fn get_state(&self) -> MeasurementState {
        MeasurementState::from_u8(self.state.load(Ordering::Acquire))
    }

    /// Degrade measurement state (only allows transitions to worse states)
    fn degrade_state(&self, new_state: MeasurementState) {
        let current = self.get_state();
        if new_state as u8 > current as u8 {
            self.state.store(new_state as u8, Ordering::Release);
        }
    }

    /// Get reference to the event log
    pub fn event_log(&self) -> &EventLogManager { &self.event_log }

    /// Get reference to the TPM backend
    pub fn tpm(&self) -> &T { &self.tpm }

    /// Get mutable reference to the TPM backend
    pub fn tpm_mut(&mut self) -> &mut T { &mut self.tpm }

    /// Get measurement statistics
    pub fn stats(&self) -> (u64, u64) {
        (self.measure_count.load(Ordering::Acquire), self.fail_count.load(Ordering::Acquire))
    }
}
