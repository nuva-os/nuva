/*
 * Nuva OS - Kernel - Core - Workqueue
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
 * Nuva OS - Kernel - Workqueue
 * 
 * Copyright (C) 2026 Nuva OS Team
 * 
 * Deferred work execution framework.
 */

use core::sync::atomic::{AtomicU32, AtomicU64, AtomicPtr, Ordering};

/// Work Function Type
pub type WorkFunc = unsafe extern "C" fn(*mut Work);

/// Work Structure
#[repr(C)]
pub struct Work {
    /// Entry for list
    pub entry: WorkEntry,
    /// Work function
    pub func: Option<WorkFunc>,
    /// Work data
    pub data: *mut core::ffi::c_void,
    /// Flags
    pub flags: AtomicU32,
    /// CPU
    pub cpu: AtomicU32,
}

/// Work Entry (for list)
#[repr(C)]
pub struct WorkEntry {
    pub next: AtomicPtr<Work>,
    pub prev: AtomicPtr<Work>,
}

impl Work {
    pub fn new(func: WorkFunc, data: *mut core::ffi::c_void) -> Self {
        Work {
            entry: WorkEntry {
                next: AtomicPtr::new(core::ptr::null_mut()),
                prev: AtomicPtr::new(core::ptr::null_mut()),
            },
            func: Some(func),
            data,
            flags: AtomicU32::new(0),
            cpu: AtomicU32::new(0),
        }
    }
    
    /// Initialize work
    pub fn init(&mut self, func: WorkFunc, data: *mut core::ffi::c_void) {
        self.func = Some(func);
        self.data = data;
        self.flags.store(0, Ordering::Release);
    }
    
    /// Check if pending
    pub fn is_pending(&self) -> bool {
        (self.flags.load(Ordering::Acquire) & WORK_PENDING) != 0
    }
}

/// Work Flags
pub mod work_flags {
    pub const WORK_PENDING: u32 = 0x01;
    pub const WORK_RUNNING: u32 = 0x02;
    pub const WORK_CANCELLED: u32 = 0x04;
    pub const WORK_DELAYED: u32 = 0x08;
}

use work_flags::*;

/// Delayed Work
#[repr(C)]
pub struct DelayedWork {
    /// Work structure
    pub work: Work,
    /// Timer
    pub timer: u64,
    /// Delay in jiffies
    pub delay: u64,
}

impl DelayedWork {
    pub fn new(func: WorkFunc, data: *mut core::ffi::c_void, delay: u64) -> Self {
        DelayedWork {
            work: Work::new(func, data),
            timer: 0,
            delay,
        }
    }
}

/// Workqueue
pub struct Workqueue {
    /// Name
    pub name: [u8; 32],
    /// Work list
    pub list: WorkList,
    /// Worker threads
    pub workers: [Worker; 8],
    /// Number of workers
    pub num_workers: u32,
    /// Flags
    pub flags: AtomicU32,
    /// Statistics
    pub stats: WqStats,
}

/// Work List
pub struct WorkList {
    pub head: AtomicPtr<Work>,
    pub tail: AtomicPtr<Work>,
    pub count: AtomicU32,
}

impl WorkList {
    pub const fn new() -> Self {
        WorkList {
            head: AtomicPtr::new(core::ptr::null_mut()),
            tail: AtomicPtr::new(core::ptr::null_mut()),
            count: AtomicU32::new(0),
        }
    }
    
    /// Add work to tail
    pub fn add_tail(&self, work: *mut Work) {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            (*work).entry.next.store(core::ptr::null_mut(), Ordering::Release);
            (*work).entry.prev.store(self.tail.load(Ordering::Acquire), Ordering::Release);
            
            let prev = self.tail.swap(work, Ordering::AcqRel);
            if !prev.is_null() {
                (*prev).entry.next.store(work, Ordering::Release);
            } else {
                self.head.store(work, Ordering::Release);
            }
            
            self.count.fetch_add(1, Ordering::AcqRel);
        }
    }
    
    /// Remove work from head
    pub fn remove_head(&self) -> *mut Work {
        let work = self.head.load(Ordering::Acquire);
        if work.is_null() {
            return core::ptr::null_mut();
        }
        
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let next = (*work).entry.next.load(Ordering::Acquire);
            self.head.store(next, Ordering::Release);
            
            if !next.is_null() {
                (*next).entry.prev.store(core::ptr::null_mut(), Ordering::Release);
            } else {
                self.tail.store(core::ptr::null_mut(), Ordering::Release);
            }
            
            self.count.fetch_sub(1, Ordering::AcqRel);
        }
        
        work
    }
    
    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.head.load(Ordering::Acquire).is_null()
    }
}

