/*
 * Nuva OS - Task Management
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
use super::user::{Uid, Gid};
use super::session::{SessionId, Pgid};

/// Process ID Type
pub type Pid = i32;

/// Thread ID Type
pub type Tid = u32;

/// MaxProcessnumber
pub const MAX_PROCESSES: usize = 4096;

/// MaxThreadnumber
pub const MAX_THREADS: usize = 16384;

/// ProcessState
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessState {
 /// Createinfix
 Creating = 0,
 /// ready
 Ready = 1,
 /// run
 Running = 2,
 /***/
 Sleeping = 3,
 /// Stop
 Stopped = 4,
 /***/
 Zombie = 5,
 /// alreadyTerminate
 Terminated = 6,
}

/// ProcessPriority
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ProcessPriority {
 /// realtime (mosthigh)
 RealTime = 0,
 /// high
 High = 1,
 /***/
 Normal = 2,
 /// low
 Low = 3,
 /// emptyidle (mostlow)
 Idle = 4,
}

/// ProcessFlag
pub mod process_flags {
 /// KernelProcess
 pub const KERNEL: u32 = 1 << 0;
 /// Process
 pub const DAEMON: u32 = 1 << 1;
 /// Sessionfirst
 pub const SESSION_LEADER: u32 = 1 << 2;
 /// ProcessGroupfirst
 pub const GROUP_LEADER: u32 = 1 << 3;
}

/// Processstruct
pub struct Process {
 /// Process ID
 pub pid: Pid,
 /// ParentProcess ID
 pub ppid: Pid,
 /// User ID
 pub uid: Uid,
 /// Group ID
 pub gid: Gid,
 /// validUser ID
 pub euid: Uid,
 /// validGroup ID
 pub egid: Gid,
 /// Session ID
 pub sid: SessionId,
 /// ProcessGroup ID
 pub pgid: Pgid,
 /// State
 pub state: AtomicU32,
 /// Priority
 pub priority: AtomicU32,
 /// Flag
 pub flags: AtomicU32,
 /// Threadnumber
 pub thread_count: AtomicU32,
 /// ChildProcessnumber
 pub child_count: AtomicU32,
 /// Exitcode
 pub exit_code: AtomicU32,
 /// CPU Time (User)
 pub utime: AtomicU64,
 /// CPU Time (Kernel)
 pub stime: AtomicU64,
 /// CreateTime
 pub start_time: AtomicU64,
}

impl Process {
 /// CreatenewProcess
 pub fn new(pid: Pid, ppid: Pid, uid: Uid, gid: Gid) -> Self {
 Process {
 pid,
 ppid,
 uid,
 gid,
 euid: uid,
 egid: gid,
 sid: 0,
 pgid: 0,
 state: AtomicU32::new(ProcessState::Creating as u32),
 priority: AtomicU32::new(ProcessPriority::Normal as u32),
 flags: AtomicU32::new(0),
 thread_count: AtomicU32::new(1),
 child_count: AtomicU32::new(0),
 exit_code: AtomicU32::new(0),
 utime: AtomicU64::new(0),
 stime: AtomicU64::new(0),
 start_time: AtomicU64::new(0),
 }
 }
 
 /// GetState
 pub fn get_state(&self) -> ProcessState {
 match self.state.load(Ordering::Acquire) {
 0 => ProcessState::Creating,
 1 => ProcessState::Ready,
 2 => ProcessState::Running,
 3 => ProcessState::Sleeping,
 4 => ProcessState::Stopped,
 5 => ProcessState::Zombie,
 6 => ProcessState::Terminated,
 _ => ProcessState::Creating,
 }
 }
 
 /// SetState
 pub fn set_state(&self, state: ProcessState) {
 self.state.store(state as u32, Ordering::Release);
 }
 
 /// GetPriority
 pub fn get_priority(&self) -> ProcessPriority {
 match self.priority.load(Ordering::Acquire) {
 0 => ProcessPriority::RealTime,
 1 => ProcessPriority::High,
 2 => ProcessPriority::Normal,
 3 => ProcessPriority::Low,
 4 => ProcessPriority::Idle,
 _ => ProcessPriority::Normal,
 }
 }
 
