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

//! NuvaFS directory implementation

use core::sync::atomic::{AtomicU32, Ordering};

/// directory entry type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DirEntryType {
    Unknown = 0,
    Regular = 1,
    Directory = 2,
    Symlink = 3,
    FIFO = 4,
    Socket = 5,
    BlockDev = 6,
    CharDev = 7,
}

/// directory entry
#[derive(Debug, Clone)]
#[repr(C, packed)]
pub struct DirEntry {
    /// Inode number
    pub ino: u64,
    
    /// directory entry type
    pub entry_type: u8,
    
    /// name length
    pub name_len: u8,
    
    /// record length
    pub rec_len: u16,
    
    /// file name (variable length)
    pub name: [u8; 256],
}

impl DirEntry {
    pub fn new(ino: u64, name: &[u8], entry_type: DirEntryType) -> Self {
        let mut entry = Self {
            ino,
            entry_type: entry_type as u8,
            name_len: name.len() as u8,
            rec_len: 0,
            name: [0; 256],
        };
        
        let len = name.len().min(256);
        entry.name[..len].copy_from_slice(&name[..len]);
        
        // Calculate record length (aligned to 8 bytes)
        let base_len = 8 + len;
        entry.rec_len = ((base_len + 7) / 8 * 8) as u16;
        
        entry
    }

    pub fn name(&self) -> &[u8] {
        &self.name[..self.name_len as usize]
    }

    pub fn is_deleted(&self) -> bool {
        self.ino == 0
    }
}

/// directory hash function
pub fn dir_hash(name: &[u8]) -> u32 {
    let mut hash: u32 = 0;
    for &b in name {
        hash = hash.wrapping_mul(31).wrapping_add(b as u32);
    }
    hash
}

/// directory index entry
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct DirIndex {
    pub hash: u32,
    pub ino: u64,
    pub block: u64,
}

/// directory index header
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct DirIndexHeader {
    pub magic: u32,
    pub count: u32,
    pub block_size: u32,
}

pub const DIR_INDEX_MAGIC: u32 = 0x4449_5858; // "DIXX"

/// directory iterator
pub struct DirIterator<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> DirIterator<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }
}

impl<'a> Iterator for DirIterator<'a> {
    type Item = DirEntry;

    fn next(&mut self) -> Option<Self::Item> {
        while self.pos + 8 <= self.data.len() {
            let ino = u64::from_le_bytes([
                self.data[self.pos],
                self.data[self.pos + 1],
                self.data[self.pos + 2],
                self.data[self.pos + 3],
                self.data[self.pos + 4],
                self.data[self.pos + 5],
                self.data[self.pos + 6],
                self.data[self.pos + 7],
            ]);

            if ino == 0 {
                // Skip deleted entry
                let rec_len = u16::from_le_bytes([
                    self.data[self.pos + 4],
                    self.data[self.pos + 5],
                ]);
                self.pos += rec_len as usize;
                continue;
            }

            let entry_type = self.data[self.pos + 8];
            let name_len = self.data[self.pos + 9] as usize;
            let rec_len = u16::from_le_bytes([
                self.data[self.pos + 10],
                self.data[self.pos + 11],
            ]) as usize;

            if self.pos + rec_len > self.data.len() {
                break;
            }

            let mut name = [0u8; 256];
            name[..name_len].copy_from_slice(&self.data[self.pos + 12..self.pos + 12 + name_len]);

            let entry = DirEntry {
                ino,
                entry_type,
                name_len: name_len as u8,
                rec_len: rec_len as u16,
                name,
            };

            self.pos += rec_len;
            return Some(entry);
        }
        None
    }
}

/// directory operations
pub struct DirOps;

impl DirOps {
    /// find directory entry
    pub fn lookup(data: &[u8], name: &[u8]) -> Option<u64> {
        for entry in DirIterator::new(data) {
            if entry.name() == name {
                return Some(entry.ino);
            }
        }
        None
    }

