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
use super::errno;
use super::stat;

/// VNode Type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VnodeType {
    /// Regular file
    Regular = 0,
    /// Directory
    Directory = 1,
    /// Character device
    Char = 2,
    /// BlockDevice
    Block = 3,
    /// FIFO pipe
    Fifo = 4,
    /// Symbolic link
    Link = 5,
    /// Socket
    Socket = 6,
}

/// VNode Flag
pub mod vnode_flags {
    pub const VROOT: u32 = 0x0001;
    pub const VTEXT: u32 = 0x0002;
    pub const VSYSTEM: u32 = 0x0004;
    pub const VISTTY: u32 = 0x0008;
    pub const VXLOCK: u32 = 0x0010;
    pub const VXWANT: u32 = 0x0020;
    pub const VNOCACHE: u32 = 0x0040;
    pub const VLOCK: u32 = 0x0080;
    pub const VBWAIT: u32 = 0x0100;
    pub const VSHARED: u32 = 0x0200;
}

/// VNode struct
pub struct Vnode {
    /// Reference count
    pub v_usecount: AtomicU32,
    /// WriteerCount
    pub v_writecount: AtomicU32,
    /// VNode Type
    pub v_type: VnodeType,
    /// Flag
    pub v_flag: AtomicU32,
    /// File mode
    pub v_mode: u32,
    /// User ID
    pub v_uid: u32,
    /// Group ID
    pub v_gid: u32,
    /// FileSize
    pub v_size: AtomicU64,
    /// Inode number
    pub v_ino: u64,
}

impl Vnode {
    /// Create a new VNode
    pub fn new(v_type: VnodeType) -> Self {
        Vnode {
            v_usecount: AtomicU32::new(1),
            v_writecount: AtomicU32::new(0),
            v_type,
            v_flag: AtomicU32::new(0),
            v_mode: 0,
            v_uid: 0,
            v_gid: 0,
            v_size: AtomicU64::new(0),
            v_ino: 0,
        }
    }
    
    /// Add reference
    pub fn add_ref(&self) {
        self.v_usecount.fetch_add(1, Ordering::AcqRel);
    }
    
    /// Release reference
    pub fn release(&self) -> u32 {
        self.v_usecount.fetch_sub(1, Ordering::AcqRel)
    }
    
    /// Get reference count
    pub fn get_ref_count(&self) -> u32 {
        self.v_usecount.load(Ordering::Acquire)
    }
    
    /// Check if is directory
    pub fn is_dir(&self) -> bool {
        self.v_type == VnodeType::Directory
    }
    
    /// Check if is regular file
    pub fn is_reg(&self) -> bool {
        self.v_type == VnodeType::Regular
    }
}

/// File structure
pub struct File {
    /// FileDescriptor
    pub f_fd: i32,
    /// Referenced VNode
    pub f_vnode: *mut Vnode,
    /// FileFlag
    pub f_flag: i32,
    /// FileOffset
    pub f_offset: AtomicU64,
    /// Reference count
    pub f_count: AtomicU32,
}

impl File {
    /// Create a new file
    pub fn new(fd: i32, vnode: *mut Vnode, flag: i32) -> Self {
        File {
            f_fd: fd,
            f_vnode: vnode,
            f_flag: flag,
            f_offset: AtomicU64::new(0),
            f_count: AtomicU32::new(1),
        }
    }
    
    /// ReadFile
    pub fn read(&self, buf: &mut [u8]) -> Result<usize, i32> {
        // Check permissions
        if (self.f_flags & 0x03) == 0x01 {
            // Write-only mode
            return Err(errno::EBADF);
        }

        // Get current offset
        let offset = self.f_offset.load(Ordering::Acquire);

        // Call underlying filesystem read
        let bytes_read = self.vnode.read(buf, offset)?;

        // Update offset
        self.f_offset.store(offset + bytes_read as u64, Ordering::Release);

        Ok(bytes_read)
    }

