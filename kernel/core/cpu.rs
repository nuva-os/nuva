/*
 * Nuva OS - Kernel - Core - Cpu
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
use crate::{pr_info};
/*
 * Nuva OS - Kernel - CPU Management
 * 
 * Copyright (C) 2026 Nuva OS Team
 * 
 * CPU topology and management.
 */

use core::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};

use crate::posix::errno::Errno;
/// CPU ID
pub type CpuId = u32;

/// CPU State
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CpuState {
    /// Offline
    Offline = 0,
    /// Booting
    Booting = 1,
    /// Online
    Online = 2,
    /// Hotplug
    Hotplug = 3,
}

/// CPU Info
#[repr(C)]
pub struct CpuInfo {
    /// CPU ID
    pub id: CpuId,
    /// State
    pub state: AtomicU32,
    /// Online
    pub online: AtomicBool,
    /// Present
    pub present: AtomicBool,
    /// Possible
    pub possible: AtomicBool,
    /// Architecture
    pub arch: CpuArch,
    /// Vendor
    pub vendor: CpuVendor,
    /// Frequency (Hz)
    pub frequency: AtomicU64,
    /// Max frequency
    pub max_freq: u64,
    /// Min frequency
    pub min_freq: u64,
    /// Core ID
    pub core_id: u32,
    /// Socket ID
    pub socket_id: u32,
    /// Thread ID (SMT)
    pub thread_id: u32,
    /// SMT siblings
    pub smt_siblings: u32,
    /// Cache info
    pub cache: CacheInfo,
    /// Topology
    pub topology: CpuTopology,
}

/// CPU Architecture
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CpuArch {
    /// Unknown
    Unknown = 0,
    /// ARM64
    AArch64 = 1,
    /// x86-64
    X86_64 = 2,
    /// RISC-V
    RiscV = 3,
}

/// CPU Vendor
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CpuVendor {
    Unknown = 0,
    // ARM vendors
    Arm = 1,
    Qualcomm = 2,
    Apple = 3,
    MediaTek = 4,
    Samsung = 5,
    HiSilicon = 6,
    // x86 vendors
    Intel = 10,
    Amd = 11,
}

/// Cache Info
#[repr(C)]
pub struct CacheInfo {
    /// L1 instruction cache size
    pub l1i_size: u32,
    /// L1 data cache size
    pub l1d_size: u32,
    /// L2 cache size
    pub l2_size: u32,
    /// L3 cache size
    pub l3_size: u32,
    /// Cache line size
    pub line_size: u32,
}

/// CPU Topology
#[repr(C)]
pub struct CpuTopology {
    /// Core siblings mask
    pub core_siblings: u64,
    /// Thread siblings mask
    pub thread_siblings: u64,
    /// NUMA node
    pub node: u32,
}

/// CPU Statistics
#[repr(C)]
pub struct CpuStats {
    /// User time
    pub user: AtomicU64,
    /// Nice time
    pub nice: AtomicU64,
    /// System time
    pub system: AtomicU64,
    /// Idle time
    pub idle: AtomicU64,
    /// I/O wait time
    pub iowait: AtomicU64,
    /// IRQ time
    pub irq: AtomicU64,
    /// Soft IRQ time
    pub softirq: AtomicU64,
    /// Steal time
    pub steal: AtomicU64,
    /// Guest time
    pub guest: AtomicU64,
    /// Guest nice time
    pub guest_nice: AtomicU64,
}

impl Clone for CpuStats {
    fn clone(&self) -> Self {
        CpuStats {
            user: AtomicU64::new(self.user.load(Ordering::Relaxed)),
            nice: AtomicU64::new(self.nice.load(Ordering::Relaxed)),
            system: AtomicU64::new(self.system.load(Ordering::Relaxed)),
            idle: AtomicU64::new(self.idle.load(Ordering::Relaxed)),
            iowait: AtomicU64::new(self.iowait.load(Ordering::Relaxed)),
            irq: AtomicU64::new(self.irq.load(Ordering::Relaxed)),
            softirq: AtomicU64::new(self.softirq.load(Ordering::Relaxed)),
            steal: AtomicU64::new(self.steal.load(Ordering::Relaxed)),
            guest: AtomicU64::new(self.guest.load(Ordering::Relaxed)),
            guest_nice: AtomicU64::new(self.guest_nice.load(Ordering::Relaxed)),
        }
    }
}

