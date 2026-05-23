/*
* Nuva OS - Kernel - Kernel
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

/// Process ID
pub type Pid = u32;

/// Thread ID
pub type Tid = u32;

/// CPU ID
pub type CpuId = u32;

/// Priority
pub type Priority = i32;

/// tuneDegreepolicy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchedPolicy {
    /// tuneDegree
    Normal = 0,
    /// firstenterfirstexit
    Fifo = 1,
    /// roundbranch
    RoundRobin = 2,
    /// Batch Processing
    Batch = 3,
    /// emptyidle
    Idle = 4,
    /// deadlineTime
    Deadline = 5,
}

/// ProcessState
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    /// runinfix
    Running = 0,
    /// canInterrupt
    Interruptible = 1,
    /// notcanInterrupt
    Uninterruptible = 2,
    /// alreadyStop
    Stopped = 3,
    /// Trackinginfix
    Traced = 4,
    /***/
    Zombie = 5,
    /// Dead
    Dead = 6,
}

/// tuneDegreeFlag
pub mod sched_flags {
    pub const ON_RQ: u32 = 1 << 0; // inrunQueue
    pub const MIGRATING: u32 = 1 << 1; // positiveinMigration
    pub const YIELDING: u32 = 1 << 2; // positiveinletexit
    pub const AFFINITY: u32 = 1 << 3; // finiteAffinity
    pub const RT: u32 = 1 << 4; // realtimeProcess
    pub const IDLE: u32 = 1 << 5; // emptyidleProcess
}

/// tuneDegreerealVolume
pub struct SchedEntity {
    /// imaginarysimulatedrunTime
    pub vruntime: AtomicU64,
    /// realactualrunTime
    pub runtime: AtomicU64,
    /// WaitTime
    pub wait_time: AtomicU64,
    /// Timeslice
    pub time_slice: AtomicU32,
    /// Priority
    pub prio: AtomicU32,
    /// StaticPriority
    pub static_prio: Priority,
    /// NormalPriority
    pub normal_prio: Priority,
    /// tuneDegreepolicy
    pub policy: AtomicU32,
    /// Flag
    pub flags: AtomicU32,
    /// CPU AffinityMask
    pub cpus_allowed: AtomicU64,
    /// Current CPU
    pub cpu: AtomicU32,
    /// mostthenrunTime
    pub last_ran: AtomicU64,
    /// Switch count
    pub switches: AtomicU64,
}

impl SchedEntity {
    /// CreatetuneDegreerealVolume
    pub fn new(prio: Priority) -> Self {
        SchedEntity {
            vruntime: AtomicU64::new(0),
            runtime: AtomicU64::new(0),
            wait_time: AtomicU64::new(0),
            time_slice: AtomicU32::new(0),
            prio: AtomicU32::new(prio as u32),
            static_prio: prio,
            normal_prio: prio,
            policy: AtomicU32::new(SchedPolicy::Normal as u32),
            flags: AtomicU32::new(0),
            cpus_allowed: AtomicU64::new(0xFFFFFFFFFFFFFFFF), // all CPU
            cpu: AtomicU32::new(0),
            last_ran: AtomicU64::new(0),
            switches: AtomicU64::new(0),
        }
    }

    /// GetPriority
    pub fn get_prio(&self) -> Priority {
        self.prio.load(Ordering::Acquire) as Priority
    }

    /// SetPriority
    pub fn set_prio(&self, prio: Priority) {
        self.prio.store(prio as u32, Ordering::Release);
    }

    /// GettuneDegreepolicy
    pub fn get_policy(&self) -> SchedPolicy {
        match self.policy.load(Ordering::Acquire) {
            0 => SchedPolicy::Normal,
            1 => SchedPolicy::Fifo,
            2 => SchedPolicy::RoundRobin,
            3 => SchedPolicy::Batch,
            4 => SchedPolicy::Idle,
            5 => SchedPolicy::Deadline,
            _ => SchedPolicy::Normal,
        }
    }

    /// SettuneDegreepolicy
    pub fn set_policy(&self, policy: SchedPolicy) {
        self.policy.store(policy as u32, Ordering::Release);
    }

