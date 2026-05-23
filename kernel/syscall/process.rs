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

use crate::posix::errno::Errno;
/// Process ID
pub type Pid = u32;

/// User ID
pub type Uid = u32;

/// Group ID
pub type Gid = u32;

/// ProcessFlag
pub mod process_flags {
 pub const PF_EXITING: u32 = 1 << 0; // positiveinExit
 pub const PF_EXITPIDONE: u32 = 1 << 1; // ExitComplete
 pub const PF_VCPU: u32 = 1 << 2; // imaginarysimulated CPU
 pub const PF_SUPERPRIV: u32 = 1 << 3; // exceedlevelPermission
 pub const PF_DUMPCORE: u32 = 1 << 4; // branchkernel
 pub const PF_SIGNALED: u32 = 1 << 5; // bySignalTerminate
 pub const PF_MEMALLOC: u32 = 1 << 6; // MemoryAllocate
 pub const PF_NPROC_EXCEEDED: u32 = 1 << 7; // Processnumberexceedlimit
 pub const PF_USED_MATH: u32 = 1 << 8; // useMathematics
 pub const PF_USED_ASYNC: u32 = 1 << 9; // useAsynchronous
 pub const PF_NOFREEZE: u32 = 1 << 10; // notFrozen
 pub const PF_FROZEN: u32 = 1 << 11; // alreadyFrozen
 pub const PF_KTHREAD: u32 = 1 << 12; // KernelThread
 pub const PF_RANDOMIZE: u32 = 1 << 13; // Random
 pub const PF_SWAPWRITE: u32 = 1 << 14; // SwapWrite
 pub const PF_NO_SETAFFINITY: u32 = 1 << 15; // notSetAffinity
 pub const PF_MCE_EARLY: u32 = 1 << 16; // MCE Early
 pub const PF_MUTEX_TESTER: u32 = 1 << 17; // MutexTest
 pub const PF_FREEZER_SKIP: u32 = 1 << 18; // jumpoverFrozen
}