impl CpuStats {
    pub const fn new() -> Self {
        CpuStats {
            user: AtomicU64::new(0),
            nice: AtomicU64::new(0),
            system: AtomicU64::new(0),
            idle: AtomicU64::new(0),
            iowait: AtomicU64::new(0),
            irq: AtomicU64::new(0),
            softirq: AtomicU64::new(0),
            steal: AtomicU64::new(0),
            guest: AtomicU64::new(0),
            guest_nice: AtomicU64::new(0),
        }
    }
}

/// Per-CPU Data
#[repr(C)]
pub struct PerCpuData {
    /// CPU ID
    pub cpu_id: CpuId,
    /// Current process
    pub current: u64,
    /// Idle process
    pub idle: u64,
    /// Kernel stack
    pub kstack: u64,
    /// Interrupt stack
    pub istack: u64,
    /// Statistics
    pub stats: CpuStats,
    /// Soft IRQ pending
    pub softirq_pending: AtomicU32,
    /// Need reschedule
    pub need_resched: AtomicBool,
    /// In interrupt
    pub in_interrupt: AtomicU32,
    /// Preempt count
    pub preempt_count: AtomicU32,
}

impl Clone for PerCpuData {
    fn clone(&self) -> Self {
        Self {
            cpu_id: self.cpu_id.clone(),
            current: self.current.clone(),
            idle: self.idle.clone(),
            kstack: self.kstack.clone(),
            istack: self.istack.clone(),
            stats: self.stats.clone(),
            softirq_pending: AtomicU32::new(self.softirq_pending.load(core::sync::atomic::Ordering::Relaxed)),
            need_resched: AtomicBool::new(self.need_resched.load(core::sync::atomic::Ordering::Relaxed)),
            in_interrupt: AtomicU32::new(self.in_interrupt.load(core::sync::atomic::Ordering::Relaxed)),
            preempt_count: AtomicU32::new(self.preempt_count.load(core::sync::atomic::Ordering::Relaxed)),
        }
    }
}

/// CPU Manager
pub struct CpuManager {
    /// CPU info array
    pub cpus: [Option<CpuInfo>; 256],
    /// Per-CPU data
    pub percpu: [PerCpuData; 256],
    /// Number of possible CPUs
    pub nr_possible: AtomicU32,
    /// Number of present CPUs
    pub nr_present: AtomicU32,
    /// Number of online CPUs
    pub nr_online: AtomicU32,
    /// Boot CPU
    pub boot_cpu: AtomicU32,
    /// Statistics
    pub stats: CpuMgrStats,
}

/// CPU Manager Statistics
pub struct CpuMgrStats {
    pub context_switches: AtomicU64,
    pub migrations: AtomicU64,
    pub hotplugs: AtomicU64,
}

impl CpuMgrStats {
    pub const fn new() -> Self {
        CpuMgrStats {
            context_switches: AtomicU64::new(0),
            migrations: AtomicU64::new(0),
            hotplugs: AtomicU64::new(0),
        }
    }
}

impl CpuManager {
    pub const fn new() -> Self {
        CpuManager {
            cpus: [const { None }; 256],
            percpu: [const { PerCpuData {
                cpu_id: 0,
                current: 0,
                idle: 0,
                kstack: 0,
                istack: 0,
                stats: CpuStats::new(),
                softirq_pending: AtomicU32::new(0),
                need_resched: AtomicBool::new(false),
                in_interrupt: AtomicU32::new(0),
                preempt_count: AtomicU32::new(0),
            } }; 256],
            nr_possible: AtomicU32::new(1),
            nr_present: AtomicU32::new(1),
            nr_online: AtomicU32::new(1),
            boot_cpu: AtomicU32::new(0),
            stats: CpuMgrStats::new(),
        }
    }
    