    /// WriteFile
    pub fn write(&self, buf: &[u8]) -> Result<usize, i32> {
        // Check permissions
        if (self.f_flags & 0x03) == 0x00 {
            // Read-only mode
            return Err(errno::EBADF);
        }

        // Get current offset
        let offset = self.f_offset.load(Ordering::Acquire);

        // Call underlying filesystem write
        let bytes_written = self.vnode.write(buf, offset)?;

        // Update offset
        self.f_offset.store(offset + bytes_written as u64, Ordering::Release);

        Ok(bytes_written)
    }

    /// Seek file position
    pub fn lseek(&self, offset: i64, whence: i32) -> Result<u64, i32> {
        let current = self.f_offset.load(Ordering::Acquire);

        let new_offset = match whence {
            0 => offset as u64,  // SEEK_SET
            1 => {  // SEEK_CUR
                if offset >= 0 {
                    current + offset as u64
                } else {
                    current - (-offset) as u64
                }
            }
            2 => {  // SEEK_END
                // Get file size
                let file_size = self.vnode.get_size()?;
                if offset >= 0 {
                    file_size + offset as u64
                } else {
                    file_size - (-offset) as u64
                }
            }
            _ => return Err(errno::EINVAL),
        };

        self.f_offset.store(new_offset, Ordering::Release);
        Ok(new_offset)
    }
}

/// BSD VFS compatibility layer
pub struct BsdVfsCompat {
    /// Number of VNodes
    vnode_count: AtomicU32,
    /// Number of files
    file_count: AtomicU32,
    /// NextFileDescriptor
    next_fd: AtomicU32,
}

impl BsdVfsCompat {
    pub const fn new() -> Self {
        BsdVfsCompat {
            vnode_count: AtomicU32::new(0),
            file_count: AtomicU32::new(0),
            next_fd: AtomicU32::new(3),
        }
    }
    
    /// Initialize
    pub fn init(&mut self) {
        log_info!("BSD VFS compatibility layer initialized");
    }
    
    /// OpenFile
    pub fn open(&self, path: &str, flags: i32, mode: u32) -> Result<i32, i32> {
        // Parse path
        let vnode = self.lookup_path(path)?;

        // Check permissions
        if !self.check_permission(&vnode, flags) {
            return Err(errno::EACCES);
        }

        // Allocate file descriptor
        let fd = self.next_fd.fetch_add(1, Ordering::AcqRel) as i32;
        self.file_count.fetch_add(1, Ordering::AcqRel);

        // Create file object
        let file = File::new(vnode, flags);

        // Register file descriptor
        self.register_fd(fd, file)?;

        Ok(fd)
    }

    /// Lookup path
    fn lookup_path(&self, path: &str) -> Result<VNode, i32> {
        // Simplified implementation: return an empty VNode
        // Actual implementation should traverse path and find corresponding VNode
        Ok(VNode::new())
    }

    /// Check permission
    fn check_permission(&self, vnode: &VNode, flags: i32) -> bool {
        // Simplified implementation: always return true
        // Actual implementation should check current process permissions
        true
    }

    /// Register file descriptor
    fn register_fd(&self, fd: i32, file: File) -> Result<(), i32> {
        // Simplified implementation: no operation
        // Actual implementation should store file object in file descriptor table
        Ok(())
    }
    
    /// CloseFile
    pub fn close(&self, fd: i32) -> Result<(), i32> {
        // Remove from file descriptor table
        self.unregister_fd(fd)?;

        self.file_count.fetch_sub(1, Ordering::AcqRel);
        Ok(())
    }

    /// ReadFile
    pub fn read(&self, fd: i32, buf: &mut [u8]) -> Result<usize, i32> {
        // Get file object from file descriptor table
        let file = self.get_file(fd)?;

        // Call file read method
        file.read(buf)
    }

    /// WriteFile
    pub fn write(&self, fd: i32, buf: &[u8]) -> Result<usize, i32> {
        // Get file object from file descriptor table
        let file = self.get_file(fd)?;

        // Call file write method
        file.write(buf)
    }

