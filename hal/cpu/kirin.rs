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
// HiSilicon Kirin CPU Register Address Definitions (Generic)
// ============================================================================

/// CPU frequency control register base address
const CPU_FREQ_BASE: u64 = 0xF580_0000;

/// CPU voltage control register base address (via PMIC)
const CPU_VOLTAGE_BASE: u64 = 0xF590_0000;

/// Temperature sensor register base address
const TSENSOR_BASE: u64 = 0xF5A0_0000;

/// CPU status register base address
const CPU_STATE_BASE: u64 = 0xF5B0_0000;

// Frequency register offsets
const FREQ_TARGET_OFFSET: u64 = 0x0000;   // Target frequency register
const FREQ_CURRENT_OFFSET: u64 = 0x0004;  // Current frequency register
const FREQ_ENABLE_OFFSET: u64 = 0x0008;   // Frequency enable register
const FREQ_DIVIDER_OFFSET: u64 = 0x000C;  // Frequency divider register

// Voltage register offsets
const VOLTAGE_TARGET_OFFSET: u64 = 0x0000;  // Target voltage register
const VOLTAGE_CURRENT_OFFSET: u64 = 0x0004; // Current voltage register
const VOLTAGE_STEP_OFFSET: u64 = 0x0008;    // Voltage step register

// Temperature sensor register offsets
const TSENSOR_TEMP_OFFSET: u64 = 0x0000;    // Temperature reading register
const TSENSOR_ENABLE_OFFSET: u64 = 0x0004;  // Sensor enable register
const TSENSOR_CALIB_OFFSET: u64 = 0x0008;   // Calibration data register

// CPU status register offsets
const STATE_POWER_OFFSET: u64 = 0x0000;     // Power state register
const STATE_IDLE_OFFSET: u64 = 0x0004;      // Idle state register

// Register operation delays (microseconds)
const REG_DELAY_US: u32 = 10;
const VOLTAGE_SETTLE_US: u32 = 100;
const FREQ_SETTLE_US: u32 = 50;

/// Kirin chip series
#[derive(Debug, Clone, Copy)]
pub enum KirinSeries {
    /// Kirin 9000 series
    Kirin9000,
    /// Kirin 9010 series
    Kirin9010,
    /// Generic Kirin
    Generic,
}

/// HiSilicon Kirin CPU configuration
pub struct KirinConfig {
    /// Number of big cores
    pub num_big: u32,
    /// Number of little cores
    pub num_little: u32,
    /// Big core minimum frequency
    pub big_min_freq: u64,
    /// Big core maximum frequency
    pub big_max_freq: u64,
    /// Little core minimum frequency
    pub little_min_freq: u64,
    /// Little core maximum frequency
    pub little_max_freq: u64,
    /// Chip series
    pub series: KirinSeries,
}

impl KirinConfig {
    /// Create Kirin 9000 configuration
    pub const fn kirin9000() -> Self {
        KirinConfig {
            num_big: 4,      // 4 big cores
            num_little: 4,   // 4 little cores
            big_min_freq: 800_000_000,    // 800 MHz
            big_max_freq: 3_130_000_000,  // 3.13 GHz
            little_min_freq: 550_000_000, // 550 MHz
            little_max_freq: 2_050_000_000, // 2.05 GHz
            series: KirinSeries::Kirin9000,
        }
    }

    /// Create Kirin 9010 configuration
    pub const fn kirin9010() -> Self {
        KirinConfig {
            num_big: 4,      // 4 big cores
            num_little: 4,   // 4 little cores
            big_min_freq: 800_000_000,    // 800 MHz
            big_max_freq: 3_300_000_000,  // 3.3 GHz
            little_min_freq: 550_000_000, // 550 MHz
            little_max_freq: 2_200_000_000, // 2.2 GHz
            series: KirinSeries::Kirin9010,
        }
    }

    /// Create default configuration (generic)
    pub const fn new() -> Self {
        Self::kirin9000()
    }
}

