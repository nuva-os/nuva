/*
 * Nuva OS - Kernel - Scheduler
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

// Scheduler submodules
// Re-export print macros from crate root
pub use crate::{pr_emerg, pr_alert, pr_crit, pr_err, pr_warn, pr_notice, pr_info, pr_debug};
pub mod rbtree;
pub mod sched_domain;
pub mod eas;
pub mod task;
pub mod ai_sched;
pub mod quant_sched;
pub mod declarative;
pub mod nv_policy;
pub mod nvsched;
pub mod nvbalancer;

// Re-export key types
pub use rbtree::{RbTree, RbNode};
pub use sched_domain::{SchedDomain, SchedGroup, LoadBalancer};
pub use eas::{EnergyModel, PerfDomain, EasData, eas_select_task_rq};
pub use nv_policy::{NvSchedPolicy, NvDeadlineParams, NvEnergyAwareParams, NvAiOptimizedParams, NvSchedConfig};

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

use crate::kernel::process::{Pid, ProcessState};
use crate::kernel::error::KernelError;
// Migrated from: posix::errno::Errno → KernelError in scheduler
// Scheduling policy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchedPolicy {
    /// Normal scheduling
    Normal = 0,
    /// First-In-First-Out real-time
    Fifo = 1,
    /// Round-robin real-time
    Rr = 2,
    /// Batch scheduling
    Batch = 3,
    /// Idle scheduling
    Idle = 4,
    /// Deadline scheduling
    Deadline = 5,
}

/// Scheduling priority
pub struct SchedPriority {
    /// Static priority (1-99 for RT, 0 for normal)
    pub static_prio: i32,
    /// Normal priority
    pub normal_prio: i32,
    /// Dynamic priority
    pub prio: i32,
    /// Real-time priority
    pub rt_priority: i32,
}

impl SchedPriority {
    /// Create new priority
    pub fn new(prio: i32) -> Self {
        SchedPriority {
            static_prio: prio,
            normal_prio: prio,
            prio: prio,
            rt_priority: 0,
        }
    }
    
    /// Create real-time priority
    pub fn new_rt(rt_prio: i32) -> Self {
        SchedPriority {
            static_prio: 100 - rt_prio,
            normal_prio: 100 - rt_prio,
            prio: 100 - rt_prio,
            rt_priority: rt_prio,
        }
    }
}

/// Run queue
pub struct RunQueue {
    /// CPU number
    pub cpu: u32,
    /// Number of running tasks
    pub nr_running: AtomicU32,
    /// Number of runnable tasks
    pub nr_running_total: AtomicU32,
    /// Load weight
    pub load_weight: AtomicU64,
    /// Clock
    pub clock: AtomicU64,
    /// Idle task
    pub idle: *mut Task,
    /// Current task
    pub curr: *mut Task,
    /// Next task to run
    pub next: *mut Task,
    /// Priority arrays
    pub active: PrioArray,
    /// Expired priority array
    pub expired: PrioArray,
    /// Expired timestamp
    pub expired_timestamp: AtomicU64,
    /// RT run queue
    pub rt_rq: RtRunQueue,
}

impl RunQueue {
    pub const fn new() -> Self {
        RunQueue {
            cpu: 0,
            nr_running: AtomicU32::new(0),
            nr_running_total: AtomicU32::new(0),
            load_weight: AtomicU64::new(0),
            clock: AtomicU64::new(0),
            idle: core::ptr::null_mut(),
            curr: core::ptr::null_mut(),
            next: core::ptr::null_mut(),
            active: PrioArray::new(),
            expired: PrioArray::new(),
            expired_timestamp: AtomicU64::new(0),
            rt_rq: RtRunQueue::new(),
        }
    }
}


impl Clone for RunQueue {
    fn clone(&self) -> Self {
        Self {
            cpu: self.cpu.clone(),
            nr_running: AtomicU32::new(self.nr_running.load(core::sync::atomic::Ordering::Relaxed)),
            nr_running_total: AtomicU32::new(self.nr_running_total.load(core::sync::atomic::Ordering::Relaxed)),
            load_weight: AtomicU64::new(self.load_weight.load(core::sync::atomic::Ordering::Relaxed)),
            clock: AtomicU64::new(self.clock.load(core::sync::atomic::Ordering::Relaxed)),
            idle: self.idle.clone(),
            curr: self.curr.clone(),
            next: self.next.clone(),
            active: self.active.clone(),
            expired: self.expired.clone(),
            expired_timestamp: AtomicU64::new(self.expired_timestamp.load(core::sync::atomic::Ordering::Relaxed)),
            rt_rq: self.rt_rq.clone(),
        }
    }
}

// Priority array

pub struct PrioArray {
    /// Bitmap of active priorities
    pub bitmap: AtomicU64,
    /// Queue for each priority
    pub queue: [TaskList; 140],
}

impl Clone for PrioArray {
    fn clone(&self) -> Self {
        PrioArray {
            bitmap: AtomicU64::new(self.bitmap.load(Ordering::Relaxed)),
            queue: self.queue.clone(),
        }
    }
}


/// Task list (simplified)
pub struct TaskList {
    pub head: *mut Task,
    pub tail: *mut Task,
    pub count: AtomicU32,
}

impl Clone for TaskList {
    fn clone(&self) -> Self {
        Self {
            head: self.head.clone(),
            tail: self.tail.clone(),
            count: AtomicU32::new(self.count.load(core::sync::atomic::Ordering::Relaxed)),
        }
    }
}

impl PrioArray {
    pub const fn new() -> Self {
        PrioArray {
            bitmap: AtomicU64::new(0),
            queue: [const { TaskList {
                head: core::ptr::null_mut(),
                tail: core::ptr::null_mut(),
                count: AtomicU32::new(0),
            } }; 140],
        }
    }
    
    /// Find first bit set
    pub fn find_first_bit(&self) -> i32 {
        let bits = self.bitmap.load(Ordering::Acquire);
        if bits == 0 {
            return Errno::Eperm.to_ret_i32();
        }
        
        for i in 0..64 {
            if (bits & (1u64 << i)) != 0 {
                return i as i32;
            }
        }
        
        -1
    }
    
    /// Add task to queue
    pub fn enqueue(&mut self, task: *mut Task, prio: i32) {
        let idx = prio as usize;
        if idx >= 140 {
            return;
        }
        
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            (*task).run_list.next = self.queue[idx].head;
            if !self.queue[idx].head.is_null() {
                (*self.queue[idx].head).run_list.prev = task;
            }
            self.queue[idx].head = task;
            if self.queue[idx].tail.is_null() {
                self.queue[idx].tail = task;
            }
            self.queue[idx].count.fetch_add(1, Ordering::AcqRel);
        }
        
        self.bitmap.fetch_or(1u64 << idx, Ordering::AcqRel);
    }
    
    /// Remove task from queue
    pub fn dequeue(&mut self, task: *mut Task, prio: i32) {
        let idx = prio as usize;
        if idx >= 140 {
            return;
        }
        
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let prev = (*task).run_list.prev;
            let next = (*task).run_list.next;
            
            if !prev.is_null() {
                (*prev).run_list.next = next;
            } else {
                self.queue[idx].head = next;
            }
            
            if !next.is_null() {
                (*next).run_list.prev = prev;
            } else {
                self.queue[idx].tail = prev;
            }
            
            self.queue[idx].count.fetch_sub(1, Ordering::AcqRel);
            
            if self.queue[idx].count.load(Ordering::Acquire) == 0 {
                self.bitmap.fetch_and(!(1u64 << idx), Ordering::AcqRel);
            }
        }
    }
}

/// Real-time run queue
pub struct RtRunQueue {
    /// Number of RT tasks
    pub rt_nr_running: AtomicU32,
    /// Highest priority RT task
    pub highest_prio: AtomicU32,
    /// RT priority array
    pub active: PrioArray,
}

impl RtRunQueue {
    pub const fn new() -> Self {
        RtRunQueue {
            rt_nr_running: AtomicU32::new(0),
            highest_prio: AtomicU32::new(100),
            active: PrioArray::new(),
        }
    }
}

impl Clone for RtRunQueue {
    fn clone(&self) -> Self {
        RtRunQueue {
            rt_nr_running: AtomicU32::new(self.rt_nr_running.load(Ordering::Relaxed)),
            highest_prio: AtomicU32::new(self.highest_prio.load(Ordering::Relaxed)),
            active: self.active.clone(),
        }
    }
}

/// Task structure (simplified)
pub struct Task {
    /// Process ID
    pub pid: Pid,
    /// Thread ID
    pub tid: u32,
    /// Task state
    pub state: AtomicU32,
    /// Scheduling policy
    pub policy: AtomicU32,
    /// Priority
    pub prio: SchedPriority,
    /// Run list
    pub run_list: TaskListEntry,
    /// CPU affinity
    pub cpus_allowed: AtomicU64,
    /// Current CPU
    pub cpu: AtomicU32,
    /// Time slice
    pub time_slice: AtomicU32,
    /// Runtime
    pub runtime: AtomicU64,
    /// Virtual runtime (CFS)
    pub vruntime: AtomicU64,
    /// Last switch time
    pub last_switch: AtomicU64,
    /// Context
    pub context: TaskContext,
    /// Kernel stack
    pub kstack: u64,
    /// User stack
    pub ustack: u64,
    /// Flags
    pub flags: AtomicU32,
    /// Next task in list
    pub next: *mut Task,
    /// Previous task in list
    pub prev: *mut Task,
}

/// Task list entry
pub struct TaskListEntry {
    pub next: *mut Task,
    pub prev: *mut Task,
}

/// Task context
#[repr(C)]
pub struct TaskContext {
    /// General purpose registers
    pub regs: [u64; 31],
    /// Stack pointer
    pub sp: u64,
    /// Program counter
    pub pc: u64,
    /// Processor state
    pub pstate: u64,
    /// TPIDR_EL0 (TLS)
    pub tpidr: u64,
    /// TPIDRRO_EL0 (TLS read-only)
    pub tpidrro: u64,
}

/// Task flags
pub mod task_flags {
    pub const TF_KTHREAD: u32 = 0x01;
    pub const TF_EXITING: u32 = 0x02;
    pub const TF_NEED_RESCHED: u32 = 0x04;
    pub const TF_MIGRATING: u32 = 0x08;
    pub const TF_NOLOAD: u32 = 0x10;
}

impl Task {
    /// Create new task
    pub fn new(pid: Pid) -> Self {
        Task {
            pid,
            tid: pid,
            state: AtomicU32::new(ProcessState::Creating as u32),
            policy: AtomicU32::new(SchedPolicy::Normal as u32),
            prio: SchedPriority::new(120),
            run_list: TaskListEntry {
                next: core::ptr::null_mut(),
                prev: core::ptr::null_mut(),
            },
            cpus_allowed: AtomicU64::new(0xFFFF),  /* All CPUs */
            cpu: AtomicU32::new(0),
            time_slice: AtomicU32::new(100),
            runtime: AtomicU64::new(0),
            vruntime: AtomicU64::new(0),
            last_switch: AtomicU64::new(0),
            context: TaskContext {
                regs: [0; 31],
                sp: 0,
                pc: 0,
                pstate: 0,
                tpidr: 0,
                tpidrro: 0,
            },
            kstack: 0,
            ustack: 0,
            flags: AtomicU32::new(0),
            next: core::ptr::null_mut(),
            prev: core::ptr::null_mut(),
        }
    }
    
    /// Check if task is running
    pub fn is_running(&self) -> bool {
        self.state.load(Ordering::Acquire) == ProcessState::Running as u32
    }
    
    /// Check if task is runnable
    pub fn is_runnable(&self) -> bool {
        let state = self.state.load(Ordering::Acquire);
        state == ProcessState::Ready as u32 || state == ProcessState::Running as u32
    }
    
    /// Set need reschedule flag
    pub fn set_need_resched(&self) {
        self.flags.fetch_or(task_flags::TF_NEED_RESCHED, Ordering::AcqRel);
    }
    
    /// Clear need reschedule flag
    pub fn clear_need_resched(&self) {
        self.flags.fetch_and(!task_flags::TF_NEED_RESCHED, Ordering::AcqRel);
    }
    
    /// Check if reschedule needed
    pub fn need_resched(&self) -> bool {
        (self.flags.load(Ordering::Acquire) & task_flags::TF_NEED_RESCHED) != 0
    }
}

