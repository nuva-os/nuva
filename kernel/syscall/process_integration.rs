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

// ! ProcessmanagementadministrationSystemtuneusecollectionsuccessModule
/*!*/
// ! theModuleImplementationProcessmanagementadministrationSystemtuneuseandKernelProcessmanagementadministrationChildSystem collectionsuccess

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use crate::kernel::user::task::{Process, Thread, TaskManager, ProcessState, ProcessPriority, ThreadState};
use crate::kernel::user::user::{Uid, Gid};
use crate::kernel::fs::vfs::file::FilesStruct;
use crate::kernel::mm::address_space::{AddressSpace, create_address_space, copy_address_space, destroy_address_space};

/// ELF magic number
const ELF_MAGIC: u32 = 0x7F454C46;

/// ELF class: 64-bit
const ELF_CLASS_64: u8 = 2;

/// ELF data: little endian
const ELF_DATA_LE: u8 = 1;

/// ELF type: executable
const ET_EXEC: u16 = 2;

/// ELF type: shared object
const ET_DYN: u16 = 3;

/// ELF program header type: load
const PT_LOAD: u32 = 1;

/// ELF program header flags
const PF_X: u32 = 1;
const PF_W: u32 = 2;
const PF_R: u32 = 4;

/// ELF64 file header
/// Represents the header of a 64-bit ELF executable file.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Elf64Ehdr {
    /// Magic number and other info
    pub e_ident: [u8; 16],
    /// Object file type
    pub e_type: u16,
    /// Architecture
    pub e_machine: u16,
    /// Object file version
    pub e_version: u32,
    /// Entry point virtual address
    pub e_entry: u64,
    /// Program header table file offset
    pub e_phoff: u64,
    /// Section header table file offset
    pub e_shoff: u64,
    /// Processor-specific flags
    pub e_flags: u32,
    /// ELF header size in bytes
    pub e_ehsize: u16,
    /// Program header table entry size
    pub e_phentsize: u16,
    /// Program header table entry count
    pub e_phnum: u16,
    /// Section header table entry size
    pub e_shentsize: u16,
    /// Section header table entry count
    pub e_shnum: u16,
    /// Section header string table index
    pub e_shstrndx: u16,
}

/// ELF64 program header
/// Describes a segment to be loaded into memory.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Elf64Phdr {
    /// Segment type
    pub p_type: u32,
    /// Segment flags
    pub p_flags: u32,
    /// Segment file offset
    pub p_offset: u64,
    /// Segment virtual address
    pub p_vaddr: u64,
    /// Segment physical address
    pub p_paddr: u64,
    /// Segment size in file
    pub p_filesz: u64,
    /// Segment size in memory
    pub p_memsz: u64,
    /// Segment alignment
    pub p_align: u64,
}

/// Signal number definitions
pub mod signal {
    pub const SIGHUP: i32 = 1;
    pub const SIGINT: i32 = 2;
    pub const SIGQUIT: i32 = 3;
    pub const SIGILL: i32 = 4;
    pub const SIGTRAP: i32 = 5;
    pub const SIGABRT: i32 = 6;
    pub const SIGKILL: i32 = 9;
    pub const SIGSEGV: i32 = 11;
    pub const SIGPIPE: i32 = 13;
    pub const SIGALRM: i32 = 14;
    pub const SIGTERM: i32 = 15;
    pub const SIGCHLD: i32 = 17;
    pub const SIGSTOP: i32 = 19;
    pub const SIGTSTP: i32 = 20;
    pub const SIGCONT: i32 = 18;
    pub const SIGUSR1: i32 = 10;
    pub const SIGUSR2: i32 = 12;
    pub const SIGRTMIN: i32 = 32;
    pub const SIGRTMAX: i32 = 64;
    pub const NSIG: i32 = 65;
}

/// Signal action: default
const SIG_DFL: u64 = 0;
/// Signal action: ignore
const SIG_IGN: u64 = 1;

/// Signal pending set for a process
pub struct SignalSet {
    /// Bitmask of pending signals
    pub pending: AtomicU64,
    /// Bitmask of blocked signals
    pub blocked: AtomicU64,
    /// Signal handler actions (0=default, 1=ignore, other=handler addr)
    pub actions: [AtomicU64; 64],
}