// Worker Thread
pub struct Worker {
    /// Worker ID
    pub id: u32,
    /// Current work
    pub current_work: AtomicPtr<Work>,
    /// State
    pub state: AtomicU32,
    /// CPU
    pub cpu: u32,
}

impl Worker {
    pub const fn new() -> Self {
        Worker {
            id: 0,
            current_work: AtomicPtr::new(core::ptr::null_mut()),
            state: AtomicU32::new(worker_state::IDLE),
            cpu: 0,
        }
    }
}

impl Clone for Worker {
    fn clone(&self) -> Self {
        Worker {
            id: self.id,
            current_work: AtomicPtr::new(self.current_work.load(Ordering::Relaxed)),
            state: AtomicU32::new(self.state.load(Ordering::Relaxed)),
            cpu: self.cpu,
        }
    }
}


/// Worker State
pub mod worker_state {
    pub const IDLE: u32 = 0;
    pub const RUNNING: u32 = 1;
    pub const SLEEPING: u32 = 2;
}

/// Workqueue Statistics
pub struct WqStats {
    pub queued: AtomicU64,
    pub executed: AtomicU64,
    pub cancelled: AtomicU64,
}

impl WqStats {
    pub const fn new() -> Self {
        WqStats {
            queued: AtomicU64::new(0),
            executed: AtomicU64::new(0),
            cancelled: AtomicU64::new(0),
        }
    }
}

impl Workqueue {
    pub fn new(name: &[u8], num_workers: u32) -> Self {
        let mut name_arr = [0u8; 32];
        let len = name.len().min(31);
        name_arr[..len].copy_from_slice(&name[..len]);
        
        Workqueue {
            name: name_arr,
            list: WorkList::new(),
            workers: [const { Worker {
                id: 0,
                current_work: AtomicPtr::new(core::ptr::null_mut()),
                state: AtomicU32::new(worker_state::IDLE),
                cpu: 0,
            } }; 8],
            num_workers: num_workers.min(8),
            flags: AtomicU32::new(0),
            stats: WqStats::new(),
        }
    }
    
    /// Initialize workqueue
    pub fn init(&self) {
        for i in 0..self.num_workers as usize {
            self.workers[i].id = i as u32;
            self.workers[i].state.store(worker_state::IDLE, Ordering::Release);
        }
        
        log_info!("Workqueue {:?} initialized with {} workers", 
                 &self.name, self.num_workers);
    }
    
    /// Queue work
    pub fn queue_work(&mut self, work: *mut Work) -> bool {
        if work.is_null() {
            return false;
        }
        
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            // Check if already pending
            if (*work).is_pending() {
                return false;
            }
            
            // Mark as pending
            (*work).flags.fetch_or(WORK_PENDING, Ordering::AcqRel);
            
            // Add to list
            self.list.add_tail(work);
            self.stats.queued.fetch_add(1, Ordering::AcqRel);
            
            // Wake up worker
            self.wake_worker();
        }
        
        true
    }
    
    /// Queue delayed work
    pub fn queue_delayed_work(&mut self, dwork: *mut DelayedWork, delay: u64) -> bool {
        if dwork.is_null() {
            return false;
        }
        
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            (*dwork).delay = delay;
            (*dwork).timer = crate::kernel::timer::get_jiffies() + delay;
            (*dwork).work.flags.fetch_or(WORK_PENDING | WORK_DELAYED, Ordering::AcqRel);
        }
        
        // TODO: Add to timer
        true
    }
    
    /// Cancel work
    pub fn cancel_work(&mut self, work: *mut Work) -> bool {
        if work.is_null() {
            return false;
        }
        
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let flags = (*work).flags.load(Ordering::Acquire);
            if (flags & WORK_RUNNING) != 0 {
                // Work is running, can't cancel
                return false;
            }
            
            (*work).flags.fetch_or(WORK_CANCELLED, Ordering::AcqRel);
            self.stats.cancelled.fetch_add(1, Ordering::AcqRel);
        }
        
        true
    }
    
    /// Flush workqueue
    pub fn flush(&mut self) {
        // Wait for all work to complete
        while !self.list.is_empty() {
            core::hint::spin_loop();
        }
    }
    
    /// Wake worker
    fn wake_worker(&mut self) {
        // Find idle worker
        for i in 0..self.num_workers as usize {
            if self.workers[i].state.load(Ordering::Acquire) == worker_state::IDLE {
                self.workers[i].state.store(worker_state::RUNNING, Ordering::Release);
                // TODO: Wake worker thread
                break;
            }
        }
    }
    
    /// Process work (called by worker)
    pub fn process_work(&mut self, worker_id: u32) {
        while let Some(work) = self.get_work() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                self.workers[worker_id as usize].current_work.store(work, Ordering::Release);
                
                // Mark as running
                (*work).flags.fetch_or(WORK_RUNNING, Ordering::AcqRel);
                (*work).flags.fetch_and(!WORK_PENDING, Ordering::AcqRel);
                
                // Execute work
                if let Some(func) = (*work).func {
                    func(work);
                }
                
                // Mark as done
                (*work).flags.fetch_and(!WORK_RUNNING, Ordering::AcqRel);
                
                self.stats.executed.fetch_add(1, Ordering::AcqRel);
            }
        }
        
        self.workers[worker_id as usize].state.store(worker_state::IDLE, Ordering::Release);
    }
    
    /// Get work from queue
    fn get_work(&mut self) -> Option<*mut Work> {
        let work = self.list.remove_head();
        if work.is_null() {
            None
        } else {
            Some(work)
        }
    }
}