 /// SetPriority
 pub fn set_priority(&self, priority: ProcessPriority) {
 self.priority.store(priority as u32, Ordering::Release);
 }
 
 /// ifisKernelProcess
 pub fn is_kernel(&self) -> bool {
 (self.flags.load(Ordering::Acquire) & process_flags::KERNEL) != 0
 }
 
 /// ifisProcess
 pub fn is_daemon(&self) -> bool {
 (self.flags.load(Ordering::Acquire) & process_flags::DAEMON) != 0
 }
 
 /// ifisSessionfirst
 pub fn is_session_leader(&self) -> bool {
 (self.flags.load(Ordering::Acquire) & process_flags::SESSION_LEADER) != 0
 }
 
 /// SetSessionfirst
 pub fn set_session_leader(&self) {
 self.flags.fetch_or(process_flags::SESSION_LEADER, Ordering::AcqRel);
 }
 
 /// increasePlusThread
 pub fn add_thread(&self) {
 self.thread_count.fetch_add(1, Ordering::AcqRel);
 }
 
 /// MinusfewThread
 pub fn remove_thread(&self) {
 self.thread_count.fetch_sub(1, Ordering::AcqRel);
 }
 
 /// increasePlusChildProcess
 pub fn add_child(&self) {
 self.child_count.fetch_add(1, Ordering::AcqRel);
 }
 
 /// MinusfewChildProcess
 pub fn remove_child(&self) {
 self.child_count.fetch_sub(1, Ordering::AcqRel);
 }
 
 /// Exit
 pub fn exit(&self, code: i32) {
 self.exit_code.store(code as u32, Ordering::Release);
 self.set_state(ProcessState::Zombie);
 }
}

/// ThreadState
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreadState {
 /// ready
 Ready = 0,
 /// run
 Running = 1,
 /***/
 Sleeping = 2,
 /// Stop
 Stopped = 3,
 /// alreadyTerminate
 Terminated = 4,
}

/// Threadstruct
pub struct Thread {
 /// Thread ID
 pub tid: Tid,
 /// Process ID
 pub pid: Pid,
 /// State
 pub state: AtomicU32,
 /// StackAddress
 pub stack_addr: AtomicU64,
 /// StackSize
 pub stack_size: usize,
 /// enterportDot
 pub entry: AtomicU64,
}

impl Thread {
 /// CreatenewThread
 pub fn new(tid: Tid, pid: Pid) -> Self {
 Thread {
 tid,
 pid,
 state: AtomicU32::new(ThreadState::Ready as u32),
 stack_addr: AtomicU64::new(0),
 stack_size: 0,
 entry: AtomicU64::new(0),
 }
 }
 
 /// GetState
 pub fn get_state(&self) -> ThreadState {
 match self.state.load(Ordering::Acquire) {
 0 => ThreadState::Ready,
 1 => ThreadState::Running,
 2 => ThreadState::Sleeping,
 3 => ThreadState::Stopped,
 4 => ThreadState::Terminated,
 _ => ThreadState::Ready,
 }
 }
 
 /// SetState
 pub fn set_state(&self, state: ThreadState) {
 self.state.store(state as u32, Ordering::Release);
 }
}

/// Task Managementdevice
pub struct TaskManager {
 /// Current process ID
 current_pid: AtomicU32,
 /// CurrentThread ID
 current_tid: AtomicU32,
 /// Next process ID
 next_pid: AtomicU32,
 /// NextThread ID
 next_tid: AtomicU32,
 /// Processcount
 process_count: AtomicU32,
 /// Threadcount
 thread_count: AtomicU32,
}

impl TaskManager {
 pub const fn new() -> Self {
 TaskManager {
 current_pid: AtomicU32::new(0),
 current_tid: AtomicU32::new(0),
 next_pid: AtomicU32::new(1),
 next_tid: AtomicU32::new(1),
 process_count: AtomicU32::new(0),
 thread_count: AtomicU32::new(0),
 }
 }
 
 /// Initialize
 pub fn init(&self) {
 log_info!("Task manager initialized");
 log_info!(" Max processes: {}", MAX_PROCESSES);
 log_info!(" Max threads: {}", MAX_THREADS);
 }
 
 /// AllocateProcess ID
 pub fn alloc_pid(&self) -> Pid {
 self.next_pid.fetch_add(1, Ordering::AcqRel) as Pid
 }
 
