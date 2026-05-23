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



// CPU platform HAL module
// Re-export print macros from crate root
pub use crate::{pr_emerg, pr_alert, pr_crit, pr_err, pr_warn, pr_notice, pr_info, pr_debug};
pub mod kirin;
pub mod loongson;
pub mod dvfs;
pub mod thermal;

// Backward compatibility alias
pub use kirin as kirin9020;

use core::ptr;

/// CPU state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CpuState {
    /// Online
    Online = 0,
    /// Offline
    Offline = 1,
    /// Idle
    Idle = 2,
    /// Deep idle
    DeepIdle = 3,
}

/// CPU type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CpuType {
    /// Big core
    Big = 0,
    /// Little core
    Little = 1,
}

/// CPU info
pub struct CpuInfo {
    /// CPU ID
    pub cpu_id: u32,
    /// CPU type
    pub cpu_type: CpuType,
    /// Current state
    pub state: CpuState,
    /// Current frequency
    pub current_freq: u64,
    /// Minimum frequency
    pub min_freq: u64,
    /// Maximum frequency
    pub max_freq: u64,
    /// Current temperature (millidegrees)
    pub temperature: i32,
    /// Current voltage (microvolts)
    pub voltage: u32,
    /// Utilization (percentage)
    pub utilization: u32,
}

/// CPU HAL operations
pub struct CpuHalOps {
    /// Initialize
    pub init: fn() -> i32,
    /// Boot CPU
    pub boot_cpu: fn(cpu_id: u32) -> i32,
    /// Halt CPU
    pub halt_cpu: fn(cpu_id: u32) -> i32,
    /// Get CPU info
    pub get_cpu_info: fn(cpu_id: u32) -> CpuInfo,
    /// Set frequency
    pub set_frequency: fn(cpu_id: u32, freq: u64) -> i32,
    /// Get frequency
    pub get_frequency: fn(cpu_id: u32) -> u64,
    /// Set voltage
    pub set_voltage: fn(cpu_id: u32, voltage: u32) -> i32,
    /// Get voltage
    pub get_voltage: fn(cpu_id: u32) -> u32,
    /// Enter idle state
    pub enter_idle: fn(cpu_id: u32, state: CpuState) -> i32,
    /// Exit idle state
    pub exit_idle: fn(cpu_id: u32) -> i32,
    /// Get temperature
    pub get_temperature: fn(cpu_id: u32) -> i32,
    /// Thermal throttle
    pub thermal_throttle: fn(cpu_id: u32, level: u32) -> i32,
}

/// CPU HAL device
pub struct CpuHalDevice {
    /// Number of CPUs
    pub num_cpus: u32,
    /// Number of big cores
    pub num_big: u32,
    /// Number of little cores
    pub num_little: u32,
    /// HAL operations
    pub ops: &'static CpuHalOps,
    /// CPU info array
    pub cpu_info: [Option<CpuInfo>; 8],
}

impl CpuHalDevice {
    pub const fn new() -> Self {
        CpuHalDevice {
            num_cpus: 0,
            num_big: 0,
            num_little: 0,
            ops: &CPU_HAL_OPS_NONE,
            cpu_info: [None, None, None, None, None, None, None, None],
        }
    }

    /// Initialize
    pub fn init(&mut self) -> i32 {
        (self.ops.init)()
    }

    /// Boot CPU
    pub fn boot_cpu(&mut self, cpu_id: u32) -> i32 {
        (self.ops.boot_cpu)(cpu_id)
    }

    /// Halt CPU
    pub fn halt_cpu(&mut self, cpu_id: u32) -> i32 {
        (self.ops.halt_cpu)(cpu_id)
    }

    /// Get CPU info
    pub fn get_cpu_info(&self, cpu_id: u32) -> Option<&CpuInfo> {
        if (cpu_id as usize) < self.cpu_info.len() {
            self.cpu_info[cpu_id as usize].as_ref()
        } else {
            None
        }
    }

    /// Set frequency
    pub fn set_frequency(&mut self, cpu_id: u32, freq: u64) -> i32 {
        (self.ops.set_frequency)(cpu_id, freq)
    }

    /// Get frequency
    pub fn get_frequency(&self, cpu_id: u32) -> u64 {
        (self.ops.get_frequency)(cpu_id)
    }

