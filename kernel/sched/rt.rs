/*
 * Nuva OS - Kernel - Sched
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


use crate::kernel::sched::task::{TaskStruct, TaskState, SchedPolicy};

/// realtimePriorityRange: 0-99
pub const MAX_RT_PRIO: u32 = 100;

/// realtimerunQueue
pub struct RtRq {
 /// PeritemPriority Processlinkform
 pub active: [u64; MAX_RT_PRIO as usize], // linkformHead
 /// CurrentmosthighPriority
 pub highest_prio: u32,
 /// runProcessnumber
 pub rt_nr_running: u64,
}

impl RtRq {
 pub const fn new() -> Self {
 RtRq {
 active: [0; MAX_RT_PRIO as usize],
 highest_prio: 0,
 rt_nr_running: 0,
 }
 }
 
 /// willProcessPlusenterQueue
 pub fn enqueue(&mut self, task: &mut TaskStruct) {
 let prio = task.rt_priority as usize;
 
 // TODO: Insertlinkform
 self.active[prio] = task as *const TaskStruct as u64;
 
 // UpdatemosthighPriority
 if task.rt_priority > self.highest_prio {
 self.highest_prio = task.rt_priority;
 }
 
 self.rt_nr_running += 1;
 task.set_state(TaskState::Ready);
 }
 
 /// selectchooseNextrun Process
 pub fn pick_next(&self) -> Option<&'static mut TaskStruct> {
 // selectchoosemosthighPriority Process
 let prio = self.highest_prio as usize;
 
 if self.active[prio] == 0 {
 return None;
 }
 
 // SAFETY: unsafe block required for low-level memory or hardware access
 unsafe {
 Some(&mut *(self.active[prio] as *mut TaskStruct))
 }
 }
}

/// realtimetuneDegreedevice
pub struct RtScheduler {
 runqueues: [RtRq; 8],
}

impl RtScheduler {
 pub const fn new() -> Self {
 RtScheduler {
 runqueues: [
 RtRq::new(), RtRq::new(), RtRq::new(), RtRq::new(),
 RtRq::new(), RtRq::new(), RtRq::new(), RtRq::new(),
 ],
 }
 }
 
 pub fn get_rq(&mut self, cpu: u32) -> &mut RtRq {
 &mut self.runqueues[cpu as usize % 8]
 }
}

static RT_SCHEDULER: crate::sync_oncelock::OnceLock<RtScheduler> = crate::sync_oncelock::OnceLock::new();

pub fn rt_scheduler() -> &'static RtScheduler {
    RT_SCHEDULER.get_or_init(RtScheduler::new)
}

pub fn init_rt() {
 log_info!("RT scheduler initialized");
}

#[cfg(test)]
mod tests {
 use super::*;
 use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

 #[test]
 fn test_max_rt_prio() {
 assert_eq!(MAX_RT_PRIO, 100);
 }

 #[test]
 fn test_rt_rq_new() {
 let rq = RtRq::new();

 assert_eq!(rq.highest_prio, 0);
 assert_eq!(rq.rt_nr_running, 0);

 // placefinitePrioritylinkformshouldasempty
 for i in 0..MAX_RT_PRIO as usize {
 assert_eq!(rq.active[i], 0);
 }
 }

 #[test]
 fn test_rt_scheduler_new() {
 let sched = RtScheduler::new();

 // shouldfinite 8 itemrunQueue
 for i in 0..8 {
 assert_eq!(sched.runqueues[i].highest_prio, 0);
 assert_eq!(sched.runqueues[i].rt_nr_running, 0);
 }
 }

 #[test]
 fn test_rt_scheduler_get_rq() {
 let mut sched = RtScheduler::new();

 let rq0 = sched.get_rq(0);
 assert_eq!(rq0.highest_prio, 0);

 let rq3 = sched.get_rq(3);
 assert_eq!(rq3.highest_prio, 0);
 }

 #[test]
 fn test_rt_scheduler_get_rq_wrap() {
 let mut sched = RtScheduler::new();

 // exceedover 8 shouldtheround
 let rq8 = sched.get_rq(8);
 assert_eq!(rq8.highest_prio, 0);

 let rq10 = sched.get_rq(10);
 assert_eq!(rq10.highest_prio, 0);
 }

 #[test]
 fn test_rt_rq_enqueue() {
 let mut rq = RtRq::new();
 let mut task = TaskStruct {
 state: AtomicU32::new(TaskState::Running as u32),
 pid: 1,
 tgid: 1,
 ppid: 0,
 prio: 50,
 static_prio: 50,
 normal_prio: 50,
 rt_priority: 50,
 policy: AtomicU32::new(SchedPolicy::Rr as u32),
 time_slice: AtomicU32::new(100),
 vruntime: AtomicU64::new(0),
 runtime: AtomicU64::new(0),
 };

 assert_eq!(rq.rt_nr_running, 0);

 rq.enqueue(&mut task);

 assert_eq!(rq.rt_nr_running, 1);
 assert_eq!(rq.highest_prio, 50);
 assert_eq!(task.state.load(Ordering::Relaxed), TaskState::Ready as u32);
 }

 #[test]
 fn test_rt_rq_enqueue_higher_prio() {
 let mut rq = RtRq::new();

 // firstenterqueuelowPriorityProcess
 let mut task1 = TaskStruct {
 state: AtomicU32::new(TaskState::Running as u32),
 pid: 1,
 tgid: 1,
 ppid: 0,
 prio: 30,
 static_prio: 30,
 normal_prio: 30,
 rt_priority: 30,
 policy: AtomicU32::new(SchedPolicy::Rr as u32),
 time_slice: AtomicU32::new(100),
 vruntime: AtomicU64::new(0),
 runtime: AtomicU64::new(0),
 };

 rq.enqueue(&mut task1);
 assert_eq!(rq.highest_prio, 30);

 // againenterqueuehighPriorityProcess
 let mut task2 = TaskStruct {
 state: AtomicU32::new(TaskState::Running as u32),
 pid: 2,
 tgid: 2,
 ppid: 0,
 prio: 80,
 static_prio: 80,
 normal_prio: 80,
 rt_priority: 80,
 policy: AtomicU32::new(SchedPolicy::Fifo as u32),
 time_slice: AtomicU32::new(100),
 vruntime: AtomicU64::new(0),
 runtime: AtomicU64::new(0),
 };

 rq.enqueue(&mut task2);
 assert_eq!(rq.highest_prio, 80);
 assert_eq!(rq.rt_nr_running, 2);
 }

 #[test]
 fn test_rt_rq_pick_next_empty() {
 let rq = RtRq::new();

 let next = rq.pick_next();
 assert!(next.is_none());
 }

 #[test]
 fn test_rt_priority_range() {
 // realtimePriorityRange 0-99
 for prio in 0..100 {
 let mut rq = RtRq::new();
 let mut task = TaskStruct {
 state: AtomicU32::new(TaskState::Running as u32),
 pid: prio,
 tgid: prio,
 ppid: 0,
 prio: prio,
 static_prio: prio,
 normal_prio: prio,
 rt_priority: prio,
 policy: AtomicU32::new(SchedPolicy::Rr as u32),
 time_slice: AtomicU32::new(100),
 vruntime: AtomicU64::new(0),
 runtime: AtomicU64::new(0),
 };

 rq.enqueue(&mut task);
 assert_eq!(rq.highest_prio, prio);
 }
 }
}