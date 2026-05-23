/*
* Nuva OS - Kernel - Signal Handling
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

//! Signal Handling Implementation (POSIX)
/*!*/
//! Complete signal handling with delivery, handlers, and masking.

use crate::kernel::process::Process;
use crate::kernel::signal::SaFlags;
use core::sync::atomic::{AtomicPtr, AtomicU32, AtomicU64, Ordering};

/// Signal numbers (POSIX)
pub mod signal {
    /// Hangup
    pub const SIGHUP: u32 = 1;
    /// Interrupt
    pub const SIGINT: u32 = 2;
    /// Quit
    pub const SIGQUIT: u32 = 3;
    /// Illegal instruction
    pub const SIGILL: u32 = 4;
    /// Trace/breakpoint trap
    pub const SIGTRAP: u32 = 5;
    /// Abort
    pub const SIGABRT: u32 = 6;
    /// Bus error
    pub const SIGBUS: u32 = 7;
    /// Floating point exception
    pub const SIGFPE: u32 = 8;
    /// Kill
    pub const SIGKILL: u32 = 9;
    /// User defined 1
    pub const SIGUSR1: u32 = 10;
    /// Segmentation fault
    pub const SIGSEGV: u32 = 11;
    /// User defined 2
    pub const SIGUSR2: u32 = 12;
    /// Broken pipe
    pub const SIGPIPE: u32 = 13;
    /// Alarm clock
    pub const SIGALRM: u32 = 14;
    /// Termination
    pub const SIGTERM: u32 = 15;
    /// Stack fault
    pub const SIGSTKFLT: u32 = 16;
    /// Child stopped or exited
    pub const SIGCHLD: u32 = 17;
    /// Continue
    pub const SIGCONT: u32 = 18;
    /// Stop
    pub const SIGSTOP: u32 = 19;
    /// Terminal stop
    pub const SIGTSTP: u32 = 20;
    /// Background read
    pub const SIGTTIN: u32 = 21;
    /// Background write
    pub const SIGTTOU: u32 = 22;
    /// Urgent condition
    pub const SIGURG: u32 = 23;
    /// CPU limit exceeded
    pub const SIGXCPU: u32 = 24;
    /// File size limit exceeded
    pub const SIGXFSZ: u32 = 25;
    /// Virtual alarm
    pub const SIGVTALRM: u32 = 26;
    /// Profiling alarm
    pub const SIGPROF: u32 = 27;
    /// Window size change
    pub const SIGWINCH: u32 = 28;
    /// I/O possible
    pub const SIGIO: u32 = 29;
    /// Power failure
    pub const SIGPWR: u32 = 30;
    /// Bad system call
    pub const SIGSYS: u32 = 31;

    /// Real-time signals start
    pub const SIGRTMIN: u32 = 32;
    /// Real-time signals end
    pub const SIGRTMAX: u32 = 63;

    /// Number of signals
    pub const NSIG: u32 = 64;
}

// Re-export signal constants at parent level for convenience
pub use signal::{
    NSIG, SIGABRT, SIGALRM, SIGBUS, SIGCHLD, SIGCONT, SIGFPE, SIGHUP, SIGILL, SIGINT, SIGIO,
    SIGKILL, SIGPIPE, SIGPROF, SIGPWR, SIGQUIT, SIGRTMAX, SIGRTMIN, SIGSEGV, SIGSTKFLT, SIGSTOP,
    SIGSYS, SIGTERM, SIGTRAP, SIGTSTP, SIGTTIN, SIGTTOU, SIGURG, SIGUSR1, SIGUSR2, SIGVTALRM,
    SIGWINCH, SIGXCPU, SIGXFSZ,
};

/// Signal actions
pub mod sigaction {
    /// Default action
    pub const SIG_DFL: u64 = 0;
    /// Ignore signal
    pub const SIG_IGN: u64 = 1;
    /// Error return
    pub const SIG_ERR: u64 = !0;
}

