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
use bitflags::bitflags;

/// ThreadFunctionType
pub type ThreadFunc = fn(*mut u8);

/// Thread ID
pub type KernelThreadId = u32;

/// ThreadState
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreadState {
 /// Initialize
 Uninitialized = 0,
 /// ready
 Ready = 1,
 /// runinfix
 Running = 2,
 /***/
 Sleeping = 3,
 /// alreadyStop
 Stopped = 4,
 /***/
 Zombie = 5,
}

/// ThreadFlag
bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct ThreadFlags: u32 {
        const KERNEL_THREAD = 1 << 0;
        const BOUND = 1 << 1;
        const PER_CPU = 1 << 2;
        const NOLOAD = 1 << 3;
        const STOPPED = 1 << 4;
        const SHOULD_STOP = 1 << 5;
        const SHOULD_PARK = 1 << 6;
        const PARKED = 1 << 7;
    }
}

/// Atomic wrapper for ThreadFlags
pub struct AtomicThreadFlags {
    inner: AtomicU32,
}

impl AtomicThreadFlags {
    pub const fn new(flags: ThreadFlags) -> Self {
        AtomicThreadFlags {
            inner: AtomicU32::new(flags.bits()),
        }
    }

    pub fn load(&self, ordering: Ordering) -> ThreadFlags {
        ThreadFlags::from_bits_truncate(self.inner.load(ordering))
    }

    pub fn store(&self, flags: ThreadFlags, ordering: Ordering) {
        self.inner.store(flags.bits(), ordering);
    }

    pub fn insert(&self, flag: ThreadFlags, ordering: Ordering) {
        self.inner.fetch_or(flag.bits(), ordering);
    }

    pub fn remove(&self, flag: ThreadFlags, ordering: Ordering) {
        self.inner.fetch_and(!flag.bits(), ordering);
    }

    pub fn contains(&self, flag: ThreadFlags, ordering: Ordering) -> bool {
        self.load(ordering).contains(flag)
    }

    pub fn fetch_or(&self, flag: ThreadFlags, ordering: Ordering) -> ThreadFlags {
        ThreadFlags::from_bits_truncate(self.inner.fetch_or(flag.bits(), ordering))
    }

    pub fn fetch_and(&self, flag: ThreadFlags, ordering: Ordering) -> ThreadFlags {
        ThreadFlags::from_bits_truncate(self.inner.fetch_and(flag.bits(), ordering))
    }
}

/// Legacy module for backward compatibility
pub mod thread_flags {
    use super::ThreadFlags;
    pub const KERNEL_THREAD: u32 = ThreadFlags::KERNEL_THREAD.bits();
    pub const BOUND: u32 = ThreadFlags::BOUND.bits();
    pub const PER_CPU: u32 = ThreadFlags::PER_CPU.bits();
    pub const NOLOAD: u32 = ThreadFlags::NOLOAD.bits();
    pub const STOPPED: u32 = ThreadFlags::STOPPED.bits();
    pub const SHOULD_STOP: u32 = ThreadFlags::SHOULD_STOP.bits();
    pub const SHOULD_PARK: u32 = ThreadFlags::SHOULD_PARK.bits();
    pub const PARKED: u32 = ThreadFlags::PARKED.bits();
}

/// KernelThread
pub struct KernelThread {
 /// Thread ID
 pub tid: KernelThreadId,
 /// ThreadFunction
 pub func: Option<ThreadFunc>,
 /// ThreadData
 pub data: *mut u8,
 /// Threadname
 pub name: [u8; 16],
 /// State
 pub state: AtomicU32,
 /// Flag
 pub flags: AtomicThreadFlags,
 /// CPU Affinity
 pub cpu: AtomicU32,
 /// Priority
 pub priority: AtomicU32,
 /// Stackpointer
 pub stack: u64,
 /// StackSize
 pub stack_size: u32,
 /// runTime
 pub runtime: AtomicU64,
 /// ContextSwitch count
 pub switches: AtomicU64,
 /// NextThread
 pub next: *mut KernelThread,
}

