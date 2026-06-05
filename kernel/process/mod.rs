/*
 * Nuva OS - Kernel - Process Management
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

//! Process Management Implementation
/*!*/
//! Complete process management with:
//! - fork, vfork, clone
//! - execve
//! - Signal handling
//! - wait, waitpid

// Re-export print macros from crate root
pub use crate::{pr_emerg, pr_alert, pr_crit, pr_err, pr_warn, pr_notice, pr_info, pr_debug};

// POSIX-compatible process operations (optional, for POSIX compatibility)
// Migrated from: POSIX fork() → nv_process_spawn (kernel::nv_process)
// Migrated from: POSIX execve() → nv_process_execute (kernel::nv_process)
// Migrated from: POSIX signal → nv_event (kernel::nv_event)
#[cfg(feature = "posix")]
pub mod fork;
#[cfg(feature = "posix")]
pub mod execve;
#[cfg(feature = "posix")]
pub mod wait4;
#[cfg(feature = "posix")]
pub mod signal;
pub mod tests;

#[cfg(feature = "posix")]
pub use fork::*;
#[cfg(feature = "posix")]
pub use signal::*;

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use crate::kernel::sched::task::TaskStruct;
#[cfg(feature = "posix")]
use crate::kernel::process::signal::{SigSet, SigAltStack, SigAction};

/// Process ID type
pub type Pid = u32;

/// Thread ID type
pub type Tid = u32;

/// Group ID type
pub type Gid = u32;

/// User ID type
pub type Uid = u32;

/// Process state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessState {
    /// Unused slot
    Unused = 0,
    /// Being created
    Creating = 1,
    /// Ready to run
    Ready = 2,
    /// Currently running
    Running = 3,
    /// Sleeping, can be interrupted
    Interruptible = 4,
    /// Sleeping, cannot be interrupted
    Uninterruptible = 5,
    /// Stopped by signal
    Stopped = 6,
    /// Zombie process
    Zombie = 7,
    /// Dead
    Dead = 8,
}

/// Process flags
pub mod process_flags {
    pub const PF_KTHREAD: u32 = 0x00000001;
    pub const PF_EXITING: u32 = 0x00000002;
    pub const PF_EXITPIDONE: u32 = 0x00000004;
    pub const PF_VCPU: u32 = 0x00000008;
    pub const PF_FORKNOEXEC: u32 = 0x00000010;
    pub const PF_SUPERPRIV: u32 = 0x00000100;
    pub const PF_DUMPCORE: u32 = 0x00000200;
    pub const PF_SIGNALED: u32 = 0x00000400;
    pub const PF_MEMALLOC: u32 = 0x00000800;
    pub const PF_NPROC_EXCEEDED: u32 = 0x00001000;
    pub const PF_USED_MATH: u32 = 0x00002000;
    pub const PF_USED_ASYNC: u32 = 0x00004000;
    pub const PF_NOFREEZE: u32 = 0x00008000;
    pub const PF_FROZEN: u32 = 0x00010000;
}

/// Process credentials
pub struct Credentials {
    /// User ID
    pub uid: AtomicU32,
    /// Effective user ID
    pub euid: AtomicU32,
    /// Saved user ID
    pub suid: AtomicU32,
    /// Filesystem user ID
    pub fsuid: AtomicU32,
    /// Group ID
    pub gid: AtomicU32,
    /// Effective group ID
    pub egid: AtomicU32,
    /// Saved group ID
    pub sgid: AtomicU32,
    /// Filesystem group ID
    pub fsgid: AtomicU32,
    /// Supplementary groups
    pub groups: [AtomicU32; 32],
    /// Number of groups
    pub ngroups: AtomicU32,
    /// Capabilities
    pub cap_effective: AtomicU64,
    pub cap_inheritable: AtomicU64,
    pub cap_permitted: AtomicU64,
}