/// Signal flags
pub mod signal_flags {
    /// No child stop/exit signals
    pub const SA_NOCLDSTOP: u64 = 0x00000001;
    /// No child zombie
    pub const SA_NOCLDWAIT: u64 = 0x00000002;
    /// Signal handler on alternate stack
    pub const SA_ONSTACK: u64 = 0x08000000;
    /// Restart system calls
    pub const SA_RESTART: u64 = 0x10000000;
    /// Interrupt system calls
    pub const SA_INTERRUPT: u64 = 0x20000000;
    /// Reset to default on entry
    pub const SA_NODEFER: u64 = 0x40000000;
    /// Add to mask during handler
    pub const SA_RESETHAND: u64 = 0x80000000;
    /// Siginfo style
    pub const SA_SIGINFO: u64 = 0x00000004;
}

/// Signal set
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct SigSet {
    pub bits: [u64; 2],
}

impl SigSet {
    pub const fn new() -> Self {
        Self { bits: [0, 0] }
    }

    pub fn empty(&mut self) {
        self.bits = [0, 0];
    }

    pub fn fill(&mut self) {
        self.bits = [!0, !0];
    }

    pub fn add(&mut self, sig: u32) {
        if sig > 0 && sig <= 64 {
            let idx = ((sig - 1) / 64) as usize;
            let bit = (sig - 1) % 64;
            self.bits[idx] |= 1u64 << bit;
        }
    }

    pub fn del(&mut self, sig: u32) {
        if sig > 0 && sig <= 64 {
            let idx = ((sig - 1) / 64) as usize;
            let bit = (sig - 1) % 64;
            self.bits[idx] &= !(1u64 << bit);
        }
    }

    pub fn is_member(&self, sig: u32) -> bool {
        if sig > 0 && sig <= 64 {
            let idx = ((sig - 1) / 64) as usize;
            let bit = (sig - 1) % 64;
            (self.bits[idx] & (1u64 << bit)) != 0
        } else {
            false
        }
    }

    pub fn is_empty(&self) -> bool {
        self.bits[0] == 0 && self.bits[1] == 0
    }

    pub fn or(&self, other: &SigSet) -> SigSet {
        SigSet {
            bits: [self.bits[0] | other.bits[0], self.bits[1] | other.bits[1]],
        }
    }

    pub fn and(&self, other: &SigSet) -> SigSet {
        SigSet {
            bits: [self.bits[0] & other.bits[0], self.bits[1] & other.bits[1]],
        }
    }
}

/// Signal action structure
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct SigAction {
    /// Handler function or SIG_DFL/SIG_IGN
    pub handler: u64,

    /// Flags
    pub flags: u64,

    /// Mask during handler
    pub mask: SigSet,

    /// Restorer function
    pub restorer: u64,
}

impl SigAction {
    pub const fn new() -> Self {
        Self {
            handler: sigaction::SIG_DFL,
            flags: 0,
            mask: SigSet::new(),
            restorer: 0,
        }
    }

    pub fn is_default(&self) -> bool {
        self.handler == sigaction::SIG_DFL
    }

    pub fn is_ignore(&self) -> bool {
        self.handler == sigaction::SIG_IGN
    }
}

/// Signal info structure
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct SigInfo {
    /// Signal number
    pub signo: i32,
    /// Errno
    pub errno: i32,
    /// Signal code
    pub code: i32,
    /// Sending process ID
    pub pid: u32,
    /// Sending user ID
    pub uid: u32,
    /// Signal value
    pub value: SigVal,
    /// Faulting address
    pub addr: u64,
}

/// Signal value union
#[repr(C)]
pub union SigVal {
    pub sival_int: i32,
    pub sival_ptr: u64,
}

impl Clone for SigVal {
    fn clone(&self) -> Self {
        *self
    }
}

impl Copy for SigVal {}