impl KernelThread {
 /// CreateKernelThread
 pub fn new(tid: KernelThreadId, func: ThreadFunc, data: *mut u8, name: &[u8]) -> Self {
 let mut thread = KernelThread {
 tid,
 func: Some(func),
 data,
 name: [0; 16],
 state: AtomicU32::new(ThreadState::Uninitialized as u32),
 flags: AtomicThreadFlags::new(ThreadFlags::KERNEL_THREAD),
 cpu: AtomicU32::new(0),
 priority: AtomicU32::new(100), // DefaultPriority
 stack: 0,
 stack_size: 0,
 runtime: AtomicU64::new(0),
 switches: AtomicU64::new(0),
 next: core::ptr::null_mut(),
 };
 
 let len = name.len().min(15);
 thread.name[..len].copy_from_slice(&name[..len]);
 
 thread
 }
 
 /// GetState
 pub fn get_state(&self) -> ThreadState {
 match self.state.load(Ordering::Acquire) {
 0 => ThreadState::Uninitialized,
 1 => ThreadState::Ready,
 2 => ThreadState::Running,
 3 => ThreadState::Sleeping,
 4 => ThreadState::Stopped,
 5 => ThreadState::Zombie,
 _ => ThreadState::Uninitialized,
 }
 }
 
 /// SetState
 pub fn set_state(&self, state: ThreadState) {
 self.state.store(state as u32, Ordering::Release);
 }
 
 /// ifshouldStop
 pub fn should_stop(&self) -> bool {
    self.flags.contains(ThreadFlags::SHOULD_STOP, Ordering::Acquire)
 }
 
 /// SetshouldStop
 pub fn set_should_stop(&self) {
    self.flags.insert(ThreadFlags::SHOULD_STOP, Ordering::AcqRel);
 }
 
 /// ifshouldSuspend
 pub fn should_park(&self) -> bool {
    self.flags.contains(ThreadFlags::SHOULD_PARK, Ordering::Acquire)
 }
 
 /// SetshouldSuspend
 pub fn set_should_park(&self) {
    self.flags.insert(ThreadFlags::SHOULD_PARK, Ordering::AcqRel);
 }
 
 /// ifalreadySuspend
 pub fn is_parked(&self) -> bool {
    self.flags.contains(ThreadFlags::PARKED, Ordering::Acquire)
 }
 
 /// SetalreadySuspend
 pub fn set_parked(&self) {
    self.flags.insert(ThreadFlags::PARKED, Ordering::AcqRel);
 }
 
 /// clearDivideSuspend
 pub fn clear_parked(&self) {
    self.flags.remove(ThreadFlags::PARKED, Ordering::AcqRel);
 }
 
 /// bind CPU
 pub fn bind_cpu(&self, cpu: u32) {
    self.cpu.store(cpu, Ordering::Release);
    self.flags.insert(ThreadFlags::BOUND, Ordering::AcqRel);
 }
 
 /// Get CPU
 pub fn get_cpu(&self) -> u32 {
 self.cpu.load(Ordering::Acquire)
 }
 
 /// SetPriority
 pub fn set_priority(&self, prio: u32) {
 self.priority.store(prio, Ordering::Release);
 }
 
 /// GetPriority
 pub fn get_priority(&self) -> u32 {
 self.priority.load(Ordering::Acquire)
 }
 
 /// increasePlusrunTime
 pub fn add_runtime(&self, ns: u64) {
 self.runtime.fetch_add(ns, Ordering::AcqRel);
 }
 
 /// GetrunTime
 pub fn get_runtime(&self) -> u64 {
 self.runtime.load(Ordering::Acquire)
 }
 
 /// increasePlusContextSwitch
 pub fn add_switch(&self) {
 self.switches.fetch_add(1, Ordering::AcqRel);
 }
 
 /// GetContextSwitch count
 pub fn get_switches(&self) -> u64 {
 self.switches.load(Ordering::Acquire)
 }
 
 /// StartThread
 pub fn start(&self) {
 self.set_state(ThreadState::Ready);
 }
 