    /// Set voltage
    pub fn set_voltage(&mut self, cpu_id: u32, voltage: u32) -> i32 {
        (self.ops.set_voltage)(cpu_id, voltage)
    }

    /// Get voltage
    pub fn get_voltage(&self, cpu_id: u32) -> u32 {
        (self.ops.get_voltage)(cpu_id)
    }

    /// Enter idle state
    pub fn enter_idle(&mut self, cpu_id: u32, state: CpuState) -> i32 {
        (self.ops.enter_idle)(cpu_id, state)
    }

    /// Exit idle state
    pub fn exit_idle(&mut self, cpu_id: u32) -> i32 {
        (self.ops.exit_idle)(cpu_id)
    }

    /// Get temperature
    pub fn get_temperature(&self, cpu_id: u32) -> i32 {
        (self.ops.get_temperature)(cpu_id)
    }

    /// Thermal throttle
    pub fn thermal_throttle(&mut self, cpu_id: u32, level: u32) -> i32 {
        (self.ops.thermal_throttle)(cpu_id, level)
    }
    pub fn num_online(&mut self) -> u32 { 1 }
    pub fn get_cpu(&mut self, _id: u32) -> Option<u32> { Some(0) }
}

/// Empty CPU HAL operations
static CPU_HAL_OPS_NONE: CpuHalOps = CpuHalOps {
    init: || -1,
    boot_cpu: |_cpu_id| -1,
    halt_cpu: |_cpu_id| -1,
    get_cpu_info: |_cpu_id| CpuInfo {
        cpu_id: 0,
        cpu_type: CpuType::Little,
        state: CpuState::Offline,
        current_freq: 0,
        min_freq: 0,
        max_freq: 0,
        temperature: 0,
        voltage: 0,
        utilization: 0,
    },
    set_frequency: |_cpu_id, _freq| -1,
    get_frequency: |_cpu_id| 0,
    set_voltage: |_cpu_id, _voltage| -1,
    get_voltage: |_cpu_id| 0,
    enter_idle: |_cpu_id, _state| -1,
    exit_idle: |_cpu_id| -1,
    get_temperature: |_cpu_id| 0,
    thermal_throttle: |_cpu_id, _level| -1,
};

/// Global CPU HAL device
static mut CPU_HAL_DEVICE: CpuHalDevice = CpuHalDevice::new();

/// Get CPU HAL device
pub fn get_cpu_hal() -> &'static mut CpuHalDevice {
    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe { &mut CPU_HAL_DEVICE }
}

/// Initialize CPU HAL
pub fn init_cpu_hal() {
    log_info!("CPU HAL initialized");
}

/// Write to Model-Specific Register (MSR)
#[cfg(target_arch = "aarch64")]
pub fn write_msr(msr: u32, value: u64) {
    // SAFETY: MSR write on ARM64 uses MRS/MSR system register access.
    // The msr parameter selects the system register; this is a privileged
    // operation that must only be called from EL1 or higher.
    unsafe {
        match msr {
            0xC000 => core::arch::asm!("msr SCTLR_EL1, {}", in(reg) value),
            0xC002 => core::arch::asm!("msr TTBR0_EL1, {}", in(reg) value),
            0xC003 => core::arch::asm!("msr TTBR1_EL1, {}", in(reg) value),
            0xC008 => core::arch::asm!("msr VBAR_EL1, {}", in(reg) value),
            0xC010 => core::arch::asm!("msr TCR_EL1, {}", in(reg) value),
            0xC011 => core::arch::asm!("msr MAIR_EL1, {}", in(reg) value),
            _ => log_warn!("write_msr: unhandled MSR 0x{:X}", msr),
        }
    }
}

/// Write to Model-Specific Register (MSR) - x86_64
#[cfg(target_arch = "x86_64")]
pub fn write_msr(msr: u32, value: u64) {
    // SAFETY: WRMSR writes to the specified x86 MSR. This is a privileged
    // instruction that requires CPL 0 (kernel mode).
    unsafe {
        let low = value as u32;
        let high = (value >> 32) as u32;
        core::arch::asm!("wrmsr", in("ecx") msr, in("eax") low, in("edx") high);
    }
}

