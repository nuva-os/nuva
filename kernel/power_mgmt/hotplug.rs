/*
 * Nuva OS - Kernel - PowerMgmt - Hotplug
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
 * Nuva OS - Kernel - Hotplug Support
 * 
 * Copyright (C) 2026 Nuva OS Team
 * 
 * Kernel hotplug support for CPUs, memory, and devices.
 */

use core::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};

use crate::syslib::posix::errno::Errno;
use crate::kernel::error::Errno;
/// Hotplug Event Type
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotplugType {
    /// CPU online
    CpuOnline = 0,
    /// CPU offline
    CpuOffline = 1,
    /// Memory online
    MemOnline = 2,
    /// Memory offline
    MemOffline = 3,
    /// Device add
    DevAdd = 4,
    /// Device remove
    DevRemove = 5,
    /// PCI device add
    PciAdd = 6,
    /// PCI device remove
    PciRemove = 7,
    /// USB device add
    UsbAdd = 8,
    /// USB device remove
    UsbRemove = 9,
}

/// Hotplug State
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotplugState {
    /// Offline
    Offline = 0,
    /// Preparing
    Preparing = 1,
    /// Online
    Online = 2,
    /// Dying
    Dying = 3,
    /// Dead
    Dead = 4,
    /// Failed
    Failed = 5,
}

/// Hotplug Callback
pub type HotplugCallback = unsafe extern "C" fn(u32, HotplugType, *mut core::ffi::c_void) -> i32;

/// Hotplug Notifier
pub struct HotplugNotifier {
    /// Callback
    pub callback: HotplugCallback,
    /// Priority
    pub priority: i32,
    /// Next
    pub next: *mut HotplugNotifier,
}

/// CPU Hotplug State
pub struct CpuHotplugState {
    /// CPU ID
    pub cpu: u32,
    /// State
    pub state: AtomicU32,
    /// Online
    pub online: AtomicBool,
    /// Present
    pub present: AtomicBool,
    /// Possible
    pub possible: AtomicBool,
    /// Active
    pub active: AtomicBool,
    /// Boot CPU
    pub boot_cpu: bool,
    /// Hotpluggable
    pub hotpluggable: bool,
    /// Notifiers
    pub notifiers: *mut HotplugNotifier,
}

impl CpuHotplugState {
    pub fn new(cpu: u32) -> Self {
        CpuHotplugState {
            cpu,
            state: AtomicU32::new(HotplugState::Offline as u32),
            online: AtomicBool::new(false),
            present: AtomicBool::new(false),
            possible: AtomicBool::new(false),
            active: AtomicBool::new(false),
            boot_cpu: false,
            hotpluggable: true,
            notifiers: core::ptr::null_mut(),
        }
    }
    
    /// Get state
    pub fn get_state(&self) -> HotplugState {
        match self.state.load(Ordering::Acquire) {
            0 => HotplugState::Offline,
            1 => HotplugState::Preparing,
            2 => HotplugState::Online,
            3 => HotplugState::Dying,
            4 => HotplugState::Dead,
            5 => HotplugState::Failed,
            _ => HotplugState::Offline,
        }
    }
    
    /// Bring CPU online
    pub fn bring_online(&mut self) -> i32 {
        if self.online.load(Ordering::Acquire) {
            return 0;
        }
        
        if !self.hotpluggable {
            return Errno::Eopnotsupp.to_ret_i32(); // EOPNOTSUPP
        }
        
        // Set state to preparing
        self.state.store(HotplugState::Preparing as u32, Ordering::Release);
        
        // Notify callbacks
        self.notify(HotplugType::CpuOnline);
        
        // TODO: Actually bring CPU online
        
        // Set state to online
        self.state.store(HotplugState::Online as u32, Ordering::Release);
        self.online.store(true, Ordering::Release);
        self.active.store(true, Ordering::Release);
        
        0
    }
    
    /// Take CPU offline
    pub fn take_offline(&mut self) -> i32 {
        if !self.online.load(Ordering::Acquire) {
            return 0;
        }
        
        if self.boot_cpu {
            return Errno::Einval.to_ret_i32(); // EINVAL - can't offline boot CPU
        }
        
        // Set state to dying
        self.state.store(HotplugState::Dying as u32, Ordering::Release);
        
        // Notify callbacks
        self.notify(HotplugType::CpuOffline);
        
        // TODO: Actually take CPU offline
        
        // Set state to offline
        self.state.store(HotplugState::Offline as u32, Ordering::Release);
        self.online.store(false, Ordering::Release);
        self.active.store(false, Ordering::Release);
        
        0
    }
    
