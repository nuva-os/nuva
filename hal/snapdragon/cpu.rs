/*
 * Nuva OS - HAL - Snapdragon
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



use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// Snapdragon 8 Gen 4 register addresses
pub mod regs {
    /// Base address
    pub const QCOM_BASE: u64 = 0x0A00_0000;

    /// CPU frequency control
    pub const CPU_FREQ_BASE: u64 = 0x0A10_0000;

    /// Oryon core frequency register
    pub const ORYON_FREQ_REG: u64 = CPU_FREQ_BASE + 0x0000;

    /// Cortex-A720 frequency register
    pub const A720_FREQ_REG: u64 = CPU_FREQ_BASE + 0x1000;

    /// Power management
    pub const PMIC_BASE: u64 = 0x0A20_0000;
    pub const PMIC_CTRL: u64 = PMIC_BASE + 0x0000;

    /// Thermal management
    pub const THERMAL_BASE: u64 = 0x0A30_0000;
    pub const THERMAL_STATUS: u64 = THERMAL_BASE + 0x0000;
    pub const THERMAL_LIMIT: u64 = THERMAL_BASE + 0x0004;

    /// Clock control
    pub const CLOCK_BASE: u64 = 0x0A40_0000;
    pub const CLOCK_CTRL: u64 = CLOCK_BASE + 0x0000;

    /// Voltage control
    pub const VOLTAGE_BASE: u64 = 0x0A50_0000;
    pub const VOLTAGE_CTRL: u64 = VOLTAGE_BASE + 0x0000;
}

/// CPU core type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoreType {
    /// Oryon super core
    Oryon,
    /// Cortex-A720 big core
    CortexA720,
}

/// CPU core
pub struct CpuCore {
    /// Core ID
    pub id: u32,
    /// Core type
    pub core_type: CoreType,
    /// Current frequency (MHz)
    pub freq_mhz: AtomicU32,
    /// Minimum frequency (MHz)
    pub min_freq_mhz: u32,
    /// Maximum frequency (MHz)
    pub max_freq_mhz: u32,
    /// If online
    pub online: AtomicU32,
    /// Current voltage (mV)
    pub voltage_mv: AtomicU32,
    /// Temperature (millidegrees)
    pub temp_mc: AtomicU32,
}

impl CpuCore {
    pub fn new(id: u32, core_type: CoreType) -> Self {
        let (min_freq, max_freq) = match core_type {
            CoreType::Oryon => (800, 4090),
            CoreType::CortexA720 => (600, 3200),
        };

        CpuCore {
            id,
            core_type,
            freq_mhz: AtomicU32::new(min_freq),
            min_freq_mhz: min_freq,
            max_freq_mhz: max_freq,
            online: AtomicU32::new(1),
            voltage_mv: AtomicU32::new(800),
            temp_mc: AtomicU32::new(25000),
        }
    }

    /// Set frequency
    pub fn set_freq(&self, freq_mhz: u32) -> bool {
        if freq_mhz < self.min_freq_mhz || freq_mhz > self.max_freq_mhz {
            return false;
        }

        // TODO: Write frequency register
        self.freq_mhz.store(freq_mhz, Ordering::Release);
        true
    }

    /// Get frequency
    pub fn get_freq(&self) -> u32 {
        self.freq_mhz.load(Ordering::Acquire)
    }

    /// Power on
    pub fn power_on(&self) {
        self.online.store(1, Ordering::Release);
    }

    /// Power off
    pub fn power_off(&self) {
        self.online.store(0, Ordering::Release);
    }

    /// If online
    pub fn is_online(&self) -> bool {
        self.online.load(Ordering::Acquire) != 0
    }
}

/// Snapdragon 8 Gen 4 CPU HAL
pub struct SnapdragonCpuHal {
    /// CPU cores
    cores: [CpuCore; 8],
    /// Total temperature
    total_temp_mc: AtomicU32,
    /// Total power consumption (mW)
    total_power_mw: AtomicU32,
}

impl SnapdragonCpuHal {
    pub fn new() -> Self {
        SnapdragonCpuHal {
            cores: [
                CpuCore::new(0, CoreType::Oryon),
                CpuCore::new(1, CoreType::Oryon),
                CpuCore::new(2, CoreType::CortexA720),
                CpuCore::new(3, CoreType::CortexA720),
                CpuCore::new(4, CoreType::CortexA720),
                CpuCore::new(5, CoreType::CortexA720),
                CpuCore::new(6, CoreType::CortexA720),
                CpuCore::new(7, CoreType::CortexA720),
            ],
            total_temp_mc: AtomicU32::new(25000),
            total_power_mw: AtomicU32::new(0),
        }
    }

    /// Initialize
    pub fn init(&mut self) {
        log_info!("Snapdragon 8 Gen 4 CPU HAL initialized");
        log_info!("  Cores: 2x Oryon + 6x Cortex-A720");
        log_info!("  Max freq: 4090 MHz (Oryon), 3200 MHz (A720)");
    }

    /// Get core
    pub fn get_core(&self, id: u32) -> Option<&CpuCore> {
        if id < 8 {
            Some(&self.cores[id as usize])
        } else {
            None
        }
    }

    /// Get online core count
    pub fn get_online_count(&self) -> u32 {
        self.cores.iter()
            .filter(|c| c.is_online())
            .count() as u32
    }

    /// DVFS adjustment
    pub fn dvfs_update(&mut self, load: u32) {
        // Adjust frequency based on load
        for core in self.cores.iter() {
            if !core.is_online() {
                continue;
            }

            let target_freq = if load > 80 {
                core.max_freq_mhz
            } else if load > 50 {
                (core.max_freq_mhz + core.min_freq_mhz) / 2
            } else if load > 20 {
                core.min_freq_mhz + (core.max_freq_mhz - core.min_freq_mhz) / 4
            } else {
                core.min_freq_mhz
            };

            core.set_freq(target_freq);
        }
    }

    /// Thermal management
    pub fn thermal_update(&mut self) {
        // Read temperature
        let temp = self.read_thermal();
        self.total_temp_mc.store(temp, Ordering::Release);

        // Overheat protection
        if temp > 85000 {  // 85°C
            // Reduce frequency
            for core in self.cores.iter() {
                let current = core.get_freq();
                if current > core.min_freq_mhz {
                    core.set_freq(current - 200);
                }
            }
        }
    }

    /// Read temperature
    fn read_thermal(&self) -> u32 {
        // TODO: Read temperature sensor
        25000
    }

    /// Get temperature
    pub fn get_temp(&self) -> u32 {
        self.total_temp_mc.load(Ordering::Acquire)
    }

    /// Get power consumption
    pub fn get_power(&self) -> u32 {
        self.total_power_mw.load(Ordering::Acquire)
    }
}

/// Global CPU HAL
static mut CPU_HAL: Option<SnapdragonCpuHal> = None;

pub fn get_cpu_hal() -> &'static mut SnapdragonCpuHal {
    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe {
        if CPU_HAL.is_none() {
            CPU_HAL = Some(SnapdragonCpuHal::new());
        }
        CPU_HAL.as_mut().unwrap()
    }
}

pub fn init_cpu_hal() {
    let hal = get_cpu_hal();
    hal.init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_type() {
        assert_eq!(CoreType::Oryon as i32, 0);
        assert_eq!(CoreType::CortexA720 as i32, 1);
    }

    #[test]
    fn test_cpu_core() {
        let core = CpuCore::new(0, CoreType::Oryon);
        assert_eq!(core.id, 0);
        assert_eq!(core.min_freq_mhz, 800);
        assert_eq!(core.max_freq_mhz, 4090);
        assert!(core.is_online());
    }

    #[test]
    fn test_snapdragon_cpu_hal() {
        let hal = SnapdragonCpuHal::new();
        assert_eq!(hal.get_online_count(), 8);
        assert!(hal.get_core(0).is_some());
        assert!(hal.get_core(8).is_none());
    }

    #[test]
    fn test_dvfs_update() {
        let mut hal = SnapdragonCpuHal::new();
        hal.dvfs_update(90);  // High load
        let core = hal.get_core(0).unwrap();
        assert_eq!(core.get_freq(), core.max_freq_mhz);
    }
}