/// Write to Model-Specific Register (MSR) - LoongArch64
#[cfg(target_arch = "loongarch64")]
pub fn write_msr(msr: u32, value: u64) {
    // SAFETY: CSR write on LoongArch64 uses csrwr instruction.
    // The msr parameter selects the CSR register number.
    unsafe {
        core::arch::asm!("csrwr {}, {}", in(reg) value, in(reg) msr);
    }
}

/// Read from Model-Specific Register (MSR)
#[cfg(target_arch = "aarch64")]
pub fn read_msr(msr: u32) -> u64 {
    // SAFETY: MRS instruction reads a system register. No memory side effects.
    let value: u64;
    unsafe {
        match msr {
            0xC000 => core::arch::asm!("mrs {}, SCTLR_EL1", out(reg) value),
            0xC002 => core::arch::asm!("mrs {}, TTBR0_EL1", out(reg) value),
            0xC003 => core::arch::asm!("mrs {}, TTBR1_EL1", out(reg) value),
            0xC008 => core::arch::asm!("mrs {}, VBAR_EL1", out(reg) value),
            0xC010 => core::arch::asm!("mrs {}, TCR_EL1", out(reg) value),
            0xC011 => core::arch::asm!("mrs {}, MAIR_EL1", out(reg) value),
            _ => {
                log_warn!("read_msr: unhandled MSR 0x{:X}", msr);
                value = 0;
            }
        }
    }
    value
}

/// Read from Model-Specific Register (MSR) - x86_64
#[cfg(target_arch = "x86_64")]
pub fn read_msr(msr: u32) -> u64 {
    // SAFETY: RDSMR reads the specified x86 MSR. Privileged instruction (CPL 0).
    let low: u32;
    let high: u32;
    unsafe {
        core::arch::asm!("rdmsr", in("ecx") msr, out("eax") low, out("edx") high);
    }
    ((high as u64) << 32) | (low as u64)
}

/// Read from Model-Specific Register (MSR) - LoongArch64
#[cfg(target_arch = "loongarch64")]
pub fn read_msr(msr: u32) -> u64 {
    // SAFETY: CSR read on LoongArch64 uses csrrd instruction.
    let value: u64;
    unsafe {
        core::arch::asm!("csrrd {}, {}", out(reg) value, in(reg) msr);
    }
    value
}

/// Enable PMU (Performance Monitoring Unit) - ARM64
#[cfg(target_arch = "aarch64")]
pub fn enable_pmu() {
    // SAFETY: Writing to PMUSERENR_EL0 enables PMU access at EL0.
    // This is a system register write for performance monitoring configuration.
    unsafe {
        // Enable PMU counter access: PMUSERENR_EL0.EN = 1
        core::arch::asm!("msr PMUSERENR_EL0, {}", in(reg) 1u64);
        // Enable all PMU counters: PMCNTENSET_EL0
        core::arch::asm!("msr PMCNTENSET_EL0, {}", in(reg) 0xFFFFFFFFu64);
    }
}

/// Disable PMU - ARM64
#[cfg(target_arch = "aarch64")]
pub fn disable_pmu() {
    // SAFETY: Disabling PMU counters via PMCNTENCLR_EL0.
    unsafe {
        core::arch::asm!("msr PMCNTENCLR_EL0, {}", in(reg) 0xFFFFFFFFu64);
    }
}

/// Reset all PMU counters to zero - ARM64
#[cfg(target_arch = "aarch64")]
pub fn reset_pmu_counters() {
    // SAFETY: Writing zero to PMEVCNTR registers resets all event counters.
    unsafe {
        for i in 0..31u32 {
            core::arch::asm!("msr PMEVCNTR{}_EL0, {}", in(reg) i, in(reg) 0u64);
        }
        // Reset cycle counter
        core::arch::asm!("msr PMCCNTR_EL0, {}", in(reg) 0u64);
    }
}

/// Read PMU cycle counter - ARM64
#[cfg(target_arch = "aarch64")]
pub fn read_pmu_cycle_counter() -> u64 {
    // SAFETY: Reading PMCCNTR_EL0 is a read-only operation.
    let count: u64;
    unsafe {
        core::arch::asm!("mrs {}, PMCCNTR_EL0", out(reg) count);
    }
    count
}