    /// Notify callbacks
    fn notify(&mut self, event: HotplugType) {
        let mut notifier = self.notifiers;
        
        while !notifier.is_null() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                let cb = (*notifier).callback;
                cb(self.cpu, event, core::ptr::null_mut());
                notifier = (*notifier).next;
            }
        }
    }
    
    /// Register notifier
    pub fn register_notifier(&mut self, notifier: *mut HotplugNotifier) {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            (*notifier).next = self.notifiers;
            self.notifiers = notifier;
        }
    }
}

/// Memory Hotplug State
pub struct MemHotplugState {
    /// Memory section ID
    pub section: u64,
    /// Start PFN
    pub start_pfn: u64,
    /// End PFN
    pub end_pfn: u64,
    /// State
    pub state: AtomicU32,
    /// Online
    pub online: AtomicBool,
    /// Present
    pub present: AtomicBool,
    /// Hotpluggable
    pub hotpluggable: bool,
    /// Movable
    pub movable: bool,
    /// Size in bytes
    pub size: u64,
}

impl MemHotplugState {
    pub fn new(section: u64, start_pfn: u64, end_pfn: u64) -> Self {
        MemHotplugState {
            section,
            start_pfn,
            end_pfn,
            state: AtomicU32::new(HotplugState::Offline as u32),
            online: AtomicBool::new(false),
            present: AtomicBool::new(false),
            hotpluggable: true,
            movable: false,
            size: (end_pfn - start_pfn) * 4096, // Assuming 4K pages
        }
    }
    
    /// Bring memory online
    pub fn bring_online(&mut self) -> i32 {
        if self.online.load(Ordering::Acquire) {
            return 0;
        }
        
        if !self.hotpluggable {
            return Errno::Eopnotsupp.to_ret_i32();
        }
        
        // TODO: Actually bring memory online
        
        self.state.store(HotplugState::Online as u32, Ordering::Release);
        self.online.store(true, Ordering::Release);
        
        0
    }
    
    /// Take memory offline
    pub fn take_offline(&mut self) -> i32 {
        if !self.online.load(Ordering::Acquire) {
            return 0;
        }
        
        // TODO: Check if memory can be offlined
        
        // TODO: Actually take memory offline
        
        self.state.store(HotplugState::Offline as u32, Ordering::Release);
        self.online.store(false, Ordering::Release);
        
        0
    }
}

/// Hotplug Manager
pub struct HotplugManager {
    /// CPU states
    pub cpu_states: [CpuHotplugState; 256],
    /// CPU count
    pub cpu_count: AtomicU32,
    /// Online CPUs
    pub online_cpus: AtomicU32,
    /// Possible CPUs
    pub possible_cpus: AtomicU32,
    /// Present CPUs
    pub present_cpus: AtomicU32,
    /// Memory sections
    pub mem_sections: *mut MemHotplugState,
    /// Memory section count
    pub mem_section_count: AtomicU32,
    /// Online memory (bytes)
    pub online_memory: AtomicU64,
    /// Total memory (bytes)
    pub total_memory: AtomicU64,
    /// Hotplug enabled
    pub enabled: AtomicBool,
    /// Statistics
    pub stats: HotplugStats,
}

/// Hotplug Statistics
pub struct HotplugStats {
    pub cpu_online_count: AtomicU64,
    pub cpu_offline_count: AtomicU64,
    pub mem_online_count: AtomicU64,
    pub mem_offline_count: AtomicU64,
    pub dev_add_count: AtomicU64,
    pub dev_remove_count: AtomicU64,
}

impl HotplugStats {
    pub const fn new() -> Self {
        HotplugStats {
            cpu_online_count: AtomicU64::new(0),
            cpu_offline_count: AtomicU64::new(0),
            mem_online_count: AtomicU64::new(0),
            mem_offline_count: AtomicU64::new(0),
            dev_add_count: AtomicU64::new(0),
            dev_remove_count: AtomicU64::new(0),
        }
    }
}

impl HotplugManager {
    pub const fn new() -> Self {
        const CPU_INIT: CpuHotplugState = CpuHotplugState {
            cpu: 0,
            state: AtomicU32::new(0),
            online: AtomicBool::new(false),
            present: AtomicBool::new(false),
            possible: AtomicBool::new(false),
            active: AtomicBool::new(false),
            boot_cpu: false,
            hotpluggable: true,
            notifiers: core::ptr::null_mut(),
        };
        
        HotplugManager {
            cpu_states: [CPU_INIT; 256],
            cpu_count: AtomicU32::new(0),
            online_cpus: AtomicU32::new(0),
            possible_cpus: AtomicU32::new(0),
            present_cpus: AtomicU32::new(0),
            mem_sections: core::ptr::null_mut(),
            mem_section_count: AtomicU32::new(0),
            online_memory: AtomicU64::new(0),
            total_memory: AtomicU64::new(0),
            enabled: AtomicBool::new(true),
            stats: HotplugStats::new(),
        }
    }
    
