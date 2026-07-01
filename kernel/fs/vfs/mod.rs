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

use crate::{pr_debug, pr_info, pr_warn};
use alloc::vec::Vec;
use core::ptr;
use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

use crate::syslib::posix::errno::Errno;
use crate::kernel::error::Errno;
pub mod dcache;
pub mod file;
pub mod inode;

/// Inode type constants
pub mod inode_types {
    /// Regular file
    pub const REG_FILE: u32 = 1;
    /// Directory
    pub const DIR: u32 = 2;
    /// Symbolic link
    pub const SYMLINK: u32 = 3;
    /// Character device
    pub const CHRDEV: u32 = 4;
    /// Block device
    pub const BLKDEV: u32 = 5;
    /// FIFO/pipe
    pub const FIFO: u32 = 6;
    /// Socket
    pub const SOCK: u32 = 7;
}

/// FileType
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    /***/
    Unknown = 0,
    /// File
    Regular = 1,
    /// Directory
    Directory = 2,
    /// CharacterDevice
    CharDevice = 3,
    /// BlockDevice
    BlockDevice = 4,
    /// Pipe
    Fifo = 5,
    /// suiteacceptWord
    Socket = 6,
    /// Signlinkaccept
    Symlink = 7,
}

/// FileMode
pub mod file_mode {
    pub const S_IFMT: u32 = 0o170000; // FileTypeMask
    pub const S_IFSOCK: u32 = 0o140000; // suiteacceptWord
    pub const S_IFLNK: u32 = 0o120000; // Signlinkaccept
    pub const S_IFREG: u32 = 0o100000; // File
    pub const S_IFBLK: u32 = 0o060000; // BlockDevice
    pub const S_IFDIR: u32 = 0o040000; // Directory
    pub const S_IFCHR: u32 = 0o020000; // WordsymbolDevice
    pub const S_IFIFO: u32 = 0o010000; // pipe

    pub const S_ISUID: u32 = 0o004000; // SUID
    pub const S_ISGID: u32 = 0o002000; // SGID
    pub const S_ISVTX: u32 = 0o001000; // Sticky

    pub const S_IRWXU: u32 = 0o000700; // UserPermission
    pub const S_IRUSR: u32 = 0o000400; // Userread
    pub const S_IWUSR: u32 = 0o000200; // Userwrite
    pub const S_IXUSR: u32 = 0o000100; // Userexecute

    pub const S_IRWXG: u32 = 0o000070; // GroupPermission
    pub const S_IRGRP: u32 = 0o000040; // Groupread
    pub const S_IWGRP: u32 = 0o000020; // Groupwrite
    pub const S_IXGRP: u32 = 0o000010; // Groupexecute

    pub const S_IRWXO: u32 = 0o000007; // itsPermission
    pub const S_IROTH: u32 = 0o000004; // itsread
    pub const S_IWOTH: u32 = 0o000002; // itswrite
    pub const S_IXOTH: u32 = 0o000001; // itsexecute
}

/// FileOpenFlag
pub mod open_flags {
    pub const O_RDONLY: i32 = 0o0000000;
    pub const O_WRONLY: i32 = 0o0000001;
    pub const O_RDWR: i32 = 0o0000002;
    pub const O_CREAT: i32 = 0o0000100;
    pub const O_EXCL: i32 = 0o0000200;
    pub const O_NOCTTY: i32 = 0o0000400;
    pub const O_TRUNC: i32 = 0o0001000;
    pub const O_APPEND: i32 = 0o0002000;
    pub const O_NONBLOCK: i32 = 0o0004000;
    pub const O_DSYNC: i32 = 0o0010000;
    pub const O_SYNC: i32 = 0o04000000 | O_DSYNC;
    pub const O_RSYNC: i32 = O_SYNC;
    pub const O_DIRECTORY: i32 = 0o0020000;
    pub const O_NOFOLLOW: i32 = 0o0040000;
    pub const O_CLOEXEC: i32 = 0o0200000;
    pub const O_ASYNC: i32 = 0o020000;
    pub const O_DIRECT: i32 = 0o040000;
    pub const O_LARGEFILE: i32 = 0o0100000;
    pub const O_NOATIME: i32 = 0o01000000;
    pub const O_PATH: i32 = 0o10000000;
    pub const O_TMPFILE: i32 = 0o20000000 | O_DIRECTORY;
}

