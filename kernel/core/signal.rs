/*
 * Nuva OS - Kernel - Core - Signal
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
use crate::{pr_debug, pr_info};
/*
 * Nuva OS - Kernel - Signal Handling
 * 
 * Copyright (C) 2026 Nuva OS Team
 * 
 * POSIX signal handling implementation.
 */

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

use crate::syslib::posix::errno::Errno;
use crate::kernel::error::Errno;
/// Signal Number
pub mod signal {
    /// Hangup
    pub const SIGHUP: i32 = 1;
    /// Interrupt
    pub const SIGINT: i32 = 2;
    /// Quit
    pub const SIGQUIT: i32 = 3;
    /// Illegal instruction
    pub const SIGILL: i32 = 4;
    /// Trap
    pub const SIGTRAP: i32 = 5;
    /// Abort
    pub const SIGABRT: i32 = 6;
    /// Bus error
    pub const SIGBUS: i32 = 7;
    /// Floating point exception
    pub const SIGFPE: i32 = 8;
    /// Kill
    pub const SIGKILL: i32 = 9;
    /// User defined 1
    pub const SIGUSR1: i32 = 10;
    /// Segmentation fault
    pub const SIGSEGV: i32 = 11;
    /// User defined 2
    pub const SIGUSR2: i32 = 12;
    /// Pipe
    pub const SIGPIPE: i32 = 13;
    /// Alarm
    pub const SIGALRM: i32 = 14;
    /// Terminate
    pub const SIGTERM: i32 = 15;
    /// Stack fault
    pub const SIGSTKFLT: i32 = 16;
    /// Child
    pub const SIGCHLD: i32 = 17;
    /// Continue
    pub const SIGCONT: i32 = 18;
    /// Stop
    pub const SIGSTOP: i32 = 19;
    /// Terminal stop
    pub const SIGTSTP: i32 = 20;
    /// Terminal input
    pub const SIGTTIN: i32 = 21;
    /// Terminal output
    pub const SIGTTOU: i32 = 22;
    /// Urgent
    pub const SIGURG: i32 = 23;
    /// CPU limit
    pub const SIGXCPU: i32 = 24;
    /// File size limit
    pub const SIGXFSZ: i32 = 25;
    /// Virtual alarm
    pub const SIGVTALRM: i32 = 26;
    /// Profiling alarm
    pub const SIGPROF: i32 = 27;
    /// Window change
    pub const SIGWINCH: i32 = 28;
    /// I/O
    pub const SIGIO: i32 = 29;
    /// Power
    pub const SIGPWR: i32 = 30;
    /// System call
    pub const SIGSYS: i32 = 31;
    
    /// Number of signals
    pub const NSIG: i32 = 64;
}

/// Signal Action Flags
bitflags::bitflags! {
    #[repr(transparent)]
    pub struct SaFlags: u32 {
        /// No child stop
        const NOCLDSTOP = 0x00000001;
        /// No child wait
        const NOCLDWAIT = 0x00000002;
        /// Restart
        const RESTART = 0x10000000;
        /// Interrupt
        const INTERRUPT = 0x20000000;
        /// No defer
        const NODEFER = 0x40000000;
        /// Reset handler
        const RESETHAND = 0x80000000;
    }
}

/// Signal Handler Type
pub type SigHandler = extern "C" fn(i32);

/// Signal Action
#[repr(C)]
pub struct SigAction {
    /// Handler function
    pub handler: Option<SigHandler>,
    /// Signal mask
    pub signal_mask: u64,
    /// Flags
    pub signal_flags: SaFlags,
    /// Restorer function
    pub restorer: Option<extern "C" fn()>,
}

impl SigAction {
    pub const fn new() -> Self {
        SigAction {
            handler: None,
            signal_mask: 0,
            signal_flags: SaFlags::empty(),
            restorer: None,
        }
    }
    
    /// Check if handler is SIG_DFL (default)
    pub fn is_default(&self) -> bool {
        self.handler.is_none()
    }
    
    /// Check if handler is SIG_IGN (ignore)
    pub fn is_ignore(&self) -> bool {
        // SIG_IGN is represented by a special handler value
        false
    }
}

/// Signal Info
#[repr(C)]
pub struct SigInfo {
    /// Signal number
    pub si_signo: i32,
    /// Error number
    pub si_errno: i32,
    /// Signal code
    pub si_code: i32,
    /// Sending process ID
    pub si_pid: u32,
    /// Sending user ID
    pub si_uid: u32,
    /// Signal value
    pub si_value: SigVal,
    /// Faulting address
    pub si_addr: u64,
    /// Timer ID
    pub si_timerid: i32,
    /// Overflow count
    pub si_overrun: i32,
    /// Exit status
    pub si_status: i32,
    /// Band event
    pub si_band: i64,
}

/// Signal Value
#[repr(C)]
pub union SigVal {
    pub sival_int: i32,
    pub sival_ptr: *mut core::ffi::c_void,
}