/// Workqueue Manager
pub struct WorkqueueManager {
    /// System workqueue
    pub system_wq: Workqueue,
    /// High priority workqueue
    pub highpri_wq: Workqueue,
    /// Unbound workqueue
    pub unbound_wq: Workqueue,
    /// Statistics
    pub stats: WqMgrStats,
}

/// Workqueue Manager Statistics
pub struct WqMgrStats {
    pub workqueues: AtomicU32,
    pub total_queued: AtomicU64,
    pub total_executed: AtomicU64,
}

impl WqMgrStats {
    pub const fn new() -> Self {
        WqMgrStats {
            workqueues: AtomicU32::new(0),
            total_queued: AtomicU64::new(0),
            total_executed: AtomicU64::new(0),
        }
    }
}

impl WorkqueueManager {
    pub const fn new() -> Self {
        WorkqueueManager {
            system_wq: Workqueue {
                name: *b"system\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                list: WorkList::new(),
                workers: [const { Worker::new() }; 8],
                num_workers: 4,
                flags: AtomicU32::new(0),
                stats: WqStats::new(),
            },
            highpri_wq: Workqueue {
                name: *b"highpri\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                list: WorkList::new(),
                workers: [const { Worker::new() }; 8],
                num_workers: 2,
                flags: AtomicU32::new(0),
                stats: WqStats::new(),
            },
            unbound_wq: Workqueue {
                name: *b"unbound\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                list: WorkList::new(),
                workers: [const { Worker::new() }; 8],
                num_workers: 4,
                flags: AtomicU32::new(0),
                stats: WqStats::new(),
            },
            stats: WqMgrStats::new(),
        }
    }
    
    /// Initialize
    pub fn init(&self) {
        self.system_wq.init();
        self.highpri_wq.init();
        self.unbound_wq.init();
        
        log_info!("Workqueue manager initialized");
    }
    
    /// Queue work on system workqueue
    pub fn queue_system_work(&mut self, work: *mut Work) -> bool {
        self.system_wq.queue_work(work)
    }
    
    /// Queue work on high priority workqueue
    pub fn queue_highpri_work(&mut self, work: *mut Work) -> bool {
        self.highpri_wq.queue_work(work)
    }
}

/// Global workqueue manager
static WQ_MANAGER: core::sync::OnceLock<WorkqueueManager> = core::sync::OnceLock::new();

/// Get workqueue manager
pub fn wq_manager() -> &'static WorkqueueManager {
    WQ_MANAGER.get_or_init(WorkqueueManager::new)
}

pub fn init_wq_manager() -> &'static WorkqueueManager {
    WQ_MANAGER.get_or_init(WorkqueueManager::new)
}

/// Initialize workqueue
pub fn init_workqueue() {
    let mgr = wq_manager();
    mgr.init();
}

// Convenience macros/functions

/// Queue work on system workqueue
pub fn schedule_work(work: *mut Work) -> bool {
    wq_manager().queue_system_work(work)
}

/// Queue delayed work
pub fn schedule_delayed_work(dwork: *mut DelayedWork, delay: u64) -> bool {
    wq_manager().system_wq.queue_delayed_work(dwork, delay)
}