 /// AllocateThread ID
 pub fn alloc_tid(&self) -> Tid {
 self.next_tid.fetch_add(1, Ordering::AcqRel)
 }
 
 /// GetCurrent process ID
 pub fn get_current_pid(&self) -> Pid {
 self.current_pid.load(Ordering::Acquire) as Pid
 }
 
 /// SetCurrent process ID
 pub fn set_current_pid(&self, pid: Pid) {
 self.current_pid.store(pid as u32, Ordering::Release);
 }
 
 /// GetCurrentThread ID
 pub fn get_current_tid(&self) -> Tid {
 self.current_tid.load(Ordering::Acquire)
 }
 
 /// SetCurrentThread ID
 pub fn set_current_tid(&self, tid: Tid) {
 self.current_tid.store(tid, Ordering::Release);
 }
 
 /// increasePlusProcessCount
 pub fn add_process(&self) {
 self.process_count.fetch_add(1, Ordering::AcqRel);
 }
 
 /// MinusfewProcessCount
 pub fn remove_process(&self) {
 self.process_count.fetch_sub(1, Ordering::AcqRel);
 }
 
 /// increasePlusThreadCount
 pub fn add_thread(&self) {
 self.thread_count.fetch_add(1, Ordering::AcqRel);
 }
 
 /// MinusfewThreadCount
 pub fn remove_thread(&self) {
 self.thread_count.fetch_sub(1, Ordering::AcqRel);
 }
 
 /// GetProcesscount
 pub fn get_process_count(&self) -> u32 {
 self.process_count.load(Ordering::Acquire)
 }
 
 /// GetThreadcount
 pub fn get_thread_count(&self) -> u32 {
 self.thread_count.load(Ordering::Acquire)
 }
}

/// GlobalTask Managementdevice
static TASK_MANAGER: crate::sync_oncelock::OnceLock<TaskManager> = crate::sync_oncelock::OnceLock::new();

pub fn task_manager() -> &'static TaskManager {
    TASK_MANAGER.get_or_init(TaskManager::new)
}

pub fn init_task_manager() {
 let manager = task_manager();
 manager.init();
}

#[cfg(test)]
mod tests {
 use super::*;

 #[test]
 fn test_constants() {
 assert_eq!(MAX_PROCESSES, 4096);
 assert_eq!(MAX_THREADS, 16384);
 }

 #[test]
 fn test_process_state_values() {
 assert_eq!(ProcessState::Creating as u32, 0);
 assert_eq!(ProcessState::Ready as u32, 1);
 assert_eq!(ProcessState::Running as u32, 2);
 assert_eq!(ProcessState::Sleeping as u32, 3);
 assert_eq!(ProcessState::Stopped as u32, 4);
 assert_eq!(ProcessState::Zombie as u32, 5);
 assert_eq!(ProcessState::Terminated as u32, 6);
 }

 #[test]
 fn test_process_priority_values() {
 assert_eq!(ProcessPriority::RealTime as u32, 0);
 assert_eq!(ProcessPriority::High as u32, 1);
 assert_eq!(ProcessPriority::Normal as u32, 2);
 assert_eq!(ProcessPriority::Low as u32, 3);
 assert_eq!(ProcessPriority::Idle as u32, 4);
 }

 #[test]
 fn test_process_priority_ordering() {
 assert!(ProcessPriority::RealTime < ProcessPriority::High);
 assert!(ProcessPriority::High < ProcessPriority::Normal);
 assert!(ProcessPriority::Normal < ProcessPriority::Low);
 assert!(ProcessPriority::Low < ProcessPriority::Idle);
 }

 #[test]
 fn test_process_flags() {
 assert_eq!(process_flags::KERNEL, 1 << 0);
 assert_eq!(process_flags::DAEMON, 1 << 1);
 assert_eq!(process_flags::SESSION_LEADER, 1 << 2);
 assert_eq!(process_flags::GROUP_LEADER, 1 << 3);
 }

