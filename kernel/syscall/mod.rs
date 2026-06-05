/*
 * Nuva OS - Kernel - System Call Handler
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

// Re-export print macros from crate root
pub use crate::{pr_emerg, pr_alert, pr_crit, pr_err, pr_warn, pr_notice, pr_info, pr_debug};
use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use crate::kernel::net::socket::{SockFd, SockAddrInet};

// Nuva native system call interface
pub mod nuva_syscall;

// NvVulkan system call interface (optional)
#[cfg(feature = "vulkan")]
pub mod nv_vulkan_syscall;

#[cfg(feature = "posix")]
use crate::posix::errno::Errno;
/// System call number type
pub type SyscallNum = u64;

/// System call handler function type
pub type SyscallHandler = fn(args: &[u64]) -> i64;

/// System call numbers (ARM64 ABI)
pub mod syscall_num {
    use super::SyscallNum;
    // Process control
    pub const SYS_GETPID: SyscallNum = 172;
    pub const SYS_GETTID: SyscallNum = 178;
    pub const SYS_FORK: SyscallNum = 1079;
    pub const SYS_VFORK: SyscallNum = 1071;
    pub const SYS_CLONE: SyscallNum = 220;
    pub const SYS_EXECVE: SyscallNum = 221;
    pub const SYS_EXIT: SyscallNum = 93;
    pub const SYS_EXIT_GROUP: SyscallNum = 94;
    pub const SYS_WAIT4: SyscallNum = 260;
    pub const SYS_WAITID: SyscallNum = 95;
    
    // File operations
    pub const SYS_OPEN: SyscallNum = 56;
    pub const SYS_OPENAT: SyscallNum = 56;
    pub const SYS_CLOSE: SyscallNum = 57;
    pub const SYS_READ: SyscallNum = 63;
    pub const SYS_WRITE: SyscallNum = 64;
    pub const SYS_LSEEK: SyscallNum = 62;
    pub const SYS_PREAD64: SyscallNum = 67;
    pub const SYS_PWRITE64: SyscallNum = 68;
    pub const SYS_READV: SyscallNum = 65;
    pub const SYS_WRITEV: SyscallNum = 66;
    pub const SYS_STAT: SyscallNum = 1061;
    pub const SYS_FSTAT: SyscallNum = 80;
    pub const SYS_LSTAT: SyscallNum = 1060;
    pub const SYS_FSTATAT: SyscallNum = 79;
    pub const SYS_MKDIR: SyscallNum = 1034;
    pub const SYS_MKDIRAT: SyscallNum = 34;
    pub const SYS_RMDIR: SyscallNum = 1035;
    pub const SYS_UNLINK: SyscallNum = 1028;
    pub const SYS_UNLINKAT: SyscallNum = 35;
    pub const SYS_RENAME: SyscallNum = 1038;
    pub const SYS_RENAMEAT: SyscallNum = 38;
    pub const SYS_LINK: SyscallNum = 1025;
    pub const SYS_LINKAT: SyscallNum = 37;
    pub const SYS_SYMLINK: SyscallNum = 1036;
    pub const SYS_SYMLINKAT: SyscallNum = 36;
    pub const SYS_READLINK: SyscallNum = 1033;
    pub const SYS_READLINKAT: SyscallNum = 78;
    pub const SYS_CHMOD: SyscallNum = 1029;
    pub const SYS_FCHMOD: SyscallNum = 52;
    pub const SYS_FCHMODAT: SyscallNum = 53;
    pub const SYS_CHOWN: SyscallNum = 1030;
    pub const SYS_FCHOWN: SyscallNum = 55;
    pub const SYS_FCHOWNAT: SyscallNum = 54;
    pub const SYS_TRUNCATE: SyscallNum = 45;
    pub const SYS_FTRUNCATE: SyscallNum = 46;
    pub const SYS_DUP: SyscallNum = 23;
    pub const SYS_DUP2: SyscallNum = 1041;
    pub const SYS_DUP3: SyscallNum = 24;
    pub const SYS_FCNTL: SyscallNum = 25;
    pub const SYS_IOCTL: SyscallNum = 29;
    pub const SYS_GETDENTS: SyscallNum = 1063;
    pub const SYS_GETDENTS64: SyscallNum = 61;
    pub const SYS_CHDIR: SyscallNum = 49;
    pub const SYS_FCHDIR: SyscallNum = 50;
    pub const SYS_GETCWD: SyscallNum = 17;
    
    // Memory management
    pub const SYS_MMAP: SyscallNum = 222;
    pub const SYS_MUNMAP: SyscallNum = 215;
    pub const SYS_MREMAP: SyscallNum = 216;
    pub const SYS_MPROTECT: SyscallNum = 226;
    pub const SYS_MSYNC: SyscallNum = 227;
    pub const SYS_MADVISE: SyscallNum = 233;
    pub const SYS_MINCORE: SyscallNum = 232;
    pub const SYS_BRK: SyscallNum = 214;
    pub const SYS_SBRK: SyscallNum = 214;
    
    // Signal handling
    pub const SYS_SIGNAL: SyscallNum = 99;
    pub const SYS_SIGACTION: SyscallNum = 134;
    pub const SYS_SIGPROCMASK: SyscallNum = 135;
    pub const SYS_SIGPENDING: SyscallNum = 136;
    pub const SYS_SIGSUSPEND: SyscallNum = 130;
    pub const SYS_SIGTIMEDWAIT: SyscallNum = 137;
    pub const SYS_SIGRETURN: SyscallNum = 139;
    pub const SYS_KILL: SyscallNum = 129;
    pub const SYS_TGKILL: SyscallNum = 131;
    pub const SYS_TKILL: SyscallNum = 130;
    pub const SYS_PAUSE: SyscallNum = 1062;
    pub const SYS_RT_SIGACTION: SyscallNum = 134;
    pub const SYS_RT_SIGPROCMASK: SyscallNum = 135;
    pub const SYS_RT_SIGPENDING: SyscallNum = 136;
    pub const SYS_RT_SIGTIMEDWAIT: SyscallNum = 137;
    pub const SYS_RT_SIGQUEUEINFO: SyscallNum = 138;
    pub const SYS_RT_SIGRETURN: SyscallNum = 139;
    pub const SYS_RT_TGSIGQUEUEINFO: SyscallNum = 240;
    
    // Socket operations
    pub const SYS_SOCKET: SyscallNum = 198;
    pub const SYS_BIND: SyscallNum = 200;
    pub const SYS_LISTEN: SyscallNum = 201;
    pub const SYS_ACCEPT: SyscallNum = 202;
    pub const SYS_CONNECT: SyscallNum = 203;
    pub const SYS_SEND: SyscallNum = 1044;
    pub const SYS_RECV: SyscallNum = 1045;
    pub const SYS_SENDTO: SyscallNum = 206;
    pub const SYS_RECVFROM: SyscallNum = 207;
    pub const SYS_SENDMSG: SyscallNum = 211;
    pub const SYS_RECVMSG: SyscallNum = 212;
    pub const SYS_SHUTDOWN: SyscallNum = 210;
    pub const SYS_SETSOCKOPT: SyscallNum = 208;
    pub const SYS_GETSOCKOPT: SyscallNum = 209;
    pub const SYS_GETSOCKNAME: SyscallNum = 204;
    pub const SYS_GETPEERNAME: SyscallNum = 205;
    pub const SYS_SOCKETPAIR: SyscallNum = 199;
    
    // Time operations
    pub const SYS_TIME: SyscallNum = 1064;
    pub const SYS_GETTIMEOFDAY: SyscallNum = 169;
    pub const SYS_SETTIMEOFDAY: SyscallNum = 170;
    pub const SYS_CLOCK_GETTIME: SyscallNum = 113;
    pub const SYS_CLOCK_SETTIME: SyscallNum = 112;
    pub const SYS_CLOCK_GETRES: SyscallNum = 114;
    pub const SYS_NANOSLEEP: SyscallNum = 101;
    pub const SYS_CLOCK_NANOSLEEP: SyscallNum = 115;
    pub const SYS_ALARM: SyscallNum = 1059;
    pub const SYS_TIMER_CREATE: SyscallNum = 107;
    pub const SYS_TIMER_DELETE: SyscallNum = 111;
    pub const SYS_TIMER_GETTIME: SyscallNum = 108;
    pub const SYS_TIMER_SETTIME: SyscallNum = 110;
    pub const SYS_TIMER_GETOVERRUN: SyscallNum = 109;
    
    // Process/thread control
    pub const SYS_NICE: SyscallNum = 1022;
    pub const SYS_GETPRIORITY: SyscallNum = 140;
    pub const SYS_SETPRIORITY: SyscallNum = 141;
    pub const SYS_SCHED_SETPARAM: SyscallNum = 118;
    pub const SYS_SCHED_GETPARAM: SyscallNum = 121;
    pub const SYS_SCHED_SETSCHEDULER: SyscallNum = 119;
    pub const SYS_SCHED_GETSCHEDULER: SyscallNum = 120;
    pub const SYS_SCHED_YIELD: SyscallNum = 124;
    pub const SYS_SCHED_GET_PRIORITY_MAX: SyscallNum = 125;
    pub const SYS_SCHED_GET_PRIORITY_MIN: SyscallNum = 126;
    pub const SYS_SCHED_RR_GET_INTERVAL: SyscallNum = 127;
    pub const SYS_SCHED_SETAFFINITY: SyscallNum = 122;
    pub const SYS_SCHED_GETAFFINITY: SyscallNum = 123;
    pub const SYS_SETUID: SyscallNum = 146;
    pub const SYS_GETUID: SyscallNum = 174;
    pub const SYS_SETEUID: SyscallNum = 175;
    pub const SYS_GETEUID: SyscallNum = 175;
    pub const SYS_SETGID: SyscallNum = 144;
    pub const SYS_GETGID: SyscallNum = 176;
    pub const SYS_SETEGID: SyscallNum = 177;
    pub const SYS_GETEGID: SyscallNum = 177;
    pub const SYS_SETGROUPS: SyscallNum = 159;
    pub const SYS_GETGROUPS: SyscallNum = 158;
    pub const SYS_SETRESUID: SyscallNum = 147;
    pub const SYS_GETRESUID: SyscallNum = 148;
    pub const SYS_SETRESGID: SyscallNum = 149;
    pub const SYS_GETRESGID: SyscallNum = 150;
    pub const SYS_SETSID: SyscallNum = 157;
    pub const SYS_GETSID: SyscallNum = 175;
    pub const SYS_SETPGID: SyscallNum = 155;
    pub const SYS_GETPGID: SyscallNum = 155;
    pub const SYS_GETPPID: SyscallNum = 173;
    pub const SYS_SETITIMER: SyscallNum = 103;
    pub const SYS_GETITIMER: SyscallNum = 102;
    pub const SYS_UMASK: SyscallNum = 166;
    
    // IPC
    pub const SYS_PIPE: SyscallNum = 1042;
    pub const SYS_PIPE2: SyscallNum = 59;
    pub const SYS_MQ_OPEN: SyscallNum = 180;
    pub const SYS_MQ_CLOSE: SyscallNum = 181;
    pub const SYS_MQ_UNLINK: SyscallNum = 182;
    pub const SYS_MQ_SEND: SyscallNum = 183;
    pub const SYS_MQ_RECEIVE: SyscallNum = 184;
    pub const SYS_MQ_NOTIFY: SyscallNum = 185;
    pub const SYS_MQ_GETSETATTR: SyscallNum = 186;
    pub const SYS_SEMGET: SyscallNum = 190;
    pub const SYS_SEMOP: SyscallNum = 193;
    pub const SYS_SEMCTL: SyscallNum = 191;
    pub const SYS_SHMGET: SyscallNum = 194;
    pub const SYS_SHMAT: SyscallNum = 196;
    pub const SYS_SHMDT: SyscallNum = 197;
    pub const SYS_SHMCTL: SyscallNum = 195;
    pub const SYS_MSGGET: SyscallNum = 186;
    pub const SYS_MSGSND: SyscallNum = 189;
    pub const SYS_MSGRCV: SyscallNum = 188;
    pub const SYS_MSGCTL: SyscallNum = 187;
    
    // System information
    pub const SYS_UNAME: SyscallNum = 160;
    pub const SYS_SYSINFO: SyscallNum = 179;
    pub const SYS_HOSTNAME: SyscallNum = 161;
    pub const SYS_SETHOSTNAME: SyscallNum = 161;
    pub const SYS_DOMAINNAME: SyscallNum = 162;
    pub const SYS_SETDOMAINNAME: SyscallNum = 162;
    
    // Other
    pub const SYS_REBOOT: SyscallNum = 142;
    pub const SYS_SYNC: SyscallNum = 81;
    pub const SYS_SYNCFS: SyscallNum = 82;
    pub const SYS_MOUNT: SyscallNum = 40;
    pub const SYS_UMOUNT: SyscallNum = 39;
    pub const SYS_UMOUNT2: SyscallNum = 39;
    pub const SYS_SWAPON: SyscallNum = 224;
    pub const SYS_SWAPOFF: SyscallNum = 225;
    pub const SYS_ACCT: SyscallNum = 89;
    pub const SYS_GETRUSAGE: SyscallNum = 98;
    pub const SYS_SYSLOG: SyscallNum = 103;
    pub const SYS_PTRACE: SyscallNum = 117;
    pub const SYS_ARCH_PRCTL: SyscallNum = 1000;
    pub const SYS_PRCTL: SyscallNum = 167;
    pub const SYS_GETRANDOM: SyscallNum = 278;
    pub const SYS_MEMFD_CREATE: SyscallNum = 279;
    pub const SYS_EVENTFD: SyscallNum = 1043;
    pub const SYS_EVENTFD2: SyscallNum = 19;
    pub const SYS_TIMERFD_CREATE: SyscallNum = 85;
    pub const SYS_TIMERFD_SETTIME: SyscallNum = 86;
    pub const SYS_TIMERFD_GETTIME: SyscallNum = 87;
    pub const SYS_SIGNALFD: SyscallNum = 1046;
    pub const SYS_SIGNALFD4: SyscallNum = 74;
    pub const SYS_EPOLL_CREATE: SyscallNum = 1047;
    pub const SYS_EPOLL_CREATE1: SyscallNum = 20;
    pub const SYS_EPOLL_CTL: SyscallNum = 21;
    pub const SYS_EPOLL_PWAIT: SyscallNum = 22;
    pub const SYS_EPOLL_WAIT: SyscallNum = 1069;
    pub const SYS_INOTIFY_INIT: SyscallNum = 1048;
    pub const SYS_INOTIFY_INIT1: SyscallNum = 26;
    pub const SYS_INOTIFY_ADD_WATCH: SyscallNum = 27;
    pub const SYS_INOTIFY_RM_WATCH: SyscallNum = 28;
    pub const SYS_FANOTIFY_INIT: SyscallNum = 262;
    pub const SYS_FANOTIFY_MARK: SyscallNum = 263;
    pub const SYS_KEXEC_LOAD: SyscallNum = 104;
    pub const SYS_KEXEC_FILE_LOAD: SyscallNum = 294;
    pub const SYS_INIT_MODULE: SyscallNum = 105;
    pub const SYS_FINIT_MODULE: SyscallNum = 273;
    pub const SYS_DELETE_MODULE: SyscallNum = 106;
    pub const SYS_SECCOMP: SyscallNum = 277;
    pub const SYS_BPF: SyscallNum = 280;
    pub const SYS_USERFAULTFD: SyscallNum = 282;
    pub const SYS_PIDFD_OPEN: SyscallNum = 434;
    pub const SYS_PIDFD_SEND_SIGNAL: SyscallNum = 424;
    pub const SYS_CLONE3: SyscallNum = 435;
    pub const SYS_OPENAT2: SyscallNum = 437;
    pub const SYS_FACCESSAT2: SyscallNum = 439;
    pub const SYS_PROCESS_MADVISE: SyscallNum = 440;
    pub const SYS_EPOLL_PWAIT2: SyscallNum = 441;
    pub const SYS_MOUNT_SETATTR: SyscallNum = 442;
    pub const SYS_QUOTACTL_FD: SyscallNum = 443;
    pub const SYS_LANDLOCK_CREATE_RULESET: SyscallNum = 444;
    pub const SYS_LANDLOCK_ADD_RULE: SyscallNum = 445;
    pub const SYS_LANDLOCK_RESTRICT_SELF: SyscallNum = 446;
    pub const SYS_MEMFD_SECRET: SyscallNum = 447;
    pub const SYS_PROCESS_MRELEASE: SyscallNum = 448;
    pub const SYS_FUTEX_WAITV: SyscallNum = 449;
    pub const SYS_SET_MEMPOLICY_HOME_NODE: SyscallNum = 450;
    pub const SYS_CACHESTAT: SyscallNum = 451;
    pub const SYS_FCHMODAT2: SyscallNum = 452;
    pub const SYS_MAP_SHADOW_STACK: SyscallNum = 453;
    pub const SYS_FUTEX_WAKE: SyscallNum = 454;
    pub const SYS_FUTEX_WAIT: SyscallNum = 455;
    pub const SYS_FUTEX_REQUEUE: SyscallNum = 456;
    pub const SYS_STATX: SyscallNum = 291;
}

/// System call table entry
pub struct SyscallEntry {
    /// System call number
    pub num: SyscallNum,
    /// Handler function
    pub handler: SyscallHandler,
    /// Name
    pub name: [u8; 32],
    /// Number of arguments
    pub nargs: u32,
    /// Flags
    pub flags: AtomicU32,
}

impl Clone for SyscallEntry {
    fn clone(&self) -> Self {
        Self {
            num: self.num.clone(),
            handler: self.handler.clone(),
            name: self.name.clone(),
            nargs: self.nargs.clone(),
            flags: AtomicU32::new(self.flags.load(core::sync::atomic::Ordering::Relaxed)),
        }
    }
}

/// Syscall flags
pub mod syscall_flags {
    pub const SF_READ: u32 = 0x01;
    pub const SF_WRITE: u32 = 0x02;
    pub const SF_BLOCKING: u32 = 0x04;
    pub const SF_PRIVILEGED: u32 = 0x08;
}

/// System call manager
pub struct SyscallManager {
    /// System call table
    pub table: [Option<SyscallEntry>; 512],
    /// Number of registered syscalls
    pub nr_syscalls: AtomicU32,
    /// Statistics
    pub stats: SyscallStats,
}

/// System call statistics
pub struct SyscallStats {
    /// Total calls
    pub total: AtomicU64,
    /// Errors
    pub errors: AtomicU64,
    /// Per-syscall count
    pub counts: [AtomicU64; 512],
}

impl SyscallStats {
    pub const fn new() -> Self {
        SyscallStats {
            total: AtomicU64::new(0),
            errors: AtomicU64::new(0),
            counts: [const { AtomicU64::new(0) }; 512],
        }
    }
}

impl SyscallManager {
    pub const fn new() -> Self {
        SyscallManager {
            table: [const { None }; 512],
            nr_syscalls: AtomicU32::new(0),
            stats: SyscallStats::new(),
        }
    }
    
    /// Initialize syscall manager
    pub fn init(&self) {
        log_info!("Initializing system call handler...");
        
        // Register core system calls
        self.register_core_syscalls();
        
        log_info!("System call handler initialized with {} syscalls", 
                 self.nr_syscalls.load(Ordering::Acquire));
    }
    
    /// Register core system calls
    fn register_core_syscalls(&mut self) {
        // Process control
        self.register(syscall_num::SYS_GETPID, sys_getpid, "getpid", 0);
        self.register(syscall_num::SYS_GETTID, sys_gettid, "gettid", 0);
        self.register(syscall_num::SYS_FORK, sys_fork, "fork", 0);
        self.register(syscall_num::SYS_EXIT, sys_exit, "exit", 1);
        self.register(syscall_num::SYS_EXIT_GROUP, sys_exit_group, "exit_group", 1);
        self.register(syscall_num::SYS_WAIT4, sys_wait4, "wait4", 4);
        self.register(syscall_num::SYS_EXECVE, sys_execve, "execve", 3);
        self.register(syscall_num::SYS_CLONE, sys_clone, "clone", 5);
        
        // File operations
        self.register(syscall_num::SYS_OPEN, sys_open, "open", 3);
        self.register(syscall_num::SYS_CLOSE, sys_close, "close", 1);
        self.register(syscall_num::SYS_READ, sys_read, "read", 3);
        self.register(syscall_num::SYS_WRITE, sys_write, "write", 3);
        self.register(syscall_num::SYS_LSEEK, sys_lseek, "lseek", 3);
        self.register(syscall_num::SYS_STAT, sys_stat, "stat", 2);
        self.register(syscall_num::SYS_FSTAT, sys_fstat, "fstat", 2);
        self.register(syscall_num::SYS_MKDIR, sys_mkdir, "mkdir", 2);
        self.register(syscall_num::SYS_UNLINK, sys_unlink, "unlink", 1);
        self.register(syscall_num::SYS_CHDIR, sys_chdir, "chdir", 1);
        self.register(syscall_num::SYS_GETCWD, sys_getcwd, "getcwd", 2);
        
        // Memory management
        self.register(syscall_num::SYS_MMAP, sys_mmap, "mmap", 6);
        self.register(syscall_num::SYS_MUNMAP, sys_munmap, "munmap", 2);
        self.register(syscall_num::SYS_BRK, sys_brk, "brk", 1);
        self.register(syscall_num::SYS_MPROTECT, sys_mprotect, "mprotect", 3);
        
        // Socket operations
        self.register(syscall_num::SYS_SOCKET, sys_socket, "socket", 3);
        self.register(syscall_num::SYS_BIND, sys_bind, "bind", 3);
        self.register(syscall_num::SYS_LISTEN, sys_listen, "listen", 2);
        self.register(syscall_num::SYS_ACCEPT, sys_accept, "accept", 3);
        self.register(syscall_num::SYS_CONNECT, sys_connect, "connect", 3);
        self.register(syscall_num::SYS_SEND, sys_send, "send", 4);
        self.register(syscall_num::SYS_RECV, sys_recv, "recv", 4);
        self.register(syscall_num::SYS_SENDTO, sys_sendto, "sendto", 6);
        self.register(syscall_num::SYS_RECVFROM, sys_recvfrom, "recvfrom", 6);
        self.register(syscall_num::SYS_SHUTDOWN, sys_shutdown, "shutdown", 2);
        self.register(syscall_num::SYS_SETSOCKOPT, sys_setsockopt, "setsockopt", 5);
        self.register(syscall_num::SYS_GETSOCKOPT, sys_getsockopt, "getsockopt", 5);
        
        // Time operations
        self.register(syscall_num::SYS_GETTIMEOFDAY, sys_gettimeofday, "gettimeofday", 2);
        self.register(syscall_num::SYS_CLOCK_GETTIME, sys_clock_gettime, "clock_gettime", 2);
        self.register(syscall_num::SYS_NANOSLEEP, sys_nanosleep, "nanosleep", 2);
        
        // Process/thread control
        self.register(syscall_num::SYS_SCHED_YIELD, sys_sched_yield, "sched_yield", 0);
        self.register(syscall_num::SYS_GETUID, sys_getuid, "getuid", 0);
        self.register(syscall_num::SYS_GETGID, sys_getgid, "getgid", 0);
        self.register(syscall_num::SYS_SETUID, sys_setuid, "setuid", 1);
        self.register(syscall_num::SYS_SETGID, sys_setgid, "setgid", 1);
        self.register(syscall_num::SYS_GETPPID, sys_getppid, "getppid", 0);
        self.register(syscall_num::SYS_SETSID, sys_setsid, "setsid", 0);
        self.register(syscall_num::SYS_UMASK, sys_umask, "umask", 1);
        
        // System information
        self.register(syscall_num::SYS_UNAME, sys_uname, "uname", 1);
        self.register(syscall_num::SYS_SYSINFO, sys_sysinfo, "sysinfo", 1);
        self.register(syscall_num::SYS_REBOOT, sys_reboot, "reboot", 4);
        
        // IPC
        self.register(syscall_num::SYS_PIPE, sys_pipe, "pipe", 1);
        self.register(syscall_num::SYS_PIPE2, sys_pipe2, "pipe2", 2);
        
        // Tombstone mechanism
        self.register(crate::kernel::tombstone::syscall::SYS_TOMBSTONE_QUERY,
            crate::kernel::tombstone::syscall::sys_tombstone_query, "tombstone_query", 1);
        self.register(crate::kernel::tombstone::syscall::SYS_TOMBSTONE_READ,
            crate::kernel::tombstone::syscall::sys_tombstone_read, "tombstone_read", 1);
        self.register(crate::kernel::tombstone::syscall::SYS_TOMBSTONE_CLEAR,
            crate::kernel::tombstone::syscall::sys_tombstone_clear, "tombstone_clear", 1);
        self.register(crate::kernel::tombstone::syscall::SYS_TOMBSTONE_STATS,
            crate::kernel::tombstone::syscall::sys_tombstone_stats, "tombstone_stats", 0);
    }
    
    /// Register a system call
    pub fn register(&mut self, num: SyscallNum, handler: SyscallHandler, name: &str, nargs: u32) {
        let idx = (num % 512) as usize;
        
        let mut name_buf = [0u8; 32];
        let len = name.len().min(31);
        name_buf[..len].copy_from_slice(&name.as_bytes()[..len]);
        
        self.table[idx] = Some(SyscallEntry {
            num,
            handler,
            name: name_buf,
            nargs,
            flags: AtomicU32::new(0),
        });
        
        self.nr_syscalls.fetch_add(1, Ordering::AcqRel);
    }
    
    /// Handle system call
    pub fn handle(&self, num: SyscallNum, args: &[u64]) -> i64 {
        self.stats.total.fetch_add(1, Ordering::AcqRel);
        
        let idx = (num % 512) as usize;
        
        if let Some(ref entry) = self.table[idx] {
            self.stats.counts[idx].fetch_add(1, Ordering::AcqRel);
            
            let result = (entry.handler)(args);
            
            if result < 0 {
                self.stats.errors.fetch_add(1, Ordering::AcqRel);
            }
            
            result
        } else {
            log_warn!("Unknown syscall: {}", num);
            -38  /* ENOSYS */
        }
    }
    
    /// Get syscall name
    pub fn get_name(&self, num: SyscallNum) -> &[u8] {
        let idx = (num % 512) as usize;
        
        if let Some(ref entry) = self.table[idx] {
            &entry.name
        } else {
            b"unknown"
        }
    }
    
    /// Print statistics
    pub fn print_stats(&self) {
        log_info!("Syscall Statistics:");
        log_info!("  Total: {}", self.stats.total.load(Ordering::Acquire));
        log_info!("  Errors: {}", self.stats.errors.load(Ordering::Acquire));
        log_info!("  Registered: {}", self.nr_syscalls.load(Ordering::Acquire));
    }
}