    /// ifisrealtimeProcess
    pub fn is_rt(&self) -> bool {
        (self.flags.load(Ordering::Acquire) & sched_flags::RT) != 0
    }

    /// ifinrunQueue
    pub fn on_rq(&self) -> bool {
        (self.flags.load(Ordering::Acquire) & sched_flags::ON_RQ) != 0
    }

    /// SetinrunQueue
    pub fn set_on_rq(&self) {
        self.flags.fetch_or(sched_flags::ON_RQ, Ordering::AcqRel);
    }

    /// clearDivideinrunQueue
    pub fn clear_on_rq(&self) {
        self.flags.fetch_and(!sched_flags::ON_RQ, Ordering::AcqRel);
    }

    /// UpdateimaginarysimulatedrunTime
    pub fn update_vruntime(&self, delta: u64) {
        // RootevidencePrioritytuneintegerWeight
        let weight = self.get_weight();
        let vdelta = (delta * 1024) / weight as u64;
        self.vruntime.fetch_add(vdelta, Ordering::AcqRel);
    }

    /// GetWeight
    pub fn get_weight(&self) -> u32 {
        // PrioritytoWeightMap
        let prio = self.get_prio();
        match prio {
            -20..=-10 => 88761, // highPriority
            -9..=0 => 29538,
            1..=10 => 1024, // Default
            11..=19 => 110, // lowPriority
            _ => 1024,
        }
    }

    /// increasePlusSwitch count
    pub fn add_switch(&self) {
        self.switches.fetch_add(1, Ordering::AcqRel);
    }
}

/// runQueue
pub struct RunQueue {
    /// CPU ID
    pub cpu: CpuId,
    /// runQueueLock
    pub lock: AtomicU32,
    /// Processcount
    pub nr_running: AtomicU32,
    /// realtimeProcesscount
    pub rt_nr_running: AtomicU32,
    /// MinimaginarysimulatedrunTime
    pub min_vruntime: AtomicU64,
    /// Current process
    pub curr: u64,
    /// emptyidleProcess
    pub idle: u64,
    /// Clock
    pub clock: AtomicU64,
    /// load
    pub load: AtomicU64,
    /// Switch count
    pub switches: AtomicU64,
}

impl RunQueue {
    /// CreaterunQueue
    pub fn new(cpu: CpuId) -> Self {
        RunQueue {
            cpu,
            lock: AtomicU32::new(0),
            nr_running: AtomicU32::new(0),
            rt_nr_running: AtomicU32::new(0),
            min_vruntime: AtomicU64::new(0),
            curr: 0,
            idle: 0,
            clock: AtomicU64::new(0),
            load: AtomicU64::new(0),
            switches: AtomicU64::new(0),
        }
    }

    /// GetLock
    pub fn acquire(&self) {
        while self
            .lock
            .compare_exchange_weak(0, 1, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            core::hint::spin_loop();
        }
    }

    /// FreeLock
    pub fn release(&self) {
        self.lock.store(0, Ordering::Release);
    }

    /// UpdateClock
    pub fn update_clock(&self, now: u64) {
        self.clock.store(now, Ordering::Release);
    }

    /// GetClock
    pub fn get_clock(&self) -> u64 {
        self.clock.load(Ordering::Acquire)
    }

    /// increasePlusProcess
    pub fn inc_nr_running(&self) {
        self.nr_running.fetch_add(1, Ordering::AcqRel);
    }

    /// MinusfewProcess
    pub fn dec_nr_running(&self) {
        self.nr_running.fetch_sub(1, Ordering::AcqRel);
    }

    /// iffiniteemptyidleProcess
    pub fn has_idle(&self) -> bool {
        self.nr_running.load(Ordering::Acquire) == 0
    }
}

