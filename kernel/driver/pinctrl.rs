/*
 * Nuva OS - Kernel - Driver - Pinctrl
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
 * Nuva OS - Kernel - Pin Control Framework
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Pin control and multiplexing framework.
 */

use crate::{pr_debug, pr_info};
use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// Pin ID
pub type PinId = u32;

/// Pin Group ID
pub type PinGroupId = u32;

/// Pin Function ID
pub type PinFuncId = u32;

/// Pin Configuration
bitflags::bitflags! {
    #[repr(transparent)]
    pub struct PinConfig: u64 {
        /// Bias pull up
        const BIAS_PULL_UP = 1 << 0;
        /// Bias pull down
        const BIAS_PULL_DOWN = 1 << 1;
        /// Bias disable
        const BIAS_DISABLE = 1 << 2;
        /// Bias high impedance
        const BIAS_HIGH_IMPEDANCE = 1 << 3;
        /// Drive strength push pull
        const DRIVE_PUSH_PULL = 1 << 4;
        /// Drive strength open drain
        const DRIVE_OPEN_DRAIN = 1 << 5;
        /// Drive strength open source
        const DRIVE_OPEN_SOURCE = 1 << 6;
        /// Drive strength
        const DRIVE_STRENGTH = 1 << 7;
        /// Input enable
        const INPUT_ENABLE = 1 << 8;
        /// Input disable
        const INPUT_DISABLE = 1 << 9;
        /// Input schmitt enable
        const INPUT_SCHMITT = 1 << 10;
        /// Input debounce
        const INPUT_DEBOUNCE = 1 << 11;
        /// Slew rate
        const SLEW_RATE = 1 << 12;
        /// Low power mode
        const LOW_POWER_MODE = 1 << 13;
        /// Output enable
        const OUTPUT_ENABLE = 1 << 14;
        /// Output disable
        const OUTPUT_DISABLE = 1 << 15;
        /// Output high
        const OUTPUT_HIGH = 1 << 16;
        /// Output low
        const OUTPUT_LOW = 1 << 17;
        /// Sleep mode
        const SLEEP_MODE = 1 << 18;
        /// Wakeup enable
        const WAKEUP_ENABLE = 1 << 19;
    }
}

/// Pin Configuration Parameter
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PinConfigParam {
    /// Bias pull up
    BiasPullUp = 0,
    /// Bias pull down
    BiasPullDown = 1,
    /// Bias disable
    BiasDisable = 2,
    /// Bias bus hold
    BiasBusHold = 3,
    /// Drive strength
    DriveStrength = 4,
    /// Drive strength uA
    DriveStrengthUa = 5,
    /// Input enable
    InputEnable = 6,
    /// Input schmitt
    InputSchmitt = 7,
    /// Input schmitt enable
    InputSchmittEnable = 8,
    /// Input debounce
    InputDebounce = 9,
    /// Slew rate
    SlewRate = 10,
    /// Low power mode
    LowPowerMode = 11,
    /// Output enable
    OutputEnable = 12,
    /// Output high
    OutputHigh = 13,
    /// Output low
    OutputLow = 14,
    /// Sleep mode
    SleepMode = 15,
    /// Wakeup enable
    WakeupEnable = 16,
}

/// Pin Configuration Item
#[repr(C)]
pub struct PinConfigItem {
    /// Parameter
    pub param: PinConfigParam,
    /// Argument
    pub arg: u32,
}

/// Pin Group
#[repr(C)]
pub struct PinGroup {
    /// Group name
    pub name: [u8; 32],
    /// Group ID
    pub id: PinGroupId,
    /// Pins
    pub pins: [PinId; 32],
    /// Number of pins
    pub num_pins: u8,
    /// Pin configuration
    pub configs: [PinConfigItem; 8],
    /// Number of configs
    pub num_configs: u8,
}

/// Pin Function
#[repr(C)]
pub struct PinFunction {
    /// Function name
    pub name: [u8; 32],
    /// Function ID
    pub id: PinFuncId,
    /// Groups
    pub groups: [PinGroupId; 16],
    /// Number of groups
    pub num_groups: u8,
}

/// Pin State
#[repr(C)]
pub struct PinState {
    /// State name
    pub name: [u8; 32],
    /// State ID
    pub id: u32,
    /// Settings
    pub settings: [PinSetting; 16],
    /// Number of settings
    pub num_settings: u8,
}

/// Pin Setting
#[repr(C)]
pub struct PinSetting {
    /// Group ID
    pub group: PinGroupId,
    /// Function ID
    pub func: PinFuncId,
    /// Configuration
    pub configs: [PinConfigItem; 8],
    /// Number of configs
    pub num_configs: u8,
}

/// Pin Controller Operations
pub struct PinctrlOps {
    /// Get groups count
    pub get_groups_count: Option<unsafe extern "C" fn(*const core::ffi::c_void) -> u32>,
    /// Get group name
    pub get_group_name: Option<unsafe extern "C" fn(*const core::ffi::c_void, u32) -> *const u8>,
    /// Get group pins
    pub get_group_pins:
        Option<unsafe extern "C" fn(*const core::ffi::c_void, u32, *mut u32, *mut u32) -> i32>,
    /// Get functions count
    pub get_functions_count: Option<unsafe extern "C" fn(*const core::ffi::c_void) -> u32>,
    /// Get function name
    pub get_function_name: Option<unsafe extern "C" fn(*const core::ffi::c_void, u32) -> *const u8>,
    /// Get function groups
    pub get_function_groups:
        Option<unsafe extern "C" fn(*const core::ffi::c_void, u32, *mut u32, *mut u32) -> i32>,
    /// Set mux
    pub set_mux:
        Option<unsafe extern "C" fn(*mut core::ffi::c_void, u32, u32, u32, u32, u32) -> i32>,
    /// GPIO request enable
    pub gpio_request_enable:
        Option<unsafe extern "C" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, u32) -> i32>,
    /// GPIO disable free
    pub gpio_disable_free:
        Option<unsafe extern "C" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, u32)>,
    /// GPIO set direction
    pub gpio_set_direction: Option<
        unsafe extern "C" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, u32, bool) -> i32,
    >,
}

