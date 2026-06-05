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



// Re-export print macros from crate root
pub use crate::{pr_emerg, pr_alert, pr_crit, pr_err, pr_warn, pr_notice, pr_info, pr_debug};
use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

// PMIC driver module
pub mod pmic;

// Suspend/resume module
pub mod suspend;

/// Power state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerState {
    /// Running
    Running = 0,
    /// Idle
    Idle = 1,
    /// Suspend
    Suspend = 2,
    /// Hibernate
    Hibernate = 3,
    /// Power off
    Off = 4,
}

/// Power domain type
#[derive(Debug, Clone, Copy)]
pub enum PowerDomainType {
    /// CPU
    Cpu = 0,
    /// GPU
    Gpu = 1,
    /// NPU
    Npu = 2,
    /// Memory
    Memory = 3,
    /// Peripheral
    Peripheral = 4,
    /// Display
    Display = 5,
}

/// Power domain state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerDomainState {
    /// On
    On = 0,
    /// Partial on
    Partial = 1,
    /// Off
    Off = 2,
}

/// Battery status
#[derive(Debug, Clone, Copy)]
pub struct BatteryStatus {
    /// If present
    pub present: bool,
    /// If charging
    pub charging: bool,
    /// Capacity percentage
    pub capacity: u32,
    /// Voltage (millivolts)
    pub voltage: u32,
    /// Current (milliamps)
    pub current: i32,
    /// Temperature (millidegrees)
    pub temperature: i32,
    /// Health
    pub health: u32,
}

/// Power info
pub struct PowerInfo {
    /// Current power state
    pub state: PowerState,
    /// AC power online
    pub ac_online: bool,
    /// USB power online
    pub usb_online: bool,
    /// Battery status
    pub battery: BatteryStatus,
}

// ============================================================================
// CPU power states (ACPI C-states)
// ============================================================================

/// CPU idle state (ACPI C-state)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CpuIdleState {
    /// C0: Running state
    C0 = 0,
    /// C1: HLT (Stop Clock)
    C1 = 1,
    /// C2: Stop Clock and bus
    C2 = 2,
    /// C3: Deep sleep (Stop all clocks)
    C3 = 3,
    /// C4: Deeper sleep (Lower voltage)
    C4 = 4,
    /// C5: Deepest sleep
    C5 = 5,
    /// C6: Deepest sleep (Power off)
    C6 = 6,
}

/// CPU idle state info
pub struct CpuIdleStateInfo {
    /// State name
    pub name: &'static str,
    /// Entry latency (microseconds)
    pub latency: u32,
    /// Target residency (microseconds)
    pub target_residency: u32,
    /// Power saving (milliwatts)
    pub power_saving: u32,
    /// Entry count
    pub usage: AtomicU64,
    /// Total residency time (microseconds)
    pub time: AtomicU64,
}

impl CpuIdleStateInfo {
    pub const fn new(name: &'static str, latency: u32, target_residency: u32, power_saving: u32) -> Self {
        CpuIdleStateInfo {
            name,
            latency,
            target_residency,
            power_saving,
            usage: AtomicU64::new(0),
            time: AtomicU64::new(0),
        }
    }
}

/// CPU idle manager
pub struct CpuIdleManager {
    /// Current state
    pub current_state: AtomicU32,
    /// Last idle entry time
    pub last_idle_time: AtomicU64,
    /// Total idle time
    pub total_idle_time: AtomicU64,
    /// Idle state array
    pub states: [CpuIdleStateInfo; 7],
    /// Deepest idle state
    pub deepest_state: AtomicU32,
}

impl CpuIdleManager {
    pub const fn new() -> Self {
        CpuIdleManager {
            current_state: AtomicU32::new(CpuIdleState::C0 as u32),
            last_idle_time: AtomicU64::new(0),
            total_idle_time: AtomicU64::new(0),
            states: [
                CpuIdleStateInfo::new("C0", 0, 0, 0),
                CpuIdleStateInfo::new("C1", 1, 1, 10),
                CpuIdleStateInfo::new("C2", 10, 100, 50),
                CpuIdleStateInfo::new("C3", 100, 1000, 100),
                CpuIdleStateInfo::new("C4", 200, 2000, 200),
                CpuIdleStateInfo::new("C5", 400, 4000, 300),
                CpuIdleStateInfo::new("C6", 800, 8000, 500),
            ],
            deepest_state: AtomicU32::new(CpuIdleState::C3 as u32),
        }
    }