/// tuneDegreedevice
pub struct Scheduler {
    /// CPU count
    pub nr_cpus: u32,
    /// runQueue
    pub run_queues: [RunQueue; 8],
    /// totalProcessnumber
    pub nr_running: AtomicU32,
    /// totalSwitch count
    pub nr_switches: AtomicU64,
    /// tuneDegreetimenumber
    pub sched_count: AtomicU64,
    /// Load BalancingInterval
    pub lb_interval: AtomicU64,
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
            nr_running: AtomicU32::new(0),
            nr_switches: AtomicU64::new(0),
            sched_count: AtomicU64::new(0),
            lb_interval: AtomicU64::new(1000000), // 1ms
        }
    }

    /// Initialize
    pub fn init(&mut self, nr_cpus: u32) {
        self.nr_cpus = nr_cpus.min(8);

        log_info!("Scheduler initialized");
        log_info!(" CPUs: {}", self.nr_cpus);
    }

    /// GetrunQueue
    pub fn get_rq(&self, cpu: CpuId) -> &RunQueue {
        &self.run_queues[cpu as usize]
    }

    /// GetrunQueue (canchange)
    pub fn get_rq_mut(&mut self, cpu: CpuId) -> &mut RunQueue {
        &mut self.run_queues[cpu as usize]
    }

    /// tuneDegree
    pub fn schedule(&mut self, cpu: CpuId) {
        let rq = self.get_rq_mut(cpu);
        rq.acquire();

        self.sched_count.fetch_add(1, Ordering::AcqRel);

        // TODO: selectchooseNext process

        rq.release();
    }

    /// addProcess
    pub fn add_task(&mut self, _se: &mut SchedEntity, cpu: CpuId) {
        let rq = self.get_rq_mut(cpu);
        rq.acquire();

        rq.inc_nr_running();
        self.nr_running.fetch_add(1, Ordering::AcqRel);

        rq.release();
    }

    /// removeProcess
    pub fn remove_task(&mut self, _se: &mut SchedEntity, cpu: CpuId) {
        let rq = self.get_rq_mut(cpu);
        rq.acquire();

        rq.dec_nr_running();
        self.nr_running.fetch_sub(1, Ordering::AcqRel);

        rq.release();
    }

    /// GettotalProcessnumber
    pub fn get_nr_running(&self) -> u32 {
        self.nr_running.load(Ordering::Acquire)
    }

    /// GettuneDegreetimenumber
    pub fn get_sched_count(&self) -> u64 {
        self.sched_count.load(Ordering::Acquire)
    }
}

/** Per-CPU run queue with cache-line alignment.
 *
 * Each CPU maintains its own run queue to avoid lock
 * contention and cache-line bouncing in multi-core systems.
 * The 128-byte alignment ensures each queue occupies its
 * own cache line(s), preventing false sharing.
 */
#[repr(C, align(128))]
pub struct PerCpuRunQueue {
    /** CPU identifier for this run queue */
    pub cpu_id: CpuId,
    /** Number of runnable tasks on this CPU */
    pub nr_running: AtomicU32,
    /** Load average for this CPU (fixed-point 10.22 format) */
    pub load_avg: AtomicU64,
}

impl PerCpuRunQueue {
    /** Create a new per-CPU run queue for the given CPU */
    pub const fn new(cpu_id: CpuId) -> Self {
        PerCpuRunQueue {
            cpu_id,
            nr_running: AtomicU32::new(0),
            load_avg: AtomicU64::new(0),
        }
    }

    /** Increment the runnable task count */
    pub fn inc_nr_running(&self) {
        self.nr_running.fetch_add(1, Ordering::AcqRel);
    }

    /** Decrement the runnable task count */
    pub fn dec_nr_running(&self) {
        self.nr_running.fetch_sub(1, Ordering::AcqRel);
    }

    /** Get the current load average */
    pub fn load_avg(&self) -> u64 {
        self.load_avg.load(Ordering::Acquire)
    }

    /** Update the load average with a new sample */
    pub fn update_load_avg(&self, delta: u64) {
        self.load_avg.fetch_add(delta, Ordering::AcqRel);
    }
}

/** Maximum number of CPUs supported by the per-CPU run queue array */
pub const MAX_CPUS: usize = 256;

/** Global per-CPU run queue array.
 *
 * Each CPU indexes into this array using its CPU ID.
 * Entries beyond nr_cpus are unused.
 */