 #[test]
 fn test_process_new() {
 let proc = Process::new(1, 0, 100, 100);

 assert_eq!(proc.pid, 1);
 assert_eq!(proc.ppid, 0);
 assert_eq!(proc.uid, 100);
 assert_eq!(proc.gid, 100);
 assert_eq!(proc.euid, 100);
 assert_eq!(proc.egid, 100);
 assert_eq!(proc.get_state(), ProcessState::Creating);
 assert_eq!(proc.get_priority(), ProcessPriority::Normal);
 }

 #[test]
 fn test_process_state_transitions() {
 let proc = Process::new(1, 0, 0, 0);

 assert_eq!(proc.get_state(), ProcessState::Creating);

 proc.set_state(ProcessState::Ready);
 assert_eq!(proc.get_state(), ProcessState::Ready);

 proc.set_state(ProcessState::Running);
 assert_eq!(proc.get_state(), ProcessState::Running);

 proc.set_state(ProcessState::Sleeping);
 assert_eq!(proc.get_state(), ProcessState::Sleeping);

 proc.set_state(ProcessState::Stopped);
 assert_eq!(proc.get_state(), ProcessState::Stopped);

 proc.set_state(ProcessState::Zombie);
 assert_eq!(proc.get_state(), ProcessState::Zombie);

 proc.set_state(ProcessState::Terminated);
 assert_eq!(proc.get_state(), ProcessState::Terminated);
 }

 #[test]
 fn test_process_priority() {
 let proc = Process::new(1, 0, 0, 0);

 assert_eq!(proc.get_priority(), ProcessPriority::Normal);

 proc.set_priority(ProcessPriority::RealTime);
 assert_eq!(proc.get_priority(), ProcessPriority::RealTime);

 proc.set_priority(ProcessPriority::High);
 assert_eq!(proc.get_priority(), ProcessPriority::High);

 proc.set_priority(ProcessPriority::Idle);
 assert_eq!(proc.get_priority(), ProcessPriority::Idle);
 }

 #[test]
 fn test_process_flags_check() {
 let proc = Process::new(1, 0, 0, 0);

 assert!(!proc.is_kernel());
 assert!(!proc.is_daemon());
 assert!(!proc.is_session_leader());

 proc.flags.fetch_or(process_flags::KERNEL, Ordering::Relaxed);
 assert!(proc.is_kernel());

 proc.flags.fetch_or(process_flags::DAEMON, Ordering::Relaxed);
 assert!(proc.is_daemon());

 proc.set_session_leader();
 assert!(proc.is_session_leader());
 }

 #[test]
 fn test_process_thread_count() {
 let proc = Process::new(1, 0, 0, 0);

 assert_eq!(proc.thread_count.load(Ordering::Relaxed), 1);

 proc.add_thread();
 assert_eq!(proc.thread_count.load(Ordering::Relaxed), 2);

 proc.add_thread();
 proc.add_thread();
 assert_eq!(proc.thread_count.load(Ordering::Relaxed), 4);

 proc.remove_thread();
 assert_eq!(proc.thread_count.load(Ordering::Relaxed), 3);
 }

 #[test]
 fn test_process_child_count() {
 let proc = Process::new(1, 0, 0, 0);

 assert_eq!(proc.child_count.load(Ordering::Relaxed), 0);

 proc.add_child();
 assert_eq!(proc.child_count.load(Ordering::Relaxed), 1);

 proc.add_child();
 assert_eq!(proc.child_count.load(Ordering::Relaxed), 2);

 proc.remove_child();
 assert_eq!(proc.child_count.load(Ordering::Relaxed), 1);
 }

 #[test]
 fn test_process_exit() {
 let proc = Process::new(1, 0, 0, 0);

 proc.set_state(ProcessState::Running);
 assert_eq!(proc.get_state(), ProcessState::Running);

 proc.exit(42);
 assert_eq!(proc.get_state(), ProcessState::Zombie);
 assert_eq!(proc.exit_code.load(Ordering::Relaxed), 42);
 }

 #[test]
 fn test_process_cpu_time() {
 let proc = Process::new(1, 0, 0, 0);

 assert_eq!(proc.utime.load(Ordering::Relaxed), 0);
 assert_eq!(proc.stime.load(Ordering::Relaxed), 0);

 proc.utime.fetch_add(1000, Ordering::Relaxed);
 proc.stime.fetch_add(500, Ordering::Relaxed);

 assert_eq!(proc.utime.load(Ordering::Relaxed), 1000);
 assert_eq!(proc.stime.load(Ordering::Relaxed), 500);
 }