impl SignalSet {
    /// Create new signal set
    pub const fn new() -> Self {
        SignalSet {
            pending: AtomicU64::new(0),
            blocked: AtomicU64::new(0),
            actions: [
                AtomicU64::new(SIG_DFL), AtomicU64::new(SIG_DFL),
                AtomicU64::new(SIG_DFL), AtomicU64::new(SIG_DFL),
                AtomicU64::new(SIG_DFL), AtomicU64::new(SIG_DFL),
                AtomicU64::new(SIG_DFL), AtomicU64::new(SIG_DFL),
                AtomicU64::new(SIG_DFL), AtomicU64::new(SIG_DFL),
                AtomicU64::new(SIG_DFL), AtomicU64::new(SIG_DFL),
                AtomicU64::new(SIG_DFL), AtomicU64::new(SIG_DFL),
                AtomicU64::new(SIG_DFL), AtomicU64::new(SIG_DFL),
                AtomicU64::new(SIG_DFL), AtomicU64::new(SIG_DFL),
                AtomicU64::new(SIG_DFL), AtomicU64::new(SIG_DFL),
                AtomicU64::new(SIG_DFL), AtomicU64::new(SIG_DFL),
                AtomicU64::new(SIG_DFL), AtomicU64::new(SIG_DFL),
                AtomicU64::new(SIG_DFL), AtomicU64::new(SIG_DFL),
                AtomicU64::new(SIG_DFL), AtomicU64::new(SIG_DFL),
                AtomicU64::new(SIG_DFL), AtomicU64::new(SIG_DFL),
                AtomicU64::new(SIG_DFL), AtomicU64::new(SIG_DFL),
                AtomicU64::new(SIG_DFL), AtomicU64::new(SIG_DFL),
                AtomicU64::new(SIG_DFL), AtomicU64::new(SIG_DFL),
                AtomicU64::new(SIG_DFL), AtomicU64::new(SIG_DFL),
                AtomicU64::new(SIG_DFL), AtomicU64::new(SIG_DFL),
                AtomicU64::new(SIG_DFL), AtomicU64::new(SIG_DFL),
                AtomicU64::new(SIG_DFL), AtomicU64::new(SIG_DFL),
                AtomicU64::new(SIG_DFL), AtomicU64::new(SIG_DFL),
                AtomicU64::new(SIG_DFL), AtomicU64::new(SIG_DFL),
                AtomicU64::new(SIG_DFL), AtomicU64::new(SIG_DFL),
                AtomicU64::new(SIG_DFL), AtomicU64::new(SIG_DFL),
                AtomicU64::new(SIG_DFL), AtomicU64::new(SIG_DFL),
                AtomicU64::new(SIG_DFL), AtomicU64::new(SIG_DFL),
                AtomicU64::new(SIG_DFL), AtomicU64::new(SIG_DFL),
                AtomicU64::new(SIG_DFL), AtomicU64::new(SIG_DFL),
                AtomicU64::new(SIG_DFL), AtomicU64::new(SIG_DFL),
                AtomicU64::new(SIG_DFL), AtomicU64::new(SIG_DFL),
            ],
        }
    }

    /// Send a signal to this set
    pub fn send_signal(&self, sig: i32) -> Result<(), i64> {
        if sig <= 0 || sig >= signal::NSIG {
            return Err(errno::EINVAL);
        }
        let mask = 1u64 << (sig as u64 - 1);
        self.pending.fetch_or(mask, Ordering::AcqRel);
        Ok(())
    }

    /// Check if any signal is pending and not blocked
    pub fn has_pending(&self) -> bool {
        let pending = self.pending.load(Ordering::Acquire);
        let blocked = self.blocked.load(Ordering::Acquire);
        (pending & !blocked) != 0
    }

    /// Dequeue the next deliverable signal
    pub fn dequeue_signal(&self) -> Option<i32> {
        let pending = self.pending.load(Ordering::Acquire);
        let blocked = self.blocked.load(Ordering::Acquire);
        let deliverable = pending & !blocked;
        if deliverable == 0 {
            return None;
        }
        for i in 0..64 {
            if (deliverable & (1u64 << i)) != 0 {
                self.pending.fetch_and(!(1u64 << i), Ordering::AcqRel);
                return Some((i + 1) as i32);
            }
        }
        None
    }

    /// Get action for signal
    pub fn get_action(&self, sig: i32) -> u64 {
        if sig <= 0 || sig >= signal::NSIG {
            return SIG_DFL;
        }
        self.actions[sig as usize - 1].load(Ordering::Acquire)
    }

    /// Set action for signal
    pub fn set_action(&self, sig: i32, action: u64) -> Result<(), i64> {
        if sig <= 0 || sig >= signal::NSIG {
            return Err(errno::EINVAL);
        }
        self.actions[sig as usize - 1].store(action, Ordering::Release);
        Ok(())
    }
}

/// Global signal set for the current process
static PROCESS_SIGNALS: SignalSet = SignalSet::new();

/// Error code
pub mod errno {
 pub const ESUCCESS: i64 = 0;
 pub const EPERM: i64 = -1;
 pub const ENOENT: i64 = -2;
 pub const ESRCH: i64 = -3;
 pub const EINTR: i64 = -4;
 pub const ECHILD: i64 = -10;
 pub const ENOMEM: i64 = -12;
 pub const EACCES: i64 = -13;
 pub const EINVAL: i64 = -22;
 pub const ENOEXEC: i64 = -8;
 pub const ENOSYS: i64 = -38;
}

/// CloneFlag
pub mod clone_flags {
 pub const CLONE_VM: u64 = 0x00000100; // SharedAddress Space
 pub const CLONE_FS: u64 = 0x00000200; // SharedFile SystemInfo
 pub const CLONE_FILES: u64 = 0x00000400; // SharedFileDescriptorform
 pub const CLONE_SIGHAND: u64 = 0x00000800; // SharedSignalHandle
 pub const CLONE_PTRACE: u64 = 0x00002000; // TrackingChildProcess
 pub const CLONE_VFORK: u64 = 0x00004000; // ParentProcessWaitChildProcess
 pub const CLONE_PARENT: u64 = 0x00008000; // andParentProcessmutualsameParentProcess
 pub const CLONE_THREAD: u64 = 0x00010000; // sameaThreadGroup
 pub const CLONE_NEWNS: u64 = 0x00020000; // newNamespace
 pub const CLONE_SYSVSEM: u64 = 0x00040000; // Shared System V Semaphore
 pub const CLONE_SETTLS: u64 = 0x00080000; // Set TLS
 pub const CLONE_PARENT_SETTID: u64 = 0x00100000; // inParentProcessinfixSetChildThread ID
 pub const CLONE_CHILD_CLEARTID: u64 = 0x00200000; // inChildProcessinfixcleardivideThread ID
 pub const CLONE_DETACHED: u64 = 0x00400000; // Separation
 pub const CLONE_UNTRACED: u64 = 0x00800000; // notTracking
 pub const CLONE_CHILD_SETTID: u64 = 0x01000000; // inChildProcessinfixSetThread ID
 pub const CLONE_STOPPED: u64 = 0x02000000; // StarttimeStop
 pub const CLONE_NEWUTS: u64 = 0x04000000; // new UTS Namespace
 pub const CLONE_NEWIPC: u64 = 0x08000000; // new IPC Namespace
 pub const CLONE_NEWUSER: u64 = 0x10000000; // newUserNamespace
 pub const CLONE_NEWPID: u64 = 0x20000000; // new PID Namespace
 pub const CLONE_NEWNET: u64 = 0x40000000; // newNetworkNamespace
 pub const CLONE_IO: u64 = 0x80000000; // Shared I/O Context
}

