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



use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use core::ptr::{read_volatile, write_volatile};
use crate::{pr_debug, pr_info, pr_warn};

// ============================================================================
// DVFS Hardware Register Definitions
// ============================================================================

/// DVFS control register base address
const DVFS_CTRL_BASE: u64 = 0xF5C0_0000;

/// DVFS register offsets
const DVFS_VOLTAGE_TARGET: u64 = 0x0000;   // Target voltage register
const DVFS_VOLTAGE_CURRENT: u64 = 0x0004;  // Current voltage register
const DVFS_FREQ_TARGET: u64 = 0x0008;      // Target frequency register
const DVFS_FREQ_CURRENT: u64 = 0x000C;     // Current frequency register
const DVFS_CTRL: u64 = 0x0010;             // Control register
const DVFS_STATUS: u64 = 0x0014;           // Status register

/// DVFS control bits
const DVFS_CTRL_ENABLE: u32 = 0x0001;      // Enable DVFS
const DVFS_CTRL_VOLT_UP: u32 = 0x0002;     // Voltage increase request
const DVFS_CTRL_VOLT_DOWN: u32 = 0x0004;   // Voltage decrease request
const DVFS_CTRL_FREQ_UP: u32 = 0x0008;     // Frequency increase request
const DVFS_CTRL_FREQ_DOWN: u32 = 0x0010;   // Frequency decrease request

/// DVFS status bits
const DVFS_STATUS_BUSY: u32 = 0x0001;      // DVFS busy
const DVFS_STATUS_VOLT_DONE: u32 = 0x0002; // Voltage switch complete
const DVFS_STATUS_FREQ_DONE: u32 = 0x0004; // Frequency switch complete

/// Voltage settle delay (microseconds)
const VOLTAGE_SETTLE_US: u32 = 100;
/// Frequency settle delay (microseconds)
const FREQ_SETTLE_US: u32 = 50;
/// DVFS switch timeout (microseconds)
const DVFS_TIMEOUT_US: u32 = 1000;

/// DVFS policy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DvfsPolicy {
    /// Performance priority
    Performance = 0,
    /// Power saving priority
    Powersave = 1,
    /// Balanced
    Balanced = 2,
    /// User defined
    Userspace = 3,
}

/// DVFS state
pub struct DvfsState {
    /// Current frequency
    pub current_freq: AtomicU64,
    /// Target frequency
    pub target_freq: AtomicU64,
    /// Current voltage
    pub current_voltage: AtomicU32,
    /// Target voltage
    pub target_voltage: AtomicU32,
    /// Policy
    pub policy: AtomicU32,
    /// Utilization
    pub utilization: AtomicU32,
}

impl DvfsState {
    pub const fn new() -> Self {
        DvfsState {
            current_freq: AtomicU64::new(0),
            target_freq: AtomicU64::new(0),
            current_voltage: AtomicU32::new(0),
            target_voltage: AtomicU32::new(0),
            policy: AtomicU32::new(DvfsPolicy::Balanced as u32),
            utilization: AtomicU32::new(0),
        }
    }
}

/// OPP (Operating Performance Point) table entry
pub struct OppEntry {
    /// Frequency
    pub freq: u64,
    /// Voltage (microvolts)
    pub voltage: u32,
    /// Power consumption (milliwatts)
    pub power: u32,
    /// Flags
    pub flags: u32,
}

/// DVFS domain
pub struct DvfsDomain {
    /// Domain name
    pub name: &'static str,
    /// Domain ID
    pub domain_id: u32,
    /// OPP table
    pub opp_table: &'static [OppEntry],
    /// Current OPP index
    pub current_opp: AtomicU32,
    /// State
    pub state: DvfsState,
}

impl DvfsDomain {
    pub const fn new(name: &'static str, domain_id: u32, opp_table: &'static [OppEntry]) -> Self {
        DvfsDomain {
            name,
            domain_id,
            opp_table,
            current_opp: AtomicU32::new(0),
            state: DvfsState::new(),
        }
    }

    // ========================================================================
    // Register Operations
    // ========================================================================