    /// Initialize
    pub fn init(&self) {
        // Detect CPU topology
        self.detect_topology();
        
        // Initialize boot CPU
        self.init_boot_cpu();
        
        log_info!("CPU manager initialized");
        log_info!("  Possible CPUs: {}", self.nr_possible.load(Ordering::Acquire));
        log_info!("  Present CPUs: {}", self.nr_present.load(Ordering::Acquire));
        log_info!("  Online CPUs: {}", self.nr_online.load(Ordering::Acquire));
    }
    
    /// Detect CPU topology
    fn detect_topology(&mut self) {
        #[cfg(target_arch = "aarch64")]
        {
            self.detect_arm_topology();
        }
        
        #[cfg(target_arch = "x86_64")]
        {
            self.detect_x86_topology();
        }
    }
    
    /// Detect ARM topology
    #[cfg(target_arch = "aarch64")]
    fn detect_arm_topology(&mut self) {
        // Read MPIDR_EL1
        let mpidr: u64;
        // SAFETY: inline assembly required for hardware instruction
        unsafe {
            core::arch::asm!(
                "mrs {0}, mpidr_el1",
                out(reg) mpidr,
                options(nostack, preserves_flags)
            );
        }
        
        // Extract affinity levels
        let _aff0 = mpidr & 0xFF;
        let _aff1 = (mpidr >> 8) & 0xFF;
        let _aff2 = (mpidr >> 16) & 0xFF;
        let _aff3 = (mpidr >> 32) & 0xFF;
        
        // Parse ACPI/Device Tree for full topology
        // In a real implementation, this would:
        // 1. Read the ACPI MADT table or Device Tree CPU nodes
        // 2. Determine the number of possible/present CPUs
        // 3. Build the full topology (sockets, cores, threads)
        // For now, we use the MPIDR affinity levels as a basic hint
        // and rely on PSCI for CPU power management.
    }
    
    /// Detect x86 topology
    #[cfg(target_arch = "x86_64")]
    fn detect_x86_topology(&mut self) {
        // Use CPUID to detect topology
        // CPUID leaf 0xB provides extended topology information
        let mut _max_cpuid: u32 = 0;
        // SAFETY: inline assembly required for hardware instruction
        unsafe {
            core::arch::asm!(
                "cpuid",
                inout("eax") 0 => _max_cpuid,
                out("ebx") _,
                out("ecx") _,
                out("edx") _,
                options(nostack, preserves_flags)
            );
        }
        // Basic CPUID support detected, topology parsing would follow
    }
    
    /// Initialize boot CPU
    fn init_boot_cpu(&mut self) {
        let cpu = CpuInfo {
            id: 0,
            state: AtomicU32::new(CpuState::Online as u32),
            online: AtomicBool::new(true),
            present: AtomicBool::new(true),
            possible: AtomicBool::new(true),
            arch: self.detect_arch(),
            vendor: self.detect_vendor(),
            frequency: AtomicU64::new(0),
            max_freq: 0,
            min_freq: 0,
            core_id: 0,
            socket_id: 0,
            thread_id: 0,
            smt_siblings: 1,
            cache: CacheInfo {
                l1i_size: 0,
                l1d_size: 0,
                l2_size: 0,
                l3_size: 0,
                line_size: 64,
            },
            topology: CpuTopology {
                core_siblings: 1,
                thread_siblings: 1,
                node: 0,
            },
        };
        
        self.cpus[0] = Some(cpu);
        self.percpu[0].cpu_id = 0;
        self.boot_cpu.store(0, Ordering::Release);
    }
    
    /// Detect architecture
    fn detect_arch(&self) -> CpuArch {
        #[cfg(target_arch = "aarch64")]
        {
            CpuArch::AArch64
        }
        
        #[cfg(target_arch = "x86_64")]
        {
            CpuArch::X86_64
        }
        
        #[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
        {
            CpuArch::Unknown
        }
    }
    
