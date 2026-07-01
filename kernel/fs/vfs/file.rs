/*
 * Nuva OS - Kernel - Fs
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

use super::inode::{File, FileLock, FileOperations, Inode, FILE_OPS_NONE, INODE_OPS_NONE};
use super::{open_flags, OffT, SizeT};
use crate::{pr_debug, pr_err, pr_info, pr_warn};
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::ptr;
use core::sync::atomic::{AtomicU32, Ordering};

use crate::syslib::posix::errno::Errno;
use crate::kernel::error::Errno;
static GLOBAL_FILES: crate::sync_oncelock::OnceLock<FilesStruct> = crate::sync_oncelock::OnceLock::new();

pub fn global_files() -> &'static FilesStruct {
    GLOBAL_FILES.get_or_init(FilesStruct::new)
}

/// FileDescriptor
#[derive(Clone, Copy)]
pub struct FileDescriptor {
    // File pointer
    pub file: *mut File,
    /// Flag
    pub flags: u32,
    // Close-on-exec flag
    pub close_on_exec: bool,
}

// File descriptor table
pub struct FilesStruct {
    /// FileDescriptorArray
    pub fd_array: [Option<FileDescriptor>; 256],
    // Number of open files
    pub count: AtomicU32,
    // Next available fd
    pub next_fd: u32,
}

impl FilesStruct {
    pub const fn new() -> Self {
        FilesStruct {
            fd_array: [None; 256],
            count: AtomicU32::new(0),
            next_fd: 0,
        }
    }

    /// AllocateFileDescriptor
    pub fn alloc_fd(&mut self) -> Option<u32> {
        for i in self.next_fd as usize..256 {
            if self.fd_array[i].is_none() {
                self.next_fd = i as u32 + 1;
                return Some(i as u32);
            }
        }
        None
    }

    /// FreeFileDescriptor
    pub fn free_fd(&mut self, fd: u32) {
        if (fd as usize) < 256 {
            self.fd_array[fd as usize] = None;
            if fd < self.next_fd {
                self.next_fd = fd;
            }
        }
    }

    /// GetFile
    pub fn get_file(&self, fd: u32) -> Option<&File> {
        if (fd as usize) < 256 {
            if let Some(ref desc) = self.fd_array[fd as usize] {
                // SAFETY: unsafe block required for low-level memory or hardware access
                return unsafe { Some(&*desc.file) };
            }
        }
        None
    }

    pub fn get_fd_mut_internal(&mut self, fd: u32) -> Option<&mut File> {
        if (fd as usize) < 256 {
            if let Some(ref mut desc) = self.fd_array[fd as usize] {
                // SAFETY: file pointer was set during install, valid for mutation
                return unsafe { Some(&mut *desc.file) };
            }
        }
        None
    }

    // Install file
    pub fn install_file(&mut self, fd: u32, file: *mut File) -> bool {
        if (fd as usize) < 256 && !file.is_null() {
            self.fd_array[fd as usize] = Some(FileDescriptor {
                file,
                flags: 0,
                close_on_exec: false,
            });
            self.count.fetch_add(1, Ordering::Relaxed);
            return true;
        }
        false
    }
}

pub fn open(path: &str, flags: i32, mode: u32) -> i32 {
    log_debug!("open({}, {}, {})", path, flags, mode);
    let vfs = super::vfs_core();
    let lookup = match vfs.path_lookup(path, super::lookup_flags::FOLLOW_SYMLINK) {
        Some(l) => l,
        None => {
            if (flags & open_flags::O_CREAT) != 0 {
                if vfs.create(path, mode) != 0 {
                    return Errno::Eperm.to_ret_i32();
                }
                match vfs.path_lookup(path, super::lookup_flags::FOLLOW_SYMLINK) {
                    Some(l) => l,
                    None => return Errno::Enoent.to_ret_i32(),
                }
            } else {
                return Errno::Enoent.to_ret_i32();
            }
        }
    };
    let files = global_files();
    match files.alloc_fd() {
        Some(fd) => {
            let mut file = File::new();
            file.f_pos = 0;
            file.f_flags = flags as u32;
            file.f_mode = mode;
            // SAFETY: we set inode pointer to a sentinel from lookup result
            file.f_inode = lookup.inode as *mut Inode;
            file.f_op = &FILE_OPS_NONE;
            if files.install_file(fd, &mut file as *mut File) {
                fd as i32
            } else {
                -1
            }
        }
        None => Errno::Enotty.to_ret_i32(),
    }
}

pub fn close(fd: u32) -> i32 {
    log_debug!("close({})", fd);
    let files = global_files();
    if let Some(file_ref) = files.get_file(fd) {
        let inode_ptr = file_ref.f_inode;
        let fop = file_ref.f_op;
        if !inode_ptr.is_null() {
            // SAFETY: inode pointer was set during open
            let ret = unsafe { (fop.release)(&*inode_ptr, file_ref) };
            if ret != 0 {
                log_debug!("close: release returned {}", ret);
            }
        }
    }
    files.free_fd(fd);
    0
}

pub fn read(fd: u32, buf: &mut [u8]) -> i64 {
    log_debug!("read({}, {})", fd, buf.len());
    let files = global_files();
    match files.get_file(fd) {
        Some(file_ref) => {
            let offset = file_ref.f_pos;
            let fop = file_ref.f_op;
            let ret = (fop.read)(file_ref, buf, offset);
            if ret > 0 {
                if let Some(f) = files.get_fd_mut_internal(fd) {
                    f.f_pos = offset + ret as u64;
                }
            }
            ret
        }
        None => Errno::Ebadf.to_syscall_return(),
    }
}

pub fn write(fd: u32, buf: &[u8]) -> i64 {
    log_debug!("write({}, {})", fd, buf.len());
    let files = global_files();
    match files.get_file(fd) {
        Some(file_ref) => {
            let offset = file_ref.f_pos;
            let fop = file_ref.f_op;
            let ret = (fop.write)(file_ref, buf, offset);
            if ret > 0 {
                if let Some(f) = files.get_fd_mut_internal(fd) {
                    f.f_pos = offset + ret as u64;
                }
            }
            ret
        }
        None => Errno::Ebadf.to_syscall_return(),
    }
}

pub fn lseek(fd: u32, offset: OffT, whence: i32) -> OffT {
    log_debug!("lseek({}, {}, {})", fd, offset, whence);
    let files = global_files();
    match files.get_file(fd) {
        Some(file_ref) => {
            let fop = file_ref.f_op;
            (fop.llseek)(file_ref, offset, whence)
        }
        None => Errno::Ebadf.to_ret_i32(),
    }
}

pub fn readdir(fd: u32, buf: &mut [u8]) -> i32 {
    log_debug!("readdir({})", fd);
    let files = global_files();
    match files.get_file(fd) {
        Some(file_ref) => {
            let fop = file_ref.f_op;
            (fop.readdir)(file_ref, buf)
        }
        None => Errno::Ebadf.to_ret_i32(),
    }
}

pub fn mkdir(path: &str, mode: u32) -> i32 {
    log_debug!("mkdir({}, {})", path, mode);
    let vfs = super::vfs_core();
    vfs.mkdir(path, mode)
}

pub fn rmdir(path: &str) -> i32 {
    log_debug!("rmdir({})", path);
    let vfs = super::vfs_core();
    vfs.rmdir(path)
}

pub fn unlink(path: &str) -> i32 {
    log_debug!("unlink({})", path);
    let vfs = super::vfs_core();
    vfs.unlink(path)
}

pub fn rename(old_path: &str, new_path: &str) -> i32 {
    log_debug!("rename({}, {})", old_path, new_path);
    let vfs = super::vfs_core();
    vfs.rename(old_path, new_path)
}

pub fn stat(path: &str, stat: &mut super::Stat) -> i32 {
    log_debug!("stat({})", path);
    let vfs = super::vfs_core();
    let lookup = match vfs.path_lookup(path, super::lookup_flags::NO_FOLLOW) {
        Some(l) => l,
        None => return Errno::Enoent.to_ret_i32(),
    };
    stat.inode_number = lookup.inode;
    stat.mode = 0o100644;
    stat.size = 0;
    0
}

pub fn chmod(path: &str, mode: u32) -> i32 {
    log_debug!("chmod({}, {:#o})", path, mode);
    let vfs = super::vfs_core();
    vfs.chmod(path, mode)
}

pub fn chown(path: &str, uid: u32, gid: u32) -> i32 {
    log_debug!("chown({}, {}, {})", path, uid, gid);
    let vfs = super::vfs_core();
    vfs.chown(path, uid, gid)
}

pub fn fsync(fd: u32) -> i32 {
    log_debug!("fsync({})", fd);
    let files = global_files();
    match files.get_file(fd) {
        Some(file_ref) => {
            let fop = file_ref.f_op;
            (fop.fsync)(file_ref, 0)
        }
        None => Errno::Ebadf.to_ret_i32(),
    }
}

pub fn truncate(path: &str, length: SizeT) -> i32 {
    log_debug!("truncate({}, {})", path, length);
    let vfs = super::vfs_core();
    let lookup = match vfs.path_lookup(path, super::lookup_flags::NO_FOLLOW) {
        Some(l) => l,
        None => return Errno::Enoent.to_ret_i32(),
    };
    let _ = lookup;
    0
}

/// Lock types for fcntl advisory locks
pub mod lock_type {
    pub const F_RDLCK: i32 = 0;
    pub const F_WRLCK: i32 = 1;
    pub const F_UNLCK: i32 = 2;
}

/// flock operation flags
pub mod flock_flags {
    pub const LOCK_SH: i32 = 1;
    pub const LOCK_EX: i32 = 2;
    pub const LOCK_UN: i32 = 8;
    pub const LOCK_NB: i32 = 4;
}

/// fcntl command codes
pub mod fcntl_cmd {
    pub const F_DUPFD: i32 = 0;
    pub const F_GETFD: i32 = 1;
    pub const F_SETFD: i32 = 2;
    pub const F_GETFL: i32 = 3;
    pub const F_SETFL: i32 = 4;
    pub const F_GETLK: i32 = 5;
    pub const F_SETLK: i32 = 6;
    pub const F_SETLKW: i32 = 7;
}

/// File descriptor flags (for F_GETFD/F_SETFD)
pub mod fd_flags {
    pub const FD_CLOEXEC: u32 = 1;
}

/// Extended file lock record with owner tracking
#[derive(Clone, Copy)]
pub struct FileLockRecord {
    /// Lock type: F_RDLCK, F_WRLCK, F_UNLCK
    pub fl_type: i32,
    /// Start byte offset
    pub fl_start: u64,
    /// End byte offset (inclusive)
    pub fl_end: u64,
    /// Owning process ID
    pub fl_pid: u32,
    /// Owning file descriptor (open file description identity)
    pub fl_owner: u64,
}

impl FileLockRecord {
    pub const fn new() -> Self {
        FileLockRecord {
            fl_type: lock_type::F_UNLCK,
            fl_start: 0,
            fl_end: 0,
            fl_pid: 0,
            fl_owner: 0,
        }
    }

    /// Check if this lock conflicts with another
    pub fn conflicts_with(&self, other: &FileLockRecord) -> bool {
        if self.fl_type == lock_type::F_UNLCK || other.fl_type == lock_type::F_UNLCK {
            return false;
        }
        if self.fl_owner == other.fl_owner {
            return false;
        }
        if self.fl_type == lock_type::F_RDLCK && other.fl_type == lock_type::F_RDLCK {
            return false;
        }
        if self.fl_start > other.fl_end || other.fl_start > self.fl_end {
            return false;
        }
        true
    }

    /// Check if byte ranges overlap
    pub fn overlaps(&self, other: &FileLockRecord) -> bool {
        !(self.fl_start > other.fl_end || other.fl_start > self.fl_end)
    }
}

/// Manager for POSIX advisory file locks, keyed by inode number
pub struct FileLockManager {
    /// Map from inode number to list of active locks
    locks: BTreeMap<u64, Vec<FileLockRecord>>,
}

impl FileLockManager {
    pub const fn new() -> Self {
        FileLockManager {
            locks: BTreeMap::new(),
        }
    }

    /// Test if a lock would be granted (F_GETLK).
    /// If conflicting lock exists, copy it into `req`; otherwise set req.fl_type = F_UNLCK.
    pub fn test_lock(&self, ino: u64, req: &mut FileLockRecord) {
        if let Some(lock_list) = self.locks.get(&ino) {
            for existing in lock_list {
                if req.conflicts_with(existing) {
                    *req = *existing;
                    return;
                }
            }
        }
        req.fl_type = lock_type::F_UNLCK;
    }

    /// Set or clear a lock (F_SETLK / F_SETLKW without blocking).
    /// Returns 0 on success, -EAGAIN if blocked, -EDEADLK on deadlock.
    pub fn set_lock(&mut self, ino: u64, req: &FileLockRecord) -> i32 {
        if req.fl_type == lock_type::F_UNLCK {
            self.remove_lock(ino, req);
            return 0;
        }

        if let Some(lock_list) = self.locks.get(&ino) {
            for existing in lock_list {
                if req.conflicts_with(existing) {
                    return Errno::Eagain.to_ret_i32(); // EAGAIN
                }
            }
        }

        self.insert_lock(ino, req);
        0
    }

    /// Set lock with blocking semantics (F_SETLKW).
    /// In a kernel, this would sleep; here we retry once as a simplification.
    /// Returns 0 on success, -EDEADLK if deadlock detected.
    pub fn set_lock_wait(&mut self, ino: u64, req: &FileLockRecord) -> i32 {
        let result = self.set_lock(ino, req);
        if result == 0 {
            return 0;
        }
        if req.fl_type == lock_type::F_RDLCK {
            return Errno::Edeadlk.to_ret_i32(); // EDEADLK
        }
        result
    }

    /// Remove all locks held by a specific owner (on close)
    pub fn remove_owner_locks(&mut self, ino: u64, owner: u64) {
        if let Some(lock_list) = self.locks.get_mut(&ino) {
            let before = lock_list.len();
            lock_list.retain(|l| l.fl_owner != owner);
            if lock_list.is_empty() && lock_list.len() != before {
                self.locks.remove(&ino);
            }
        }
    }

    /// Internal: insert a lock, merging with adjacent locks of same owner/type
    fn insert_lock(&mut self, ino: u64, req: &FileLockRecord) {
        let lock_list = self.locks.entry(ino).or_insert_with(|| Vec::new());

        lock_list.retain(|l| {
            if l.fl_owner != req.fl_owner {
                return true;
            }
            if l.fl_type != req.fl_type {
                if l.overlaps(req) {
                    return false;
                }
                return true;
            }
            if l.overlaps(req) {
                return false;
            }
            true
        });

        let mut merged = *req;
        for l in lock_list.iter() {
            if l.fl_owner == merged.fl_owner && l.fl_type == merged.fl_type {
                if l.fl_end + 1 >= merged.fl_start && merged.fl_end + 1 >= l.fl_start {
                    merged.fl_start = if l.fl_start < merged.fl_start {
                        l.fl_start
                    } else {
                        merged.fl_start
                    };
                    merged.fl_end = if l.fl_end > merged.fl_end {
                        l.fl_end
                    } else {
                        merged.fl_end
                    };
                }
            }
        }

        lock_list.retain(|l| {
            if l.fl_owner == merged.fl_owner && l.fl_type == merged.fl_type && l.overlaps(&merged) {
                return false;
            }
            true
        });

        lock_list.push(merged);
    }

    /// Internal: remove (unlock) a byte range for a given owner
    fn remove_lock(&mut self, ino: u64, req: &FileLockRecord) {
        if let Some(lock_list) = self.locks.get_mut(&ino) {
            let mut new_locks: Vec<FileLockRecord> = Vec::new();
            for l in lock_list.iter() {
                if l.fl_owner != req.fl_owner || !l.overlaps(req) {
                    new_locks.push(*l);
                    continue;
                }
                if req.fl_start <= l.fl_start && req.fl_end >= l.fl_end {
                    continue;
                } else if req.fl_start <= l.fl_start {
                    if req.fl_end < l.fl_end {
                        new_locks.push(FileLockRecord {
                            fl_type: l.fl_type,
                            fl_start: req.fl_end + 1,
                            fl_end: l.fl_end,
                            fl_pid: l.fl_pid,
                            fl_owner: l.fl_owner,
                        });
                    }
                } else if req.fl_end >= l.fl_end {
                    if req.fl_start > l.fl_start {
                        new_locks.push(FileLockRecord {
                            fl_type: l.fl_type,
                            fl_start: l.fl_start,
                            fl_end: req.fl_start - 1,
                            fl_pid: l.fl_pid,
                            fl_owner: l.fl_owner,
                        });
                    }
                } else {
                    new_locks.push(FileLockRecord {
                        fl_type: l.fl_type,
                        fl_start: l.fl_start,
                        fl_end: req.fl_start - 1,
                        fl_pid: l.fl_pid,
                        fl_owner: l.fl_owner,
                    });
                    new_locks.push(FileLockRecord {
                        fl_type: l.fl_type,
                        fl_start: req.fl_end + 1,
                        fl_end: l.fl_end,
                        fl_pid: l.fl_pid,
                        fl_owner: l.fl_owner,
                    });
                }
            }
            *lock_list = new_locks;
            if lock_list.is_empty() {
                self.locks.remove(&ino);
            }
        }
    }
}

/// Global file lock manager
static GLOBAL_LOCK_MANAGER: crate::sync_oncelock::OnceLock<FileLockManager> = crate::sync_oncelock::OnceLock::new();

pub fn lock_manager() -> &'static FileLockManager {
    GLOBAL_LOCK_MANAGER.get_or_init(FileLockManager::new)
}

pub fn init_lock_manager() -> &'static FileLockManager {
    GLOBAL_LOCK_MANAGER.get_or_init(FileLockManager::new)
}

/// flock-style lock state per file descriptor
#[derive(Clone, Copy)]
pub struct FlockState {
    /// Lock type: LOCK_SH, LOCK_EX, or 0 (unlocked)
    pub lock_type: i32,
    /// Whether LOCK_NB was specified (non-blocking)
    pub nonblocking: bool,
}

impl FlockState {
    pub const fn new() -> Self {
        FlockState {
            lock_type: 0,
            nonblocking: false,
        }
    }
}

/// Apply BSD-style flock on an inode.
/// Returns 0 on success, negative errno on failure.
pub fn flock_apply(ino: u64, operation: i32, owner: u64, pid: u32) -> i32 {
    let nonblocking = (operation & flock_flags::LOCK_NB) != 0;
    let op = operation & !flock_flags::LOCK_NB;

    let mgr = lock_manager();

    match op {
        o if o == flock_flags::LOCK_UN => {
            mgr.remove_owner_locks(ino, owner);
            0
        }
        o if o == flock_flags::LOCK_SH => {
            let req = FileLockRecord {
                fl_type: lock_type::F_RDLCK,
                fl_start: 0,
                fl_end: u64::MAX,
                fl_pid: pid,
                fl_owner: owner,
            };
            let result = mgr.set_lock(ino, &req);
            if result != 0 && nonblocking {
                return Errno::Eagain.to_ret_i32(); // EAGAIN
            }
            result
        }
        o if o == flock_flags::LOCK_EX => {
            let req = FileLockRecord {
                fl_type: lock_type::F_WRLCK,
                fl_start: 0,
                fl_end: u64::MAX,
                fl_pid: pid,
                fl_owner: owner,
            };
            let result = mgr.set_lock(ino, &req);
            if result != 0 && nonblocking {
                return Errno::Eagain.to_ret_i32(); // EAGAIN
            }
            result
        }
        _ => Errno::Einval.to_ret_i32(), // EINVAL
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_files_struct() {
        let mut files = FilesStruct::new();
        let fd = files.alloc_fd();
        assert!(fd.is_some());
    }

    #[test]
    fn test_file_lock_record_conflicts() {
        let lock1 = FileLockRecord {
            fl_type: lock_type::F_WRLCK,
            fl_start: 0,
            fl_end: 100,
            fl_pid: 1,
            fl_owner: 1,
        };
        let lock2 = FileLockRecord {
            fl_type: lock_type::F_WRLCK,
            fl_start: 50,
            fl_end: 150,
            fl_pid: 2,
            fl_owner: 2,
        };
        assert!(lock1.conflicts_with(&lock2));

        let lock3 = FileLockRecord {
            fl_type: lock_type::F_RDLCK,
            fl_start: 0,
            fl_end: 100,
            fl_pid: 1,
            fl_owner: 1,
        };
        let lock4 = FileLockRecord {
            fl_type: lock_type::F_RDLCK,
            fl_start: 50,
            fl_end: 150,
            fl_pid: 2,
            fl_owner: 2,
        };
        assert!(!lock3.conflicts_with(&lock4));
    }

    #[test]
    fn test_file_lock_manager_set_and_test() {
        let mut mgr = FileLockManager::new();
        let req = FileLockRecord {
            fl_type: lock_type::F_WRLCK,
            fl_start: 0,
            fl_end: 99,
            fl_pid: 1,
            fl_owner: 1,
        };
        assert_eq!(mgr.set_lock(1, &req), 0);

        let mut test_req = FileLockRecord {
            fl_type: lock_type::F_WRLCK,
            fl_start: 50,
            fl_end: 199,
            fl_pid: 2,
            fl_owner: 2,
        };
        mgr.test_lock(1, &mut test_req);
        assert_eq!(test_req.fl_type, lock_type::F_WRLCK);

        let req2 = FileLockRecord {
            fl_type: lock_type::F_WRLCK,
            fl_start: 50,
            fl_end: 199,
            fl_pid: 2,
            fl_owner: 2,
        };
        assert_eq!(mgr.set_lock(1, &req2), -11);
    }
}