    /// Select optimal idle state
    pub fn select_idle_state(&self, predicted_idle_time: u64) -> CpuIdleState {
        let deepest = self.deepest_state.load(Ordering::Acquire) as usize;

        // Start from deepest state
        for i in (1..=deepest).rev() {
            let target = self.states[i].target_residency as u64;
            if predicted_idle_time >= target {
                return match i {
                    1 => CpuIdleState::C1,
                    2 => CpuIdleState::C2,
                    3 => CpuIdleState::C3,
                    4 => CpuIdleState::C4,
                    5 => CpuIdleState::C5,
                    6 => CpuIdleState::C6,
                    _ => CpuIdleState::C1,
                };
            }
        }

        CpuIdleState::C1
    }

    /// Enter idle state
    pub fn enter_idle(&self, state: CpuIdleState) {
        self.current_state.store(state as u32, Ordering::Release);
        self.states[state as usize].usage.fetch_add(1, Ordering::AcqRel);

        // Execute idle instruction
        match state {
            CpuIdleState::C0 => {}
            CpuIdleState::C1 => self.enter_c1(),
            CpuIdleState::C2 => self.enter_c2(),
            CpuIdleState::C3 => self.enter_c3(),
            CpuIdleState::C4 => self.enter_c4(),
            CpuIdleState::C5 => self.enter_c5(),
            CpuIdleState::C6 => self.enter_c6(),
        }
    }

    /// Exit idle state
    pub fn exit_idle(&self) {
        self.current_state.store(CpuIdleState::C0 as u32, Ordering::Release);
    }

    fn enter_c1(&self) {
        // HLT instruction (x86 halt until next interrupt)
        // SAFETY: inline assembly required for hardware instruction
        unsafe { core::arch::asm!("hlt", options(nomem, nostack)); }
    }

    fn enter_c2(&self) {
        // C2: Stop Clock and bus
        // SAFETY: unsafe block required for low-level memory or hardware access
        #[cfg(target_arch = "x86_64")]
        unsafe {
            let eax: u32 = 0x01; // C2 state hint
            let ecx: u32 = 0;
            core::arch::asm!("mwait", in("eax") eax, in("ecx") ecx, options(nomem, nostack));
        }
        #[cfg(target_arch = "aarch64")]
        unsafe {
            core::arch::asm!("wfi");
        }
        #[cfg(target_arch = "loongarch64")]
        unsafe {
            core::arch::asm!("idle 0");
        }
    }

    fn enter_c3(&self) {
        // C3: Deep sleep (Stop all clocks)
        // SAFETY: unsafe block required for low-level memory or hardware access
        #[cfg(target_arch = "x86_64")]
        unsafe {
            let eax: u32 = 0x10; // C3 state hint
            let ecx: u32 = 0;
            core::arch::asm!("mwait", in("eax") eax, in("ecx") ecx, options(nomem, nostack));
        }
        #[cfg(target_arch = "aarch64")]
        unsafe {
            core::arch::asm!("wfi");
        }
        #[cfg(target_arch = "loongarch64")]
        unsafe {
            core::arch::asm!("idle 0");
        }
    }

    fn enter_c4(&self) {
        // C4: Deeper sleep (Lower voltage)
        // SAFETY: unsafe block required for low-level memory or hardware access
        #[cfg(target_arch = "x86_64")]
        unsafe {
            let eax: u32 = 0x20; // C4 state hint
            let ecx: u32 = 0;
            core::arch::asm!("mwait", in("eax") eax, in("ecx") ecx, options(nomem, nostack));
        }
        #[cfg(target_arch = "aarch64")]
        unsafe {
            core::arch::asm!("wfi");
        }
        #[cfg(target_arch = "loongarch64")]
        unsafe {
            core::arch::asm!("idle 0");
        }
    }

    fn enter_c5(&self) {
        // C5: Deepest sleep
        // SAFETY: unsafe block required for low-level memory or hardware access
        #[cfg(target_arch = "x86_64")]
        unsafe {
            let eax: u32 = 0x30; // C5 state hint
            let ecx: u32 = 0;
            core::arch::asm!("mwait", in("eax") eax, in("ecx") ecx, options(nomem, nostack));
        }
        #[cfg(target_arch = "aarch64")]
        unsafe {
            core::arch::asm!("wfi");
        }
        #[cfg(target_arch = "loongarch64")]
        unsafe {
            core::arch::asm!("idle 0");
        }
    }