 #[test]
 fn test_thread_state_values() {
 assert_eq!(ThreadState::Ready as u32, 0);
 assert_eq!(ThreadState::Running as u32, 1);
 assert_eq!(ThreadState::Sleeping as u32, 2);
 assert_eq!(ThreadState::Stopped as u32, 3);
 assert_eq!(ThreadState::Terminated as u32, 4);
 }

 #[test]
 fn test_thread_new() {
 let thread = Thread::new(1, 1);

 assert_eq!(thread.tid, 1);
 assert_eq!(thread.pid, 1);
 assert_eq!(thread.get_state(), ThreadState::Ready);
 assert_eq!(thread.stack_addr.load(Ordering::Relaxed), 0);
 assert_eq!(thread.stack_size, 0);
 }

 #[test]
 fn test_thread_state_transitions() {
 let thread = Thread::new(1, 1);

 assert_eq!(thread.get_state(), ThreadState::Ready);

 thread.set_state(ThreadState::Running);
 assert_eq!(thread.get_state(), ThreadState::Running);

 thread.set_state(ThreadState::Sleeping);
 assert_eq!(thread.get_state(), ThreadState::Sleeping);

 thread.set_state(ThreadState::Stopped);
 assert_eq!(thread.get_state(), ThreadState::Stopped);

 thread.set_state(ThreadState::Terminated);
 assert_eq!(thread.get_state(), ThreadState::Terminated);
 }

 #[test]
 fn test_thread_stack() {
 let mut thread = Thread::new(1, 1);

 thread.stack_addr.store(0x7FFF0000, Ordering::Relaxed);
 thread.stack_size = 8192;

 assert_eq!(thread.stack_addr.load(Ordering::Relaxed), 0x7FFF0000);
 assert_eq!(thread.stack_size, 8192);
 }

 #[test]
 fn test_task_manager_new() {
 let mgr = TaskManager::new();

 assert_eq!(mgr.get_current_pid(), 0);
 assert_eq!(mgr.get_current_tid(), 0);
 assert_eq!(mgr.get_process_count(), 0);
 assert_eq!(mgr.get_thread_count(), 0);
 }

 #[test]
 fn test_task_manager_alloc_pid() {
 let mgr = TaskManager::new();

 let pid1 = mgr.alloc_pid();
 assert_eq!(pid1, 1);

 let pid2 = mgr.alloc_pid();
 assert_eq!(pid2, 2);

 let pid3 = mgr.alloc_pid();
 assert_eq!(pid3, 3);
 }

 #[test]
 fn test_task_manager_alloc_tid() {
 let mgr = TaskManager::new();

 let tid1 = mgr.alloc_tid();
 assert_eq!(tid1, 1);

 let tid2 = mgr.alloc_tid();
 assert_eq!(tid2, 2);
 }

 #[test]
 fn test_task_manager_current() {
 let mgr = TaskManager::new();

 assert_eq!(mgr.get_current_pid(), 0);
 assert_eq!(mgr.get_current_tid(), 0);

 mgr.set_current_pid(100);
 mgr.set_current_tid(200);

 assert_eq!(mgr.get_current_pid(), 100);
 assert_eq!(mgr.get_current_tid(), 200);
 }

 #[test]
 fn test_task_manager_process_count() {
 let mgr = TaskManager::new();

 assert_eq!(mgr.get_process_count(), 0);

 mgr.add_process();
 assert_eq!(mgr.get_process_count(), 1);

 mgr.add_process();
 mgr.add_process();
 assert_eq!(mgr.get_process_count(), 3);

 mgr.remove_process();
 assert_eq!(mgr.get_process_count(), 2);
 }

 #[test]
 fn test_task_manager_thread_count() {
 let mgr = TaskManager::new();

 assert_eq!(mgr.get_thread_count(), 0);

 mgr.add_thread();
 assert_eq!(mgr.get_thread_count(), 1);

 mgr.add_thread();
 assert_eq!(mgr.get_thread_count(), 2);

 mgr.remove_thread();
 assert_eq!(mgr.get_thread_count(), 1);
 }
}