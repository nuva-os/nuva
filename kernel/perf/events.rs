/*
 * Nuva OS - Kernel - Perf - Events
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
/*
 * Nuva OS - Kernel - Performance Events
 * 
 * Copyright (C) 2026 Nuva OS Team
 * 
 * Kernel performance monitoring and events.
 */

use core::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, AtomicPtr, Ordering};
use alloc::alloc::{alloc, dealloc, Layout};
use crate::{pr_info};

use crate::syslib::posix::errno::Errno;
use crate::kernel::error::Errno;
/// Perf Event Type
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerfEventType {
    /// Hardware CPU cycles
    HwCpuCycles = 0,
    /// Hardware instructions
    HwInstructions = 1,
    /// Cache references
    HwCacheRefs = 2,
    /// Cache misses
    HwCacheMisses = 3,
    /// Branch instructions
    HwBranchInstr = 4,
    /// Branch misses
    HwBranchMisses = 5,
    /// Bus cycles
    HwBusCycles = 6,
    /// Stalled cycles frontend
    HwStalledCyclesFrontend = 7,
    /// Stalled cycles backend
    HwStalledCyclesBackend = 8,
    /// Ref CPU cycles
    HwRefCpuCycles = 9,
    
    /// Software context switches
    SwContextSwitches = 32,
    /// Software CPU migrations
    SwCpuMigrations = 33,
    /// Software page faults
    SwPageFaults = 34,
    /// Software page faults major
    SwPageFaultsMaj = 35,
    /// Software alignment faults
    SwAlignmentFaults = 36,
    /// Software emulation faults
    SwEmulationFaults = 37,
    /// Software dummy
    SwDummy = 38,
    /// Software bpf output
    SwBpfOutput = 39,
    
    /// Tracepoint
    Tracepoint = 64,
    /// Kprobe
    Kprobe = 65,
    /// Uprobe
    Uprobe = 66,
    
    /// Hardware cache L1D read
    HwCacheL1dRead = 128,
    /// Hardware cache L1D write
    HwCacheL1dWrite = 129,
    /// Hardware cache L1D prefetch
    HwCacheL1dPrefetch = 130,
    /// Hardware cache L1I read
    HwCacheL1iRead = 131,
    /// Hardware cache LL read
    HwCacheLlRead = 132,
    /// Hardware cache DTLB read
    HwCacheDtlbRead = 133,
    /// Hardware cache ITLB read
    HwCacheItlbRead = 134,
    /// Hardware cache BPU read
    HwCacheBpuRead = 135,
}

/// Perf Event Attr
#[repr(C)]
#[derive(Clone, Copy)]
pub struct PerfEventAttr {
    /// Event type
    pub event_type: u32,
    /// Event config
    pub config: u64,
    /// Sample period
    pub sample_period: u64,
    /// Sample frequency
    pub sample_freq: u64,
    /// Flags
    pub flags: u64,
    /// Read format
    pub read_format: u64,
    /// Wakeup events
    pub wakeup_events: u32,
    /// Wakeup watermark
    pub wakeup_watermark: u32,
    /// BP type
    pub bp_type: u32,
    /// BP addr
    pub bp_addr: u64,
    /// BP len
    pub bp_len: u64,
    /// Branch sample type
    pub branch_sample_type: u64,
    /// Sample regs user
    pub sample_regs_user: u64,
    /// Sample stack user
    pub sample_stack_user: u32,
    /// Clock ID
    pub clockid: i32,
    /// Sample regs intr
    pub sample_regs_intr: u64,
    /// Aux watermark
    pub aux_watermark: u32,
    /// Sample max stack
    pub sample_max_stack: u16,
    /// Namespace ID
    pub namespace_id: u16,
}

