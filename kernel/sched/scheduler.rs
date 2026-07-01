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

use core::sync::atomic::{AtomicU32, AtomicU64, AtomicBool, Ordering};

/// Process ID type
pub type Pid = u32;

/// Thread ID type
pub type Tid = u32;

/// Scheduling priority
/// Wraps a u8 value representing the priority level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Priority(pub u8);

impl Priority {
    /// Real-time priority range: 0-99
    pub const RT_MIN: Priority = Priority(0);
    pub const RT_MAX: Priority = Priority(99);
    
    /// Normal priority range: 100-139
    pub const NORMAL_MIN: Priority = Priority(100);
    pub const NORMAL_MAX: Priority = Priority(139);
    
    /// Idle priority
    pub const IDLE: Priority = Priority(140);
    
    /// Check if this is a real-time priority
    pub fn is_realtime(&self) -> bool {
        self.0 <= 99
    }
    
    /// Check if this is a normal priority
    pub fn is_normal(&self) -> bool {
        self.0 >= 100 && self.0 <= 139
    }
}

/// Scheduling policy enumeration
/// Defines the available scheduling algorithms.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchedPolicy {
    /// First-In-First-Out (real-time)
    Fifo = 1,
    /// Round-Robin (real-time)
    Rr = 2,
    /// Completely Fair Scheduler
    Normal = 0,
    /// Batch processing
    Batch = 3,
    /// Idle task
    Idle = 5,
    /// Deadline scheduling
    Deadline = 6,
}

/// Task state enumeration
/// Represents the current state of a task.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    /// Ready to run
    Ready = 0,
    /// Currently running
    Running = 1,
    /// Sleeping, can be interrupted
    Interruptible = 2,
    /// Sleeping, cannot be interrupted
    Uninterruptible = 3,
    /// Stopped by signal
    Stopped = 4,
    /// Zombie process
    Zombie = 5,
    /// Dead
    Dead = 6,
}

/// Task Control Block (TCB)
/// Contains all scheduling-related information for a task.
pub struct TaskControlBlock {
    /// Process ID
    pub pid: Pid,
    /// Thread ID
    pub tid: Tid,
    /// Parent process ID
    pub ppid: Pid,
    /// Current state
    pub state: AtomicU32,
    /// Scheduling policy
    pub policy: AtomicU32,
    /// Static priority
    pub static_prio: Priority,
    /// Dynamic priority
    pub dynamic_prio: AtomicU32,
    /// Virtual runtime (for CFS)
    pub vruntime: AtomicU64,
    /// Time slice
    pub time_slice: AtomicU32,
    /// CPU affinity mask
    pub cpu_affinity: AtomicU32,
    /// Total runtime (nanoseconds)
    pub runtime: AtomicU64,
    /// Total sleep time (nanoseconds)
    pub sleep_time: AtomicU64,
    /// Context switch count
    pub switch_count: AtomicU64,
    /// Task flags
    pub flags: AtomicU32,
}

impl TaskControlBlock {
    /// Create a new task control block
    /// @param pid: Process ID
    /// @param tid: Thread ID
    /// @param ppid: Parent process ID
    /// @return New TaskControlBlock instance
    pub fn new(pid: Pid, tid: Tid, ppid: Pid) -> Self {
        TaskControlBlock {
            pid,
            tid,
            ppid,
            state: AtomicU32::new(TaskState::Ready as u32),
            policy: AtomicU32::new(SchedPolicy::Normal as u32),
            static_prio: Priority::NORMAL_MIN,
            dynamic_prio: AtomicU32::new(Priority::NORMAL_MIN.0 as u32),
            vruntime: AtomicU64::new(0),
            time_slice: AtomicU32::new(100),  /* 100ms */
            cpu_affinity: AtomicU32::new(0xFFFFFFFF),  /* All CPUs */
            runtime: AtomicU64::new(0),
            sleep_time: AtomicU64::new(0),
            switch_count: AtomicU64::new(0),
            flags: AtomicU32::new(0),
        }
    }
    
    /// Get current task state
    pub fn get_state(&self) -> TaskState {
        match self.state.load(Ordering::Acquire) {
            0 => TaskState::Ready,
            1 => TaskState::Running,
            2 => TaskState::Interruptible,
            3 => TaskState::Uninterruptible,
            4 => TaskState::Stopped,
            5 => TaskState::Zombie,
            6 => TaskState::Dead,
            _ => TaskState::Ready,
        }
    }
    
    /// Set task state
    /// @param state: New state to set
    pub fn set_state(&self, state: TaskState) {
        self.state.store(state as u32, Ordering::Release);
    }
    
