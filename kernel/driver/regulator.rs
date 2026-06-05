/*
 * Nuva OS - Kernel - Driver - Regulator
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
 * Nuva OS - Kernel - Regulator Framework
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Voltage/Current regulator framework for device drivers.
 */

use crate::{pr_debug, pr_info};
use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// Regulator ID
pub type RegulatorId = u32;

/// Regulator Mode
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegulatorMode {
    /// Invalid
    Invalid = 0,
    /// Fast (no bypass)
    Fast = 1,
    /// Normal
    Normal = 2,
    /// Idle (low power)
    Idle = 3,
    /// Standby
    Standby = 4,
    /// Bypass
    Bypass = 5,
}

/// Regulator Status
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegulatorStatus {
    /// Off
    Off = 0,
    /// On
    On = 1,
    /// Error
    Error = 2,
    /// Fast
    Fast = 3,
    /// Normal
    Normal = 4,
    /// Idle
    Idle = 5,
    /// Standby
    Standby = 6,
    /// Bypass
    Bypass = 7,
    /// Unknown
    Unknown = 8,
}

/// Regulator Type
bitflags::bitflags! {
    #[repr(transparent)]
    pub struct RegulatorType: u32 {
        /// Voltage regulator
        const VOLTAGE = 1 << 0;
        /// Current regulator
        const CURRENT = 1 << 1;
        /// Bypass supported
        const BYPASS = 1 << 2;
    }
}

/// Regulator Constraints
#[repr(C)]
pub struct RegulatorConstraints {
    /// Name
    pub name: [u8; 32],
    /// Minimum voltage (uV)
    pub min_uV: i32,
    /// Maximum voltage (uV)
    pub max_uV: i32,
    /// Minimum current (uA)
    pub min_uA: i32,
    /// Maximum current (uA)
    pub max_uA: i32,
    /// System critical
    pub always_on: bool,
    /// Boot on
    pub boot_on: bool,
    /// Apply voltage
    pub apply_uV: bool,
    /// Ramp delay (uV/us)
    pub ramp_delay: u32,
    /// Enable time (us)
    pub enable_time: u32,
    /// Valid modes
    pub valid_modes: u32,
    /// Initial mode
    pub initial_mode: RegulatorMode,
}

/// Regulator Config
#[repr(C)]
pub struct RegulatorConfig {
    /// Constraints
    pub constraints: RegulatorConstraints,
    /// Number of consumers
    pub num_consumer_supplies: u32,
    /// Parent regulator
    pub parent: RegulatorId,
    /// Enable GPIO
    pub enable_gpio: u32,
    /// Active low enable
    pub enable_active_low: bool,
}

/// Regulator Operations
pub struct RegulatorOps {
    /// Enable
    pub enable: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    /// Disable
    pub disable: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    /// Is enabled
    pub is_enabled: Option<unsafe extern "C" fn(*const core::ffi::c_void) -> bool>,

    /// Get voltage
    pub get_voltage: Option<unsafe extern "C" fn(*const core::ffi::c_void) -> i32>,
    /// Set voltage
    pub set_voltage:
        Option<unsafe extern "C" fn(*mut core::ffi::c_void, i32, i32, *mut i32) -> i32>,
    /// List voltage
    pub list_voltage: Option<unsafe extern "C" fn(*const core::ffi::c_void, u32) -> i32>,
    /// Map voltage
    pub map_voltage: Option<unsafe extern "C" fn(*mut core::ffi::c_void, i32, i32) -> i32>,

    /// Get current
    pub get_current: Option<unsafe extern "C" fn(*const core::ffi::c_void) -> i32>,
    /// Set current
    pub set_current: Option<unsafe extern "C" fn(*mut core::ffi::c_void, i32, i32) -> i32>,

    /// Set mode
    pub set_mode: Option<unsafe extern "C" fn(*mut core::ffi::c_void, RegulatorMode) -> i32>,
    /// Get mode
    pub get_mode: Option<unsafe extern "C" fn(*const core::ffi::c_void) -> RegulatorMode>,

    /// Get status
    pub get_status: Option<unsafe extern "C" fn(*const core::ffi::c_void) -> RegulatorStatus>,

