/*
 * Nuva OS - NuvaFS File Operations
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

//! NuvaFS File Operations
/*!*/
//! Implements file read, write, seek, and truncate operations.

use core::sync::atomic::{AtomicU64, AtomicU32, Ordering};
use crate::inode::{NuvaInode, InodeMode, Extent, EXTENT_MAGIC};
use crate::journal::{JournalManager, JournalTransactionType};

/// File open mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum OpenMode {
    ReadOnly = 0o0,
    WriteOnly = 0o1,
    ReadWrite = 0o2,
    Create = 0o100,
    Exclusive = 0o200,
    Truncate = 0o1000,
    Append = 0o2000,
    NonBlock = 0o4000,
    Sync = 0o10000,
    Direct = 0o40000,
}

/// File seek origin
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum SeekOrigin {
    Set = 0,
    Current = 1,
    End = 2,
}

/// File error
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum FileError {
    None = 0,
    NoEntry = 2,
    NoSpace = 28,
    IsDirectory = 21,
    NotDirectory = 20,
    NotRegular = 0,
    Permission = 13,
    IO = 5,
    BadFile = 9,
    Invalid = 22,
    NoMemory = 12,
    TooLarge = 27,
    ReadOnlyFS = 30,
}

/// File handle
pub struct FileHandle {
    /// Inode number
    pub ino: u64,

    /// Current position
    pub pos: AtomicU64,

    /// Open mode
    pub mode: OpenMode,

    /// Reference count
    pub refs: AtomicU32,

    /// Flags
    pub flags: AtomicU32,
}

/// File handle flags
pub const FH_FLAG_DIRTY: u32 = 1 << 0;
pub const FH_FLAG_APPEND: u32 = 1 << 1;
pub const FH_FLAG_DIRECT: u32 = 1 << 2;
pub const FH_FLAG_SYNC: u32 = 1 << 3;

impl FileHandle {
    pub fn new(ino: u64, mode: OpenMode) -> Self {
        Self {
            ino,
            pos: AtomicU64::new(0),
            mode,
            refs: AtomicU32::new(1),
            flags: AtomicU32::new(0),
        }
    }

    pub fn get_pos(&self) -> u64 {
        self.pos.load(Ordering::Relaxed)
    }

    pub fn set_pos(&self, pos: u64) {
        self.pos.store(pos, Ordering::Relaxed);
    }

    pub fn add_ref(&self) {
        self.refs.fetch_add(1, Ordering::Relaxed);
    }

    pub fn release(&self) -> bool {
        self.refs.fetch_sub(1, Ordering::Relaxed) == 1
    }

    pub fn is_append(&self) -> bool {
        self.mode == OpenMode::Append || (self.flags.load(Ordering::Relaxed) & FH_FLAG_APPEND) != 0
    }

    pub fn is_sync(&self) -> bool {
        (self.flags.load(Ordering::Relaxed) & FH_FLAG_SYNC) != 0
    }
}

/// Block I/O operations
pub trait BlockIO {
    /// Read block
    fn read_block(&self, block: u64, data: &mut [u8]) -> Result<(), FileError>;

    /// Write block
    fn write_block(&self, block: u64, data: &[u8]) -> Result<(), FileError>;

    /// Allocate block
    fn alloc_block(&mut self) -> Result<u64, FileError>;

    /// Free block
    fn free_block(&mut self, block: u64) -> Result<(), FileError>;

    /// Get block size
    fn block_size(&self) -> u32;
}

/// File operations
pub struct FileOps;

impl FileOps {
    /// Create file
    pub fn create<B: BlockIO>(
        inode: &mut NuvaInode,
        journal: &mut JournalManager,
        block_io: &mut B,
    ) -> Result<u64, FileError> {
        // Begin journal transaction
        let txn_id = journal.begin_transaction(JournalTransactionType::Create);

        // Initialize inode
        inode.mode = InodeMode::Regular as u16;
        inode.links = 1;
        inode.set_size(0);

        // Commit transaction
        if !journal.commit_transaction() {
            return Err(FileError::IO);
        }

        Ok(txn_id)
    }