/// Perf Event Flags
bitflags::bitflags! {
    #[repr(transparent)]
    pub struct PerfEventFlags: u64 {
        /// Disabled
        const DISABLED = 1 << 0;
        /// Inherit
        const INHERIT = 1 << 1;
        /// Pinned
        const PINNED = 1 << 2;
        /// Exclusive
        const EXCLUSIVE = 1 << 3;
        /// Exclude user
        const EXCLUDE_USER = 1 << 4;
        /// Exclude kernel
        const EXCLUDE_KERNEL = 1 << 5;
        /// Exclude HV
        const EXCLUDE_HV = 1 << 6;
        /// Exclude idle
        const EXCLUDE_IDLE = 1 << 7;
        /// Mmap
        const MMAP = 1 << 8;
        /// Comm
        const COMM = 1 << 9;
        /// Freq
        const FREQ = 1 << 10;
        /// Inherit stat
        const INHERIT_STAT = 1 << 11;
        /// Enable on exec
        const ENABLE_ON_EXEC = 1 << 12;
        /// Task
        const TASK = 1 << 13;
        /// Watermark
        const WATERMARK = 1 << 14;
        /// Precise IP 0
        const PRECISE_IP_0 = 0 << 15;
        /// Precise IP 1
        const PRECISE_IP_1 = 1 << 15;
        /// Precise IP 2
        const PRECISE_IP_2 = 2 << 15;
        /// Precise IP 3
        const PRECISE_IP_3 = 3 << 15;
        /// Mmap data
        const MMAP_DATA = 1 << 17;
        /// Sample ID all
        const SAMPLE_ID_ALL = 1 << 18;
        /// Exclude host
        const EXCLUDE_HOST = 1 << 21;
        /// Exclude guest
        const EXCLUDE_GUEST = 1 << 22;
        /// Exclude callchain kernel
        const EXCLUDE_CALLCHAIN_KERNEL = 1 << 23;
        /// Exclude callchain user
        const EXCLUDE_CALLCHAIN_USER = 1 << 24;
        /// Mmap2
        const MMAP2 = 1 << 25;
        /// Comm exec
        const COMM_EXEC = 1 << 26;
        /// Use clock ID
        const USE_CLOCKID = 1 << 27;
        /// Context switch
        const CONTEXT_SWITCH = 1 << 28;
        /// Write backward
        const WRITE_BACKWARD = 1 << 29;
        /// Namespaces
        const NAMESPACES = 1 << 30;
        /// Ksymbol
        const KSYMBOL = 1 << 31;
        /// Bpf event
        const BPF_EVENT = 1 << 32;
    }
}

/// Perf Event Value
#[repr(C)]
#[derive(Clone, Copy)]
pub struct PerfEventValue {
    /// Event count
    pub count: u64,
    /// Time enabled
    pub time_enabled: u64,
    /// Time running
    pub time_running: u64,
    /// Next event in group
    pub next: *mut PerfEvent,
}

/// Perf Event
pub struct PerfEvent {
    /// Event ID
    pub id: u64,
    /// Event attributes
    pub attr: PerfEventAttr,
    /// Event value
    pub value: PerfEventValue,
    /// CPU
    pub cpu: u32,
    /// PID
    pub pid: u32,
    /// TID
    pub tid: u32,
    /// Group leader
    pub group_leader: *mut PerfEvent,
    /// Next sibling
    pub sibling: *mut PerfEvent,
    /// State
    pub state: AtomicU32,
    /// Active
    pub active: AtomicBool,
    /// Overflow handler
    pub overflow_handler: Option<unsafe extern "C" fn(*mut PerfEvent, *mut core::ffi::c_void)>,
    /// Context
    pub context: *mut core::ffi::c_void,
    /// Ring buffer
    pub rb: AtomicPtr<PerfRingBuffer>,
    pub next: *mut PerfEvent,
}

/// Perf Event State
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventState {
    Off = 0,
    Inactive = 1,
    Active = 2,
    Error = 3,
}

/// Perf Ring Buffer
pub struct PerfRingBuffer {
    /// Base address
    pub base: *mut u8,
    /// Size
    pub size: u32,
    /// Page size
    pub page_size: u32,
    /// Head
    pub head: AtomicU64,
    /// Tail
    pub tail: AtomicU64,
    /// Overwrite
    pub overwrite: bool,
    /// Flags
    pub flags: AtomicU32,
}