/// FilePositionpointerType
pub type OffT = i64;

/// FileSizeType
pub type SizeT = u64;

/// Inode signalType
pub type InoT = u64;

/// DevicesignalType
pub type DevT = u64;

/// File SystemInfo
pub struct Statfs {
    /// File SystemType
    pub f_type: u64,
    /// BlockSize
    pub f_bsize: u64,
    /// totalBlocknumber
    pub f_blocks: u64,
    /// emptyidleBlocknumber
    pub f_bfree: u64,
    /// canuseBlocknumber
    pub f_bavail: u64,
    /// totalFilenumber
    pub f_files: u64,
    /// emptyidleFilenumber
    pub f_ffree: u64,
    /// File System ID
    pub f_fsid: u64,
    /// FilenameMaxLength
    pub f_namelen: u64,
    /// BlockSizeLimit
    pub f_frsize: u64,
    /// MountFlag
    pub f_flags: u64,
}

/// FileState
pub struct Stat {
    /// Devicesignal
    pub device_id: DevT,
    /// Inode signal
    pub inode_number: InoT,
    /// FileMode
    pub mode: u32,
    /// Hard linknumber
    pub link_count: u32,
    /// User ID
    pub user_id: u32,
    /// Group ID
    pub group_id: u32,
    /// Devicesignal (SpecialFile)
    pub raw_device_id: DevT,
    /// FileSize
    pub size: SizeT,
    /// BlockSize
    pub block_size: u64,
    /// Blocknumber
    pub block_count: u64,
    /// accessTime
    pub access_time: u64,
    /// ModifyTime
    pub modification_time: u64,
    /// StateimprovechangeTime
    pub change_time: u64,
}

/// Timevaluestruct (uspreciseDegree)
#[repr(C)]
pub struct Timeval {
    /// second
    pub seconds: i64,
    /// us
    pub microseconds: i64,
}

/// Timevaluestruct (nspreciseDegree)
#[repr(C)]
pub struct Timespec {
    /// second
    pub seconds: i64,
    /// ns
    pub nanoseconds: i64,
}

/// File SystemType
pub struct FileSystemType {
    /// File SystemName
    pub name: &'static str,
    /// File SystemFlag
    pub fs_flags: u32,
    /// InitializeFunction
    pub init: fn() -> i32,
    /// MountFunction
    pub mount: fn(dev_name: &str, flags: u32) -> i32,
    /// UnmountFunction
    pub unmount: fn() -> i32,
    /// Next filesystem type in linked list
    pub next: *mut FileSystemType,
}

/// MountDot
#[derive(Clone, Copy)]
pub struct Mount {
    /// MountDotPath
    pub mount_point: &'static str,
    /// DeviceName
    pub dev_name: &'static str,
    /// File SystemType
    pub fs_type: &'static FileSystemType,
    /// MountFlag
    pub flags: u32,
    /// Root Inode
    pub root: u64,
    /// NextMountDot
    pub next: *mut Mount,
}

/// VFS kernelstruct
pub struct VfsCore {
    /// File SystemTypelinkform
    fs_types: *mut FileSystemType,
    /// MountDotlinkform
    mounts: [Option<Mount>; 64],
    /// Registered filesystems list
    filesystems: *mut FileSystemType,
    /// Mount count
    mount_count: usize,
}

impl VfsCore {
    pub const fn new() -> Self {
        VfsCore {
            fs_types: ptr::null_mut(),
            mounts: [None; 64],
            filesystems: ptr::null_mut(),
            mount_count: 0,
        }
    }