/// ProcessState
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessState {
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

/// ProcessInfo
pub struct ProcessInfo {
 /// Process ID
 pub pid: Pid,
 /// ParentProcess ID
 pub ppid: Pid,
 /// ThreadGroup ID
 pub tgid: Pid,
 /// Session ID
 pub sid: Pid,
 /// ProcessGroup ID
 pub pgid: Pid,
 /// User ID
 pub uid: Uid,
 /// validUser ID
 pub euid: Uid,
 /// SaveUser ID
 pub suid: Uid,
 /// File SystemUser ID
 pub fsuid: Uid,
 /// Group ID
 pub gid: Gid,
 /// validGroup ID
 pub egid: Gid,
 /// SaveGroup ID
 pub sgid: Gid,
 /// File SystemGroup ID
 pub fsgid: Gid,
 /// State
 pub state: AtomicU32,
 /// Flag
 pub flags: AtomicU32,
 /// Exitcode
 pub exit_code: AtomicU32,
 /// ExitSignal
 pub exit_signal: AtomicU32,
}

impl ProcessInfo {
 pub fn new(pid: Pid) -> Self {
 ProcessInfo {
 pid,
 ppid: 0,
 tgid: pid,
 sid: pid,
 pgid: pid,
 uid: 0,
 euid: 0,
 suid: 0,
 fsuid: 0,
 gid: 0,
 egid: 0,
 sgid: 0,
 fsgid: 0,
 state: AtomicU32::new(ProcessState::Running as u32),
 flags: AtomicU32::new(0),
 exit_code: AtomicU32::new(0),
 exit_signal: AtomicU32::new(0),
 }
 }
 
 /// GetState
 pub fn get_state(&self) -> ProcessState {
 match self.state.load(Ordering::Acquire) {
 0 => ProcessState::Running,
 1 => ProcessState::Interruptible,
 2 => ProcessState::Uninterruptible,
 3 => ProcessState::Stopped,
 4 => ProcessState::Traced,
 5 => ProcessState::Zombie,
 6 => ProcessState::Dead,
 _ => ProcessState::Running,
 }
 }
 
 /// SetState
 pub fn set_state(&self, state: ProcessState) {
 self.state.store(state as u32, Ordering::Release);
 }
 
 /// ifisProcess
 pub fn is_zombie(&self) -> bool {
 self.get_state() == ProcessState::Zombie
 }
 
 /// ifalreadyExit
 pub fn is_exiting(&self) -> bool {
 (self.flags.load(Ordering::Acquire) & process_flags::PF_EXITING) != 0
 }
}

/// Resource usage
#[repr(C)]
pub struct Rusage {
 /// User CPU Time
 pub ru_utime: (i64, i64),
 /// System CPU Time
 pub ru_stime: (i64, i64),
 /// MaxcollectionSize
 pub ru_maxrss: i64,
 /// SharedMemorySize
 pub ru_ixrss: i64,
 /// nonSharedDataSize
 pub ru_idrss: i64,
 /// SharedStackSize
 pub ru_isrss: i64,
 /// timewantpageError
 pub ru_minflt: i64,
 /// mainwantpageError
 pub ru_majflt: i64,
 /// timewantpageError (Swap)
 pub ru_nswap: i64,
 /// InputBlockOperation
 pub ru_inblock: i64,
 /// OutputBlockOperation
 pub ru_oublock: i64,
 /// MessageSend
 pub ru_msgsnd: i64,
 /// MessageReceive
 pub ru_msgrcv: i64,
 /// SignalReceive
 pub ru_nsignals: i64,
 /// selfContextSwitch
 pub ru_nvcsw: i64,
 /// selfContextSwitch
 pub ru_nivcsw: i64,
}

/// WaitOption
pub mod wait_options {
 pub const WNOHANG: i32 = 1; // notBlocking
 pub const WUNTRACED: i32 = 2; // reportStop
 pub const WCONTINUED: i32 = 8; // reportcontinue
}

/// ProcessSystemcallImplementation

/// GetProcess ID
pub fn sys_getpid() -> Pid {
 // Return current process ID
 use kernel::process;
 if let Some(current) = process::get_current_process() {
 current.pid
 } else {
 1 // Default to init process
 }
}

/// GetParentProcess ID
pub fn sys_getppid() -> Pid {
 // Return parent process ID
 use kernel::process;
 if let Some(current) = process::get_current_process() {
 current.parent_pid
 } else {
 0 // No parent
 }
}

/// GetUser ID
pub fn sys_getuid() -> Uid {
 // Return user ID
 use kernel::process;
 if let Some(current) = process::get_current_process() {
 current.uid
 } else {
 0 // Root user
 }
}

/// GetvalidUser ID
pub fn sys_geteuid() -> Uid {
 // Return effective user ID
 use kernel::process;
 if let Some(current) = process::get_current_process() {
 current.euid
 } else {
 0 // Root user
 }
}

/// GetGroup ID
pub fn sys_getgid() -> Gid {
 // Return group ID
 use kernel::process;
 if let Some(current) = process::get_current_process() {
 current.gid
 } else {
 0 // Root group
 }
}

/// GetvalidGroup ID
pub fn sys_getegid() -> Gid {
 // Return effective group ID
 use kernel::process;
 if let Some(current) = process::get_current_process() {
 current.egid
 } else {
 0 // Root group
 }
}

/// SetUser ID
pub fn sys_setuid(uid: Uid) -> i32 {
 // ImplementationSetUser ID
 use kernel::process;
 if let Some(current) = process::get_current_process_mut() {
 // Check permissions
 if current.euid == 0 {
 current.uid = uid;
 current.euid = uid;
 0
 } else if current.uid == uid || current.euid == uid {
 current.euid = uid;
 0
 } else {
 -1 // Permission denied
 }
 } else {
 -1 // No current process
 }
}

/// SetGroup ID
pub fn sys_setgid(gid: Gid) -> i32 {
 // ImplementationSetGroup ID
 use kernel::process;
 if let Some(current) = process::get_current_process_mut() {
 // Check permissions
 if current.euid == 0 {
 current.gid = gid;
 current.egid = gid;
 0
 } else if current.gid == gid || current.egid == gid {
 current.egid = gid;
 0
 } else {
 -1 // Permission denied
 }
 } else {
 -1 // No current process
 }
}

/// CreateProcess (fork)
pub fn sys_fork() -> Pid {
 // ImplementationCreateProcess
 use kernel::process;
 
 if let Some(parent) = process::get_current_process() {
 // Create child process
 match process::fork_process(parent) {
 Ok(child_pid) => {
 log_debug!("Fork: parent={}, child={}", parent.pid, child_pid);
 child_pid
 }
 Err(_) => {
 log_warn!("Fork failed");
 -1
 }
 }
 } else {
 -1 // No current process
 }
}

/// CreateProcess (vfork)
pub fn sys_vfork() -> Pid {
 // ImplementationCreateProcess (virtual fork)
 // vfork is similar to fork but shares address space until exec or exit
 use kernel::process;
 
 if let Some(parent) = process::get_current_process() {
 // Create child process with shared address space
 match process::vfork_process(parent) {
 Ok(child_pid) => {
 log_debug!("Vfork: parent={}, child={}", parent.pid, child_pid);
 child_pid
 }
 Err(_) => {
 log_warn!("Vfork failed");
 -1
 }
 }
 } else {
 -1 // No current process
 }
}

/// CloneProcess
pub fn sys_clone(
 flags: u64,
 child_stack: u64,
 ptid: *mut Pid,
 ctid: *mut Pid,
 newtls: u64,
) -> Pid {
 // ImplementationCloneProcess
 use kernel::process;
 
 if let Some(parent) = process::get_current_process() {
 // Clone process with specified flags
 match process::clone_process(parent, flags, child_stack, ptid, ctid, newtls) {
 Ok(child_pid) => {
 log_debug!("Clone: parent={}, child={}, flags={:#x}", parent.pid, child_pid, flags);
 child_pid
 }
 Err(_) => {
 log_warn!("Clone failed");
 -1
 }
 }
 } else {
 -1 // No current process
 }
}

/// executeprocessorder
pub fn sys_execve(
 filename: *const u8,
 argv: *const *const u8,
 envp: *const *const u8,
) -> i32 {
 // Implementationexecuteprocessorder
 use kernel::process;
 
 if filename.is_null() {
 return Errno::Eperm.to_ret_i32(); // Invalid pointer
 }
 
 // Convert filename to string
 // SAFETY: unsafe block required for low-level memory or hardware access
 let filename_str = unsafe {
 use core::ffi::CStr;
 match CStr::from_ptr(filename as *const i8).to_str() {
 Ok(s) => s,
 Err(_) => return Errno::Eperm.to_ret_i32(), // Invalid UTF-8
 }
 };
 
 // Execute program
 match process::exec_process(filename_str, argv, envp) {
 Ok(_) => {
 log_debug!("Exec: {}", filename_str);
 0
 }
 Err(_) => {
 log_warn!("Exec failed: {}", filename_str);
 -1
 }
 }
}

/// ExitProcess
pub fn sys_exit(status: i32) -> ! {
 // ImplementationExitProcess
 use kernel::process;
 
 if let Some(current) = process::get_current_process() {
 log_debug!("Exit: pid={}, status={}", current.pid, status);
 process::exit_process(current, status);
 }
 
 // If no current process, halt
 loop {
 // SAFETY: inline assembly required for hardware instruction
 unsafe { core::arch::asm!("hlt"); }
 }
}

/// WaitProcess
pub fn sys_wait4(pid: Pid, status: *mut i32, options: i32, _rusage: *mut Rusage) -> Pid {
 // ImplementationWaitProcess
 use kernel::process;
 
 if let Some(current) = process::get_current_process() {
 match process::wait_process(current, pid, status, options) {
 Ok(child_pid) => {
 log_debug!("Wait4: parent={}, child={}", current.pid, child_pid);
 child_pid
 }
 Err(_) => {
 -1 // No child to wait for
 }
 }
 } else {
 -1 // No current process
 }
}

/// SendSignal
pub fn sys_kill(pid: Pid, sig: i32) -> i32 {
 // ImplementationSendSignal
 use kernel::process;
 use kernel::signal;
 
 if sig < 0 || sig > 64 {
 return Errno::Eperm.to_ret_i32(); // Invalid signal
 }
 
 // Find target process
 let target = if pid == 0 {
 process::get_current_process()
 } else if pid == -1 {
 // Send to all processes (except current)
 None
 } else if pid < -1 {
 // Send to process group
 process::find_process_by_group(-pid)
 } else {
 process::find_process_by_pid(pid)
 };
 
 if let Some(target) = target {
 match signal::send_signal(target, sig) {
 Ok(_) => {
 log_debug!("Kill: pid={}, sig={}", target.pid, sig);
 0
 }
 Err(_) => Errno::Eperm.to_ret_i32()
 }
 } else {
 -1 // Process not found
 }
}

/// CreateSession
pub fn sys_setsid() -> Pid {
 // ImplementationCreateSession
 use kernel::process;
 
 if let Some(current) = process::get_current_process() {
 match process::create_session(current) {
 Ok(sid) => {
 log_debug!("Setsid: pid={}, sid={}", current.pid, sid);
 sid
 }
 Err(_) => Errno::Eperm.to_ret_i32()
 }
 } else {
 -1 // No current process
 }
}

/// GetSession ID
pub fn sys_getsid(pid: Pid) -> Pid {
 // ImplementationGetSession ID
 use kernel::process;
 
 let target = if pid == 0 {
 process::get_current_process()
 } else {
 process::find_process_by_pid(pid)
 };
 
 if let Some(process) = target {
 process.session_id
 } else {
 -1 // Process not found
 }
}

/// SetProcessGroup
pub fn sys_setpgid(pid: Pid, pgid: Pid) -> i32 {
 // ImplementationSetProcessGroup
 use kernel::process;
 
 let target = if pid == 0 {
 process::get_current_process()
 } else {
 process::find_process_by_pid(pid)
 };
 
 if let Some(process) = target {
 match process::set_process_group(process, pgid) {
 Ok(_) => {
 log_debug!("Setpgid: pid={}, pgid={}", process.pid, pgid);
 0
 }
 Err(_) => Errno::Eperm.to_ret_i32()
 }
 } else {
 -1 // Process not found
 }
}

/// GetProcessGroup
pub fn sys_getpgid(pid: Pid) -> Pid {
 // ImplementationGetProcessGroup
 use kernel::process;
 
 let target = if pid == 0 {
 process::get_current_process()
 } else {
 process::find_process_by_pid(pid)
 };
 
 if let Some(process) = target {
 process.process_group_id
 } else {
 -1 // Process not found
 }
}

/// letexit CPU
pub fn sys_sched_yield() -> i32 {
 // Implementationletexit CPU (Yield CPU)
 use kernel::scheduler;
 
 scheduler::yield_cpu();
 0
}

/// GetPriority
pub fn sys_getpriority(which: i32, who: i32) -> i32 {
 // ImplementationGetPriority
 use kernel::process;
 
 match which {
 0 => {
 // PRIO_PROCESS
 if who == 0 {
 if let Some(current) = process::get_current_process() {
 current.priority
 } else {
 -1
 }
 } else {
 if let Some(process) = process::find_process_by_pid(who as Pid) {
 process.priority
 } else {
 -1
 }
 }
 }
 1 => {
 // PRIO_PGRP
 if let Some(process) = process::find_process_by_group(who as Pid) {
 process.priority
 } else {
 -1
 }
 }
 2 => {
 // PRIO_USER
 if let Some(process) = process::find_process_by_user(who as Uid) {
 process.priority
 } else {
 -1
 }
 }
 _ => Errno::Eperm.to_ret_i32() // Invalid which parameter
 }
}

/// SetPriority
pub fn sys_setpriority(which: i32, who: i32, prio: i32) -> i32 {
 // ImplementationSetPriority
 use kernel::process;
 
 // Clamp priority to valid range
 let prio = prio.clamp(-20, 19);
 
 match which {
 0 => {
 // PRIO_PROCESS
 if who == 0 {
 if let Some(current) = process::get_current_process_mut() {
 current.priority = prio;
 log_debug!("Setpriority: pid={}, prio={}", current.pid, prio);
 0
 } else {
 -1
 }
 } else {
 if let Some(process) = process::find_process_by_pid_mut(who as Pid) {
 process.priority = prio;
 log_debug!("Setpriority: pid={}, prio={}", process.pid, prio);
 0
 } else {
 -1
 }
 }
 }
 1 => {
 // PRIO_PGRP
 if let Some(process) = process::find_process_by_group_mut(who as Pid) {
 process.priority = prio;
 log_debug!("Setpriority: pgid={}, prio={}", who, prio);
 0
 } else {
 -1
 }
 }
 2 => {
 // PRIO_USER
 if let Some(process) = process::find_process_by_user_mut(who as Uid) {
 process.priority = prio;
 log_debug!("Setpriority: uid={}, prio={}", who, prio);
 0
 } else {
 -1
 }
 }
 _ => Errno::Eperm.to_ret_i32() // Invalid which parameter
 }
}

/// GetProcessnumber
pub fn sys_getprocs() -> u32 {
 // Return process count
 use kernel::process;
 process::get_process_count()
}

#[cfg(test)]
mod tests {
 use super::*;

 #[test]
 fn test_process_flags() {
 assert_eq!(process_flags::PF_EXITING, 1 << 0);
 assert_eq!(process_flags::PF_EXITPIDONE, 1 << 1);
 assert_eq!(process_flags::PF_VCPU, 1 << 2);
 assert_eq!(process_flags::PF_SUPERPRIV, 1 << 3);
 assert_eq!(process_flags::PF_DUMPCORE, 1 << 4);
 assert_eq!(process_flags::PF_SIGNALED, 1 << 5);
 assert_eq!(process_flags::PF_KTHREAD, 1 << 12);
 }

 #[test]
 fn test_process_state_values() {
 assert_eq!(ProcessState::Running as u32, 0);
 assert_eq!(ProcessState::Interruptible as u32, 1);
 assert_eq!(ProcessState::Uninterruptible as u32, 2);
 assert_eq!(ProcessState::Stopped as u32, 3);
 assert_eq!(ProcessState::Traced as u32, 4);
 assert_eq!(ProcessState::Zombie as u32, 5);
 assert_eq!(ProcessState::Dead as u32, 6);
 }

 #[test]
 fn test_process_info_new() {
 let proc = ProcessInfo::new(1);

 assert_eq!(proc.pid, 1);
 assert_eq!(proc.ppid, 0);
 assert_eq!(proc.tgid, 1);
 assert_eq!(proc.uid, 0);
 assert_eq!(proc.gid, 0);
 assert_eq!(proc.get_state(), ProcessState::Running);
 }

 #[test]
 fn test_process_info_state_transitions() {
 let proc = ProcessInfo::new(1);

 assert_eq!(proc.get_state(), ProcessState::Running);

 proc.set_state(ProcessState::Interruptible);
 assert_eq!(proc.get_state(), ProcessState::Interruptible);

 proc.set_state(ProcessState::Zombie);
 assert_eq!(proc.get_state(), ProcessState::Zombie);
 assert!(proc.is_zombie());
 }

 #[test]
 fn test_process_info_is_zombie() {
 let proc = ProcessInfo::new(1);

 assert!(!proc.is_zombie());

 proc.set_state(ProcessState::Zombie);
 assert!(proc.is_zombie());
 }

 #[test]
 fn test_process_info_is_exiting() {
 let proc = ProcessInfo::new(1);

 assert!(!proc.is_exiting());

 proc.flags.fetch_or(process_flags::PF_EXITING, Ordering::Relaxed);
 assert!(proc.is_exiting());
 }

 #[test]
 fn test_process_info_exit_code() {
 let proc = ProcessInfo::new(1);

 assert_eq!(proc.exit_code.load(Ordering::Relaxed), 0);

 proc.exit_code.store(42, Ordering::Release);
 assert_eq!(proc.exit_code.load(Ordering::Relaxed), 42);
 }

 #[test]
 fn test_wait_options() {
 assert_eq!(wait_options::WNOHANG, 1);
 assert_eq!(wait_options::WUNTRACED, 2);
 assert_eq!(wait_options::WCONTINUED, 8);
 }

 #[test]
 fn test_sys_getpid() {
 let pid = sys_getpid();
 assert_eq!(pid, 1);
 }

 #[test]
 fn test_sys_getppid() {
 let ppid = sys_getppid();
 assert_eq!(ppid, 0);
 }

 #[test]
 fn test_sys_getuid() {
 let uid = sys_getuid();
 assert_eq!(uid, 0);
 }

 #[test]
 fn test_sys_geteuid() {
 let euid = sys_geteuid();
 assert_eq!(euid, 0);
 }

 #[test]
 fn test_sys_getgid() {
 let gid = sys_getgid();
 assert_eq!(gid, 0);
 }

 #[test]
 fn test_sys_getegid() {
 let egid = sys_getegid();
 assert_eq!(egid, 0);
 }

 #[test]
 fn test_sys_setuid() {
 let result = sys_setuid(1000);
 assert_eq!(result, -1); // TODO ImplementationthenshouldReturn 0
 }

 #[test]
 fn test_sys_setgid() {
 let result = sys_setgid(1000);
 assert_eq!(result, -1); // TODO ImplementationthenshouldReturn 0
 }

 #[test]
 fn test_sys_fork() {
 let pid = sys_fork();
 assert_eq!(pid, 0); // TODO ImplementationthenshouldReturnChildProcess PID
 }

 #[test]
 fn test_sys_vfork() {
 let pid = sys_vfork();
 assert_eq!(pid, 0); // TODO ImplementationthenshouldReturnChildProcess PID
 }

 #[test]
 fn test_sys_sched_yield() {
 let result = sys_sched_yield();
 assert_eq!(result, 0);
 }

 #[test]
 fn test_sys_getprocs() {
 let count = sys_getprocs();
 assert_eq!(count, 1);
 }

 #[test]
 fn test_sys_setsid() {
 let sid = sys_setsid();
 assert_eq!(sid, -1); // TODO ImplementationthenshouldReturnSession ID
 }

 #[test]
 fn test_sys_getpriority() {
 let prio = sys_getpriority(0, 0);
 assert_eq!(prio, -1); // TODO ImplementationthenshouldReturnPriority
 }

 #[test]
 fn test_sys_setpriority() {
 let result = sys_setpriority(0, 0, 10);
 assert_eq!(result, -1); // TODO ImplementationthenshouldReturn 0
 }

 #[test]
 fn test_process_state_equality() {
 assert_eq!(ProcessState::Running, ProcessState::Running);
 assert_ne!(ProcessState::Running, ProcessState::Zombie);
 assert_ne!(ProcessState::Stopped, ProcessState::Traced);
 }

 #[test]
 fn test_type_aliases() {
 let pid: Pid = 1234;
 let uid: Uid = 1000;
 let gid: Gid = 100;

 assert_eq!(pid, 1234u32);
 assert_eq!(uid, 1000u32);
 assert_eq!(gid, 100u32);
 }

 #[test]
 fn test_rusage() {
 let rusage = Rusage {
 ru_utime: (1, 0),
 ru_stime: (0, 500000),
 ru_maxrss: 1024,
 ru_ixrss: 0,
 ru_idrss: 0,
 ru_isrss: 0,
 ru_minflt: 100,
 ru_majflt: 5,
 ru_nswap: 0,
 ru_inblock: 50,
 ru_oublock: 30,
 ru_msgsnd: 0,
 ru_msgrcv: 0,
 ru_nsignals: 0,
 ru_nvcsw: 10,
 ru_nivcsw: 5,
 };

 assert_eq!(rusage.ru_utime, (1, 0));
 assert_eq!(rusage.ru_maxrss, 1024);
 assert_eq!(rusage.ru_minflt, 100);
 }
}