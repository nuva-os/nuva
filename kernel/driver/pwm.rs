/*
 * Nuva OS - Kernel - Driver - Pwm
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
 * Nuva OS - Kernel - PWM Framework
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * PWM (Pulse Width Modulation) framework for device drivers.
 */

use crate::{pr_debug, pr_info};
use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// PWM Channel ID
pub type PwmChannel = u32;

/// PWM Polarity
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PwmPolarity {
    /// Normal polarity (active high)
    Normal = 0,
    /// Inversed polarity (active low)
    Inversed = 1,
}

/// PWM Capture Result
#[repr(C)]
pub struct PwmCapture {
    /// Period (ns)
    pub period: u64,
    /// Duty cycle (ns)
    pub duty_cycle: u64,
    /// Polarity
    pub polarity: PwmPolarity,
}

/// PWM State
#[repr(C)]
pub struct PwmState {
    /// Period (ns)
    pub period: u64,
    /// Duty cycle (ns)
    pub duty_cycle: u64,
    /// Polarity
    pub polarity: PwmPolarity,
    /// Enabled
    pub enabled: bool,
    /// Power usage count
    pub usage_power: u32,
}

impl Default for PwmState {
    fn default() -> Self {
        PwmState {
            period: 1_000_000,   // 1ms default
            duty_cycle: 500_000, // 50% default
            polarity: PwmPolarity::Normal,
            enabled: false,
            usage_power: 0,
        }
    }
}

impl PwmState {
    /// Get duty cycle as percentage (0-100)
    pub fn duty_percent(&self) -> u8 {
        if self.period == 0 {
            return 0;
        }
        ((self.duty_cycle * 100 / self.period) as u8).min(100)
    }

    /// Get frequency in Hz
    pub fn frequency_hz(&self) -> u32 {
        if self.period == 0 {
            return 0;
        }
        (1_000_000_000 / self.period) as u32
    }
}

/// PWM Args (for device tree)
#[repr(C)]
pub struct PwmArgs {
    /// PWM chip ID
    pub chip_id: u32,
    /// Channel number
    pub channel: u32,
    /// Period (ns)
    pub period: u64,
    /// Polarity
    pub polarity: PwmPolarity,
}

/// PWM Operations
pub struct PwmOps {
    /// Request PWM
    pub request: Option<unsafe extern "C" fn(*mut core::ffi::c_void, PwmChannel) -> i32>,
    /// Free PWM
    pub free: Option<unsafe extern "C" fn(*mut core::ffi::c_void, PwmChannel)>,
    /// Config
    pub config: Option<unsafe extern "C" fn(*mut core::ffi::c_void, PwmChannel, u64, u64) -> i32>,
    /// Set polarity
    pub set_polarity:
        Option<unsafe extern "C" fn(*mut core::ffi::c_void, PwmChannel, PwmPolarity) -> i32>,
    /// Enable
    pub enable: Option<unsafe extern "C" fn(*mut core::ffi::c_void, PwmChannel) -> i32>,
    /// Disable
    pub disable: Option<unsafe extern "C" fn(*mut core::ffi::c_void, PwmChannel) -> i32>,
    /// Get state
    pub get_state:
        Option<unsafe extern "C" fn(*const core::ffi::c_void, PwmChannel, *mut PwmState)>,
    /// Apply state
    pub apply:
        Option<unsafe extern "C" fn(*mut core::ffi::c_void, PwmChannel, *const PwmState) -> i32>,
    /// Capture
    pub capture: Option<
        unsafe extern "C" fn(*mut core::ffi::c_void, PwmChannel, *mut PwmCapture, u32) -> i32,
    >,
}

/// PWM Chip
pub struct PwmChip {
    /// Chip name
    pub name: [u8; 32],
    /// Chip ID
    pub id: u32,
    /// Operations
    pub ops: PwmOps,
    /// Private data
    pub data: *mut core::ffi::c_void,
    /// Parent device
    pub parent: u32,
    /// Base channel number
    pub base: u32,
    /// Number of PWMs
    pub npwm: u16,
    /// Of PWM args
    pub of_xlate:
        Option<unsafe extern "C" fn(*const PwmChip, *const u32, usize, *mut PwmArgs) -> i32>,
}

/// PWM Device
#[repr(C)]
pub struct PwmDevice {
    /// Label
    pub label: [u8; 32],
    /// Chip ID
    pub chip_id: u32,
    /// Channel
    pub hwpwm: u32,
    /// Global PWM number
    pub pwm: u32,
    /// State
    pub state: PwmState,
    /// Flags
    pub flags: PwmFlags,
}

/// PWM Flags
bitflags::bitflags! {
    #[repr(transparent)]
    pub struct PwmFlags: u32 {
        /// PWM is active
        const ACTIVE = 1 << 0;
        /// Output is enabled
        const OUTPUT_ENABLED = 1 << 1;
        /// Capture mode
        const CAPTURE = 1 << 2;
    }
}

/// PWM Manager
pub struct PwmManager {
    /// Chip count
    chip_count: AtomicU32,
    /// Statistics
    stats: PwmStats,
}

/// PWM Statistics
pub struct PwmStats {
    /// Enable count
    pub enable_count: AtomicU64,
    /// Disable count
    pub disable_count: AtomicU64,
    /// Config count
    pub config_count: AtomicU64,
}