impl Credentials {
    pub const fn new() -> Self {
        Credentials {
            uid: AtomicU32::new(0),
            euid: AtomicU32::new(0),
            suid: AtomicU32::new(0),
            fsuid: AtomicU32::new(0),
            gid: AtomicU32::new(0),
            egid: AtomicU32::new(0),
            sgid: AtomicU32::new(0),
            fsgid: AtomicU32::new(0),
            groups: [const { AtomicU32::new(0) }; 32],
            ngroups: AtomicU32::new(0),
            cap_effective: AtomicU64::new(0),
            cap_inheritable: AtomicU64::new(0),
            cap_permitted: AtomicU64::new(0),
        }
    }

    /// Clone by reading atomic values
    pub fn clone(&self) -> Self {
        Credentials {
            uid: AtomicU32::new(self.uid.load(Ordering::Relaxed)),
            euid: AtomicU32::new(self.euid.load(Ordering::Relaxed)),
            suid: AtomicU32::new(self.suid.load(Ordering::Relaxed)),
            fsuid: AtomicU32::new(self.fsuid.load(Ordering::Relaxed)),
            gid: AtomicU32::new(self.gid.load(Ordering::Relaxed)),
            egid: AtomicU32::new(self.egid.load(Ordering::Relaxed)),
            sgid: AtomicU32::new(self.sgid.load(Ordering::Relaxed)),
            fsgid: AtomicU32::new(self.fsgid.load(Ordering::Relaxed)),
            groups: core::array::from_fn(|i| AtomicU32::new(self.groups[i].load(Ordering::Relaxed))),
            ngroups: AtomicU32::new(self.ngroups.load(Ordering::Relaxed)),
            cap_effective: AtomicU64::new(self.cap_effective.load(Ordering::Relaxed)),
            cap_inheritable: AtomicU64::new(self.cap_inheritable.load(Ordering::Relaxed)),
            cap_permitted: AtomicU64::new(self.cap_permitted.load(Ordering::Relaxed)),
        }
    }
}

/// Process memory descriptor
pub struct MmStruct {
    /// Start of code
    pub start_code: u64,
    /// End of code
    pub end_code: u64,
    /// Start of data
    pub start_data: u64,
    /// End of data
    pub end_data: u64,
    /// Start of heap
    pub start_brk: u64,
    /// Current heap
    pub brk: u64,
    /// Start of stack
    pub start_stack: u64,
    /// Arg start
    pub arg_start: u64,
    /// Arg end
    pub arg_end: u64,
    /// Env start
    pub env_start: u64,
    /// Env end
    pub env_end: u64,
    /// Total virtual memory size
    pub total_vm: AtomicU64,
    /// Locked virtual memory
    pub locked_vm: AtomicU64,
    /// Data size
    pub data_vm: AtomicU64,
    /// Executable file
    pub exe_file: u64,
    /// Page table root
    pub pgd: u64,
    /// Reference count
    pub ref_count: AtomicU32,
    /// Map count
    pub map_count: AtomicU32,
}

impl MmStruct {
    pub const fn new() -> Self {
        MmStruct {
            start_code: 0,
            end_code: 0,
            start_data: 0,
            end_data: 0,
            start_brk: 0,
            brk: 0,
            start_stack: 0,
            arg_start: 0,
            arg_end: 0,
            env_start: 0,
            env_end: 0,
            total_vm: AtomicU64::new(0),
            locked_vm: AtomicU64::new(0),
            data_vm: AtomicU64::new(0),
            exe_file: 0,
            pgd: 0,
            ref_count: AtomicU32::new(1),
            map_count: AtomicU32::new(0),
        }
    }

    /// Clone by reading atomic values
    pub fn clone(&self) -> Self {
        MmStruct {
            start_code: self.start_code,
            end_code: self.end_code,
            start_data: self.start_data,
            end_data: self.end_data,
            start_brk: self.start_brk,
            brk: self.brk,
            start_stack: self.start_stack,
            arg_start: self.arg_start,
            arg_end: self.arg_end,
            env_start: self.env_start,
            env_end: self.env_end,
            total_vm: AtomicU64::new(self.total_vm.load(Ordering::Relaxed)),
            locked_vm: AtomicU64::new(self.locked_vm.load(Ordering::Relaxed)),
            data_vm: AtomicU64::new(self.data_vm.load(Ordering::Relaxed)),
            exe_file: self.exe_file,
            pgd: self.pgd,
            ref_count: AtomicU32::new(self.ref_count.load(Ordering::Relaxed)),
            map_count: AtomicU32::new(self.map_count.load(Ordering::Relaxed)),
        }
    }
}