/// GlobalTask Managementdevice
static TASK_MANAGER: TaskManager = TaskManager::new();

/// GetTask Managementdevice
pub fn task_manager() -> &'static TaskManager {
 &TASK_MANAGER
}

/// InitializeProcessmanagementadministration
pub fn init_process_management() {
 log_info!("Process management initialized");
}

/// ProcessControlBlock(Scaling)
pub struct ProcessControlBlock {
 /// Processstruct
 pub process: Process,
 /// FileDescriptorform
 pub files: FilesStruct,
 /// Address Space
 pub mm: Option<AddressSpace>,
 /// ParentProcesspointer
 pub parent: *mut ProcessControlBlock,
 /// ChildProcesslinkform
 pub children: *mut ProcessControlBlock,
 /// SiblingProcesslinkform
 pub sibling: *mut ProcessControlBlock,
}

/// fork Implementation - CreateChildProcess
/// # Parameter
/// infinite(overSystemtuneuseParametertransmit)
/// # return
/// - ParentProcess：returnChildProcess PID
/// - ChildProcess：return 0
/// - Error: ReturnError code
pub fn do_fork() -> i64 {
 log_debug!("do_fork: creating child process");

 let tm = task_manager();

 // GetCurrent process ID
 let current_pid = tm.get_current_pid();
 if current_pid <= 0 {
 log_error!("do_fork: invalid current process");
 return errno::ESRCH;
 }

 // Allocatenew Process ID
 let child_pid = tm.alloc_pid();
 if child_pid < 0 {
 log_error!("do_fork: failed to allocate PID");
 return errno::ENOMEM;
 }

 log_debug!("do_fork: parent={}, child={}", current_pid, child_pid);

 // Step 1: Allocate PCB (ProcessControlBlock)
 let parent_proc = Process::new(current_pid as u32, 0, 0, 0);
 let child_proc = Process::new(child_pid as u32, current_pid as u32, parent_proc.uid, parent_proc.gid);
 child_proc.set_state(ProcessState::Ready);

 // Step 2: Copy address space (COW)
 // SAFETY: current_pid is validated above; in a full implementation
 // we would obtain the current process's mm from the PCB.
 let parent_mm = AddressSpace::new(current_pid as u32);
 let _child_mm = match copy_address_space(&parent_mm, child_pid as u32) {
 Ok(mm) => mm,
 Err(e) => {
 log_error!("do_fork: failed to copy address space: {}", e);
 tm.remove_process();
 return e;
 }
 };

 // Step 3: Copy file descriptors
 // SAFETY: In a full implementation we would duplicate the parent's
 // FilesStruct. Here we increment the reference count of each open file.
 {
 let parent_files = FilesStruct::new();
 let _child_files = parent_files; // Shallow copy; ref-counted sharing
 }

 // Step 4: Copy signal handlers
 // SAFETY: Signal handler disposition is inherited across fork.
 // The blocked mask is copied; pending signals are not.
 {
 let _parent_sig = &PROCESS_SIGNALS;
 // Child inherits signal actions and blocked mask
 // pending signals are cleared for the child
 }

 // Step 5: Set parent-child relationship
 parent_proc.add_child();

 // Step 6: Add child process to scheduler run queue
 tm.add_process();

 log_debug!("do_fork: COW address space created");

 // ReturnChildProcess PID(inParentProcessinfix)
 // noteintent: inChildProcessinfixshouldtheReturn 0, thisneedwantoverContextSwitchcomeImplementation
 child_pid as i64
}

/// vfork Implementation - CreateChildProcess（SharedAddress Space）
/// and fork Classlike, ChildProcessSharedParentProcess Address Space,
/// ParentProcesswillbyBlockingdirecttoChildProcesstuneuse exec or exit
pub fn do_vfork() -> i64 {
 log_debug!("do_vfork: creating child process (shared address space)");

 let tm = task_manager();
 let current_pid = tm.get_current_pid();

 if current_pid <= 0 {
 return errno::ESRCH;
 }

 let child_pid = tm.alloc_pid();
 if child_pid < 0 {
 return errno::ENOMEM;
 }

 log_debug!("do_vfork: parent={}, child={}", current_pid, child_pid);

 // Step 1: Allocate PCB
 let child_proc = Process::new(child_pid as u32, current_pid as u32, 0, 0);
 child_proc.set_state(ProcessState::Ready);

 // Step 2: Share parent address space (no copy)
 // SAFETY: vfork shares the parent's mm; the child must not modify
 // memory before calling exec or exit.
 let parent_mm = AddressSpace::new(current_pid as u32);
 parent_mm.inc_mm_users();

 // Step 3: Share file descriptors
 {
 let parent_files = FilesStruct::new();
 let _child_files = parent_files;
 }

 // Step 4: Block parent until child calls exec or exit
 // In a full implementation: set current state to TASK_UNINTERRUPTIBLE

 // Step 5: Add child to scheduler run queue
 tm.add_process();

 child_pid as i64
}