    /// Detect vendor using CPUID (x86) or MIDR (ARM)
    fn detect_vendor(&self) -> CpuVendor {
        #[cfg(target_arch = "x86_64")]
        {
            // Use CPUID to detect x86 vendor
            let mut ebx: u32 = 0;
            let mut ecx: u32 = 0;
            let mut edx: u32 = 0;
            // SAFETY: inline assembly required for hardware instruction
            unsafe {
                core::arch::asm!(
                    "cpuid",
                    inout("eax") 0 => _,
                    out("ebx") ebx,
                    out("ecx") ecx,
                    out("edx") edx,
                    options(nostack, preserves_flags)
                );
            }
            
            // Reconstruct vendor string from registers (EBX, EDX, ECX order)
            let mut vendor_bytes = [0u8; 12];
            vendor_bytes[0..4].copy_from_slice(&ebx.to_le_bytes());
            vendor_bytes[4..8].copy_from_slice(&edx.to_le_bytes());
            vendor_bytes[8..12].copy_from_slice(&ecx.to_le_bytes());
            
            // Match known vendor strings
            match &vendor_bytes {
                b"GenuineIntel" => CpuVendor::Intel,
                b"AuthenticAMD" => CpuVendor::Amd,
                _ => CpuVendor::Unknown,
            }
        }
        
        #[cfg(target_arch = "aarch64")]
        {
            // Read MIDR_EL1 to detect ARM implementer
            let midr: u64;
            // SAFETY: inline assembly required for hardware instruction
            unsafe {
                core::arch::asm!(
                    "mrs {0}, midr_el1",
                    out(reg) midr,
                    options(nostack, preserves_flags)
                );
            }
            
            // Extract implementer (bits [31:24])
            let implementer = ((midr >> 24) & 0xFF) as u32;
            match implementer {
                0x41 => CpuVendor::Arm,        // ARM Ltd
                0x51 => CpuVendor::Qualcomm,   // Qualcomm
                0x61 => CpuVendor::Apple,      // Apple
                0x4D => CpuVendor::MediaTek,   // MediaTek
                0x53 => CpuVendor::Samsung,    // Samsung
                0x48 => CpuVendor::HiSilicon,  // HiSilicon
                _ => CpuVendor::Unknown,
            }
        }
        
        #[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
        {
            CpuVendor::Unknown
        }
    }
    
    /// Get CPU info
    pub fn get_cpu(&self, id: CpuId) -> Option<&CpuInfo> {
        if id >= 256 {
            return None;
        }
        self.cpus[id as usize].as_ref()
    }
    
    /// Get current CPU ID
    pub fn get_current_cpu(&self) -> CpuId {
        #[cfg(target_arch = "aarch64")]
        {
            // Read from TPIDR_EL0 which stores current CPU ID
            let cpu_id: u64;
            // SAFETY: inline assembly required for hardware instruction
            unsafe {
                core::arch::asm!(
                    "mrs {0}, tpidr_el0",
                    out(reg) cpu_id,
                    options(nostack, preserves_flags)
                );
            }
            cpu_id as u32
        }
        
        #[cfg(target_arch = "x86_64")]
        {
            // Use GS segment register base to find current CPU ID
            // In a real implementation, this reads from the per-CPU area
            0
        }
        
        #[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
        {
            0
        }
    }
    
    /// Get per-CPU data
    pub fn get_percpu(&self, id: CpuId) -> Option<&PerCpuData> {
        if id >= 256 {
            return None;
        }
        Some(&self.percpu[id as usize])
    }
    
    /// Get per-CPU data mutable
    pub fn get_percpu_mut(&mut self, id: CpuId) -> Option<&mut PerCpuData> {
        if id >= 256 {
            return None;
        }
        Some(&mut self.percpu[id as usize])
    }
    