    fn enter_c6(&self) {
        // C6: Power off (deepest C-state)
        // SAFETY: unsafe block required for low-level memory or hardware access
        #[cfg(target_arch = "x86_64")]
        unsafe {
            let eax: u32 = 0x40; // C6 state hint
            let ecx: u32 = 0;
            core::arch::asm!("mwait", in("eax") eax, in("ecx") ecx, options(nomem, nostack));
        }
        #[cfg(target_arch = "aarch64")]
        unsafe {
            core::arch::asm!("wfi");
        }
        #[cfg(target_arch = "loongarch64")]
        unsafe {
            core::arch::asm!("idle 0");
        }
    }

    /// Get idle time percentage
    pub fn get_idle_percent(&self, total_time: u64) -> u32 {
        let idle_time = self.total_idle_time.load(Ordering::Acquire);
        if total_time == 0 {
            return 0;
        }
        ((idle_time * 100) / total_time) as u32
    }
}

// ============================================================================
// CPU frequency scaling (DVFS)
// ============================================================================

/// CPU frequency scaling policy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CpuFreqPolicy {
    /// Performance priority
    Performance = 0,
    /// On-demand scaling
    Ondemand = 1,
    /// Conservative scaling
    Conservative = 2,
    /// Power saving priority
    Powersave = 3,
    /// User specified
    Userspace = 4,
}

/// CPU frequency info
pub struct CpuFreqInfo {
    /// Current frequency (kHz)
    pub cur_freq: AtomicU32,
    /// Minimum frequency (kHz)
    pub min_freq: u32,
    /// Maximum frequency (kHz)
    pub max_freq: u32,
    /// Scaling policy
    pub policy: AtomicU32,
    /// Frequency transition count
    pub transitions: AtomicU64,
}

impl CpuFreqInfo {
    pub const fn new(min_freq: u32, max_freq: u32) -> Self {
        CpuFreqInfo {
            cur_freq: AtomicU32::new(max_freq),
            min_freq,
            max_freq,
            policy: AtomicU32::new(CpuFreqPolicy::Ondemand as u32),
            transitions: AtomicU64::new(0),
        }
    }

    /// Set frequency
    pub fn set_freq(&self, freq: u32) -> i32 {
        if freq < self.min_freq || freq > self.max_freq {
            return -1;
        }

        self.cur_freq.store(freq, Ordering::Release);
        self.transitions.fetch_add(1, Ordering::AcqRel);

        dvfs_set_frequency(freq)
    }

    /// Get frequency
    pub fn get_freq(&self) -> u32 {
        self.cur_freq.load(Ordering::Acquire)
    }
}

/// CPU frequency manager
pub struct CpuFreqManager {
    /// Each CPU frequency info
    pub cpu_freq: [CpuFreqInfo; 8],
    /// Number of CPUs
    pub num_cpus: u32,
    /// On-demand scaling parameters
    pub ondemand_up_threshold: u32,
    pub ondemand_sampling_rate: u32,
}

impl CpuFreqManager {
    pub const fn new() -> Self {
        CpuFreqManager {
            cpu_freq: [
                CpuFreqInfo::new(300000, 2200000),  // 300 MHz - 2.2 GHz
                CpuFreqInfo::new(300000, 2200000),
                CpuFreqInfo::new(300000, 2200000),
                CpuFreqInfo::new(300000, 2200000),
                CpuFreqInfo::new(300000, 2200000),
                CpuFreqInfo::new(300000, 2200000),
                CpuFreqInfo::new(300000, 2200000),
                CpuFreqInfo::new(300000, 2200000),
            ],
            num_cpus: 4,
            ondemand_up_threshold: 80,  // 80% CPU usage
            ondemand_sampling_rate: 10000,  // 10 ms
        }
    }

    /// Set policy
    pub fn set_policy(&self, cpu: u32, policy: CpuFreqPolicy) -> i32 {
        if cpu as usize >= self.cpu_freq.len() {
            return -1;
        }

        self.cpu_freq[cpu as usize].policy.store(policy as u32, Ordering::Release);

        match policy {
            CpuFreqPolicy::Performance => {
                // Set to maximum frequency
                self.cpu_freq[cpu as usize].set_freq(self.cpu_freq[cpu as usize].max_freq);
            }
            CpuFreqPolicy::Powersave => {
                // Set to minimum frequency
                self.cpu_freq[cpu as usize].set_freq(self.cpu_freq[cpu as usize].min_freq);
            }
            _ => {}
        }

        0
    }