impl PerfRingBuffer {
    pub fn new(base: *mut u8, size: u32, overwrite: bool) -> Self {
        PerfRingBuffer {
            base,
            size,
            page_size: 4096,
            head: AtomicU64::new(0),
            tail: AtomicU64::new(0),
            overwrite,
            flags: AtomicU32::new(0),
        }
    }
    
    /// Write data
    pub fn write(&mut self, data: &[u8]) -> i32 {
        let head = self.head.load(Ordering::Acquire);
        let tail = self.tail.load(Ordering::Acquire);
        let size = self.size as u64;
        
        let available = if head >= tail {
            size - (head - tail)
        } else {
            tail - head
        };
        
        if data.len() as u64 > available - 1 {
            if self.overwrite {
                // Overwrite old data
                self.tail.fetch_add(data.len() as u64, Ordering::AcqRel);
            } else {
                return Errno::Enospc.to_ret_i32(); // ENOSPC
            }
        }
        
        // Copy data
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let offset = (head % size) as usize;
            let base = self.base.add(offset);
            
            if offset + data.len() <= self.size as usize {
                core::ptr::copy_nonoverlapping(data.as_ptr(), base, data.len());
            } else {
                // Wrap around
                let first = self.size as usize - offset;
                core::ptr::copy_nonoverlapping(data.as_ptr(), base, first);
                core::ptr::copy_nonoverlapping(
                    data.as_ptr().add(first),
                    self.base,
                    data.len() - first,
                );
            }
        }
        
        self.head.fetch_add(data.len() as u64, Ordering::AcqRel);
        data.len() as i32
    }
    
    /// Read data
    pub fn read(&mut self, buf: &mut [u8]) -> i32 {
        let head = self.head.load(Ordering::Acquire);
        let tail = self.tail.load(Ordering::Acquire);
        
        if head == tail {
            return 0;
        }
        
        let available = if head >= tail {
            head - tail
        } else {
            self.size as u64 - tail + head
        };
        
        let len = buf.len().min(available as usize);
        
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let offset = (tail % self.size as u64) as usize;
            let base = self.base.add(offset);
            
            if offset + len <= self.size as usize {
                core::ptr::copy_nonoverlapping(base, buf.as_mut_ptr(), len);
            } else {
                let first = self.size as usize - offset;
                core::ptr::copy_nonoverlapping(base, buf.as_mut_ptr(), first);
                core::ptr::copy_nonoverlapping(
                    self.base,
                    buf.as_mut_ptr().add(first),
                    len - first,
                );
            }
        }
        
        self.tail.fetch_add(len as u64, Ordering::AcqRel);
        len as i32
    }
}

/// Perf Context
pub struct PerfContext {
    /// Events
    pub events: *mut PerfEvent,
    /// Event count
    pub event_count: AtomicU32,
    /// CPU context
    pub cpu_ctx: [PerfCpuContext; 256],
    /// Statistics
    pub stats: PerfStats,
    pub event_list: AtomicPtr<PerfEvent>,
}

/// Perf CPU Context
pub struct PerfCpuContext {
    /// Active events
    pub active: *mut PerfEvent,
    /// Active count
    pub active_count: AtomicU32,
    /// NMI context
    pub in_nmi: AtomicBool,
    /// Lock
    pub lock: AtomicU32,
}

impl PerfCpuContext {
    pub const fn new() -> Self {
        PerfCpuContext {
            active: core::ptr::null_mut(),
            active_count: AtomicU32::new(0),
            in_nmi: AtomicBool::new(false),
            lock: AtomicU32::new(0),
        }
    }
}

/// Perf Statistics
pub struct PerfStats {
    pub event_count: AtomicU64,
    pub overflow_count: AtomicU64,
    pub sample_count: AtomicU64,
    pub lost_count: AtomicU64,
}

impl PerfStats {
    pub const fn new() -> Self {
        PerfStats {
            event_count: AtomicU64::new(0),
            overflow_count: AtomicU64::new(0),
            sample_count: AtomicU64::new(0),
            lost_count: AtomicU64::new(0),
        }
    }
}