/// Global syscall manager
static SYSCALL_MANAGER: core::sync::OnceLock<SyscallManager> = core::sync::OnceLock::new();

/// Get syscall manager
pub fn syscall_manager() -> &'static SyscallManager {
    SYSCALL_MANAGER.get_or_init(SyscallManager::new)
}

pub fn init_syscall_manager() -> &'static SyscallManager {
    SYSCALL_MANAGER.get_or_init(SyscallManager::new)
}

/// Initialize syscall handler
pub fn init_syscall() {
    let mgr = syscall_manager();
    mgr.init();
}

/// System call entry point (called from assembly)
#[no_mangle]
pub extern "C" fn syscall_handler(num: SyscallNum, a0: u64, a1: u64, a2: u64, 
                                   a3: u64, a4: u64, a5: u64) -> i64 {
    let args = [a0, a1, a2, a3, a4, a5];
    syscall_manager().handle(num, &args)
}

/// Dispatch a system call by number
pub fn dispatch(num: SyscallNum, args: &[u64; 6]) -> i64 {
    syscall_manager().handle(num, args)
}

// System call implementations

/// getpid
fn sys_getpid(_args: &[u64]) -> i64 {
    // TODO: Get current process PID
    1
}

/// gettid
fn sys_gettid(_args: &[u64]) -> i64 {
    // TODO: Get current thread TID
    1
}

