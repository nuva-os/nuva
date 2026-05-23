/*
 * Nuva OS - Syslib - POSIX Errno Adapter
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */

use crate::syslib::posix::errno::Errno;

/// Adapter for mapping kernel error codes to POSIX errno values
pub struct ErrnoAdapter;

impl ErrnoAdapter {
    /// Map a kernel internal error code to the corresponding POSIX errno
    /// This handles the translation between Nuva kernel error codes
    /// and standard errno values.
    pub fn from_kernel(code: i32) -> Errno {
        Errno::from_kernel_error(code)
    }

    /// Map POSIX errno back to kernel error code
    /// Used when translating POSIX-layer errors to kernel IPC messages.
    pub fn to_kernel(errno: Errno) -> i32 {
        errno as i32
    }

    /// Check if an errno represents a transient/retryable error
    /// POSIX EAGAIN, EWOULDBLOCK, EINTR are retryable.
    pub fn is_retryable(errno: Errno) -> bool {
        matches!(errno, Errno::Eagain | Errno::Ewouldblock | Errno::Eintr)
    }

    /// Check if an errno represents a resource exhaustion condition
    pub fn is_resource_exhausted(errno: Errno) -> bool {
        matches!(errno,
            Errno::Enomem | Errno::Enospc | Errno::Eagain |
            Errno::Enfile | Errno::Enobufs | Errno::Edquot
        )
    }

    /// Check if an errno represents a permission/authorization failure
    pub fn is_permission_denied(errno: Errno) -> bool {
        matches!(errno,
            Errno::Eperm | Errno::Eacces | Errno::Erofs
        )
    }

    /// Check if an errno indicates the operation is not supported
    pub fn is_not_supported(errno: Errno) -> bool {
        matches!(errno,
            Errno::Enosys | Errno::Eopnotsupp | Errno::Enotsup |
            Errno::Eprotonosupport | Errno::Esocktnosupport |
            Errno::Epfnosupport | Errno::Eafnosupport
        )
    }
}