    /// Get DVFS register address
    #[inline]
    fn get_reg_addr(offset: u64) -> u64 {
        DVFS_CTRL_BASE + offset
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

    /// Wait for DVFS idle
    fn wait_dvfs_idle(&self) -> bool {
        let mut timeout = DVFS_TIMEOUT_US;
        while timeout > 0 {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                let status = Self::read_reg(DVFS_STATUS);
                if (status & DVFS_STATUS_BUSY) == 0 {
                    return true;
                }
            }
            Self::udelay(1);
            timeout -= 1;
        }
        false
    }

    // ========================================================================
    // OPP Switch Implementation
    // ========================================================================

    /// Get current OPP
    pub fn get_current_opp(&self) -> &OppEntry {
        let idx = self.current_opp.load(Ordering::Acquire) as usize;
        &self.opp_table[idx.min(self.opp_table.len() - 1)]
    }

    /// Set OPP (safe voltage/frequency switch)
    pub fn set_opp(&self, idx: u32) -> i32 {
        if (idx as usize) >= self.opp_table.len() {
            return -1;
        }

        let current_idx = self.current_opp.load(Ordering::Acquire);
        if current_idx == idx {
            return 0; // Already at target OPP
        }

        let current_opp = &self.opp_table[current_idx as usize];
        let target_opp = &self.opp_table[idx as usize];

        log_debug!("DVFS {}: Setting OPP {} -> {} ({} MHz -> {} MHz, {} mV -> {} mV)",
            self.name, current_idx, idx,
            current_opp.freq / 1_000_000, target_opp.freq / 1_000_000,
            current_opp.voltage / 1000, target_opp.voltage / 1000);

        // Wait for DVFS idle
        if !self.wait_dvfs_idle() {
            log_warn!("DVFS {}: Timeout waiting for idle", self.name);
            return -2;
        }

        // Safe OPP switch sequence
        let result = if target_opp.freq > current_opp.freq {
            // Risk operation: increase frequency
            // Safe sequence: increase voltage first, then increase frequency
            self.set_opp_up(current_opp, target_opp)
        } else {
            // Safe operation: decrease frequency
            // Safe sequence: decrease frequency first, then decrease voltage
            self.set_opp_down(current_opp, target_opp)
        };

        if result == 0 {
            // Update state
            self.current_opp.store(idx, Ordering::Release);
            self.state.current_freq.store(target_opp.freq, Ordering::Release);
            self.state.current_voltage.store(target_opp.voltage, Ordering::Release);
        }

        result
    }

    /// Increase frequency and voltage (risk operation, increase voltage first)
    fn set_opp_up(&self, current: &OppEntry, target: &OppEntry) -> i32 {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            // Step 1: If voltage increase needed, increase voltage first
            if target.voltage > current.voltage {
                Self::write_reg(DVFS_VOLTAGE_TARGET, target.voltage / 1000); // mV
                Self::write_reg(DVFS_CTRL, DVFS_CTRL_ENABLE | DVFS_CTRL_VOLT_UP);

                // Wait for voltage to settle
                Self::udelay(VOLTAGE_SETTLE_US);

                // Verify voltage
                let actual_volt = Self::read_reg(DVFS_VOLTAGE_CURRENT);
                if actual_volt != target.voltage / 1000 {
                    log_warn!("DVFS {}: Voltage mismatch: expected {}, got {}",
                        self.name, target.voltage / 1000, actual_volt);
                    return -3;
                }
            }

            // Step 2: Risk operation - increase frequency
            Self::write_reg(DVFS_FREQ_TARGET, (target.freq / 1000) as u32); // kHz
            Self::write_reg(DVFS_CTRL, DVFS_CTRL_ENABLE | DVFS_CTRL_FREQ_UP);

            // Wait for frequency to settle
            Self::udelay(FREQ_SETTLE_US);

            // Step 3: If target voltage lower than current voltage, decrease voltage
            if target.voltage < current.voltage {
                Self::write_reg(DVFS_VOLTAGE_TARGET, target.voltage / 1000);
                Self::write_reg(DVFS_CTRL, DVFS_CTRL_ENABLE | DVFS_CTRL_VOLT_DOWN);
                Self::udelay(VOLTAGE_SETTLE_US);
            }
        }

