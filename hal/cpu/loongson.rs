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



use super::{CpuInfo, CpuState, CpuType, CpuHalOps};
use core::ptr::{read_volatile, write_volatile};
use crate::{pr_debug, pr_info, pr_warn};

// ============================================================================
// Loongson CPU Register Address Definitions
// ============================================================================

/// CPU frequency control register base address
const CPU_FREQ_BASE: u64 = 0x1FE0_0000;

/// CPU voltage control register base address
const CPU_VOLTAGE_BASE: u64 = 0x1FE1_0000;

/// Temperature sensor register base address
const TSENSOR_BASE: u64 = 0x1FE2_0000;

/// CPU status register base address
const CPU_STATE_BASE: u64 = 0x1FE3_0000;

// Frequency register offsets
const FREQ_TARGET_OFFSET: u64 = 0x0000;
const FREQ_CURRENT_OFFSET: u64 = 0x0004;
const FREQ_ENABLE_OFFSET: u64 = 0x0008;
const FREQ_DIVIDER_OFFSET: u64 = 0x000C;

// Voltage register offsets
const VOLTAGE_TARGET_OFFSET: u64 = 0x0000;
const VOLTAGE_CURRENT_OFFSET: u64 = 0x0004;

// Temperature sensor register offsets
const TSENSOR_TEMP_OFFSET: u64 = 0x0000;
const TSENSOR_ENABLE_OFFSET: u64 = 0x0004;

// CPU status register offsets
const STATE_POWER_OFFSET: u64 = 0x0000;
const STATE_IDLE_OFFSET: u64 = 0x0004;

// Register operation delays (microseconds)
const REG_DELAY_US: u32 = 10;
const VOLTAGE_SETTLE_US: u32 = 100;
const FREQ_SETTLE_US: u32 = 50;

/// Loongson chip series
#[derive(Debug, Clone, Copy)]
pub enum LoongsonSeries {
    /// Loongson 3A6000 series (desktop)
    Loongson3A6000,
    /// Loongson 3C6000 series (server)
    Loongson3C6000,
}

/// Loongson CPU configuration
pub struct LoongsonConfig {
    /// Number of CPU cores
    pub num_cores: u32,
    /// Minimum frequency
    pub min_freq: u64,
    /// Maximum frequency
    pub max_freq: u64,
    /// L3 cache size (KB)
    pub l3_cache_kb: u32,
    /// Supported instruction extensions
    pub extensions: LoongArchExtensions,
    /// Chip series
    pub series: LoongsonSeries,
}

/// LoongArch instruction extensions
#[derive(Debug, Clone, Copy)]
pub struct LoongArchExtensions {
    /// LSX: 128-bit SIMD extension
    pub lsx: bool,
    /// LASX: 256-bit SIMD extension
    pub lasx: bool,
    /// LVZ: Virtualization extension
    pub lvz: bool,
    /// LBT: Binary translation extension
    pub lbt: bool,
}

impl Default for LoongArchExtensions {
    fn default() -> Self {
        Self {
            lsx: true,
            lasx: true,
            lvz: true,
            lbt: true,
        }
    }
}

impl LoongsonConfig {
    /// Create Loongson 3A6000 configuration
    pub const fn loongson3a6000() -> Self {
        LoongsonConfig {
            num_cores: 4,
            min_freq: 800_000_000,    // 800 MHz
            max_freq: 2_500_000_000,  // 2.5 GHz
            l3_cache_kb: 16384,       // 16 MB
            extensions: LoongArchExtensions {
                lsx: true,
                lasx: true,
                lvz: true,
                lbt: true,
            },
            series: LoongsonSeries::Loongson3A6000,
        }
    }

    /// Create Loongson 3C6000 configuration (server)
    pub const fn loongson3c6000() -> Self {
        LoongsonConfig {
            num_cores: 16,
            min_freq: 800_000_000,    // 800 MHz
            max_freq: 2_600_000_000,  // 2.6 GHz
            l3_cache_kb: 32768,       // 32 MB
            extensions: LoongArchExtensions {
                lsx: true,
                lasx: true,
                lvz: true,
                lbt: true,
            },
            series: LoongsonSeries::Loongson3C6000,
        }
    }

    /// Create default configuration
    pub const fn new() -> Self {
        Self::loongson3a6000()
    }
}

/// Loongson CPU HAL
pub struct LoongsonCpuHal {
    config: LoongsonConfig,
}

impl LoongsonCpuHal {
    /// Create Loongson 3A6000 HAL
    pub const fn loongson3a6000() -> Self {
        LoongsonCpuHal {
            config: LoongsonConfig::loongson3a6000(),
        }
    }