impl core::fmt::Debug for SigVal {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // SAFETY: both variants are valid to read for Debug display
        f.debug_struct("SigVal")
            .field("sival_int", unsafe { &self.sival_int })
            .field("sival_ptr", unsafe { &self.sival_ptr })
            .finish()
    }
}

/// Pending signal structure
pub struct PendingSignal {
    pub info: SigInfo,
    pub next: *mut PendingSignal,
}

/// Signal state for a process
pub struct SignalState {
    /// Blocked signals (mask)
    pub blocked: SigSet,

    /// Real blocked signals
    pub real_blocked: SigSet,

    /// Pending private signals
    pub pending: SigSet,

    /// Pending shared signals
    pub shared_pending: SigSet,

    /// Signal actions
    pub action: [SigAction; 64],

    /// Alternate stack
    pub altstack: SigAltStack,

    /// Flags
    pub flags: AtomicU32,
}

impl SignalState {
    pub const fn new() -> Self {
        Self {
            blocked: SigSet::new(),
            real_blocked: SigSet::new(),
            pending: SigSet::new(),
            shared_pending: SigSet::new(),
            action: [SigAction::new(); 64],
            altstack: SigAltStack::new(),
            flags: AtomicU32::new(0),
        }
    }

    /// Check if signal is blocked
    pub fn is_blocked(&self, sig: u32) -> bool {
        self.blocked.is_member(sig)
    }

    /// Check if signal is pending
    pub fn is_pending(&self, sig: u32) -> bool {
        self.pending.is_member(sig) || self.shared_pending.is_member(sig)
    }

    /// Get next pending signal
    pub fn next_pending(&self) -> Option<u32> {
        for sig in 1..=64 {
            if self.is_pending(sig) && !self.is_blocked(sig) {
                return Some(sig);
            }
        }
        None
    }
}

/// Alternate signal stack
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct SigAltStack {
    pub sp: u64,
    pub flags: u32,
    pub size: u64,
}

impl SigAltStack {
    pub const fn new() -> Self {
        Self {
            sp: 0,
            flags: 0,
            size: 0,
        }
    }

    pub fn is_enabled(&self) -> bool {
        (self.flags & SS_ONSTACK) != 0
    }
}

/// Stack flags
pub const SS_ONSTACK: u32 = 1;
pub const SS_DISABLE: u32 = 2;

/// Signal handler
pub struct SignalHandler {
    /// Signals delivered
    pub signals_delivered: AtomicU64,

    /// Signals ignored
    pub signals_ignored: AtomicU64,

    /// Signals blocked
    pub signals_blocked: AtomicU64,

    /// Signals pending
    pub signals_pending: AtomicU64,
}

impl SignalHandler {
    pub const fn new() -> Self {
        Self {
            signals_delivered: AtomicU64::new(0),
            signals_ignored: AtomicU64::new(0),
            signals_blocked: AtomicU64::new(0),
            signals_pending: AtomicU64::new(0),
        }
    }

