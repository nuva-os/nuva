/*
 * Nuva OS - Syslib - POSIX stat.h compatibility
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */

use super::errno::Errno;

/// Time specification
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
}

/// File metadata structure (POSIX stat)
/// Reuses the FileMetadata design from kernel/posix.rs Phase 8.
#[derive(Debug, Clone, Copy)]
pub struct FileMetadata {
    pub device_id: u64,
    pub inode_number: u64,
    pub mode: u32,
    pub link_count: u32,
    pub user_id: u32,
    pub group_id: u32,
    pub raw_device_id: u64,
    pub size: i64,
    pub block_size: i32,
    pub block_count: i64,
    pub access_time: TimeSpec,
    pub modification_time: TimeSpec,
    pub change_time: TimeSpec,
}

impl FileMetadata {
    /// Create empty metadata
    pub const fn zero() -> Self {
        FileMetadata {
            device_id: 0,
            inode_number: 0,
            mode: 0,
            link_count: 0,
            user_id: 0,
            group_id: 0,
            raw_device_id: 0,
            size: 0,
            block_size: 0,
            block_count: 0,
            access_time: TimeSpec::ZERO,
            modification_time: TimeSpec::ZERO,
            change_time: TimeSpec::ZERO,
        }
    }

    /// Test file type from mode bits
    pub fn file_type(&self) -> FileType {
        let fmt = self.mode & 0o170000;
        match fmt {
            0o100000 => FileType::Regular,
            0o040000 => FileType::Directory,
            0o120000 => FileType::Symlink,
            0o020000 => FileType::CharDevice,
            0o060000 => FileType::BlockDevice,
            0o010000 => FileType::Fifo,
            0o140000 => FileType::Socket,
            _ => FileType::Unknown,
        }
    }

    /// Check if the file is a regular file
    pub fn is_regular(&self) -> bool {
        self.file_type() == FileType::Regular
    }

    /// Check if the file is a directory
    pub fn is_directory(&self) -> bool {
        self.file_type() == FileType::Directory
    }
}

/// File type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    Unknown,
    Regular,
    Directory,
    Symlink,
    CharDevice,
    BlockDevice,
    Fifo,
    Socket,
}

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

/// Get file status by path
/// POSIX.1-2017: stat() obtains information about the named file.
pub fn stat(_path: &str, _buf: &mut FileMetadata) -> Result<(), Errno> {
    Err(Errno::Enosys)
}

/// Get file status by file descriptor
/// POSIX.1-2017: fstat() obtains information about the file descriptor.
pub fn fstat(_fd: i32, _buf: &mut FileMetadata) -> Result<(), Errno> {
    Err(Errno::Enosys)
}

/// Get file status by path (no symlink follow)
/// POSIX.1-2017: lstat() is like stat() but does not follow symlinks.
pub fn lstat(_path: &str, _buf: &mut FileMetadata) -> Result<(), Errno> {
    Err(Errno::Enosys)
}