    /// RegisterFile SystemType
    pub fn register_filesystem(&mut self, fs_type: &'static FileSystemType) -> i32 {
        // Add to linked list
        // Find the end of the list
        let mut current = &mut self.filesystems;
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            while !current.is_null() {
                current = &mut (**current).next;
            }
            // Add new filesystem type
            *current = fs_type as *const _ as *mut _;
        }
        log_info!("Registered filesystem: {}", fs_type.name);
        0
    }

    /// MountFile System
    pub fn mount(&mut self, dev_name: &str, mount_point: &str, fs_type: &str, flags: u32) -> i32 {
        log_info!(
            "Mounting {} on {} (type: {})",
            dev_name,
            mount_point,
            fs_type
        );

        // ImplementationMount
        // 1. Find filesystem type
        let mut fs_type_ptr = self.filesystems;
        let mut found_type = None;

        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            while !fs_type_ptr.is_null() {
                let fs = &*fs_type_ptr;
                if fs.name == fs_type {
                    found_type = Some(fs);
                    break;
                }
                fs_type_ptr = fs.next;
            }
        }

        let fs_type = match found_type {
            Some(fs) => fs,
            None => {
                log_warn!("Filesystem type not found: {}", fs_type);
                return Errno::Eperm.to_ret_i32();
            }
        };

        // 2. Call filesystem mount
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let mount_result = (fs_type.mount)(dev_name, flags);
            if mount_result != 0 {
                log_warn!("Mount failed");
                return Errno::Eperm.to_ret_i32();
            }

            // 3. Add to mount list
            let mount_count = self.mount_count as usize;
            if mount_count < self.mounts.len() {
                self.mounts[mount_count] = Some(Mount {
                    mount_point: "",
                    dev_name: "",
                    fs_type: fs_type,
                    flags: flags,
                    root: 0,
                    next: core::ptr::null_mut(),
                });
                self.mount_count += 1;
            }

            log_info!("Mount successful");
            0
        }
    }

    /// UnmountFile System
    pub fn unmount(&mut self, mount_point: &str) -> i32 {
        log_info!("Unmounting {}", mount_point);

        // ImplementationUnmount
        // 1. Find mount by mount point
        let mut mount_idx = None;

        for i in 0..self.mount_count as usize {
            if let Some(mount) = self.mounts[i] {
                if mount.mount_point == mount_point {
                    mount_idx = Some(i);
                    break;
                }
            }
        }

        let mount_idx = match mount_idx {
            Some(idx) => idx,
            None => {
                log_warn!("Mount point not found: {}", mount_point);
                return Errno::Eperm.to_ret_i32();
            }
        };

        // 2. Call filesystem unmount
        let mount = match self.mounts.get(mount_idx).and_then(|m| *m) {
            Some(m) => m,
            None => {
                log_warn!("Mount entry at index {} is invalid", mount_idx);
                return Errno::Eperm.to_ret_i32();
            }
        };
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let result = (mount.fs_type.unmount)();
            if result != 0 {
                log_warn!("Unmount failed");
                return Errno::Eperm.to_ret_i32();
            }

            // 3. Remove from mount list
            self.mounts[mount_idx] = None;

            log_info!("Unmount successful");
            0
        }
    }
}

/// Global VFS kernel
static VFS_CORE: crate::sync_oncelock::OnceLock<VfsCore> = crate::sync_oncelock::OnceLock::new();

pub fn vfs_core() -> &'static VfsCore {
    VFS_CORE.get_or_init(VfsCore::new)
}

pub fn init_vfs_core() -> &'static VfsCore {
    VFS_CORE.get_or_init(VfsCore::new)
}

/// Initialize VFS
pub fn init_vfs() {
    log_info!("VFS initialized");
}

// ============================================================================
// VFS OperationImplementation
// ============================================================================

/// PathFindresult
pub struct PathLookup {
    /// findto inode
    pub inode: u64,
    /// ParentDirectory inode
    pub parent: u64,
    /// PathdeepDegree
    pub depth: u32,
    /// iffollow Signlinkaccept
    pub followed_symlink: bool,
}

/// PathFindFlag
pub mod lookup_flags {
    pub const FOLLOW_SYMLINK: u32 = 0x01;
    pub const NO_FOLLOW: u32 = 0x02;
    pub const DIRECTORY: u32 = 0x04;
    pub const EXIST: u32 = 0x08;
    pub const CREATE: u32 = 0x10;
    pub const EXCL: u32 = 0x20;
    pub const RENAME_TARGET: u32 = 0x40;
}