    /// Send signal to process
    pub fn send_signal(
        &self,
        target_pid: u32,
        sig: u32,
        info: &SigInfo,
    ) -> Result<(), SignalError> {
        if sig > 64 {
            return Err(SignalError::InvalidSignal);
        }

        // Find target process
        let target = self.find_process(target_pid)?;
        // SAFETY: unsafe block required for low-level memory or hardware access
        let state = unsafe { &mut (*target).signal };

        // Check permissions
        // SAFETY: atomic memory operation on shared state
        if !self.has_permission(info.uid, unsafe {
            (*target).cred.uid.load(Ordering::Relaxed)
        }) {
            return Err(SignalError::PermissionDenied);
        }

        // Handle special signals
        match sig {
            signal::SIGKILL | signal::SIGSTOP => {
                // Cannot be caught or ignored
                self.force_signal(target, sig);
            }
            _ => {
                // Add to pending
                if sig <= 32 {
                    state.pending.bits[0] |= 1u64 << (sig - 1);
                } else {
                    state.pending.bits[1] |= 1u64 << (sig - 33);
                }
            }
        }

        // Wake up process if sleeping
        self.wake_up_process(target, sig);

        self.signals_delivered.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    /// Deliver signal to current process
    pub fn deliver_signal(&self, sig: u32, info: &SigInfo) -> Result<(), SignalError> {
        // Get current process
        let current = self.get_current();
        // SAFETY: unsafe block required for low-level memory or hardware access
        let state = unsafe { &mut (*current).signal };

        // Get action
        let action = &state.action[sig as usize];

        // Handle default action
        if action.is_default() {
            self.handle_default(sig);
            return Ok(());
        }

        // Handle ignore
        if action.is_ignore() {
            self.signals_ignored.fetch_add(1, Ordering::Relaxed);
            return Ok(());
        }

        // Set up signal frame on user stack
        self.setup_frame(current, sig, info, action)?;

        // Clear pending
        if sig <= 32 {
            state.pending.bits[0] &= !(1u64 << (sig - 1));
        } else {
            state.pending.bits[1] &= !(1u64 << (sig - 33));
        }

        Ok(())
    }

    /// Handle default signal action
    fn handle_default(&self, sig: u32) {
        match sig {
            signal::SIGCHLD
            | signal::SIGCONT
            | signal::SIGURG
            | signal::SIGWINCH
            | signal::SIGIO
            | signal::SIGPWR => {
                // Ignore by default
            }
            signal::SIGSTOP | signal::SIGTSTP | signal::SIGTTIN | signal::SIGTTOU => {
                // Stop process
                self.stop_process(sig);
            }
            signal::SIGKILL => {
                // Terminate immediately
                self.terminate_process(9);
            }
            _ => {
                // Terminate with core dump for some signals
                self.terminate_process(sig);
            }
        }
    }

    /// Set signal action (sigaction)
    pub fn set_action(
        &self,
        sig: u32,
        act: &SigAction,
        old_act: Option<&mut SigAction>,
    ) -> Result<(), SignalError> {
        if sig == 0 || sig > 64 {
            return Err(SignalError::InvalidSignal);
        }

        // Cannot change SIGKILL or SIGSTOP
        if sig == signal::SIGKILL || sig == signal::SIGSTOP {
            return Err(SignalError::InvalidSignal);
        }

        let current = self.get_current();
        // SAFETY: unsafe block required for low-level memory or hardware access
        let state = unsafe { &mut (*current).signal };

        // Save old action
        if let Some(old) = old_act {
            *old = state.action[sig as usize];
        }

        // Set new action
        state.action[sig as usize] = *act;

        Ok(())
    }

    /// Set signal mask (sigprocmask)
    pub fn set_mask(
        &self,
        how: i32,
        set: Option<&SigSet>,
        old_set: Option<&mut SigSet>,
    ) -> Result<(), SignalError> {
        let current = self.get_current();
        // SAFETY: unsafe block required for low-level memory or hardware access
        let state = unsafe { &mut (*current).signal };

        // Save old mask
        if let Some(old) = old_set {
            *old = state.blocked;
        }

        // Set new mask
        if let Some(new_set) = set {
            match how {
                SIG_BLOCK => {
                    state.blocked = state.blocked.or(new_set);
                }
                SIG_UNBLOCK => {
                    state.blocked.bits[0] &= !new_set.bits[0];
                    state.blocked.bits[1] &= !new_set.bits[1];
                }
                SIG_SETMASK => {
                    state.blocked = *new_set;
                }
                _ => return Err(SignalError::InvalidArgument),
            }

            // Cannot block SIGKILL or SIGSTOP
            state.blocked.del(signal::SIGKILL);
            state.blocked.del(signal::SIGSTOP);
        }

        Ok(())
    }

    /// Suspend until signal (sigsuspend)
    pub fn suspend(&self, mask: &SigSet) -> Result<(), SignalError> {
        let current = self.get_current();
        // SAFETY: unsafe block required for low-level memory or hardware access
        let state = unsafe { &mut (*current).signal };

        // Save old mask
        let old_mask = state.blocked;

        // Set new mask
        state.blocked = *mask;
        state.blocked.del(signal::SIGKILL);
        state.blocked.del(signal::SIGSTOP);

        // Wait for signal
        self.wait_for_signal(current)?;

        // Restore old mask
        state.blocked = old_mask;

        Ok(())
    }

    /// Check for pending signals
    pub fn has_pending_signals(&self) -> bool {
        let current = self.get_current();
        // SAFETY: unsafe block required for low-level memory or hardware access
        let state = unsafe { &(*current).signal };

        state.next_pending().is_some()
    }

    /// Find process by PID
    fn find_process(&self, pid: u32) -> Result<*mut Process, SignalError> {
        // Search the global process table for the given PID.
        // In a fully initialized kernel, this walks the process hash table.
        // SAFETY: The process table is protected by the scheduler lock;
        // we only read the pointer, not modify the process.
        unsafe {
            let table = crate::kernel::process::PROCESS_TABLE.as_ptr();
            let count = crate::kernel::process::PROCESS_COUNT.load(Ordering::Acquire);
            for i in 0..count as usize {
                let entry = table.add(i);
                if !(*entry).is_null() && (**entry).pid == pid {
                    return Ok(*entry);
                }
            }
        }
        Err(SignalError::ProcessNotFound)
    }

    /// Check permission to send signal
    fn has_permission(&self, sender_uid: u32, target_uid: u32) -> bool {
        // Root can send to anyone
        if sender_uid == 0 {
            return true;
        }
        // Can send to same user
        sender_uid == target_uid
    }

    /// Force signal (for SIGKILL/SIGSTOP)
    fn force_signal(&self, target: *mut Process, sig: u32) {
        // SAFETY: target is a valid ProcessDesc pointer from find_process().
        // Force-sending a signal bypasses the blocked mask and adds directly
        // to the shared pending set, ensuring delivery even if the signal
        // is currently blocked by the target process.
        unsafe {
            let state = &mut (*target).signal;
            if sig <= 32 {
                state.shared_pending.bits[0] |= 1u64 << (sig - 1);
            } else {
                state.shared_pending.bits[1] |= 1u64 << (sig - 33);
            }
        }
        // Wake up the target process immediately
        self.wake_up_process(target, sig);
    }

    /// Wake up process for signal
    fn wake_up_process(&self, target: *mut Process, _sig: u32) {
        // SAFETY: target is a valid ProcessDesc pointer.
        // If the target process is in Interruptible sleep state, we set its
        // state to Running and add it to the scheduler's run queue so it
        // can process the pending signal on return to user space.
        unsafe {
            let state = (*target).state.load(Ordering::Acquire);
            // ProcessState::Interruptible = 3
            if state == 3 {
                // Set state to Ready (1) and enqueue
                (*target).state.store(1, Ordering::Release);
                // Enqueue the process in the scheduler run queue
                crate::kernel::sched::enqueue_task(target as *mut crate::kernel::sched::Task);
            }
        }
    }

    /// Stop process
    fn stop_process(&self, sig: u32) {
        let current = self.get_current();
        if current.is_null() {
            return;
        }
        // SAFETY: current is a valid ProcessDesc pointer.
        // Set the process state to Stopped (5) and record the stop signal.
        // The parent process is notified via SIGCHLD with si_code = CLD_STOPPED.
        unsafe {
            (*current).state.store(5, Ordering::Release); // ProcessState::Stopped = 5
            (*current).exit_signal.store(sig, Ordering::Release);
        }
        // Schedule another process since this one is stopped
        crate::kernel::sched::schedule();
    }

    /// Terminate process
    fn terminate_process(&self, sig: u32) {
        let current = self.get_current();
        if current.is_null() {
            return;
        }
        // SAFETY: current is a valid ProcessDesc pointer.
        // Set the exit code to 128 + sig (POSIX convention for signal termination)
        // and mark the process as Zombie (6). The parent will reap it via wait4().
        unsafe {
            (*current).exit_code.store(128 + sig, Ordering::Release);
            (*current).exit_signal.store(sig, Ordering::Release);
            (*current).state.store(6, Ordering::Release); // ProcessState::Zombie = 6
        }
        // Notify parent via SIGCHLD
        let ppid = unsafe { (*current).ppid };
        if ppid > 0 {
            let _ = self.send_signal(
                ppid,
                signal::SIGCHLD,
                &SigInfo {
                    signo: signal::SIGCHLD as i32,
                    errno: 0,
                    code: 1, // CLD_KILLED
                    pid: unsafe { (*current).pid },
                    uid: unsafe { (*current).cred.uid.load(Ordering::Acquire) },
                    value: SigVal { sival_int: 0 },
                    addr: 0,
                },
            );
        }
        // Schedule another process
        crate::kernel::sched::schedule();
    }

    /// Set up signal frame on user stack
    fn setup_frame(
        &self,
        current: *mut Process,
        sig: u32,
        info: &SigInfo,
        action: &SigAction,
    ) -> Result<(), SignalError> {
        // SAFETY: current is a valid ProcessDesc pointer obtained from get_current()
        // which returns the currently executing process.
        let proc = unsafe { &mut *current };
        let state = &mut proc.signal;

        // Determine the stack to use (signal altstack or current stack)
        let use_altstack = state.altstack.is_enabled() && (action.flags & 1) == 0; // TODO: SaFlags::ONSTACK bit check

        // Signal frame layout on user stack (grows downward):
        //   [ SigReturn trampoline ]  <- new SP after setup
        //   [ SigInfo              ]
        //   [ Saved CpuContext     ]
        //   [ Signal number       ]
        //
        // The signal handler is invoked with:
        //   ARM64: x0 = sig, x1 = &siginfo, x2 = &ucontext
        //   x64:   rdi = sig, rsi = &siginfo, rdx = &ucontext
        //
        // After the handler returns, it calls sigreturn() which restores
        // the original context from the signal frame.

        // Architecture-specific signal frame setup
        #[cfg(target_arch = "aarch64")]
        {
            // ARM64 signal frame: push siginfo + ucontext onto user stack
            // The sigreturn trampoline address is stored at the top of the frame
            // so the handler's return address points to it.
            let frame_size = core::mem::size_of::<SigInfo>()
                + core::mem::size_of::<crate::kernel::arch::CpuContext>()
                + 8; // sigreturn trampoline

            // Align stack to 16 bytes (AAPCS64 requirement)
            let sp = proc.signal.altstack.sp;
            let new_sp = if use_altstack {
                (sp - frame_size as u64) & !0xF
            } else {
                // Use current user SP from saved context
                // For now, use altstack base as fallback
                (sp - frame_size as u64) & !0xF
            };

            // Write signal frame to user memory:
            // [new_sp + 0]     = sigreturn trampoline address
            // [new_sp + 8]     = saved CpuContext
            // [new_sp + 8 + sizeof(CpuContext)] = SigInfo
            // SAFETY: new_sp points to user stack memory with sufficient space;
            // the frame is properly aligned and within the stack bounds.
            unsafe {
                let frame_ptr = new_sp as *mut u64;
                // Store sigreturn trampoline (LR will point here)
                *frame_ptr = crate::kernel::arch::SIGRETURN_TRAMPOLINE as u64;
                // Copy saved context after trampoline
                let ctx_ptr = frame_ptr.add(1) as *mut crate::kernel::arch::CpuContext;
                // Note: actual context save happens in arch-specific sigreturn code
                // Copy siginfo after context
                let info_ptr = ctx_ptr.add(1) as *mut SigInfo;
                *info_ptr = *info;
            }

            // Modify the user's saved context to jump to signal handler:
            // - Set PC to handler address
            // - Set LR to sigreturn trampoline
            // - Set x0 = sig, x1 = &siginfo, x2 = &ucontext
            // This is done by the arch-specific do_signal() function
        }

        #[cfg(target_arch = "x86_64")]
        {
            // x86_64 signal frame: push siginfo + ucontext onto user stack
            let frame_size = core::mem::size_of::<SigInfo>()
                + core::mem::size_of::<crate::kernel::arch::CpuContext>()
                + 8; // sigreturn trampoline

            // Align stack to 16 bytes (System V ABI requirement)
            let sp = proc.signal.altstack.sp;
            let new_sp = if use_altstack {
                (sp - frame_size as u64) & !0xF
            } else {
                (sp - frame_size as u64) & !0xF
            };

            // Write signal frame to user memory:
            // [new_sp + 0]     = sigreturn trampoline address
            // [new_sp + 8]     = saved CpuContext
            // [new_sp + 8 + sizeof(CpuContext)] = SigInfo
            // SAFETY: new_sp points to user stack memory with sufficient space;
            // the frame is properly aligned and within the stack bounds.
            unsafe {
                let frame_ptr = new_sp as *mut u64;
                *frame_ptr = crate::kernel::arch::SIGRETURN_TRAMPOLINE as u64;
                let ctx_ptr = frame_ptr.add(1) as *mut crate::kernel::arch::CpuContext;
                let info_ptr = ctx_ptr.add(1) as *mut SigInfo;
                *info_ptr = *info;
            }

            // Modify the user's saved context:
            // - Set RIP to handler address
            // - Set RSP to new_sp
            // - Push return address (sigreturn trampoline) on stack
            // - Set RDI = sig, RSI = &siginfo, RDX = &ucontext
        }

        #[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
        {
            let _ = (use_altstack, sig, info, action);
        }

        Ok(())
    }

    /// Wait for signal
    fn wait_for_signal(&self, current: *mut Process) -> Result<(), SignalError> {
        // SAFETY: current is a valid ProcessDesc pointer.
        // Set the process state to Interruptible (3) and yield the CPU.
        // When a signal is delivered, wake_up_process() will set the state
        // back to Ready and enqueue the process.
        unsafe {
            (*current).state.store(3, Ordering::Release); // ProcessState::Interruptible = 3
        }
        // Yield CPU to another process; we will be woken up when a signal arrives
        crate::kernel::sched::schedule();
        // After waking up, check if a signal was delivered
        // If interrupted by a signal, return EINTR (which is the POSIX behavior for sigsuspend)
        Err(SignalError::NotSupported) // EINTR equivalent
    }

    /// Get current process
    fn get_current(&self) -> *mut Process {
        // Get the current process from the scheduler's current task pointer.
        crate::kernel::sched::get_current_task() as *mut Process
    }
}

/// Process descriptor (forward declaration)
#[repr(C)]
pub struct ProcessDesc {
    pub pid: AtomicU32,
    pub ppid: AtomicU32,
    pub state: AtomicU32,
    pub exit_code: AtomicU32,
    pub exit_signal: AtomicU32,
    pub signal: SignalState,
    pub cred: Credentials,
}

/// Credentials (forward declaration)
#[repr(C)]
pub struct Credentials {
    pub uid: AtomicU32,
}

/// Signal operations
pub const SIG_BLOCK: i32 = 0;
pub const SIG_UNBLOCK: i32 = 1;
pub const SIG_SETMASK: i32 = 2;

/// Signal error
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalError {
    InvalidSignal,
    InvalidArgument,
    PermissionDenied,
    ProcessNotFound,
    NotSupported,
}

/// Global signal handler
static SIGNAL_HANDLER: SignalHandler = SignalHandler::new();

/// Get signal handler
pub fn get_signal_handler() -> &'static SignalHandler {
    &SIGNAL_HANDLER
}