        0
    }

    /// Decrease frequency and voltage (safe operation, decrease frequency first)
    fn set_opp_down(&self, current: &OppEntry, target: &OppEntry) -> i32 {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            // Step 1: Safe operation - decrease frequency first
            Self::write_reg(DVFS_FREQ_TARGET, (target.freq / 1000) as u32); // kHz
            Self::write_reg(DVFS_CTRL, DVFS_CTRL_ENABLE | DVFS_CTRL_FREQ_DOWN);

            // Wait for frequency to settle
            Self::udelay(FREQ_SETTLE_US);

            // Step 2: If voltage decrease needed, decrease voltage
            if target.voltage < current.voltage {
                Self::write_reg(DVFS_VOLTAGE_TARGET, target.voltage / 1000); // mV
                Self::write_reg(DVFS_CTRL, DVFS_CTRL_ENABLE | DVFS_CTRL_VOLT_DOWN);

                // Wait for voltage to settle
                Self::udelay(VOLTAGE_SETTLE_US);
            }

            // Step 3: If target voltage higher than current voltage, increase voltage
            if target.voltage > current.voltage {
                Self::write_reg(DVFS_VOLTAGE_TARGET, target.voltage / 1000);
                Self::write_reg(DVFS_CTRL, DVFS_CTRL_ENABLE | DVFS_CTRL_VOLT_UP);
                Self::udelay(VOLTAGE_SETTLE_US);
            }
        }

        0
    }

    /// Adjust frequency based on utilization
    pub fn adjust_frequency(&self, utilization: u32) {
        self.state.utilization.store(utilization, Ordering::Release);

        let policy = match self.state.policy.load(Ordering::Acquire) {
            0 => DvfsPolicy::Performance,
            1 => DvfsPolicy::Powersave,
            2 => DvfsPolicy::Balanced,
            3 => DvfsPolicy::Userspace,
            _ => DvfsPolicy::Balanced,
        };

        let target_opp = match policy {
            DvfsPolicy::Performance => {
                // Performance priority: quickly increase frequency at high utilization
                if utilization > 80 {
                    self.opp_table.len() - 1  // Highest frequency
                } else if utilization > 50 {
                    (self.opp_table.len() * 3 / 4) - 1
                } else {
                    (self.opp_table.len() / 2) - 1
                }
            }
            DvfsPolicy::Powersave => {
                // Power saving priority: quickly decrease frequency at low utilization
                if utilization < 20 {
                    0  // Lowest frequency
                } else if utilization < 50 {
                    (self.opp_table.len() / 4) - 1
                } else {
                    (self.opp_table.len() / 2) - 1
                }
            }
            DvfsPolicy::Balanced => {
                // Balanced: adjust linearly based on utilization
                (utilization as usize * self.opp_table.len() / 100).min(self.opp_table.len() - 1)
            }
            DvfsPolicy::Userspace => {
                // Userspace control: no automatic adjustment
                return;
            }
        };

        let _ = self.set_opp(target_opp as u32);
    }

    /// Set policy
    pub fn set_policy(&self, policy: DvfsPolicy) {
        self.state.policy.store(policy as u32, Ordering::Release);
        log_debug!("DVFS {}: Policy set to {:?}", self.name, policy);
    }
}

/// Big core OPP table
static BIG_CORE_OPP: [OppEntry; 8] = [
    OppEntry { freq: 800_000_000, voltage: 700_000, power: 500, flags: 0 },
    OppEntry { freq: 1_200_000_000, voltage: 800_000, power: 800, flags: 0 },
    OppEntry { freq: 1_600_000_000, voltage: 900_000, power: 1200, flags: 0 },
    OppEntry { freq: 2_000_000_000, voltage: 950_000, power: 1600, flags: 0 },
    OppEntry { freq: 2_400_000_000, voltage: 1000_000, power: 2000, flags: 0 },
    OppEntry { freq: 2_700_000_000, voltage: 1050_000, power: 2400, flags: 0 },
    OppEntry { freq: 2_900_000_000, voltage: 1100_000, power: 2800, flags: 0 },
    OppEntry { freq: 3_130_000_000, voltage: 1150_000, power: 3200, flags: 0 },
];