    /// create directory entry
    pub fn create(data: &mut [u8], ino: u64, name: &[u8], entry_type: DirEntryType) -> bool {
        let entry = DirEntry::new(ino, name, entry_type);
        let rec_len = entry.rec_len as usize;

        // Find free space
        let mut pos = 0;
        while pos + rec_len <= data.len() {
            let existing_rec_len = u16::from_le_bytes([
                data[pos + 10],
                data[pos + 11],
            ]) as usize;

            if existing_rec_len == 0 {
                // Empty slot
                // SAFETY: unsafe block required for low-level memory or hardware access
                let entry_bytes = unsafe {
                    core::slice::from_raw_parts(
                        &entry as *const DirEntry as *const u8,
                        rec_len
                    )
                };
                data[pos..pos + rec_len].copy_from_slice(entry_bytes);
                return true;
            }

            pos += existing_rec_len;
        }

        false
    }

    /// delete directory entry
    pub fn remove(data: &mut [u8], name: &[u8]) -> bool {
        let mut pos = 0;
        
        while pos + 8 <= data.len() {
            let rec_len = u16::from_le_bytes([
                data[pos + 10],
                data[pos + 11],
            ]) as usize;

            if pos + rec_len > data.len() {
                break;
            }

            let name_len = data[pos + 9] as usize;
            if name_len == name.len() 
                && data[pos + 12..pos + 12 + name_len] == *name {
                // Mark as deleted
                data[pos..pos + 8].copy_from_slice(&[0; 8]);
                return true;
            }

            pos += rec_len;
        }

        false
    }

    /// count directory entries
    pub fn count(data: &[u8]) -> u32 {
        let mut count = 0;
        for _ in DirIterator::new(data) {
            count += 1;
        }
        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dir_entry_new() {
        let entry = DirEntry::new(1, b"test.txt", DirEntryType::Regular);
        assert_eq!(entry.ino, 1);
        assert_eq!(entry.entry_type, DirEntryType::Regular as u8);
        assert_eq!(entry.name_len, 8);
        assert_eq!(entry.name(), b"test.txt");
        assert!(!entry.is_deleted());
    }

    #[test]
    fn test_dir_entry_deleted() {
        let mut entry = DirEntry::new(1, b"test", DirEntryType::Regular);
        entry.ino = 0;
        assert!(entry.is_deleted());
    }

    #[test]
    fn test_dir_hash() {
        let hash1 = dir_hash(b"test");
        let hash2 = dir_hash(b"test");
        let hash3 = dir_hash(b"other");

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_dir_entry_type() {
        assert_eq!(DirEntryType::Regular as u8, 1);
        assert_eq!(DirEntryType::Directory as u8, 2);
        assert_eq!(DirEntryType::Symlink as u8, 3);
    }

    #[test]
    fn test_dir_iterator() {
        let mut data = [0u8; 4096];

        // Create two directory entries
        let entry1 = DirEntry::new(1, b"file1.txt", DirEntryType::Regular);
        let entry2 = DirEntry::new(2, b"file2.txt", DirEntryType::Regular);

        // Manually write directory entries
        let mut pos = 0;
        let rec_len1 = entry1.rec_len as usize;
        // SAFETY: unsafe block required for low-level memory or hardware access
        let bytes1 = unsafe {
            core::slice::from_raw_parts(&entry1 as *const DirEntry as *const u8, rec_len1)
        };
        data[pos..pos + rec_len1].copy_from_slice(bytes1);
        pos += rec_len1;

        let rec_len2 = entry2.rec_len as usize;
        // SAFETY: unsafe block required for low-level memory or hardware access
        let bytes2 = unsafe {
            core::slice::from_raw_parts(&entry2 as *const DirEntry as *const u8, rec_len2)
        };
        data[pos..pos + rec_len2].copy_from_slice(bytes2);

        // Iterate
        let mut iter = DirIterator::new(&data);
        let first = iter.next();
        assert!(first.is_some());
        let first = first.unwrap();
        assert_eq!(first.ino, 1);

        let second = iter.next();
        assert!(second.is_some());
        let second = second.unwrap();
        assert_eq!(second.ino, 2);

        assert!(iter.next().is_none());
    }

    #[test]
    fn test_dir_ops_count() {
        let mut data = [0u8; 4096];

        // Create directory entry
        let entry = DirEntry::new(1, b"test", DirEntryType::Regular);
        let rec_len = entry.rec_len as usize;
        // SAFETY: unsafe block required for low-level memory or hardware access
        let bytes = unsafe {
            core::slice::from_raw_parts(&entry as *const DirEntry as *const u8, rec_len)
        };
        data[..rec_len].copy_from_slice(bytes);

        let count = DirOps::count(&data);
        assert_eq!(count, 1);
    }
}