/// fork
fn sys_fork(_args: &[u64]) -> i64 {
    // TODO: Implement fork
    -1
}

/// exit
fn sys_exit(args: &[u64]) -> i64 {
    let status = args[0] as i64;
    log_info!("Process exiting with status {}", status);
    
    // TODO: Implement process exit
    0
}

/// exit_group
fn sys_exit_group(args: &[u64]) -> i64 {
    sys_exit(args)
}

/// wait4
fn sys_wait4(_args: &[u64]) -> i64 {
    // TODO: Implement wait4
    -10  /* ECHILD */
}

/// execve
fn sys_execve(_args: &[u64]) -> i64 {
    // TODO: Implement execve
    -2  /* ENOENT */
}

/// clone
fn sys_clone(_args: &[u64]) -> i64 {
    // TODO: Implement clone
    -1
}

/// open
fn sys_open(args: &[u64]) -> i64 {
    let path = args[0] as *const u8;
    let flags = args[1] as u32;
    let mode = args[2] as u32;
    
    crate::kernel::fs::sys_open(path, flags, mode)
}

/// close
fn sys_close(args: &[u64]) -> i64 {
    let fd = args[0] as i32;
    crate::kernel::fs::sys_close(fd) as i64

}

/// read
fn sys_read(args: &[u64]) -> i64 {
    let fd = args[0] as i32;
    let buf = args[1] as *mut u8;
    let count = args[2] as usize;
    
    crate::kernel::fs::sys_read(fd, buf, count) as i64

}