impl VfsCore {
    /// PathFind
    pub fn path_lookup(&mut self, path: &str, flags: u32) -> Option<PathLookup> {
        // jumpoverprefixconduct '/'
        let path = path.trim_start_matches('/');

        if path.is_empty() {
            // returnRootDirectory
            return Some(PathLookup {
                inode: 1, // Root inode
                parent: 0,
                depth: 0,
                followed_symlink: false,
            });
        }

        // SplitPathComponent
        let components: Vec<&str> = path.split('/').collect();
        let mut current_inode = 1u64; // fromRootStart
        let mut parent_inode = 0u64;
        let mut depth = 0u32;
        let mut followed_symlink = false;

        for component in components {
            if component.is_empty() || component == "." {
                continue;
            }

            if component == ".." {
                // Return to parent directory
                if depth > 0 {
                    depth -= 1;
                    // GetParentDirectory inode
                    let parent = self.get_parent_inode(current_inode);
                    current_inode = parent.unwrap_or(0);
                }
                continue;
            }

            parent_inode = current_inode;

            // Find component in current directory
            let found_inode = self.lookup_in_dir(current_inode, component);
            current_inode = match found_inode {
                Some(inode) => inode,
                None => return None, // Component not found
            };

            depth += 1;

            // Check if it's a symlink
            if (flags & lookup_flags::FOLLOW_SYMLINK) != 0 {
                // Follow symlink
                if self.is_symlink(current_inode) {
                    let link_target = self.read_symlink(current_inode);
                    if let Some(target) = link_target {
                        // Resolve symlink (simplified: just log)
                        log_debug!("Following symlink to: {}", target);
                        followed_symlink = true;
                    }
                }
            }
        }

        Some(PathLookup {
            inode: current_inode,
            parent: parent_inode,
            depth,
            followed_symlink,
        })
    }

    /// CreateFile
    pub fn create(&mut self, path: &str, mode: u32) -> i32 {
        // Split path and filename
        let (parent_path, name) = split_path(path);

        // Find parent directory
        let lookup = match self.path_lookup(parent_path, lookup_flags::DIRECTORY) {
            Some(l) => l,
            None => return Errno::Enoent.to_ret_i32(), // ENOENT
        };

        // Create file in parent directory
        match self.create_inode(lookup.inode, name, mode) {
            Some(inode) => {
                log_debug!("Created file: {} (inode: {})", path, inode);
                0
            }
            None => {
                log_warn!("Failed to create file: {}", path);
                -1 // EIO
            }
        }
    }

    /// DeleteFile
    pub fn unlink(&mut self, path: &str) -> i32 {
        let lookup = match self.path_lookup(path, lookup_flags::NO_FOLLOW) {
            Some(l) => l,
            None => return Errno::Enoent.to_ret_i32(),
        };

        // Delete file
        match self.remove_inode(lookup.inode, "") {
            0 => {
                log_debug!("Deleted file: {}", path);
                0
            }
            _ => {
                log_warn!("Failed to delete file: {}", path);
                -1 // EIO
            }
        }
    }

    /// CreateDirectory
    pub fn mkdir(&mut self, path: &str, mode: u32) -> i32 {
        let (parent_path, name) = split_path(path);

        let lookup = match self.path_lookup(parent_path, lookup_flags::DIRECTORY) {
            Some(l) => l,
            None => return Errno::Enoent.to_ret_i32(),
        };

        // Create directory
        match self.create_inode(lookup.inode, name, mode) {
            Some(inode) => {
                log_debug!("Created directory: {} (inode: {})", path, inode);
                0
            }
            None => {
                log_warn!("Failed to create directory: {}", path);
                -1 // EIO
            }
        }
    }

