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


use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

use crate::syslib::posix::errno::Errno;
use crate::kernel::error::Errno;
/// Process ID Type
pub type Pid = u32;

/// Thread ID Type
pub type Tid = u32;

/// User ID Type
pub type Uid = u32;

/// Group ID Type
pub type Gid = u32;

/// ProcessState
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
 /// ready
 Ready = 0,
 /// runinfix
 Running = 1,
 /***/
 Sleeping = 2,
 /// Stop
 Stopped = 3,
 /***/
 Zombie = 4,
 /// deadperish
 Dead = 5,
}

/// tuneDegreepolicy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchedPolicy {
 /// tuneDegree (CFS)
 Normal = 0,
 /// firstenterfirstexit (realtime)
 Fifo = 1,
 /// Timesliceroundbranch (realtime)
 RoundRobin = 2,
 /// Batch Processing
 Batch = 3,
 /// emptyidle
 Idle = 4,
 /// deadlineTime
 Deadline = 5,
}

/// ProcessPriority
pub const MAX_PRIO: i32 = 139;
pub const MIN_PRIO: i32 = 0;
pub const DEFAULT_PRIO: i32 = 120; // DefaultPriority
pub const RT_PRIO_BASE: i32 = 100; // realtimePrioritybaseaddress

/// ProcessFlag
pub mod task_flags {
 pub const PF_EXITING: u32 = 0x00000001; // positiveinExit
 pub const PF_EXITPIDONE: u32 = 0x00000002; // PID alreadyFree
 pub const PF_VCPU: u32 = 0x00000004; // imaginarysimulated CPU
 pub const PF_IDLE: u32 = 0x00000008; // emptyidleProcess
 pub const PF_KTHREAD: u32 = 0x00000010; // KernelThread
 pub const PF_WAKEUP_IDLE: u32 = 0x00000020; // Wakeemptyidle
 pub const PF_NO_SETAFFINITY: u32 = 0x00000040; // DisableSetAffinity
 pub const PF_MCE_PROCESS: u32 = 0x00000080; // MCE Handle
 pub const PF_USED_MATH: u32 = 0x00000100; // useMathematics
 pub const PF_USER_WORKER: u32 = 0x00000200; // UserworkmakeThread
}

/// CPU Context
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CpuContext {
 /// GeneralRegister x19-x28
 pub x19: u64,
 pub x20: u64,
 pub x21: u64,
 pub x22: u64,
 pub x23: u64,
 pub x24: u64,
 pub x25: u64,
 pub x26: u64,
 pub x27: u64,
 pub x28: u64,
 /// Framepointer
 pub fp: u64,
 /// linkacceptRegister
 pub sp: u64,
 /// Program counter
 pub pc: u64,
    pub regs: [u64; 31],
}

impl CpuContext {
 pub const fn new() -> Self {
 CpuContext {
 x19: 0, x20: 0, x21: 0, x22: 0, x23: 0,
 x24: 0, x25: 0, x26: 0, x27: 0, x28: 0,
 fp: 0, sp: 0, pc: 0,
 regs: [0; 31],
 }
 }
}

/// ProcessControlBlock
#[repr(C)]
pub struct TaskStruct {
 /// Process ID
 pub pid: Pid,
 /// Thread ID
 pub tid: Tid,
 /// ParentProcess ID
 pub ppid: Pid,
 /// User ID
 pub uid: Uid,
 /// Group ID
 pub gid: Gid,
 
 /// ProcessState
 pub state: AtomicU32,
 /// tuneDegreepolicy
 pub policy: SchedPolicy,
 /// StaticPriority
 pub static_prio: i32,
 /// DynamicPriority
 pub prio: i32,
 /// realtimePriority
 pub rt_priority: u32,
 
 /// ProcessFlag
 pub flags: AtomicU32,
 
 /// CPU Context
 pub cpu_context: CpuContext,
 
 /// KernelStackpointer
 pub kernel_stack: u64,
 /// UserStackpointer
 pub user_stack: u64,
 
 /// ProcessName
 pub name: [u8; 16],
 
 /// runTime (ns)
 pub runtime: AtomicU64,
 /// imaginarysimulatedrunTime (CFS)
 pub vruntime: AtomicU64,
 /// Timeslice
 pub time_slice: u64,
 
 /// CPU AffinityMask
 pub cpu_affinity: u64,
 /// Current CPU
 pub cpu: AtomicU32,
 
 /// mostthenrunTime
 pub last_run: AtomicU64,
 /// WaitTime
 pub wait_time: AtomicU64,
 
 /// ProcessAddress Space
 pub mm: u64, // *mut MmStruct
 