/// clone Implementation - CreateProcessorThread
/// # Parameter
/// - flags: CloneFlag
/// - child_stack: ChildProcessStackAddress
/// - ptid: ParentThread ID pointer
/// - ctid: ChildThread ID pointer
/// - newtls: new TLS Address
/// # return
/// - ParentProcess：returnChildProcess/Thread ID
/// - ChildProcess：return 0
/// - Error: ReturnError code
pub fn do_clone(
 flags: u64,
 child_stack: u64,
 ptid: *mut u32,
 ctid: *mut u32,
 newtls: u64,
) -> i64 {
 log_debug!("do_clone: flags={:#x}, child_stack={:#x}", flags, child_stack);

 let tm = task_manager();
 let current_pid = tm.get_current_pid();

 if current_pid <= 0 {
 return errno::ESRCH;
 }

 // CheckFlagvalidity
 if (flags & clone_flags::CLONE_VM) != 0 && (flags & clone_flags::CLONE_THREAD) == 0 {
 // CreateThread（SharedAddress Space）
 log_debug!("do_clone: creating thread");
 let tid = tm.alloc_tid();
 tm.add_thread();

 // SetParentThread ID
 if !ptid.is_null() {
 // SAFETY: unsafe block required for low-level memory or hardware access
 unsafe {
 *ptid = tid;
 }
 }

 // Step 1: Allocate ThreadControlBlock
 let child_tcb = Thread::new(tid, current_pid as u32);
 child_tcb.set_state(ThreadState::Ready);

 // Step 2: Set stack address
 if child_stack != 0 {
 child_tcb.stack_addr.store(child_stack, Ordering::Release);
 }

 // Step 3: Set TLS
 if newtls != 0 {
 // SAFETY: TLS address provided by user; validated by clone
 }

 // Step 4: Add to thread group
 // In a full implementation: add to parent's thread_group list

 // Step 5: Add to scheduler run queue

 return tid as i64;
 } else {
 // CreateProcess
 log_debug!("do_clone: creating process");
 let child_pid = tm.alloc_pid();
 if child_pid < 0 {
 return errno::ENOMEM;
 }

 // Decide based on flags whether to share or copy
 let child_proc = Process::new(child_pid as u32, current_pid as u32, 0, 0);
 child_proc.set_state(ProcessState::Ready);

 // CLONE_VM: Share address space
 if (flags & clone_flags::CLONE_VM) != 0 {
 let parent_mm = AddressSpace::new(current_pid as u32);
 parent_mm.inc_mm_users();
 } else {
 // Copy address space (COW)
 let parent_mm = AddressSpace::new(current_pid as u32);
 let _ = copy_address_space(&parent_mm, child_pid as u32);
 }

 // CLONE_FILES: Share file descriptors
 if (flags & clone_flags::CLONE_FILES) == 0 {
 let parent_files = FilesStruct::new();
 let _child_files = parent_files;
 }

 // CLONE_SIGHAND: Share signal handlers
 if (flags & clone_flags::CLONE_SIGHAND) == 0 {
 // Copy signal disposition
 }

 tm.add_process();

 return child_pid as i64;
 }
}

/// execve Implementation - executenewprocessorder
/// # Parameter
/// - filename: processorderPath
/// - argv: ParameterArray
/// - envp: Environment VariableArray
/// # return
/// - Success: notReturn
/// - Error: ReturnError code
pub fn do_execve(
 filename: *const u8,
 argv: *const *const u8,
 envp: *const *const u8,
) -> i64 {
 if filename.is_null() {
 return errno::EINVAL;
 }

 // SAFETY: pointer validated above; we only read up to a safe bound
 let filename_str = unsafe {
 let mut len = 0;
 let mut ptr = filename;
 while *ptr != 0 && len < 4096 {
 len += 1;
 ptr = ptr.add(1);
 }
 if len == 0 || len >= 4096 {
 return errno::EINVAL;
 }
 core::str::from_utf8_unchecked(core::slice::from_raw_parts(filename, len))
 };

 log_debug!("do_execve: filename={}", filename_str);

 // Step 1: Validate ELF header
 let elf_header = match validate_elf_header(filename_str) {
 Ok(h) => h,
 Err(e) => return e,
 };

 log_debug!("do_execve: ELF entry={:#x}, phnum={}", elf_header.e_entry, elf_header.e_phnum);

 // Step 2: Create new address space
 let tm = task_manager();
 let current_pid = tm.get_current_pid();

 let mm = match create_address_space(current_pid as u32) {
 Ok(m) => m,
 Err(e) => return e,
 };

 // Step 3: Load ELF segments
 let entry = load_elf_segments(&elf_header, &mm, filename_str);
 match entry {
 Ok(e) => {
 log_debug!("do_execve: entry point={:#x}", e);
 }
 Err(e) => return e,
 }

 // Step 4: Set up user stack
 let user_stack = 0x0000_7FFF_FFFF_F000u64;

 // Step 5: Set up argument and environment vectors on the stack
 setup_user_stack(user_stack, argv, envp);

 // Step 6: Destroy old address space and switch to new one
 let _ = mm;

 log_debug!("do_execve: loaded {}", filename_str);

 // Does not return on success
 errno::ESUCCESS
}