/// Perf Manager
pub struct PerfManager {
    /// Context
    pub ctx: PerfContext,
    /// Next event ID
    pub next_event_id: AtomicU64,
    /// Enabled
    pub enabled: AtomicBool,
}

impl PerfManager {
    pub const fn new() -> Self {
        const CPU_CTX_INIT: PerfCpuContext = PerfCpuContext::new();
        
        PerfManager {
            ctx: PerfContext {
                events: core::ptr::null_mut(),
                event_count: AtomicU32::new(0),
                cpu_ctx: [CPU_CTX_INIT; 256],
                stats: PerfStats::new(),
                event_list: AtomicPtr::new(core::ptr::null_mut()),
            },
            next_event_id: AtomicU64::new(1),
            enabled: AtomicBool::new(true),
        }
    }
    
    /// Initialize
    pub fn init(&self) {
        log_info!("Perf manager initialized");
    }
    
    /// Create event
    pub fn event_create(&mut self, attr: &PerfEventAttr, cpu: i32, pid: u32) -> Result<*mut PerfEvent, i32> {
        if !self.enabled.load(Ordering::Acquire) {
            return Err(-95); // EOPNOTSUPP
        }
        
        // Allocate event
        let new_event = PerfEvent {
            id: self.next_event_id.fetch_add(1, Ordering::AcqRel),
            attr: *attr,
            value: PerfEventValue {
                count: 0,
                time_enabled: 0,
                time_running: 0,
                next: core::ptr::null_mut(),
            },
            cpu: if cpu < 0 { 0xFFFFFFFF } else { cpu as u32 },
            pid,
            tid: pid,
            group_leader: core::ptr::null_mut(),
            sibling: core::ptr::null_mut(),
            state: AtomicU32::new(EventState::Off as u32),
            active: AtomicBool::new(false),
            overflow_handler: None,
            context: core::ptr::null_mut(),
            rb: AtomicPtr::new(core::ptr::null_mut()),
            next: core::ptr::null_mut(),
        };
        
        // Allocate and add event to the event list
        // SAFETY: allocating memory for perf event
        let event_box = unsafe { alloc(Layout::new::<PerfEvent>()) } as *mut PerfEvent;
        if event_box.is_null() {
            return Err(-12); // ENOMEM
        }
        // SAFETY: event_box was just allocated with the correct layout
        unsafe {
            core::ptr::write(event_box, new_event);
            // Insert into the event list (head insertion)
            (*event_box).next = self.ctx.event_list.load(Ordering::Acquire);
            self.ctx.event_list.store(event_box, Ordering::Release);
        }
        
        self.ctx.event_count.fetch_add(1, Ordering::AcqRel);
        self.ctx.stats.event_count.fetch_add(1, Ordering::AcqRel);
        
        Ok(event_box)
    }
    
    /// Enable event
    pub fn event_enable(&mut self, event: *mut PerfEvent) -> i32 {
        if event.is_null() {
            return Errno::Einval.to_ret_i32();
        }
        
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            (*event).state.store(EventState::Active as u32, Ordering::Release);
            (*event).active.store(true, Ordering::Release);
        }
        
