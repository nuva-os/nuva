/*
 * Nuva OS - Kernel - PowerMgmt - Nvpowermgr - DeviceController
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
 * Nuva OS - Kernel - NvPowerMgr Device Power Controller
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Per-device independent power control with sleep/wake
 * management. Critical devices never sleep.
 */

use core::sync::atomic::{AtomicU32, AtomicBool, Ordering};

use crate::kernel::error::{KernelError, KernelResult};

/// Device sleep level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SleepLevel {
    /// Active (full power)
    Active = 0,
    /// Idle (reduced clock)
    Idle = 1,
    /// Light sleep (clock gated)
    LightSleep = 2,
    /// Deep sleep (power gated)
    DeepSleep = 3,
}

/// Wake condition
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum WakeCondition {
    /// Any interrupt
    AnyInterrupt = 0,
    /// Specific event
    SpecificEvent = 1,
    /// Timer expiry
    TimerExpiry = 2,
    /// Software request
    SoftwareRequest = 3,
}

/// Per-device power state
pub struct DevicePowerState {
    /// Current sleep level
    sleep_level: AtomicU8,
    /// Whether device is critical (never sleeps)
    is_critical: AtomicBool,
    /// Whether device is currently sleeping
    is_sleeping: AtomicBool,
    /// Idle time counter (milliseconds)
    idle_time_ms: AtomicU32,
    /// Wake condition for auto-wake
    wake_condition: AtomicU8,
}

impl DevicePowerState {
    /// Create a new device power state
    pub const fn new(is_critical: bool) -> Self {
        DevicePowerState {
            sleep_level: AtomicU8::new(SleepLevel::Active as u8),
            is_critical: AtomicBool::new(is_critical),
            is_sleeping: AtomicBool::new(false),
            idle_time_ms: AtomicU32::new(0),
            wake_condition: AtomicU8::new(WakeCondition::AnyInterrupt as u8),
        }
    }

    /// Check if device is critical
    #[inline(always)]
    pub fn is_critical(&self) -> bool {
        self.is_critical.load(Ordering::Acquire)
    }

    /// Check if device is sleeping
    #[inline(always)]
    pub fn is_sleeping(&self) -> bool {
        self.is_sleeping.load(Ordering::Acquire)
    }

    /// Get current sleep level
    pub fn sleep_level(&self) -> SleepLevel {
        match self.sleep_level.load(Ordering::Acquire) {
            0 => SleepLevel::Active,
            1 => SleepLevel::Idle,
            2 => SleepLevel::LightSleep,
            _ => SleepLevel::DeepSleep,
        }
    }

    /// Update idle time
    pub fn update_idle(&self, delta_ms: u32) {
        self.idle_time_ms.fetch_add(delta_ms, Ordering::Relaxed);
    }

    /// Reset idle time (on activity)
    pub fn reset_idle(&self) {
        self.idle_time_ms.store(0, Ordering::Release);
    }
}

/// DevicePowerController: per-device power management
pub struct DevicePowerController {
    /// Per-device power states
    device_states: [DevicePowerState; super::MAX_POWER_DEVICES],
    /// Sleep events count
    sleep_events: AtomicU32,
    /// Wake events count
    wake_events: AtomicU32,
}

impl DevicePowerController {
    /// Create a new device power controller
    pub const fn new() -> Self {
        DevicePowerController {
            device_states: [
                DevicePowerState::new(false), DevicePowerState::new(false),
                DevicePowerState::new(false), DevicePowerState::new(false),
                DevicePowerState::new(false), DevicePowerState::new(false),
                DevicePowerState::new(false), DevicePowerState::new(false),
                DevicePowerState::new(false), DevicePowerState::new(false),
                DevicePowerState::new(false), DevicePowerState::new(false),
                DevicePowerState::new(false), DevicePowerState::new(false),
                DevicePowerState::new(false), DevicePowerState::new(false),
            ],
            sleep_events: AtomicU32::new(0),
            wake_events: AtomicU32::new(0),
        }
    }

    /// Get device power state
    pub fn get_state(&self, device_index: usize) -> Option<&DevicePowerState> {
        if device_index < super::MAX_POWER_DEVICES {
            Some(&self.device_states[device_index])
        } else {
            None
        }
    }

    /// Put device to sleep
    ///
    /// Critical devices are never put to sleep.
    pub fn sleep(&self, device_index: usize, level: SleepLevel) -> KernelResult<()> {
        if device_index >= super::MAX_POWER_DEVICES {
            return Err(KernelError::InvalidArgument);
        }

        let state = &self.device_states[device_index];
        if state.is_critical() {
            return Ok(());
        }

        state.sleep_level.store(level as u8, Ordering::Release);
        state.is_sleeping.store(true, Ordering::Release);
        self.sleep_events.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    /// Wake a device
    pub fn wake(&self, device_index: usize) -> KernelResult<()> {
        if device_index >= super::MAX_POWER_DEVICES {
            return Err(KernelError::InvalidArgument);
        }

        let state = &self.device_states[device_index];
        state.sleep_level.store(SleepLevel::Active as u8, Ordering::Release);
        state.is_sleeping.store(false, Ordering::Release);
        state.reset_idle();
        self.wake_events.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    /// Mark a device as critical (never sleeps)
    pub fn set_critical(&self, device_index: usize, critical: bool) -> KernelResult<()> {
        if device_index >= super::MAX_POWER_DEVICES {
            return Err(KernelError::InvalidArgument);
        }
        self.device_states[device_index].is_critical.store(critical, Ordering::Release);
        if critical && self.device_states[device_index].is_sleeping() {
            self.wake(device_index)?;
        }
        Ok(())
    }

    /// Get statistics
    pub fn stats(&self) -> (u32, u32) {
        (
            self.sleep_events.load(Ordering::Acquire),
            self.wake_events.load(Ordering::Acquire),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sleep_device() {
        let ctrl = DevicePowerController::new();
        let result = ctrl.sleep(0, SleepLevel::LightSleep);
        assert!(result.is_ok());
        assert!(ctrl.get_state(0).unwrap().is_sleeping());
    }

    #[test]
    fn test_critical_device_never_sleeps() {
        let ctrl = DevicePowerController::new();
        ctrl.set_critical(0, true).unwrap();
        let result = ctrl.sleep(0, SleepLevel::DeepSleep);
        assert!(result.is_ok());
        assert!(!ctrl.get_state(0).unwrap().is_sleeping());
    }

    #[test]
    fn test_wake_device() {
        let ctrl = DevicePowerController::new();
        ctrl.sleep(0, SleepLevel::DeepSleep).unwrap();
        ctrl.wake(0).unwrap();
        assert!(!ctrl.get_state(0).unwrap().is_sleeping());
        assert_eq!(ctrl.get_state(0).unwrap().sleep_level(), SleepLevel::Active);
    }
}