    /// Create Loongson 3C6000 HAL
    pub const fn loongson3c6000() -> Self {
        LoongsonCpuHal {
            config: LoongsonConfig::loongson3c6000(),
        }
    }

    /// Create default HAL
    pub const fn new() -> Self {
        Self::loongson3a6000()
    }

    // ========================================================================
    // Register read/write functions
    // ========================================================================

    /// Read 32-bit register
    #[inline]
    unsafe fn read_reg(addr: u64) -> u32 {
        read_volatile(addr as *const u32)
    }

    /// Write 32-bit register
    #[inline]
    unsafe fn write_reg(addr: u64, value: u32) {
        write_volatile(addr as *mut u32, value);
    }

    /// Microsecond delay
    #[inline]
    fn udelay(us: u32) {
        let cycles = us * 100;
        let mut _dummy: u32 = 0;
        for _ in 0..cycles {
            core::hint::spin_loop();
            _dummy = _dummy.wrapping_add(1);
        }
    }

    // ========================================================================
    // CPU frequency control
    // ========================================================================

    /// Get CPU frequency register address
    #[inline]
    pub fn get_freq_reg_addr(&self, cpu_id: u32, offset: u64) -> u64 {
        CPU_FREQ_BASE + (cpu_id as u64 * 0x1000) + offset
    }

    /// Read actual frequency from hardware
    pub fn cpu_get_freq_hw(&self, cpu_id: u32) -> u64 {
        // SAFETY: Reading the current frequency register at CPU_FREQ_BASE + cpu_id*0x1000
        // + FREQ_CURRENT_OFFSET, which is a valid Loongson MMIO register for frequency readback.
        unsafe {
            let addr = self.get_freq_reg_addr(cpu_id, FREQ_CURRENT_OFFSET);
            let freq_khz = Self::read_reg(addr);
            (freq_khz as u64) * 1000
        }
    }

    /// Write target frequency to hardware
    pub fn cpu_set_freq_hw(&mut self, cpu_id: u32, freq: u64) -> i32 {
        let info = self.get_cpu_info(cpu_id);

        if freq < info.min_freq || freq > info.max_freq {
            log_warn!("CPU {}: Invalid frequency {} MHz", cpu_id, freq / 1_000_000);
            return -1;
        }

        // SAFETY: Writing to Loongson CPU frequency control MMIO registers at
        // CPU_FREQ_BASE + cpu_id*0x1000: divider, target frequency, and enable
        // registers are valid hardware registers for DVFS on Loongson 3A6000/3C6000.
        unsafe {
            let ref_clk: u64 = 33_000_000; // Loongson reference clock 33 MHz
            let divider = (ref_clk * 1000) / freq;

            let div_addr = self.get_freq_reg_addr(cpu_id, FREQ_DIVIDER_OFFSET);
            Self::write_reg(div_addr, divider as u32);

            let target_addr = self.get_freq_reg_addr(cpu_id, FREQ_TARGET_OFFSET);
            Self::write_reg(target_addr, (freq / 1000) as u32);

            Self::udelay(FREQ_SETTLE_US);

            let enable_addr = self.get_freq_reg_addr(cpu_id, FREQ_ENABLE_OFFSET);
            Self::write_reg(enable_addr, 1);
        }

        log_debug!("CPU {} frequency set to {} MHz", cpu_id, freq / 1_000_000);
        0
    }

    // ========================================================================
    // CPU voltage control
    // ========================================================================

    /// Get CPU voltage register address
    #[inline]
    pub fn get_voltage_reg_addr(&self, cpu_id: u32, offset: u64) -> u64 {
        CPU_VOLTAGE_BASE + (cpu_id as u64 * 0x1000) + offset
    }

    /// Read actual voltage from hardware (microvolts)
    pub fn cpu_get_voltage_hw(&self, cpu_id: u32) -> u32 {
        // SAFETY: Reading the current voltage register at CPU_VOLTAGE_BASE + cpu_id*0x1000
        // + VOLTAGE_CURRENT_OFFSET, which is a valid Loongson MMIO register for voltage readback.
        unsafe {
            let addr = self.get_voltage_reg_addr(cpu_id, VOLTAGE_CURRENT_OFFSET);
            let voltage_mv = Self::read_reg(addr);
            voltage_mv * 1000
        }
    }

