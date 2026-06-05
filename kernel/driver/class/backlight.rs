/*
 * Nuva OS - Kernel - Driver - Class - Backlight
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
 * Nuva OS - Kernel - Backlight Device Class
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Standard interface for backlight devices.
 */

use crate::{pr_debug, pr_info};
use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// Backlight Type
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BacklightType {
    /// Unknown
    Unknown = 0,
    /// Raw (direct control)
    Raw = 1,
    /// Platform
    Platform = 2,
    /// Firmware
    Firmware = 3,
    /// PWM
    Pwm = 4,
    /// LED
    Led = 5,
}

/// Backlight Scale
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BacklightScale {
    /// Linear
    Linear = 0,
    /// Non-linear (perceptual)
    NonLinear = 1,
    /// Inverse non-linear
    InverseNonLinear = 2,
}

/// Backlight State
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BacklightState {
    /// Off
    Off = 0,
    /// On
    On = 1,
    /// Suspended
    Suspended = 2,
    /// Unspecified
    Unspecified = 3,
}

/// Backlight Power
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BacklightPower {
    /// Power on
    On = 0,
    /// Power off
    Off = 1,
    /// Suspend
    Suspend = 2,
    /// Resume
    Resume = 3,
}

/// Backlight Properties
#[repr(C)]
pub struct BacklightProps {
    /// Backlight name
    pub name: [u8; 32],
    /// Backlight type
    pub bl_type: BacklightType,
    /// Current brightness
    pub brightness: u32,
    /// Maximum brightness
    pub max_brightness: u32,
    /// Actual brightness (after scaling)
    pub actual_brightness: u32,
    /// Default brightness
    pub default_brightness: u32,
    /// Minimum brightness
    pub min_brightness: u32,
    /// Scale type
    pub scale: BacklightScale,
    /// State
    pub state: BacklightState,
    /// Power
    pub power: BacklightPower,
    /// Flags
    pub flags: BacklightFlags,
}

/// Backlight Flags
bitflags::bitflags! {
    #[repr(transparent)]
    pub struct BacklightFlags: u32 {
        /// Supports suspend
        const SUSPEND = 1 << 0;
        /// Auto brightness
        const AUTO_BRIGHTNESS = 1 << 1;
        /// External control
        const EXTERNAL = 1 << 2;
        /// Connected
        const CONNECTED = 1 << 3;
    }
}

/// Backlight Operations
pub struct BacklightDeviceOps {
    /// Update status
    pub update_status: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    /// Get brightness
    pub get_brightness: Option<unsafe extern "C" fn(*const core::ffi::c_void) -> u32>,
    /// Check blank
    pub check_fb_blank: Option<unsafe extern "C" fn(*const core::ffi::c_void, i32) -> bool>,
}

/// Backlight ioctl commands
pub mod backlight_ioctl {
    /// Get properties
    pub const GET_PROPS: u32 = 0x1101;
    /// Set brightness
    pub const SET_BRIGHTNESS: u32 = 0x1102;
    /// Get brightness
    pub const GET_BRIGHTNESS: u32 = 0x1103;
    /// Set power
    pub const SET_POWER: u32 = 0x1104;
    /// Get power
    pub const GET_POWER: u32 = 0x1105;
    /// Set state
    pub const SET_STATE: u32 = 0x1106;
    /// Get state
    pub const GET_STATE: u32 = 0x1107;
}

/// Backlight Manager
pub struct BacklightManager {
    /// Backlight count
    bl_count: AtomicU32,
    /// Statistics
    stats: BacklightStats,
}

/// Backlight Statistics
pub struct BacklightStats {
    /// Set count
    pub set_count: AtomicU64,
    /// Total brightness change
    pub total_change: AtomicU64,
}

impl BacklightStats {
    pub const fn new() -> Self {
        BacklightStats {
            set_count: AtomicU64::new(0),
            total_change: AtomicU64::new(0),
        }
    }
}

impl BacklightManager {
    pub const fn new() -> Self {
        BacklightManager {
            bl_count: AtomicU32::new(0),
            stats: BacklightStats::new(),
        }
    }

    /// Initialize
    pub fn init(&self) {
        log_info!("Backlight manager initialized");
    }

    /// Register backlight
    pub fn register_backlight(&mut self) -> u32 {
        self.bl_count.fetch_add(1, Ordering::AcqRel)
    }

    /// Get backlight count
    pub fn get_backlight_count(&self) -> u32 {
        self.bl_count.load(Ordering::Acquire)
    }

    /// Set brightness
    pub fn set_brightness(&mut self, bl_id: u32, brightness: u32) -> i32 {
        self.stats.set_count.fetch_add(1, Ordering::AcqRel);
        log_debug!("backlight_set: id={}, brightness={}", bl_id, brightness);
        0
    }

    /// Get brightness
    pub fn get_brightness(&self, _bl_id: u32) -> u32 {
        0
    }
}

/// Global backlight manager
static BACKLIGHT_MANAGER: core::sync::OnceLock<BacklightManager> = core::sync::OnceLock::new();

/// Get backlight manager
pub fn backlight_manager() -> &'static BacklightManager {
    BACKLIGHT_MANAGER.get_or_init(BacklightManager::new)
}

pub fn init_backlight_manager() -> &'static BacklightManager {
    BACKLIGHT_MANAGER.get_or_init(BacklightManager::new)
}

/// Initialize backlight manager
pub fn init_backlight_manager() {
    let mgr = backlight_manager();
    mgr.init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backlight_type() {
        assert_eq!(BacklightType::Pwm as i32, 4);
        assert_eq!(BacklightType::Led as i32, 5);
    }

    #[test]
    fn test_backlight_scale() {
        assert_eq!(BacklightScale::Linear as i32, 0);
        assert_eq!(BacklightScale::NonLinear as i32, 1);
    }

    #[test]
    fn test_backlight_state() {
        assert_eq!(BacklightState::On as i32, 1);
        assert_eq!(BacklightState::Suspended as i32, 2);
    }
}