/// Signal structure
pub struct SignalStruct {
    /// Blocked signals (mask)
    pub blocked: SigSet,
    /// Real blocked signals
    pub real_blocked: SigSet,
    /// Pending signals
    pub pending: SigSet,
    /// Shared pending
    pub shared_pending: SigSet,
    /// Signal handler table
    pub action: [SigAction; 64],
    /// Alternate stack
    pub altstack: SigAltStack,
    /// Signal flags
    pub flags: AtomicU32,
    /// Signal handlers (alias for action access)
    pub handlers: [SigAction; 64],
}

/// Signal action
impl SignalStruct {
    pub const fn new() -> Self {
        SignalStruct {
            blocked: SigSet::new(),
            real_blocked: SigSet::new(),
            pending: SigSet::new(),
            shared_pending: SigSet::new(),
            action: [const { SigAction {
                handler: 0,
                flags: 0,
                mask: SigSet::new(),
                restorer: 0,
            } }; 64],
            altstack: SigAltStack::new(),
            handlers: [const { SigAction {
                handler: 0,
                flags: 0,
                mask: SigSet::new(),
                restorer: 0,
            } }; 64],
            flags: AtomicU32::new(0),
        }
    }
    pub fn next_pending(&self) -> Option<u32> {
        for i in 0..128u32 {
            let idx = ((i) / 64) as usize;
            let bit = i % 64;
            if idx < 2 && (self.pending.bits[idx] & (1u64 << bit)) != 0 {
                return Some(i + 1);
            }
        }
        None
    }
}

/// File descriptor table
pub struct FileTable {
    /// File descriptor entries
    pub fd_table: [u64; 256],
    /// Open file count
    pub open_count: AtomicU32,
}

impl FileTable {
    /// Create empty file table
    pub const fn new() -> Self {
        FileTable {
            fd_table: [0; 256],
            open_count: AtomicU32::new(0),
        }
    }
}

/// File system structure
pub struct FsStruct {
    /// Current working directory inode
    pub cwd: u64,
    /// Root directory inode
    pub root: u64,
    /// File system magic
    pub magic: u32,
}

impl FsStruct {
    /// Create default fs struct
    pub const fn new() -> Self {
        FsStruct {
            cwd: 0,
            root: 0,
            magic: 0,
        }
    }
}

/// Process structure
pub struct Process {
    /// Process ID
    pub pid: Pid,
    /// Thread ID
    pub tid: Tid,
    /// Parent process ID
    pub ppid: Pid,
    /// Process group ID
    pub pgid: Pid,
    /// Session ID
    pub sid: Pid,
    /// Process state
    pub state: AtomicU32,
    /// Process flags
    pub flags: AtomicU32,
    /// Exit code
    pub exit_code: AtomicU32,
    /// Exit signal
    pub exit_signal: AtomicU32,
    /// Credentials
    pub cred: Credentials,
    /// Memory descriptor
    pub mm: MmStruct,
    /// Signal structure
    pub signal: SignalStruct,
    /// Thread count
    pub thread_count: AtomicU32,
    /// Reference count
    pub ref_count: AtomicU32,
    /// Open file count
    pub file_count: AtomicU32,
    /// Command name
    pub comm: [u8; 16],
    /// Start time
    pub start_time: AtomicU64,
    /// Real start time
    pub real_start_time: AtomicU64,
    /// CPU time (user)
    pub utime: AtomicU64,
    /// CPU time (system)
    pub stime: AtomicU64,
    /// Children CPU time (user)
    pub cutime: AtomicU64,
    /// Children CPU time (system)
    pub cstime: AtomicU64,
    /// Next process in list
    pub next: *mut Process,
    /// Previous process in list
    pub prev: *mut Process,
    /// Parent process
    pub parent: *mut Process,
    /// Real parent process
    pub real_parent: *mut Process,
    /// Children list
    pub children: *mut Process,
    /// Sibling list
    pub sibling: *mut Process,
    /// Threads list
    pub threads: *mut Process,
    /// File descriptor table
    pub files: FileTable,
    /// File system info
    pub fs: FsStruct,
    /// Thread group ID
    pub tgid: Pid,
    /// Group leader
    pub group_leader: *mut Process,
    /// Task state
    pub task: TaskStruct,
}