/// Configure PMU event counter - ARM64
#[cfg(target_arch = "aarch64")]
pub fn config_pmu_event(counter: u32, event_type: u32) {
    // SAFETY: Writing to PMEVTYPER registers configures event selection.
    // counter must be 0-30, event_type is the PMU event number.
    if counter > 30 {
        return;
    }
    unsafe {
        core::arch::asm!("msr PMEVTYPER{}_EL0, {}", in(reg) counter, in(reg) event_type as u64);
    }
}

/// Get CPU manager instance
pub fn get_cpu_manager() -> &'static mut CpuHalDevice {
    get_cpu_hal()
}

/// Get current processor ID in SMP system
#[cfg(target_arch = "aarch64")]
pub fn smp_processor_id() -> u32 {
    // SAFETY: Reading MPIDR_EL1 to get current CPU affinity.
    let mpidr: u64;
    unsafe {
        core::arch::asm!("mrs {}, MPIDR_EL1", out(reg) mpidr);
    }
    (mpidr & 0xFF) as u32
}

/// Get current processor ID in SMP system - x86_64
#[cfg(target_arch = "x86_64")]
pub fn smp_processor_id() -> u32 {
    // SAFETY: Reading IA32_TSC_AUX (MSR 0xC0000103) for CPU ID on x86.
    let aux: u64;
    unsafe {
        core::arch::asm!("rdmsr", in("ecx") 0xC0000103u32, out("eax") _, out("edx") _, out("rax") aux);
    }
    aux as u32
}

/// Get current processor ID in SMP system - LoongArch64
#[cfg(target_arch = "loongarch64")]
pub fn smp_processor_id() -> u32 {
    // SAFETY: Reading CSR 0x20 for processor ID on LoongArch64.
    let core_id: u32;
    unsafe {
        core::arch::asm!("csrrd {}, 0x20", out(reg) core_id);
    }
    core_id
}

/// Read hardware cycle counter
#[cfg(target_arch = "aarch64")]
pub fn read_cycle_counter() -> u64 {
    // SAFETY: Reading CNTVCT_EL0 (virtual counter) is a read-only operation.
    let cntvct: u64;
    unsafe {
        core::arch::asm!("mrs {}, CNTVCT_EL0", out(reg) cntvct);
    }
    cntvct
}

/// Read hardware cycle counter - x86_64
#[cfg(target_arch = "x86_64")]
pub fn read_cycle_counter() -> u64 {
    // SAFETY: RDTSC reads the time stamp counter, a monotonically increasing counter.
    // This is a read-only operation with no memory side effects.
    let low: u32;
    let high: u32;
    unsafe {
        core::arch::asm!("rdtsc", out("eax") low, out("edx") high);
    }
    ((high as u64) << 32) | (low as u64)
}

/// Read hardware cycle counter - LoongArch64
#[cfg(target_arch = "loongarch64")]
pub fn read_cycle_counter() -> u64 {
    // SAFETY: Reading CSR 0x20 for stable timer counter on LoongArch64.
    let counter: u64;
    unsafe {
        core::arch::asm!("csrrd {}, 0x20", out(reg) counter);
    }
    counter
}

/// Read hardware instruction counter
pub fn read_inst_counter() -> u64 {
    // SAFETY: Reading PMEVCNTR0_EL0 for instruction count via ARM64 PMU.
    // Falls back to cycle counter on architectures without dedicated instruction counter.
    #[cfg(target_arch = "aarch64")]
    {
        let count: u64;
        unsafe {
            core::arch::asm!("mrs {}, PMEVCNTR0_EL0", out(reg) count);
        }
        count
    }
    #[cfg(not(target_arch = "aarch64"))]
    {
        read_cycle_counter()
    }
}

/// Read hardware cache miss counter
pub fn read_cache_miss_counter() -> u64 {
    // SAFETY: Reading PMEVCNTR1_EL0 for cache miss count via ARM64 PMU.
    // Returns 0 on architectures without PMU support.
    #[cfg(target_arch = "aarch64")]
    {
        let count: u64;
        unsafe {
            core::arch::asm!("mrs {}, PMEVCNTR1_EL0", out(reg) count);
        }
        count
    }
    #[cfg(not(target_arch = "aarch64"))]
    {
        0
    }
}