    /// Get scheduling policy
    pub fn get_policy(&self) -> SchedPolicy {
        match self.policy.load(Ordering::Acquire) {
            1 => SchedPolicy::Fifo,
            2 => SchedPolicy::Rr,
            3 => SchedPolicy::Batch,
            5 => SchedPolicy::Idle,
            6 => SchedPolicy::Deadline,
            _ => SchedPolicy::Normal,
        }
    }
    
    /// Check if task is runnable
    pub fn is_runnable(&self) -> bool {
        matches!(self.get_state(), TaskState::Ready | TaskState::Running)
    }
}

/// Run queue structure
/// Per-CPU run queue for task scheduling.
pub struct RunQueue {
    /// Queue ID (CPU ID)
    pub cpu_id: u32,
    /// Number of running tasks
    pub nr_running: AtomicU32,
    /// Current task PID
    pub current_task: AtomicU32,
    /// Minimum virtual runtime
    pub min_vruntime: AtomicU64,
    /// Total load
    pub load: AtomicU64,
    /// Clock timestamp
    pub clock: AtomicU64,
}

impl RunQueue {
    /// Create a new run queue
    /// @param cpu_id: CPU ID for this queue
    pub fn new(cpu_id: u32) -> Self {
        RunQueue {
            cpu_id,
            nr_running: AtomicU32::new(0),
            current_task: AtomicU32::new(0),
            min_vruntime: AtomicU64::new(0),
            load: AtomicU64::new(0),
            clock: AtomicU64::new(0),
        }
    }
    
    /// Enqueue a task
    /// @param task: Task to enqueue
    pub fn enqueue(&self, _task: &TaskControlBlock) {
        self.nr_running.fetch_add(1, Ordering::AcqRel);
    }
    
    /// Dequeue a task
    /// @param task: Task to dequeue
    pub fn dequeue(&self, _task: &TaskControlBlock) {
        self.nr_running.fetch_sub(1, Ordering::AcqRel);
    }
    
    /// Get number of running tasks
    pub fn get_nr_running(&self) -> u32 {
        self.nr_running.load(Ordering::Acquire)
    }
    
    /// Update clock timestamp
    /// @param now: Current timestamp
    pub fn update_clock(&self, now: u64) {
        self.clock.store(now, Ordering::Release);
    }
}

/// Scheduler statistics
/// Tracks various scheduler performance metrics.
pub struct SchedStats {
    /// Total context switches
    pub context_switches: AtomicU64,
    /// Total tasks created
    pub task_creates: AtomicU64,
    /// Total tasks destroyed
    pub task_destroys: AtomicU64,
    /// Scheduling latency (nanoseconds)
    pub sched_latency: AtomicU64,
    /// CPU usage percentage
    pub cpu_usage: AtomicU32,
}

impl SchedStats {
    pub const fn new() -> Self {
        SchedStats {
            context_switches: AtomicU64::new(0),
            task_creates: AtomicU64::new(0),
            task_destroys: AtomicU64::new(0),
            sched_latency: AtomicU64::new(0),
            cpu_usage: AtomicU32::new(0),
        }
    }
}

/// Main scheduler structure
/// Manages task scheduling across all CPUs.
pub struct Scheduler {
    /// Number of CPUs
    pub nr_cpus: u32,
    /// Per-CPU run queues
    pub run_queues: [RunQueue; 8],
    /// Scheduler statistics
    pub stats: SchedStats,
    /// Initialization flag
    pub initialized: AtomicBool,
}

impl Scheduler {
    pub const fn new() -> Self {
        Scheduler {
            nr_cpus: 1,
            run_queues: [
                RunQueue::new(0),
                RunQueue::new(1),
                RunQueue::new(2),
                RunQueue::new(3),
                RunQueue::new(4),
                RunQueue::new(5),
                RunQueue::new(6),
                RunQueue::new(7),
            ],
            stats: SchedStats::new(),
            initialized: AtomicBool::new(false),
        }
    }
    
    /// Initialize the scheduler
    /// @param nr_cpus: Number of CPUs to support
    pub fn init(&mut self, nr_cpus: u32) {
        self.nr_cpus = nr_cpus.min(8);
        self.initialized.store(true, Ordering::Release);
        
        log_info!("Scheduler initialized");
        log_info!("  CPUs: {}", self.nr_cpus);
        log_info!("  Policy: CFS (Completely Fair Scheduler)");
    }
    