static PER_CPU_RQ: [core::sync::atomic::AtomicPtr<PerCpuRunQueue>; MAX_CPUS] = {
    const NULL: core::sync::atomic::AtomicPtr<PerCpuRunQueue> =
        core::sync::atomic::AtomicPtr::new(core::ptr::null_mut());
    [NULL; MAX_CPUS]
};

/** Get the per-CPU run queue for the current CPU.
 *
 * Returns a pointer to the PerCpuRunQueue for the
 * calling CPU. The caller must ensure the scheduler
 * has been initialized before calling this function.
 */
pub fn this_cpu_rq(cpu_id: CpuId) -> Option<&'static PerCpuRunQueue> {
    if (cpu_id as usize) >= MAX_CPUS {
        return None;
    }
    let ptr = PER_CPU_RQ[cpu_id as usize].load(Ordering::Acquire);
    if ptr.is_null() {
        return None;
    }
    // SAFETY: The pointer was set during scheduler initialization
    // and points to a properly allocated PerCpuRunQueue that lives
    // for the lifetime of the kernel. The atomic load with Acquire
    // ordering ensures we see the fully initialized structure.
    Some(unsafe { &*ptr })
}

/// Global scheduler instance
static SCHEDULER: core::sync::OnceLock<Scheduler> = core::sync::OnceLock::new();

pub fn scheduler() -> &'static Scheduler {
    SCHEDULER.get_or_init(Scheduler::new)
}