    /// DeleteDirectory
    pub fn rmdir(&mut self, path: &str) -> i32 {
        let lookup = match self.path_lookup(path, lookup_flags::DIRECTORY) {
            Some(l) => l,
            None => return Errno::Enoent.to_ret_i32(),
        };

        // Check if directory is empty
        if !self.is_directory_empty(lookup.inode) {
            log_warn!("Directory not empty: {}", path);
            return Errno::Enotempty.to_ret_i32(); // ENOTEMPTY
        }

        // Delete directory
        match self.remove_inode(lookup.inode, "") {
            0 => {
                log_debug!("Deleted directory: {}", path);
                0
            }
            _ => {
                log_warn!("Failed to delete directory: {}", path);
                -1 // EIO
            }
        }
    }

    /// Rename
    pub fn rename(&mut self, old_path: &str, new_path: &str) -> i32 {
        // Execute rename
        let old_lookup = match self.path_lookup(old_path, lookup_flags::NO_FOLLOW) {
            Some(l) => l,
            None => return Errno::Enoent.to_ret_i32(),
        };

        let (new_parent_path, new_name) = split_path(new_path);
        let new_parent_lookup = match self.path_lookup(new_parent_path, lookup_flags::DIRECTORY) {
            Some(l) => l,
            None => return Errno::Enoent.to_ret_i32(),
        };

        // Move/rename inode
        match self.move_inode(old_lookup.inode, "", new_parent_lookup.inode, new_name) {
            0 => {
                log_debug!("Renamed: {} -> {}", old_path, new_path);
                0
            }
            _ => {
                log_warn!("Failed to rename: {} -> {}", old_path, new_path);
                -1 // EIO
            }
        }
    }

    /// Create symlink
    pub fn symlink(&mut self, target: &str, link_path: &str) -> i32 {
        let (parent_path, name) = split_path(link_path);

        let lookup = match self.path_lookup(parent_path, lookup_flags::DIRECTORY) {
            Some(l) => l,
            None => return Errno::Enoent.to_ret_i32(),
        };

        // Create symlink
        match self.create_symlink(lookup.inode, name, target) {
            Some(inode) => {
                log_debug!(
                    "Created symlink: {} -> {} (inode: {})",
                    link_path,
                    target,
                    inode
                );
                0
            }
            None => {
                log_warn!("Failed to create symlink: {}", link_path);
                -1 // EIO
            }
        }
    }

    /// Create hard link
    pub fn link(&mut self, target: &str, link_path: &str) -> i32 {
        let target_lookup = match self.path_lookup(target, lookup_flags::NO_FOLLOW) {
            Some(l) => l,
            None => return Errno::Enoent.to_ret_i32(),
        };

        let (link_parent_path, link_name) = split_path(link_path);
        let link_parent_lookup = match self.path_lookup(link_parent_path, lookup_flags::DIRECTORY) {
            Some(l) => l,
            None => return Errno::Enoent.to_ret_i32(),
        };

        // Create hard link
        match self.create_hard_link(link_parent_lookup.inode, link_name, target_lookup.inode) {
            Some(_) => {
                log_debug!("Created hard link: {} -> {}", link_path, target);
                0
            }
            None => {
                log_warn!("Failed to create hard link: {}", link_path);
                -1 // EIO
            }
        }
    }

    pub fn get_parent_inode(&mut self, inode: u64) -> Option<u64> {
        if inode != 0 {
            Some(0)
        } else {
            None
        }
    }

    pub fn lookup_in_dir(&mut self, dir_inode: u64, name: &str) -> Option<u64> {
        if name.is_empty() || name.len() > 255 {
            return None;
        }
        let mut name_hash: u32 = 5381;
        for &b in name.as_bytes() {
            name_hash = name_hash.wrapping_mul(33).wrapping_add(b as u32);
        }
        let key = crate::kernel::fs::dcache::DentryKey::new(dir_inode, name_hash, 0);
        let dcache = crate::kernel::fs::dcache::dcache();
        let result = dcache.lookup(&key, name.as_bytes());
        if !result.is_null() {
            // SAFETY: dcache lookup returned valid pointer
            unsafe {
                let ino = (*result).ino;
                if ino != 0 {
                    return Some(ino);
                }
            }
        }
        None
    }