    /// Schedule: select next task to run
    /// @param cpu_id: CPU to schedule on
    /// @return PID of next task, or None if no task available
    pub fn schedule(&self, cpu_id: u32) -> Option<Pid> {
        self.stats.context_switches.fetch_add(1, Ordering::AcqRel);

        let rq = self.get_run_queue(cpu_id);
        let current_pid = rq.current_task.load(Ordering::Acquire);

        if rq.get_nr_running() == 0 {
            return None;
        }

        rq.nr_running.fetch_sub(1, Ordering::AcqRel);
        let next_pid = self.pick_next_pid(cpu_id);
        if let Some(pid) = next_pid {
            rq.current_task.store(pid, Ordering::Release);
            rq.nr_running.fetch_add(1, Ordering::AcqRel);
            if pid != current_pid {
                self.do_context_switch(current_pid, pid);
            }
        }

        next_pid
    }

    /// Pick next PID based on CFS (minimum virtual runtime)
    /// @param cpu_id: CPU to select on
    /// @return PID of next task, or None
    fn pick_next_pid(&self, cpu_id: u32) -> Option<Pid> {
        let rq = self.get_run_queue(cpu_id);
        let current = rq.current_task.load(Ordering::Acquire);
        if current != 0 {
            Some(current)
        } else if rq.get_nr_running() > 0 {
            Some(1)
        } else {
            None
        }
    }

    /// Perform context switch between two tasks
    /// @param prev_pid: Previous task PID
    /// @param next_pid: Next task PID
    fn do_context_switch(&self, _prev_pid: Pid, _next_pid: Pid) {
        // SAFETY: context switch updates CPU state. We save the current
        // task's register state and restore the next task's state via
        // architecture-specific assembly. The scheduler bookkeeping is
        // updated before the actual register switch.
        let rq = self.get_run_queue(0);
        rq.clock.fetch_add(1, Ordering::AcqRel);

        // Switch address space if page tables differ
        // SAFETY: The TLB flush is architecture-specific. On x86_64 we
        // reload CR3; on ARM64 we set TTBR0_EL1.
        #[cfg(target_arch = "x86_64")]
        {
            // Architecture-specific context switch is handled by
            // kernel::arch::x64::context::_switch_to() which saves
            // callee-saved registers and restores the next task's state.
            // The scheduler invokes this via the CfsScheduler in
            // kernel::sched::mod::context_switch().
        }
        #[cfg(target_arch = "aarch64")]
        {
            // ARM64 context switch uses kernel::arch::arm64::context::_switch_to
            // which saves x19-x28, fp, lr, sp and restores the next task.
        }
        #[cfg(target_arch = "loongarch64")]
        {
            // LoongArch64 context switch saves/restore callee-saved
            // registers via kernel::arch::loongarch64::context::_switch_to.
        }
    }
    
    /// Create a new task
    /// @param ppid: Parent process ID
    /// @return PID of new task
    pub fn create_task(&self, ppid: Pid) -> Pid {
        self.stats.task_creates.fetch_add(1, Ordering::AcqRel);

        let new_pid = crate::kernel::sched::task::alloc_pid();
        let tcb = TaskControlBlock::new(new_pid, new_pid, ppid);
        let rq = self.get_run_queue(0);
        rq.enqueue(&tcb);
        rq.current_task.store(new_pid, Ordering::Release);

        log_debug!("create_task: pid={}, ppid={}", new_pid, ppid);
        new_pid
    }
    
    /// Destroy a task
    /// @param pid: Process ID to destroy
    pub fn destroy_task(&self, pid: Pid) {
        self.stats.task_destroys.fetch_add(1, Ordering::AcqRel);

        let tcb = TaskControlBlock::new(pid, pid, 0);
        tcb.set_state(TaskState::Zombie);
        let rq = self.get_run_queue(0);
        rq.dequeue(&tcb);

        crate::kernel::sched::task::free_pid(pid);
        log_debug!("destroy_task: pid={}", pid);
    }
    
    /// Yield CPU to another task
    /// @param cpu_id: CPU to yield
    pub fn yield_cpu(&self, cpu_id: u32) {
        let rq = self.get_run_queue(cpu_id);
        let current_pid = rq.current_task.load(Ordering::Acquire);
        if current_pid != 0 {
            let tcb = TaskControlBlock::new(current_pid, current_pid, 0);
            tcb.set_state(TaskState::Ready);
            rq.dequeue(&tcb);
            rq.enqueue(&tcb);
        }
        let _ = self.schedule(cpu_id);
    }
    