    /// Seek file position
    pub fn lseek(&self, fd: i32, offset: i64, whence: i32) -> Result<u64, i32> {
        // Get file object from file descriptor table
        let file = self.get_file(fd)?;

        // Call file seek method
        file.lseek(offset, whence)
    }

    /// CreateDirectory
    pub fn mkdir(&self, path: &str, mode: u32) -> Result<(), i32> {
        // Find parent directory
        let parent_vnode = self.lookup_parent_path(path)?;

        // Create directory
        parent_vnode.mkdir(path, mode)?;

        Ok(())
    }

    /// DeleteFile
    pub fn unlink(&self, path: &str) -> Result<(), i32> {
        // Find parent directory
        let parent_vnode = self.lookup_parent_path(path)?;

        // Delete file
        parent_vnode.unlink(path)?;

        Ok(())
    }

    /// DeleteDirectory
    pub fn rmdir(&self, path: &str) -> Result<(), i32> {
        // Find parent directory
        let parent_vnode = self.lookup_parent_path(path)?;

        // Delete directory
        parent_vnode.rmdir(path)?;

        Ok(())
    }

    /// Rename file
    pub fn rename(&self, old_path: &str, new_path: &str) -> Result<(), i32> {
        // Find source file
        let old_vnode = self.lookup_path(old_path)?;

        // Find target directory
        let new_parent = self.lookup_parent_path(new_path)?;

        // Rename file
        old_vnode.rename(new_parent, new_path)?;

        Ok(())
    }

    /// GetFileState
    pub fn stat(&self, path: &str) -> Result<u32, i32> {
        // Find file
        let vnode = self.lookup_path(path)?;

        // Get file status
        vnode.get_stat()
    }

    /// Get file object from file descriptor table
    fn get_file(&self, fd: i32) -> Result<&File, i32> {
        // Simplified implementation: return error
        // Actual implementation should look up from file descriptor table
        Err(errno::EBADF)
    }

    /// Remove from file descriptor table
    fn unregister_fd(&self, fd: i32) -> Result<(), i32> {
        // Simplified implementation: no operation
        // Actual implementation should remove from file descriptor table
        Ok(())
    }

    /// Lookup parent directory path
    fn lookup_parent_path(&self, path: &str) -> Result<VNode, i32> {
        // Simplified implementation: return an empty VNode
        // Actual implementation should parse path and find parent directory
        Ok(VNode::new())
    }
    
    /// Get VNode count
    pub fn get_vnode_count(&self) -> u32 {
        self.vnode_count.load(Ordering::Acquire)
    }
    
    /// Get file count
    pub fn get_file_count(&self) -> u32 {
        self.file_count.load(Ordering::Acquire)
    }
}

/// Global BSD VFS compatibility layer
static BSD_VFS_COMPAT: crate::sync_oncelock::OnceLock<BsdVfsCompat> = crate::sync_oncelock::OnceLock::new();

pub fn bsd_vfs() -> &'static BsdVfsCompat {
    BSD_VFS_COMPAT.get_or_init(BsdVfsCompat::new)
}

