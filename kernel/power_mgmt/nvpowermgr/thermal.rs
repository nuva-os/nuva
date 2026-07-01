/*
 * Nuva OS - Kernel - PowerMgmt - Nvpowermgr - Thermal
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
 * Nuva OS - Kernel - NvPowerMgr Thermal Monitor
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Per-device temperature monitoring with proactive
 * throttling at 85% threshold and conservative fallback
 * on sensor failure.
 */

use core::sync::atomic::{AtomicU32, AtomicU8, AtomicBool, Ordering};

use crate::kernel::error::KernelResult;

/// Default throttle threshold (85 degrees C)
pub const DEFAULT_THROTTLE_THRESHOLD_C: u32 = 85;

/// Default critical threshold (100 degrees C)
pub const DEFAULT_CRITICAL_THRESHOLD_C: u32 = 100;

/// Temperature sensor state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SensorState {
    /// Sensor operating normally
    Healthy = 0,
    /// Sensor degraded (readings may be inaccurate)
    Degraded = 1,
    /// Sensor failed (using conservative policy)
    Failed = 2,
}

/// ThermalMonitor: per-device temperature monitoring
///
/// When temperature exceeds 85% of critical threshold,
/// proactively reduces power to avoid thermal throttling.
/// On sensor failure, applies conservative policy
/// (restrict max frequency).
pub struct ThermalMonitor {
    /// Per-device temperatures (degrees C, scaled by 10)
    temperatures: [AtomicU32; super::MAX_POWER_DEVICES],
    /// Throttle threshold (degrees C)
    throttle_threshold_c: AtomicU32,
    /// Critical threshold (degrees C)
    critical_threshold_c: AtomicU32,
    /// Per-device sensor health
    sensor_health: [AtomicU8; super::MAX_POWER_DEVICES],
    /// Thermal throttle events
    throttle_events: AtomicU32,
    /// Whether any device is throttled
    any_throttled: AtomicBool,
}

impl ThermalMonitor {
    /// Create a new thermal monitor
    pub const fn new() -> Self {
        ThermalMonitor {
            temperatures: [
                AtomicU32::new(250), AtomicU32::new(250),
                AtomicU32::new(250), AtomicU32::new(250),
                AtomicU32::new(250), AtomicU32::new(250),
                AtomicU32::new(250), AtomicU32::new(250),
                AtomicU32::new(250), AtomicU32::new(250),
                AtomicU32::new(250), AtomicU32::new(250),
                AtomicU32::new(250), AtomicU32::new(250),
                AtomicU32::new(250), AtomicU32::new(250),
            ],
            throttle_threshold_c: AtomicU32::new(DEFAULT_THROTTLE_THRESHOLD_C),
            critical_threshold_c: AtomicU32::new(DEFAULT_CRITICAL_THRESHOLD_C),
            sensor_health: [
                AtomicU8::new(0), AtomicU8::new(0), AtomicU8::new(0), AtomicU8::new(0),
                AtomicU8::new(0), AtomicU8::new(0), AtomicU8::new(0), AtomicU8::new(0),
                AtomicU8::new(0), AtomicU8::new(0), AtomicU8::new(0), AtomicU8::new(0),
                AtomicU8::new(0), AtomicU8::new(0), AtomicU8::new(0), AtomicU8::new(0),
            ],
            throttle_events: AtomicU32::new(0),
            any_throttled: AtomicBool::new(false),
        }
    }

    /// Update temperature for a device
    pub fn update_temperature(&self, device_index: usize, temp_c: u32) -> KernelResult<()> {
        if device_index >= super::MAX_POWER_DEVICES {
            return Err(crate::kernel::error::KernelError::InvalidArgument);
        }

        // Validate temperature range (0-150 C)
        if temp_c > 1500 {
            self.sensor_health[device_index].store(SensorState::Failed as u8, Ordering::Release);
            return Ok(());
        }

        self.temperatures[device_index].store(temp_c, Ordering::Release);
        self.sensor_health[device_index].store(SensorState::Healthy as u8, Ordering::Release);

        if self.needs_throttle(temp_c) {
            self.throttle_events.fetch_add(1, Ordering::Relaxed);
            self.any_throttled.store(true, Ordering::Release);
        }

        Ok(())
    }

    /// Get temperature for a device (degrees C)
    pub fn temperature_c(&self, device_index: usize) -> u32 {
        if device_index < super::MAX_POWER_DEVICES {
            self.temperatures[device_index].load(Ordering::Acquire) / 10
        } else {
            0
        }
    }

    /// Get sensor state for a device
    pub fn sensor_state(&self, device_index: usize) -> SensorState {
        if device_index < super::MAX_POWER_DEVICES {
            match self.sensor_health[device_index].load(Ordering::Acquire) {
                0 => SensorState::Healthy,
                1 => SensorState::Degraded,
                _ => SensorState::Failed,
            }
        } else {
            SensorState::Failed
        }
    }

    /// Check if a device needs throttling
    pub fn needs_throttle(&self, temp_c: u32) -> bool {
        let threshold = self.throttle_threshold_c.load(Ordering::Acquire);
        temp_c / 10 >= threshold
    }

    /// Check if a device is at critical temperature
    pub fn is_critical(&self, temp_c: u32) -> bool {
        let threshold = self.critical_threshold_c.load(Ordering::Acquire);
        temp_c / 10 >= threshold
    }

    /// Get throttle threshold
    #[inline(always)]
    pub fn throttle_threshold_c(&self) -> u32 {
        self.throttle_threshold_c.load(Ordering::Acquire)
    }

    /// Set throttle threshold
    pub fn set_throttle_threshold_c(&self, threshold: u32) {
        self.throttle_threshold_c.store(threshold, Ordering::Release);
    }

    /// Get throttle event count
    pub fn throttle_events(&self) -> u32 {
        self.throttle_events.load(Ordering::Acquire)
    }

    /// Check if any device is throttled
    pub fn any_throttled(&self) -> bool {
        self.any_throttled.load(Ordering::Acquire)
    }

    /// Mark sensor as degraded (use conservative policy)
    pub fn mark_sensor_degraded(&self, device_index: usize) {
        if device_index < super::MAX_POWER_DEVICES {
            self.sensor_health[device_index].store(SensorState::Degraded as u8, Ordering::Release);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
use core::sync::atomic::AtomicU8;
use crate::kernel::error::KernelError;

    #[test]
    fn test_normal_temperature() {
        let mon = ThermalMonitor::new();
        mon.update_temperature(0, 450).unwrap();
        assert_eq!(mon.temperature_c(0), 45);
        assert!(!mon.needs_throttle(450));
    }

    #[test]
    fn test_throttle_temperature() {
        let mon = ThermalMonitor::new();
        mon.update_temperature(0, 870).unwrap();
        assert!(mon.needs_throttle(870));
        assert!(mon.throttle_events() > 0);
    }

    #[test]
    fn test_critical_temperature() {
        let mon = ThermalMonitor::new();
        assert!(mon.is_critical(1050));
    }

    #[test]
    fn test_sensor_failure() {
        let mon = ThermalMonitor::new();
        mon.update_temperature(0, 2000).unwrap();
        assert_eq!(mon.sensor_state(0), SensorState::Failed);
    }

    #[test]
    fn test_mark_degraded() {
        let mon = ThermalMonitor::new();
        mon.mark_sensor_degraded(0);
        assert_eq!(mon.sensor_state(0), SensorState::Degraded);
    }
}