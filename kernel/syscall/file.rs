/*
 * Nuva OS - Kernel - Kernel
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


use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

use crate::posix::errno::Errno;
/// FileDescriptor
pub type Fd = i32;

/// POSIX file open flags — values preserved for ABI compatibility.
pub mod file_flags {
    /// Open read-only
    pub const O_RDONLY: i32 = 0o0;
    /// Open write-only
    pub const O_WRONLY: i32 = 0o1;
    /// Open read-write
    pub const O_RDWR: i32 = 0o2;
    /// Create file if it does not exist
    pub const O_CREAT: i32 = 0o100;
    /// Fail if file already exists (with O_CREAT)
    pub const O_EXCL: i32 = 0o200;
    /// Do not assign controlling terminal
    pub const O_NOCTTY: i32 = 0o400;
    /// Truncate file to zero length
    pub const O_TRUNC: i32 = 0o1000;
    /// Append to file
    pub const O_APPEND: i32 = 0o2000;
    /// Non-blocking mode
    pub const O_NONBLOCK: i32 = 0o4000;
    /// Synchronous writes
    pub const O_SYNC: i32 = 0o10000;
    /// Asynchronous I/O notifications
    pub const O_ASYNC: i32 = 0o20000;
    /// Direct I/O (bypass page cache)
    pub const O_DIRECT: i32 = 0o40000;
    /// Allow large file access
    pub const O_LARGEFILE: i32 = 0o100000;
    /// Must be a directory
    pub const O_DIRECTORY: i32 = 0o200000;
    /// Do not follow symbolic links
    pub const O_NOFOLLOW: i32 = 0o400000;
    /// Close on exec
    pub const O_CLOEXEC: i32 = 0o2000000;
}

/// POSIX file mode bits (permissions and type) — values preserved for ABI compatibility.
pub mod file_mode {
    /// File type mask
    pub const S_IFMT: u32 = 0o170000;
    /// Socket
    pub const S_IFSOCK: u32 = 0o140000;
    /// Symbolic link
    pub const S_IFLNK: u32 = 0o120000;
    /// Regular file
    pub const S_IFREG: u32 = 0o100000;
    /// Block device
    pub const S_IFBLK: u32 = 0o060000;
    /// Directory
    pub const S_IFDIR: u32 = 0o040000;
    /// Character device
    pub const S_IFCHR: u32 = 0o020000;
    /// Named pipe (FIFO)
    pub const S_IFIFO: u32 = 0o010000;
    /// Set-user-ID
    pub const S_ISUID: u32 = 0o4000;
    /// Set-group-ID
    pub const S_ISGID: u32 = 0o2000;
    /// Sticky bit
    pub const S_ISVTX: u32 = 0o1000;
    /// User read/write/execute
    pub const S_IRWXU: u32 = 0o700;
    /// User read
    pub const S_IRUSR: u32 = 0o400;
    /// User write
    pub const S_IWUSR: u32 = 0o200;
    /// User execute
    pub const S_IXUSR: u32 = 0o100;
    /// Group read/write/execute
    pub const S_IRWXG: u32 = 0o070;
    /// Group read
    pub const S_IRGRP: u32 = 0o040;
    /// Group write
    pub const S_IWGRP: u32 = 0o020;
    /// Group execute
    pub const S_IXGRP: u32 = 0o010;
    /// Others read/write/execute
    pub const S_IRWXO: u32 = 0o007;
    /// Others read
    pub const S_IROTH: u32 = 0o004;
    /// Others write
    pub const S_IWOTH: u32 = 0o002;
    /// Others execute
    pub const S_IXOTH: u32 = 0o001;
}

/// File status information — POSIX stat structure.
/// Field layout preserved for ABI compatibility.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Stat {
    /// Device ID of file system
    pub device_id: u64,
    /// Inode number
    pub inode_number: u64,
    /// File type and mode (permissions)
    pub mode: u32,
    /// Number of hard links
    pub link_count: u32,
    /// User ID of owner
    pub user_id: u32,
    /// Group ID of owner
    pub group_id: u32,
    /// Device ID (if special file)
    pub raw_device_id: u64,
    /// Total size in bytes
    pub size: i64,
    /// Block size for file system I/O
    pub block_size: i64,
    /// Number of 512B blocks allocated
    pub block_count: i64,
    /// Time of last access
    pub access_time: i64,
    /// Time of last modification
    pub modification_time: i64,
    /// Time of last status change
    pub change_time: i64,
}

impl Stat {
 pub fn new() -> Self {
 Stat {
 device_id: 0,
 inode_number: 0,
 mode: 0,
 link_count: 0,
 user_id: 0,
 group_id: 0,
 raw_device_id: 0,
 size: 0,
 block_size: 4096,
 block_count: 0,
 access_time: 0,
 modification_time: 0,
 change_time: 0,
 }
 }
}

/// Directoryproject
#[repr(C)]
pub struct Dirent {
 /// IndexNode
 pub d_ino: u64,
 /// Offset
 pub d_off: i64,
 /// Length
 pub d_reclen: u16,
 /// Type
 pub d_type: u8,
 /// Name
 pub d_name: [u8; 256],
}

/// FileDescriptorform
pub struct FdTable {
 /// FileDescriptorArray
 pub fds: [Option<u64>; 1024],
 /// Nextcanuse fd
 pub next_fd: AtomicU32,
 /// OpenFilenumber
 pub open_count: AtomicU32,
}

impl FdTable {
 pub const fn new() -> Self {
 FdTable {
 fds: [None; 1024],
 next_fd: AtomicU32::new(0),
 open_count: AtomicU32::new(0),
 }
 }
 
 /// AllocateFileDescriptor
 pub fn alloc_fd(&mut self) -> Option<Fd> {
 let start = self.next_fd.load(Ordering::Acquire);
 
 for i in 0..1024 {
 let fd = (start + i as u32) % 1024;
 
 if self.fds[fd as usize].is_none() {
 self.next_fd.store((fd + 1) % 1024, Ordering::Release);
 self.open_count.fetch_add(1, Ordering::AcqRel);
 return Some(fd as Fd);
 }
 }
 
 None
 }
 
 /// FreeFileDescriptor
 pub fn free_fd(&mut self, fd: Fd) {
 if fd >= 0 && (fd as usize) < 1024 {
 self.fds[fd as usize] = None;
 self.open_count.fetch_sub(1, Ordering::AcqRel);
 }
 }
 
 /// GetFile
 pub fn get_file(&self, fd: Fd) -> Option<u64> {
 if fd >= 0 && (fd as usize) < 1024 {
 self.fds[fd as usize]
 } else {
 None
 }
 }
 
 /// SetFile
 pub fn set_file(&mut self, fd: Fd, file: u64) {
 if fd >= 0 && (fd as usize) < 1024 {
 self.fds[fd as usize] = Some(file);
 }
 }
}

/// File SystemcallImplementation

/// OpenFile
pub fn sys_openat(_dirfd: Fd, _path: *const u8, _flags: i32, _mode: u32) -> Fd {
 // TODO: ImplementationOpen
 -1
}

/// CreateFile
pub fn sys_creat(_path: *const u8, _mode: u32) -> Fd {
 // TODO: ImplementationCreate
 -1
}

/// ReadFile
pub fn sys_read(_fd: Fd, _buf: *mut u8, _count: usize) -> i64 {
 // TODO: ImplementationRead
 -1
}

/// WriteFile
pub fn sys_write(_fd: Fd, _buf: *const u8, _count: usize) -> i64 {
 // TODO: ImplementationWrite
 -1
}

/// fixedBitFile
pub fn sys_lseek(_fd: Fd, _offset: i64, _whence: i32) -> i64 {
 // TODO: ImplementationfixedBit
 -1
}

/// CloseFile
pub fn sys_close(_fd: Fd) -> i32 {
 // TODO: ImplementationClose
 -1
}

/// KStat - Linux-compatible stat structure (kernel internal)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct KStat {
    /// Device
    pub device_id: u64,
    /// Inode number
    pub inode_number: u64,
    /// File mode
    pub mode: u32,
    /// Number of hard links
    pub link_count: u32,
    /// User ID
    pub user_id: u32,
    /// Group ID
    pub group_id: u32,
    /// Device type (for special files)
    pub raw_device_id: u64,
    /// File size
    pub size: i64,
    /// Preferred I/O block size
    pub block_size: i64,
    /// Number of 512B blocks allocated
    pub block_count: i64,
    /// Last access time (seconds since epoch)
    pub access_time: i64,
    /// Last modification time (seconds since epoch)
    pub modification_time: i64,
    /// Last status change time (seconds since epoch)
    pub change_time: i64,
}

impl KStat {
    pub const fn new() -> Self {
        KStat {
            device_id: 0,
            inode_number: 0,
            mode: 0,
            link_count: 0,
            user_id: 0,
            group_id: 0,
            raw_device_id: 0,
            size: 0,
            block_size: 4096,
            block_count: 0,
            access_time: 0,
            modification_time: 0,
            change_time: 0,
        }
    }
}

/// GetFileState
pub fn sys_fstat(fd: Fd, stat: *mut Stat) -> i32 {
    if fd < 0 {
        return Errno::Ebadf.to_ret_i32(); // EBADF
    }

    if stat.is_null() {
        return Errno::Efault.to_ret_i32(); // EFAULT
    }

    // Lookup file by fd in VFS
    let files = crate::fs::vfs::file::get_global_files();
    let file_ref = match files.get_file(fd as u32) {
        Some(f) => f,
        None => return Errno::Ebadf.to_ret_i32(), // EBADF
    };

    let inode_ptr = file_ref.f_inode;
    if inode_ptr.is_null() {
        return Errno::Ebadf.to_ret_i32(); // EBADF
    }

    // SAFETY: inode pointer was set during open and remains valid
    let inode = unsafe { &*inode_ptr };

    // Fill stat from inode metadata
    // SAFETY: stat is non-null and user-validated
    unsafe {
        (*stat).device_id = inode.i_sb;
        (*stat).inode_number = inode.i_ino;
        (*stat).mode = inode.i_mode;
        (*stat).link_count = inode.i_nlink.load(Ordering::Acquire);
        (*stat).user_id = inode.i_uid;
        (*stat).group_id = inode.i_gid;
        (*stat).raw_device_id = inode.i_rdev;
        (*stat).size = inode.i_size.load(Ordering::Acquire) as i64;
        (*stat).block_size = 1i64 << inode.i_blkbits;
        (*stat).block_count = inode.i_blocks as i64;
        (*stat).access_time = inode.i_atime as i64;
        (*stat).modification_time = inode.i_mtime as i64;
        (*stat).change_time = inode.i_ctime as i64;
    }

    0
}

/// GetFileState (throughPath)
pub fn sys_stat(path: *const u8, stat: *mut Stat) -> i32 {
    if path.is_null() {
        return Errno::Efault.to_ret_i32(); // EFAULT
    }

    if stat.is_null() {
        return Errno::Efault.to_ret_i32(); // EFAULT
    }

    // SAFETY: unsafe block required for low-level memory or hardware access
    let path_str = unsafe {
        let mut len = 0;
        let mut ptr = path;
        while *ptr != 0 && len < 4096 {
            len += 1;
            ptr = ptr.add(1);
        }
        if len == 0 || len >= 4096 {
            return Errno::Enametoolong.to_ret_i32(); // ENAMETOOLONG
        }
        core::str::from_utf8_unchecked(core::slice::from_raw_parts(path, len))
    };

    // Lookup inode by path, following symlinks
    let vfs = crate::fs::vfs::vfs_core();
    let lookup = match vfs.path_lookup(path_str, crate::fs::vfs::lookup_flags::FOLLOW_SYMLINK) {
        Some(l) => l,
        None => return Errno::Enoent.to_ret_i32(), // ENOENT
    };

    // Fill stat from lookup result
    // SAFETY: stat is non-null and user-validated
    unsafe {
        (*stat).device_id = 0;
        (*stat).inode_number = lookup.inode;
        (*stat).mode = 0o100644;
        (*stat).link_count = 1;
        (*stat).user_id = 0;
        (*stat).group_id = 0;
        (*stat).raw_device_id = 0;
        (*stat).size = 0;
        (*stat).block_size = 4096;
        (*stat).block_count = 0;
        (*stat).access_time = 0;
        (*stat).modification_time = 0;
        (*stat).change_time = 0;
    }

    0
}

/// GetFileState (notfollowSignlinkaccept)
pub fn sys_lstat(path: *const u8, stat: *mut Stat) -> i32 {
    if path.is_null() {
        return Errno::Efault.to_ret_i32(); // EFAULT
    }

    if stat.is_null() {
        return Errno::Efault.to_ret_i32(); // EFAULT
    }

    // SAFETY: unsafe block required for low-level memory or hardware access
    let path_str = unsafe {
        let mut len = 0;
        let mut ptr = path;
        while *ptr != 0 && len < 4096 {
            len += 1;
            ptr = ptr.add(1);
        }
        if len == 0 || len >= 4096 {
            return Errno::Enametoolong.to_ret_i32(); // ENAMETOOLONG
        }
        core::str::from_utf8_unchecked(core::slice::from_raw_parts(path, len))
    };

    // Lookup inode by path, NOT following symlinks
    let vfs = crate::fs::vfs::vfs_core();
    let lookup = match vfs.path_lookup(path_str, crate::fs::vfs::lookup_flags::NO_FOLLOW) {
        Some(l) => l,
        None => return Errno::Enoent.to_ret_i32(), // ENOENT
    };

    // Fill stat from lookup result (symlink info preserved, not dereferenced)
    // SAFETY: stat is non-null and user-validated
    unsafe {
        (*stat).device_id = 0;
        (*stat).inode_number = lookup.inode;
        (*stat).mode = if lookup.followed_symlink { 0o120644 } else { 0o100644 };
        (*stat).link_count = 1;
        (*stat).user_id = 0;
        (*stat).group_id = 0;
        (*stat).raw_device_id = 0;
        (*stat).size = 0;
        (*stat).block_size = 4096;
        (*stat).block_count = 0;
        (*stat).access_time = 0;
        (*stat).modification_time = 0;
        (*stat).change_time = 0;
    }

    0
}

/// CreateDirectory
pub fn sys_mkdir(_path: *const u8, _mode: u32) -> i32 {
 // TODO: ImplementationCreateDirectory
 -1
}

/// DeleteDirectory
pub fn sys_rmdir(_path: *const u8) -> i32 {
 // TODO: ImplementationDeleteDirectory
 -1
}

/// DeleteFile
pub fn sys_unlink(_path: *const u8) -> i32 {
 // TODO: ImplementationDeleteFile
 -1
}

/// rename
pub fn sys_rename(_oldpath: *const u8, _newpath: *const u8) -> i32 {
 // TODO: Implementationrename
 -1
}

/// CreateHard link
pub fn sys_link(_oldpath: *const u8, _newpath: *const u8) -> i32 {
 // TODO: ImplementationCreatelinkaccept
 -1
}

/// CreateSignlinkaccept
pub fn sys_symlink(_target: *const u8, _linkpath: *const u8) -> i32 {
 // TODO: ImplementationCreateSignlinkaccept
 -1
}

/// ReadSignlinkaccept
pub fn sys_readlink(_path: *const u8, _buf: *mut u8, _size: usize) -> i64 {
 // TODO: ImplementationReadSignlinkaccept
 -1
}

/// ModifyPermission
pub fn sys_chmod(_path: *const u8, _mode: u32) -> i32 {
 // TODO: ImplementationModifyPermission
 -1
}

/// ModifyOwner
pub fn sys_chown(_path: *const u8, _owner: u32, _group: u32) -> i32 {
 // TODO: ImplementationModifyOwner
 -1
}

/// SynchronousFile
pub fn sys_fsync(_fd: Fd) -> i32 {
 // TODO: ImplementationSynchronous
 -1
}

/// SynchronousFileData
pub fn sys_fdatasync(_fd: Fd) -> i32 {
 // TODO: ImplementationSynchronousData
 -1
}

/// TruncationFile
pub sys_truncate(_path: *const u8, _length: i64) -> i32 {
 // TODO: ImplementationTruncation
 -1
}

/// Apply BSD-style flock on a file descriptor.
/// operation: LOCK_SH(1), LOCK_EX(2), LOCK_UN(8), optionally OR'd with LOCK_NB(4).
/// Returns 0 on success, negative errno on failure.
pub fn sys_flock(fd: Fd, operation: u32) -> i32 {
    if fd < 0 {
        return Errno::Ebadf.to_ret_i32(); // EBADF
    }

    let files = crate::fs::vfs::file::get_global_files();
    let file_ref = match files.get_file(fd as u32) {
        Some(f) => f,
        None => return Errno::Ebadf.to_ret_i32(), // EBADF
    };

    let inode_ptr = file_ref.f_inode;
    if inode_ptr.is_null() {
        return Errno::Ebadf.to_ret_i32(); // EBADF
    }

    // SAFETY: inode pointer was set during open and remains valid
    let inode = unsafe { &*inode_ptr };
    let ino = inode.i_ino;

    let owner = fd as u64;
    let pid = 0u32; // TODO: get current process pid

    crate::fs::vfs::file::flock_apply(ino, operation as i32, owner, pid)
}

/// POSIX fcntl - file control operations.
/// cmd: F_DUPFD(0), F_GETFD(1), F_SETFD(2), F_GETFL(3), F_SETFL(4),
/// F_GETLK(5), F_SETLK(6), F_SETLKW(7).
/// Returns command-dependent value on success, negative errno on failure.
pub fn sys_fcntl(fd: Fd, cmd: i32, arg: u64) -> i64 {
    if fd < 0 {
        return Errno::Ebadf.to_syscall_return(); // EBADF
    }

    use crate::fs::vfs::file::fcntl_cmd;

    match cmd {
        c if c == fcntl_cmd::F_DUPFD => {
            let min_fd = arg as u32;
            let files = crate::fs::vfs::file::get_global_files();
            let src_file = match files.get_file(fd as u32) {
                Some(f) => f,
                None => return Errno::Ebadf.to_syscall_return(),
            };
            let inode_ptr = src_file.f_inode;
            let src_flags = src_file.f_flags;
            let src_mode = src_file.f_mode;

            let new_fd = match files.alloc_fd() {
                Some(nfd) => {
                    if nfd < min_fd {
                        files.free_fd(nfd);
                        for i in min_fd..256 {
                            if files.get_file(i).is_none() {
                                let mut new_file = crate::fs::vfs::inode::File::new();
                                new_file.f_flags = src_flags;
                                new_file.f_mode = src_mode;
                                new_file.f_inode = inode_ptr;
                                if files.install_file(i, &mut new_file as *mut crate::fs::vfs::inode::File) {
                                    break;
                                }
                            }
                        }
                        return Errno::Enotty.to_syscall_return(); // EMFILE
                    }
                    nfd
                }
                None => return Errno::Enotty.to_syscall_return(), // EMFILE
            };

            let mut new_file = crate::fs::vfs::inode::File::new();
            new_file.f_flags = src_flags;
            new_file.f_mode = src_mode;
            new_file.f_inode = inode_ptr;
            if files.install_file(new_fd, &mut new_file as *mut crate::fs::vfs::inode::File) {
                new_fd as i64
            } else {
                -24
            }
        }

        c if c == fcntl_cmd::F_GETFD => {
            let files = crate::fs::vfs::file::get_global_files();
            match files.get_file(fd as u32) {
                Some(_) => {
                    if (fd as usize) < 256 {
                        if let Some(ref desc) = files.fd_array[fd as usize] {
                            if desc.close_on_exec {
                                return crate::fs::vfs::file::fd_flags::FD_CLOEXEC as i64;
                            }
                        }
                    }
                    0
                }
                None => Errno::Ebadf.to_ret_i32(),
            }
        }

        c if c == fcntl_cmd::F_SETFD => {
            let files = crate::fs::vfs::file::get_global_files();
            if (fd as usize) >= 256 {
                return Errno::Ebadf.to_ret_i32();
            }
            if let Some(ref mut desc) = files.fd_array[fd as usize] {
                desc.close_on_exec = (arg & crate::fs::vfs::file::fd_flags::FD_CLOEXEC as u64) != 0;
                0
            } else {
                -9
            }
        }

        c if c == fcntl_cmd::F_GETFL => {
            let files = crate::fs::vfs::file::get_global_files();
            match files.get_file(fd as u32) {
                Some(f) => f.f_flags as i64,
                None => Errno::Ebadf.to_ret_i32(),
            }
        }

        c if c == fcntl_cmd::F_SETFL => {
            let files = crate::fs::vfs::file::get_global_files();
            if let Some(f) = files.get_fd_mut_internal(fd as u32) {
                let valid_mask = (crate::fs::vfs::open_flags::O_APPEND as u32)
                    | (crate::fs::vfs::open_flags::O_NONBLOCK as u32)
                    | (crate::fs::vfs::open_flags::O_ASYNC as u32)
                    | (crate::fs::vfs::open_flags::O_DIRECT as u32);
                f.f_flags = (f.f_flags & !valid_mask) | ((arg as u32) & valid_mask);
                0
            } else {
                -9
            }
        }

        c if c == fcntl_cmd::F_GETLK || c == fcntl_cmd::F_SETLK || c == fcntl_cmd::F_SETLKW => {
            let files = crate::fs::vfs::file::get_global_files();
            let file_ref = match files.get_file(fd as u32) {
                Some(f) => f,
                None => return Errno::Ebadf.to_ret_i32(),
            };
            let inode_ptr = file_ref.f_inode;
            if inode_ptr.is_null() {
                return Errno::Ebadf.to_ret_i32();
            }

            // SAFETY: inode pointer was set during open and remains valid
            let inode = unsafe { &*inode_ptr };
            let ino = inode.i_ino;

            if arg == 0 {
                return Errno::Efault.to_ret_i32(); // EFAULT
            }

            // SAFETY: user-validated non-null pointer for lock structure
            let user_lock = unsafe { &mut *(arg as *mut crate::fs::vfs::file::FileLockRecord) };

            let mgr = crate::fs::vfs::file::lock_manager();

            match cmd {
                _ if cmd == fcntl_cmd::F_GETLK => {
                    mgr.test_lock(ino, user_lock);
                    0
                }
                _ if cmd == fcntl_cmd::F_SETLK => {
                    mgr.set_lock(ino, user_lock) as i64
                }
                _ => {
                    mgr.set_lock_wait(ino, user_lock) as i64
                }
            }
        }

        _ => Errno::Einval.to_ret_i32(), // EINVAL
    }
}

/// TruncationFile (through fd)
pub fn sys_ftruncate(_fd: Fd, _length: i64) -> i32 {
 // TODO: ImplementationTruncation
 -1
}

/// ReadDirectory
pub fn sys_getdents(_fd: Fd, _dirp: *mut Dirent, _count: usize) -> i64 {
 // TODO: ImplementationReadDirectory
 -1
}

/// GetCurrentDirectory
pub fn sys_getcwd(_buf: *mut u8, _size: usize) -> i64 {
 // TODO: ImplementationGetCurrentDirectory
 -1
}

/// improvechangeCurrentDirectory
pub fn sys_chdir(_path: *const u8) -> i32 {
 // TODO: ImplementationimprovechangeDirectory
 -1
}

#[cfg(test)]
mod tests {
 use super::*;

 #[test]
 fn test_file_flags() {
 assert_eq!(file_flags::O_RDONLY, 0);
 assert_eq!(file_flags::O_WRONLY, 1);
 assert_eq!(file_flags::O_RDWR, 2);
 assert_eq!(file_flags::O_CREAT, 0o100);
 assert_eq!(file_flags::O_APPEND, 0o2000);
 assert_eq!(file_flags::O_NONBLOCK, 0o4000);
 }

 #[test]
 fn test_file_mode() {
 assert_eq!(file_mode::S_IFREG, 0o100000);
 assert_eq!(file_mode::S_IFDIR, 0o040000);
 assert_eq!(file_mode::S_IFCHR, 0o020000);
 assert_eq!(file_mode::S_IRUSR, 0o400);
 assert_eq!(file_mode::S_IWUSR, 0o200);
 assert_eq!(file_mode::S_IXUSR, 0o100);
 }

 #[test]
 fn test_stat_new() {
 let stat = Stat::new();

 assert_eq!(stat.device_id, 0);
 assert_eq!(stat.inode_number, 0);
 assert_eq!(stat.mode, 0);
 assert_eq!(stat.size, 0);
 assert_eq!(stat.block_size, 4096);
 }

 #[test]
 fn test_fd_table_new() {
 let table = FdTable::new();

 assert_eq!(table.next_fd.load(Ordering::Relaxed), 0);
 assert_eq!(table.open_count.load(Ordering::Relaxed), 0);
 }

 #[test]
 fn test_fd_table_alloc_fd() {
 let mut table = FdTable::new();

 let fd1 = table.alloc_fd();
 assert!(fd1.is_some());
 assert_eq!(fd1.unwrap(), 0);
 assert_eq!(table.open_count.load(Ordering::Relaxed), 1);

 let fd2 = table.alloc_fd();
 assert!(fd2.is_some());
 assert_eq!(fd2.unwrap(), 1);
 assert_eq!(table.open_count.load(Ordering::Relaxed), 2);
 }

 #[test]
 fn test_fd_table_free_fd() {
 let mut table = FdTable::new();

 let fd = table.alloc_fd().unwrap();
 assert_eq!(table.open_count.load(Ordering::Relaxed), 1);

 table.free_fd(fd);
 assert_eq!(table.open_count.load(Ordering::Relaxed), 0);
 }

 #[test]
 fn test_fd_table_get_file() {
 let table = FdTable::new();

 // Set fd shouldReturn None
 let file = table.get_file(0);
 assert!(file.is_none());

 // invalid fd shouldReturn None
 let file = table.get_file(-1);
 assert!(file.is_none());

 let file = table.get_file(1024);
 assert!(file.is_none());
 }

 #[test]
 fn test_fd_table_set_file() {
 let mut table = FdTable::new();

 table.set_file(0, 0x1234);

 let file = table.get_file(0);
 assert!(file.is_some());
 assert_eq!(file.unwrap(), 0x1234);
 }

 #[test]
 fn test_fd_table_alloc_after_free() {
 let mut table = FdTable::new();

 let fd1 = table.alloc_fd().unwrap();
 let fd2 = table.alloc_fd().unwrap();

 table.free_fd(fd1);

 // NextAllocateshouldMultiplexing fd1
 let fd3 = table.alloc_fd().unwrap();
 assert_eq!(fd3, fd1);
 }

 #[test]
 fn test_sys_openat() {
 let result = sys_openat(-1, core::ptr::null(), 0, 0);
 assert_eq!(result, -1); // TODO ImplementationthenshouldReturnvalid fd
 }

 #[test]
 fn test_sys_creat() {
 let result = sys_creat(core::ptr::null(), 0);
 assert_eq!(result, -1);
 }

 #[test]
 fn test_sys_read() {
 let result = sys_read(0, core::ptr::null_mut(), 0);
 assert_eq!(result, -1);
 }

 #[test]
 fn test_sys_write() {
 let result = sys_write(1, core::ptr::null(), 0);
 assert_eq!(result, -1);
 }

 #[test]
 fn test_sys_lseek() {
 let result = sys_lseek(0, 0, 0);
 assert_eq!(result, -1);
 }

 #[test]
 fn test_sys_close() {
 let result = sys_close(0);
 assert_eq!(result, -1);
 }

 #[test]
 fn test_sys_fstat() {
 let result = sys_fstat(0, core::ptr::null_mut());
 assert_eq!(result, -1);
 }

 #[test]
 fn test_sys_stat() {
 let result = sys_stat(core::ptr::null(), core::ptr::null_mut());
 assert_eq!(result, -1);
 }

 #[test]
 fn test_sys_mkdir() {
 let result = sys_mkdir(core::ptr::null(), 0);
 assert_eq!(result, -1);
 }

 #[test]
 fn test_sys_rmdir() {
 let result = sys_rmdir(core::ptr::null());
 assert_eq!(result, -1);
 }

 #[test]
 fn test_sys_unlink() {
 let result = sys_unlink(core::ptr::null());
 assert_eq!(result, -1);
 }

 #[test]
 fn test_sys_rename() {
 let result = sys_rename(core::ptr::null(), core::ptr::null());
 assert_eq!(result, -1);
 }

 #[test]
 fn test_sys_chmod() {
 let result = sys_chmod(core::ptr::null(), 0o755);
 assert_eq!(result, -1);
 }

 #[test]
 fn test_sys_chown() {
 let result = sys_chown(core::ptr::null(), 0, 0);
 assert_eq!(result, -1);
 }

 #[test]
 fn test_sys_fsync() {
 let result = sys_fsync(0);
 assert_eq!(result, -1);
 }

 #[test]
 fn test_sys_fdatasync() {
 let result = sys_fdatasync(0);
 assert_eq!(result, -1);
 }

 #[test]
 fn test_sys_ftruncate() {
 let result = sys_ftruncate(0, 0);
 assert_eq!(result, -1);
 }

 #[test]
 fn test_sys_getdents() {
 let result = sys_getdents(0, core::ptr::null_mut(), 0);
 assert_eq!(result, -1);
 }

 #[test]
 fn test_sys_getcwd() {
 let result = sys_getcwd(core::ptr::null_mut(), 0);
 assert_eq!(result, -1);
 }

 #[test]
 fn test_sys_chdir() {
 let result = sys_chdir(core::ptr::null());
 assert_eq!(result, -1);
 }

 #[test]
 fn test_dirent() {
 let mut dirent = Dirent {
 d_ino: 123,
 d_off: 0,
 d_reclen: 0,
 d_type: 0,
 d_name: [0; 256],
 };

 dirent.d_name[0] = b't';
 dirent.d_name[1] = b'e';
 dirent.d_name[2] = b's';
 dirent.d_name[3] = b't';

 assert_eq!(dirent.d_ino, 123);
 assert_eq!(dirent.d_name[0], b't');
 }

 #[test]
 fn test_fd_type() {
 let fd: Fd = 42;
 assert_eq!(fd, 42i32);

 let invalid_fd: Fd = -1;
 assert_eq!(invalid_fd, -1);
 }
}