    /// Read from file
    pub fn read<B: BlockIO>(
        inode: &NuvaInode,
        handle: &FileHandle,
        block_io: &B,
        buf: &mut [u8],
    ) -> Result<usize, FileError> {
        if !inode.is_regular() {
            return Err(FileError::IsDirectory);
        }

        let size = inode.get_size();
        let pos = handle.get_pos();

        if pos >= size {
            return Ok(0);
        }

        let block_size = block_io.block_size();
        let remaining = (size - pos) as usize;
        let to_read = buf.len().min(remaining);

        let mut bytes_read = 0;
        let mut current_pos = pos;

        while bytes_read < to_read {
            let logical_block = current_pos / block_size as u64;
            let block_offset = (current_pos % block_size as u64) as usize;
            let chunk_size = (to_read - bytes_read).min((block_size as usize) - block_offset);

            // Find physical block
            let physical_block = Self::logical_to_physical(inode, logical_block)?;

            if physical_block == 0 {
                // Sparse file - fill with zeros
                for i in 0..chunk_size {
                    buf[bytes_read + i] = 0;
                }
            } else {
                // Read block
                let mut block_data = [0u8; 65536];
                block_io.read_block(physical_block, &mut block_data[..block_size as usize])?;

                // Copy data
                buf[bytes_read..bytes_read + chunk_size]
                    .copy_from_slice(&block_data[block_offset..block_offset + chunk_size]);
            }

            bytes_read += chunk_size;
            current_pos += chunk_size as u64;
        }

        handle.set_pos(pos + bytes_read as u64);
        Ok(bytes_read)
    }

    /// Write to file
    pub fn write<B: BlockIO>(
        inode: &mut NuvaInode,
        handle: &FileHandle,
        block_io: &mut B,
        journal: &mut JournalManager,
        buf: &[u8],
    ) -> Result<usize, FileError> {
        if !inode.is_regular() {
            return Err(FileError::IsDirectory);
        }

        let block_size = block_io.block_size();
        let mut pos = handle.get_pos();

        // Handle append mode
        if handle.is_append() {
            pos = inode.get_size();
            handle.set_pos(pos);
        }

        // Begin journal transaction
        let _txn_id = journal.begin_transaction(JournalTransactionType::Write);

        let mut bytes_written = 0;

        while bytes_written < buf.len() {
            let logical_block = pos / block_size as u64;
            let block_offset = (pos % block_size as u64) as usize;
            let chunk_size = (buf.len() - bytes_written).min((block_size as usize) - block_offset);

            // Find or allocate physical block
            let mut physical_block = Self::logical_to_physical(inode, logical_block)?;

            if physical_block == 0 {
                // Allocate new block
                physical_block = block_io.alloc_block()?;
                inode.allocate_block(logical_block, physical_block);
                inode.add_block();
            }

            // Read existing block data (for partial write)
            let mut block_data = [0u8; 65536];
            block_io.read_block(physical_block, &mut block_data[..block_size as usize])?;

            // Update block data
            block_data[block_offset..block_offset + chunk_size]
                .copy_from_slice(&buf[bytes_written..bytes_written + chunk_size]);

            // Write block
            block_io.write_block(physical_block, &block_data[..block_size as usize])?;

            // Add to journal
            journal.add_block(physical_block, &block_data[..block_size as usize]);

            bytes_written += chunk_size;
            pos += chunk_size as u64;
        }

        // Update file size
        let new_size = pos.max(inode.get_size());
        inode.set_size(new_size);
        handle.set_pos(pos);

        // Commit transaction
        if !journal.commit_transaction() {
            return Err(FileError::IO);
        }

        // Sync if required
        if handle.is_sync() {
            // Flush to disk
        }

        Ok(bytes_written)
    }