pub fn init_scheduler() {
    let sched = get_scheduler();
    sched.init(1);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sched_policy_values() {
        assert_eq!(SchedPolicy::Normal as u32, 0);
        assert_eq!(SchedPolicy::Fifo as u32, 1);
        assert_eq!(SchedPolicy::RoundRobin as u32, 2);
        assert_eq!(SchedPolicy::Batch as u32, 3);
        assert_eq!(SchedPolicy::Idle as u32, 4);
        assert_eq!(SchedPolicy::Deadline as u32, 5);
    }

    #[test]
    fn test_task_state_values() {
        assert_eq!(TaskState::Running as u32, 0);
        assert_eq!(TaskState::Interruptible as u32, 1);
        assert_eq!(TaskState::Uninterruptible as u32, 2);
        assert_eq!(TaskState::Stopped as u32, 3);
        assert_eq!(TaskState::Traced as u32, 4);
        assert_eq!(TaskState::Zombie as u32, 5);
        assert_eq!(TaskState::Dead as u32, 6);
    }

    #[test]
    fn test_sched_flags() {
        assert_eq!(sched_flags::ON_RQ, 1 << 0);
        assert_eq!(sched_flags::MIGRATING, 1 << 1);
        assert_eq!(sched_flags::YIELDING, 1 << 2);
        assert_eq!(sched_flags::AFFINITY, 1 << 3);
        assert_eq!(sched_flags::RT, 1 << 4);
        assert_eq!(sched_flags::IDLE, 1 << 5);
    }

    #[test]
    fn test_sched_entity_new() {
        let se = SchedEntity::new(0);

        assert_eq!(se.get_prio(), 0);
        assert_eq!(se.static_prio, 0);
        assert_eq!(se.get_policy(), SchedPolicy::Normal);
        assert!(!se.is_rt());
        assert!(!se.on_rq());
    }

    #[test]
    fn test_sched_entity_priority() {
        let se = SchedEntity::new(10);

        assert_eq!(se.get_prio(), 10);

        se.set_prio(-5);
        assert_eq!(se.get_prio(), -5);
    }

    #[test]
    fn test_sched_entity_policy() {
        let se = SchedEntity::new(0);

        se.set_policy(SchedPolicy::Fifo);
        assert_eq!(se.get_policy(), SchedPolicy::Fifo);

        se.set_policy(SchedPolicy::RoundRobin);
        assert_eq!(se.get_policy(), SchedPolicy::RoundRobin);
    }

    #[test]
    fn test_sched_entity_on_rq() {
        let se = SchedEntity::new(0);

        assert!(!se.on_rq());

        se.set_on_rq();
        assert!(se.on_rq());

        se.clear_on_rq();
        assert!(!se.on_rq());
    }

    #[test]
    fn test_sched_entity_weight() {
        // highPriority
        let se_high = SchedEntity::new(-20);
        assert_eq!(se_high.get_weight(), 88761);

        // DefaultPriority
        let se_default = SchedEntity::new(5);
        assert_eq!(se_default.get_weight(), 1024);

        // lowPriority
        let se_low = SchedEntity::new(15);
        assert_eq!(se_low.get_weight(), 110);
    }

    #[test]
    fn test_sched_entity_vruntime() {
        let se = SchedEntity::new(0);

        assert_eq!(se.vruntime.load(Ordering::Relaxed), 0);

        se.update_vruntime(1000);
        assert!(se.vruntime.load(Ordering::Relaxed) > 0);
    }

    #[test]
    fn test_sched_entity_switches() {
        let se = SchedEntity::new(0);

        assert_eq!(se.switches.load(Ordering::Relaxed), 0);

        se.add_switch();
        se.add_switch();
        se.add_switch();

        assert_eq!(se.switches.load(Ordering::Relaxed), 3);
    }

    #[test]
    fn test_run_queue_new() {
        let rq = RunQueue::new(0);

        assert_eq!(rq.cpu, 0);
        assert_eq!(rq.nr_running.load(Ordering::Relaxed), 0);
        assert_eq!(rq.rt_nr_running.load(Ordering::Relaxed), 0);
        assert!(rq.has_idle());
    }

    #[test]
    fn test_run_queue_lock() {
        let rq = RunQueue::new(0);

        rq.acquire();
        assert_eq!(rq.lock.load(Ordering::Relaxed), 1);

        rq.release();
        assert_eq!(rq.lock.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_run_queue_nr_running() {
        let rq = RunQueue::new(0);

        rq.inc_nr_running();
        assert_eq!(rq.nr_running.load(Ordering::Relaxed), 1);
        assert!(!rq.has_idle());

        rq.inc_nr_running();
        assert_eq!(rq.nr_running.load(Ordering::Relaxed), 2);

        rq.dec_nr_running();
        assert_eq!(rq.nr_running.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_run_queue_clock() {
        let rq = RunQueue::new(0);

        rq.update_clock(12345);
        assert_eq!(rq.get_clock(), 12345);
    }

    #[test]
    fn test_scheduler_new() {
        let sched = Scheduler::new();

        assert_eq!(sched.nr_cpus, 1);
        assert_eq!(sched.get_nr_running(), 0);
        assert_eq!(sched.get_sched_count(), 0);
    }

    #[test]
    fn test_scheduler_init() {
        let mut sched = Scheduler::new();
        sched.init(4);

        assert_eq!(sched.nr_cpus, 4);
    }

    #[test]
    fn test_scheduler_init_max_cpus() {
        let mut sched = Scheduler::new();
        sched.init(16); // exceedoverMaxvalue

        assert_eq!(sched.nr_cpus, 8); // Limitas 8
    }

    #[test]
    fn test_scheduler_get_rq() {
        let sched = Scheduler::new();

        let rq0 = sched.get_rq(0);
        assert_eq!(rq0.cpu, 0);

        let rq3 = sched.get_rq(3);
        assert_eq!(rq3.cpu, 3);
    }

    #[test]
    fn test_scheduler_add_remove_task() {
        let mut sched = Scheduler::new();
        let mut se = SchedEntity::new(0);

        assert_eq!(sched.get_nr_running(), 0);

        sched.add_task(&mut se, 0);
        assert_eq!(sched.get_nr_running(), 1);

        sched.remove_task(&mut se, 0);
        assert_eq!(sched.get_nr_running(), 0);
    }

    #[test]
    fn test_scheduler_schedule() {
        let mut sched = Scheduler::new();

        assert_eq!(sched.get_sched_count(), 0);

        sched.schedule(0);
        assert_eq!(sched.get_sched_count(), 1);
    }

    #[test]
    fn test_type_aliases() {
        let pid: Pid = 1234;
        let tid: Tid = 5678;
        let cpu: CpuId = 2;
        let prio: Priority = -5;

        assert_eq!(pid, 1234u32);
        assert_eq!(tid, 5678u32);
        assert_eq!(cpu, 2u32);
        assert_eq!(prio, -5i32);
    }
}