    /// Write target voltage to hardware (microvolts)
    pub fn cpu_set_voltage_hw(&mut self, cpu_id: u32, voltage: u32) -> i32 {
        if voltage < 700_000 || voltage > 1_300_000 {
            log_warn!("CPU {}: Invalid voltage {} mV", cpu_id, voltage / 1000);
            return -1;
        }

        // SAFETY: Writing to Loongson CPU voltage control MMIO register at
        // CPU_VOLTAGE_BASE + cpu_id*0x1000 + VOLTAGE_TARGET_OFFSET, which is a
        // valid hardware register for voltage scaling on Loongson 3A6000/3C6000.
        unsafe {
            let target_addr = self.get_voltage_reg_addr(cpu_id, VOLTAGE_TARGET_OFFSET);
            Self::write_reg(target_addr, voltage / 1000);
            Self::udelay(VOLTAGE_SETTLE_US);
        }

        log_debug!("CPU {} voltage set to {} mV", cpu_id, voltage / 1000);
        0
    }

    // ========================================================================
    // Temperature sensor
    // ========================================================================

    /// Get temperature sensor register address
    #[inline]
    pub fn get_tsensor_reg_addr(&self, cpu_id: u32, offset: u64) -> u64 {
        TSENSOR_BASE + (cpu_id as u64 * 0x1000) + offset
    }

    /// Read actual temperature from hardware (millidegrees)
    pub fn cpu_get_temp_hw(&self, cpu_id: u32) -> i32 {
        // SAFETY: Reading Loongson temperature sensor MMIO register at TSENSOR_BASE +
        // cpu_id*0x1000 + TSENSOR_TEMP_OFFSET; the raw value directly represents
        // millidegrees on Loongson 3A6000/3C6000 per the LoongArch specification.
        unsafe {
            let addr = self.get_tsensor_reg_addr(cpu_id, TSENSOR_TEMP_OFFSET);
            let raw_temp = Self::read_reg(addr);
            // Loongson temperature sensor: raw value directly represents millidegrees
            raw_temp as i32
        }
    }

    // ========================================================================
    // CPU status control
    // ========================================================================

    /// Get CPU status register address
    #[inline]
    pub fn get_state_reg_addr(&self, cpu_id: u32, offset: u64) -> u64 {
        CPU_STATE_BASE + (cpu_id as u64 * 0x1000) + offset
    }

    /// Set CPU power state
    pub fn cpu_set_power_state(&mut self, cpu_id: u32, state: u32) -> i32 {
        // SAFETY: Writing to Loongson CPU power state MMIO register at CPU_STATE_BASE +
        // cpu_id*0x1000 + STATE_POWER_OFFSET, which is a valid hardware register
        // for CPU power state control on Loongson 3A6000/3C6000.
        unsafe {
            let addr = self.get_state_reg_addr(cpu_id, STATE_POWER_OFFSET);
            Self::write_reg(addr, state);
            Self::udelay(REG_DELAY_US);
        }
        0
    }

    /// Set CPU idle state
    pub fn cpu_set_idle_state(&mut self, cpu_id: u32, state: u32) -> i32 {
        // SAFETY: Writing to Loongson CPU idle state MMIO register at CPU_STATE_BASE +
        // cpu_id*0x1000 + STATE_IDLE_OFFSET, which is a valid hardware register
        // for CPU idle state control on Loongson 3A6000/3C6000.
        unsafe {
            let addr = self.get_state_reg_addr(cpu_id, STATE_IDLE_OFFSET);
            Self::write_reg(addr, state);
            Self::udelay(REG_DELAY_US);
        }
        0
    }

    // ========================================================================
    // HAL interface implementation
    // ========================================================================

    /// Initialize
    pub fn init(&mut self) -> i32 {
        let series_name = match self.config.series {
            LoongsonSeries::Loongson3A6000 => "Loongson 3A6000",
            LoongsonSeries::Loongson3C6000 => "Loongson 3C6000",
        };

        log_info!("Loongson {} CPU HAL initialized", series_name);
        log_info!("  Cores: {} ({}-{} MHz)",
            self.config.num_cores,
            self.config.min_freq / 1_000_000,
            self.config.max_freq / 1_000_000);
        log_info!("  L3 Cache: {} KB", self.config.l3_cache_kb);
        log_info!("  Extensions: LSX={}, LASX={}, LVZ={}, LBT={}",
            self.config.extensions.lsx,
            self.config.extensions.lasx,
            self.config.extensions.lvz,
            self.config.extensions.lbt);

        // Initialize temperature sensors
        for cpu_id in 0..self.config.num_cores {
            // SAFETY: Writing to Loongson temperature sensor enable MMIO register at
            // TSENSOR_BASE + cpu_id*0x1000 + TSENSOR_ENABLE_OFFSET, which is a valid
            // hardware register for sensor enable/disable on Loongson 3A6000/3C6000.
            unsafe {
                let enable_addr = self.get_tsensor_reg_addr(cpu_id, TSENSOR_ENABLE_OFFSET);
                Self::write_reg(enable_addr, 1);
            }
        }

        0
    }