/// HiSilicon Kirin CPU HAL
pub struct KirinCpuHal {
    config: KirinConfig,
}

impl KirinCpuHal {
    /// Create Kirin 9000 HAL
    pub const fn kirin9000() -> Self {
        KirinCpuHal {
            config: KirinConfig::kirin9000(),
        }
    }

    /// Create Kirin 9010 HAL
    pub const fn kirin9010() -> Self {
        KirinCpuHal {
            config: KirinConfig::kirin9010(),
        }
    }

    /// Create default HAL
    pub const fn new() -> Self {
        Self::kirin9000()
    }

    /// Create Kirin 9020 HAL - TODO: implement properly
    pub const fn kirin9020() -> Self {
        Self::kirin9010()
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

    /// Read 64-bit register
    #[inline]
    unsafe fn read_reg64(addr: u64) -> u64 {
        read_volatile(addr as *const u64)
    }

    /// Write 64-bit register
    #[inline]
    unsafe fn write_reg64(addr: u64, value: u64) {
        write_volatile(addr as *mut u64, value);
    }

    /// Microsecond delay
    #[inline]
    fn udelay(us: u32) {
        // Simple busy-wait delay
        // In actual system should use timer
        let cycles = us * 100; // Assume 100 MHz clock
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
        // Each CPU has independent frequency control register group, spaced by 0x1000
        CPU_FREQ_BASE + (cpu_id as u64 * 0x1000) + offset
    }

    /// Read actual frequency from hardware
    pub fn cpu_get_freq_hw(&self, cpu_id: u32) -> u64 {
        // SAFETY: Reading the current frequency register at CPU_FREQ_BASE + cpu_id*0x1000
        // + FREQ_CURRENT_OFFSET, which is a valid Kirin MMIO register for frequency readback.
        unsafe {
            let addr = self.get_freq_reg_addr(cpu_id, FREQ_CURRENT_OFFSET);
            let freq_khz = Self::read_reg(addr);
            (freq_khz as u64) * 1000 // Convert to Hz
        }
    }

    /// Write target frequency to hardware
    pub fn cpu_set_freq_hw(&mut self, cpu_id: u32, freq: u64) -> i32 {
        let info = self.get_cpu_info(cpu_id);

        // Validate frequency range
        if freq < info.min_freq || freq > info.max_freq {
            log_warn!("CPU {}: Invalid frequency {} MHz", cpu_id, freq / 1_000_000);
            return -1;
        }

        // SAFETY: Writing to Kirin CPU frequency control MMIO registers at
        // CPU_FREQ_BASE + cpu_id*0x1000: divider, target frequency, and enable
        // registers are valid hardware registers for DVFS control on Kirin SoCs.
        unsafe {
            // Calculate divider value (assume reference clock is 26 MHz)
            let ref_clk: u64 = 26_000_000;
            let divider = (ref_clk * 1000) / freq;

            // Write divider register
            let div_addr = self.get_freq_reg_addr(cpu_id, FREQ_DIVIDER_OFFSET);
            Self::write_reg(div_addr, divider as u32);

            // Write target frequency (kHz)
            let target_addr = self.get_freq_reg_addr(cpu_id, FREQ_TARGET_OFFSET);
            Self::write_reg(target_addr, (freq / 1000) as u32);

            // Wait for frequency to settle
            Self::udelay(FREQ_SETTLE_US);

            // Enable frequency switch
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
        // + VOLTAGE_CURRENT_OFFSET, which is a valid Kirin PMIC MMIO register for voltage readback.
        unsafe {
            let addr = self.get_voltage_reg_addr(cpu_id, VOLTAGE_CURRENT_OFFSET);
            let voltage_mv = Self::read_reg(addr);
            voltage_mv * 1000 // Convert to microvolts
        }
    }

    /// Write target voltage to hardware (microvolts)
    pub fn cpu_set_voltage_hw(&mut self, cpu_id: u32, voltage: u32) -> i32 {
        // Validate voltage range (600mV - 1200mV)
        if voltage < 600_000 || voltage > 1_200_000 {
            log_warn!("CPU {}: Invalid voltage {} mV", cpu_id, voltage / 1000);
            return -1;
        }

        // SAFETY: Writing to Kirin CPU voltage control MMIO register at
        // CPU_VOLTAGE_BASE + cpu_id*0x1000 + VOLTAGE_TARGET_OFFSET, which is a
        // valid PMIC register for voltage scaling on Kirin SoCs.
        unsafe {
            // Write target voltage (mV)
            let target_addr = self.get_voltage_reg_addr(cpu_id, VOLTAGE_TARGET_OFFSET);
            Self::write_reg(target_addr, voltage / 1000);

            // Wait for voltage to settle
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
        // SAFETY: Reading Kirin temperature sensor MMIO registers at TSENSOR_BASE +
        // cpu_id*0x1000: TSENSOR_TEMP_OFFSET for raw temperature and TSENSOR_CALIB_OFFSET
        // for calibration data; both are valid hardware registers on Kirin SoCs.
        unsafe {
            let addr = self.get_tsensor_reg_addr(cpu_id, TSENSOR_TEMP_OFFSET);
            let raw_temp = Self::read_reg(addr);

            // Temperature sensor raw value to millidegrees conversion
            // Formula: temp = (raw - calib_offset) * scale
            // Assume: raw value is already in millidegrees, needs calibration offset
            let calib_addr = self.get_tsensor_reg_addr(cpu_id, TSENSOR_CALIB_OFFSET);
            let calib = Self::read_reg(calib_addr);

            let temp = if raw_temp > calib {
                ((raw_temp - calib) as i32) * 100
            } else {
                -(((calib - raw_temp) as i32) * 100)
            };

            // Add base temperature (25°C)
            temp + 25_000
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
        // SAFETY: Writing to Kirin CPU power state MMIO register at CPU_STATE_BASE +
        // cpu_id*0x1000 + STATE_POWER_OFFSET, which is a valid hardware register
        // for CPU power state control on Kirin SoCs.
        unsafe {
            let addr = self.get_state_reg_addr(cpu_id, STATE_POWER_OFFSET);
            Self::write_reg(addr, state);
            Self::udelay(REG_DELAY_US);
        }
        0
    }

    /// Set CPU idle state
    pub fn cpu_set_idle_state(&mut self, cpu_id: u32, state: u32) -> i32 {
        // SAFETY: Writing to Kirin CPU idle state MMIO register at CPU_STATE_BASE +
        // cpu_id*0x1000 + STATE_IDLE_OFFSET, which is a valid hardware register
        // for CPU idle state control on Kirin SoCs.
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
            KirinSeries::Kirin9000 => "Kirin 9000",
            KirinSeries::Kirin9010 => "Kirin 9010",
            KirinSeries::Generic => "Kirin (Generic)",
        };

        log_info!("HiSilicon {} CPU HAL initialized", series_name);
        log_info!("  Big cores: {} ({}-{} MHz)",
            self.config.num_big,
            self.config.big_min_freq / 1_000_000,
            self.config.big_max_freq / 1_000_000);
        log_info!("  Little cores: {} ({}-{} MHz)",
            self.config.num_little,
            self.config.little_min_freq / 1_000_000,
            self.config.little_max_freq / 1_000_000);

        // Initialize temperature sensors
        for cpu_id in 0..(self.config.num_big + self.config.num_little) {
            // SAFETY: Writing to Kirin temperature sensor enable MMIO register at
            // TSENSOR_BASE + cpu_id*0x1000 + TSENSOR_ENABLE_OFFSET, which is a valid
            // hardware register for sensor enable/disable on Kirin SoCs.
            unsafe {
                let enable_addr = self.get_tsensor_reg_addr(cpu_id, TSENSOR_ENABLE_OFFSET);
                Self::write_reg(enable_addr, 1);
            }
        }

        0
    }

    /// Boot CPU via PSCI CPU_ON SMC call
    pub fn boot_cpu(&mut self, cpu_id: u32) -> i32 {
        if cpu_id >= self.config.num_big + self.config.num_little {
            return -1;
        }

        log_debug!("Booting CPU {} via PSCI CPU_ON", cpu_id);

        // PSCI CPU_ON: SMC call with x0=0xC4000003 (PSCI 1.0+ 64-bit CPU_ON)
        // x1 = target CPU ID (MPIDR affinity)
        // x2 = entry point address (kernel secondary CPU entry)
        // x3 = context ID (passed to the booted CPU, typically 0)
        // SAFETY: SMC instruction transfers control to secure firmware (ARM Trusted FW)
        // for PSCI CPU_ON. The firmware validates parameters and either boots the
        // target CPU or returns an error code. This is the standard PSCI interface.
        let psci_fn_id: u64 = 0xC400_0003;
        let entry_point: u64 = 0; // Secondary CPU entry point; set by platform
        let context_id: u64 = 0;
        #[cfg(target_arch = "aarch64")]
        let result: u64;
        #[cfg(target_arch = "aarch64")]
        unsafe {
            core::arch::asm!(
                "smc #0",
                inlateout("x0") psci_fn_id => result,
                inlateout("x1") cpu_id as u64 => _,
                inlateout("x2") entry_point => _,
                inlateout("x3") context_id => _,
            );
        }
        #[cfg(not(target_arch = "aarch64"))]
        let result: u64 = 0;

        // PSCI return codes: 0 = SUCCESS, -1 = NOT_SUPPORTED, -2 = INVALID_PARAMETERS,
        // -3 = DENIED, -4 = ALREADY_ON
        if result != 0 {
            log_warn!("PSCI CPU_ON for CPU {} failed: {}", cpu_id, result as i64);
            return result as i32;
        }

        // Set power state to online
        self.cpu_set_power_state(cpu_id, CpuState::Online as u32);

        0
    }

    /// Halt CPU via PSCI CPU_OFF SMC call
    pub fn halt_cpu(&mut self, cpu_id: u32) -> i32 {
        if cpu_id >= self.config.num_big + self.config.num_little {
            return -1;
        }

        log_debug!("Halting CPU {} via PSCI CPU_OFF", cpu_id);

        // PSCI CPU_OFF: SMC call with x0=0x84000002 (PSCI 64-bit CPU_OFF)
        // No additional parameters. This call does not return on success.
        // SAFETY: SMC instruction transfers control to secure firmware for PSCI CPU_OFF.
        // The firmware powers off the target CPU. This call does not return on success;
        // on failure, it returns a PSCI error code.
        let psci_fn_id: u64 = 0x8400_0002;
        #[cfg(target_arch = "aarch64")]
        let result: u64;
        #[cfg(target_arch = "aarch64")]
        unsafe {
            core::arch::asm!(
                "smc #0",
                in("x0") psci_fn_id,
                lateout("x0") result,
                out("x1") _,
                out("x2") _,
                out("x3") _,
            );
        }
        #[cfg(not(target_arch = "aarch64"))]
        let result: u64 = 0;

        if result != 0 {
            log_warn!("PSCI CPU_OFF for CPU {} failed: {}", cpu_id, result as i64);
            return result as i32;
        }

        // Set power state to offline (only reached on failure path)
        self.cpu_set_power_state(cpu_id, CpuState::Offline as u32);

        0
    }

    /// Get CPU info
    pub fn get_cpu_info(&self, cpu_id: u32) -> CpuInfo {
        let cpu_type = if cpu_id < self.config.num_big {
            CpuType::Big
        } else {
            CpuType::Little
        };

        let (min_freq, max_freq) = match cpu_type {
            CpuType::Big => (self.config.big_min_freq, self.config.big_max_freq),
            CpuType::Little => (self.config.little_min_freq, self.config.little_max_freq),
        };

        CpuInfo {
            cpu_id,
            cpu_type,
            state: CpuState::Online,
            current_freq: max_freq,
            min_freq,
            max_freq,
            temperature: 45000,  // 45°C
            voltage: 1100000,    // 1.1V
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

        // Call actual hardware frequency setting
        self.cpu_set_freq_hw(cpu_id, freq)
    }

    /// Get frequency
    pub fn get_frequency(&self, cpu_id: u32) -> u64 {
        // Read actual frequency from hardware
        self.cpu_get_freq_hw(cpu_id)
    }

    /// Set voltage
    pub fn set_voltage(&mut self, cpu_id: u32, voltage: u32) -> i32 {
        log_debug!("Setting CPU {} voltage to {} mV", cpu_id, voltage / 1000);

        // Call actual hardware voltage setting
        self.cpu_set_voltage_hw(cpu_id, voltage)
    }

    /// Get voltage
    pub fn get_voltage(&self, cpu_id: u32) -> u32 {
        // Read actual voltage from hardware
        self.cpu_get_voltage_hw(cpu_id)
    }

    /// Enter idle state via idle state register configuration and WFI
    pub fn enter_idle(&mut self, cpu_id: u32, state: CpuState) -> i32 {
        if cpu_id >= self.config.num_big + self.config.num_little {
            return -1;
        }

        log_debug!("CPU {} entering idle state {:?}", cpu_id, state);

        // Map CpuState to idle state register value
        let idle_val = match state {
            CpuState::Online => 0,  // No idle (running)
            CpuState::Offline => 3, // Deepest idle (power off)
            CpuState::Idle => 1,    // WFI light idle
            CpuState::DeepIdle => 2, // Retention idle
        };

        // Configure idle state register before entering idle
        // SAFETY: Writing to Kirin CPU idle state MMIO register at CPU_STATE_BASE +
        // cpu_id*0x1000 + STATE_IDLE_OFFSET, which is a valid hardware register
        // for CPU idle state control on Kirin SoCs.
        unsafe {
            let addr = self.get_state_reg_addr(cpu_id, STATE_IDLE_OFFSET);
            Self::write_reg(addr, idle_val);
            Self::udelay(REG_DELAY_US);
        }

        // Execute WFI (Wait For Interrupt) to enter low power state
        // The CPU will resume on the next interrupt
        // SAFETY: WFI is a standard ARM64 hint instruction that suspends execution
        // until an interrupt occurs. It is always safe to execute in kernel mode.
        #[cfg(target_arch = "aarch64")]
        unsafe {
            core::arch::asm!("wfi");
        }

        0
    }

    /// Exit idle state: restore CPU to running state
    pub fn exit_idle(&mut self, cpu_id: u32) -> i32 {
        if cpu_id >= self.config.num_big + self.config.num_little {
            return -1;
        }

        log_debug!("CPU {} exiting idle state", cpu_id);

        // Set idle state register back to 0 (running/no idle)
        // SAFETY: Writing to Kirin CPU idle state MMIO register at CPU_STATE_BASE +
        // cpu_id*0x1000 + STATE_IDLE_OFFSET, which is a valid hardware register
        // for CPU idle state control on Kirin SoCs.
        unsafe {
            let idle_addr = self.get_state_reg_addr(cpu_id, STATE_IDLE_OFFSET);
            Self::write_reg(idle_addr, 0);
            Self::udelay(REG_DELAY_US);
        }

        // Set power state back to online/running
        self.cpu_set_power_state(cpu_id, CpuState::Online as u32);

        0
    }
}

/// Get the Kirin CPU HAL instance
pub fn get_kirin_hal() -> KirinCpuHal {
    KirinCpuHal::kirin9020()
}
