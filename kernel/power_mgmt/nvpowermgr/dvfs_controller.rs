/*
 * Nuva OS - Kernel - PowerMgmt - Nvpowermgr - DvfsController
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
 * Nuva OS - Kernel - NvPowerMgr DVFS Controller
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Per-device DVFS (Dynamic Voltage and Frequency Scaling)
 * state management with safe switching sequences.
 */

use core::sync::atomic::{AtomicU32, AtomicU16, Ordering};

use crate::kernel::error::{KernelError, KernelResult};

/// Maximum DVFS levels per device
pub const MAX_DVFS_LEVELS: usize = 16;

/// DVFS level descriptor
#[derive(Clone, Debug)]
pub struct DvfsLevel {
    /// Voltage in microvolts
    pub voltage_uv: u32,
    /// Frequency in kHz
    pub freq_khz: u32,
    /// Power at this level in milliwatts
    pub power_mw: u32,
}

/// Per-device DVFS state
#[derive(Clone, Debug)]
pub struct DvfsState {
    /// Current voltage in microvolts
    pub current_voltage_uv: AtomicU32,
    /// Current frequency in kHz
    pub current_freq_khz: AtomicU32,
    /// Current DVFS level index
    pub current_level: AtomicU16,
    /// Number of available levels
    pub num_levels: AtomicU16,
}

impl DvfsState {
    /// Create a new DVFS state
    pub const fn new() -> Self {
        DvfsState {
            current_voltage_uv: AtomicU32::new(0),
            current_freq_khz: AtomicU32::new(0),
            current_level: AtomicU16::new(0),
            num_levels: AtomicU16::new(0),
        }
    }

    /// Get current level
    #[inline(always)]
    pub fn level(&self) -> u16 {
        self.current_level.load(Ordering::Acquire)
    }
}

/// DvfsController: per-device DVFS management
///
/// Manages DVFS transitions with safe switching sequences:
/// - Scale up: voltage first, then frequency
/// - Scale down: frequency first, then voltage
pub struct DvfsController {
    /// Per-device DVFS states
    device_states: [DvfsState; super::MAX_POWER_DEVICES],
    /// Total DVFS adjustments
    total_adjustments: AtomicU32,
}

impl DvfsController {
    /// Create a new DVFS controller
    pub const fn new() -> Self {
        DvfsController {
            device_states: [
                DvfsState::new(), DvfsState::new(), DvfsState::new(), DvfsState::new(),
                DvfsState::new(), DvfsState::new(), DvfsState::new(), DvfsState::new(),
                DvfsState::new(), DvfsState::new(), DvfsState::new(), DvfsState::new(),
                DvfsState::new(), DvfsState::new(), DvfsState::new(), DvfsState::new(),
            ],
            total_adjustments: AtomicU32::new(0),
        }
    }

    /// Get DVFS state for a device
    pub fn get_state(&self, device_index: usize) -> Option<&DvfsState> {
        if device_index < super::MAX_POWER_DEVICES {
            Some(&self.device_states[device_index])
        } else {
            None
        }
    }

    /// Scale up: increase voltage then frequency
    ///
    /// Safe sequence: raise voltage first to ensure
    /// sufficient supply at higher frequency.
    ///
    /// @param device_index: Target device
    /// @param target_voltage_uv: Target voltage
    /// @param target_freq_khz: Target frequency
    /// @return: Ok on success
    pub fn scale_up(
        &self,
        device_index: usize,
        target_voltage_uv: u32,
        target_freq_khz: u32,
    ) -> KernelResult<()> {
        if device_index >= super::MAX_POWER_DEVICES {
            return Err(KernelError::InvalidArgument);
        }

        let state = &self.device_states[device_index];
        let current_voltage = state.current_voltage_uv.load(Ordering::Acquire);

        // Step 1: Raise voltage first
        if target_voltage_uv > current_voltage {
            // TODO: Call PmicOps::set_voltage(device_index, target_voltage_uv)
            state.current_voltage_uv.store(target_voltage_uv, Ordering::Release);
        }

        // Step 2: Raise frequency
        // TODO: Call PmicOps::set_frequency(device_index, target_freq_khz)
        state.current_freq_khz.store(target_freq_khz, Ordering::Release);

        self.total_adjustments.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    /// Scale down: decrease frequency then voltage
    ///
    /// Safe sequence: lower frequency first to ensure
    /// circuit stability at reduced voltage.
    ///
    /// @param device_index: Target device
    /// @param target_freq_khz: Target frequency
    /// @param target_voltage_uv: Target voltage
    /// @return: Ok on success
    pub fn scale_down(
        &self,
        device_index: usize,
        target_freq_khz: u32,
        target_voltage_uv: u32,
    ) -> KernelResult<()> {
        if device_index >= super::MAX_POWER_DEVICES {
            return Err(KernelError::InvalidArgument);
        }

        let state = &self.device_states[device_index];
        let current_freq = state.current_freq_khz.load(Ordering::Acquire);

        // Step 1: Lower frequency first
        if target_freq_khz < current_freq {
            // TODO: Call PmicOps::set_frequency(device_index, target_freq_khz)
            state.current_freq_khz.store(target_freq_khz, Ordering::Release);
        }

        // Step 2: Lower voltage
        // TODO: Call PmicOps::set_voltage(device_index, target_voltage_uv)
        state.current_voltage_uv.store(target_voltage_uv, Ordering::Release);

        self.total_adjustments.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    /// Set DVFS level for a device
    pub fn set_level(&self, device_index: usize, level: u16) -> KernelResult<()> {
        if device_index >= super::MAX_POWER_DEVICES {
            return Err(KernelError::InvalidArgument);
        }
        let state = &self.device_states[device_index];
        let num = state.num_levels.load(Ordering::Acquire);
        if level >= num {
            return Err(KernelError::InvalidArgument);
        }
        state.current_level.store(level, Ordering::Release);
        self.total_adjustments.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    /// Get total adjustments count
    pub fn total_adjustments(&self) -> u32 {
        self.total_adjustments.load(Ordering::Acquire)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scale_up() {
        let ctrl = DvfsController::new();
        ctrl.device_states[0].current_voltage_uv.store(800_000, Ordering::Release);
        ctrl.device_states[0].current_freq_khz.store(1_000_000, Ordering::Release);

        let result = ctrl.scale_up(0, 900_000, 1_500_000);
        assert!(result.is_ok());
        assert_eq!(ctrl.device_states[0].current_voltage_uv.load(Ordering::Relaxed), 900_000);
        assert_eq!(ctrl.device_states[0].current_freq_khz.load(Ordering::Relaxed), 1_500_000);
    }

    #[test]
    fn test_scale_down() {
        let ctrl = DvfsController::new();
        ctrl.device_states[0].current_voltage_uv.store(900_000, Ordering::Release);
        ctrl.device_states[0].current_freq_khz.store(1_500_000, Ordering::Release);

        let result = ctrl.scale_down(0, 1_000_000, 800_000);
        assert!(result.is_ok());
        assert_eq!(ctrl.device_states[0].current_freq_khz.load(Ordering::Relaxed), 1_000_000);
        assert_eq!(ctrl.device_states[0].current_voltage_uv.load(Ordering::Relaxed), 800_000);
    }

    #[test]
    fn test_invalid_device() {
        let ctrl = DvfsController::new();
        let result = ctrl.scale_up(99, 900_000, 1_500_000);
        assert!(result.is_err());
    }
}