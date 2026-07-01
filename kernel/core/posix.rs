use crate::{pr_info};
/*
 * Nuva OS - Kernel - Posix.Rs
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


use core::sync::atomic::{AtomicU64, Ordering};
use crate::syslib::posix::errno::Errno;
use crate::kernel::error::Errno;

/// FileDescriptorType
pub type Fd = i32;

/// Process ID Type
pub type Pid = i32;

/// User ID Type
pub type Uid = u32;

/// Group ID Type
pub type Gid = u32;

/// ModeType
pub type Mode = u32;

/// OffsetquantificationType
pub type Off = i64;

/// SizeType
pub type Size = usize;

/// TimeType
pub type Time = i64;

/// Systemtuneusesignal
#[derive(Debug, Clone, Copy)]
pub enum Syscall {
 // FileOperation
 Open = 0,
 Close = 1,
 Read = 2,
 Write = 3,
 Lseek = 4,
 Fstat = 5,
 Stat = 6,
 Mkdir = 7,
 Rmdir = 8,
 Unlink = 9,
 Rename = 10,
 Chmod = 11,
 Chown = 12,
 Dup = 13,
 Dup2 = 14,
 Pipe = 15,
 Fcntl = 16,
 Fsync = 17,
 Truncate = 18,
 
 // ProcessOperation
 Fork = 20,
 Execve = 21,
 Exit = 22,
 Waitpid = 23,
 Getpid = 24,
 Getppid = 25,
 Getuid = 26,
 Getgid = 27,
 Geteuid = 28,
 Getegid = 29,
 Setuid = 30,
 Setgid = 31,
 Kill = 32,
 Signal = 33,
 Sigaction = 34,
 Sigprocmask = 35,
 Sigpending = 36,
 Sigsuspend = 37,
 Pause = 38,
 Alarm = 39,
 
 // MemoryOperation
 Brk = 40,
 Mmap = 41,
 Munmap = 42,
 Mprotect = 43,
 Msync = 44,
 Mlock = 45,
 Munlock = 46,
 
 // TimeOperation
 Time = 50,
 Gettimeofday = 51,
 Settimeofday = 52,
 Clock_gettime = 53,
 Clock_settime = 54,
 Nanosleep = 55,
 
 // Semaphore
 Semget = 60,
 Semop = 61,
 Semctl = 62,
 
 // SharedMemory
 Shmget = 65,
 Shmat = 66,
 Shmdt = 67,
 Shmctl = 68,
 
 // Message Queue
 Msgget = 70,
 Msgsnd = 71,
 Msgrcv = 72,
 Msgctl = 73,
 
 // Thread
 Clone = 80,
 Futex = 81,
 Set_robust_list = 82,
 Get_robust_list = 83,
 
 // Network
 Socket = 90,
 Bind = 91,
 Listen = 92,
 Accept = 93,
 Connect = 94,
 Send = 95,
 Recv = 96,
 Sendto = 97,
 Recvfrom = 98,
 Shutdown = 99,
 Getsockname = 100,
 Getpeername = 101,
 Setsockopt = 102,
 Getsockopt = 103,
 
 // its
 Ioctl = 110,
 Pread = 111,
 Pwrite = 112,
 Readv = 113,
 Writev = 114,
 Getdents = 115,
 Getcwd = 116,
 Chdir = 117,
 Fchdir = 118,
 Umask = 119,
 Nice = 120,
 Sched_yield = 121,
 Sched_getparam = 122,
 Sched_setparam = 123,
 Sched_getscheduler = 124,
 Sched_setscheduler = 125,
 Sched_get_priority_max = 126,
 Sched_get_priority_min = 127,
}

/// stat struct (deprecated: use FileMetadata for Rust-native, CStat for FFI)
#[deprecated(since = "1.0.0", note = "Use FileMetadata for Rust-native or CStat for FFI")]
#[repr(C)]
pub struct Stat {
 pub device_id: u64,
 pub inode_number: u64,
 pub mode: Mode,
 pub link_count: u32,
 pub user_id: Uid,
 pub group_id: Gid,
 pub raw_device_id: u64,
 pub size: Off,
 pub block_size: i32,
 pub block_count: i64,
 pub access_time: Time,
 pub access_time_nsec: i64,
 pub modification_time: Time,
 pub modification_time_nsec: i64,
 pub change_time: Time,
 pub change_time_nsec: i64,
}

/// File metadata - Rust-idiomatic representation
#[derive(Debug, Clone, Copy)]
pub struct FileMetadata {
    pub device_id: u64,
    pub inode_number: u64,
    pub mode: Mode,
    pub link_count: u32,
    pub user_id: Uid,
    pub group_id: Gid,
    pub raw_device_id: u64,
    pub size: Off,
    pub block_size: i32,
    pub block_count: i64,
    pub access_time: TimeSpec,
    pub modification_time: TimeSpec,
    pub change_time: TimeSpec,
}

/// Time specification - Rust-idiomatic representation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeSpec {
    pub seconds: i64,
    pub nanoseconds: i64,
}

impl TimeSpec {
    pub const ZERO: TimeSpec = TimeSpec { seconds: 0, nanoseconds: 0 };

    pub const fn new(seconds: i64, nanoseconds: i64) -> Self {
        TimeSpec { seconds, nanoseconds }
    }

    pub fn normalize(&mut self) {
        if self.nanoseconds >= 1_000_000_000 {
            self.seconds += self.nanoseconds / 1_000_000_000;
            self.nanoseconds %= 1_000_000_000;
        } else if self.nanoseconds < 0 {
            let borrow = (-self.nanoseconds + 999_999_999) / 1_000_000_000;
            self.seconds -= borrow;
            self.nanoseconds += borrow * 1_000_000_000;
        }
    }
}

/// Signal action - Rust-idiomatic representation
#[derive(Debug, Clone, Copy)]
pub struct SignalAction {
    pub handler: usize,
    pub mask: u64,
    pub flags: u32,
    pub restorer: usize,
}

/// timeval struct
#[repr(C)]
pub struct Timeval {
 pub seconds: Time,
 pub microseconds: i64,
}

/// timespec struct (deprecated: use TimeSpec for Rust-native, CTimeSpec for FFI)
#[deprecated(since = "1.0.0", note = "Use TimeSpec for Rust-native or CTimeSpec for FFI")]
#[repr(C)]
pub struct Timespec {
 pub seconds: Time,
 pub nanoseconds: i64,
}

/// sigaction struct (deprecated: use SignalAction for Rust-native, CSignalAction for FFI)
#[deprecated(since = "1.0.0", note = "Use SignalAction for Rust-native or CSignalAction for FFI")]
#[repr(C)]
pub struct Sigaction {
 pub handler: usize,
 pub signal_mask: u64,
 pub signal_flags: u32,
 pub restorer: usize,
}

// ============================================================================
// FFI Compatibility Layer (#[repr(C)] with POSIX C-style field names)
// ============================================================================

/// C-compatible stat structure (POSIX field names)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CStat {
    pub st_dev: u64,
    pub st_ino: u64,
    pub st_mode: u32,
    pub st_nlink: u32,
    pub st_uid: u32,
    pub st_gid: u32,
    pub st_rdev: u64,
    pub st_size: i64,
    pub st_blksize: i32,
    pub st_blocks: i64,
    pub st_atime: i64,
    pub st_atime_nsec: i64,
    pub st_mtime: i64,
    pub st_mtime_nsec: i64,
    pub st_ctime: i64,
    pub st_ctime_nsec: i64,
}

/// C-compatible timespec structure (POSIX field names)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CTimeSpec {
    pub tv_sec: i64,
    pub tv_nsec: i64,
}

/// C-compatible sigaction structure (POSIX field names)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CSignalAction {
    pub sa_handler: usize,
    pub sa_mask: u64,
    pub sa_flags: u32,
    pub sa_restorer: usize,
}

// ============================================================================
// From trait implementations for FFI conversions
// ============================================================================

impl From<FileMetadata> for CStat {
    fn from(m: FileMetadata) -> Self {
        CStat {
            st_dev: m.device_id,
            st_ino: m.inode_number,
            st_mode: m.mode,
            st_nlink: m.link_count,
            st_uid: m.user_id,
            st_gid: m.group_id,
            st_rdev: m.raw_device_id,
            st_size: m.size,
            st_blksize: m.block_size,
            st_blocks: m.block_count,
            st_atime: m.access_time.seconds,
            st_atime_nsec: m.access_time.nanoseconds,
            st_mtime: m.modification_time.seconds,
            st_mtime_nsec: m.modification_time.nanoseconds,
            st_ctime: m.change_time.seconds,
            st_ctime_nsec: m.change_time.nanoseconds,
        }
    }
}

impl From<CStat> for FileMetadata {
    fn from(s: CStat) -> Self {
        FileMetadata {
            device_id: s.st_dev,
            inode_number: s.st_ino,
            mode: s.st_mode,
            link_count: s.st_nlink,
            user_id: s.st_uid,
            group_id: s.st_gid,
            raw_device_id: s.st_rdev,
            size: s.st_size,
            block_size: s.st_blksize,
            block_count: s.st_blocks,
            access_time: TimeSpec::new(s.st_atime, s.st_atime_nsec),
            modification_time: TimeSpec::new(s.st_mtime, s.st_mtime_nsec),
            change_time: TimeSpec::new(s.st_ctime, s.st_ctime_nsec),
        }
    }
}

impl From<TimeSpec> for CTimeSpec {
    fn from(ts: TimeSpec) -> Self {
        CTimeSpec {
            tv_sec: ts.seconds,
            tv_nsec: ts.nanoseconds,
        }
    }
}

impl From<CTimeSpec> for TimeSpec {
    fn from(cts: CTimeSpec) -> Self {
        TimeSpec::new(cts.tv_sec, cts.tv_nsec)
    }
}

impl From<SignalAction> for CSignalAction {
    fn from(sa: SignalAction) -> Self {
        CSignalAction {
            sa_handler: sa.handler,
            sa_mask: sa.mask,
            sa_flags: sa.flags,
            sa_restorer: sa.restorer,
        }
    }
}

impl From<CSignalAction> for SignalAction {
    fn from(csa: CSignalAction) -> Self {
        SignalAction {
            handler: csa.sa_handler,
            mask: csa.sa_mask,
            flags: csa.sa_flags,
            restorer: csa.sa_restorer,
        }
    }
}

/// POSIX SystemcallCount
pub struct SyscallStats {
 pub total_calls: AtomicU64,
 pub errors: AtomicU64,
}

impl SyscallStats {
 pub const fn new() -> Self {
 SyscallStats {
 total_calls: AtomicU64::new(0),
 errors: AtomicU64::new(0),
 }
 }
 
 pub fn record_call(&self) {
 self.total_calls.fetch_add(1, Ordering::Relaxed);
 }
 
 pub fn record_error(&self) {
 self.errors.fetch_add(1, Ordering::Relaxed);
 }
}

static SYSCALL_STATS: SyscallStats = SyscallStats::new();

/// Systemtuneusesplit
pub fn syscall_dispatch(num: u64, args: [u64; 6]) -> i64 {
 SYSCALL_STATS.record_call();
 
 let result = match num {
 0 => sys_open(args[0] as *const u8, args[1] as i32, args[2] as u32),
 1 => sys_close(args[0] as i32),
 2 => sys_read(args[0] as i32, args[1] as *mut u8, args[2] as usize),
 3 => sys_write(args[0] as i32, args[1] as *const u8, args[2] as usize),
 4 => sys_lseek(args[0] as i32, args[1] as i64, args[2] as i32),
 20 => sys_fork(),
 21 => sys_execve(args[0] as *const u8, args[1] as *const *const u8, args[2] as *const *const u8),
 22 => sys_exit(args[0] as i32),
 24 => sys_getpid(),
 _ => {
 SYSCALL_STATS.record_error();
 Errno::Enosys.to_syscall_return()
 }
 };
 
 if result < 0 {
 SYSCALL_STATS.record_error();
 }
 
 result
}

// File SystemcallImplementation

fn sys_open(_path: *const u8, _flags: i32, _mode: u32) -> i64 {
 // TODO: ImplementationOpenFile
 Errno::Enoent.to_syscall_return()
}

fn sys_close(_fd: i32) -> i64 {
 // TODO: ImplementationCloseFile
 Errno::Ebadf.to_syscall_return()
}

fn sys_read(_fd: i32, _buf: *mut u8, _count: usize) -> i64 {
 // TODO: ImplementationReadFile
 Errno::Ebadf.to_syscall_return()
}

fn sys_write(_fd: i32, _buf: *const u8, _count: usize) -> i64 {
 // TODO: ImplementationWriteFile
 Errno::Ebadf.to_syscall_return()
}

fn sys_lseek(_fd: i32, _offset: i64, _whence: i32) -> i64 {
 // TODO: ImplementationFilefixedBit
 Errno::Ebadf.to_syscall_return()
}

// ProcessSystemcallImplementation

fn sys_fork() -> i64 {
 // TODO: ImplementationProcessCopy
 Errno::Enomem.to_syscall_return()
}

fn sys_execve(_path: *const u8, _argv: *const *const u8, _envp: *const *const u8) -> i64 {
 // TODO: Implementationexecuteprocessorder
 Errno::Enoent.to_syscall_return()
}

fn sys_exit(_status: i32) -> i64 {
 // TODO: ImplementationProcessExit
 0
}

fn sys_getpid() -> i64 {
 // TODO: ImplementationGetProcess ID
 1
}

pub fn init_posix() {
 log_info!("POSIX subsystem initialized");
 log_info!(" POSIX version: 201701L");
}