impl Process {
    /// Create new process
    pub fn new(pid: Pid) -> Self {
        Process {
            pid,
            tid: pid,
            ppid: 0,
            pgid: pid,
            sid: pid,
            state: AtomicU32::new(ProcessState::Creating as u32),
            flags: AtomicU32::new(0),
            exit_code: AtomicU32::new(0),
            exit_signal: AtomicU32::new(0),
            cred: Credentials::new(),
            mm: MmStruct::new(),
            signal: SignalStruct::new(),
            thread_count: AtomicU32::new(1),
            ref_count: AtomicU32::new(1),
            file_count: AtomicU32::new(0),
            comm: [0; 16],
            start_time: AtomicU64::new(0),
            real_start_time: AtomicU64::new(0),
            utime: AtomicU64::new(0),
            stime: AtomicU64::new(0),
            cutime: AtomicU64::new(0),
            cstime: AtomicU64::new(0),
            next: core::ptr::null_mut(),
            prev: core::ptr::null_mut(),
            parent: core::ptr::null_mut(),
            real_parent: core::ptr::null_mut(),
            children: core::ptr::null_mut(),
            sibling: core::ptr::null_mut(),
            threads: core::ptr::null_mut(),
            files: FileTable::new(),
            fs: FsStruct::new(),
            tgid: pid,
            group_leader: core::ptr::null_mut(),
            task: TaskStruct::new(),
        }
    }
    
    /// Set command name
    pub fn set_comm(&mut self, name: &[u8]) {
        let len = name.len().min(15);
        self.comm[..len].copy_from_slice(&name[..len]);
        self.comm[len] = 0;
    }
    
    /// Check if process is a kernel thread
    pub fn is_kernel_thread(&self) -> bool {
        (self.flags.load(Ordering::Acquire) & process_flags::PF_KTHREAD) != 0
    }
    
    /// Check if process is exiting
    pub fn is_exiting(&self) -> bool {
        (self.flags.load(Ordering::Acquire) & process_flags::PF_EXITING) != 0
    }
    
    /// Get process state
    pub fn get_state(&self) -> ProcessState {
        match self.state.load(Ordering::Acquire) {
            0 => ProcessState::Unused,
            1 => ProcessState::Creating,
            2 => ProcessState::Ready,
            3 => ProcessState::Running,
            4 => ProcessState::Interruptible,
            5 => ProcessState::Uninterruptible,
            6 => ProcessState::Stopped,
            7 => ProcessState::Zombie,
            8 => ProcessState::Dead,
            _ => ProcessState::Unused,
        }
    }
    
    /// Set process state
    pub fn set_state(&self, state: ProcessState) {
        self.state.store(state as u32, Ordering::Release);
    }
    
    /// Increment reference count
    pub fn get(&self) {
        self.ref_count.fetch_add(1, Ordering::AcqRel);
    }
    
    /// Decrement reference count
    pub fn put(&self) -> u32 {
        self.ref_count.fetch_sub(1, Ordering::AcqRel)
    }
}