 /// Open FileDescriptorform
 pub files: u64, // *mut FilesStruct
 
 /// SignalHandle
 pub signal: u64, // *mut SignalStruct
 
 /// tuneDegreerealVolume (redblackTreeNode)
 pub rb_node: u64, // redblackTreeNode
 
 /// Next process (linkform)
 pub next: *mut TaskStruct,
 /// prefixaitemProcess
 pub prev: *mut TaskStruct,
}

impl TaskStruct {
 /// Create new ProcessControlBlock
 pub const fn new() -> Self {
 TaskStruct {
 pid: 0,
 tid: 0,
 ppid: 0,
 uid: 0,
 gid: 0,
 state: AtomicU32::new(TaskState::Ready as u32),
 policy: SchedPolicy::Normal,
 static_prio: DEFAULT_PRIO,
 prio: DEFAULT_PRIO,
 rt_priority: 0,
 flags: AtomicU32::new(0),
 cpu_context: CpuContext::new(),
 kernel_stack: 0,
 user_stack: 0,
 name: [0; 16],
 runtime: AtomicU64::new(0),
 vruntime: AtomicU64::new(0),
 time_slice: 0,
 cpu_affinity: 0xFF, // Defaultall CPU
 cpu: AtomicU32::new(0),
 last_run: AtomicU64::new(0),
 wait_time: AtomicU64::new(0),
 mm: 0,
 files: 0,
 signal: 0,
 rb_node: 0,
 next: core::ptr::null_mut(),
 prev: core::ptr::null_mut(),
 }
 }
 
 /// GetProcessState
 pub fn get_state(&self) -> TaskState {
 match self.state.load(Ordering::Acquire) {
 0 => TaskState::Ready,
 1 => TaskState::Running,
 2 => TaskState::Sleeping,
 3 => TaskState::Stopped,
 4 => TaskState::Zombie,
 5 => TaskState::Dead,
 _ => TaskState::Ready,
 }
 }
 
 /// SetProcessState
 pub fn set_state(&self, state: TaskState) {
 self.state.store(state as u32, Ordering::Release);
 }
 
 /// CheckifasKernelThread
 pub fn is_kernel_thread(&self) -> bool {
 (self.flags.load(Ordering::Acquire) & task_flags::PF_KTHREAD) != 0
 }
 
 /// Check if emptyidleProcess
 pub fn is_idle(&self) -> bool {
 (self.flags.load(Ordering::Acquire) & task_flags::PF_IDLE) != 0
 }
 
 /// CheckifpositiveinExit
 pub fn is_exiting(&self) -> bool {
 (self.flags.load(Ordering::Acquire) & task_flags::PF_EXITING) != 0
 }
 
 /// SetProcessName
 pub fn set_name(&mut self, name: &str) {
 let bytes = name.as_bytes();
 let len = bytes.len().min(self.name.len());
 self.name[..len].copy_from_slice(&bytes[..len]);
 if len < self.name.len() {
 self.name[len] = 0;
 }
 }
 
 /// GetProcessName
 pub fn get_name(&self) -> &str {
 let len = self.name.iter().position(|&c| c == 0).unwrap_or(self.name.len());
 // SAFETY: unsafe block required for low-level memory or hardware access
 unsafe { core::str::from_utf8_unchecked(&self.name[..len]) }
 }
 
 /// UpdaterunTime
 pub fn update_runtime(&self, delta: u64) {
 self.runtime.fetch_add(delta, Ordering::Relaxed);
 }
 
 /// UpdateimaginarysimulatedrunTime
 pub fn update_vruntime(&self, delta: u64) {
 self.vruntime.fetch_add(delta, Ordering::Relaxed);
 }
 
 /// GetCurrent CPU
 pub fn get_cpu(&self) -> u32 {
 self.cpu.load(Ordering::Acquire)
 }
 
 /// SetCurrent CPU
 pub fn set_cpu(&self, cpu: u32) {
 self.cpu.store(cpu, Ordering::Release);
 }

 /// Check if process is zombie
 pub fn is_zombie(&self) -> bool {
 self.get_state() == TaskState::Zombie
 }

 /// Reap zombie process resources
 /// @return exit code of the reaped process
 pub fn reap_zombie(&self) -> i32 {
 if !self.is_zombie() {
 return Errno::Eperm.to_ret_i32();
 }
 let exit_code = self.flags.load(Ordering::Acquire) as i32;
 self.set_state(TaskState::Dead);
 free_pid(self.pid);
 exit_code
 }