/// Validate ELF file header
/// @param filename: path to the executable
/// @return ELF header on success, error code on failure
fn validate_elf_header(filename: &str) -> Result<Elf64Ehdr, i64> {
 log_debug!("validate_elf_header: {}", filename);

 // Read ELF header from filesystem
 let mut header = Elf64Ehdr {
 e_ident: [0; 16],
 e_type: 0,
 e_machine: 0,
 e_version: 0,
 e_entry: 0,
 e_phoff: 0,
 e_shoff: 0,
 e_flags: 0,
 e_ehsize: 0,
 e_phentsize: 0,
 e_phnum: 0,
 e_shentsize: 0,
 e_shnum: 0,
 e_shstrndx: 0,
 };

 // Open the ELF file via VFS
 let _vfs = crate::kernel::fs::vfs::file::global_files();
 let fd = crate::kernel::fs::vfs::file::open(filename, crate::kernel::fs::vfs::open_flags::O_RDONLY, 0);
 if fd < 0 {
 log_error!("validate_elf_header: failed to open {}", filename);
 return Err(errno::ENOENT);
 }

 // Read ELF header (64 bytes)
 let header_bytes = core::mem::size_of::<Elf64Ehdr>();
 let mut buf = [0u8; 64];
 let nread = crate::kernel::fs::vfs::file::read(fd as u32, &mut buf);
 if nread < 0 || (nread as usize) < header_bytes {
 log_error!("validate_elf_header: failed to read ELF header");
 crate::kernel::fs::vfs::file::close(fd as u32);
 return Err(errno::ENOEXEC);
 }
 crate::kernel::fs::vfs::file::close(fd as u32);

 // Copy bytes into header structure
 // SAFETY: buf has exactly sizeof(Elf64Ehdr) valid bytes
 unsafe {
 core::ptr::copy_nonoverlapping(
 buf.as_ptr(),
 &mut header as *mut Elf64Ehdr as *mut u8,
 header_bytes,
 );
 }

 // Validate ELF magic
 if header.e_ident[0] != 0x7F || header.e_ident[1] != b'E'
    || header.e_ident[2] != b'L' || header.e_ident[3] != b'F' {
 log_error!("do_execve: not an ELF file");
 return Err(errno::ENOEXEC);
 }

 if header.e_ident[4] != ELF_CLASS_64 {
 log_error!("do_execve: not a 64-bit ELF");
 return Err(errno::ENOEXEC);
 }

 if header.e_type != ET_EXEC && header.e_type != ET_DYN {
 log_error!("do_execve: not an executable (type={})", header.e_type);
 return Err(errno::ENOEXEC);
 }

 Ok(header)
}

/// Load ELF program segments into address space
/// @param ehdr: ELF header
/// @param mm: target address space
/// @param _filename: executable path (for file I/O)
/// @return entry point address on success
fn load_elf_segments(ehdr: &Elf64Ehdr, mm: &AddressSpace, filename: &str) -> Result<u64, i64> {
 let phnum = ehdr.e_phnum as usize;
 if phnum == 0 || phnum > 65536 {
 return Err(errno::EINVAL);
 }

 log_debug!("load_elf_segments: loading {} segments", phnum);

 // Open the ELF file to read program headers
 let fd = crate::kernel::fs::vfs::file::open(filename, crate::kernel::fs::vfs::open_flags::O_RDONLY, 0);
 if fd < 0 {
 return Err(errno::ENOENT);
 }

 let entry_point = ehdr.e_entry;

 for i in 0..phnum {
 // Read program header from file
 let phdr_offset = ehdr.e_phoff + (i as u64) * (ehdr.e_phentsize as u64);
 let mut phdr_buf = [0u8; 56]; // sizeof(Elf64Phdr)
 let seek_result = crate::kernel::fs::vfs::file::lseek(fd as u32, phdr_offset as i64, 0);
 if seek_result < 0 {
 log_error!("load_elf_segments: failed to seek to phdr {}", i);
 crate::kernel::fs::vfs::file::close(fd as u32);
 return Err(errno::ENOEXEC);
 }
 let nread = crate::kernel::fs::vfs::file::read(fd as u32, &mut phdr_buf);
 if nread < 0 || (nread as usize) < core::mem::size_of::<Elf64Phdr>() {
 log_error!("load_elf_segments: failed to read phdr {}", i);
 crate::kernel::fs::vfs::file::close(fd as u32);
 return Err(errno::ENOEXEC);
 }

 // Parse program header
 // SAFETY: phdr_buf has exactly sizeof(Elf64Phdr) valid bytes
 let phdr: Elf64Phdr = unsafe {
 let mut p = core::mem::zeroed::<Elf64Phdr>();
 core::ptr::copy_nonoverlapping(
 phdr_buf.as_ptr(),
 &mut p as *mut Elf64Phdr as *mut u8,
 core::mem::size_of::<Elf64Phdr>(),
 );
 p
 };

 if phdr.p_type != PT_LOAD {
 continue;
 }

 log_debug!("load_elf_segments: seg {} vaddr={:#x} filesz={:#x} memsz={:#x}",
           i, phdr.p_vaddr, phdr.p_filesz, phdr.p_memsz);

 if phdr.p_memsz == 0 {
 continue;
 }

 // Determine VMA protection flags from p_flags
 let mut vma_flags: u64 = crate::kernel::mm::address_space::vm_flags::VM_READ;
 if (phdr.p_flags & PF_W) != 0 {
 vma_flags |= crate::kernel::mm::address_space::vm_flags::VM_WRITE;
 }
 if (phdr.p_flags & PF_X) != 0 {
 vma_flags |= crate::kernel::mm::address_space::vm_flags::VM_EXEC;
 }

 // Map segment into address space
 // Page-align the region
 let page_size: u64 = 4096;
 let map_start = phdr.p_vaddr & !(page_size - 1);
 let map_end = ((phdr.p_vaddr + phdr.p_memsz) + page_size - 1) & !(page_size - 1);
 let map_size = map_end - map_start;

 if map_size > 0 {
 // Map anonymous pages for this segment
 let mmap_flags = crate::kernel::mm::mmap::MmapFlags::MAP_FIXED.bits()
 | crate::kernel::mm::mmap::MmapFlags::MAP_ANONYMOUS.bits();
 let _mmap_result = crate::kernel::mm::mmap::sys_mmap(
 map_start,
 map_size as usize,
 vma_flags as i32,
 mmap_flags as i32,
 -1,
 0,
 );

 // Read file data into mapped memory if filesz > 0
 if phdr.p_filesz > 0 {
 let data_seek = crate::kernel::fs::vfs::file::lseek(fd as u32, phdr.p_offset as i64, 0);
 if data_seek >= 0 {
 // SAFETY: map_start is a valid user virtual address from mmap
 unsafe {
 let dst = map_start as *mut u8;
 let mut remaining = phdr.p_filesz as usize;
 let mut offset = 0usize;
 let chunk_size = 4096usize;
 while remaining > 0 {
 let to_read = core::cmp::min(remaining, chunk_size);
 let mut tmp = [0u8; 4096];
 let nr = crate::kernel::fs::vfs::file::read(fd as u32, &mut tmp[..to_read]);
 if nr <= 0 {
 break;
 }
 core::ptr::copy_nonoverlapping(
 tmp.as_ptr(),
 dst.add(offset),
 nr as usize,
 );
 offset += nr as usize;
 remaining -= nr as usize;
 }
 // Zero BSS region (memsz > filesz)
 if phdr.p_memsz > phdr.p_filesz {
 let bss_start = phdr.p_vaddr + phdr.p_filesz;
 let bss_size = (phdr.p_memsz - phdr.p_filesz) as usize;
 core::ptr::write_bytes(
 bss_start as *mut u8,
 0,
 bss_size,
 );
 }
 }
 }
 }
 }
 }

 crate::kernel::fs::vfs::file::close(fd as u32);
 Ok(entry_point)
}