/// Pin Controller
pub struct PinctrlDev {
    /// Controller name
    pub name: [u8; 32],
    /// Controller ID
    pub id: u32,
    /// Operations
    pub ops: PinctrlOps,
    /// Private data
    pub data: *mut core::ffi::c_void,
    /// Parent device
    pub parent: u32,
    /// Number of pins
    pub npins: u32,
    /// Pin descriptor
    pub pins: *const PinDesc,
    /// Number of groups
    pub num_groups: u32,
    /// Number of functions
    pub num_functions: u32,
    /// Flags
    pub flags: PinctrlFlags,
}

/// Pin Descriptor
#[repr(C)]
pub struct PinDesc {
    /// Pin number
    pub number: PinId,
    /// Pin name
    pub name: [u8; 16],
    /// Dynamic name
    pub dynamic_name: bool,
}

/// Pin Controller Flags
bitflags::bitflags! {
    #[repr(transparent)]
    pub struct PinctrlFlags: u32 {
        /// No mapping
        const NO_MAPPING = 1 << 0;
        /// GPIO range
        const GPIO_RANGE = 1 << 1;
    }
}

/// Pin Control Manager
pub struct PinctrlManager {
    /// Controller count
    ctrl_count: AtomicU32,
    /// Statistics
    stats: PinctrlStats,
}

/// Pin Control Statistics
pub struct PinctrlStats {
    /// Mux changes
    pub mux_changes: AtomicU64,
    /// Config changes
    pub config_changes: AtomicU64,
    /// State changes
    pub state_changes: AtomicU64,
}

impl PinctrlStats {
    pub const fn new() -> Self {
        PinctrlStats {
            mux_changes: AtomicU64::new(0),
            config_changes: AtomicU64::new(0),
            state_changes: AtomicU64::new(0),
        }
    }
}

impl PinctrlManager {
    pub const fn new() -> Self {
        PinctrlManager {
            ctrl_count: AtomicU32::new(0),
            stats: PinctrlStats::new(),
        }
    }

    /// Initialize
    pub fn init(&self) {
        log_info!("Pin control manager initialized");
    }

    /// Register controller
    pub fn register_controller(&mut self, _ctrl: &PinctrlDev) -> u32 {
        self.ctrl_count.fetch_add(1, Ordering::AcqRel)
    }

    /// Unregister controller
    pub fn unregister_controller(&mut self, ctrl_id: u32) {
        log_debug!("pinctrl_unregister: id={}", ctrl_id);
    }

    /// Select state
    pub fn select_state(&mut self, dev_id: u32, state_id: u32) -> i32 {
        self.stats.state_changes.fetch_add(1, Ordering::AcqRel);
        log_debug!("pinctrl_select_state: dev={}, state={}", dev_id, state_id);
        0
    }

    /// Lookup state
    pub fn lookup_state(&self, dev_id: u32, name: &[u8]) -> u32 {
        log_debug!("pinctrl_lookup_state: dev={}, name={:?}", dev_id, name);
        0
    }

    /// Set config
    pub fn set_config(&mut self, ctrl_id: u32, pin: PinId, configs: &[PinConfigItem]) -> i32 {
        self.stats.config_changes.fetch_add(1, Ordering::AcqRel);
        log_debug!("pinctrl_set_config: ctrl={}, pin={}", ctrl_id, pin);
        0
    }

    /// Get config
    pub fn get_config(&self, ctrl_id: u32, pin: PinId, param: PinConfigParam) -> u32 {
        log_debug!(
            "pinctrl_get_config: ctrl={}, pin={}, param={:?}",
            ctrl_id,
            pin,
            param
        );
        0
    }
}

/// Global pin control manager
static PINCTRL_MANAGER: crate::sync_oncelock::OnceLock<PinctrlManager> = crate::sync_oncelock::OnceLock::new();

/// Get pin control manager
pub fn pinctrl_manager() -> &'static PinctrlManager {
    PINCTRL_MANAGER.get_or_init(PinctrlManager::new)
}

/// Initialize pin control manager
pub fn init_pinctrl_manager() {
    let mgr = pinctrl_manager();
    mgr.init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pin_config() {
        let config = PinConfig::BIAS_PULL_UP | PinConfig::DRIVE_PUSH_PULL;
        assert!(config.contains(PinConfig::BIAS_PULL_UP));
        assert!(config.contains(PinConfig::DRIVE_PUSH_PULL));
    }

    #[test]
    fn test_pin_config_param() {
        assert_eq!(PinConfigParam::BiasPullUp as i32, 0);
        assert_eq!(PinConfigParam::DriveStrength as i32, 4);
    }

    #[test]
    fn test_pinctrl_flags() {
        let flags = PinctrlFlags::GPIO_RANGE;
        assert!(flags.contains(PinctrlFlags::GPIO_RANGE));
    }
}