    /// Boot CPU
    pub fn boot_cpu(&mut self, cpu_id: u32) -> i32 {
        if cpu_id >= self.config.num_cores {
            return -1;
        }

        log_debug!("Booting CPU {}", cpu_id);
        0
    }

    /// Halt CPU
    pub fn halt_cpu(&mut self, cpu_id: u32) -> i32 {
        if cpu_id >= self.config.num_cores {
            return -1;
        }

        log_debug!("Halting CPU {}", cpu_id);
        0
    }

    /// Get CPU info
    pub fn get_cpu_info(&self, cpu_id: u32) -> CpuInfo {
        CpuInfo {
            cpu_id,
            cpu_type: CpuType::Big, // All Loongson cores are homogeneous
            state: CpuState::Online,
            current_freq: self.config.max_freq,
            min_freq: self.config.min_freq,
            max_freq: self.config.max_freq,
            temperature: 45000,
            voltage: 1100000,
            utilization: 0,
        }
    }

    /// Set frequency
    pub fn set_frequency(&mut self, cpu_id: u32, freq: u64) -> i32 {
        let info = self.get_cpu_info(cpu_id);

        if freq < info.min_freq || freq > info.max_freq {
            return -1;
        }

        log_debug!("Setting CPU {} frequency to {} MHz", cpu_id, freq / 1_000_000);
        self.cpu_set_freq_hw(cpu_id, freq)
    }

    /// Get frequency
    pub fn get_frequency(&self, cpu_id: u32) -> u64 {
        self.cpu_get_freq_hw(cpu_id)
    }

    /// Set voltage
    pub fn set_voltage(&mut self, cpu_id: u32, voltage: u32) -> i32 {
        log_debug!("Setting CPU {} voltage to {} mV", cpu_id, voltage / 1000);
        self.cpu_set_voltage_hw(cpu_id, voltage)
    }

    /// Get voltage
    pub fn get_voltage(&self, cpu_id: u32) -> u32 {
        self.cpu_get_voltage_hw(cpu_id)
    }

    /// Enter idle state
    pub fn enter_idle(&mut self, cpu_id: u32, state: CpuState) -> i32 {
        log_debug!("CPU {} entering idle state {:?}", cpu_id, state);
        0
    }

    /// Exit idle state
    pub fn exit_idle(&mut self, cpu_id: u32) -> i32 {
        log_debug!("CPU {} exiting idle state", cpu_id);
        0
    }

    /// Get temperature
    pub fn get_temperature(&self, cpu_id: u32) -> i32 {
        self.cpu_get_temp_hw(cpu_id)
    }

    /// Thermal throttle
    pub fn thermal_throttle(&mut self, cpu_id: u32, level: u32) -> i32 {
        log_debug!("CPU {} thermal throttle level {}", cpu_id, level);

        let info = self.get_cpu_info(cpu_id);
        let max_freq = info.max_freq;

        let throttle_factor = 100 - (level * 20).min(80);
        let target_freq = max_freq * throttle_factor as u64 / 100;
        let final_freq = target_freq.max(info.min_freq);

        self.set_frequency(cpu_id, final_freq)
    }

    /// Check instruction extensions support
    pub fn check_extensions(&self) -> &LoongArchExtensions {
        &self.config.extensions
    }

    /// Binary translation support check
    pub fn has_binary_translation(&self) -> bool {
        self.config.extensions.lbt
    }

    /// Virtualization support check
    pub fn has_virtualization(&self) -> bool {
        self.config.extensions.lvz
    }
}

/// Loongson CPU HAL operations
pub static LOONGSON_CPU_OPS: CpuHalOps = CpuHalOps {
    init: || 0,
    boot_cpu: |_cpu_id| 0,
    halt_cpu: |_cpu_id| 0,
    get_cpu_info: |_cpu_id| CpuInfo {
        cpu_id: 0,
        cpu_type: CpuType::Big,
        state: CpuState::Online,
        current_freq: 0,
        min_freq: 0,
        max_freq: 0,
        temperature: 0,
        voltage: 0,
        utilization: 0,
    },
    set_frequency: |_cpu_id, _freq| 0,
    get_frequency: |_cpu_id| 0,
    set_voltage: |_cpu_id, _voltage| 0,
    get_voltage: |_cpu_id| 0,
    enter_idle: |_cpu_id, _state| 0,
    exit_idle: |_cpu_id| 0,
    get_temperature: |_cpu_id| 0,
    thermal_throttle: |_cpu_id, _level| 0,
};

/// Get the Loongson CPU HAL instance
pub fn get_loongson_hal() -> LoongsonCpuHal {
    LoongsonCpuHal::loongson3a6000()
}