    /// Set load
    pub set_load: Option<unsafe extern "C" fn(*mut core::ffi::c_void, i32) -> i32>,
    /// Set bypass
    pub set_bypass: Option<unsafe extern "C" fn(*mut core::ffi::c_void, bool) -> i32>,
    /// Get bypass
    pub get_bypass: Option<unsafe extern "C" fn(*const core::ffi::c_void, *mut bool) -> i32>,

    /// Enable time
    pub enable_time: Option<unsafe extern "C" fn(*const core::ffi::c_void) -> i32>,
    /// Set ramp delay
    pub set_ramp_delay: Option<unsafe extern "C" fn(*mut core::ffi::c_void, i32) -> i32>,
    /// Get ramp delay
    pub get_ramp_delay: Option<unsafe extern "C" fn(*const core::ffi::c_void) -> i32>,

    /// Suspend enable
    pub suspend_enable: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    /// Suspend disable
    pub suspend_disable: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    /// Set suspend voltage
    pub set_suspend_voltage: Option<unsafe extern "C" fn(*mut core::ffi::c_void, i32) -> i32>,
    /// Set suspend mode
    pub set_suspend_mode:
        Option<unsafe extern "C" fn(*mut core::ffi::c_void, RegulatorMode) -> i32>,
}

/// Regulator Description
pub struct RegulatorDesc {
    /// Name
    pub name: [u8; 32],
    /// Supply name
    pub supply_name: [u8; 32],
    /// ID
    pub id: RegulatorId,
    /// Operations
    pub ops: RegulatorOps,
    /// Type
    pub reg_type: RegulatorType,
    /// Number of voltages
    pub n_voltages: u32,
    /// Owner module
    pub owner: u32,
    /// Minimum voltage (uV)
    pub min_uV: i32,
    /// Maximum voltage (uV)
    pub max_uV: i32,
    /// Linear ranges
    pub linear_ranges: *const LinearRange,
    /// Number of linear ranges
    pub n_linear_ranges: u32,
    /// Voltage step
    pub uV_step: u32,
    /// Enable register
    pub enable_reg: u32,
    /// Enable mask
    pub enable_mask: u32,
    /// Enable value
    pub enable_val: u32,
    /// Disable value
    pub disable_val: u32,
    /// Enable time (us)
    pub enable_time_us: u32,
    /// Ramp delay
    pub ramp_delay: u32,
}

/// Linear Range
#[repr(C)]
pub struct LinearRange {
    /// Minimum selector
    pub min_sel: u32,
    /// Maximum selector
    pub max_sel: u32,
    /// Minimum voltage (uV)
    pub min_uV: u32,
    /// Step (uV)
    pub uV_step: u32,
}

/// Regulator Device
pub struct RegulatorDev {
    /// Description
    pub desc: *const RegulatorDesc,
    /// Private data
    pub data: *mut core::ffi::c_void,
    /// Parent device
    pub parent: u32,
    /// Use count
    pub use_count: AtomicU32,
    /// Open count
    pub open_count: AtomicU32,
    /// Bypass count
    pub bypass_count: AtomicU32,
    /// Constraints
    pub constraints: RegulatorConstraints,
    /// Cached voltage
    pub cached_uV: AtomicU32,
}

/// Regulator Manager
pub struct RegulatorManager {
    /// Regulator count
    reg_count: AtomicU32,
    /// Statistics
    stats: RegulatorStats,
}

/// Regulator Statistics
pub struct RegulatorStats {
    /// Enable count
    pub enable_count: AtomicU64,
    /// Disable count
    pub disable_count: AtomicU64,
    /// Set voltage count
    pub set_voltage_count: AtomicU64,
}

impl RegulatorStats {
    pub const fn new() -> Self {
        RegulatorStats {
            enable_count: AtomicU64::new(0),
            disable_count: AtomicU64::new(0),
            set_voltage_count: AtomicU64::new(0),
        }
    }
}

impl RegulatorManager {
    pub const fn new() -> Self {
        RegulatorManager {
            reg_count: AtomicU32::new(0),
            stats: RegulatorStats::new(),
        }
    }

    /// Initialize
    pub fn init(&self) {
        log_info!("Regulator manager initialized");
    }

    /// Register regulator
    pub fn register(&mut self, _reg: &RegulatorDesc) -> RegulatorId {
        let id = self.reg_count.fetch_add(1, Ordering::AcqRel);
        id
    }