    /// Initialize
    pub fn init(&self) {
        // Mark boot CPU as online
        self.cpu_states[0].cpu = 0;
        self.cpu_states[0].state.store(HotplugState::Online as u32, Ordering::Release);
        self.cpu_states[0].online.store(true, Ordering::Release);
        self.cpu_states[0].present.store(true, Ordering::Release);
        self.cpu_states[0].possible.store(true, Ordering::Release);
        self.cpu_states[0].active.store(true, Ordering::Release);
        self.cpu_states[0].boot_cpu = true;
        
        self.cpu_count.store(1, Ordering::Release);
        self.online_cpus.store(1, Ordering::Release);
        self.possible_cpus.store(1, Ordering::Release);
        self.present_cpus.store(1, Ordering::Release);
        
        log_info!("Hotplug manager initialized");
    }
    
    /// Add CPU
    pub fn add_cpu(&mut self, cpu: u32) -> i32 {
        if cpu >= 256 {
            return Errno::Einval.to_ret_i32();
        }
        
        let state = &mut self.cpu_states[cpu as usize];
        state.cpu = cpu;
        state.present.store(true, Ordering::Release);
        state.possible.store(true, Ordering::Release);
        
        self.cpu_count.fetch_add(1, Ordering::AcqRel);
        self.possible_cpus.fetch_add(1, Ordering::AcqRel);
        self.present_cpus.fetch_add(1, Ordering::AcqRel);
        
        0
    }
    
    /// Remove CPU
    pub fn remove_cpu(&mut self, cpu: u32) -> i32 {
        if cpu >= 256 {
            return Errno::Einval.to_ret_i32();
        }
        
        let state = &mut self.cpu_states[cpu as usize];
        
        if state.boot_cpu {
            return Errno::Einval.to_ret_i32();
        }
        
        if state.online.load(Ordering::Acquire) {
            return Errno::Ebusy.to_ret_i32(); // EBUSY
        }
        
        state.present.store(false, Ordering::Release);
        state.possible.store(false, Ordering::Release);
        
        self.cpu_count.fetch_sub(1, Ordering::AcqRel);
        self.possible_cpus.fetch_sub(1, Ordering::AcqRel);
        self.present_cpus.fetch_sub(1, Ordering::AcqRel);
        
        0
    }
    
    /// Bring CPU online
    pub fn cpu_online(&mut self, cpu: u32) -> i32 {
        if cpu >= 256 {
            return Errno::Einval.to_ret_i32();
        }
        
        let state = &mut self.cpu_states[cpu as usize];
        let ret = state.bring_online();
        
        if ret == 0 {
            self.online_cpus.fetch_add(1, Ordering::AcqRel);
            self.stats.cpu_online_count.fetch_add(1, Ordering::AcqRel);
        }
        
        ret
    }
    
    /// Take CPU offline
    pub fn cpu_offline(&mut self, cpu: u32) -> i32 {
        if cpu >= 256 {
            return Errno::Einval.to_ret_i32();
        }
        
        let state = &mut self.cpu_states[cpu as usize];
        let ret = state.take_offline();
        
        if ret == 0 {
            self.online_cpus.fetch_sub(1, Ordering::AcqRel);
            self.stats.cpu_offline_count.fetch_add(1, Ordering::AcqRel);
        }
        
        ret
    }
    
    /// Add memory
    pub fn add_memory(&mut self, start_pfn: u64, end_pfn: u64) -> i32 {
        let section = self.mem_section_count.load(Ordering::Acquire);
        
        // TODO: Allocate and add memory section
        
        let size = (end_pfn - start_pfn) * 4096;
        self.total_memory.fetch_add(size, Ordering::AcqRel);
        self.mem_section_count.fetch_add(1, Ordering::AcqRel);
        
        let _ = section;
        0
    }
    
    /// Remove memory
    pub fn remove_memory(&mut self, start_pfn: u64, end_pfn: u64) -> i32 {
        // TODO: Find and remove memory section
        
        let size = (end_pfn - start_pfn) * 4096;
        self.total_memory.fetch_sub(size, Ordering::AcqRel);
        
        0
    }
    
    /// Bring memory online
    pub fn memory_online(&mut self, section: u64) -> i32 {
        // TODO: Find section and bring online
        
        self.stats.mem_online_count.fetch_add(1, Ordering::AcqRel);
        0
    }
    