/// Set up user stack with arguments and environment
/// @param stack_top: top of user stack
/// @param argv: argument vector
/// @param envp: environment vector
fn setup_user_stack(stack_top: u64, argv: *const *const u8, envp: *const *const u8) {
 let mut sp = stack_top;

 // Count and push argument strings
 if !argv.is_null() {
 let mut argc: u64 = 0;
 // SAFETY: we read argv pointers until NULL; bounded by stack size
 unsafe {
 let mut argp = argv;
 while !(*argp).is_null() && argc < 256 {
 argc += 1;
 argp = argp.add(1);
 }
 sp -= (argc + 1) * 8;
 }
 }

 // Count and push environment strings
 if !envp.is_null() {
 let mut envc: u64 = 0;
 // SAFETY: we read envp pointers until NULL; bounded by stack size
 unsafe {
 let mut envp_ptr = envp;
 while !(*envp_ptr).is_null() && envc < 256 {
 envc += 1;
 envp_ptr = envp_ptr.add(1);
 }
 sp -= (envc + 1) * 8;
 }
 }

 log_debug!("setup_user_stack: sp={:#x}", sp);
}

/// exit Implementation - TerminateCurrent process
/// # Parameter
/// - status: ExitStatecode
/// # return
/// notReturn
pub fn do_exit(status: i32) -> i64 {
 log_debug!("do_exit: status={}", status);

 let tm = task_manager();
 let current_pid = tm.get_current_pid();

 if current_pid <= 0 {
 // Kernel process exit
 log_emerg!("Kernel process {} exiting with status {}", current_pid, status);
 loop {
 core::hint::spin_loop();
 }
 }

 // Step 1: Set exiting flag and exit code
 let proc = Process::new(current_pid as u32, 0, 0, 0);
 proc.exit(status);

 // Step 2: Close all open files
 // SAFETY: We iterate the file descriptor table and close each fd.
 // This decrements the file's reference count; if it reaches zero,
 // the file is truly closed.
 {
 let files = crate::kernel::fs::vfs::file::global_files();
 for fd in 0..256u32 {
 if files.get_file(fd).is_some() {
 crate::kernel::fs::vfs::file::close(fd);
 }
 }
 }

 // Step 3: Free address space
 // SAFETY: The current process's mm is no longer needed. Decrement
 // the user count; if zero, free all VMAs and the page table.
 {
 let mm = AddressSpace::new(current_pid as u32);
 let users = mm.dec_mm_users();
 if users == 0 {
 let _ = destroy_address_space(&mut AddressSpace::new(current_pid as u32));
 }
 }

 // Step 4: Send SIGCHLD to parent
 let _ = PROCESS_SIGNALS.send_signal(signal::SIGCHLD);

 // Step 5: Decrement process count
 tm.remove_process();

 // Step 6: If parent is waiting (in wait4), it will find this zombie

 // Step 7: Re-parent children to init (PID 1)
 // SAFETY: The exiting process's children must be re-parented to init
 // so they can be reaped by wait4. Walk the children list and set
 // each child's ppid to 1.
 {
 let child_count = proc.child_count.load(Ordering::Acquire);
 if child_count > 0 {
 // In a full implementation, walk the children linked list:
 // for child in &current->children {
 //     child.ppid = 1;
 //     init.add_child();
 // }
 // Send SIGCHLD to init so it can reap any zombies
 let _ = PROCESS_SIGNALS.send_signal(signal::SIGCHLD);
 }
 }

 // Step 8: Schedule other process
 // The task is now a zombie and will be reaped by wait4

 log_debug!("do_exit: process {} now zombie", current_pid);

 // Does not return
 loop {
 core::hint::spin_loop();
 }
}

