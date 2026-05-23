/*
 * Nuva OS - Syslib - POSIX signal.h compatibility
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */

use super::errno::Errno;

/// POSIX standard signals
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum Signal {
    Sighup = 1,
    Sigint = 2,
    Sigquit = 3,
    Sigill = 4,
    Sigtrap = 5,
    Sigabrt = 6,
    Sigkill = 9,
    Sigusr1 = 10,
    Sigsegv = 11,
    Sigusr2 = 12,
    Sigpipe = 13,
    Sigalrm = 14,
    Sigterm = 15,
    Sigchld = 17,
    Sigcont = 18,
    Sigstop = 19,
    Sigtstp = 20,
    Sigttin = 21,
    Sigttou = 22,
    Sigurg = 23,
    Sigxcpu = 24,
    Sigxfsz = 25,
    Sigvtalrm = 26,
    Sigprof = 27,
    Sigwinch = 28,
    Sigio = 29,
    Sigsys = 31,
}

impl Signal {
    /// Check if this signal can be caught or ignored
    /// SIGKILL and SIGSTOP cannot be caught, blocked, or ignored per POSIX.
    pub fn is_catchable(self) -> bool {
        self != Signal::Sigkill && self != Signal::Sigstop
    }

    /// Convert from u32 signal number
    pub fn from_u32(val: u32) -> Option<Self> {
        match val {
            1 => Some(Signal::Sighup),
            2 => Some(Signal::Sigint),
            3 => Some(Signal::Sigquit),
            4 => Some(Signal::Sigill),
            5 => Some(Signal::Sigtrap),
            6 => Some(Signal::Sigabrt),
            9 => Some(Signal::Sigkill),
            10 => Some(Signal::Sigusr1),
            11 => Some(Signal::Sigsegv),
            12 => Some(Signal::Sigusr2),
            13 => Some(Signal::Sigpipe),
            14 => Some(Signal::Sigalrm),
            15 => Some(Signal::Sigterm),
            17 => Some(Signal::Sigchld),
            18 => Some(Signal::Sigcont),
            19 => Some(Signal::Sigstop),
            20 => Some(Signal::Sigtstp),
            21 => Some(Signal::Sigttin),
            22 => Some(Signal::Sigttou),
            23 => Some(Signal::Sigurg),
            24 => Some(Signal::Sigxcpu),
            25 => Some(Signal::Sigxfsz),
            26 => Some(Signal::Sigvtalrm),
            27 => Some(Signal::Sigprof),
            28 => Some(Signal::Sigwinch),
            29 => Some(Signal::Sigio),
            31 => Some(Signal::Sigsys),
            _ => None,
        }
    }
}

bitflags::bitflags! {
    /// POSIX sigaction flags
    #[repr(transparent)]
    pub struct SignalFlags: u32 {
        const SA_NOCLDSTOP = 0x00000001;
        const SA_NOCLDWAIT = 0x00000002;
        const SA_SIGINFO = 0x00000004;
        const SA_RESTART = 0x10000000;
    }
}

/// Signal action handler specification
#[derive(Debug, Clone, Copy)]
pub struct SignalAction {
    pub handler: SignalHandler,
    pub flags: SignalFlags,
    pub mask: u64,
}

/// Signal handler type
pub type SignalHandler = extern "C" fn(i32);

/// Send a signal to a process
/// POSIX.1-2017: kill() sends a signal to a process or process group.
/// Error conditions:
///   - EINVAL: sig is an invalid or unsupported signal number
///   - EPERM: the process does not have permission to send the signal
///   - ESRCH: no process or process group corresponds to pid
pub fn kill(_pid: i32, _sig: Signal) -> Result<(), Errno> {
    Err(Errno::Enosys)
}

/// Examine and change a signal action
/// POSIX.1-2017: sigaction() examines or changes the action associated with a signal.
/// Error conditions:
///   - EINVAL: sig is an invalid signal number or not catchable (SIGKILL/SIGSTOP)
pub fn sigaction(_sig: Signal, _act: Option<&SignalAction>, _oact: Option<&mut SignalAction>) -> Result<(), Errno> {
    Err(Errno::Enosys)
}