/// write
fn sys_write(args: &[u64]) -> i64 {
    let fd = args[0] as i32;
    let buf = args[1] as *const u8;
    let count = args[2] as usize;
    
    crate::kernel::fs::sys_write(fd, buf, count) as i64

}

/// lseek
fn sys_lseek(args: &[u64]) -> i64 {
    let fd = args[0] as i32;
    let offset = args[1] as i64;
    let whence = args[2] as i32;
    
    crate::kernel::fs::sys_lseek(fd, offset, whence as u32)
}

/// stat
fn sys_stat(args: &[u64]) -> i64 {
    let path = args[0] as *const u8;
    let stat_buf = args[1] as *mut crate::kernel::fs::vfs::Stat;

    if path.is_null() {
        return Errno::Efault.to_syscall_return(); // EFAULT
    }
    if stat_buf.is_null() {
        return Errno::Efault.to_syscall_return(); // EFAULT
    }

    // SAFETY: unsafe block required for low-level memory or hardware access
    let path_str = unsafe {
        let mut len = 0;
        let mut ptr = path;
        while *ptr != 0 && len < 4096 {
            len += 1;
            ptr = ptr.add(1);
        }
        if len == 0 || len >= 4096 {
            return Errno::Enametoolong.to_syscall_return(); // ENAMETOOLONG
        }
        core::str::from_utf8_unchecked(core::slice::from_raw_parts(path, len))
    };

    // Lookup inode by path, following symlinks
    let vfs = crate::kernel::fs::vfs::vfs_core();
    let lookup = match vfs.path_lookup(path_str, crate::kernel::fs::vfs::lookup_flags::FOLLOW_SYMLINK) {
        Some(l) => l,
        None => return Errno::Enoent.to_syscall_return(), // ENOENT
    };

    // Fill stat from lookup result
    // SAFETY: stat_buf is non-null and user-validated
    unsafe {
        (*stat_buf).device_id = 0;
        (*stat_buf).inode_number = lookup.inode;
        (*stat_buf).mode = 0o100644;
        (*stat_buf).link_count = 1;
        (*stat_buf).user_id = 0;
        (*stat_buf).group_id = 0;
        (*stat_buf).raw_device_id = 0;
        (*stat_buf).size = 0;
        (*stat_buf).block_size = 4096;
        (*stat_buf).block_count = 0;
        (*stat_buf).access_time = 0;
        (*stat_buf).modification_time = 0;
        (*stat_buf).change_time = 0;
    }

    0
}