/// Scheduler
pub struct Scheduler {
    /// Run queues per CPU
    pub run_queues: [RunQueue; 16],
    /// Number of CPUs
    pub nr_cpus: u32,
    /// Number of tasks
    pub nr_tasks: AtomicU32,
    /// Number of running tasks
    pub nr_running: AtomicU32,
    /// Context switch count
    pub nr_switches: AtomicU64,
    /// Current task per CPU
    pub current: [*mut Task; 16],
    /// Idle task per CPU
    pub idle_tasks: [*mut Task; 16],
    /// Init task
    pub init_task: *mut Task,
    /// Task list
    pub task_list: *mut Task,
    /// Scheduler clock
    pub clock: AtomicU64,
    /// Tick count
    pub ticks: AtomicU64,
}

impl Scheduler {
    pub const fn new() -> Self {
        Scheduler {
            run_queues: [const { RunQueue::new() }; 16],
            nr_cpus: 1,
            nr_tasks: AtomicU32::new(0),
            nr_running: AtomicU32::new(0),
            nr_switches: AtomicU64::new(0),
            current: [core::ptr::null_mut(); 16],
            idle_tasks: [core::ptr::null_mut(); 16],
            init_task: core::ptr::null_mut(),
            task_list: core::ptr::null_mut(),
            clock: AtomicU64::new(0),
            ticks: AtomicU64::new(0),
        }
    }
    