/// Signal Code
pub mod sig_code {
    /// User sent
    pub const SI_USER: i32 = 0;
    /// Kernel sent
    pub const SI_KERNEL: i32 = 0x80;
    /// Queue
    pub const SI_QUEUE: i32 = -1;
    /// Timer
    pub const SI_TIMER: i32 = -2;
    /// Asynchronous I/O
    pub const SI_ASYNCIO: i32 = -4;
    /// Message queue
    pub const SI_MESGQ: i32 = -3;
    /// Signal
    pub const SI_SIGIO: i32 = -5;
    /// Trap
    pub const SI_TKILL: i32 = -6;
    
    // SIGSEGV codes
    pub const SEGV_MAPERR: i32 = 1;
    pub const SEGV_ACCERR: i32 = 2;
    
    // SIGBUS codes
    pub const BUS_ADRALN: i32 = 1;
    pub const BUS_ADRERR: i32 = 2;
    pub const BUS_OBJERR: i32 = 3;
    
    // SIGFPE codes
    pub const FPE_INTDIV: i32 = 1;
    pub const FPE_INTOVF: i32 = 2;
    pub const FPE_FLTDIV: i32 = 3;
    pub const FPE_FLTOVF: i32 = 4;
    pub const FPE_FLTUND: i32 = 5;
    pub const FPE_FLTRES: i32 = 6;
    pub const FPE_FLTINV: i32 = 7;
    pub const FPE_FLTSUB: i32 = 8;
    
    // SIGILL codes
    pub const ILL_ILLOPC: i32 = 1;
    pub const ILL_ILLOPN: i32 = 2;
    pub const ILL_ILLADR: i32 = 3;
    pub const ILL_ILLTRP: i32 = 4;
    pub const ILL_PRVOPC: i32 = 5;
    pub const ILL_PRVREG: i32 = 6;
    pub const ILL_COPROC: i32 = 7;
    pub const ILL_BADSTK: i32 = 8;
    
    // SIGTRAP codes
    pub const TRAP_BRKPT: i32 = 1;
    pub const TRAP_TRACE: i32 = 2;
}

/// Signal Set
#[repr(C)]
pub struct SigSet {
    pub bits: [u64; 2], // 128 bits for 64 signals
}

impl SigSet {
    pub const fn new() -> Self {
        SigSet { bits: [0; 2] }
    }
    
    /// Add signal to set
    pub fn add(&mut self, sig: i32) {
        if sig > 0 && sig <= 64 {
            self.bits[0] |= 1u64 << (sig - 1);
        } else if sig > 64 && sig <= 128 {
            self.bits[1] |= 1u64 << (sig - 65);
        }
    }
    
    /// Remove signal from set
    pub fn remove(&mut self, sig: i32) {
        if sig > 0 && sig <= 64 {
            self.bits[0] &= !(1u64 << (sig - 1));
        } else if sig > 64 && sig <= 128 {
            self.bits[1] &= !(1u64 << (sig - 65));
        }
    }
    
    /// Check if signal is in set
    pub fn contains(&self, sig: i32) -> bool {
        if sig > 0 && sig <= 64 {
            (self.bits[0] & (1u64 << (sig - 1))) != 0
        } else if sig > 64 && sig <= 128 {
            (self.bits[1] & (1u64 << (sig - 65))) != 0
        } else {
            false
        }
    }
    
    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.bits[0] == 0 && self.bits[1] == 0
    }
    
    /// Clear all signals
    pub fn clear(&mut self) {
        self.bits = [0; 2];
    }
    
    /// Fill all signals
    pub fn fill(&mut self) {
        self.bits = [u64::MAX; 2];
    }
}

/// Pending Signal
#[repr(C)]
pub struct PendingSig {
    /// Signal info
    pub info: SigInfo,
    /// Next pending signal
    pub next: *mut PendingSig,
}

/// Signal Pending
pub struct SigPending {
    /// Signal set
    pub signal: SigSet,
    /// Pending list
    pub list: *mut PendingSig,
}

impl SigPending {
    pub const fn new() -> Self {
        SigPending {
            signal: SigSet::new(),
            list: core::ptr::null_mut(),
        }
    }
}

/// Signal Manager
pub struct SigManager {
    /// Statistics
    pub stats: SigStats,
}

/// Signal Statistics
pub struct SigStats {
    /// Signals sent
    pub sent: AtomicU64,
    /// Signals delivered
    pub delivered: AtomicU64,
    /// Signals dropped
    pub dropped: AtomicU64,
}

impl SigStats {
    pub const fn new() -> Self {
        SigStats {
            sent: AtomicU64::new(0),
            delivered: AtomicU64::new(0),
            dropped: AtomicU64::new(0),
        }
    }
}

impl SigManager {
    pub const fn new() -> Self {
        SigManager {
            stats: SigStats::new(),
        }
    }
    
    /// Initialize
    pub fn init(&self) {
        log_info!("Signal manager initialized");
    }
    