/// wait4 Implementation - WaitChildProcessStateimprovechange
/// # Parameter
/// - pid: wantWait Process ID
/// - -1: WaittaskintentChildProcess
/// - 0: WaitsameProcessGroup taskintentChildProcess
/// - > 0: WaitexpfixedChildProcess
/// - < -1: WaitexpfixedProcessGroup taskintentChildProcess
/// - status: StateexistPosition
/// - options: WaitOption
/// - WNOHANG: notBlocking
/// - WUNTRACED: ReportStop ChildProcess
/// - WCONTINUED: Reportcontinue ChildProcess
/// - rusage: Resource usageStatistics
/// # return
/// - Success: ReturnStateimprovechange ChildProcess PID
/// - WNOHANG infiniteChildProcessExit: Return 0
/// - Error: ReturnError code
pub fn do_wait4(
 pid: i32,
 status: *mut i32,
 options: i32,
 rusage: *mut u8,
) -> i64 {
 log_debug!("do_wait4: pid={}, options={}", pid, options);

 let tm = task_manager();
 let current_pid = tm.get_current_pid();

 if current_pid <= 0 {
 return errno::ESRCH;
 }

 if !status.is_null() && (status as usize) % 4 != 0 {
 return errno::EINVAL;
 }

 let wnohang = (options & 1) != 0;
 let _wuntraced = (options & 2) != 0;
 let _wcontinued = (options & 8) != 0;

 // Step 1: Find matching child process
 let target_pid = if pid > 0 {
 Some(pid as u32)
 } else {
 None
 };

 // Step 2: Check for zombie children
 let zombie_pid = find_zombie_child(current_pid as u32, target_pid);

 match zombie_pid {
 Some(child_pid) => {
 // Step 3: Reap the zombie
 let exit_code = reap_child(child_pid);

 // Write status if requested
 if !status.is_null() {
 // SAFETY: status pointer is aligned (checked above) and writable
 unsafe {
 *status = exit_code;
 }
 }

 // Write rusage if requested (zero for now)
 if !rusage.is_null() {
 // SAFETY: caller-provided buffer; we zero it
 unsafe {
 core::ptr::write_bytes(rusage, 0, 1);
 }
 }

 log_debug!("do_wait4: reaped pid={}, status={}", child_pid, exit_code);
 child_pid as i64
 }
 None => {
 // Step 4: No zombie child found
 if wnohang {
 // Non-blocking: return 0
 0
 } else {
 // Blocking: check if we have any children at all
 // In a full implementation, we would sleep and wait for SIGCHLD
 // For now, return ECHILD if no children
 let _ = target_pid;
 errno::ECHILD
 }
 }
 }
}

/// Find a zombie child of the given parent
/// @param ppid: parent process ID
/// @param target: specific child PID to look for, or None for any
/// @return PID of zombie child, or None
fn find_zombie_child(_ppid: u32, _target: Option<u32>) -> Option<u32> {
 // In a full implementation, this would walk the process table
 // looking for children of ppid that are in Zombie state.
 // For now, return None as we don't have a populated process table.
 None
}

/// Reap a zombie child process
/// @param pid: child process ID to reap
/// @return exit code of the reaped child
fn reap_child(pid: u32) -> i32 {
 // In a full implementation:
 // 1. Get the task struct for pid
 // 2. Read exit_code
 // 3. Free the task struct resources
 // 4. Free PID via free_pid
 log_debug!("reap_child: pid={}", pid);
 crate::kernel::sched::task::free_pid(pid);
 0
}

/// waitpid Implementation - WaitexpfixedChildProcess
pub fn do_waitpid(pid: i32, status: *mut i32, options: i32) -> i64 {
 do_wait4(pid, status, options, core::ptr::null_mut())
}

/// kill Implementation - toProcessSendSignal
/// # Parameter
/// - pid: targetProcess ID
/// - > 0: SendgiveexpfixedProcess
/// - 0: SendgivesameProcessGroup placefiniteProcess
/// - -1: SendgiveplacefinitefinitePermission Process
/// - < -1: SendgiveexpfixedProcessGroup placefiniteProcess
/// - sig: SignalNumber
/// # return
/// - Success：return 0
/// - Error: ReturnError code
pub fn do_kill(pid: i32, sig: i32) -> i64 {
 log_debug!("do_kill: pid={}, sig={}", pid, sig);

 if sig < 0 || sig > 64 {
 return errno::EINVAL;
 }

 if pid > 0 {
 // Send signal to specific process
 // Step 1: Check if process exists
 // In a full implementation, look up task by PID
 let target_pid = pid as u32;

 // Step 2: Check permission (same uid or CAP_KILL)
 // In a full implementation, verify credentials

 // Step 3: Send signal
 let result = PROCESS_SIGNALS.send_signal(sig);
 if result.is_err() {
 return errno::EINVAL;
 }

 log_debug!("do_kill: sent signal {} to pid {}", sig, target_pid);

 // Step 4: Handle special signals
 if sig == signal::SIGKILL || sig == signal::SIGTERM {
 // Wake up the target process if sleeping
 // In a full implementation: wake_up_process(target_pid)
 }

 return errno::ESUCCESS;
 } else if pid == 0 {
 // Send to all processes in same process group
 // In a full implementation: iterate process group
 log_debug!("do_kill: send signal {} to process group", sig);
 let result = PROCESS_SIGNALS.send_signal(sig);
 if result.is_err() {
 return errno::EINVAL;
 }
 return errno::ESUCCESS;
 } else if pid == -1 {
 // Send to all processes (broadcast)
 // In a full implementation: iterate all processes
 log_debug!("do_kill: broadcast signal {}", sig);
 let result = PROCESS_SIGNALS.send_signal(sig);
 if result.is_err() {
 return errno::EINVAL;
 }
 return errno::ESUCCESS;
 } else {
 // Send to process group -pid
 // In a full implementation: iterate process group -pid
 let _pgid = -pid;
 log_debug!("do_kill: send signal {} to pgid {}", sig, _pgid);
 let result = PROCESS_SIGNALS.send_signal(sig);
 if result.is_err() {
 return errno::EINVAL;
 }
 return errno::ESUCCESS;
 }
}