 /// Process exit handler
 /// @param exit_code: exit status code
 pub fn do_exit(&self, exit_code: i32) {
 self.flags.fetch_or(task_flags::PF_EXITING, Ordering::AcqRel);
 self.flags.store(exit_code as u32, Ordering::Release);
 self.set_state(TaskState::Zombie);
 }

 /// Get exit code
 pub fn get_exit_code(&self) -> i32 {
 self.flags.load(Ordering::Acquire) as i32
 }
}

/// Current processpointer (Per CPU Variable)
static mut CURRENT_TASK: *mut TaskStruct = core::ptr::null_mut();

/// GetCurrent process
pub fn current() -> Option<&'static mut TaskStruct> {
 // SAFETY: unsafe block required for low-level memory or hardware access
 unsafe {
 if CURRENT_TASK.is_null() {
 None
 } else {
 Some(&mut *CURRENT_TASK)
 }
 }
}

/// SetCurrent process
pub fn set_current(task: *mut TaskStruct) {
 // SAFETY: unsafe block required for low-level memory or hardware access
 unsafe {
 CURRENT_TASK = task;
 }
}

/// Process ID Allocatedevice
static NEXT_PID: AtomicU32 = AtomicU32::new(1);

/// PID pool for recycled PIDs
static PID_POOL: [AtomicU32; 64] = [
    AtomicU32::new(0), AtomicU32::new(0), AtomicU32::new(0), AtomicU32::new(0),
    AtomicU32::new(0), AtomicU32::new(0), AtomicU32::new(0), AtomicU32::new(0),
    AtomicU32::new(0), AtomicU32::new(0), AtomicU32::new(0), AtomicU32::new(0),
    AtomicU32::new(0), AtomicU32::new(0), AtomicU32::new(0), AtomicU32::new(0),
    AtomicU32::new(0), AtomicU32::new(0), AtomicU32::new(0), AtomicU32::new(0),
    AtomicU32::new(0), AtomicU32::new(0), AtomicU32::new(0), AtomicU32::new(0),
    AtomicU32::new(0), AtomicU32::new(0), AtomicU32::new(0), AtomicU32::new(0),
    AtomicU32::new(0), AtomicU32::new(0), AtomicU32::new(0), AtomicU32::new(0),
    AtomicU32::new(0), AtomicU32::new(0), AtomicU32::new(0), AtomicU32::new(0),
    AtomicU32::new(0), AtomicU32::new(0), AtomicU32::new(0), AtomicU32::new(0),
    AtomicU32::new(0), AtomicU32::new(0), AtomicU32::new(0), AtomicU32::new(0),
    AtomicU32::new(0), AtomicU32::new(0), AtomicU32::new(0), AtomicU32::new(0),
    AtomicU32::new(0), AtomicU32::new(0), AtomicU32::new(0), AtomicU32::new(0),
    AtomicU32::new(0), AtomicU32::new(0), AtomicU32::new(0), AtomicU32::new(0),
    AtomicU32::new(0), AtomicU32::new(0), AtomicU32::new(0), AtomicU32::new(0),
    AtomicU32::new(0), AtomicU32::new(0), AtomicU32::new(0), AtomicU32::new(0),
];

/// PID pool count
static PID_POOL_COUNT: AtomicU32 = AtomicU32::new(0);

/// Allocate Process ID
pub fn alloc_pid() -> Pid {
    let count = PID_POOL_COUNT.load(Ordering::Acquire);
    if count > 0 {
        for slot in &PID_POOL {
            let pid = slot.load(Ordering::Acquire);
            if pid != 0 && slot.compare_exchange_weak(
                pid, 0, Ordering::AcqRel, Ordering::Relaxed
            ).is_ok() {
                PID_POOL_COUNT.fetch_sub(1, Ordering::AcqRel);
                return pid;
            }
        }
    }
    NEXT_PID.fetch_add(1, Ordering::Relaxed)
}

/// Free Process ID (recycle to pool)
pub fn free_pid(pid: Pid) {
    if pid == 0 {
        return;
    }
    for slot in &PID_POOL {
        if slot.compare_exchange_weak(
            0, pid, Ordering::AcqRel, Ordering::Relaxed
        ).is_ok() {
            PID_POOL_COUNT.fetch_add(1, Ordering::AcqRel);
            return;
        }
    }
}

#[cfg(test)]
mod tests {
 use super::*;
 
 #[test]
 fn test_task_struct() {
 let mut task = TaskStruct::new();
 task.pid = alloc_pid();
 task.set_name("test");
 
 assert_eq!(task.get_state(), TaskState::Ready);
 assert_eq!(task.get_name(), "test");
 assert!(!task.is_kernel_thread());
 }
}