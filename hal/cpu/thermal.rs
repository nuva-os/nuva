/*
 * Nuva OS - HAL - Cpu
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



use core::sync::atomic::{AtomicI32, AtomicU32, Ordering};
use core::ptr::{read_volatile, write_volatile};
use crate::{pr_debug, pr_info, pr_warn};

// ============================================================================
// Temperature Sensor Hardware Register Definitions
// ============================================================================

/// Temperature sensor base address
const TSENSOR_HW_BASE: u64 = 0xF5A0_0000;

/// Temperature sensor register offsets
const TSENSOR_TEMP_RAW: u64 = 0x0000;      // Raw temperature value
const TSENSOR_TEMP_CALIB: u64 = 0x0004;    // Calibration data
const TSENSOR_THRESHOLD: u64 = 0x0008;     // Temperature threshold
const TSENSOR_ALARM: u64 = 0x000C;         // Alarm status
const TSENSOR_CTRL: u64 = 0x0010;          // Control register

/// Cooling device register base address
const COOLING_DEV_BASE: u64 = 0xF5A1_0000;

/// Cooling device register offsets
const COOLING_LEVEL: u64 = 0x0000;         // Throttle level
const COOLING_STATE: u64 = 0x0004;         // Status register
const COOLING_TARGET: u64 = 0x0008;        // Target value

/// Temperature sampling interval (milliseconds)
const THERMAL_SAMPLE_INTERVAL_MS: u32 = 100;
/// Temperature hysteresis value (millidegrees)
const THERMAL_HYSTERESIS: i32 = 2000;

/// Thermal management state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThermalState {
    /// Normal
    Normal = 0,
    /// Warning
    Warning = 1,
    /// Light throttle
    Throttle1 = 2,
    /// Medium throttle
    Throttle2 = 3,
    /// Heavy throttle
    Throttle3 = 4,
    /// Critical
    Critical = 5,
}

/// Temperature thresholds
pub struct ThermalThresholds {
    /// Warning temperature (millidegrees)
    pub warning_temp: i32,
    /// Light throttle temperature
    pub throttle1_temp: i32,
    /// Medium throttle temperature
    pub throttle2_temp: i32,
    /// Heavy throttle temperature
    pub throttle3_temp: i32,
    /// Critical temperature
    pub critical_temp: i32,
}

impl ThermalThresholds {
    pub const fn new() -> Self {
        ThermalThresholds {
            warning_temp: 75000,    // 75°C
            throttle1_temp: 85000,  // 85°C
            throttle2_temp: 95000,  // 95°C
            throttle3_temp: 105000, // 105°C
            critical_temp: 115000, // 115°C
        }
    }
}

/// Thermal management zone
pub struct ThermalZone {
    /// Zone name
    pub name: &'static str,
    /// Zone ID
    pub zone_id: u32,
    /// Current temperature (millidegrees)
    pub temperature: AtomicI32,
    /// Thresholds
    pub thresholds: ThermalThresholds,
    /// Current state
    pub state: AtomicU32,
    /// Throttle level
    pub throttle_level: AtomicU32,
    /// Cooling devices
    pub cooling_devices: &'static [u32],
}

impl ThermalZone {
    pub const fn new(
        name: &'static str,
        zone_id: u32,
        thresholds: ThermalThresholds,
        cooling_devices: &'static [u32],
    ) -> Self {
        ThermalZone {
            name,
            zone_id,
            temperature: AtomicI32::new(0),
            thresholds,
            state: AtomicU32::new(ThermalState::Normal as u32),
            throttle_level: AtomicU32::new(0),
            cooling_devices,
        }
    }

    // ========================================================================
    // Register operations
    // ========================================================================

    /// Get temperature sensor register address
    #[inline]
    fn get_tsensor_reg_addr(offset: u64) -> u64 {
        TSENSOR_HW_BASE + offset
    }

    /// Read temperature sensor register
    #[inline]
    unsafe fn read_tsensor_reg(offset: u64) -> u32 {
        read_volatile(Self::get_tsensor_reg_addr(offset) as *const u32)
    }

    /// Write temperature sensor register
    #[inline]
    unsafe fn write_tsensor_reg(offset: u64, value: u32) {
        write_volatile(Self::get_tsensor_reg_addr(offset) as *mut u32, value);
    }

    /// Read temperature from hardware (millidegrees)
    pub fn read_temp_from_hw(&self) -> i32 {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let base_offset = self.zone_id as u64 * 0x1000;

            // Read raw temperature value
            let raw_temp = Self::read_tsensor_reg(base_offset + TSENSOR_TEMP_RAW);

            // Read calibration data
            let calib = Self::read_tsensor_reg(base_offset + TSENSOR_TEMP_CALIB);

            // Temperature conversion formula: temp = (raw - calib) * scale + base
            // Assume scale = 100, base = 25000 (25°C)
            let temp = if raw_temp > calib {
                ((raw_temp - calib) as i32) * 100 + 25_000
            } else {
                25_000 - ((calib - raw_temp) as i32) * 100
            };

            temp
        }
    }

    /// Update temperature (read from hardware)
    pub fn update_temperature_from_hw(&self) {
        let temp = self.read_temp_from_hw();
        self.update_temperature(temp);
    }

    /// Update temperature
    pub fn update_temperature(&self, temp: i32) {
        self.temperature.store(temp, Ordering::Release);

        // Update state based on temperature (with hysteresis)
        let current_state = self.get_state();
        let new_state = self.calculate_state_with_hysteresis(temp, current_state);

        let old_state = self.state.swap(new_state as u32, Ordering::AcqRel);

        if old_state != new_state as u32 {
            log_info!("Thermal zone {}: {} -> {:?} ({}°C)",
                self.name, old_state, new_state, temp / 1000);

            // Apply throttling
            self.apply_throttling(new_state);
        }
    }

    /// State calculation with hysteresis
    fn calculate_state_with_hysteresis(&self, temp: i32, current_state: ThermalState) -> ThermalState {
        // Calculate new state
        let new_state = if temp >= self.thresholds.critical_temp {
            ThermalState::Critical
        } else if temp >= self.thresholds.throttle3_temp {
            ThermalState::Throttle3
        } else if temp >= self.thresholds.throttle2_temp {
            ThermalState::Throttle2
        } else if temp >= self.thresholds.throttle1_temp {
            ThermalState::Throttle1
        } else if temp >= self.thresholds.warning_temp {
            ThermalState::Warning
        } else {
            ThermalState::Normal
        };

        // Apply hysteresis: when temperature decreases, need to be below threshold-hysteresis to switch
        if (new_state as u32) < current_state as u32 {
            // Temperature decreasing, check hysteresis
            let threshold = match current_state {
                ThermalState::Critical => self.thresholds.critical_temp - THERMAL_HYSTERESIS,
                ThermalState::Throttle3 => self.thresholds.throttle3_temp - THERMAL_HYSTERESIS,
                ThermalState::Throttle2 => self.thresholds.throttle2_temp - THERMAL_HYSTERESIS,
                ThermalState::Throttle1 => self.thresholds.throttle1_temp - THERMAL_HYSTERESIS,
                ThermalState::Warning => self.thresholds.warning_temp - THERMAL_HYSTERESIS,
                ThermalState::Normal => 0,
            };

            if temp < threshold {
                new_state
            } else {
                current_state
            }
        } else {
            new_state
        }
    }

    /// Apply throttling
    fn apply_throttling(&self, state: ThermalState) {
        let level = match state {
            ThermalState::Normal => 0,
            ThermalState::Warning => 0,
            ThermalState::Throttle1 => 1,
            ThermalState::Throttle2 => 2,
            ThermalState::Throttle3 => 3,
            ThermalState::Critical => 4,
        };

        self.throttle_level.store(level, Ordering::Release);

        // Notify cooling devices
        for &device_id in self.cooling_devices {
            if let Some(device) = get_cooling_device(device_id) {
                let _ = device.set_level(level);
            }
        }

        // Critical state: set hardware alarm
        if state == ThermalState::Critical {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                let base_offset = self.zone_id as u64 * 0x1000;
                Self::write_tsensor_reg(base_offset + TSENSOR_ALARM, 1);
            }
            log_warn!("Thermal zone {}: CRITICAL temperature!", self.name);
        }
    }

    /// Get temperature
    pub fn get_temperature(&self) -> i32 {
        self.temperature.load(Ordering::Acquire)
    }

    /// Get state
    pub fn get_state(&self) -> ThermalState {
        match self.state.load(Ordering::Acquire) {
            0 => ThermalState::Normal,
            1 => ThermalState::Warning,
            2 => ThermalState::Throttle1,
            3 => ThermalState::Throttle2,
            4 => ThermalState::Throttle3,
            5 => ThermalState::Critical,
            _ => ThermalState::Normal,
        }
    }

    /// Set temperature thresholds to hardware
    pub fn set_thresholds_to_hw(&self) {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let base_offset = self.zone_id as u64 * 0x1000;
            // Write critical temperature threshold
            Self::write_tsensor_reg(base_offset + TSENSOR_THRESHOLD,
                self.thresholds.critical_temp as u32);
        }
    }
}

/// Cooling device type
#[derive(Debug, Clone, Copy)]
pub enum CoolingDeviceType {
    /// CPU frequency limit
    Cpufreq = 0,
    /// GPU frequency limit
    Gpufreq = 1,
    /// Fan
    Fan = 2,
    /// CPU hotplug
    CpuHotplug = 3,
}

/// Cooling device
pub struct CoolingDevice {
    /// Device name
    pub name: &'static str,
    /// Device ID
    pub device_id: u32,
    /// Device type
    pub device_type: CoolingDeviceType,
    /// Max throttle level
    pub max_level: u32,
    /// Current throttle level
    pub current_level: AtomicU32,
}

impl CoolingDevice {
    pub const fn new(
        name: &'static str,
        device_id: u32,
        device_type: CoolingDeviceType,
        max_level: u32,
    ) -> Self {
        CoolingDevice {
            name,
            device_id,
            device_type,
            max_level,
            current_level: AtomicU32::new(0),
        }
    }

    // ========================================================================
    // Register operations
    // ========================================================================

    /// Get register address
    #[inline]
    fn get_reg_addr(offset: u64) -> u64 {
        COOLING_DEV_BASE + offset
    }

    /// Read register
    #[inline]
    unsafe fn read_reg(offset: u64) -> u32 {
        read_volatile(Self::get_reg_addr(offset) as *const u32)
    }

    /// Write register
    #[inline]
    unsafe fn write_reg(offset: u64, value: u32) {
        write_volatile(Self::get_reg_addr(offset) as *mut u32, value);
    }

    /// Set throttle level
    pub fn set_level(&self, level: u32) -> i32 {
        if level > self.max_level {
            return -1;
        }

        let old_level = self.current_level.swap(level, Ordering::AcqRel);

        if old_level != level {
            log_debug!("Cooling device {}: level {} -> {}",
                self.name, old_level, level);

            // Apply throttling
            self.apply_level(level);
        }

        0
    }

    /// Apply throttle level (actual hardware operation)
    fn apply_level(&self, level: u32) {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            // Write throttle level to hardware
            let base_offset = self.device_id as u64 * 0x1000;
            Self::write_reg(base_offset + COOLING_LEVEL, level);
        }

        match self.device_type {
            CoolingDeviceType::Cpufreq => {
                // CPU frequency limit
                // Reduce 20% frequency limit per level
                let max_freq_percent = 100 - (level * 20).min(80);
                // SAFETY: unsafe block required for low-level memory or hardware access
                unsafe {
                    let base_offset = self.device_id as u64 * 0x1000;
                    Self::write_reg(base_offset + COOLING_TARGET, max_freq_percent);
                }
                log_debug!("CPU max freq: {}%", max_freq_percent);
            }
            CoolingDeviceType::Gpufreq => {
                // GPU frequency limit
                let max_freq_percent = 100 - (level * 20).min(80);
                // SAFETY: unsafe block required for low-level memory or hardware access
                unsafe {
                    let base_offset = self.device_id as u64 * 0x1000;
                    Self::write_reg(base_offset + COOLING_TARGET, max_freq_percent);
                }
                log_debug!("GPU max freq: {}%", max_freq_percent);
            }
            CoolingDeviceType::Fan => {
                // Fan speed control
                // Increase 25% speed per level
                let fan_speed = if level == 0 { 0 } else { 25 + level * 25 };
                // SAFETY: unsafe block required for low-level memory or hardware access
                unsafe {
                    let base_offset = self.device_id as u64 * 0x1000;
                    Self::write_reg(base_offset + COOLING_TARGET, fan_speed.min(100));
                }
                log_debug!("Fan speed: {}%", fan_speed.min(100));
            }
            CoolingDeviceType::CpuHotplug => {
                // CPU hotplug
                // Higher level, more CPUs offline
                let cpus_to_offline = level.min(4); // Offline at most 4 CPUs
                // SAFETY: unsafe block required for low-level memory or hardware access
                unsafe {
                    let base_offset = self.device_id as u64 * 0x1000;
                    Self::write_reg(base_offset + COOLING_TARGET, cpus_to_offline);
                }
                log_debug!("CPUs to offline: {}", cpus_to_offline);
            }
        }
    }

    /// Get current state
    pub fn get_state(&self) -> u32 {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let base_offset = self.device_id as u64 * 0x1000;
            Self::read_reg(base_offset + COOLING_STATE)
        }
    }
}

/// CPU thermal zone cooling devices
static CPU_COOLING_DEVICES: [u32; 2] = [0, 1];  // CPU freq, GPU freq

/// CPU thermal zone
static CPU_THERMAL_ZONE: ThermalZone = ThermalZoneType::new(
    "cpu",
    0,
    ThermalThresholds::new(),
    &CPU_COOLING_DEVICES,
);

/// GPU thermal zone
static GPU_THERMAL_ZONE: ThermalZone = ThermalZoneType::new(
    "gpu",
    1,
    ThermalThresholds::new(),
    &[1],  // GPU freq
);

/// Cooling device array
static COOLING_DEVICES: [CoolingDevice; 2] = [
    CoolingDevice::new("cpufreq", 0, CoolingDeviceType::Cpufreq, 4),
    CoolingDevice::new("gpufreq", 1, CoolingDeviceType::Gpufreq, 4),
];

/// Get thermal zone
pub fn get_thermal_zone(zone_id: u32) -> Option<&'static ThermalZone> {
    match zone_id {
        0 => Some(&CPU_THERMAL_ZONE),
        1 => Some(&GPU_THERMAL_ZONE),
        _ => None,
    }
}

/// Get cooling device
pub fn get_cooling_device(device_id: u32) -> Option<&'static CoolingDevice> {
    if (device_id as usize) < COOLING_DEVICES.len() {
        Some(&COOLING_DEVICES[device_id as usize])
    } else {
        None
    }
}

/// Initialize thermal management
pub fn init_thermal() {
    log_info!("Thermal management initialized");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thermal() {
        let zone = get_thermal_zone(0).unwrap();
        zone.update_temperature(90000);
    }
}