/// fstat
fn sys_fstat(args: &[u64]) -> i64 {
    let fd = args[0] as i32;
    let stat_buf = args[1] as *mut crate::kernel::fs::vfs::Stat;

    if fd < 0 {
        return Errno::Ebadf.to_syscall_return(); // EBADF
    }
    if stat_buf.is_null() {
        return Errno::Efault.to_syscall_return(); // EFAULT
    }

    // Get file from VFS file table
    let files = crate::kernel::fs::vfs::file::get_global_files();
    let file_ref = match files.get_file(fd as u32) {
        Some(f) => f,
        None => return Errno::Ebadf.to_syscall_return(), // EBADF
    };

    let inode_ptr = file_ref.f_inode;
    if inode_ptr.is_null() {
        return Errno::Ebadf.to_syscall_return(); // EBADF
    }

    // SAFETY: inode pointer was set during open and remains valid
    let inode = unsafe { &*inode_ptr };

    // Fill stat struct from inode metadata
    // SAFETY: stat_buf is non-null and user-validated
    unsafe {
        (*stat_buf).device_id = inode.i_sb;
        (*stat_buf).inode_number = inode.i_ino;
        (*stat_buf).mode = inode.i_mode;
        (*stat_buf).link_count = inode.i_nlink.load(Ordering::Acquire);
        (*stat_buf).user_id = inode.i_uid;
        (*stat_buf).group_id = inode.i_gid;
        (*stat_buf).raw_device_id = inode.i_rdev;
        (*stat_buf).size = inode.i_size.load(Ordering::Acquire);
        (*stat_buf).block_size = 1u64 << inode.i_blkbits;
        (*stat_buf).block_count = inode.i_blocks;
        (*stat_buf).access_time = inode.i_atime;
        (*stat_buf).modification_time = inode.i_mtime;
        (*stat_buf).change_time = inode.i_ctime;
    }

    0
}