    /// On-demand scaling
    pub fn ondemand_update(&self, cpu: u32, load: u32) {
        if cpu as usize >= self.cpu_freq.len() {
            return;
        }

        let freq_info = &self.cpu_freq[cpu as usize];
        let cur_freq = freq_info.get_freq();

        if load > self.ondemand_up_threshold {
            // High load, increase frequency
            let target_freq = (cur_freq as u64 * 120 / 100).min(freq_info.max_freq as u64) as u32;
            freq_info.set_freq(target_freq);
        } else {
            // Low load, decrease frequency
            let target_freq = (cur_freq as u64 * 90 / 100).max(freq_info.min_freq as u64) as u32;
            freq_info.set_freq(target_freq);
        }
    }
}

// ============================================================================
// Power HAL operations
// ============================================================================

/// Power HAL operations
pub struct PowerHalOps {
    /// Initialize
    pub init: fn() -> i32,
    /// Get power info
    pub get_power_info: fn() -> PowerInfo,
    /// Set power state
    pub set_power_state: fn(state: PowerState) -> i32,
    /// Get power domain state
    pub get_domain_state: fn(domain: PowerDomainType) -> PowerDomainState,
    /// Set power domain state
    pub set_domain_state: fn(domain: PowerDomainType, state: PowerDomainState) -> i32,
    /// Enter suspend
    pub suspend: fn() -> i32,
    /// Resume
    pub resume: fn() -> i32,
    /// Power off
    pub power_off: fn() -> i32,
    /// Reboot
    pub reboot: fn() -> i32,
}

/// Power HAL device
pub struct PowerHalDevice {
    /// Power info
    pub info: PowerInfo,
    /// HAL operations
    pub ops: &'static PowerHalOps,
    /// Number of power domains
    pub num_domains: u32,
    /// CPU idle manager
    pub idle_manager: CpuIdleManager,
    /// CPU frequency manager
    pub freq_manager: CpuFreqManager,
}

impl PowerHalDevice {
    pub const fn new() -> Self {
        PowerHalDevice {
            info: PowerInfo {
                state: PowerState::Running,
                ac_online: false,
                usb_online: false,
                battery: BatteryStatus {
                    present: false,
                    charging: false,
                    capacity: 0,
                    voltage: 0,
                    current: 0,
                    temperature: 0,
                    health: 0,
                },
            },
            ops: &POWER_HAL_OPS_NONE,
            num_domains: 0,
            idle_manager: CpuIdleManager::new(),
            freq_manager: CpuFreqManager::new(),
        }
    }

    /// Initialize
    pub fn init(&mut self) -> i32 {
        (self.ops.init)()
    }
}

/// Empty power HAL operations
static POWER_HAL_OPS_NONE: PowerHalOps = PowerHalOps {
    init: || -1,
    get_power_info: || PowerInfo {
        state: PowerState::Running,
        ac_online: false,
        usb_online: false,
        battery: BatteryStatus {
            present: false,
            charging: false,
            capacity: 0,
            voltage: 0,
            current: 0,
            temperature: 0,
            health: 0,
        },
    },
    set_power_state: |_state| -1,
    get_domain_state: |_domain| PowerDomainState::Off,
    set_domain_state: |_domain, _state| -1,
    suspend: || -1,
    resume: || -1,
    power_off: || -1,
    reboot: || -1,
};

/// Global power HAL device
static mut POWER_HAL_DEVICE: PowerHalDevice = PowerHalDevice::new();

pub fn get_power_hal() -> &'static mut PowerHalDevice {
    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe { &mut POWER_HAL_DEVICE }
}

pub fn init_power_hal() {
    log_info!("Power HAL initialized");
}

// ============================================================================
// DVFS Hardware Interface
// ============================================================================

const DVFS_CTRL_BASE: u64 = 0x0B000000;
const DVFS_CTRL_FREQ: u64 = 0x000;
const DVFS_CTRL_VOLT: u64 = 0x004;
const DVFS_CTRL_STATUS: u64 = 0x008;

pub fn dvfs_set_frequency(freq_khz: u32) -> i32 {
    unsafe {
        core::ptr::write_volatile((DVFS_CTRL_BASE + DVFS_CTRL_FREQ) as *mut u32, freq_khz);
        let mut timeout = 100_000u32;
        while timeout > 0 {
            let status = core::ptr::read_volatile((DVFS_CTRL_BASE + DVFS_CTRL_STATUS) as *const u32);
            if status & 0x1 != 0 { return 0; }
            timeout -= 1;
        }
    }
    -1
}