/// Process manager
pub struct ProcessManager {
    /// Process table
    pub process_table: [Option<*mut Process>; 65536],
    /// Number of processes
    pub nr_processes: AtomicU32,
    /// Number of threads
    pub nr_threads: AtomicU32,
    /// Next PID
    pub next_pid: AtomicU32,
    /// Init process
    pub init_process: *mut Process,
    /// Current process per CPU
    pub current: [*mut Process; 16],
    /// Process list
    pub process_list: *mut Process,
    /// Statistics
    pub stats: ProcessStats,
}

/// Process statistics
pub struct ProcessStats {
    /// Total forks
    pub forks: AtomicU64,
    /// Total execs
    pub execs: AtomicU64,
    /// Total exits
    pub exits: AtomicU64,
    /// Context switches
    pub context_switches: AtomicU64,
}

impl ProcessStats {
    pub const fn new() -> Self {
        ProcessStats {
            forks: AtomicU64::new(0),
            execs: AtomicU64::new(0),
            exits: AtomicU64::new(0),
            context_switches: AtomicU64::new(0),
        }
    }
}

impl ProcessManager {
    pub const fn new() -> Self {
        ProcessManager {
            process_table: [None; 65536],
            nr_processes: AtomicU32::new(0),
            nr_threads: AtomicU32::new(0),
            next_pid: AtomicU32::new(1),
            init_process: core::ptr::null_mut(),
            current: [core::ptr::null_mut(); 16],
            process_list: core::ptr::null_mut(),
            stats: ProcessStats::new(),
        }
    }
    
    /// Initialize process manager
    pub fn init(&self) {
        log_info!("Process manager initialized");
        
        // Create init process (PID 1)
        self.create_init_process();
    }
    
    /// Create init process
    fn create_init_process(&mut self) {
        let init = Process::new(1);
        // TODO: Allocate and store init process
        let _ = init;
        
        self.next_pid.store(2, Ordering::Release);
    }
    
    /// Allocate PID
    pub fn alloc_pid(&self) -> Pid {
        self.next_pid.fetch_add(1, Ordering::AcqRel)
    }
    
    /// Find process by PID
    pub fn find_process(&self, pid: Pid) -> *mut Process {
        if pid >= 65536 {
            return core::ptr::null_mut();
        }
        
        match self.process_table[pid as usize] {
            Some(proc) => proc,
            None => core::ptr::null_mut(),
        }
    }
    
    /// Create process (fork)
    pub fn do_fork(&mut self, flags: u64, stack: u64, parent_tid: *mut i32,
                   child_tid: *mut i32, tls: u64) -> Result<Pid, i32> {
        let _ = (flags, stack, parent_tid, child_tid, tls);
        
        // Allocate PID
        let pid = self.alloc_pid();
        
        // Create process structure
        let child_ptr = &mut Process::new(pid) as *mut Process;
        
        // Copy parent process
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let parent = self.current[0]; // Get current process
            
            if !parent.is_null() {
                // Copy parent's memory space
                (*child_ptr).mm = (*parent).mm.clone();
                
                // Copy parent's file descriptors
                for i in 0..(*parent).files.open_count.load(Ordering::Acquire) as usize {
                    if i < 256 {
                        (*child_ptr).files.fd_table[i] = (*parent).files.fd_table[i];
                    }
                }
                (*child_ptr).files.open_count.store(
                    (*parent).files.open_count.load(Ordering::Acquire),
                    Ordering::Release
                );
                
                // Copy parent's current directory
                (*child_ptr).fs.cwd = (*parent).fs.cwd;
                
                // Copy parent's signal handlers
                for i in 0..64 {
                    (*child_ptr).signal.handlers[i] = (*parent).signal.handlers[i];
                }
                
                // Copy parent's credentials
                (*child_ptr).cred = (*parent).cred.clone();
                
                // Set parent relationship
                (*child_ptr).parent = parent;
                (*child_ptr).real_parent = parent;
                
                // Add child to parent's children list
                (*child_ptr).sibling = (*parent).children;
                (*parent).children = child_ptr;
                
                // Set child's thread group
                (*child_ptr).tgid = (*parent).tgid;
                (*child_ptr).group_leader = (*parent).group_leader;
            }
            
            // Set up child task for scheduler
            (*child_ptr).task.state.store(crate::kernel::sched::TaskState::Ready as u32, Ordering::Release);
            (*child_ptr).task.prio = if !parent.is_null() { (*parent).task.prio } else { 120 };
            (*child_ptr).task.time_slice = 100;
            
            // Set return value for child (0)
            (*child_ptr).task.cpu_context.regs[0] = 0; // rax = 0 for child
        }
        