 /// StopThread
 pub fn stop(&self) {
 self.set_should_stop();
 }
 
 /// SuspendThread
 pub fn park(&self) {
 self.set_should_park();
 }
 
 /// WakeThread
 pub fn unpark(&self) {
 self.clear_parked();
 }
}

/// KernelThreadManager
pub struct KernelThreadManager {
 /// ThreadlinkformHead
 pub thread_list: *mut KernelThread,
 /// Threadcount
 pub thread_count: AtomicU32,
 /// NextThread ID
 pub next_tid: AtomicU32,
 /// runinfixThreadnumber
 pub running_count: AtomicU32,
 /// Threadnumber
 pub sleeping_count: AtomicU32,
}

impl KernelThreadManager {
 pub const fn new() -> Self {
 KernelThreadManager {
 thread_list: core::ptr::null_mut(),
 thread_count: AtomicU32::new(0),
 next_tid: AtomicU32::new(1),
 running_count: AtomicU32::new(0),
 sleeping_count: AtomicU32::new(0),
 }
 }
 
 /// Initialize (no-op with OnceLock, initialization happens on first access)
 pub fn init(&self) {
 log_info!("Kernel thread manager initialized");
 }
 
 /// CreateKernelThread
 pub fn create_thread(&mut self, func: ThreadFunc, data: *mut u8, name: &[u8]) -> Option<KernelThreadId> {
 let tid = self.next_tid.fetch_add(1, Ordering::AcqRel);
 
 // TODO: AllocateThreadstructsumStack
 
 self.thread_count.fetch_add(1, Ordering::AcqRel);
 
 log_info!("Created kernel thread: {} (tid={})", 
 core::str::from_utf8(name).unwrap_or("?"), tid);
 
 Some(tid)
 }
 
 /// FindThread
 pub fn find_thread(&self, tid: KernelThreadId) -> Option<&KernelThread> {
 let mut current = self.thread_list;
 
 while !current.is_null() {
 // SAFETY: unsafe block required for low-level memory or hardware access
 unsafe {
 let thread = &*current;
 if thread.tid == tid {
 return Some(thread);
 }
 current = thread.next;
 }
 }
 
 None
 }
 
 /// StopThread
 pub fn stop_thread(&self, tid: KernelThreadId) -> bool {
 if let Some(thread) = self.find_thread(tid) {
 thread.stop();
 true
 } else {
 false
 }
 }
 
 /// SuspendThread
 pub fn park_thread(&self, tid: KernelThreadId) -> bool {
 if let Some(thread) = self.find_thread(tid) {
 thread.park();
 true
 } else {
 false
 }
 }
 
 /// WakeThread
 pub fn unpark_thread(&self, tid: KernelThreadId) -> bool {
 if let Some(thread) = self.find_thread(tid) {
 thread.unpark();
 true
 } else {
 false
 }
 }
 
 /// GetThreadcount
 pub fn get_thread_count(&self) -> u32 {
 self.thread_count.load(Ordering::Acquire)
 }
 
 /// GetruninfixThreadnumber
 pub fn get_running_count(&self) -> u32 {
 self.running_count.load(Ordering::Acquire)
 }
 
 /// GetThreadnumber
 pub fn get_sleeping_count(&self) -> u32 {
 self.sleeping_count.load(Ordering::Acquire)
 }
}

/// GlobalKernelThreadManager
static KERNEL_THREAD_REGISTRY: core::sync::OnceLock<KernelThreadManager> = core::sync::OnceLock::new();

pub fn kthread_manager() -> &'static KernelThreadManager {
    KERNEL_THREAD_REGISTRY.get_or_init(KernelThreadManager::new)
}

pub fn init_kthread_manager() -> &'static KernelThreadManager {
    KERNEL_THREAD_REGISTRY.get_or_init(KernelThreadManager::new)
}

pub fn init_kthread() {
 let mgr = init_kthread_manager();
 mgr.init();
}

#[cfg(test)]
mod tests {
 use super::*;