    /// Enable regulator
    pub fn enable(&mut self, reg_id: RegulatorId) -> i32 {
        self.stats.enable_count.fetch_add(1, Ordering::AcqRel);
        log_debug!("regulator_enable: id={}", reg_id);
        0
    }

    /// Disable regulator
    pub fn disable(&mut self, reg_id: RegulatorId) -> i32 {
        self.stats.disable_count.fetch_add(1, Ordering::AcqRel);
        log_debug!("regulator_disable: id={}", reg_id);
        0
    }

    /// Get voltage
    pub fn get_voltage(&self, reg_id: RegulatorId) -> i32 {
        log_debug!("regulator_get_voltage: id={}", reg_id);
        0
    }

    /// Set voltage
    pub fn set_voltage(&mut self, reg_id: RegulatorId, min_uV: i32, max_uV: i32) -> i32 {
        self.stats.set_voltage_count.fetch_add(1, Ordering::AcqRel);
        log_debug!(
            "regulator_set_voltage: id={}, min={}, max={}",
            reg_id,
            min_uV,
            max_uV
        );
        0
    }

    /// Get current
    pub fn get_current(&self, reg_id: RegulatorId) -> i32 {
        log_debug!("regulator_get_current: id={}", reg_id);
        0
    }

    /// Set current
    pub fn set_current(&mut self, reg_id: RegulatorId, min_uA: i32, max_uA: i32) -> i32 {
        log_debug!(
            "regulator_set_current: id={}, min={}, max={}",
            reg_id,
            min_uA,
            max_uA
        );
        0
    }

    /// Set mode
    pub fn set_mode(&mut self, reg_id: RegulatorId, mode: RegulatorMode) -> i32 {
        log_debug!("regulator_set_mode: id={}, mode={:?}", reg_id, mode);
        0
    }

    /// Get mode
    pub fn get_mode(&self, reg_id: RegulatorId) -> RegulatorMode {
        log_debug!("regulator_get_mode: id={}", reg_id);
        RegulatorMode::Normal
    }
}

/// Global regulator manager
static REGULATOR_MANAGER: core::sync::OnceLock<RegulatorManager> = core::sync::OnceLock::new();

/// Get regulator manager
pub fn regulator_manager() -> &'static RegulatorManager {
    REGULATOR_MANAGER.get_or_init(RegulatorManager::new)
}

pub fn init_regulator_manager() -> &'static RegulatorManager {
    REGULATOR_MANAGER.get_or_init(RegulatorManager::new)
}

/// Initialize regulator manager
pub fn init_regulator_manager() {
    let mgr = regulator_manager();
    mgr.init();
}

// Convenience functions

/// Enable regulator
pub fn regulator_enable(reg_id: RegulatorId) -> i32 {
    regulator_manager().enable(reg_id)
}

/// Disable regulator
pub fn regulator_disable(reg_id: RegulatorId) -> i32 {
    regulator_manager().disable(reg_id)
}

/// Get voltage
pub fn regulator_get_voltage(reg_id: RegulatorId) -> i32 {
    regulator_manager().get_voltage(reg_id)
}

/// Set voltage
pub fn regulator_set_voltage(reg_id: RegulatorId, min_uV: i32, max_uV: i32) -> i32 {
    regulator_manager().set_voltage(reg_id, min_uV, max_uV)
}

/// Get current
pub fn regulator_get_current(reg_id: RegulatorId) -> i32 {
    regulator_manager().get_current(reg_id)
}

/// Set current
pub fn regulator_set_current(reg_id: RegulatorId, min_uA: i32, max_uA: i32) -> i32 {
    regulator_manager().set_current(reg_id, min_uA, max_uA)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regulator_mode() {
        assert_eq!(RegulatorMode::Fast as i32, 1);
        assert_eq!(RegulatorMode::Normal as i32, 2);
        assert_eq!(RegulatorMode::Bypass as i32, 5);
    }

    #[test]
    fn test_regulator_status() {
        assert_eq!(RegulatorStatus::Off as i32, 0);
        assert_eq!(RegulatorStatus::On as i32, 1);
    }

    #[test]
    fn test_regulator_type() {
        let reg_type = RegulatorType::VOLTAGE | RegulatorType::BYPASS;
        assert!(reg_type.contains(RegulatorType::VOLTAGE));
        assert!(reg_type.contains(RegulatorType::BYPASS));
    }
}