/// mkdir
fn sys_mkdir(args: &[u64]) -> i64 {
    let path = args[0] as *const u8;
    let mode = args[1] as u32;
    
    crate::kernel::fs::sys_mkdir(path, mode)
}

/// unlink
fn sys_unlink(args: &[u64]) -> i64 {
    let path = args[0] as *const u8;
    crate::kernel::fs::sys_unlink(path)
}

/// chdir
fn sys_chdir(_args: &[u64]) -> i64 {
    // TODO: Implement chdir
    -2
}

/// getcwd
fn sys_getcwd(_args: &[u64]) -> i64 {
    // TODO: Implement getcwd
    -2
}

/// mmap
fn sys_mmap(_args: &[u64]) -> i64 {
    // TODO: Implement mmap
    -12  /* ENOMEM */
}

/// munmap
fn sys_munmap(_args: &[u64]) -> i64 {
    // TODO: Implement munmap
    -22  /* EINVAL */
}

/// brk
fn sys_brk(args: &[u64]) -> i64 {
    let addr = args[0];

    // Default RLIMIT_DATA: 256 MB
    const RLIMIT_DATA: u64 = 256 * 1024 * 1024;
    const PAGE_SIZE: u64 = 4096;

    // Get current process mm_struct
    let current = crate::kernel::process::get_current();
    if current.is_null() {
        return Errno::Enomem.to_syscall_return(); // ENOMEM
    }

    // SAFETY: current is a valid process pointer from get_current()
    let mm = unsafe { &mut (*current).mm };

    // If addr is 0, return current brk value
    if addr == 0 {
        return mm.brk as i64;
    }

    // Heap not initialized yet
    if mm.start_brk == 0 {
        return Errno::Enomem.to_syscall_return(); // ENOMEM
    }

    // Cannot shrink below start_brk
    if addr < mm.start_brk {
        return Errno::Einval.to_syscall_return(); // EINVAL
    }

    // Check RLIMIT_DATA: new brk must not exceed start_brk + RLIMIT_DATA
    if addr > mm.start_brk + RLIMIT_DATA {
        return Errno::Enomem.to_syscall_return(); // ENOMEM
    }

    if addr > mm.brk {
        // Expand heap: page-align the new brk
        let old_aligned = (mm.brk + PAGE_SIZE - 1) & !(PAGE_SIZE - 1);
        let new_aligned = (addr + PAGE_SIZE - 1) & !(PAGE_SIZE - 1);

        if new_aligned > old_aligned {
            let _nr_pages = (new_aligned - old_aligned) / PAGE_SIZE;
            // TODO: actual page table mapping via arch-specific routine
            // For now, update bookkeeping only.
            mm.total_vm.fetch_add(new_aligned - old_aligned, Ordering::AcqRel);
            mm.data_vm.fetch_add(new_aligned - old_aligned, Ordering::AcqRel);
        }
    } else if addr < mm.brk {
        // Shrink heap: page-align the new brk
        let old_aligned = (mm.brk + PAGE_SIZE - 1) & !(PAGE_SIZE - 1);
        let new_aligned = (addr + PAGE_SIZE - 1) & !(PAGE_SIZE - 1);

        if new_aligned < old_aligned {
            let _nr_pages = (old_aligned - new_aligned) / PAGE_SIZE;
            // TODO: actual page table unmapping and frame freeing
            // For now, update bookkeeping only.
            let freed = old_aligned - new_aligned;
            if mm.total_vm.load(Ordering::Acquire) >= freed {
                mm.total_vm.fetch_sub(freed, Ordering::AcqRel);
            }
            if mm.data_vm.load(Ordering::Acquire) >= freed {
                mm.data_vm.fetch_sub(freed, Ordering::AcqRel);
            }
        }
    }

    // Update the brk
    mm.brk = addr;
    addr as i64
}