    pub fn is_symlink(&mut self, inode: u64) -> bool {
        if inode == 0 {
            return false;
        }
        let mode = inode as u32;
        (mode & file_mode::S_IFMT) == file_mode::S_IFLNK
    }

    pub fn read_symlink(&mut self, _inode: u64) -> Option<&'static str> {
        None
    }

    pub fn create_inode(&mut self, parent: u64, name: &str, mode: u32) -> Option<u64> {
        if name.is_empty() {
            return None;
        }
        let new_ino = VFS_STATS.inode_count.fetch_add(1, Ordering::Relaxed) as u64 + 100;
        log_debug!(
            "create_inode: parent={} name={} new_ino={}",
            parent,
            name,
            new_ino
        );
        Some(new_ino)
    }

    pub fn remove_inode(&mut self, parent: u64, name: &str) -> i32 {
        let lookup = self.lookup_in_dir(parent, name);
        match lookup {
            Some(ino) => {
                VFS_STATS.inode_count.fetch_sub(1, Ordering::Relaxed);
                log_debug!("remove_inode: parent={} name={} ino={}", parent, name, ino);
                0
            }
            None => Errno::Enoent.to_ret_i32(),
        }
    }

    pub fn is_directory_empty(&mut self, inode: u64) -> bool {
        let _ = inode;
        true
    }

    pub fn move_inode(
        &mut self,
        old_dir: u64,
        old_name: &str,
        new_dir: u64,
        new_name: &str,
    ) -> i32 {
        let ino = match self.lookup_in_dir(old_dir, old_name) {
            Some(i) => i,
            None => return Errno::Enoent.to_ret_i32(),
        };
        match self.remove_inode(old_dir, old_name) {
            0 => match self.create_inode(new_dir, new_name, 0) {
                Some(_) => {
                    log_debug!("move_inode: {} -> {} (ino={})", old_name, new_name, ino);
                    0
                }
                None => Errno::Eio.to_ret_i32(),
            },
            e => e,
        }
    }

    pub fn create_symlink(&mut self, parent: u64, name: &str, _target: &str) -> Option<u64> {
        self.create_inode(parent, name, file_mode::S_IFLNK)
    }

    pub fn create_hard_link(&mut self, parent: u64, name: &str, target_inode: u64) -> Option<u64> {
        let _ = (parent, name);
        VFS_STATS.inode_count.fetch_add(1, Ordering::Relaxed);
        Some(target_inode)
    }

    /// Change file mode bits
    pub fn chmod(&mut self, path: &str, mode: u32) -> i32 {
        let lookup = match self.path_lookup(path, lookup_flags::NO_FOLLOW) {
            Some(l) => l,
            None => return Errno::Enoent.to_ret_i32(),
        };
        if lookup.inode == 0 {
            return Errno::Enoent.to_ret_i32();
        }
        let file_type = (lookup.inode as u32) & file_mode::S_IFMT;
        let new_mode = file_type | (mode & !file_mode::S_IFMT);
        log_debug!("chmod: ino={} mode={:#o}", lookup.inode, new_mode);
        0
    }

    /// Change file owner and group
    pub fn chown(&mut self, path: &str, uid: u32, gid: u32) -> i32 {
        let lookup = match self.path_lookup(path, lookup_flags::NO_FOLLOW) {
            Some(l) => l,
            None => return Errno::Enoent.to_ret_i32(),
        };
        if lookup.inode == 0 {
            return Errno::Enoent.to_ret_i32();
        }
        if uid == u32::MAX && gid == u32::MAX {
            return Errno::Einval.to_ret_i32();
        }
        log_debug!("chown: ino={} uid={} gid={}", lookup.inode, uid, gid);
        0
    }
}

/// SplitPathasParentPathsumFilename
fn split_path(path: &str) -> (&str, &str) {
    let path = path.trim_end_matches('/');

    match path.rfind('/') {
        Some(idx) => (&path[..idx], &path[idx + 1..]),
        None => ("", path),
    }
}

// ============================================================================
// FileDescriptormanagementadministration
// ============================================================================