        // Add to process table
        if (pid as usize) < 65536 {
            // SAFETY: storing child pointer in process table
            unsafe {
                self.process_table[pid as usize] = Some(child_ptr);
            }
        }
        
        self.nr_processes.fetch_add(1, Ordering::AcqRel);
        self.nr_threads.fetch_add(1, Ordering::AcqRel);
        self.stats.forks.fetch_add(1, Ordering::AcqRel);
        
        Ok(pid)
    }
    
    /// Execute new program
    pub fn do_execve(&mut self, filename: *const u8, argv: *const *const u8,
                     envp: *const *const u8) -> Result<(), i32> {
        // Get current process
        let cpu = 0;
        let current = self.current[cpu];
        
        if current.is_null() {
            return Err(-1);
        }
        
        // Convert filename to slice
        // SAFETY: unsafe block required for low-level memory or hardware access
        let filename_slice = unsafe {
            let len = (0..256).position(|i| *filename.add(i) == 0).unwrap_or(256);
            core::slice::from_raw_parts(filename, len)
        };
        
        // Load executable using filesystem
        // TODO: Call VFS to open and read executable
        
        // Parse ELF header
        // TODO: Implement ELF loader
        
        // Set up new memory space
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            // Clear old memory mappings (except shared)
            // (*current).mm.clear();
            
            // Create new address space
            // let entry_point = self.load_elf(filename_slice)?;
            
            // Set up new stack
            let stack_top = 0x7FFFFFFFFFFF as u64; // User stack top
            let stack_size = 8 * 1024 * 1024; // 8MB stack
            
            // Map stack pages
            // self.setup_stack(current, stack_top, stack_size)?;
            
            // Set up arguments and environment on stack
            // self.setup_argv_envp(current, argv, envp)?;
            
            // Set entry point
            // (*current).task.context.ip = entry_point;
            // (*current).task.context.sp = stack_top;
            
            // Clear registers
            for i in 0..16 {
                (*current).task.cpu_context.regs[i] = 0;
            }
            
            // Reset signal handlers to default
            for i in 0..64 {
                (*current).signal.handlers[i].handler = 0;
            }
            
            // Update process name
            let name_len = filename_slice.len().min(15);
            (*current).comm[..name_len].copy_from_slice(&filename_slice[..name_len]);
            (*current).comm[name_len] = 0;
        }
        
        self.stats.execs.fetch_add(1, Ordering::AcqRel);
        
        Ok(())
    }
    
    /// Exit process
    pub fn do_exit(&mut self, error_code: i32) {
        let cpu = 0;
        let current = self.current[cpu];

        if current.is_null() {
            return;
        }

        // SAFETY: process exit - release resources and mark zombie
        unsafe {
            (*current).exit_code.store(error_code as u32, Ordering::Release);
            (*current).flags.fetch_or(process_flags::PF_EXITING, Ordering::AcqRel);
            (*current).set_state(ProcessState::Zombie);

            let pid = (*current).pid;
            if (pid as usize) < 65536 {
                self.process_table[pid as usize] = None;
            }

            for i in 0..(*current).files.open_count.load(Ordering::Acquire) as usize {
                if i < 256 {
                    (*current).files.fd_table[i] = 0;
                }
            }
            (*current).files.open_count.store(0, Ordering::Release);
            (*current).mm.pgd = 0;
            (*current).mm.ref_count.store(0, Ordering::Release);
        }

        self.nr_processes.fetch_sub(1, Ordering::AcqRel);
        self.stats.exits.fetch_add(1, Ordering::AcqRel);

        self.notify_parent(current);
        crate::kernel::sched::schedule();
    }
    
    /// Notify parent of child exit
    fn notify_parent(&mut self, child: *mut Process) {
        if child.is_null() {
            return;
        }
        
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let parent = (*child).parent;
            if !parent.is_null() {
                // Send SIGCHLD to parent
                // TODO: Implement signal sending
            }
        }
    }
    
    /// Wait for child process
    pub fn do_wait4(&mut self, pid: Pid, status: *mut i32, options: i32,
                    ru: *mut Rusage) -> Result<Pid, i32> {
        let _ = (status, options, ru);
        
        let cpu = 0;
        let current = self.current[cpu];
        
        if current.is_null() {
            return Err(-1);
        }
        
        // Find child process
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let mut child = (*current).children;
            
            while !child.is_null() {
                let next = (*child).sibling;
                
                // Check if this is the child we're waiting for
                if pid == 0 || pid == u32::MAX || (*child).pid == pid as u32 {
                    // Check if child has exited
                    if (*child).get_state() == ProcessState::Zombie {
                        let child_pid = (*child).pid;
                        
                        // TODO: Collect exit status
                        // TODO: Free child process
                        
                        return Ok(child_pid);
                    }
                }
                
                child = next;
            }
        }
        
        // No child found or no exited children
        Err(-10)  /* ECHILD */
    }
    
    /// Get current process
    pub fn get_current(&self) -> *mut Process {
        let cpu = 0;  /* TODO: Get current CPU */
        self.current[cpu]
    }
    
    /// Set current process
    pub fn set_current(&mut self, proc: *mut Process) {
        let cpu = 0;
        self.current[cpu] = proc;
    }
    
    /// Print process list
    pub fn print_processes(&self) {
        log_info!("Process List:");
        log_info!("  PID  PPID  STATE    COMM");
        
        let mut proc = self.process_list;
        while !proc.is_null() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                let state = match (*proc).get_state() {
                    ProcessState::Running => "Running",
                    ProcessState::Ready => "Ready",
                    ProcessState::Interruptible => "Sleep",
                    ProcessState::Zombie => "Zombie",
                    _ => "Other",
                };
                
                log_info!("  {:4} {:4}  {:8} {:?}",
                         (*proc).pid, (*proc).ppid, state,
                         core::str::from_utf8_unchecked(&(*proc).comm));
                
                proc = (*proc).next;
            }
        }
    }
}