/// Little core OPP table
static LITTLE_CORE_OPP: [OppEntry; 6] = [
    OppEntry { freq: 550_000_000, voltage: 600_000, power: 300, flags: 0 },
    OppEntry { freq: 800_000_000, voltage: 700_000, power: 500, flags: 0 },
    OppEntry { freq: 1_200_000_000, voltage: 800_000, power: 800, flags: 0 },
    OppEntry { freq: 1_600_000_000, voltage: 850_000, power: 1100, flags: 0 },
    OppEntry { freq: 1_800_000_000, voltage: 900_000, power: 1400, flags: 0 },
    OppEntry { freq: 2_050_000_000, voltage: 950_000, power: 1800, flags: 0 },
];

/// DVFS domain array
static mut DVFS_DOMAINS: [Option<DvfsDomain>; 2] = [
    Some(DvfsDomain::new("big", 0, &BIG_CORE_OPP)),
    Some(DvfsDomain::new("little", 1, &LITTLE_CORE_OPP)),
];

/// Get DVFS domain
pub fn get_dvfs_domain(domain_id: u32) -> Option<&'static DvfsDomain> {
    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe {
        if (domain_id as usize) < DVFS_DOMAINS.len() {
            DVFS_DOMAINS[domain_id as usize].as_ref()
        } else {
            None
        }
    }
}

/// Initialize DVFS
pub fn init_dvfs() {
    log_info!("DVFS initialized");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dvfs() {
        if let Some(domain) = get_dvfs_domain(0) {
            assert_eq!(domain.name, "big");
            domain.set_opp(4);
        }
    }
}

/// DVFS governor types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DvfsGovernor {
    /// Always use the highest available frequency
    Performance = 0,
    /// Always use the lowest available frequency
    Powersave = 1,
    /// Dynamically adjust frequency based on CPU utilization
    Ondemand = 2,
    /// Use the scheduled frequency from kernel scheduler (EAS)
    Scheduled = 3,
}

/// Current governor policy (default: Ondemand)
static mut CURRENT_GOVERNOR: DvfsGovernor = DvfsGovernor::Ondemand;

/// Set DVFS governor policy
pub fn set_governor(governor: u32) {
    let new_gov = match governor {
        0 => DvfsGovernor::Performance,
        1 => DvfsGovernor::Powersave,
        2 => DvfsGovernor::Ondemand,
        3 => DvfsGovernor::Scheduled,
        _ => {
            log_warn!("DVFS: Unknown governor {}, using Ondemand", governor);
            DvfsGovernor::Ondemand
        }
    };

    // SAFETY: Writing to CURRENT_GOVERNOR, a static mut global.
    // In a real system this would be protected by a spinlock.
    unsafe {
        CURRENT_GOVERNOR = new_gov;
    }

    match new_gov {
        DvfsGovernor::Performance => {
            log_info!("DVFS: Governor set to Performance (max frequency)");
            // Set all domains to highest OPP
            // SAFETY: DVFS_DOMAINS is a mutable static accessed during governor
            // change, which is a controlled operation.
            unsafe {
                for i in 0..DVFS_DOMAINS.len() as u32 {
                    if let Some(domain) = get_dvfs_domain(i) {
                        let max_opp = domain.opp_table.len() - 1;
                        domain.set_opp(max_opp as u32);
                    }
                }
            }
        }
        DvfsGovernor::Powersave => {
            log_info!("DVFS: Governor set to Powersave (min frequency)");
            // Set all domains to lowest OPP
            // SAFETY: Same as Performance case above.
            unsafe {
                for i in 0..DVFS_DOMAINS.len() as u32 {
                    if let Some(domain) = get_dvfs_domain(i) {
                        domain.set_opp(0);
                    }
                }
            }
        }
        DvfsGovernor::Ondemand => {
            log_info!("DVFS: Governor set to Ondemand (dynamic scaling)");
        }
        DvfsGovernor::Scheduled => {
            log_info!("DVFS: Governor set to Scheduled (EAS-driven)");
        }
    }
}

/// Get current DVFS governor
pub fn get_governor() -> u32 {
    // SAFETY: Reading CURRENT_GOVERNOR, a static mut global.
    unsafe {
        CURRENT_GOVERNOR as u32
    }
}