    /// Bring up CPU
    pub fn cpu_up(&mut self, id: CpuId) -> i32 {
        if id >= 256 {
            return Errno::Einval.to_ret_i32();
        }
        
        if let Some(ref mut cpu) = self.cpus[id as usize] {
            cpu.state.store(CpuState::Booting as u32, Ordering::Release);
            
            // Send PSCI CPU_ON or ACPI request to bring up CPU
            // For ARM: Call PSCI cpu_on with the target CPU's MPIDR and entry point
            // For x86: Send INIT/SIPI via IPI to the target APIC ID
            // Then wait for the CPU to signal it is online
            
            cpu.state.store(CpuState::Online as u32, Ordering::Release);
            cpu.online.store(true, Ordering::Release);
            self.nr_online.fetch_add(1, Ordering::AcqRel);
            self.stats.hotplugs.fetch_add(1, Ordering::AcqRel);
            
            return 0;
        }
        
        -1
    }
    
    /// Bring down CPU
    pub fn cpu_down(&mut self, id: CpuId) -> i32 {
        if id >= 256 || id == 0 {
            return Errno::Einval.to_ret_i32(); // Cannot offline boot CPU
        }
        
        if let Some(ref mut cpu) = self.cpus[id as usize] {
            if !cpu.online.load(Ordering::Acquire) {
                return Errno::Eperm.to_ret_i32();
            }
            
            // Migrate tasks to other CPUs:
            // 1. Iterate over the runqueue of the target CPU
            // 2. For each runnable task, find a new CPU and migrate it
            // 3. Ensure no timers or IRQs are affined to this CPU
            // Then send PSCI CPU_OFF or ACPI request to bring down CPU
            
            cpu.state.store(CpuState::Offline as u32, Ordering::Release);
            cpu.online.store(false, Ordering::Release);
            self.nr_online.fetch_sub(1, Ordering::AcqRel);
            self.stats.hotplugs.fetch_add(1, Ordering::AcqRel);
            
            return 0;
        }
        
        -1
    }
    
    /// Check if CPU is online
    pub fn is_online(&self, id: CpuId) -> bool {
        if let Some(cpu) = self.get_cpu(id) {
            cpu.online.load(Ordering::Acquire)
        } else {
            false
        }
    }
    
    /// Get number of online CPUs
    pub fn num_online(&self) -> u32 {
        self.nr_online.load(Ordering::Acquire)
    }
    
    /// Iterate over online CPUs
    pub fn for_each_online<F: FnMut(CpuId)>(&self, mut f: F) {
        for i in 0..256 {
            if self.is_online(i) {
                f(i);
            }
        }
    }
}

/// Global CPU manager
static CPU_MANAGER: core::sync::OnceLock<CpuManager> = core::sync::OnceLock::new();

/// Get CPU manager
pub fn cpu_manager() -> &'static CpuManager {
    CPU_MANAGER.get_or_init(CpuManager::new)
}

pub fn init_cpu_manager() -> &'static CpuManager {
    CPU_MANAGER.get_or_init(CpuManager::new)
}

/// Initialize CPU management
pub fn init_cpu() {
    let mgr = cpu_manager();
    mgr.init();
}

/// Get current CPU ID
pub fn smp_processor_id() -> CpuId {
    cpu_manager().get_current_cpu()
}

/// Get number of online CPUs
pub fn num_online_cpus() -> u32 {
    cpu_manager().num_online()
}

/// Check if CPU is online
pub fn cpu_online(cpu: CpuId) -> bool {
    cpu_manager().is_online(cpu)
}

/// Get per-CPU variable pointer
/// Returns a pointer to the per-CPU data area for the current CPU.
/// The caller must disable preemption while accessing per-CPU data
/// to prevent migration to another CPU.
pub fn this_cpu_ptr<T>() -> *mut T {
    let cpu_id = smp_processor_id();
    let mgr = cpu_manager();
    if let Some(percpu) = mgr.get_percpu_mut(cpu_id) {
        percpu as *mut PerCpuData as *mut T
    } else {
        core::ptr::null_mut()
    }
}