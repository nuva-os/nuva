/*
 * Nuva OS - Nuva OS
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

//! NuvaFS Inode implementation

use core::sync::atomic::{AtomicU64, AtomicU32, Ordering};

/// Inode type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum InodeMode {
    Unknown = 0,
    Regular = 0x8000,
    Directory = 0x4000,
    Symlink = 0xA000,
    FIFO = 0x1000,
    Socket = 0xC000,
    BlockDev = 0x6000,
    CharDev = 0x2000,
}

/// Inode flags
pub const INODE_FLAG_SYNC: u32 = 1 << 0;
pub const INODE_FLAG_IMMUTABLE: u32 = 1 << 1;
pub const INODE_FLAG_APPEND: u32 = 1 << 2;
pub const INODE_FLAG_NODUMP: u32 = 1 << 3;
pub const INODE_FLAG_NOATIME: u32 = 1 << 4;
pub const INODE_FLAG_COMPRESSED: u32 = 1 << 5;
pub const INODE_FLAG_ENCRYPTED: u32 = 1 << 6;

/// Direct block count
pub const DIRECT_BLOCKS: usize = 12;

/// Indirect block levels
pub const INDIRECT_LEVELS: usize = 3;

/// Extent structure
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct Extent {
    /// Logical block number
    pub logical: u64,
    
    /// Physical block number
    pub physical: u64,
    
    /// Length (block count)
    pub length: u32,
}

impl Extent {
    pub const fn new(logical: u64, physical: u64, length: u32) -> Self {
        Self { logical, physical, length }
    }

    pub fn end(&self) -> u64 {
        self.logical + self.length as u64
    }

    pub fn contains(&self, block: u64) -> bool {
        block >= self.logical && block < self.end()
    }
}

/// Extent header
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct ExtentHeader {
    pub magic: u16,
    pub entries: u16,
    pub max_entries: u16,
    pub depth: u16,
    pub generation: u32,
}

pub const EXTENT_MAGIC: u16 = 0xF30A;

/// NuvaFS Inode
#[derive(Debug)]
#[repr(C)]
pub struct NuvaInode {
    /// Inode number
    pub ino: u64,
    
    /// Type and permissions
    pub mode: u16,
    
    /// Link count
    pub links: u16,
    
    /// User ID
    pub uid: u32,
    
    /// Group ID
    pub gid: u32,
    
    /// File size
    pub size: AtomicU64,
    
    /// Allocated block count
    pub blocks: AtomicU64,
    
    /// Access time
    pub atime: AtomicU64,
    
    /// Modification time
    pub mtime: AtomicU64,
    
    /// Status change time
    pub ctime: AtomicU64,
    
    /// Creation time
    pub crtime: u64,
    
    /// Flags
    pub flags: AtomicU32,
    
    /// Block size (for compression)
    pub block_size: u32,
    
    /// Direct block pointers
    pub direct: [u64; DIRECT_BLOCKS],
    
    /// Indirect block pointers
    pub indirect: [u64; INDIRECT_LEVELS],
    
    /// Extent header (for extent mode)
    pub extent_header: ExtentHeader,
    
    /// Extent array
    pub extents: [Extent; 4],
    
    /// Extended attribute block
    pub xattr_block: u64,
    
    /// Checksum
    pub checksum: u32,
}

impl NuvaInode {
    pub fn new(ino: u64, mode: InodeMode) -> Self {
        Self {
            ino,
            mode: mode as u16,
            links: 1,
            uid: 0,
            gid: 0,
            size: AtomicU64::new(0),
            blocks: AtomicU64::new(0),
            atime: AtomicU64::new(0),
            mtime: AtomicU64::new(0),
            ctime: AtomicU64::new(0),
            crtime: 0,
            flags: AtomicU32::new(0),
            block_size: 4096,
            direct: [0; DIRECT_BLOCKS],
            indirect: [0; INDIRECT_LEVELS],
            extent_header: ExtentHeader {
                magic: EXTENT_MAGIC,
                entries: 0,
                max_entries: 4,
                depth: 0,
                generation: 0,
            },
            extents: [Extent::new(0, 0, 0); 4],
            xattr_block: 0,
            checksum: 0,
        }
    }

    pub fn is_dir(&self) -> bool {
        (self.mode & 0xF000) == InodeMode::Directory as u16
    }

    pub fn is_regular(&self) -> bool {
        (self.mode & 0xF000) == InodeMode::Regular as u16
    }

    pub fn is_symlink(&self) -> bool {
        (self.mode & 0xF000) == InodeMode::Symlink as u16
    }

    pub fn get_size(&self) -> u64 {
        self.size.load(Ordering::Relaxed)
    }

    pub fn set_size(&self, size: u64) {
        self.size.store(size, Ordering::Relaxed);
    }

    pub fn add_block(&self) {
        self.blocks.fetch_add(1, Ordering::Relaxed);
    }

    pub fn get_blocks(&self) -> u64 {
        self.blocks.load(Ordering::Relaxed)
    }

    /// Find extent
    pub fn find_extent(&self, logical: u64) -> Option<&Extent> {
        if self.extent_header.magic != EXTENT_MAGIC {
            return None;
        }

        for i in 0..self.extent_header.entries as usize {
            if i < self.extents.len() && self.extents[i].contains(logical) {
                return Some(&self.extents[i]);
            }
        }
        None
    }

    /// Add extent
    pub fn add_extent(&mut self, extent: Extent) -> bool {
        if self.extent_header.magic != EXTENT_MAGIC {
            self.extent_header.magic = EXTENT_MAGIC;
            self.extent_header.entries = 0;
        }

        let idx = self.extent_header.entries as usize;
        if idx < self.extents.len() {
            self.extents[idx] = extent;
            self.extent_header.entries += 1;
            return true;
        }
        false
    }

    /// Allocate block
    pub fn allocate_block(&mut self, logical: u64, physical: u64) {
        // Try to extend existing extent
        for i in 0..self.extent_header.entries as usize {
            if i < self.extents.len() {
                let ext = &mut self.extents[i];
                if ext.end() == logical && ext.physical + ext.length as u64 == physical {
                    ext.length += 1;
                    return;
                }
            }
        }

        // Create new extent
        self.add_extent(Extent::new(logical, physical, 1));
    }
}

/// Inode cache
pub struct InodeCache {
    inodes: [Option<NuvaInode>; 1024],
    lru: [u64; 1024],
    head: AtomicU32,
}

impl InodeCache {
    pub const fn new() -> Self {
        Self {
            inodes: [None; 1024],
            lru: [0; 1024],
            head: AtomicU32::new(0),
        }
    }

    pub fn get(&self, ino: u64) -> Option<&NuvaInode> {
        for i in 0..1024 {
            if let Some(ref inode) = self.inodes[i] {
                if inode.ino == ino {
                    return Some(inode);
                }
            }
        }
        None
    }

    pub fn get_mut(&mut self, ino: u64) -> Option<&mut NuvaInode> {
        for i in 0..1024 {
            if let Some(ref mut inode) = self.inodes[i] {
                if inode.ino == ino {
                    return Some(inode);
                }
            }
        }
        None
    }

    pub fn insert(&mut self, inode: NuvaInode) {
        let head = self.head.load(Ordering::Relaxed) as usize;
        self.inodes[head % 1024] = Some(inode);
        self.head.fetch_add(1, Ordering::Relaxed);
    }

    pub fn remove(&mut self, ino: u64) {
        for i in 0..1024 {
            if let Some(ref inode) = self.inodes[i] {
                if inode.ino == ino {
                    self.inodes[i] = None;
                    return;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inode_mode() {
        assert_eq!(InodeMode::Regular as u16, 0x8000);
        assert_eq!(InodeMode::Directory as u16, 0x4000);
        assert_eq!(InodeMode::Symlink as u16, 0xA000);
    }

    #[test]
    fn test_inode_new() {
        let inode = NuvaInode::new(1, InodeMode::Regular);
        assert_eq!(inode.ino, 1);
        assert_eq!(inode.mode, InodeMode::Regular as u16);
        assert_eq!(inode.links, 1);
        assert_eq!(inode.get_size(), 0);
        assert!(inode.is_regular());
        assert!(!inode.is_dir());
    }

    #[test]
    fn test_inode_directory() {
        let inode = NuvaInode::new(2, InodeMode::Directory);
        assert!(inode.is_dir());
        assert!(!inode.is_regular());
    }

    #[test]
    fn test_inode_size() {
        let inode = NuvaInode::new(3, InodeMode::Regular);
        assert_eq!(inode.get_size(), 0);

        inode.set_size(1024);
        assert_eq!(inode.get_size(), 1024);

        inode.set_size(4096);
        assert_eq!(inode.get_size(), 4096);
    }

    #[test]
    fn test_inode_blocks() {
        let inode = NuvaInode::new(4, InodeMode::Regular);
        assert_eq!(inode.get_blocks(), 0);

        inode.add_block();
        assert_eq!(inode.get_blocks(), 1);

        inode.add_block();
        assert_eq!(inode.get_blocks(), 2);
    }

    #[test]
    fn test_extent() {
        let extent = Extent::new(0, 100, 10);
        assert_eq!(extent.logical, 0);
        assert_eq!(extent.physical, 100);
        assert_eq!(extent.length, 10);
        assert_eq!(extent.end(), 10);

        assert!(extent.contains(0));
        assert!(extent.contains(5));
        assert!(extent.contains(9));
        assert!(!extent.contains(10));
        assert!(!extent.contains(100));
    }

    #[test]
    fn test_inode_extent() {
        let mut inode = NuvaInode::new(5, InodeMode::Regular);

        // Find non-existent extent
        assert!(inode.find_extent(0).is_none());

        // Add extent
        let extent = Extent::new(0, 1000, 10);
        assert!(inode.add_extent(extent));

        // Find existing extent
        let found = inode.find_extent(5);
        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.logical, 0);
        assert_eq!(found.physical, 1000);
        assert_eq!(found.length, 10);
    }

    #[test]
    fn test_inode_allocate_block() {
        let mut inode = NuvaInode::new(6, InodeMode::Regular);

        // Allocate first block
        inode.allocate_block(0, 100);

        // Verify extent
        let extent = inode.find_extent(0);
        assert!(extent.is_some());
        let extent = extent.unwrap();
        assert_eq!(extent.logical, 0);
        assert_eq!(extent.physical, 100);
        assert_eq!(extent.length, 1);

        // Allocate contiguous block (should extend extent)
        inode.allocate_block(1, 101);
        let extent = inode.find_extent(0).unwrap();
        assert_eq!(extent.length, 2);
    }

    #[test]
    fn test_inode_cache() {
        let mut cache = InodeCache::new();

        // Insert inode
        let inode = NuvaInode::new(100, InodeMode::Regular);
        cache.insert(inode);

        // Find
        let found = cache.get(100);
        assert!(found.is_some());

        // Find non-existent
        let not_found = cache.get(999);
        assert!(not_found.is_none());

        // Remove
        cache.remove(100);
        let not_found = cache.get(100);
        assert!(not_found.is_none());
    }

    #[test]
    fn test_inode_flags() {
        let inode = NuvaInode::new(7, InodeMode::Regular);

        // Default no flags
        let flags = inode.flags.load(core::sync::atomic::Ordering::Relaxed);
        assert_eq!(flags, 0);

        // Set flag
        inode.flags.store(INODE_FLAG_SYNC, core::sync::atomic::Ordering::Relaxed);
        assert_eq!(inode.flags.load(core::sync::atomic::Ordering::Relaxed), INODE_FLAG_SYNC);
    }
}