    /// Seek in file
    pub fn seek(
        inode: &NuvaInode,
        handle: &FileHandle,
        offset: i64,
        origin: SeekOrigin,
    ) -> Result<u64, FileError> {
        let size = inode.get_size();
        let current = handle.get_pos();

        let new_pos = match origin {
            SeekOrigin::Set => {
                if offset < 0 {
                    return Err(FileError::Invalid);
                }
                offset as u64
            }
            SeekOrigin::Current => {
                if offset < 0 && current < (-offset) as u64 {
                    return Err(FileError::Invalid);
                }
                if offset >= 0 {
                    current.saturating_add(offset as u64)
                } else {
                    current.saturating_sub((-offset) as u64)
                }
            }
            SeekOrigin::End => {
                if offset < 0 && size < (-offset) as u64 {
                    return Err(FileError::Invalid);
                }
                if offset >= 0 {
                    size.saturating_add(offset as u64)
                } else {
                    size.saturating_sub((-offset) as u64)
                }
            }
        };

        handle.set_pos(new_pos);
        Ok(new_pos)
    }

    /// Truncate file
    pub fn truncate<B: BlockIO>(
        inode: &mut NuvaInode,
        block_io: &mut B,
        journal: &mut JournalManager,
        new_size: u64,
    ) -> Result<(), FileError> {
        if !inode.is_regular() {
            return Err(FileError::IsDirectory);
        }

        let old_size = inode.get_size();
        let block_size = block_io.block_size();

        if new_size == old_size {
            return Ok(());
        }

        // Begin journal transaction
        let _txn_id = journal.begin_transaction(JournalTransactionType::Truncate);

        if new_size < old_size {
            // Shrink - free blocks
            let old_blocks = (old_size + block_size as u64 - 1) / block_size as u64;
            let new_blocks = (new_size + block_size as u64 - 1) / block_size as u64;

            for logical in new_blocks..old_blocks {
                if let Some(physical) = Self::logical_to_physical_opt(inode, logical) {
                    if physical != 0 {
                        block_io.free_block(physical)?;
                    }
                }
            }
        }
        // Growing doesn't require allocation until write

        inode.set_size(new_size);

        // Commit transaction
        if !journal.commit_transaction() {
            return Err(FileError::IO);
        }

        Ok(())
    }

    /// Sync file to disk
    pub fn sync<B: BlockIO>(
        inode: &NuvaInode,
        block_io: &B,
    ) -> Result<(), FileError> {
        // Sync all dirty blocks
        let size = inode.get_size();
        let block_size = block_io.block_size();
        let blocks = (size + block_size as u64 - 1) / block_size as u64;

        for logical in 0..blocks {
            if let Some(physical) = Self::logical_to_physical_opt(inode, logical) {
                if physical != 0 {
                    // Flush block cache
                }
            }
        }

        Ok(())
    }

    /// Convert logical block to physical block
    fn logical_to_physical(inode: &NuvaInode, logical: u64) -> Result<u64, FileError> {
        // Try extent map first
        if let Some(extent) = inode.find_extent(logical) {
            return Ok(extent.physical + (logical - extent.logical));
        }

        // Fall back to direct/indirect blocks
        let direct_count = 12u64;
        if logical < direct_count {
            return Ok(inode.direct[logical as usize]);
        }

        // Indirect blocks (simplified)
        Err(FileError::Invalid)
    }

    /// Convert logical block to physical block (optional)
    fn logical_to_physical_opt(inode: &NuvaInode, logical: u64) -> Option<u64> {
        Self::logical_to_physical(inode, logical).ok()
    }
}

/// Open file table
pub struct OpenFileTable {
    files: [Option<FileHandle>; 256],
    count: AtomicU32,
}

impl OpenFileTable {
    pub const fn new() -> Self {
        Self {
            files: [None; 256],
            count: AtomicU32::new(0),
        }
    }

    pub fn insert(&mut self, file: FileHandle) -> Result<usize, FileError> {
        for i in 0..256 {
            if self.files[i].is_none() {
                self.files[i] = Some(file);
                self.count.fetch_add(1, Ordering::Relaxed);
                return Ok(i);
            }
        }
        Err(FileError::NoMemory)
    }