/// mprotect
fn sys_mprotect(_args: &[u64]) -> i64 {
    // TODO: Implement mprotect
    0
}

/// socket
fn sys_socket(args: &[u64]) -> i64 {
    let domain = args[0] as i32;
    let sock_type = args[1] as i32;
    let protocol = args[2] as i32;
    
    crate::kernel::net::sys_socket(domain, sock_type, protocol)
}

/// bind
fn sys_bind(args: &[u64]) -> i64 {
    let sockfd = args[0] as i32;
    let addr = args[1] as *const u8;
    let addrlen = args[2] as usize;
    
    crate::kernel::net::sys_bind(sockfd, addr as *const SockAddrInet, addrlen)
}

/// listen
fn sys_listen(args: &[u64]) -> i64 {
    let sockfd = args[0] as i32;
    let backlog = args[1] as i64;
    
    crate::kernel::net::sys_listen(sockfd, backlog as i32)
}

/// accept
fn sys_accept(args: &[u64]) -> i64 {
    let sockfd = args[0] as i32;
    let addr = args[1] as *mut u8;
    let addrlen = args[2] as *mut u32;
    
    crate::kernel::net::sys_accept(sockfd, addr as *mut SockAddrInet, addrlen)
}

/// connect
fn sys_connect(args: &[u64]) -> i64 {
    let sockfd = args[0] as i32;
    let addr = args[1] as *const u8;
    let addrlen = args[2] as usize;
    
    crate::kernel::net::sys_connect(sockfd, addr as *const SockAddrInet, addrlen)
}

/// send
fn sys_send(args: &[u64]) -> i64 {
    let sockfd = args[0] as i32;
    let buf = args[1] as *const u8;
    let len = args[2] as usize;
    let flags = args[3] as i64;
    
    crate::kernel::net::sys_send(sockfd, buf, len, flags as i32)
}

/// recv
fn sys_recv(args: &[u64]) -> i64 {
    let sockfd = args[0] as i32;
    let buf = args[1] as *mut u8;
    let len = args[2] as usize;
    let flags = args[3] as i64;
    
    crate::kernel::net::sys_recv(sockfd, buf, len, flags as i32)
}

/// sendto
fn sys_sendto(args: &[u64]) -> i64 {
    let sockfd = args[0] as i32;
    let buf = args[1] as *const u8;
    let len = args[2] as usize;
    let flags = args[3] as i64;
    let dest_addr = args[4] as *const u8;
    let addrlen = args[5] as usize;
    
    crate::kernel::net::sys_sendto(sockfd, buf, len, flags as i32, dest_addr as *const SockAddrInet, addrlen)
}

/// recvfrom
fn sys_recvfrom(args: &[u64]) -> i64 {
    let sockfd = args[0] as i32;
    let buf = args[1] as *mut u8;
    let len = args[2] as usize;
    let flags = args[3] as i64;
    let src_addr = args[4] as *mut u8;
    let addrlen = args[5] as *mut u32;
    
    crate::kernel::net::sys_recvfrom(sockfd, buf, len, flags as i32, src_addr as *mut SockAddrInet, addrlen)
}

/// shutdown
fn sys_shutdown(args: &[u64]) -> i64 {
    let sockfd = args[0] as i32;
    let how = args[1] as i64;
    
    crate::kernel::net::sys_shutdown(sockfd, how as i32)
}

/// setsockopt
fn sys_setsockopt(args: &[u64]) -> i64 {
    let sockfd = args[0] as i32;
    let level = args[1] as i32;
    let optname = args[2] as i32;
    let optval = args[3] as *const u8;
    let optlen = args[4] as u32;
    
    crate::kernel::net::sys_setsockopt(sockfd, level, optname, optval, optlen)
}

/// getsockopt
fn sys_getsockopt(args: &[u64]) -> i64 {
    let sockfd = args[0] as i32;
    let level = args[1] as i32;
    let optname = args[2] as i32;
    let optval = args[3] as *mut u8;
    let optlen = args[4] as *mut u32;
    
    crate::kernel::net::sys_getsockopt(sockfd, level, optname, optval, optlen)
}

/// gettimeofday
fn sys_gettimeofday(_args: &[u64]) -> i64 {
    // TODO: Implement gettimeofday
    0
}

/// clock_gettime
fn sys_clock_gettime(_args: &[u64]) -> i64 {
    // TODO: Implement clock_gettime
    0
}