impl PwmStats {
    pub const fn new() -> Self {
        PwmStats {
            enable_count: AtomicU64::new(0),
            disable_count: AtomicU64::new(0),
            config_count: AtomicU64::new(0),
        }
    }
}

impl PwmManager {
    pub const fn new() -> Self {
        PwmManager {
            chip_count: AtomicU32::new(0),
            stats: PwmStats::new(),
        }
    }

    /// Initialize
    pub fn init(&self) {
        log_info!("PWM manager initialized");
    }

    /// Register chip
    pub fn register_chip(&mut self, _chip: &PwmChip) -> u32 {
        let id = self.chip_count.fetch_add(1, Ordering::AcqRel);
        id
    }

    /// Request PWM
    pub fn request(&mut self, chip_id: u32, channel: PwmChannel) -> i32 {
        log_debug!("pwm_request: chip={}, channel={}", chip_id, channel);
        0
    }

    /// Free PWM
    pub fn free(&mut self, chip_id: u32, channel: PwmChannel) {
        log_debug!("pwm_free: chip={}, channel={}", chip_id, channel);
    }

    /// Config PWM
    pub fn config(
        &mut self,
        chip_id: u32,
        channel: PwmChannel,
        period: u64,
        duty_cycle: u64,
    ) -> i32 {
        self.stats.config_count.fetch_add(1, Ordering::AcqRel);
        log_debug!(
            "pwm_config: chip={}, channel={}, period={}, duty={}",
            chip_id,
            channel,
            period,
            duty_cycle
        );
        0
    }

    /// Enable PWM
    pub fn enable(&mut self, chip_id: u32, channel: PwmChannel) -> i32 {
        self.stats.enable_count.fetch_add(1, Ordering::AcqRel);
        log_debug!("pwm_enable: chip={}, channel={}", chip_id, channel);
        0
    }

    /// Disable PWM
    pub fn disable(&mut self, chip_id: u32, channel: PwmChannel) -> i32 {
        self.stats.disable_count.fetch_add(1, Ordering::AcqRel);
        log_debug!("pwm_disable: chip={}, channel={}", chip_id, channel);
        0
    }

    /// Apply state
    pub fn apply_state(&mut self, chip_id: u32, channel: PwmChannel, state: &PwmState) -> i32 {
        log_debug!(
            "pwm_apply: chip={}, channel={}, period={}, duty={}",
            chip_id,
            channel,
            state.period,
            state.duty_cycle
        );
        0
    }

    /// Get state
    pub fn get_state(&self, chip_id: u32, channel: PwmChannel) -> PwmState {
        log_debug!("pwm_get_state: chip={}, channel={}", chip_id, channel);
        PwmState::default()
    }
}

/// Global PWM manager
static PWM_MANAGER: crate::sync_oncelock::OnceLock<PwmManager> = crate::sync_oncelock::OnceLock::new();

/// Get PWM manager
pub fn pwm_manager() -> &'static PwmManager {
    PWM_MANAGER.get_or_init(PwmManager::new)
}

/// Initialize PWM manager
pub fn init_pwm_manager() {
    let mgr = pwm_manager();
    mgr.init();
}

// Convenience functions

/// Request PWM
pub fn pwm_request(chip_id: u32, channel: PwmChannel) -> i32 {
    pwm_manager().request(chip_id, channel)
}

/// Free PWM
pub fn pwm_free(chip_id: u32, channel: PwmChannel) {
    pwm_manager().free(chip_id, channel);
}

/// Config PWM
pub fn pwm_config(chip_id: u32, channel: PwmChannel, period: u64, duty_cycle: u64) -> i32 {
    pwm_manager().config(chip_id, channel, period, duty_cycle)
}

/// Enable PWM
pub fn pwm_enable(chip_id: u32, channel: PwmChannel) -> i32 {
    pwm_manager().enable(chip_id, channel)
}

/// Disable PWM
pub fn pwm_disable(chip_id: u32, channel: PwmChannel) -> i32 {
    pwm_manager().disable(chip_id, channel)
}

/// Apply state
pub fn pwm_apply_state(chip_id: u32, channel: PwmChannel, state: &PwmState) -> i32 {
    pwm_manager().apply_state(chip_id, channel, state)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pwm_polarity() {
        assert_eq!(PwmPolarity::Normal as i32, 0);
        assert_eq!(PwmPolarity::Inversed as i32, 1);
    }

    #[test]
    fn test_pwm_state_default() {
        let state = PwmState::default();
        assert!(!state.enabled);
        assert_eq!(state.polarity, PwmPolarity::Normal);
    }

    #[test]
    fn test_pwm_state_duty_percent() {
        let mut state = PwmState::default();
        state.period = 1_000_000;
        state.duty_cycle = 250_000;
        assert_eq!(state.duty_percent(), 25);

        state.duty_cycle = 500_000;
        assert_eq!(state.duty_percent(), 50);
    }

    #[test]
    fn test_pwm_state_frequency() {
        let mut state = PwmState::default();
        state.period = 1_000_000; // 1ms
        assert_eq!(state.frequency_hz(), 1000);

        state.period = 20_000_000; // 20ms (typical servo)
        assert_eq!(state.frequency_hz(), 50);
    }
}