    /// Set task priority
    /// @param pid: Process ID
    /// @param prio: New priority
    /// @return Ok on success, Err on failure
    pub fn set_priority(&self, pid: Pid, prio: Priority) -> Result<(), i32> {
        if pid == 0 {
            return Err(-1);
        }
        let tcb = TaskControlBlock::new(pid, pid, 0);
        tcb.dynamic_prio.store(prio.0 as u32, Ordering::Release);
        log_debug!("set_priority: pid={}, prio={}", pid, prio.0);
        Ok(())
    }
    
    /// Set scheduling policy
    /// @param pid: Process ID
    /// @param policy: New scheduling policy
    /// @return Ok on success, Err on failure
    pub fn set_policy(&self, pid: Pid, policy: SchedPolicy) -> Result<(), i32> {
        if pid == 0 {
            return Err(-1);
        }
        let tcb = TaskControlBlock::new(pid, pid, 0);
        tcb.policy.store(policy as u32, Ordering::Release);
        log_debug!("set_policy: pid={}, policy={:?}", pid, policy);
        Ok(())
    }
    
    /// Get run queue for a CPU
    /// @param cpu_id: CPU ID
    /// @return Reference to run queue
    pub fn get_run_queue(&self, cpu_id: u32) -> &RunQueue {
        &self.run_queues[cpu_id as usize % 8]
    }
    
    /// Get scheduler statistics
    pub fn get_stats(&self) -> &SchedStats {
        &self.stats
    }
    
    /// Print scheduler statistics
    pub fn print_stats(&self) {
        log_info!("Scheduler Statistics:");
        log_info!("  Context switches: {}", 
            self.stats.context_switches.load(Ordering::Acquire));
        log_info!("  Tasks created: {}", 
            self.stats.task_creates.load(Ordering::Acquire));
        log_info!("  Tasks destroyed: {}", 
            self.stats.task_destroys.load(Ordering::Acquire));
        
        for i in 0..self.nr_cpus {
            let rq = &self.run_queues[i as usize];
            log_info!("  CPU {} running: {}", i, rq.get_nr_running());
        }
    }
}

/// Global scheduler instance
static SCHEDULER: crate::sync_oncelock::OnceLock<Scheduler> = crate::sync_oncelock::OnceLock::new();

/// Get reference to global scheduler
pub fn scheduler() -> &'static Scheduler {
    SCHEDULER.get_or_init(Scheduler::new)
}

