/*
 * Nuva OS - Syslib - POSIX fcntl.h compatibility
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */

use super::errno::Errno;

bitflags::bitflags! {
    /// POSIX open() flags
    #[repr(transparent)]
    pub struct OpenFlags: u32 {
        const O_RDONLY = 0;
        const O_WRONLY = 1;
        const O_RDWR = 2;
        const O_CREAT = 0o100;
        const O_EXCL = 0o200;
        const O_NOCTTY = 0o400;
        const O_TRUNC = 0o1000;
        const O_APPEND = 0o2000;
        const O_NONBLOCK = 0o4000;
        const O_DIRECTORY = 0o200000;
        const O_NOFOLLOW = 0o400000;
        const O_CLOEXEC = 0o2000000;
    }
}

bitflags::bitflags! {
    /// POSIX file mode bits
    #[repr(transparent)]
    pub struct FileMode: u32 {
        const S_ISUID = 0o4000;
        const S_ISGID = 0o2000;
        const S_ISVTX = 0o1000;
        const S_IRWXU = 0o700;
        const S_IRUSR = 0o400;
        const S_IWUSR = 0o200;
        const S_IXUSR = 0o100;
        const S_IRWXG = 0o070;
        const S_IRGRP = 0o040;
        const S_IWGRP = 0o020;
        const S_IXGRP = 0o010;
        const S_IRWXO = 0o007;
        const S_IROTH = 0o004;
        const S_IWOTH = 0o002;
        const S_IXOTH = 0o001;
    }
}

impl FileMode {
    /// Test if mode represents a regular file
    pub fn is_regular(self) -> bool {
        (self.bits() & 0o170000) == 0o100000
    }

    /// Test if mode represents a directory
    pub fn is_directory(self) -> bool {
        (self.bits() & 0o170000) == 0o040000
    }

    /// Test if mode represents a symbolic link
    pub fn is_symlink(self) -> bool {
        (self.bits() & 0o170000) == 0o120000
    }

    /// Test if mode represents a FIFO
    pub fn is_fifo(self) -> bool {
        (self.bits() & 0o170000) == 0o010000
    }

    /// Test if mode represents a socket
    pub fn is_socket(self) -> bool {
        (self.bits() & 0o170000) == 0o140000
    }

    /// Test if mode represents a character device
    pub fn is_char_device(self) -> bool {
        (self.bits() & 0o170000) == 0o020000
    }

    /// Test if mode represents a block device
    pub fn is_block_device(self) -> bool {
        (self.bits() & 0o170000) == 0o060000
    }
}

/// Open a file
/// POSIX.1-2017: open() opens a file and returns a file descriptor.
/// Error conditions:
///   - ENOENT: O_CREAT not set and path does not exist
///   - EEXIST: O_CREAT|O_EXCL and file already exists
///   - EACCES: permission denied for path or mode
///   - ENAMETOOLONG: path exceeds PATH_MAX
///   - ENOTDIR: component of path prefix is not a directory
///   - EISDIR: path is a directory and write access requested
///   - ENOSPC: no space left on device (O_CREAT)
///   - EROFS: read-only filesystem and write access requested
pub fn open(_path: &str, _flags: OpenFlags, _mode: FileMode) -> Result<i32, Errno> {
    Err(Errno::Enosys)
}

/// Close a file descriptor
/// POSIX.1-2017: close() closes a file descriptor.
/// Error conditions:
///   - EBADF: fd is not a valid open file descriptor
///   - EINTR: close() was interrupted by a signal
///   - EIO: an I/O error occurred
pub fn close(_fd: i32) -> Result<(), Errno> {
    Err(Errno::Enosys)
}

/// Duplicate a file descriptor
/// POSIX.1-2017: dup() duplicates an existing file descriptor.
/// Error conditions:
///   - EBADF: fd is not a valid open file descriptor
///   - EMFILE: per-process file descriptor limit would be exceeded
pub fn dup(_fd: i32) -> Result<i32, Errno> {
    Err(Errno::Enosys)
}

/// Duplicate a file descriptor to a specific number
/// POSIX.1-2017: dup2() duplicates fd to fd2.
/// Error conditions:
///   - EBADF: fd is not valid, or fd2 is negative or >= OPEN_MAX
pub fn dup2(_fd: i32, _fd2: i32) -> Result<i32, Errno> {
    Err(Errno::Enosys)
}