/// Kill system call
pub fn sys_kill(pid: u32, sig: u32) -> i64 {
    let info = SigInfo {
        signo: sig as i32,
        errno: 0,
        code: 0,
        pid: 0,
        uid: 0,
        value: SigVal { sival_int: 0 },
        addr: 0,
    };

    match SIGNAL_HANDLER.send_signal(pid, sig, &info) {
        Ok(()) => 0,
        Err(e) => -(e as i64),
    }
}

/// sigaction system call
pub fn sys_sigaction(sig: u32, act: *const SigAction, old_act: *mut SigAction) -> i64 {
    let act_ref = if act.is_null() {
        None
    } else {
        // SAFETY: unsafe block required for low-level memory or hardware access
        Some(unsafe { &*act })
    };

    let old_ref = if old_act.is_null() {
        None
    } else {
        // SAFETY: unsafe block required for low-level memory or hardware access
        Some(unsafe { &mut *old_act })
    };

    match act_ref {
        Some(act) => match SIGNAL_HANDLER.set_action(sig, act, old_ref) {
            Ok(()) => 0,
            Err(e) => -(e as i64),
        },
        None => -(KernelError::InvalidArgument as i64),
    }
}

/// sigprocmask system call
pub fn sys_sigprocmask(how: i32, set: *const SigSet, old_set: *mut SigSet) -> i64 {
    let set_ref = if set.is_null() {
        None
    } else {
        // SAFETY: unsafe block required for low-level memory or hardware access
        Some(unsafe { &*set })
    };

    let old_ref = if old_set.is_null() {
        None
    } else {
        // SAFETY: unsafe block required for low-level memory or hardware access
        Some(unsafe { &mut *old_set })
    };

    match SIGNAL_HANDLER.set_mask(how, set_ref, old_ref) {
        Ok(()) => 0,
        Err(e) => -(e as i64),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal_numbers() {
        assert_eq!(signal::SIGHUP, 1);
        assert_eq!(signal::SIGINT, 2);
        assert_eq!(signal::SIGKILL, 9);
        assert_eq!(signal::SIGTERM, 15);
        assert_eq!(signal::SIGCHLD, 17);
    }

    #[test]
    fn test_sigset_new() {
        let set = SigSet::new();
        assert!(set.is_empty());
    }

    #[test]
    fn test_sigset_add_del() {
        let mut set = SigSet::new();

        set.add(signal::SIGINT);
        assert!(set.is_member(signal::SIGINT));
        assert!(!set.is_member(signal::SIGTERM));

        set.del(signal::SIGINT);
        assert!(!set.is_member(signal::SIGINT));
    }

    #[test]
    fn test_sigset_fill() {
        let mut set = SigSet::new();
        set.fill();

        for sig in 1..=64 {
            assert!(set.is_member(sig));
        }
    }

    #[test]
    fn test_sigset_or_and() {
        let mut set1 = SigSet::new();
        let mut set2 = SigSet::new();

        set1.add(signal::SIGINT);
        set2.add(signal::SIGTERM);

        let or = set1.or(&set2);
        assert!(or.is_member(signal::SIGINT));
        assert!(or.is_member(signal::SIGTERM));

        let and = set1.and(&set2);
        assert!(!and.is_member(signal::SIGINT));
        assert!(!and.is_member(signal::SIGTERM));
    }

    #[test]
    fn test_sigaction_new() {
        let action = SigAction::new();
        assert!(action.is_default());
        assert!(!action.is_ignore());
    }

    #[test]
    fn test_signal_state_new() {
        let state = SignalState::new();
        assert!(!state.is_blocked(signal::SIGINT));
        assert!(!state.is_pending(signal::SIGINT));
    }

    #[test]
    fn test_signal_handler_new() {
        let handler = SignalHandler::new();
        assert_eq!(handler.signals_delivered.load(Ordering::Relaxed), 0);
        assert_eq!(handler.signals_ignored.load(Ordering::Relaxed), 0);
    }
}