pub fn init_bsd_vfs() {
    let vfs = get_bsd_vfs();
    vfs.init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vnode_type_values() {
        assert_eq!(VnodeType::Regular as u32, 0);
        assert_eq!(VnodeType::Directory as u32, 1);
        assert_eq!(VnodeType::Char as u32, 2);
        assert_eq!(VnodeType::Block as u32, 3);
        assert_eq!(VnodeType::Fifo as u32, 4);
        assert_eq!(VnodeType::Link as u32, 5);
        assert_eq!(VnodeType::Socket as u32, 6);
    }

    #[test]
    fn test_vnode_flags() {
        assert_eq!(vnode_flags::VROOT, 0x0001);
        assert_eq!(vnode_flags::VTEXT, 0x0002);
        assert_eq!(vnode_flags::VSYSTEM, 0x0004);
        assert_eq!(vnode_flags::VISTTY, 0x0008);
        assert_eq!(vnode_flags::VXLOCK, 0x0010);
        assert_eq!(vnode_flags::VXWANT, 0x0020);
        assert_eq!(vnode_flags::VNOCACHE, 0x0040);
        assert_eq!(vnode_flags::VLOCK, 0x0080);
        assert_eq!(vnode_flags::VBWAIT, 0x0100);
        assert_eq!(vnode_flags::VSHARED, 0x0200);
    }

    #[test]
    fn test_vnode_new() {
        let vnode = Vnode::new(VnodeType::Regular);

        assert_eq!(vnode.get_ref_count(), 1);
        assert_eq!(vnode.v_writecount.load(Ordering::Relaxed), 0);
        assert_eq!(vnode.v_type, VnodeType::Regular);
        assert_eq!(vnode.v_flag.load(Ordering::Relaxed), 0);
        assert_eq!(vnode.v_size.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_vnode_ref_count() {
        let vnode = Vnode::new(VnodeType::Regular);

        assert_eq!(vnode.get_ref_count(), 1);

        vnode.add_ref();
        assert_eq!(vnode.get_ref_count(), 2);

        vnode.add_ref();
        vnode.add_ref();
        assert_eq!(vnode.get_ref_count(), 4);

        vnode.release();
        assert_eq!(vnode.get_ref_count(), 3);
    }

    #[test]
    fn test_vnode_type_checks() {
        let reg_vnode = Vnode::new(VnodeType::Regular);
        assert!(reg_vnode.is_reg());
        assert!(!reg_vnode.is_dir());

        let dir_vnode = Vnode::new(VnodeType::Directory);
        assert!(dir_vnode.is_dir());
        assert!(!dir_vnode.is_reg());
    }

    #[test]
    fn test_vnode_flags_operations() {
        let vnode = Vnode::new(VnodeType::Regular);

        assert_eq!(vnode.v_flag.load(Ordering::Relaxed), 0);

        vnode.v_flag.fetch_or(vnode_flags::VROOT, Ordering::Relaxed);
        assert_eq!(vnode.v_flag.load(Ordering::Relaxed), vnode_flags::VROOT);

        vnode.v_flag.fetch_or(vnode_flags::VSYSTEM, Ordering::Relaxed);
        assert_eq!(vnode.v_flag.load(Ordering::Relaxed), vnode_flags::VROOT | vnode_flags::VSYSTEM);
    }

    #[test]
    fn test_vnode_size() {
        let vnode = Vnode::new(VnodeType::Regular);

        assert_eq!(vnode.v_size.load(Ordering::Relaxed), 0);

        vnode.v_size.store(1024, Ordering::Relaxed);
        assert_eq!(vnode.v_size.load(Ordering::Relaxed), 1024);
    }

    #[test]
    fn test_file_new() {
        let vnode = Vnode::new(VnodeType::Regular);
        let file = File::new(3, &vnode as *const _ as *mut Vnode, 0);

        assert_eq!(file.f_fd, 3);
        assert_eq!(file.f_flag, 0);
        assert_eq!(file.f_offset.load(Ordering::Relaxed), 0);
        assert_eq!(file.f_count.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_file_read() {
        let vnode = Vnode::new(VnodeType::Regular);
        let file = File::new(3, &vnode as *const _ as *mut Vnode, 0);

        let mut buf = [0u8; 100];
        let result = file.read(&mut buf);
        assert!(result.is_ok());
    }

    #[test]
    fn test_file_write() {
        let vnode = Vnode::new(VnodeType::Regular);
        let file = File::new(3, &vnode as *const _ as *mut Vnode, 0);

        let buf = b"hello";
        let result = file.write(buf);
        assert!(result.is_ok());
    }

    #[test]
    fn test_file_lseek_set() {
        let vnode = Vnode::new(VnodeType::Regular);
        let file = File::new(3, &vnode as *const _ as *mut Vnode, 0);

        let result = file.lseek(100, 0); // SEEK_SET
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 100);
        assert_eq!(file.f_offset.load(Ordering::Relaxed), 100);
    }

    #[test]
    fn test_file_lseek_cur() {
        let vnode = Vnode::new(VnodeType::Regular);
        let file = File::new(3, &vnode as *const _ as *mut Vnode, 0);

        file.f_offset.store(50, Ordering::Relaxed);

        let result = file.lseek(10, 1); // SEEK_CUR
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 60);
    }

    #[test]
    fn test_file_lseek_cur_negative() {
        let vnode = Vnode::new(VnodeType::Regular);
        let file = File::new(3, &vnode as *const _ as *mut Vnode, 0);

        file.f_offset.store(100, Ordering::Relaxed);

        let result = file.lseek(-30, 1); // SEEK_CUR
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 70);
    }

    #[test]
    fn test_file_lseek_invalid() {
        let vnode = Vnode::new(VnodeType::Regular);
        let file = File::new(3, &vnode as *const _ as *mut Vnode, 0);

        let result = file.lseek(0, 99);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), errno::EINVAL);
    }

    #[test]
    fn test_bsd_vfs_compat_new() {
        let vfs = BsdVfsCompat::new();

        assert_eq!(vfs.get_vnode_count(), 0);
        assert_eq!(vfs.get_file_count(), 0);
    }

    #[test]
    fn test_bsd_vfs_compat_open() {
        let vfs = BsdVfsCompat::new();

        let result = vfs.open("/test", 0, 0o644);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);
        assert_eq!(vfs.get_file_count(), 1);

        let result = vfs.open("/test2", 0, 0o644);
        assert_eq!(result.unwrap(), 4);
        assert_eq!(vfs.get_file_count(), 2);
    }

    #[test]
    fn test_bsd_vfs_compat_close() {
        let vfs = BsdVfsCompat::new();

        vfs.open("/test", 0, 0).unwrap();
        assert_eq!(vfs.get_file_count(), 1);

        let result = vfs.close(3);
        assert!(result.is_ok());
        assert_eq!(vfs.get_file_count(), 0);
    }

    #[test]
    fn test_bsd_vfs_compat_read() {
        let vfs = BsdVfsCompat::new();

        let mut buf = [0u8; 100];
        let result = vfs.read(3, &mut buf);
        assert!(result.is_ok());
    }

    #[test]
    fn test_bsd_vfs_compat_write() {
        let vfs = BsdVfsCompat::new();

        let buf = b"hello";
        let result = vfs.write(3, buf);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 5);
    }

    #[test]
    fn test_bsd_vfs_compat_lseek() {
        let vfs = BsdVfsCompat::new();

        let result = vfs.lseek(3, 100, 0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_bsd_vfs_compat_mkdir() {
        let vfs = BsdVfsCompat::new();

        let result = vfs.mkdir("/test", 0o755);
        assert!(result.is_ok());
    }

    #[test]
    fn test_bsd_vfs_compat_unlink() {
        let vfs = BsdVfsCompat::new();

        let result = vfs.unlink("/test");
        assert!(result.is_ok());
    }

    #[test]
    fn test_bsd_vfs_compat_rmdir() {
        let vfs = BsdVfsCompat::new();

        let result = vfs.rmdir("/test");
        assert!(result.is_ok());
    }

    #[test]
    fn test_bsd_vfs_compat_rename() {
        let vfs = BsdVfsCompat::new();

        let result = vfs.rename("/old", "/new");
        assert!(result.is_ok());
    }

    #[test]
    fn test_bsd_vfs_compat_stat() {
        let vfs = BsdVfsCompat::new();

        let result = vfs.stat("/test");
        assert!(result.is_ok());
    }

    #[test]
    fn test_vnode_all_types() {
        let types = [
            VnodeType::Regular,
            VnodeType::Directory,
            VnodeType::Char,
            VnodeType::Block,
            VnodeType::Fifo,
            VnodeType::Link,
            VnodeType::Socket,
        ];

        for (i, t) in types.iter().enumerate() {
            let vnode = Vnode::new(*t);
            assert_eq!(vnode.v_type as u32, i as u32);
        }
    }
}