    /// Take memory offline
    pub fn memory_offline(&mut self, section: u64) -> i32 {
        // TODO: Find section and take offline
        
        self.stats.mem_offline_count.fetch_add(1, Ordering::AcqRel);
        0
    }
    
    /// Check if CPU is online
    pub fn cpu_is_online(&self, cpu: u32) -> bool {
        if cpu >= 256 {
            return false;
        }
        self.cpu_states[cpu as usize].online.load(Ordering::Acquire)
    }
    
    /// Check if CPU is present
    pub fn cpu_is_present(&self, cpu: u32) -> bool {
        if cpu >= 256 {
            return false;
        }
        self.cpu_states[cpu as usize].present.load(Ordering::Acquire)
    }
    
    /// Get online CPU mask
    pub fn get_online_cpu_mask(&self) -> u64 {
        let mut mask: u64 = 0;
        
        for i in 0..64 {
            if self.cpu_is_online(i as u32) {
                mask |= 1 << i;
            }
        }
        
        mask
    }
    
    /// For each online CPU
    pub fn for_each_online_cpu<F>(&self, mut f: F)
    where
        F: FnMut(u32),
    {
        for i in 0..256 {
            if self.cpu_is_online(i as u32) {
                f(i as u32);
            }
        }
    }
    
    /// For each present CPU
    pub fn for_each_present_cpu<F>(&self, mut f: F)
    where
        F: FnMut(u32),
    {
        for i in 0..256 {
            if self.cpu_is_present(i as u32) {
                f(i as u32);
            }
        }
    }
}

/// Global hotplug manager
static HOTPLUG_MANAGER: crate::sync_oncelock::OnceLock<HotplugManager> = crate::sync_oncelock::OnceLock::new();

/// Get hotplug manager
pub fn hotplug_manager() -> &'static HotplugManager {
    HOTPLUG_MANAGER.get_or_init(HotplugManager::new)
}

pub fn init_hotplug_manager() -> &'static HotplugManager {
    HOTPLUG_MANAGER.get_or_init(HotplugManager::new)
}

/// Initialize hotplug
pub fn init_hotplug() {
    let mgr = hotplug_manager();
    mgr.init();
}

// Convenience functions

/// Check if CPU is online
pub fn cpu_online(cpu: u32) -> bool {
    hotplug_manager().cpu_is_online(cpu)
}

/// Check if CPU is present
pub fn cpu_present(cpu: u32) -> bool {
    hotplug_manager().cpu_is_present(cpu)
}

/// Get number of online CPUs
pub fn num_online_cpus() -> u32 {
    hotplug_manager().online_cpus.load(Ordering::Acquire)
}

/// Get number of possible CPUs
pub fn num_possible_cpus() -> u32 {
    hotplug_manager().possible_cpus.load(Ordering::Acquire)
}

/// Get number of present CPUs
pub fn num_present_cpus() -> u32 {
    hotplug_manager().present_cpus.load(Ordering::Acquire)
}

/// For each online CPU
pub fn for_each_online_cpu<F>(f: F)
where
    F: FnMut(u32),
{
    hotplug_manager().for_each_online_cpu(f);
}

/// For each possible CPU
pub fn for_each_possible_cpu<F>(mut f: F)
where
    F: FnMut(u32),
{
    for i in 0..num_possible_cpus() {
        f(i);
    }
}

/// CPU Hotplug Lock
pub struct CpuHotplugLock {
    pub locked: AtomicBool,
}

impl CpuHotplugLock {
    pub const fn new() -> Self {
        CpuHotplugLock {
            locked: AtomicBool::new(false),
        }
    }
    
    /// Lock
    pub fn lock(&self) {
        while self.locked.compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire).is_err() {
            core::hint::spin_loop();
        }
    }
    
    /// Unlock
    pub fn unlock(&self) {
        self.locked.store(false, Ordering::Release);
    }
    
    /// Try lock
    pub fn try_lock(&self) -> bool {
        self.locked.compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire).is_ok()
    }
}

/// Global CPU hotplug lock
static CPU_HOTPLUG_LOCK: CpuHotplugLock = CpuHotplugLock::new();

/// Get CPU hotplug lock
pub fn cpu_hotplug_lock() {
    CPU_HOTPLUG_LOCK.lock();
}

/// Release CPU hotplug lock
pub fn cpu_hotplug_unlock() {
    CPU_HOTPLUG_LOCK.unlock();
}

/// Try to get CPU hotplug lock
pub fn cpu_hotplug_trylock() -> bool {
    CPU_HOTPLUG_LOCK.try_lock()
}
