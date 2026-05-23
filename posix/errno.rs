/*
 * Nuva OS - POSIX errno compatibility
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */

// POSIX error numbers - type-safe enum

/// POSIX error number enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum Errno {
    Ok = 0,
    Eperm = 1,
    Enoent = 2,
    Esrch = 3,
    Eintr = 4,
    Eio = 5,
    Enxio = 6,
    E2big = 7,
    Enoexec = 8,
    Ebadf = 9,
    Echild = 10,
    Eagain = 11,
    Enomem = 12,
    Eacces = 13,
    Efault = 14,
    Enotblk = 15,
    Ebusy = 16,
    Eexist = 17,
    Exdev = 18,
    Enodev = 19,
    Enotdir = 20,
    Eisdir = 21,
    Einval = 22,
    Enfile = 23,
    Enotty = 24,
    Etxtbsy = 26,
    Efbig = 27,
    Enospc = 28,
    Espipe = 29,
    Erofs = 30,
    Emlink = 31,
    Epipe = 32,
    Edom = 33,
    Erange = 34,
    Edeadlk = 35,
    Enametoolong = 36,
    Enolck = 37,
    Enosys = 38,
    Enotempty = 39,
    Eloop = 40,
    Eopnotsupp = 95,
}

impl Errno {
    /// Convert errno to syscall return value (negative errno convention)
    pub fn to_syscall_return(self) -> i64 {
        -(self as i32 as i64)
    }

    /// Convert errno to i32 syscall return
    pub fn to_ret_i32(self) -> i32 {
        -(self as i32)
    }

    /// Attempt to convert from i32 to Errno
    pub fn from_i32(val: i32) -> Option<Self> {
        match val {
            0 => Some(Errno::Ok),
            1 => Some(Errno::Eperm),
            2 => Some(Errno::Enoent),
            3 => Some(Errno::Esrch),
            4 => Some(Errno::Eintr),
            5 => Some(Errno::Eio),
            6 => Some(Errno::Enxio),
            7 => Some(Errno::E2big),
            8 => Some(Errno::Enoexec),
            9 => Some(Errno::Ebadf),
            10 => Some(Errno::Echild),
            11 => Some(Errno::Eagain),
            12 => Some(Errno::Enomem),
            13 => Some(Errno::Eacces),
            14 => Some(Errno::Efault),
            15 => Some(Errno::Enotblk),
            16 => Some(Errno::Ebusy),
            17 => Some(Errno::Eexist),
            18 => Some(Errno::Exdev),
            19 => Some(Errno::Enodev),
            20 => Some(Errno::Enotdir),
            21 => Some(Errno::Eisdir),
            22 => Some(Errno::Einval),
            23 => Some(Errno::Enfile),
            24 => Some(Errno::Enotty),
            26 => Some(Errno::Etxtbsy),
            27 => Some(Errno::Efbig),
            28 => Some(Errno::Enospc),
            29 => Some(Errno::Espipe),
            30 => Some(Errno::Erofs),
            31 => Some(Errno::Emlink),
            32 => Some(Errno::Epipe),
            33 => Some(Errno::Edom),
            34 => Some(Errno::Erange),
            35 => Some(Errno::Edeadlk),
            36 => Some(Errno::Enametoolong),
            37 => Some(Errno::Enolck),
            38 => Some(Errno::Enosys),
            39 => Some(Errno::Enotempty),
            40 => Some(Errno::Eloop),
            95 => Some(Errno::Eopnotsupp),
            _ => None,
        }
    }
}

// Legacy constant aliases for backward compatibility
pub const EOK: i32 = Errno::Ok as i32;
pub const EPERM: i32 = Errno::Eperm as i32;
pub const ENOENT: i32 = Errno::Enoent as i32;
pub const ESRCH: i32 = Errno::Esrch as i32;
pub const EINTR: i32 = Errno::Eintr as i32;
pub const EIO: i32 = Errno::Eio as i32;
pub const EBADF: i32 = Errno::Ebadf as i32;
pub const ENOMEM: i32 = Errno::Enomem as i32;
pub const EACCES: i32 = Errno::Eacces as i32;
pub const EINVAL: i32 = Errno::Einval as i32;