    /// Initialize scheduler
    pub fn init(&mut self, nr_cpus: u32) {
        self.nr_cpus = nr_cpus.min(16);
        
        log_info!("Scheduler initialized for {} CPUs", self.nr_cpus);
        
        // Create idle tasks
        for cpu in 0..self.nr_cpus {
            self.create_idle_task(cpu as usize);
            self.run_queues[cpu as usize].cpu = cpu as u32;
        }
        
        // Create init task
        self.create_init_task();
    }
    
    /// Create idle task
    fn create_idle_task(&mut self, cpu: usize) {
        // Allocate and initialize the idle task for a CPU.
        // The idle task (PID 0) is a per-CPU kernel thread that runs
        // when no other tasks are runnable. It must:
        // 1. Be allocated from the task slab cache
        // 2. Have PID 0 and be a kernel thread (TF_KTHREAD)
        // 3. Be permanently on the run queue (TF_NOLOAD)
        // 4. Have the lowest possible priority
        // 5. Execute the cpu_idle_loop() function
        // In a full implementation:
        // let idle = alloc_task_struct();
        // idle.pid = 0;
        // idle.state = TASK_RUNNING;
        // idle.flags = TF_KTHREAD | TF_NOLOAD;
        // idle.prio = MAX_PRIO;  // Lowest priority
        // idle.cpu = cpu;
        // idle.context.pc = cpu_idle_loop as u64;
        // self.idle_tasks[cpu] = idle;
        // self.run_queues[cpu].idle = idle;
        // self.run_queues[cpu].curr = idle;
        // self.current[cpu] = idle;
        let idle = Task::new(0);
        let _ = idle;

        // Store idle task pointer in the run queue
        // In a full implementation, idle would be heap-allocated
    }
    