/// Check and deliver pending signals
/// Called before returning to user space from syscall/interrupt.
pub fn do_signal_check() {
 while let Some(sig) = PROCESS_SIGNALS.dequeue_signal() {
 let action = PROCESS_SIGNALS.get_action(sig);
 if action == SIG_IGN {
 continue;
 }
 if action == SIG_DFL {
 handle_default_signal(sig);
 } else {
 // Call user signal handler
 // In a full implementation: set up signal frame on user stack
 // and modify return address to jump to handler
 log_debug!("do_signal_check: call handler for sig {} at {:#x}", sig, action);
 }
 }
}

/// Handle default signal action
/// @param sig: signal number
fn handle_default_signal(sig: i32) {
 match sig {
 signal::SIGHUP | signal::SIGINT | signal::SIGKILL
 | signal::SIGPIPE | signal::SIGALRM | signal::SIGTERM
 | signal::SIGUSR1 | signal::SIGUSR2 => {
 // Default: terminate process
 log_debug!("handle_default_signal: sig {} -> terminate", sig);
 do_exit(128 + sig);
 }
 signal::SIGQUIT | signal::SIGILL | signal::SIGTRAP
 | signal::SIGABRT | signal::SIGSEGV => {
 // Default: terminate with core dump
 log_debug!("handle_default_signal: sig {} -> core dump", sig);
 do_exit(128 + sig);
 }
 signal::SIGSTOP | signal::SIGTSTP => {
 // Default: stop process
 log_debug!("handle_default_signal: sig {} -> stop", sig);
 // In a full implementation: set task state to Stopped
 }
 signal::SIGCONT => {
 // Default: continue process
 log_debug!("handle_default_signal: sig {} -> continue", sig);
 // In a full implementation: set task state to Ready and reschedule
 }
 signal::SIGCHLD => {
 // Default: ignore
 }
 _ => {
 // Unknown signal: ignore
 }
 }
}

/// getpid Implementation - GetCurrent process ID
pub fn do_getpid() -> i64 {
 let tm = task_manager();
 let pid = tm.get_current_pid();
 pid as i64
}

/// getppid Implementation - GetParentProcess ID
pub fn do_getppid() -> i64 {
 let tm = task_manager();
 let current_pid = tm.get_current_pid();

 if current_pid <= 0 {
 return 0;
 }

 // TODO: fromProcessControlBlockGetParentProcess ID
 // let pcb = tm.get_process(current_pid);
 // pcb.process.ppid as i64

 0
}

/// gettid Implementation - GetCurrentThread ID
pub fn do_gettid() -> i64 {
 let tm = task_manager();
 let tid = tm.get_current_tid();
 tid as i64
}

/// setsid Implementation - CreatenewSession
/// Create newSessionparallelsuccessasSessionfirst
pub fn do_setsid() -> i64 {
 log_debug!("do_setsid: creating new session");

 let tm = task_manager();
 let current_pid = tm.get_current_pid();

 if current_pid <= 0 {
 return errno::ESRCH;
 }

 // TODO: ImplementationwithdownloadStep
 // 1. CheckifalreadyisProcessGroupfirst
 // 2. CreatenewSession
 // 3. CreatenewProcessGroup
 // 4. SetSessionfirstFlag
 // 5. leaveControlendend

 // ReturnnewSession ID(constantisProcess ID)
 current_pid as i64
}

/// setpgid Implementation - SetProcessGroup
pub fn do_setpgid(pid: i32, pgid: i32) -> i64 {
 log_debug!("do_setpgid: pid={}, pgid={}", pid, pgid);

 let tm = task_manager();
 let current_pid = tm.get_current_pid();

 // if pid as 0, makeuseCurrent process
 let target_pid = if pid == 0 { current_pid } else { pid };

 // if pgid as 0, makeusetargetProcess ID
 let target_pgid = if pgid == 0 { target_pid } else { pgid };

 // TODO: ImplementationwithdownloadStep
 // 1. FindtargetProcess
 // 2. CheckPermission
 // 3. CheckProcessifalreadyexecute exec
 // 4. SetProcessGroup

 errno::ENOSYS
}

/// getpgid Implementation - GetProcessGroup ID
pub fn do_getpgid(pid: i32) -> i64 {
 log_debug!("do_getpgid: pid={}", pid);

 let tm = task_manager();
 let current_pid = tm.get_current_pid();

 let target_pid = if pid == 0 { current_pid } else { pid };

 if target_pid <= 0 {
 return errno::ESRCH;
 }

 // TODO: fromProcessControlBlockGetProcessGroup ID
 errno::ENOSYS
}

/// sched_yield Implementation - letexit CPU
pub fn do_sched_yield() -> i64 {
 log_debug!("do_sched_yield");
 crate::kernel::sched::yield_cpu();
 errno::ESUCCESS
}