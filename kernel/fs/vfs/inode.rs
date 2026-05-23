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

use super::{file_mode, DevT, FileType, InoT, Stat};
use core::ptr;
use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

use crate::posix::errno::Errno;
/// Inode Operation
pub struct InodeOperations {
    /// CreateFile
    pub create: fn(parent: &Inode, name: &str, mode: u32) -> i32,
    /// FindDirectoryproject
    pub lookup: fn(parent: &Inode, name: &str) -> Option<&'static Inode>,
    /// Createlinkaccept
    pub link: fn(old: &Inode, parent: &Inode, name: &str) -> i32,
    /// Deletelinkaccept
    pub unlink: fn(parent: &Inode, name: &str) -> i32,
    /// CreateSignlinkaccept
    pub symlink: fn(parent: &Inode, name: &str, target: &str) -> i32,
    /// CreateDirectory
    pub mkdir: fn(parent: &Inode, name: &str, mode: u32) -> i32,
    /// DeleteDirectory
    pub rmdir: fn(parent: &Inode, name: &str) -> i32,
    /// CreateDeviceNode
    pub mknod: fn(parent: &Inode, name: &str, mode: u32, dev: DevT) -> i32,
    /// rename
    pub rename: fn(old_dir: &Inode, old_name: &str, new_dir: &Inode, new_name: &str) -> i32,
    /// Readlinkaccept
    pub readlink: fn(inode: &Inode, buf: &mut [u8]) -> i32,
    /// followlinkaccept
    pub follow_link: fn(inode: &Inode) -> Option<&'static Inode>,
    /// Truncation
    pub truncate: fn(inode: &Inode, size: u64) -> i32,
    /// PermissionCheck
    pub permission: fn(inode: &Inode, mask: u32) -> i32,
    /// GetProperty
    pub getattr: fn(inode: &Inode, stat: &mut Stat) -> i32,
    /// SetProperty
    pub setattr: fn(inode: &Inode, stat: &Stat) -> i32,
}

/// FileOperation
pub struct FileOperations {
    /// fixedBit
    pub llseek: fn(file: &File, offset: i64, whence: i32) -> i64,
    /// Read
    pub read: fn(file: &File, buf: &mut [u8], offset: u64) -> i64,
    /// Write
    pub write: fn(file: &File, buf: &[u8], offset: u64) -> i64,
    /// ReadDirectory
    pub readdir: fn(file: &File, buf: &mut [u8]) -> i32,
    /// selectchoose
    pub poll: fn(file: &File, wait: &mut u32) -> u32,
    /// ioctl
    pub ioctl: fn(file: &File, cmd: u32, arg: u64) -> i32,
    /// mmap
    pub mmap: fn(file: &File, addr: u64, len: u64, prot: u32, flags: u32, offset: u64) -> i64,
    /// Open
    pub open: fn(inode: &Inode, file: &mut File) -> i32,
    /// Refresh
    pub flush: fn(file: &File) -> i32,
    /// Free
    pub release: fn(inode: &Inode, file: &File) -> i32,
    /// Synchronous
    pub fsync: fn(file: &File, datasync: i32) -> i32,
    /// Asynchronous I/O
    pub aio_fsync: fn(file: &File, datasync: i32) -> i32,
    /// Lockfixed
    pub lock: fn(file: &File, cmd: i32, lock: &FileLock) -> i32,
}

/// Address SpaceOperation
pub struct AddressSpaceOperations {
    /// ReadpageFace
    pub readpage: fn(file: &File, page: &mut Page) -> i32,
    /// WritepageFace
    pub writepage: fn(page: &Page) -> i32,
    /// SynchronouspageFace
    pub sync_page: fn(page: &Page) -> i32,
    /// criterionWrite
    pub prepare_write: fn(file: &File, page: &mut Page, offset: u32, len: u32) -> i32,
    /// CommitWrite
    pub commit_write: fn(file: &File, page: &Page, offset: u32, len: u32) -> i32,
    /// FreepageFace
    pub releasepage: fn(page: &Page, gfp_mask: u32) -> i32,
    /// directaccept I/O
    pub direct_IO: fn(rw: i32, file: &File, buf: &mut [u8], offset: u64) -> i64,
}

/// Inode struct
pub struct Inode {
    /// Inode signal
    pub i_ino: InoT,
    /// referenceCount
    pub i_count: AtomicU32,
    /// FileType
    pub i_mode: u32,
    /// User ID
    pub i_uid: u32,
    /// Group ID
    pub i_gid: u32,
    /// Devicesignal
    pub i_rdev: DevT,
    /// FileSize
    pub i_size: AtomicU64,
    /// accessTime
    pub i_atime: u64,
    /// ModifyTime
    pub i_mtime: u64,
    /// StateimprovechangeTime
    pub i_ctime: u64,
    /// BlockSize
    pub i_blkbits: u32,
    /// Blocknumber
    pub i_blocks: u64,
    /// Hard linknumber
    pub i_nlink: AtomicU32,