pub fn dvfs_set_voltage(voltage_uv: u32) -> i32 {
    unsafe {
        core::ptr::write_volatile((DVFS_CTRL_BASE + DVFS_CTRL_VOLT) as *mut u32, voltage_uv);
    }
    0
}

// ============================================================================
// Thermal Management
// ============================================================================

const THERMAL_SENSOR_BASE: u64 = 0x0C000000;
const THERMAL_SENSOR_TEMP: u64 = 0x000;
const THERMAL_SENSOR_THRESH: u64 = 0x004;
const THERMAL_SENSOR_CTRL: u64 = 0x008;

const THERMAL_TRIP_PASSIVE: u32 = 85000;
const THERMAL_TRIP_CRITICAL: u32 = 105000;

pub fn thermal_read_temperature() -> i32 {
    unsafe {
        core::ptr::read_volatile((THERMAL_SENSOR_BASE + THERMAL_SENSOR_TEMP) as *const i32)
    }
}

pub fn thermal_set_threshold(temp_mc: i32) -> i32 {
    unsafe {
        core::ptr::write_volatile((THERMAL_SENSOR_BASE + THERMAL_SENSOR_THRESH) as *mut i32, temp_mc);
    }
    0
}

pub fn thermal_check_and_throttle() -> i32 {
    let temp = thermal_read_temperature();
    if temp >= THERMAL_TRIP_CRITICAL as i32 {
        log_error!("Thermal: Critical temperature {} mC, emergency shutdown", temp);
        return -2;
    }
    if temp >= THERMAL_TRIP_PASSIVE as i32 {
        let hal = get_power_hal();
        for i in 0..hal.freq_manager.num_cpus as usize {
            let info = &hal.freq_manager.cpu_freq[i];
            let cur = info.get_freq();
            let throttled = cur * 80 / 100;
            let _ = info.set_freq(throttled.max(info.min_freq));
        }
        return -1;
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_power_state() {
        assert_eq!(PowerState::Running as i32, 0);
        assert_eq!(PowerState::Idle as i32, 1);
        assert_eq!(PowerState::Suspend as i32, 2);
        assert_eq!(PowerState::Hibernate as i32, 3);
        assert_eq!(PowerState::Off as i32, 4);
    }

    #[test]
    fn test_power_domain_type() {
        assert_eq!(PowerDomainType::Cpu as i32, 0);
        assert_eq!(PowerDomainType::Gpu as i32, 1);
        assert_eq!(PowerDomainType::Npu as i32, 2);
        assert_eq!(PowerDomainType::Memory as i32, 3);
        assert_eq!(PowerDomainType::Peripheral as i32, 4);
        assert_eq!(PowerDomainType::Display as i32, 5);
    }

    #[test]
    fn test_power_domain_state() {
        assert_eq!(PowerDomainState::On as i32, 0);
        assert_eq!(PowerDomainState::Partial as i32, 1);
        assert_eq!(PowerDomainState::Off as i32, 2);
    }

    #[test]
    fn test_cpu_idle_state() {
        assert_eq!(CpuIdleState::C0 as i32, 0);
        assert_eq!(CpuIdleState::C1 as i32, 1);
        assert_eq!(CpuIdleState::C2 as i32, 2);
        assert_eq!(CpuIdleState::C3 as i32, 3);
        assert_eq!(CpuIdleState::C4 as i32, 4);
        assert_eq!(CpuIdleState::C5 as i32, 5);
        assert_eq!(CpuIdleState::C6 as i32, 6);
    }

    #[test]
    fn test_cpu_freq_policy() {
        assert_eq!(CpuFreqPolicy::Performance as i32, 0);
        assert_eq!(CpuFreqPolicy::Ondemand as i32, 1);
        assert_eq!(CpuFreqPolicy::Conservative as i32, 2);
        assert_eq!(CpuFreqPolicy::Powersave as i32, 3);
        assert_eq!(CpuFreqPolicy::Userspace as i32, 4);
    }

    #[test]
    fn test_cpu_idle_manager() {
        let manager = CpuIdleManager::new();
        assert_eq!(manager.current_state.load(Ordering::Acquire), CpuIdleState::C0 as u32);
    }

    #[test]
    fn test_cpu_freq_manager() {
        let manager = CpuFreqManager::new();
        assert_eq!(manager.num_cpus, 4);
        assert_eq!(manager.ondemand_up_threshold, 80);
    }

    #[test]
    fn test_power_hal_device() {
        let device = PowerHalDevice::new();
        assert_eq!(device.num_domains, 0);
    }
}