/// nanosleep
fn sys_nanosleep(_args: &[u64]) -> i64 {
    // TODO: Implement nanosleep
    0
}

/// sched_yield
fn sys_sched_yield(_args: &[u64]) -> i64 {
    crate::kernel::sched::yield_cpu();
    0
}

/// getuid
fn sys_getuid(_args: &[u64]) -> i64 {
    // TODO: Get current UID
    0
}

/// getgid
fn sys_getgid(_args: &[u64]) -> i64 {
    // TODO: Get current GID
    0
}

/// setuid
fn sys_setuid(_args: &[u64]) -> i64 {
    // TODO: Implement setuid
    -1  /* EPERM */
}

/// setgid
fn sys_setgid(_args: &[u64]) -> i64 {
    // TODO: Implement setgid
    -1
}

/// getppid
fn sys_getppid(_args: &[u64]) -> i64 {
    // TODO: Get parent PID
    0
}

/// setsid
fn sys_setsid(_args: &[u64]) -> i64 {
    // TODO: Implement setsid
    -1
}

/// Core system call implementations

/// getpid - Get current process ID
fn sys_getpid_impl(_args: &[u64]) -> i64 {
    // TODO: Get from current process
    // let current = current_process();
    // current.pid as i64
    1  // Return PID 1 for now
}

/// gettid - Get current thread ID
fn sys_gettid_impl(_args: &[u64]) -> i64 {
    // TODO: Get from current thread
    // let current = current_thread();
    // current.tid as i64
    1  // Return TID 1 for now
}

/// fork - Create a child process
fn sys_fork_impl(_args: &[u64]) -> i64 {
    // Call fork implementation
    crate::kernel::process::fork::sys_fork()
}

/// vfork - Create a child process and block parent
fn sys_vfork_impl(_args: &[u64]) -> i64 {
    // Call vfork implementation
    crate::kernel::process::fork::sys_vfork()
}

/// clone - Create a child process with more control
fn sys_clone_impl(args: &[u64]) -> i64 {
    let flags = args[0];
    // Call clone implementation
    crate::kernel::process::fork::sys_clone(flags)
}

/// execve - Execute a program
fn sys_execve_impl(args: &[u64]) -> i64 {
    let filename = args[0] as *const u8;
    let argv = args[1] as *const *const u8;
    let envp = args[2] as *const *const u8;
    
    // Call execve implementation
    crate::kernel::process::execve::sys_execve(filename, argv, envp)
}

/// exit - Terminate the current process
fn sys_exit_impl(args: &[u64]) -> i64 {
    let status = args[0] as i64;
    
    // Call exit implementation
    crate::kernel::process::sys_exit(status as i32);
    
    // Should never reach here
    0
}

/// exit_group - Terminate all threads in the process
fn sys_exit_group_impl(args: &[u64]) -> i64 {
    let status = args[0] as i64;
    
    // Call exit_group implementation
    crate::kernel::process::sys_exit_group(status as i32);
    
    // Should never reach here
    0
}

/// wait4 - Wait for a child process
fn sys_wait4_impl(args: &[u64]) -> i64 {
    let pid = args[0] as i64;
    let status = args[1] as *mut i32;
    let options = args[2] as i32;
    let rusage = args[3] as *mut u8;
    
    // Call wait4 implementation
    crate::kernel::process::wait4::sys_wait4(pid as i32, status, options, rusage)
}

/// mmap - Map files or devices into memory
fn sys_mmap_impl(args: &[u64]) -> i64 {
    let addr = args[0];
    let length = args[1];
    let prot = args[2] as i64;
    let flags = args[3] as i64;
    let fd = args[4] as i64;
    let offset = args[5];
    
    // Call mmap implementation
    crate::kernel::mm::mmap::sys_mmap(addr, length as usize, prot as i32, flags as i32, fd as i32, offset)
}

/// munmap - Unmap files or devices from memory
fn sys_munmap_impl(args: &[u64]) -> i64 {
    let addr = args[0];
    let length = args[1];
    
    // Call munmap implementation
    crate::kernel::mm::mmap::sys_munmap(addr, length as usize)
}

/// mprotect - Set protection on a region of memory
fn sys_mprotect_impl(args: &[u64]) -> i64 {
    let addr = args[0];
    let len = args[1];
    let prot = args[2] as i64;
    
    // Call mprotect implementation
    crate::kernel::mm::mmap::sys_mprotect(addr, len as usize, prot as i32)
}

/// brk - Change data segment size
fn sys_brk_impl(args: &[u64]) -> i64 {
    let addr = args[0];
    
    // Call brk implementation
    crate::kernel::mm::mmap::sys_brk(addr)
}

/// umask
fn sys_umask(_args: &[u64]) -> i64 {
    // TODO: Implement umask
    0o022
}

/// uname
fn sys_uname(_args: &[u64]) -> i64 {
    // TODO: Implement uname
    0
}

/// sysinfo
fn sys_sysinfo(_args: &[u64]) -> i64 {
    // TODO: Implement sysinfo
    0
}

/// reboot
fn sys_reboot(args: &[u64]) -> i64 {
    let magic1 = args[0] as u32;
    let magic2 = args[1] as u32;
    let cmd = args[2] as u32;
    
    // Check magic numbers
    if magic1 != 0xfee1dead || magic2 != 672274793 {
        return Errno::Einval.to_syscall_return();  /* EINVAL */
    }
    
    match cmd {
        0x01234567 => {
            // RB_AUTOBOOT
            log_info!("System rebooting...");
            // TODO: Implement reboot
        }
        0xCDEF0123 => {
            // RB_HALT_SYSTEM
            log_info!("System halting...");
            // TODO: Implement halt
        }
        0x4321FEDC => {
            // RB_POWER_OFF
            log_info!("System powering off...");
            // TODO: Implement power off
        }
        _ => {
            return Errno::Einval.to_syscall_return();
        }
    }
    
    0
}

/// pipe
fn sys_pipe(_args: &[u64]) -> i64 {
    // TODO: Implement pipe
    -1
}

/// pipe2
fn sys_pipe2(_args: &[u64]) -> i64 {
    // TODO: Implement pipe2
    -1
}