 fn test_thread_func(_data: *mut u8) {}

 #[test]
 fn test_thread_state_values() {
 assert_eq!(ThreadState::Uninitialized as u32, 0);
 assert_eq!(ThreadState::Ready as u32, 1);
 assert_eq!(ThreadState::Running as u32, 2);
 assert_eq!(ThreadState::Sleeping as u32, 3);
 assert_eq!(ThreadState::Stopped as u32, 4);
 assert_eq!(ThreadState::Zombie as u32, 5);
 }

 #[test]
 fn test_thread_flags() {
 assert_eq!(thread_flags::KERNEL_THREAD, 1 << 0);
 assert_eq!(thread_flags::BOUND, 1 << 1);
 assert_eq!(thread_flags::PER_CPU, 1 << 2);
 assert_eq!(thread_flags::NOLOAD, 1 << 3);
 assert_eq!(thread_flags::STOPPED, 1 << 4);
 assert_eq!(thread_flags::SHOULD_STOP, 1 << 5);
 assert_eq!(thread_flags::SHOULD_PARK, 1 << 6);
 assert_eq!(thread_flags::PARKED, 1 << 7);
 }

 #[test]
 fn test_kernel_thread_new() {
 let thread = KernelThread::new(1, test_thread_func, core::ptr::null_mut(), b"test");

 assert_eq!(thread.tid, 1);
 assert!(thread.func.is_some());
 assert_eq!(thread.get_state(), ThreadState::Uninitialized);
 assert_eq!(thread.get_priority(), 100);
 }

 #[test]
 fn test_kernel_thread_name() {
 let thread = KernelThread::new(1, test_thread_func, core::ptr::null_mut(), b"test_thread");

 assert_eq!(thread.name[0], b't');
 assert_eq!(thread.name[1], b'e');
 assert_eq!(thread.name[2], b's');
 assert_eq!(thread.name[3], b't');
 }

 #[test]
 fn test_kernel_thread_state_transitions() {
 let thread = KernelThread::new(1, test_thread_func, core::ptr::null_mut(), b"test");

 assert_eq!(thread.get_state(), ThreadState::Uninitialized);

 thread.set_state(ThreadState::Ready);
 assert_eq!(thread.get_state(), ThreadState::Ready);

 thread.set_state(ThreadState::Running);
 assert_eq!(thread.get_state(), ThreadState::Running);

 thread.set_state(ThreadState::Sleeping);
 assert_eq!(thread.get_state(), ThreadState::Sleeping);
 }

 #[test]
 fn test_kernel_thread_should_stop() {
 let thread = KernelThread::new(1, test_thread_func, core::ptr::null_mut(), b"test");

 assert!(!thread.should_stop());

 thread.set_should_stop();
 assert!(thread.should_stop());
 }

 #[test]
 fn test_kernel_thread_should_park() {
 let thread = KernelThread::new(1, test_thread_func, core::ptr::null_mut(), b"test");

 assert!(!thread.should_park());

 thread.set_should_park();
 assert!(thread.should_park());
 }

 #[test]
 fn test_kernel_thread_parked() {
 let thread = KernelThread::new(1, test_thread_func, core::ptr::null_mut(), b"test");

 assert!(!thread.is_parked());

 thread.set_parked();
 assert!(thread.is_parked());

 thread.clear_parked();
 assert!(!thread.is_parked());
 }

 #[test]
 fn test_kernel_thread_cpu_binding() {
 let thread = KernelThread::new(1, test_thread_func, core::ptr::null_mut(), b"test");

 assert_eq!(thread.get_cpu(), 0);

 thread.bind_cpu(2);
 assert_eq!(thread.get_cpu(), 2);
 assert!(thread.flags.contains(ThreadFlags::BOUND, Ordering::Relaxed));
 }

 #[test]
 fn test_kernel_thread_priority() {
 let thread = KernelThread::new(1, test_thread_func, core::ptr::null_mut(), b"test");

 assert_eq!(thread.get_priority(), 100);

 thread.set_priority(50);
 assert_eq!(thread.get_priority(), 50);

 thread.set_priority(150);
 assert_eq!(thread.get_priority(), 150);
 }