    /// Create init task
    fn create_init_task(&mut self) {
        // Allocate and initialize the init task (PID 1).
        // The init task is the first user-space process, created
        // from the initramfs or the init= kernel parameter.
        // It must:
        // 1. Be allocated from the task slab cache
        // 2. Have PID 1 and be a kernel thread initially
        // 3. Be set to TASK_RUNNING state
        // 4. Be enqueued in CPU 0's run queue
        // 5. Be stored as the init_task global
        // In a full implementation:
        // let init = alloc_task_struct();
        // init.pid = 1;
        // init.state = TASK_RUNNING;
        // init.prio = DEFAULT_PRIO;  // 120
        // init.cpu = 0;
        // init.flags = TF_KTHREAD;  // Initially a kernel thread
        // init.context.pc = kernel_init as u64;
        // self.init_task = init;
        // self.add_task(init);
        let init = Task::new(1);
        let _ = init;

        // Store init task and add to run queue
        // In a full implementation, init would be heap-allocated
    }
    
    /// Add task to scheduler
    pub fn add_task(&mut self, task: *mut Task) {
        if task.is_null() {
            return;
        }
        
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            // Add to task list
            (*task).next = self.task_list;
            if !self.task_list.is_null() {
                (*self.task_list).prev = task;
            }
            self.task_list = task;
            
            // Set state to ready
            (*task).state.store(ProcessState::Ready as u32, Ordering::Release);
            
            // Enqueue in run queue
            let cpu = (*task).cpu.load(Ordering::Acquire) as usize;
            let prio = (*task).prio.prio;
            self.run_queues[cpu].active.enqueue(task, prio);
            
            // Update counts
            self.nr_tasks.fetch_add(1, Ordering::AcqRel);
            self.nr_running.fetch_add(1, Ordering::AcqRel);
            self.run_queues[cpu].nr_running.fetch_add(1, Ordering::AcqRel);
        }
    }
    
    /// Remove task from scheduler
    pub fn remove_task(&mut self, task: *mut Task) {
        if task.is_null() {
            return;
        }
        
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            // Remove from task list
            let prev = (*task).prev;
            let next = (*task).next;
            
            if !prev.is_null() {
                (*prev).next = next;
            } else {
                self.task_list = next;
            }
            
            if !next.is_null() {
                (*next).prev = prev;
            }
            
            // Dequeue from run queue
            let cpu = (*task).cpu.load(Ordering::Acquire) as usize;
            let prio = (*task).prio.prio;
            self.run_queues[cpu].active.dequeue(task, prio);
            
            // Update counts
            self.nr_tasks.fetch_sub(1, Ordering::AcqRel);
            if (*task).is_runnable() {
                self.nr_running.fetch_sub(1, Ordering::AcqRel);
                self.run_queues[cpu].nr_running.fetch_sub(1, Ordering::AcqRel);
            }
        }
    }
    
    /// Pick next task to run
    pub fn pick_next_task(&mut self, cpu: usize) -> *mut Task {
        let rq = &mut self.run_queues[cpu];

        // Check RT tasks first (SCHED_FIFO and SCHED_RR)
        if rq.rt_rq.rt_nr_running.load(Ordering::Acquire) > 0 {
            let prio = rq.rt_rq.active.find_first_bit();
            if prio >= 0 {
                // Get the highest-priority RT task from the queue.
                // In a full implementation:
                // let idx = prio as usize;
                // let task = rq.rt_rq.active.queue[idx].head;
                // if !task.is_null() {
                // return task;
                // }
                let idx = prio as usize;
                if !rq.rt_rq.active.queue[idx].head.is_null() {
                    return rq.rt_rq.active.queue[idx].head;
                }
            }
        }

        // Check normal tasks (CFS / SCHED_NORMAL)
        let prio = rq.active.find_first_bit();
        if prio >= 0 {
            // Get the highest-priority normal task from the queue.
            // In a full implementation with CFS:
            // let task = rq.cfs.rb_leftmost;
            // if !task.is_null() {
            // return task;
            // }
            // With the O(1) scheduler, we pick from the active
            // priority array's highest-priority queue.
            let idx = prio as usize;
            if !rq.active.queue[idx].head.is_null() {
                return rq.active.queue[idx].head;
            }
        }

        // No runnable tasks - return idle task
        rq.idle
    }
    
    /// Schedule: switch to next task
    pub fn schedule(&mut self) {
        let cpu = 0;  /* TODO: Get current CPU */
        let prev;
        {
            let rq = &mut self.run_queues[cpu];
            prev = rq.curr;
        }
        if prev.is_null() {
            return;
        }

        // Pick next task
        let next = self.pick_next_task(cpu);
        if next.is_null() || next == prev {
            return;
        }
        
        // Update run queue
        self.run_queues[cpu].curr = next;
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            (*next).state.store(ProcessState::Running as u32, Ordering::Release);
            (*prev).state.store(ProcessState::Ready as u32, Ordering::Release);
        }
        
        // Update statistics
        self.nr_switches.fetch_add(1, Ordering::AcqRel);
        
        // Perform context switch
        self.context_switch(prev, next);
    }
    
    /// Context switch
    fn context_switch(&mut self, prev: *mut Task, next: *mut Task) {
        if prev.is_null() || next.is_null() {
            return;
        }
        
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            // Save current context
            self.save_context(prev);
            
            // Switch to new context
            self.switch_to(prev, next);
            
            // Restore new context
            self.restore_context(next);
        }
    }
    
    /// Save task context
    fn save_context(&mut self, task: *mut Task) {
        if task.is_null() {
            return;
        }
        
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            // Get current CPU context
            let cpu = 0;
            let rq = &self.run_queues[cpu];
            
            // Save current register state to task
            // In real implementation, this would be done by assembly code
            // Here we save the essential state
            
            // Save stack pointer
            let sp: u64;
            core::arch::asm!(
                "mov {}, rsp",
                out(reg) sp,
                options(nostack, preserves_flags)
            );
            (*task).context.sp = sp;
            
            // Save instruction pointer (return address)
            let ip: u64;
            core::arch::asm!(
                "lea 0(%rip), {}",
                out(reg) ip,
                options(nostack, preserves_flags)
            );
            (*task).context.pc = ip;
            
            // Save flags
            let flags: u64;
            core::arch::asm!(
                "pushfq",
                "pop {}",
                out(reg) flags,
                options(nostack, preserves_flags)
            );
            (*task).context.pstate = flags;
            
            // Update task state
            (*task).state.store(ProcessState::Running as u32, Ordering::Release);
        }
    }
    
    /// Switch to new task
    fn switch_to(&mut self, prev: *mut Task, next: *mut Task) {
        if next.is_null() {
            return;
        }
        
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            // Update run queue current
            let cpu = 0;
            let rq = &mut self.run_queues[cpu];
            rq.curr = next;
            
            // Update task states
            if !prev.is_null() {
                (*prev).state.store(ProcessState::Ready as u32, Ordering::Release);
            }
            (*next).state.store(ProcessState::Running as u32, Ordering::Release);
            
            // Switch page tables if needed
            let prev_pml4: u64 = 0;
            let next_pml4: u64 = 0;
            
            if prev_pml4 != next_pml4 && next_pml4 != 0 {
                // Load new page table
                core::arch::asm!(
                    "mov cr3, {}",
                    in(reg) next_pml4,
                    options(nostack, preserves_flags)
                );
            }
            
            // Update statistics
            self.nr_switches.fetch_add(1, Ordering::AcqRel);
        }
    }
    
    /// Restore task context
    fn restore_context(&mut self, task: *mut Task) {
        if task.is_null() {
            return;
        }
        
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            // Restore task context from saved state
            // In real implementation, this would be done by assembly code
            
            let sp = (*task).context.sp;
            let flags = (*task).context.pstate;
            
            // Restore flags
            core::arch::asm!(
                "push {}",
                "popfq",
                in(reg) flags,
                options(nostack, preserves_flags)
            );
            
            // Restore stack pointer and jump to saved IP
            // This is a simplified version - real implementation would use
            // a proper context switch assembly routine
            let ip = (*task).context.pc;
            
            core::arch::asm!(
                "mov rsp, {}",
                "jmp {}",
                in(reg) sp,
                in(reg) ip,
                options(nostack, noreturn)
            );
        }
    }
    
    /// Timer tick
    pub fn tick(&mut self) {
        self.ticks.fetch_add(1, Ordering::AcqRel);
        self.clock.fetch_add(1, Ordering::AcqRel);
        
        let cpu = 0;  /* TODO: Get current CPU */
        let rq = &mut self.run_queues[cpu];
        rq.clock.fetch_add(1, Ordering::AcqRel);
        
        // Update current task
        let curr = rq.curr;
        if !curr.is_null() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                // Decrement time slice
                let slice = (*curr).time_slice.fetch_sub(1, Ordering::AcqRel);
                if slice <= 1 {
                    // Time slice expired, need reschedule
                    (*curr).set_need_resched();
                    (*curr).time_slice.store(100, Ordering::Release);
                }
            }
        }
        
        // Check if reschedule needed
        if !curr.is_null() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                if (*curr).need_resched() {
                    self.schedule();
                }
            }
        }
    }
    
    /// Yield current CPU
    pub fn yield_cpu(&mut self) {
        let cpu = 0;  /* TODO: Get current CPU */
        let curr = self.run_queues[cpu].curr;
        
        if !curr.is_null() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                (*curr).set_need_resched();
            }
        }
        
        self.schedule();
    }
    
    /// Get current task
    pub fn get_current(&self) -> *mut Task {
        let cpu = 0;  /* TODO: Get current CPU */
        self.current[cpu]
    }
    
    /// Get task by PID
    pub fn get_task_by_pid(&self, pid: Pid) -> *mut Task {
        let mut task = self.task_list;
        while !task.is_null() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                if (*task).pid == pid {
                    return task;
                }
                task = (*task).next;
            }
        }
        core::ptr::null_mut()
    }
    
    /// Print scheduler statistics
    pub fn print_stats(&self) {
        log_info!("Scheduler Statistics:");
        log_info!("  Tasks: {}", self.nr_tasks.load(Ordering::Acquire));
        log_info!("  Running: {}", self.nr_running.load(Ordering::Acquire));
        log_info!("  Switches: {}", self.nr_switches.load(Ordering::Acquire));
        log_info!("  Ticks: {}", self.ticks.load(Ordering::Acquire));
    }
}