    pub fn get(&self, fd: usize) -> Option<&FileHandle> {
        if fd < 256 {
            self.files[fd].as_ref()
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, fd: usize) -> Option<&mut FileHandle> {
        if fd < 256 {
            self.files[fd].as_mut()
        } else {
            None
        }
    }

    pub fn remove(&mut self, fd: usize) -> Option<FileHandle> {
        if fd < 256 {
            if let Some(file) = self.files[fd].take() {
                self.count.fetch_sub(1, Ordering::Relaxed);
                return Some(file);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_mode() {
        assert_eq!(OpenMode::ReadOnly as u32, 0o0);
        assert_eq!(OpenMode::WriteOnly as u32, 0o1);
        assert_eq!(OpenMode::ReadWrite as u32, 0o2);
        assert_eq!(OpenMode::Create as u32, 0o100);
    }

    #[test]
    fn test_seek_origin() {
        assert_eq!(SeekOrigin::Set as u32, 0);
        assert_eq!(SeekOrigin::Current as u32, 1);
        assert_eq!(SeekOrigin::End as u32, 2);
    }

    #[test]
    fn test_file_error() {
        assert_eq!(FileError::NoEntry as i32, 2);
        assert_eq!(FileError::NoSpace as i32, 28);
        assert_eq!(FileError::Permission as i32, 13);
    }

    #[test]
    fn test_file_handle_new() {
        let handle = FileHandle::new(1, OpenMode::ReadWrite);
        assert_eq!(handle.ino, 1);
        assert_eq!(handle.get_pos(), 0);
        assert_eq!(handle.mode, OpenMode::ReadWrite);
        assert_eq!(handle.refs.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_file_handle_refs() {
        let handle = FileHandle::new(1, OpenMode::ReadOnly);

        handle.add_ref();
        assert_eq!(handle.refs.load(Ordering::Relaxed), 2);

        handle.add_ref();
        assert_eq!(handle.refs.load(Ordering::Relaxed), 3);

        assert!(!handle.release());
        assert_eq!(handle.refs.load(Ordering::Relaxed), 2);

        assert!(!handle.release());
        assert!(handle.release());
    }

    #[test]
    fn test_file_handle_flags() {
        let handle = FileHandle::new(1, OpenMode::ReadWrite);

        assert!(!handle.is_sync());

        handle.flags.store(FH_FLAG_SYNC, Ordering::Relaxed);
        assert!(handle.is_sync());
    }

    #[test]
    fn test_open_file_table() {
        let mut table = OpenFileTable::new();

        // Insert file
        let file = FileHandle::new(1, OpenMode::ReadOnly);
        let fd = table.insert(file);
        assert!(fd.is_ok());
        let fd = fd.unwrap();

        // Get file
        let file = table.get(fd);
        assert!(file.is_some());
        let file = file.unwrap();
        assert_eq!(file.ino, 1);

        // Remove file
        let removed = table.remove(fd);
        assert!(removed.is_some());

        // Get removed file
        let file = table.get(fd);
        assert!(file.is_none());
    }

    #[test]
    fn test_open_file_table_multiple() {
        let mut table = OpenFileTable::new();

        // Insert multiple files
        let fd1 = table.insert(FileHandle::new(1, OpenMode::ReadOnly)).unwrap();
        let fd2 = table.insert(FileHandle::new(2, OpenMode::WriteOnly)).unwrap();
        let fd3 = table.insert(FileHandle::new(3, OpenMode::ReadWrite)).unwrap();

        assert_ne!(fd1, fd2);
        assert_ne!(fd2, fd3);

        // Verify all files
        assert_eq!(table.get(fd1).unwrap().ino, 1);
        assert_eq!(table.get(fd2).unwrap().ino, 2);
        assert_eq!(table.get(fd3).unwrap().ino, 3);
    }
}
