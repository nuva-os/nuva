/*
 * Nuva OS - Syslib - POSIX Response Adapter
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */

use crate::syslib::posix::errno::Errno;

/// Adapter for translating kernel IPC responses to POSIX return values
pub struct ResponseAdapter;

impl ResponseAdapter {
    /// Convert a raw kernel return value to a POSIX result
    /// Kernel uses negative errno convention: negative value = -errno
    pub fn from_kernel_return(raw: i64) -> Result<i64, Errno> {
        if raw < 0 {
            match Errno::from_i32(-raw as i32) {
                Some(errno) => Err(errno),
                None => Err(Errno::Einval),
            }
        } else {
            Ok(raw)
        }
    }

    /// Convert a kernel error code to POSIX errno
    pub fn error_from_kernel(code: i32) -> Errno {
        Errno::from_kernel_error(code)
    }

    /// Build a POSIX-conformant return for a file descriptor operation
    pub fn fd_result(fd: i32) -> Result<i32, Errno> {
        if fd < 0 {
            Err(Errno::Ebadf)
        } else {
            Ok(fd)
        }
    }

    /// Build a POSIX-conformant return for a byte count operation
    /// Returns 0 for EOF, error for failure, count for success.
    pub fn io_result(count: isize) -> Result<usize, Errno> {
        if count < 0 {
            Err(Errno::from_kernel_error(-(count as i32)))
        } else {
            Ok(count as usize)
        }
    }
}