/// Global scheduler
static SCHEDULER: core::sync::OnceLock<Scheduler> = core::sync::OnceLock::new();

/// Get scheduler
pub fn scheduler() -> &'static Scheduler {
    SCHEDULER.get_or_init(Scheduler::new)
}

/// Initialize scheduler
pub fn init_scheduler(nr_cpus: u32) {
    let sched = get_scheduler();
    sched.init(nr_cpus);
}

/// Schedule
pub fn schedule() {
    get_scheduler().schedule();
}

/// Yield CPU
pub fn yield_cpu() {
    get_scheduler().yield_cpu();
}

/// Get current task
pub fn get_current_task() -> *mut Task {
    get_scheduler().get_current()
}

/// Scheduler tick (called from timer interrupt)
pub fn scheduler_tick() {
    get_scheduler().tick();
}

// Task state enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    /// Task is running
    Running = 0,
    /// Task is ready to run
    Ready = 1,
    /// Task is blocked/sleeping
    Blocked = 2,
    /// Task has exited
    Exited = 3,
}

/// Enqueue a task into the scheduler
pub fn enqueue_task(task: *mut Task) {
    if task.is_null() {
        return;
    }
    let sched = get_scheduler();
    // SAFETY: task pointer is valid, checked above
    unsafe {
        (*task).state.store(TaskState::Ready as u32, Ordering::Release);
    }
    sched.nr_running.fetch_add(1, Ordering::AcqRel);
    sched.nr_tasks.fetch_add(1, Ordering::AcqRel);
}

/// Get current process descriptor
pub fn current_process_desc() -> *mut Task {
    get_current_task()
}

/// Set scheduler latency
pub fn set_sched_latency(_latency_us: u64) {
    // TODO: implement scheduler latency configuration
}