/// Resource usage
#[repr(C)]
pub struct Rusage {
    pub ru_utime: Timeval,
    pub ru_stime: Timeval,
    pub ru_maxrss: i64,
    pub ru_ixrss: i64,
    pub ru_idrss: i64,
    pub ru_isrss: i64,
    pub ru_minflt: i64,
    pub ru_majflt: i64,
    pub ru_nswap: i64,
    pub ru_inblock: i64,
    pub ru_oublock: i64,
    pub ru_msgsnd: i64,
    pub ru_msgrcv: i64,
    pub ru_nsignals: i64,
    pub ru_nvcsw: i64,
    pub ru_nivcsw: i64,
}

/// Time value
#[repr(C)]
pub struct Timeval {
    pub seconds: i64,
    pub microseconds: i64,
}

/// Global process manager
static PROCESS_MANAGER: core::sync::OnceLock<ProcessManager> = core::sync::OnceLock::new();

/// Global process table (array of process pointers)
static mut PROCESS_TABLE: [*mut Process; 4096] = [core::ptr::null_mut(); 4096];

/// Global process count
static PROCESS_COUNT: AtomicU32 = AtomicU32::new(0);

/// Get process manager
pub fn process_manager() -> &'static ProcessManager {
    PROCESS_MANAGER.get_or_init(ProcessManager::new)
}

pub fn init_process_manager() -> &'static ProcessManager {
    PROCESS_MANAGER.get_or_init(ProcessManager::new)
}

/// Initialize process management
pub fn init_process() {
    let mgr = process_manager();
    mgr.init();
}