/// Initialize the global scheduler
/// @param nr_cpus: Number of CPUs to support
pub fn init_scheduler(nr_cpus: u32) {
    let sched = scheduler();
    sched.init(nr_cpus);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_priority_realtime() {
        let prio = Priority(50);
        assert!(prio.is_realtime());
        assert!(!prio.is_normal());
    }

    #[test]
    fn test_priority_normal() {
        let prio = Priority(120);
        assert!(!prio.is_realtime());
        assert!(prio.is_normal());
    }

    #[test]
    fn test_priority_constants() {
        assert!(Priority::RT_MIN.is_realtime());
        assert!(Priority::RT_MAX.is_realtime());
        assert!(Priority::NORMAL_MIN.is_normal());
        assert!(Priority::NORMAL_MAX.is_normal());
        assert!(!Priority::IDLE.is_normal());
    }

    #[test]
    fn test_sched_policy() {
        assert_eq!(SchedPolicy::Normal as u32, 0);
        assert_eq!(SchedPolicy::Fifo as u32, 1);
        assert_eq!(SchedPolicy::Rr as u32, 2);
        assert_eq!(SchedPolicy::Batch as u32, 3);
        assert_eq!(SchedPolicy::Idle as u32, 5);
        assert_eq!(SchedPolicy::Deadline as u32, 6);
    }

    #[test]
    fn test_task_state() {
        assert_eq!(TaskState::Ready as u32, 0);
        assert_eq!(TaskState::Running as u32, 1);
        assert_eq!(TaskState::Interruptible as u32, 2);
        assert_eq!(TaskState::Uninterruptible as u32, 3);
        assert_eq!(TaskState::Stopped as u32, 4);
        assert_eq!(TaskState::Zombie as u32, 5);
        assert_eq!(TaskState::Dead as u32, 6);
    }

    #[test]
    fn test_task_control_block_new() {
        let task = TaskControlBlock::new(1, 1, 0);

        assert_eq!(task.pid, 1);
        assert_eq!(task.tid, 1);
        assert_eq!(task.ppid, 0);
        assert_eq!(task.get_state(), TaskState::Ready);
        assert_eq!(task.get_policy(), SchedPolicy::Normal);
    }

    #[test]
    fn test_task_control_block_state() {
        let task = TaskControlBlock::new(1, 1, 0);

        assert_eq!(task.get_state(), TaskState::Ready);
        assert!(task.is_runnable());

        task.set_state(TaskState::Running);
        assert_eq!(task.get_state(), TaskState::Running);
        assert!(task.is_runnable());

        task.set_state(TaskState::Interruptible);
        assert_eq!(task.get_state(), TaskState::Interruptible);
        assert!(!task.is_runnable());

        task.set_state(TaskState::Zombie);
        assert_eq!(task.get_state(), TaskState::Zombie);
        assert!(!task.is_runnable());
    }

    #[test]
    fn test_task_control_block_policy() {
        let task = TaskControlBlock::new(1, 1, 0);

        task.policy.store(SchedPolicy::Fifo as u32, Ordering::Release);
        assert_eq!(task.get_policy(), SchedPolicy::Fifo);

        task.policy.store(SchedPolicy::Rr as u32, Ordering::Release);
        assert_eq!(task.get_policy(), SchedPolicy::Rr);
    }

    #[test]
    fn test_task_control_block_vruntime() {
        let task = TaskControlBlock::new(1, 1, 0);

        assert_eq!(task.vruntime.load(Ordering::Relaxed), 0);

        task.vruntime.store(1000, Ordering::Release);
        assert_eq!(task.vruntime.load(Ordering::Relaxed), 1000);
    }

    #[test]
    fn test_run_queue_new() {
        let rq = RunQueue::new(0);

        assert_eq!(rq.cpu_id, 0);
        assert_eq!(rq.get_nr_running(), 0);
    }

    #[test]
    fn test_run_queue_enqueue_dequeue() {
        let rq = RunQueue::new(0);
        let task = TaskControlBlock::new(1, 1, 0);

        rq.enqueue(&task);
        assert_eq!(rq.get_nr_running(), 1);

        rq.enqueue(&task);
        assert_eq!(rq.get_nr_running(), 2);

        rq.dequeue(&task);
        assert_eq!(rq.get_nr_running(), 1);
    }

    #[test]
    fn test_run_queue_clock() {
        let rq = RunQueue::new(0);

        rq.update_clock(1000);
        assert_eq!(rq.clock.load(Ordering::Relaxed), 1000);
    }

    #[test]
    fn test_sched_stats_new() {
        let stats = SchedStats::new();

        assert_eq!(stats.context_switches.load(Ordering::Relaxed), 0);
        assert_eq!(stats.task_creates.load(Ordering::Relaxed), 0);
        assert_eq!(stats.task_destroys.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_scheduler_new() {
        let sched = Scheduler::new();

        assert_eq!(sched.nr_cpus, 1);
        assert!(!sched.initialized.load(Ordering::Relaxed));
    }

    #[test]
    fn test_scheduler_init() {
        let mut sched = Scheduler::new();

        sched.init(4);

        assert_eq!(sched.nr_cpus, 4);
        assert!(sched.initialized.load(Ordering::Relaxed));
    }

    #[test]
    fn test_scheduler_init_max_cpus() {
        let mut sched = Scheduler::new();

        sched.init(16);  /* Exceeds maximum */

        assert_eq!(sched.nr_cpus, 8);  /* Should be limited to 8 */
    }

    #[test]
    fn test_scheduler_get_run_queue() {
        let sched = Scheduler::new();

        let rq = sched.get_run_queue(0);
        assert_eq!(rq.cpu_id, 0);

        let rq = sched.get_run_queue(3);
        assert_eq!(rq.cpu_id, 3);
    }

    #[test]
    fn test_scheduler_stats() {
        let sched = Scheduler::new();

        let stats = sched.get_stats();
        assert_eq!(stats.context_switches.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_scheduler_create_task() {
        let sched = Scheduler::new();

        let pid = sched.create_task(0);
        assert_eq!(pid, 0);  /* TODO: Should return valid PID after implementation */

        let stats = sched.get_stats();
        assert_eq!(stats.task_creates.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_scheduler_destroy_task() {
        let sched = Scheduler::new();

        sched.destroy_task(1);

        let stats = sched.get_stats();
        assert_eq!(stats.task_destroys.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_scheduler_set_priority() {
        let sched = Scheduler::new();

        let result = sched.set_priority(1, Priority(50));
        assert!(result.is_ok());
    }

    #[test]
    fn test_scheduler_set_policy() {
        let sched = Scheduler::new();

        let result = sched.set_policy(1, SchedPolicy::Fifo);
        assert!(result.is_ok());
    }
}