    /// Inode Operation
    pub i_op: &'static InodeOperations,
    /// FileOperation
    pub i_fop: &'static FileOperations,
    /// Address Space
    pub i_mapping: u64,

    /// placeinDevice
    pub i_sb: u64,
    /// StateFlag
    pub i_state: AtomicU32,
    /// Lock
    pub i_lock: u64,

    /// linkformNode
    pub i_list: *mut Inode,
    pub i_hash: *mut Inode,
}

impl Inode {
    pub const fn new() -> Self {
        Inode {
            i_ino: 0,
            i_count: AtomicU32::new(0),
            i_mode: 0,
            i_uid: 0,
            i_gid: 0,
            i_rdev: 0,
            i_size: AtomicU64::new(0),
            i_atime: 0,
            i_mtime: 0,
            i_ctime: 0,
            i_blkbits: 12, // 4KB
            i_blocks: 0,
            i_nlink: AtomicU32::new(0),
            i_op: &INODE_OPS_NONE,
            i_fop: &FILE_OPS_NONE,
            i_mapping: 0,
            i_sb: 0,
            i_state: AtomicU32::new(0),
            i_lock: 0,
            i_list: ptr::null_mut(),
            i_hash: ptr::null_mut(),
        }
    }

    /// increasePlusreferenceCount
    pub fn inc_count(&self) {
        self.i_count.fetch_add(1, Ordering::Relaxed);
    }

    /// MinusfewreferenceCount
    pub fn dec_count(&self) -> u32 {
        self.i_count.fetch_sub(1, Ordering::Relaxed)
    }

    /// GetFileType
    pub fn get_type(&self) -> FileType {
        match self.i_mode & super::file_mode::S_IFMT {
            super::file_mode::S_IFREG => FileType::Regular,
            super::file_mode::S_IFDIR => FileType::Directory,
            super::file_mode::S_IFCHR => FileType::CharDevice,
            super::file_mode::S_IFBLK => FileType::BlockDevice,
            super::file_mode::S_IFIFO => FileType::Fifo,
            super::file_mode::S_IFLNK => FileType::Symlink,
            super::file_mode::S_IFSOCK => FileType::Socket,
            _ => FileType::Unknown,
        }
    }

    /// CheckifasDirectory
    pub fn is_dir(&self) -> bool {
        self.get_type() == FileType::Directory
    }

    /// CheckifasFile
    pub fn is_regular(&self) -> bool {
        self.get_type() == FileType::Regular
    }
}

/// empty Inode Operation
pub static INODE_OPS_NONE: InodeOperations = InodeOperations {
    create: |_parent, _name, _mode| -1,
    lookup: |_parent, _name| None,
    link: |_old, _parent, _name| -1,
    unlink: |_parent, _name| -1,
    symlink: |_parent, _name, _target| -1,
    mkdir: |_parent, _name, _mode| -1,
    rmdir: |_parent, _name| -1,
    mknod: |_parent, _name, _mode, _dev| -1,
    rename: |_old_dir, _old_name, _new_dir, _new_name| -1,
    readlink: |_inode, _buf| -1,
    follow_link: |_inode| None,
    truncate: |_inode, _size| -1,
    permission: |_inode, _mask| 0,
    getattr: |_inode, _stat| -1,
    setattr: |_inode, _stat| -1,
};

/// empty FileOperation
pub static FILE_OPS_NONE: FileOperations = FileOperations {
    llseek: |_file, _offset, _whence| -1,
    read: |_file, _buf, _offset| -1,
    write: |_file, _buf, _offset| -1,
    readdir: |_file, _buf| -1,
    poll: |_file, _wait| 0,
    ioctl: |_file, _cmd, _arg| -1,
    mmap: |_file, _addr, _len, _prot, _flags, _offset| -1,
    open: |_inode, _file| -1,
    flush: |_file| -1,
    release: |_inode, _file| -1,
    fsync: |_file, _datasync| -1,
    aio_fsync: |_file, _datasync| -1,
    lock: |_file, _cmd, _lock| -1,
};

/// Filestruct
pub struct File {
    /// referenceCount
    pub f_count: AtomicU32,
    /// Lock
    pub f_lock: u64,
    /// FilePosition
    pub f_pos: u64,
    /// FileMode
    pub f_mode: u32,
    /// FileFlag
    pub f_flags: u32,
    /// close Inode
    pub f_inode: *mut Inode,
    /// FileOperation
    pub f_op: &'static FileOperations,
    /// privatefiniteData
    pub f_private: u64,
}

impl File {
    pub const fn new() -> Self {
        File {
            f_count: AtomicU32::new(1),
            f_lock: 0,
            f_pos: 0,
            f_mode: 0,
            f_flags: 0,
            f_inode: ptr::null_mut(),
            f_op: &FILE_OPS_NONE,
            f_private: 0,
        }
    }
}