 #[test]
 fn test_kernel_thread_runtime() {
 let thread = KernelThread::new(1, test_thread_func, core::ptr::null_mut(), b"test");

 assert_eq!(thread.get_runtime(), 0);

 thread.add_runtime(1000);
 assert_eq!(thread.get_runtime(), 1000);

 thread.add_runtime(500);
 assert_eq!(thread.get_runtime(), 1500);
 }

 #[test]
 fn test_kernel_thread_switches() {
 let thread = KernelThread::new(1, test_thread_func, core::ptr::null_mut(), b"test");

 assert_eq!(thread.get_switches(), 0);

 thread.add_switch();
 thread.add_switch();
 thread.add_switch();

 assert_eq!(thread.get_switches(), 3);
 }

 #[test]
 fn test_kernel_thread_start_stop() {
 let thread = KernelThread::new(1, test_thread_func, core::ptr::null_mut(), b"test");

 thread.start();
 assert_eq!(thread.get_state(), ThreadState::Ready);

 thread.stop();
 assert!(thread.should_stop());
 }

 #[test]
 fn test_kernel_thread_park_unpark() {
 let thread = KernelThread::new(1, test_thread_func, core::ptr::null_mut(), b"test");

 thread.park();
 assert!(thread.should_park());

 thread.unpark();
 assert!(!thread.is_parked());
 }

 #[test]
 fn test_kernel_thread_manager_new() {
 let mgr = KernelThreadManager::new();

 assert_eq!(mgr.get_thread_count(), 0);
 assert_eq!(mgr.get_running_count(), 0);
 assert_eq!(mgr.get_sleeping_count(), 0);
 }

 #[test]
 fn test_kernel_thread_manager_create_thread() {
 let mut mgr = KernelThreadManager::new();

 let tid = mgr.create_thread(test_thread_func, core::ptr::null_mut(), b"test");

 assert!(tid.is_some());
 assert!(tid.unwrap() >= 1);
 assert_eq!(mgr.get_thread_count(), 1);
 }

 #[test]
 fn test_kernel_thread_manager_create_multiple_threads() {
 let mut mgr = KernelThreadManager::new();

 let tid1 = mgr.create_thread(test_thread_func, core::ptr::null_mut(), b"thread1");
 let tid2 = mgr.create_thread(test_thread_func, core::ptr::null_mut(), b"thread2");
 let tid3 = mgr.create_thread(test_thread_func, core::ptr::null_mut(), b"thread3");

 assert_ne!(tid1.unwrap(), tid2.unwrap());
 assert_ne!(tid2.unwrap(), tid3.unwrap());
 assert_eq!(mgr.get_thread_count(), 3);
 }

 #[test]
 fn test_kernel_thread_manager_find_thread_empty() {
 let mgr = KernelThreadManager::new();

 let result = mgr.find_thread(1);
 assert!(result.is_none());
 }

 #[test]
 fn test_kernel_thread_manager_stop_thread_not_found() {
 let mgr = KernelThreadManager::new();

 let result = mgr.stop_thread(999);
 assert!(!result);
 }

 #[test]
 fn test_kernel_thread_manager_park_thread_not_found() {
 let mgr = KernelThreadManager::new();

 let result = mgr.park_thread(999);
 assert!(!result);
 }

 #[test]
 fn test_kernel_thread_manager_unpark_thread_not_found() {
 let mgr = KernelThreadManager::new();

 let result = mgr.unpark_thread(999);
 assert!(!result);
 }

 #[test]
 fn test_thread_state_equality() {
 assert_eq!(ThreadState::Running, ThreadState::Running);
 assert_ne!(ThreadState::Running, ThreadState::Sleeping);
 assert_ne!(ThreadState::Ready, ThreadState::Zombie);
 }

 #[test]
 fn test_kthread_id_type() {
 let tid: KernelThreadId = 42;
 assert_eq!(tid, 42u32);
 }
}