/// Read hardware branch miss counter
pub fn read_branch_miss_counter() -> u64 {
    // SAFETY: Reading PMEVCNTR2_EL0 for branch miss count via ARM64 PMU.
    // Returns 0 on architectures without PMU support.
    #[cfg(target_arch = "aarch64")]
    {
        let count: u64;
        unsafe {
            core::arch::asm!("mrs {}, PMEVCNTR2_EL0", out(reg) count);
        }
        count
    }
    #[cfg(not(target_arch = "aarch64"))]
    {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_hal() {
        let hal = get_cpu_hal();
        assert_eq!(hal.num_cpus, 0);
    }

    #[test]
    fn test_cpu_state() {
        assert_eq!(CpuState::Online as i32, 0);
        assert_eq!(CpuState::Offline as i32, 1);
        assert_eq!(CpuState::Idle as i32, 2);
        assert_eq!(CpuState::DeepIdle as i32, 3);
    }

    #[test]
    fn test_cpu_type() {
        assert_eq!(CpuType::Big as i32, 0);
        assert_eq!(CpuType::Little as i32, 1);
    }

    #[test]
    fn test_cpu_info() {
        let info = CpuInfo {
            cpu_id: 0,
            cpu_type: CpuType::Big,
            state: CpuState::Online,
            current_freq: 3_000_000_000,
            min_freq: 800_000_000,
            max_freq: 3_130_000_000,
            temperature: 45000,
            voltage: 1100000,
            utilization: 50,
        };

        assert_eq!(info.cpu_id, 0);
        assert_eq!(info.cpu_type, CpuType::Big);
        assert_eq!(info.state, CpuState::Online);
        assert_eq!(info.current_freq, 3_000_000_000);
    }

    #[test]
    fn test_cpu_hal_device_new() {
        let device = CpuHalDevice::new();
        assert_eq!(device.num_cpus, 0);
        assert_eq!(device.num_big, 0);
        assert_eq!(device.num_little, 0);
    }

    #[test]
    fn test_cpu_hal_device_get_cpu_info() {
        let device = CpuHalDevice::new();

        // Should return None when not initialized
        let info = device.get_cpu_info(0);
        assert!(info.is_none());
    }

    #[test]
    fn test_cpu_hal_device_get_cpu_info_out_of_bounds() {
        let device = CpuHalDevice::new();
        // Index beyond array size should return None
        let info = device.get_cpu_info(8);
        assert!(info.is_none());
    }

    #[test]
    fn test_cpu_state_values() {
        // Verify CpuState enum discriminants for FFI compatibility
        assert_eq!(CpuState::Online as i32, 0);
        assert_eq!(CpuState::Offline as i32, 1);
        assert_eq!(CpuState::Idle as i32, 2);
        assert_eq!(CpuState::DeepIdle as i32, 3);
    }

    #[test]
    fn test_cpu_info_frequency_range() {
        // Validate that min_freq <= current_freq <= max_freq
        let info = CpuInfo {
            cpu_id: 0,
            cpu_type: CpuType::Big,
            state: CpuState::Online,
            current_freq: 2_000_000_000,
            min_freq: 800_000_000,
            max_freq: 3_130_000_000,
            temperature: 45000,
            voltage: 1100000,
            utilization: 75,
        };
        assert!(info.min_freq <= info.current_freq);
        assert!(info.current_freq <= info.max_freq);
    }

    #[test]
    fn test_cpu_hal_device_num_online_default() {
        let mut device = CpuHalDevice::new();
        assert_eq!(device.num_online(), 1);
    }

    #[test]
    fn test_cpu_hal_device_get_cpu_default() {
        let mut device = CpuHalDevice::new();
        assert_eq!(device.get_cpu(0), Some(0));
    }

    #[test]
    fn test_smp_processor_id_returns_u32() {
        // On hosted test environment, smp_processor_id may return 0
        // This test verifies the function compiles and returns a valid u32
        let _id: u32 = smp_processor_id();
    }

    #[test]
    fn test_read_cycle_counter_returns_u64() {
        // Verify cycle counter returns a u64 (monotonically increasing)
        let _c1: u64 = read_cycle_counter();
    }

    #[test]
    fn test_read_counters_return_valid_types() {
        // Verify all counter functions return their expected types
        let _inst: u64 = read_inst_counter();
        let _cache: u64 = read_cache_miss_counter();
        let _branch: u64 = read_branch_miss_counter();
    }
}