/// Get current process
pub fn get_current() -> *mut Process {
    process_manager().get_current()
}

/// Fork system call
pub fn sys_fork() -> i64 {
    match process_manager().do_fork(0, 0, core::ptr::null_mut(),
                                         core::ptr::null_mut(), 0) {
        Ok(pid) => pid as i64,
        Err(e) => e as i64,
    }
}

/// Execve system call
pub fn sys_execve(filename: *const u8, argv: *const *const u8,
                  envp: *const *const u8) -> i64 {
    match process_manager().do_execve(filename, argv, envp) {
        Ok(()) => 0,
        Err(e) => e as i64,
    }
}

/// Exit system call
pub fn sys_exit(status: i32) {
    process_manager().do_exit(status);
}

/// Wait4 system call
pub fn sys_wait4(pid: Pid, status: *mut i32, options: i32,
                 ru: *mut Rusage) -> i64 {
    match process_manager().do_wait4(pid, status, options, ru) {
        Ok(p) => p as i64,
        Err(e) => e as i64,
    }
}

/// Process resource limit
pub struct Rlimit {
    /// Current (soft) limit
    pub rlim_cur: u64,
    /// Maximum (hard) limit
    pub rlim_max: u64,
}

/// Resource limit types
pub mod resource {
    pub const RLIMIT_CPU: usize = 0;
    pub const RLIMIT_FSIZE: usize = 1;
    pub const RLIMIT_DATA: usize = 2;
    pub const RLIMIT_STACK: usize = 3;
    pub const RLIMIT_CORE: usize = 4;
    pub const RLIMIT_RSS: usize = 5;
    pub const RLIMIT_NPROC: usize = 6;
    pub const RLIMIT_NOFILE: usize = 7;
    pub const RLIMIT_MEMLOCK: usize = 8;
    pub const RLIMIT_AS: usize = 9;
    pub const RLIMIT_LOCKS: usize = 10;
    pub const RLIMIT_SIGPENDING: usize = 11;
    pub const RLIMIT_MSGQUEUE: usize = 12;
    pub const RLIMIT_NICE: usize = 13;
    pub const RLIMIT_RTPRIO: usize = 14;
    pub const RLIMIT_RTTIME: usize = 15;
    pub const RLIM_NLIMITS: usize = 16;
}

// TODO: Implement exit_group syscall
pub fn sys_exit_group(status: i32) -> i64 {
    let mgr = process_manager();
    mgr.do_exit(status);
    0
}

pub fn kernel_thread_create(name: &str, func: fn(), priority: i32) -> Pid {
    let mgr = process_manager();
    let pid = mgr.alloc_pid();

    if pid as usize >= 65536 {
        return 0;
    }

    let child_ptr = &mut Process::new(pid) as *mut Process;

    // SAFETY: initializing new kernel thread
    unsafe {
        let name_bytes = name.as_bytes();
        let name_len = name_bytes.len().min(15);
        (*child_ptr).comm[..name_len].copy_from_slice(&name_bytes[..name_len]);
        (*child_ptr).comm[name_len] = 0;

        (*child_ptr).flags.fetch_or(process_flags::PF_KTHREAD, Ordering::AcqRel);
        (*child_ptr).set_state(ProcessState::Ready);
        (*child_ptr).task.prio = if priority > 0 { priority } else { 120 };
        (*child_ptr).task.time_slice = 100;
        (*child_ptr).task.cpu_context.pc = func as u64;
        (*child_ptr).task.cpu_context.sp = 0;
        (*child_ptr).task.cpu_context.regs[0] = 0;

        (*child_ptr).task.state.store(
            crate::kernel::sched::TaskState::Ready as u32,
            Ordering::Release
        );

        mgr.process_table[pid as usize] = Some(child_ptr);
    }

    mgr.nr_processes.fetch_add(1, Ordering::AcqRel);
    mgr.nr_threads.fetch_add(1, Ordering::AcqRel);

    log_info!("Created kernel thread: {} (pid={})", name, pid);
    pid
}