/// Macro to convert Errno to negative i64 for syscall returns
#[macro_export]
macro_rules! errno_ret {
    (Eperm) => { -$crate::posix::errno::Errno::Eperm as i64 };
    (Enoent) => { -$crate::posix::errno::Errno::Enoent as i64 };
    (Esrch) => { -$crate::posix::errno::Errno::Esrch as i64 };
    (Eintr) => { -$crate::posix::errno::Errno::Eintr as i64 };
    (Eio) => { -$crate::posix::errno::Errno::Eio as i64 };
    (Enxio) => { -$crate::posix::errno::Errno::Enxio as i64 };
    (E2big) => { -$crate::posix::errno::Errno::E2big as i64 };
    (Enoexec) => { -$crate::posix::errno::Errno::Enoexec as i64 };
    (Ebadf) => { -$crate::posix::errno::Errno::Ebadf as i64 };
    (Echild) => { -$crate::posix::errno::Errno::Echild as i64 };
    (Eagain) => { -$crate::posix::errno::Errno::Eagain as i64 };
    (Enomem) => { -$crate::posix::errno::Errno::Enomem as i64 };
    (Eacces) => { -$crate::posix::errno::Errno::Eacces as i64 };
    (Efault) => { -$crate::posix::errno::Errno::Efault as i64 };
    (Enotblk) => { -$crate::posix::errno::Errno::Enotblk as i64 };
    (Ebusy) => { -$crate::posix::errno::Errno::Ebusy as i64 };
    (Eexist) => { -$crate::posix::errno::Errno::Eexist as i64 };
    (Exdev) => { -$crate::posix::errno::Errno::Exdev as i64 };
    (Enodev) => { -$crate::posix::errno::Errno::Enodev as i64 };
    (Enotdir) => { -$crate::posix::errno::Errno::Enotdir as i64 };
    (Eisdir) => { -$crate::posix::errno::Errno::Eisdir as i64 };
    (Einval) => { -$crate::posix::errno::Errno::Einval as i64 };
    (Enfile) => { -$crate::posix::errno::Errno::Enfile as i64 };
    (Enotty) => { -$crate::posix::errno::Errno::Enotty as i64 };
    (Etxtbsy) => { -$crate::posix::errno::Errno::Etxtbsy as i64 };
    (Efbig) => { -$crate::posix::errno::Errno::Efbig as i64 };
    (Enospc) => { -$crate::posix::errno::Errno::Enospc as i64 };
    (Espipe) => { -$crate::posix::errno::Errno::Espipe as i64 };
    (Erofs) => { -$crate::posix::errno::Errno::Erofs as i64 };
    (Emlink) => { -$crate::posix::errno::Errno::Emlink as i64 };
    (Epipe) => { -$crate::posix::errno::Errno::Epipe as i64 };
    (Edom) => { -$crate::posix::errno::Errno::Edom as i64 };
    (Erange) => { -$crate::posix::errno::Errno::Erange as i64 };
    (Edeadlk) => { -$crate::posix::errno::Errno::Edeadlk as i64 };
    (Enametoolong) => { -$crate::posix::errno::Errno::Enametoolong as i64 };
    (Enolck) => { -$crate::posix::errno::Errno::Enolck as i64 };
    (Enosys) => { -$crate::posix::errno::Errno::Enosys as i64 };
    (Enotempty) => { -$crate::posix::errno::Errno::Enotempty as i64 };
    (Eloop) => { -$crate::posix::errno::Errno::Eloop as i64 };
    (Eopnotsupp) => { -$crate::posix::errno::Errno::Eopnotsupp as i64 };
}