        0
    }
    
    /// Disable event
    pub fn event_disable(&mut self, event: *mut PerfEvent) -> i32 {
        if event.is_null() {
            return Errno::Einval.to_ret_i32();
        }
        
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            (*event).state.store(EventState::Inactive as u32, Ordering::Release);
            (*event).active.store(false, Ordering::Release);
        }
        
        0
    }
    
    /// Read event
    pub fn event_read(&self, event: *mut PerfEvent) -> PerfEventValue {
        if event.is_null() {
            return PerfEventValue {
                count: 0,
                time_enabled: 0,
                time_running: 0,
                next: core::ptr::null_mut(),
            };
        }
        
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe { (*event).value }
    }
    
    /// Release event
    pub fn event_release(&mut self, event: *mut PerfEvent) -> i32 {
        if event.is_null() {
            return Errno::Einval.to_ret_i32();
        }
        
        // Disable first
        self.event_disable(event);
        
        // Free event memory
        // SAFETY: event was allocated with Layout::new::<PerfEvent>() in event_create
        unsafe {
            dealloc(event as *mut u8, Layout::new::<PerfEvent>());
        }
        
        self.ctx.event_count.fetch_sub(1, Ordering::AcqRel);
        0
    }
    
    /// Handle overflow
    pub fn handle_overflow(&mut self, event: *mut PerfEvent) {
        if event.is_null() {
            return;
        }
        
        self.ctx.stats.overflow_count.fetch_add(1, Ordering::AcqRel);
        
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            if let Some(handler) = (*event).overflow_handler {
                handler(event, (*event).context);
            }
        }
    }
    
    /// Record sample
    pub fn record_sample(&mut self, event: *mut PerfEvent, data: &[u8]) {
        if event.is_null() {
            return;
        }
        
        self.ctx.stats.sample_count.fetch_add(1, Ordering::AcqRel);
        
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let rb = (*event).rb.load(Ordering::Acquire);
            if !rb.is_null() {
                (*rb).write(data);
            }
        }
    }
}

/// Global perf manager
static PERF_MANAGER: crate::sync_oncelock::OnceLock<PerfManager> = crate::sync_oncelock::OnceLock::new();

/// Get perf manager
pub fn perf_manager() -> &'static PerfManager {
    PERF_MANAGER.get_or_init(PerfManager::new)
}

pub fn init_perf_manager() -> &'static PerfManager {
    PERF_MANAGER.get_or_init(PerfManager::new)
}

/// Initialize perf
pub fn init_perf() {
    let mgr = perf_manager();
    mgr.init();
}

// Convenience functions

/// Read CPU cycles
pub fn perf_read_cycles() -> u64 {
    // Read CPU cycle counter from PMU
    crate::hal::cpu::read_cycle_counter()
}

/// Read instructions
pub fn perf_read_instructions() -> u64 {
    // Read retired instruction count from PMU
    crate::hal::cpu::read_inst_counter()
}

/// Read cache misses
pub fn perf_read_cache_misses() -> u64 {
    // Read L1D cache miss count from PMU
    crate::hal::cpu::read_cache_miss_counter()
}

/// Read branch misses
pub fn perf_read_branch_misses() -> u64 {
    // Read branch misprediction count from PMU
    crate::hal::cpu::read_branch_miss_counter()
}

/// Tracepoint
pub struct Tracepoint {
    /// Name
    pub name: [u8; 64],
    /// ID
    pub id: u32,
    /// Enabled
    pub enabled: AtomicBool,
    /// Probe function
    pub probe: Option<unsafe extern "C" fn(*mut core::ffi::c_void)>,
    /// Reference count
    pub ref_count: AtomicU32,
}

impl Tracepoint {
    pub fn new(name: &[u8], id: u32) -> Self {
        let mut name_arr = [0u8; 64];
        let len = name.len().min(63);
        name_arr[..len].copy_from_slice(&name[..len]);
        
        Tracepoint {
            name: name_arr,
            id,
            enabled: AtomicBool::new(false),
            probe: None,
            ref_count: AtomicU32::new(0),
        }
    }
    
    /// Enable
    pub fn enable(&self) {
        self.enabled.store(true, Ordering::Release);
    }
    
    /// Disable
    pub fn disable(&self) {
        self.enabled.store(false, Ordering::Release);
    }
    
    /// Call probe
    pub fn call(&self, data: *mut core::ffi::c_void) {
        if !self.enabled.load(Ordering::Acquire) {
            return;
        }
        
        if let Some(probe) = self.probe {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe { probe(data); }
        }
    }
}

/// Tracepoint macro
#[macro_export]
macro_rules! DEFINE_TRACEPOINT {
    ($name:ident, $probe:expr) => {
        static $name: $crate::kernel::perf::Tracepoint = {
            let mut tp = $crate::kernel::perf::Tracepoint::new(
                stringify!($name).as_bytes(),
                0,
            );
            tp.probe = Some($probe);
            tp
        };
    };
}
