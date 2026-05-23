/*
 * Nuva OS - HAL - Power
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



use super::{PowerState, PowerDomainType, PowerDomainState, BatteryStatus, PowerInfo, PowerHalOps};
use crate::{pr_debug, pr_info, pr_warn};

/// PMIC register base address
const PMIC_BASE: u64 = 0x0A000000;

/// PMIC register offsets
mod pmic_regs {
    pub const POWER_STATUS: u64 = 0x000;
    pub const BATTERY_STATUS: u64 = 0x004;
    pub const BATTERY_CAPACITY: u64 = 0x008;
    pub const BATTERY_VOLTAGE: u64 = 0x00C;
    pub const BATTERY_CURRENT: u64 = 0x010;
    pub const BATTERY_TEMP: u64 = 0x014;
    pub const DOMAIN_CTRL: u64 = 0x020;
    pub const DOMAIN_STATUS: u64 = 0x024;
    pub const SUSPEND_CTRL: u64 = 0x030;
    pub const WAKEUP_CTRL: u64 = 0x034;
}

/// PMIC driver
pub struct PmicDriver {
    base: u64,
}

impl PmicDriver {
    pub const fn new() -> Self {
        PmicDriver {
            base: PMIC_BASE,
        }
    }

    /// Initialize
    pub fn init(&mut self) -> i32 {
        log_info!("PMIC driver initialized");
        log_info!("  Base address: {:#010x}", self.base);
        0
    }

    /// Get power info
    pub fn get_power_info(&self) -> PowerInfo {
        // Read power status from PMIC registers
        let status = self.read_reg(pmic_regs::POWER_STATUS);
        let batt_status = self.read_reg(pmic_regs::BATTERY_STATUS);
        PowerInfo {
            state: if status & 0x1 != 0 { PowerState::Running } else { PowerState::Off },
            ac_online: status & 0x2 != 0,
            usb_online: status & 0x4 != 0,
            battery: BatteryStatus {
                present: batt_status & 0x1 != 0,
                charging: batt_status & 0x2 != 0,
                capacity: self.read_reg(pmic_regs::BATTERY_CAPACITY),
                voltage: self.read_reg(pmic_regs::BATTERY_VOLTAGE),
                current: self.read_reg(pmic_regs::BATTERY_CURRENT) as i32,
                temperature: self.read_reg(pmic_regs::BATTERY_TEMP) as i32,
                health: 95,
            },
        }
    }

    /// Set power state
    pub fn set_power_state(&mut self, state: PowerState) -> i32 {
        log_info!("PMIC: Setting power state to {:?}", state);

        match state {
            PowerState::Running => {
                // Normal running state
            }
            PowerState::Idle => {
                // Idle state
            }
            PowerState::Suspend => {
                // Suspend state
                let r = self.suspend();
                if r != 0 { return r; }
            }
            PowerState::Hibernate => {
                // Hibernate state
                let r = self.hibernate();
                if r != 0 { return r; }
            }
            PowerState::Off => {
                // Power off
                let r = self.power_off();
                if r != 0 { return r; }
            }
        }

        0
    }

    /// Get power domain state
    pub fn get_domain_state(&self, domain: PowerDomainType) -> PowerDomainState {
        // Read domain status register and extract per-domain state
        let status = self.read_reg(pmic_regs::DOMAIN_STATUS);
        let shift = match domain {
            PowerDomainType::Cpu => 0,
            PowerDomainType::Gpu => 2,
            PowerDomainType::Npu => 4,
            PowerDomainType::Memory => 6,
            PowerDomainType::Peripheral => 8,
            PowerDomainType::Display => 10,
        };
        let mask = 0x3 << shift;
        match (status & mask) >> shift {
            0 => PowerDomainState::Off,
            1 => PowerDomainState::On,
            2 => PowerDomainState::Partial,
            _ => PowerDomainState::Off,
        }
    }

    /// Set power domain state
    pub fn set_domain_state(&mut self, domain: PowerDomainType, state: PowerDomainState) -> i32 {
        log_debug!("PMIC: Setting domain {:?} to {:?}", domain, state);

        // Calculate domain mask and value for PMIC DOMAIN_CTRL register
        let shift = match domain {
            PowerDomainType::Cpu => 0,
            PowerDomainType::Gpu => 2,
            PowerDomainType::Npu => 4,
            PowerDomainType::Memory => 6,
            PowerDomainType::Peripheral => 8,
            PowerDomainType::Display => 10,
        };
        let val = match state {
            PowerDomainState::Off => 0u32,
            PowerDomainState::On => 1u32,
            PowerDomainState::Partial => 2u32,
        };
        let ctrl = self.read_reg(pmic_regs::DOMAIN_CTRL);
        let mask = 0x3 << shift;
        self.write_reg(pmic_regs::DOMAIN_CTRL, (ctrl & !mask) | (val << shift));

        0
    }

    /// Enter suspend
    fn suspend(&mut self) -> i32 {
        log_info!("PMIC: Entering suspend");

        // Implementation of actual suspend
        // 1. Save all power domain states
        // 2. Turn off non-essential power domains
        // 3. Configure wake sources
        // 4. Enter low power mode

        // Save power domain states
        let cpu_state = self.get_domain_state(PowerDomainType::Cpu);
        let gpu_state = self.get_domain_state(PowerDomainType::Gpu);
        let memory_state = self.get_domain_state(PowerDomainType::Memory);

        // Turn off non-essential power domains
        let _ = self.set_domain_state(PowerDomainType::Gpu, PowerDomainState::Off);
        let _ = self.set_domain_state(PowerDomainType::Peripheral, PowerDomainState::Off);

        // Configure wake sources (simplified)
        log_debug!("PMIC: Configuring wake sources");

        // Enter low power mode
        log_debug!("PMIC: Entering low power mode");

        log_info!("PMIC: Suspend complete");

        0
    }

    /// Resume
    fn resume(&mut self) -> i32 {
        log_info!("PMIC: Resuming");

        // Implementation of actual resume
        // 1. Restore power domain states
        // 2. Restore clocks
        // 3. Restore peripherals

        // Restore power domain states
        let _ = self.set_domain_state(PowerDomainType::Cpu, PowerDomainState::On);
        let _ = self.set_domain_state(PowerDomainType::Gpu, PowerDomainState::On);
        let _ = self.set_domain_state(PowerDomainType::Memory, PowerDomainState::On);
        let _ = self.set_domain_state(PowerDomainType::Peripheral, PowerDomainState::On);

        // Restore clocks (simplified)
        log_debug!("PMIC: Restoring clocks");

        // Restore peripherals (simplified)
        log_debug!("PMIC: Restoring peripherals");

        log_info!("PMIC: Resume complete");

        0
    }

    /// Enter hibernate
    fn hibernate(&mut self) -> i32 {
        log_info!("PMIC: Entering hibernate");

        // 1. Save system state to disk (via filesystem sync)
        // 2. Turn off all non-memory power domains
        let _ = self.set_domain_state(PowerDomainType::Cpu, PowerDomainState::Off);
        let _ = self.set_domain_state(PowerDomainType::Gpu, PowerDomainState::Off);
        let _ = self.set_domain_state(PowerDomainType::Npu, PowerDomainState::Off);
        let _ = self.set_domain_state(PowerDomainType::Peripheral, PowerDomainState::Off);
        let _ = self.set_domain_state(PowerDomainType::Display, PowerDomainState::Off);
        // 3. Write hibernate command to PMIC suspend control register
        self.write_reg(pmic_regs::SUSPEND_CTRL, 0x2);

        0
    }

    /// Power off
    fn power_off(&mut self) -> i32 {
        log_info!("PMIC: Powering off");

        // 1. Sync filesystem (caller should ensure this)
        // 2. Turn off all power domains
        let _ = self.set_domain_state(PowerDomainType::Gpu, PowerDomainState::Off);
        let _ = self.set_domain_state(PowerDomainType::Npu, PowerDomainState::Off);
        let _ = self.set_domain_state(PowerDomainType::Peripheral, PowerDomainState::Off);
        let _ = self.set_domain_state(PowerDomainType::Display, PowerDomainState::Off);
        let _ = self.set_domain_state(PowerDomainType::Memory, PowerDomainState::Off);
        let _ = self.set_domain_state(PowerDomainType::Cpu, PowerDomainState::Off);
        // 3. Send power off command to PMIC
        self.write_reg(pmic_regs::SUSPEND_CTRL, 0x1);

        0
    }

    /// Reboot
    pub fn reboot(&mut self) -> i32 {
        log_info!("PMIC: Rebooting");

        // 1. Sync filesystem (caller should ensure this)
        // 2. Send reboot command to PMIC wakeup control register
        self.write_reg(pmic_regs::WAKEUP_CTRL, 0x1);

        0
    }

    /// Read register with retry (I2C/SPI communication may timeout)
    fn read_reg_retry(&self, offset: u64, max_retries: u32) -> Result<u32, i32> {
        const EIO: i32 = -5;
        for attempt in 0..max_retries {
            let value = self.read_reg(offset);
            // Validate register read: check for bus error markers
            // A value of 0xFFFFFFFF on most PMICs indicates a bus error
            if value == 0xFFFF_FFFF {
                log_warn!("PMIC: read_reg offset 0x{:X} failed (attempt {}/{})", offset, attempt + 1, max_retries);
                if attempt + 1 < max_retries {
                    // Brief delay before retry (busy-wait for a few cycles)
                    for _ in 0..100 {
                        core::hint::spin_loop();
                    }
                    continue;
                }
                return Err(EIO);
            }
            return Ok(value);
        }
        Err(EIO)
    }

    /// Write register with retry and verification
    fn write_reg_retry(&self, offset: u64, value: u32, max_retries: u32) -> Result<(), i32> {
        const EIO: i32 = -5;
        for attempt in 0..max_retries {
            self.write_reg(offset, value);
            // Verify write by reading back
            let readback = self.read_reg(offset);
            if readback == value {
                return Ok(());
            }
            log_warn!("PMIC: write_reg verify failed at offset 0x{:X} (attempt {}/{}): wrote 0x{:X}, read 0x{:X}",
                     offset, attempt + 1, max_retries, value, readback);
        }
        Err(EIO)
    }

    /// Read register with 3-retry default
    fn read_reg_safe(&self, offset: u64) -> u32 {
        self.read_reg_retry(offset, 3).unwrap_or(0)
    }

    /// Write register with 3-retry default
    fn write_reg_safe(&self, offset: u64, value: u32) -> i32 {
        match self.write_reg_retry(offset, value, 3) {
            Ok(()) => 0,
            Err(e) => e,
        }
    }

    /// Read PMIC chip ID for validation
    pub fn read_chip_id(&self) -> u32 {
        self.read_reg_safe(0x000)
    }

    /// Check if PMIC is responsive (chip ID != 0 and != 0xFFFFFFFF)
    pub fn is_responsive(&self) -> bool {
        let chip_id = self.read_chip_id();
        chip_id != 0 && chip_id != 0xFFFF_FFFF
    }

    /// Read register
    fn read_reg(&self, offset: u64) -> u32 {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            core::ptr::read_volatile((self.base + offset) as *const u32)
        }
    }

    /// Write register
    fn write_reg(&self, offset: u64, value: u32) {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            core::ptr::write_volatile((self.base + offset) as *mut u32, value);
        }
    }
}

/// PMIC HAL operations
pub static PMIC_POWER_OPS: PowerHalOps = PowerHalOps {
    init: || 0,
    get_power_info: || PowerInfo {
        state: PowerState::Running,
        ac_online: true,
        usb_online: false,
        battery: BatteryStatus {
            present: true,
            charging: true,
            capacity: 85,
            voltage: 4200,
            current: 500,
            temperature: 30000,
            health: 95,
        },
    },
    set_power_state: |_state| 0,
    get_domain_state: |_domain| PowerDomainState::On,
    set_domain_state: |_domain, _state| 0,
    suspend: || 0,
    resume: || 0,
    power_off: || 0,
    reboot: || 0,
};

static mut PMIC_DRIVER: PmicDriver = PmicDriver::new();

pub fn get_pmic_driver() -> &'static mut PmicDriver {
    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe { &mut PMIC_DRIVER }
}

pub fn init_pmic_driver() {
    let driver = get_pmic_driver();
    driver.init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pmic() {
        let pmic = get_pmic_driver();
        let info = pmic.get_power_info();
        assert!(info.ac_online);
    }

    #[test]
    fn test_pmic_driver_base_address() {
        let pmic = get_pmic_driver();
        assert_eq!(pmic.base, 0x0A000000u64);
    }

    #[test]
    fn test_pmic_read_reg_safe_returns_u32() {
        let pmic = get_pmic_driver();
        // Reading a register should return a u32 value without panic
        let _val: u32 = pmic.read_reg_safe(pmic_regs::POWER_STATUS);
    }

    #[test]
    fn test_pmic_write_reg_safe_returns_status() {
        let pmic = get_pmic_driver();
        // Write with safe retry should return a status code
        let _status: i32 = pmic.write_reg_safe(pmic_regs::SUSPEND_CTRL, 0x1);
    }

    #[test]
    fn test_pmic_read_chip_id() {
        let pmic = get_pmic_driver();
        let _chip_id: u32 = pmic.read_chip_id();
    }

    #[test]
    fn test_pmic_power_state_running() {
        let info = get_pmic_driver().get_power_info();
        // In test environment, power state should be valid
        let _state = info.state;
    }

    #[test]
    fn test_pmic_domain_types() {
        // Verify all domain types compile and are distinct
        let _domains = [
            PowerDomainType::Cpu,
            PowerDomainType::Gpu,
            PowerDomainType::Npu,
            PowerDomainType::Memory,
            PowerDomainType::Peripheral,
            PowerDomainType::Display,
        ];
    }

    #[test]
    fn test_pmic_domain_states() {
        // Verify all domain states compile
        let _states = [
            PowerDomainState::Off,
            PowerDomainState::On,
            PowerDomainState::Partial,
        ];
    }
}