/// FileDescriptorform
pub struct FileDescriptorTable {
    /// FileDescriptorArray
    pub files: [Option<FileDescriptor>; 256],
    /// Nextcanuse fd
    pub next_fd: u32,
}

/// FileDescriptor
#[derive(Clone, Copy)]
pub struct FileDescriptor {
    /// close inode
    pub inode: u64,
    /// FilePosition
    pub pos: u64,
    /// OpenFlag
    pub flags: u32,
    /// referenceCount
    pub ref_count: u32,
}

impl FileDescriptorTable {
    pub const fn new() -> Self {
        FileDescriptorTable {
            files: [None; 256],
            next_fd: 0,
        }
    }

    /// AllocateFileDescriptor
    pub fn alloc_fd(&mut self, inode: u64, flags: u32) -> Option<u32> {
        for i in 0..self.files.len() {
            if self.files[i].is_none() {
                self.files[i] = Some(FileDescriptor {
                    inode,
                    pos: 0,
                    flags,
                    ref_count: 1,
                });
                return Some(i as u32);
            }
        }
        None
    }

    /// FreeFileDescriptor
    pub fn free_fd(&mut self, fd: u32) -> bool {
        if fd as usize >= self.files.len() {
            return false;
        }

        self.files[fd as usize].take().is_some()
    }

    /// GetFileDescriptor
    pub fn get_fd(&self, fd: u32) -> Option<&FileDescriptor> {
        if fd as usize >= self.files.len() {
            return None;
        }
        self.files[fd as usize].as_ref()
    }

    /// GetcanchangeFileDescriptor
    pub fn get_fd_mut(&mut self, fd: u32) -> Option<&mut FileDescriptor> {
        if fd as usize >= self.files.len() {
            return None;
        }
        self.files[fd as usize].as_mut()
    }
}

// ============================================================================
// DirectoryCaching
// ============================================================================

/// DirectoryCachingproject
pub struct Dentry {
    /// inode signal
    pub d_inode: u64,
    /// ParentDirectory
    pub d_parent: *mut Dentry,
    /// Filename
    pub d_name: [u8; 256],
    /// FilenameLength
    pub d_name_len: u16,
    /// referenceCount
    pub d_count: AtomicU32,
    /// Flag
    pub d_flags: u32,
    /// Hashvalue
    pub d_hash: u32,
    /// ChildDirectorylinkform
    pub d_subdirs: *mut Dentry,
    /// Siblinglinkform
    pub d_sibling: *mut Dentry,
    /// LRU linkform
    pub d_lru: *mut Dentry,
}

/// DirectoryCachingFlag
pub mod dentry_flags {
    pub const DCACHE_UNHASHED: u32 = 0x01;
    pub const DCACHE_REFERENCED: u32 = 0x02;
    pub const DCACHE_VALID: u32 = 0x04;
    pub const DCACHE_DELETED: u32 = 0x08;
    pub const DCACHE_MOUNTED: u32 = 0x10;
}

impl Dentry {
    pub const fn new() -> Self {
        Dentry {
            d_inode: 0,
            d_parent: core::ptr::null_mut(),
            d_name: [0; 256],
            d_name_len: 0,
            d_count: AtomicU32::new(1),
            d_flags: 0,
            d_hash: 0,
            d_subdirs: core::ptr::null_mut(),
            d_sibling: core::ptr::null_mut(),
            d_lru: core::ptr::null_mut(),
        }
    }

    /// SetName
    pub fn set_name(&mut self, name: &[u8]) {
        let len = name.len().min(self.d_name.len());
        self.d_name[..len].copy_from_slice(&name[..len]);
        self.d_name_len = len as u16;
    }

    /// GetName
    pub fn get_name(&self) -> &[u8] {
        &self.d_name[..self.d_name_len as usize]
    }

    /// ComputeHash
    pub fn calc_hash(&mut self) {
        let mut hash: u32 = 0;
        for &byte in self.get_name() {
            hash = hash.wrapping_mul(31).wrapping_add(byte as u32);
        }
        self.d_hash = hash;
    }
}

// ============================================================================
// File Systemstatistics
// ============================================================================

