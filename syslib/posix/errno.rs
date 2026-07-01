/*
 * Nuva OS - Syslib - POSIX errno compatibility
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */

/// error number enumeration (50+ variants)
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
    Ewouldblock = 41,
    Einprogress = 115,
    Ealready = 114,
    Enotsock = 88,
    Edestaddrreq = 89,
    Emsgsize = 90,
    Eprototype = 91,
    Enoprotoopt = 92,
    Eprotonosupport = 93,
    Esocktnosupport = 94,
    Eopnotsupp = 95,
    Epfnosupport = 96,
    Eafnosupport = 97,
    Eaddrinuse = 98,
    Eaddrnotavail = 99,
    Enetdown = 100,
    Enetunreach = 101,
    Enetreset = 102,
    Econnaborted = 103,
    Econnreset = 104,
    Enobufs = 105,
    Eisconn = 106,
    Enotconn = 107,
    Eshutdown = 108,
    Etoomanyrefs = 109,
    Etimedout = 110,
    Econnrefused = 111,
    Ehostdown = 112,
    Ehostunreach = 113,
    Eproclim = 127,
    Eusers = 87,
    Edquot = 122,
    Estale = 116,
    Eremote = 66,
    Enotsup = 126,
    Emultihop = 72,
    Eidlchain = 73,
    Eoverflow = 75,
    Eilseq = 84,
    Ebadmsg = 77,
    Eidrm = 43,
    Enomsg = 42,
    Enodata = 61,
    Enosr = 63,
    Etime = 62,
    Enolink = 67,
    Emedia = 78,
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
            41 => Some(Errno::Ewouldblock),
            42 => Some(Errno::Enomsg),
            43 => Some(Errno::Eidrm),
            61 => Some(Errno::Enodata),
            62 => Some(Errno::Etime),
            63 => Some(Errno::Enosr),
            66 => Some(Errno::Eremote),
            67 => Some(Errno::Eproclim),
            72 => Some(Errno::Emultihop),
            73 => Some(Errno::Eidlchain),
            75 => Some(Errno::Eoverflow),
            77 => Some(Errno::Ebadmsg),
            78 => Some(Errno::Emedia),
            84 => Some(Errno::Eilseq),
            87 => Some(Errno::Eusers),
            88 => Some(Errno::Enotsock),
            89 => Some(Errno::Edestaddrreq),
            90 => Some(Errno::Emsgsize),
            91 => Some(Errno::Eprototype),
            92 => Some(Errno::Enoprotoopt),
            93 => Some(Errno::Eprotonosupport),
            94 => Some(Errno::Esocktnosupport),
            95 => Some(Errno::Eopnotsupp),
            96 => Some(Errno::Epfnosupport),
            97 => Some(Errno::Eafnosupport),
            98 => Some(Errno::Eaddrinuse),
            99 => Some(Errno::Eaddrnotavail),
            100 => Some(Errno::Enetdown),
            101 => Some(Errno::Enetunreach),
            102 => Some(Errno::Enetreset),
            103 => Some(Errno::Econnaborted),
            104 => Some(Errno::Econnreset),
            105 => Some(Errno::Enobufs),
            106 => Some(Errno::Eisconn),
            107 => Some(Errno::Enotconn),
            108 => Some(Errno::Eshutdown),
            109 => Some(Errno::Etoomanyrefs),
            110 => Some(Errno::Etimedout),
            111 => Some(Errno::Econnrefused),
            112 => Some(Errno::Ehostdown),
            113 => Some(Errno::Ehostunreach),
            114 => Some(Errno::Ealready),
            115 => Some(Errno::Einprogress),
            116 => Some(Errno::Estale),
            122 => Some(Errno::Edquot),
            _ => None,
        }
    }

    /// Map from kernel internal error code to POSIX errno
    pub fn from_kernel_error(code: i32) -> Self {
        Self::from_i32(code).unwrap_or(Errno::Einval)
    }
}

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
    (Eperm) => { -$crate::syslib::posix::errno::Errno::Eperm as i64 };
    (Enoent) => { -$crate::syslib::posix::errno::Errno::Enoent as i64 };
    (Esrch) => { -$crate::syslib::posix::errno::Errno::Esrch as i64 };
    (Eintr) => { -$crate::syslib::posix::errno::Errno::Eintr as i64 };
    (Eio) => { -$crate::syslib::posix::errno::Errno::Eio as i64 };
    (Enxio) => { -$crate::syslib::posix::errno::Errno::Enxio as i64 };
    (E2big) => { -$crate::syslib::posix::errno::Errno::E2big as i64 };
    (Enoexec) => { -$crate::syslib::posix::errno::Errno::Enoexec as i64 };
    (Ebadf) => { -$crate::syslib::posix::errno::Errno::Ebadf as i64 };
    (Echild) => { -$crate::syslib::posix::errno::Errno::Echild as i64 };
    (Eagain) => { -$crate::syslib::posix::errno::Errno::Eagain as i64 };
    (Enomem) => { -$crate::syslib::posix::errno::Errno::Enomem as i64 };
    (Eacces) => { -$crate::syslib::posix::errno::Errno::Eacces as i64 };
    (Efault) => { -$crate::syslib::posix::errno::Errno::Efault as i64 };
    (Enotblk) => { -$crate::syslib::posix::errno::Errno::Enotblk as i64 };
    (Ebusy) => { -$crate::syslib::posix::errno::Errno::Ebusy as i64 };
    (Eexist) => { -$crate::syslib::posix::errno::Errno::Eexist as i64 };
    (Exdev) => { -$crate::syslib::posix::errno::Errno::Exdev as i64 };
    (Enodev) => { -$crate::syslib::posix::errno::Errno::Enodev as i64 };
    (Enotdir) => { -$crate::syslib::posix::errno::Errno::Enotdir as i64 };
    (Eisdir) => { -$crate::syslib::posix::errno::Errno::Eisdir as i64 };
    (Einval) => { -$crate::syslib::posix::errno::Errno::Einval as i64 };
    (Enfile) => { -$crate::syslib::posix::errno::Errno::Enfile as i64 };
    (Enotty) => { -$crate::syslib::posix::errno::Errno::Enotty as i64 };
    (Etxtbsy) => { -$crate::syslib::posix::errno::Errno::Etxtbsy as i64 };
    (Efbig) => { -$crate::syslib::posix::errno::Errno::Efbig as i64 };
    (Enospc) => { -$crate::syslib::posix::errno::Errno::Enospc as i64 };
    (Espipe) => { -$crate::syslib::posix::errno::Errno::Espipe as i64 };
    (Erofs) => { -$crate::syslib::posix::errno::Errno::Erofs as i64 };
    (Emlink) => { -$crate::syslib::posix::errno::Errno::Emlink as i64 };
    (Epipe) => { -$crate::syslib::posix::errno::Errno::Epipe as i64 };
    (Edom) => { -$crate::syslib::posix::errno::Errno::Edom as i64 };
    (Erange) => { -$crate::syslib::posix::errno::Errno::Erange as i64 };
    (Edeadlk) => { -$crate::syslib::posix::errno::Errno::Edeadlk as i64 };
    (Enametoolong) => { -$crate::syslib::posix::errno::Errno::Enametoolong as i64 };
    (Enolck) => { -$crate::syslib::posix::errno::Errno::Enolck as i64 };
    (Enosys) => { -$crate::syslib::posix::errno::Errno::Enosys as i64 };
    (Enotempty) => { -$crate::syslib::posix::errno::Errno::Enotempty as i64 };
    (Eloop) => { -$crate::syslib::posix::errno::Errno::Eloop as i64 };
    (Eopnotsupp) => { -$crate::syslib::posix::errno::Errno::Eopnotsupp as i64 };
}