    /// Send signal to process
    pub fn kill(&mut self, pid: u32, sig: i32) -> i32 {
        self.stats.sent.fetch_add(1, Ordering::AcqRel);
        
        if sig < 0 || sig > signal::NSIG {
            return Errno::Einval.to_ret_i32(); // EINVAL
        }
        
        // TODO: Find process and send signal
        log_debug!("kill: pid={}, sig={}", pid, sig);
        0
    }
    
    /// Send signal to thread
    pub fn tgkill(&mut self, tgid: u32, tid: u32, sig: i32) -> i32 {
        self.stats.sent.fetch_add(1, Ordering::AcqRel);
        
        if sig < 0 || sig > signal::NSIG {
            return Errno::Einval.to_ret_i32();
        }
        
        // TODO: Find thread and send signal
        log_debug!("tgkill: tgid={}, tid={}, sig={}", tgid, tid, sig);
        0
    }
    
    /// Send signal to process group
    pub fn killpg(&mut self, pgrp: u32, sig: i32) -> i32 {
        self.stats.sent.fetch_add(1, Ordering::AcqRel);
        
        if sig < 0 || sig > signal::NSIG {
            return Errno::Einval.to_ret_i32();
        }
        
        // TODO: Find process group and send signal
        log_debug!("killpg: pgrp={}, sig={}", pgrp, sig);
        0
    }
    
    /// Check if signal is pending
    pub fn is_pending(&self, _sig: i32) -> bool {
        // TODO: Check pending signals
        false
    }
    
    /// Get pending signals
    pub fn get_pending(&self) -> SigSet {
        // TODO: Get pending signals
        SigSet::new()
    }
    
    /// Block signals
    pub fn sigprocmask(&mut self, _how: i32, _set: &SigSet, _oldset: *mut SigSet) -> i32 {
        // TODO: Implement signal mask
        0
    }
    
    /// Suspend until signal
    pub fn sigsuspend(&mut self, _mask: &SigSet) -> i32 {
        // TODO: Implement sigsuspend
        -4 // EINTR
    }
    
    /// Wait for signal
    pub fn sigwaitinfo(&mut self, _set: &SigSet, _info: *mut SigInfo) -> i32 {
        // TODO: Implement sigwaitinfo
        -4 // EINTR
    }
    
    /// Alternate signal stack
    pub fn sigaltstack(&mut self, _ss: *mut SigAltStack, _old_ss: *mut SigAltStack) -> i32 {
        // TODO: Implement sigaltstack
        0
    }
}

/// Signal Alternate Stack
#[repr(C)]
pub struct SigAltStack {
    pub ss_sp: *mut core::ffi::c_void,
    pub ss_flags: i32,
    pub ss_size: usize,
}

/// Global signal manager
static SIG_MANAGER: crate::sync_oncelock::OnceLock<SigManager> = crate::sync_oncelock::OnceLock::new();

/// Get signal manager
pub fn sig_manager() -> &'static SigManager {
    SIG_MANAGER.get_or_init(SigManager::new)
}

pub fn init_sig_manager() -> &'static SigManager {
    SIG_MANAGER.get_or_init(SigManager::new)
}

/// Initialize signal
pub fn init_signal() {
    let mgr = sig_manager();
    mgr.init();
}

// System call wrappers

/// sys_kill
pub fn sys_kill(pid: u32, sig: i32) -> i64 {
    sig_manager().kill(pid, sig) as i64
}

/// sys_tgkill
pub fn sys_tgkill(tgid: u32, tid: u32, sig: i32) -> i64 {
    sig_manager().tgkill(tgid, tid, sig) as i64
}

/// sys_sigprocmask
pub fn sys_sigprocmask(how: i32, set: *const SigSet, oldset: *mut SigSet) -> i64 {
    if set.is_null() {
        return Errno::Einval.to_syscall_return();
    }
    // SAFETY: unsafe block required for low-level memory or hardware access
    sig_manager().sigprocmask(how, unsafe { &*set }, oldset) as i64
}

/// sys_sigaction
pub fn sys_sigaction(sig: i32, act: *const SigAction, oldact: *mut SigAction) -> i64 {
    if sig < 1 || sig > signal::NSIG {
        return Errno::Einval.to_syscall_return();
    }
    
    // TODO: Implement sigaction
    let _ = (act, oldact);
    0
}

/// sys_sigsuspend
pub fn sys_sigsuspend(mask: *const SigSet) -> i64 {
    if mask.is_null() {
        return Errno::Einval.to_syscall_return();
    }
    // SAFETY: unsafe block required for low-level memory or hardware access
    sig_manager().sigsuspend(unsafe { &*mask }) as i64
}

/// sys_sigpending
pub fn sys_sigpending(set: *mut SigSet) -> i64 {
    if set.is_null() {
        return Errno::Einval.to_syscall_return();
    }
    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe {
        *set = sig_manager().get_pending();
    }
    0
}