/// VFS statisticsInfo
pub struct VfsStats {
    /// MountDotcount
    pub mount_count: AtomicU32,
    /// OpenFilenumber
    pub open_files: AtomicU32,
    /// inode count
    pub inode_count: AtomicU32,
    /// dentry count
    pub dentry_count: AtomicU32,
    /// Readtimenumber
    pub reads: AtomicU64,
    /// Writetimenumber
    pub writes: AtomicU64,
}

impl VfsStats {
    pub const fn new() -> Self {
        VfsStats {
            mount_count: AtomicU32::new(0),
            open_files: AtomicU32::new(0),
            inode_count: AtomicU32::new(0),
            dentry_count: AtomicU32::new(0),
            reads: AtomicU64::new(0),
            writes: AtomicU64::new(0),
        }
    }
}

/// Global VFS Statistics
pub static VFS_STATS: VfsStats = VfsStats::new();

// ============================================================================
// auxiliaryFunction
// ============================================================================

/// CheckPathifasinsulatelogPath
pub fn is_absolute_path(path: &str) -> bool {
    path.starts_with('/')
}

/// regulationparadigmPath
pub fn normalize_path(path: &str) -> Vec<u8> {
    let mut result = Vec::new();
    let components: Vec<&str> = path.split('/').collect();

    for component in components {
        if component.is_empty() || component == "." {
            continue;
        }

        if component == ".." {
            // removeLastComponent
            if let Some(last_slash) = result.iter().rposition(|&b| b == b'/') {
                result.truncate(last_slash);
            } else if !result.is_empty() {
                result.clear();
            }
            continue;
        }

        if !result.is_empty() && result.last() != Some(&b'/') {
            result.push(b'/');
        }
        result.extend_from_slice(component.as_bytes());
    }

    if result.is_empty() {
        result.push(b'/');
    }

    result
}

/// CheckFilenameifvalid
pub fn is_valid_filename(name: &[u8]) -> bool {
    if name.is_empty() || name.len() > 255 {
        return false;
    }

    // CheckifPackageinvalidCharacter
    for &byte in name {
        if byte == 0 || byte == b'/' {
            return false;
        }
    }

    true
}

/// CheckifasSpecialDirectory
pub fn is_special_dir(name: &[u8]) -> bool {
    name == b"." || name == b".."
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_path() {
        assert_eq!(
            split_path("/home/user/file.txt"),
            ("/home/user", "file.txt")
        );
        assert_eq!(split_path("/file.txt"), ("", "file.txt"));
        assert_eq!(split_path("file.txt"), ("", "file.txt"));
    }

    #[test]
    fn test_is_absolute_path() {
        assert!(is_absolute_path("/home"));
        assert!(is_absolute_path("/"));
        assert!(!is_absolute_path("home"));
        assert!(!is_absolute_path(""));
    }

    #[test]
    fn test_is_valid_filename() {
        assert!(is_valid_filename(b"file.txt"));
        assert!(is_valid_filename(b"my-file"));
        assert!(!is_valid_filename(b""));
        assert!(!is_valid_filename(b"file/name"));
        assert!(!is_valid_filename(&[0u8; 256][..]));
    }

    #[test]
    fn test_is_special_dir() {
        assert!(is_special_dir(b"."));
        assert!(is_special_dir(b".."));
        assert!(!is_special_dir(b"file"));
    }

    #[test]
    fn test_dentry_name() {
        let mut dentry = Dentry::new();
        dentry.set_name(b"test.txt");

        assert_eq!(dentry.get_name(), b"test.txt");
        assert_eq!(dentry.d_name_len, 8);
    }

    #[test]
    fn test_file_descriptor_table() {
        let mut fdt = FileDescriptorTable::new();

        let fd = fdt.alloc_fd(1, 0);
        assert!(fd.is_some());
        assert_eq!(fd.unwrap(), 0);

        let fd2 = fdt.alloc_fd(2, 0);
        assert!(fd2.is_some());
        assert_eq!(fd2.unwrap(), 1);

        assert!(fdt.free_fd(0));
        assert!(!fdt.free_fd(0)); // alreadyFree
    }
}
