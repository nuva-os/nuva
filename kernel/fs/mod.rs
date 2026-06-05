/*
 * Nuva OS - Kernel - File System
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

// File system submodules
// Re-export print macros from crate root
pub use crate::{pr_emerg, pr_alert, pr_crit, pr_err, pr_warn, pr_notice, pr_info, pr_debug};
pub mod vfs;
pub mod buffer;
pub mod filesystem;
pub mod page_cache;
pub mod dcache;
pub mod io_uring;
pub mod wal;
pub mod cow;
pub mod snapshot;

// Re-export key types
pub use page_cache::{PageCache, PageCacheEntry, init_page_cache};
pub use dcache::{DentryCache, Dentry, init_dcache};
pub use io_uring::{IoUring, IoSqe, IoCqe, init_io_uring};

// Re-export VFS types
pub use vfs::{InoT as Ino, OffT as Off, FileType, FileSystemType as FsType};


use core::sync::atomic::Ordering;

use crate::posix::errno::Errno;
static CURRENT_FD_TABLE: core::sync::OnceLock<vfs::FileDescriptorTable> = core::sync::OnceLock::new();

fn fd_table() -> &'static vfs::FileDescriptorTable {
    CURRENT_FD_TABLE.get_or_init(vfs::FileDescriptorTable::new)
}

fn copy_path_from_user(ptr: *const u8, max_len: usize) -> Option<alloc::string::String> {
    if ptr.is_null() {
        return None;
    }
    let mut len = 0;
    // SAFETY: reading user-provided pointer up to max_len or null terminator
    unsafe {
        while len < max_len && *ptr.add(len) != 0 {
            len += 1;
        }
        if len == 0 {
            return None;
        }
        let slice = core::slice::from_raw_parts(ptr, len);
        alloc::string::String::from_utf8(slice.to_vec()).ok()
    }
}

pub fn sys_open(path: *const u8, flags: u32, mode: u32) -> i64 {
    let path_str = match copy_path_from_user(path, 4096) {
        Some(s) => s,
        None => return Errno::Efault.to_syscall_return(),
    };

    let vfs = vfs::vfs_core();
    let lookup = match vfs.path_lookup(&path_str, vfs::lookup_flags::FOLLOW_SYMLINK) {
        Some(l) => l,
        None => {
            if (flags as i32 & vfs::open_flags::O_CREAT) != 0 {
                if vfs.create(&path_str, mode) != 0 {
                    return Errno::Enoent.to_syscall_return();
                }
                match vfs.path_lookup(&path_str, vfs::lookup_flags::FOLLOW_SYMLINK) {
                    Some(l) => l,
                    None => return Errno::Enoent.to_syscall_return(),
                }
            } else {
                return Errno::Enoent.to_syscall_return();
            }
        }
    };

    let fd_table = get_fd_table();
    match fd_table.alloc_fd(lookup.inode, flags) {
        Some(fd) => {
            vfs::VFS_STATS.open_files.fetch_add(1, Ordering::Relaxed);
            fd as i64
        }
        None => Errno::Enotty.to_syscall_return(),
    }
}

pub fn sys_close(fd: i32) -> i64 {
    if fd < 0 {
        return Errno::Ebadf.to_syscall_return();
    }
    let fd_table = get_fd_table();
    if fd_table.free_fd(fd as u32) {
        vfs::VFS_STATS.open_files.fetch_sub(1, Ordering::Relaxed);
        0
    } else {
        -9
    }
}

pub fn sys_read(fd: i32, buf: *mut u8, count: usize) -> i64 {
    if fd < 0 || buf.is_null() || count == 0 {
        return Errno::Einval.to_syscall_return();
    }
    let fd_table = get_fd_table();
    if fd_table.get_fd(fd as u32).is_none() {
        return Errno::Ebadf.to_syscall_return();
    }

    // SAFETY: writing to user buffer
    let user_buf = unsafe { core::slice::from_raw_parts_mut(buf, count) };
    vfs::VFS_STATS.reads.fetch_add(1, Ordering::Relaxed);
    vfs::file::read(fd as u32, user_buf)
}

pub fn sys_write(fd: i32, buf: *const u8, count: usize) -> i64 {
    if fd < 0 || buf.is_null() || count == 0 {
        return Errno::Einval.to_syscall_return();
    }
    let fd_table = get_fd_table();
    if fd_table.get_fd(fd as u32).is_none() {
        return Errno::Ebadf.to_syscall_return();
    }

    // SAFETY: reading from user buffer
    let user_buf = unsafe { core::slice::from_raw_parts(buf, count) };
    vfs::VFS_STATS.writes.fetch_add(1, Ordering::Relaxed);
    vfs::file::write(fd as u32, user_buf)
}

pub fn sys_lseek(fd: i32, offset: i64, whence: u32) -> i64 {
    if fd < 0 {
        return Errno::Ebadf.to_syscall_return();
    }
    if whence > 2 {
        return Errno::Einval.to_syscall_return();
    }
    vfs::file::lseek(fd as u32, offset, whence as i32)
}

pub fn sys_mkdir(path: *const u8, mode: u32) -> i64 {
    let path_str = match copy_path_from_user(path, 4096) {
        Some(s) => s,
        None => return Errno::Efault.to_syscall_return(),
    };
    let vfs = vfs::vfs_core();
    vfs.mkdir(&path_str, mode) as i64
}

pub fn sys_unlink(path: *const u8) -> i64 {
    let path_str = match copy_path_from_user(path, 4096) {
        Some(s) => s,
        None => return Errno::Efault.to_syscall_return(),
    };
    let vfs = vfs::vfs_core();
    vfs.unlink(&path_str) as i64
}