/// pageFacestruct
pub struct Page {
    pub flags: AtomicU64,
    pub count: AtomicU32,
    pub mapping: u64,
    pub index: u64,
}

/// FileLock
pub struct FileLock {
    pub fl_start: u64,
    pub fl_end: u64,
    pub fl_type: i32,
    pub fl_flags: u32,
}

/// Permission check masks (POSIX access())
pub mod access_mode {
    pub const F_OK: u32 = 0;
    pub const R_OK: u32 = 4;
    pub const W_OK: u32 = 2;
    pub const X_OK: u32 = 1;
}

/// Check inode permission for the given access mask.
/// Returns 0 on success, negative errno on failure.
/// - F_OK(0): check file existence
/// - R_OK(4): check read permission
/// - W_OK(2): check write permission
/// - X_OK(1): check execute permission
/// Superuser (uid == 0) bypasses permission checks for R/W;
/// for X, at least one execute bit must be set.
pub fn inode_permission(inode: &Inode, mask: u32, uid: u32, gid: u32) -> i32 {
    if mask == access_mode::F_OK {
        return 0;
    }

    let mode = inode.i_mode;
    let inode_uid = inode.i_uid;
    let inode_gid = inode.i_gid;

    let perm_bits: u32;
    if uid == 0 {
        if (mask & access_mode::X_OK) != 0 {
            let any_exec = (mode & file_mode::S_IXUSR) != 0
                || (mode & file_mode::S_IXGRP) != 0
                || (mode & file_mode::S_IXOTH) != 0;
            if !any_exec {
                return Errno::Eacces.to_ret_i32(); // EACCES
            }
        }
        return 0;
    }

    if uid == inode_uid {
        perm_bits = (mode >> 6) & 0o7;
    } else if gid == inode_gid {
        perm_bits = (mode >> 3) & 0o7;
    } else {
        perm_bits = mode & 0o7;
    }

    if (mask & access_mode::R_OK) != 0 && (perm_bits & 0o4) == 0 {
        return Errno::Eacces.to_ret_i32(); // EACCES
    }
    if (mask & access_mode::W_OK) != 0 && (perm_bits & 0o2) == 0 {
        return Errno::Eacces.to_ret_i32(); // EACCES
    }
    if (mask & access_mode::X_OK) != 0 && (perm_bits & 0o1) == 0 {
        return Errno::Eacces.to_ret_i32(); // EACCES
    }

    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inode() {
        let inode = Inode::new();
        assert_eq!(inode.i_count.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_inode_permission_f_ok() {
        let inode = Inode::new();
        assert_eq!(inode_permission(&inode, access_mode::F_OK, 100, 100), 0);
    }

    #[test]
    fn test_inode_permission_root_bypass() {
        let mut inode = Inode::new();
        inode.i_mode = 0o100444;
        inode.i_uid = 50;
        inode.i_gid = 50;
        assert_eq!(inode_permission(&inode, access_mode::R_OK, 0, 0), 0);
        assert_eq!(inode_permission(&inode, access_mode::W_OK, 0, 0), 0);
    }

    #[test]
    fn test_inode_permission_root_exec_no_bits() {
        let mut inode = Inode::new();
        inode.i_mode = 0o100644;
        assert_eq!(inode_permission(&inode, access_mode::X_OK, 0, 0), -13);
    }

    #[test]
    fn test_inode_permission_owner() {
        let mut inode = Inode::new();
        inode.i_mode = 0o100754;
        inode.i_uid = 100;
        inode.i_gid = 200;
        assert_eq!(inode_permission(&inode, access_mode::R_OK, 100, 300), 0);
        assert_eq!(inode_permission(&inode, access_mode::W_OK, 100, 300), 0);
        assert_eq!(inode_permission(&inode, access_mode::X_OK, 100, 300), 0);
    }

    #[test]
    fn test_inode_permission_group() {
        let mut inode = Inode::new();
        inode.i_mode = 0o100750;
        inode.i_uid = 50;
        inode.i_gid = 200;
        assert_eq!(inode_permission(&inode, access_mode::R_OK, 100, 200), 0);
        assert_eq!(inode_permission(&inode, access_mode::W_OK, 100, 200), 0);
        assert_eq!(inode_permission(&inode, access_mode::X_OK, 100, 200), -13);
    }

    #[test]
    fn test_inode_permission_other() {
        let mut inode = Inode::new();
        inode.i_mode = 0o100754;
        inode.i_uid = 50;
        inode.i_gid = 200;
        assert_eq!(inode_permission(&inode, access_mode::R_OK, 300, 300), 0);
        assert_eq!(inode_permission(&inode, access_mode::W_OK, 300, 300), -13);
        assert_eq!(inode_permission(&inode, access_mode::X_OK, 300, 300), 0);
    }
}
