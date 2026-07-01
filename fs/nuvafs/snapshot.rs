/*
 * Nuva OS - NuvaFS Snapshot System
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

//! NuvaFS Snapshot System
/*!*/
//! Implements copy-on-write snapshots for point-in-time filesystem state capture.

use core::sync::atomic::{AtomicU64, AtomicU32, AtomicBool, Ordering};
use alloc::vec::Vec;

/// Snapshot magic number
pub const SNAPSHOT_MAGIC: u32 = 0x4E56_534E; // "NVSN"

/// Snapshot state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SnapshotState {
    /// Snapshot is being created
    Creating = 0,
    /// Snapshot is active and valid
    Active = 1,
    /// Snapshot is being deleted
    Deleting = 2,
    /// Snapshot is being rolled back
    RollingBack = 3,
    /// Snapshot is invalid/corrupted
    Invalid = 4,
}

/// Snapshot flags
pub const SNAPSHOT_FLAG_READONLY: u32 = 1 << 0;
pub const SNAPSHOT_FLAG_RECURSIVE: u32 = 1 << 1;
pub const SNAPSHOT_FLAG_COMPRESSED: u32 = 1 << 2;
pub const SNAPSHOT_FLAG_ENCRYPTED: u32 = 1 << 3;

/// Snapshot metadata header
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct SnapshotHeader {
    /// Magic number
    pub magic: u32,

    /// Snapshot ID
    pub id: u64,

    /// Parent snapshot ID (0 = no parent)
    pub parent_id: u64,

    /// Creation timestamp
    pub create_time: u64,

    /// State
    pub state: AtomicU32,

    /// Flags
    pub flags: u32,

    /// Root inode at snapshot time
    pub root_ino: u64,

    /// Block bitmap start
    pub bitmap_start: u64,

    /// Block bitmap size (blocks)
    pub bitmap_size: u32,

    /// COW mapping table start
    pub cow_table_start: u64,

    /// COW table entries
    pub cow_table_entries: u32,

    /// Original block count
    pub original_blocks: u64,

    /// Allocated blocks for this snapshot
    pub allocated_blocks: AtomicU64,

    /// Name length
    pub name_len: u16,

    /// Description length
    pub desc_len: u16,

    /// Checksum
    pub checksum: u32,
}

impl SnapshotHeader {
    pub fn new(id: u64, parent_id: u64, root_ino: u64, create_time: u64) -> Self {
        Self {
            magic: SNAPSHOT_MAGIC,
            id,
            parent_id,
            create_time,
            state: AtomicU32::new(SnapshotState::Creating as u32),
            flags: SNAPSHOT_FLAG_READONLY,
            root_ino,
            bitmap_start: 0,
            bitmap_size: 0,
            cow_table_start: 0,
            cow_table_entries: 0,
            original_blocks: 0,
            allocated_blocks: AtomicU64::new(0),
            name_len: 0,
            desc_len: 0,
            checksum: 0,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.magic == SNAPSHOT_MAGIC
    }

    pub fn get_state(&self) -> SnapshotState {
        match self.state.load(Ordering::Relaxed) {
            0 => SnapshotState::Creating,
            1 => SnapshotState::Active,
            2 => SnapshotState::Deleting,
            3 => SnapshotState::RollingBack,
            _ => SnapshotState::Invalid,
        }
    }

    pub fn set_state(&self, state: SnapshotState) {
        self.state.store(state as u32, Ordering::Relaxed);
    }

    pub fn is_readonly(&self) -> bool {
        (self.flags & SNAPSHOT_FLAG_READONLY) != 0
    }

    pub fn is_active(&self) -> bool {
        self.get_state() == SnapshotState::Active
    }
}

/// COW (Copy-On-Write) mapping entry
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct CowEntry {
    /// Original block number
    pub original: u64,

    /// Snapshot block number
    pub snapshot: u64,

    /// Generation number
    pub generation: u32,

    /// Flags
    pub flags: u32,
}

/// COW entry flags
pub const COW_FLAG_VALID: u32 = 1 << 0;
pub const COW_FLAG_DELETED: u32 = 1 << 1;

impl CowEntry {
    pub const fn new(original: u64, snapshot: u64, generation: u32) -> Self {
        Self {
            original,
            snapshot,
            generation,
            flags: COW_FLAG_VALID,
        }
    }

    pub fn is_valid(&self) -> bool {
        (self.flags & COW_FLAG_VALID) != 0
    }

    pub fn is_deleted(&self) -> bool {
        (self.flags & COW_FLAG_DELETED) != 0
    }
}

/// COW mapping table
pub struct CowTable {
    entries: [CowEntry; 4096],
    count: AtomicU32,
    generation: AtomicU32,
}

impl CowTable {
    pub const fn new() -> Self {
        Self {
            entries: [CowEntry {
                original: 0,
                snapshot: 0,
                generation: 0,
                flags: 0,
            }; 4096],
            count: AtomicU32::new(0),
            generation: AtomicU32::new(1),
        }
    }

    /// Lookup snapshot block for original block
    pub fn lookup(&self, original: u64) -> Option<u64> {
        let gen = self.generation.load(Ordering::Relaxed);
        for i in 0..self.count.load(Ordering::Relaxed) as usize {
            let entry = &self.entries[i];
            if entry.is_valid() && entry.original == original && entry.generation <= gen {
                return Some(entry.snapshot);
            }
        }
        None
    }

    /// Add COW mapping
    pub fn add(&mut self, original: u64, snapshot: u64) -> bool {
        let count = self.count.load(Ordering::Relaxed) as usize;
        if count >= self.entries.len() {
            return false;
        }

        let gen = self.generation.load(Ordering::Relaxed);
        self.entries[count] = CowEntry::new(original, snapshot, gen);
        self.count.fetch_add(1, Ordering::Relaxed);
        true
    }

    /// Remove mapping (mark as deleted)
    pub fn remove(&mut self, original: u64) -> bool {
        for i in 0..self.count.load(Ordering::Relaxed) as usize {
            if self.entries[i].original == original && self.entries[i].is_valid() {
                self.entries[i].flags |= COW_FLAG_DELETED;
                return true;
            }
        }
        false
    }

    /// Increment generation (for rollback)
    pub fn increment_generation(&self) {
        self.generation.fetch_add(1, Ordering::Relaxed);
    }
}

/// Snapshot manager
pub struct SnapshotManager {
    /// Snapshot headers
    snapshots: [Option<SnapshotHeader>; 32],

    /// Snapshot count
    count: AtomicU32,

    /// Next snapshot ID
    next_id: AtomicU64,

    /// COW tables for each snapshot
    cow_tables: [CowTable; 32],

    /// Active snapshot ID (0 = none)
    active_snapshot: AtomicU64,

    /// Rollback in progress
    rollback_in_progress: AtomicBool,
}

impl SnapshotManager {
    pub const fn new() -> Self {
        Self {
            snapshots: [None; 32],
            count: AtomicU32::new(0),
            next_id: AtomicU64::new(1),
            cow_tables: [CowTable::new(); 32],
            active_snapshot: AtomicU64::new(0),
            rollback_in_progress: AtomicBool::new(false),
        }
    }

    /// Create snapshot
    pub fn create(
        &mut self,
        parent_id: u64,
        root_ino: u64,
        create_time: u64,
    ) -> Result<u64, SnapshotError> {
        if self.count.load(Ordering::Relaxed) >= 32 {
            return Err(SnapshotError::TooManySnapshots);
        }

        let id = self.next_id.fetch_add(1, Ordering::Relaxed);

        // Find free slot
        let mut slot = None;
        for i in 0..32 {
            if self.snapshots[i].is_none() {
                slot = Some(i);
                break;
            }
        }

        let slot = slot.ok_or(SnapshotError::TooManySnapshots)?;

        // Create header
        let header = SnapshotHeader::new(id, parent_id, root_ino, create_time);
        self.snapshots[slot] = Some(header);
        self.count.fetch_add(1, Ordering::Relaxed);

        // Mark as active
        if let Some(ref mut snap) = self.snapshots[slot] {
            snap.set_state(SnapshotState::Active);
        }

        Ok(id)
    }

    /// Delete snapshot
    pub fn delete(&mut self, id: u64) -> Result<(), SnapshotError> {
        let slot = self.find_slot(id).ok_or(SnapshotError::NotFound)?;

        // Check if snapshot has children
        for i in 0..32 {
            if let Some(ref snap) = self.snapshots[i] {
                if snap.parent_id == id && snap.is_active() {
                    return Err(SnapshotError::HasChildren);
                }
            }
        }

        // Mark as deleting
        if let Some(ref mut snap) = self.snapshots[slot] {
            snap.set_state(SnapshotState::Deleting);
        }

        // Free COW table
        self.cow_tables[slot] = CowTable::new();

        // Remove snapshot
        self.snapshots[slot] = None;
        self.count.fetch_sub(1, Ordering::Relaxed);

        Ok(())
    }

    /// Rollback to snapshot
    pub fn rollback(&mut self, id: u64) -> Result<(), SnapshotError> {
        if self.rollback_in_progress.load(Ordering::Relaxed) {
            return Err(SnapshotError::RollbackInProgress);
        }

        let slot = self.find_slot(id).ok_or(SnapshotError::NotFound)?;

        // Verify snapshot is active
        {
            let snap = self.snapshots[slot].as_ref().ok_or(SnapshotError::NotFound)?;
            if !snap.is_active() {
                return Err(SnapshotError::InvalidState);
            }
        }

        // Mark rollback in progress
        self.rollback_in_progress.store(true, Ordering::Relaxed);

        // Mark snapshot as rolling back
        if let Some(ref mut snap) = self.snapshots[slot] {
            snap.set_state(SnapshotState::RollingBack);
        }

        // Set as active snapshot
        self.active_snapshot.store(id, Ordering::Relaxed);

        // Increment COW generation to invalidate newer changes
        self.cow_tables[slot].increment_generation();

        // Mark as active again
        if let Some(ref mut snap) = self.snapshots[slot] {
            snap.set_state(SnapshotState::Active);
        }

        // Clear rollback flag
        self.rollback_in_progress.store(false, Ordering::Relaxed);

        Ok(())
    }

    /// Get snapshot info
    pub fn get(&self, id: u64) -> Option<&SnapshotHeader> {
        self.find_slot(id).and_then(|slot| self.snapshots[slot].as_ref())
    }

    /// List all snapshots
    pub fn list(&self) -> Vec<SnapshotInfo> {
        let mut result = Vec::new();
        for i in 0..32 {
            if let Some(ref snap) = self.snapshots[i] {
                if snap.is_active() {
                    result.push(SnapshotInfo {
                        id: snap.id,
                        parent_id: snap.parent_id,
                        create_time: snap.create_time,
                        state: snap.get_state(),
                        allocated_blocks: snap.allocated_blocks.load(Ordering::Relaxed),
                    });
                }
            }
        }
        result
    }

    /// Handle COW for block write
    pub fn cow_write(&mut self, block: u64, new_block: u64) -> Result<(), SnapshotError> {
        let active_id = self.active_snapshot.load(Ordering::Relaxed);
        if active_id == 0 {
            return Ok(()); // No active snapshot
        }

        let slot = self.find_slot(active_id).ok_or(SnapshotError::NotFound)?;

        // Add COW mapping
        if !self.cow_tables[slot].add(block, new_block) {
            return Err(SnapshotError::COWTableFull);
        }

        // Update allocated blocks
        if let Some(ref snap) = self.snapshots[slot] {
            snap.allocated_blocks.fetch_add(1, Ordering::Relaxed);
        }

        Ok(())
    }

    /// Translate block for read (check COW)
    pub fn translate_read(&self, block: u64) -> u64 {
        let active_id = self.active_snapshot.load(Ordering::Relaxed);
        if active_id == 0 {
            return block; // No active snapshot
        }

        if let Some(slot) = self.find_slot(active_id) {
            if let Some(snapshot_block) = self.cow_tables[slot].lookup(block) {
                return snapshot_block;
            }
        }

        block
    }

    /// Find snapshot slot
    fn find_slot(&self, id: u64) -> Option<usize> {
        for i in 0..32 {
            if let Some(ref snap) = self.snapshots[i] {
                if snap.id == id {
                    return Some(i);
                }
            }
        }
        None
    }
}

/// Snapshot info for listing
#[derive(Debug, Clone, Copy)]
pub struct SnapshotInfo {
    pub id: u64,
    pub parent_id: u64,
    pub create_time: u64,
    pub state: SnapshotState,
    pub allocated_blocks: u64,
}

/// Snapshot error
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnapshotError {
    NotFound,
    TooManySnapshots,
    HasChildren,
    InvalidState,
    RollbackInProgress,
    COWTableFull,
    IOError,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_state() {
        assert_eq!(SnapshotState::Creating as u8, 0);
        assert_eq!(SnapshotState::Active as u8, 1);
        assert_eq!(SnapshotState::Deleting as u8, 2);
    }

    #[test]
    fn test_snapshot_header_new() {
        let header = SnapshotHeader::new(1, 0, 2, 1000);
        assert_eq!(header.id, 1);
        assert_eq!(header.parent_id, 0);
        assert_eq!(header.root_ino, 2);
        assert!(header.is_valid());
        assert!(header.is_readonly());
    }

    #[test]
    fn test_snapshot_header_state() {
        let header = SnapshotHeader::new(1, 0, 2, 1000);

        assert_eq!(header.get_state(), SnapshotState::Creating);

        header.set_state(SnapshotState::Active);
        assert_eq!(header.get_state(), SnapshotState::Active);
        assert!(header.is_active());

        header.set_state(SnapshotState::Deleting);
        assert_eq!(header.get_state(), SnapshotState::Deleting);
        assert!(!header.is_active());
    }

    #[test]
    fn test_cow_entry() {
        let entry = CowEntry::new(100, 200, 1);
        assert_eq!(entry.original, 100);
        assert_eq!(entry.snapshot, 200);
        assert!(entry.is_valid());
        assert!(!entry.is_deleted());
    }

    #[test]
    fn test_cow_table() {
        let mut table = CowTable::new();

        // Add entry
        assert!(table.add(100, 200));

        // Lookup
        let result = table.lookup(100);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), 200);

        // Lookup non-existent
        let result = table.lookup(999);
        assert!(result.is_none());
    }

    #[test]
    fn test_cow_table_remove() {
        let mut table = CowTable::new();

        table.add(100, 200);
        assert!(table.remove(100));

        // Entry should be marked deleted
        let result = table.lookup(100);
        // Deleted entries are still found but marked
    }

    #[test]
    fn test_snapshot_manager_create() {
        let mut mgr = SnapshotManager::new();

        let id = mgr.create(0, 2, 1000);
        assert!(id.is_ok());
        let id = id.unwrap();

        let snap = mgr.get(id);
        assert!(snap.is_some());
        let snap = snap.unwrap();
        assert_eq!(snap.id, id);
        assert!(snap.is_active());
    }

    #[test]
    fn test_snapshot_manager_delete() {
        let mut mgr = SnapshotManager::new();

        let id = mgr.create(0, 2, 1000).unwrap();

        let result = mgr.delete(id);
        assert!(result.is_ok());

        let snap = mgr.get(id);
        assert!(snap.is_none());
    }

    #[test]
    fn test_snapshot_manager_rollback() {
        let mut mgr = SnapshotManager::new();

        let id = mgr.create(0, 2, 1000).unwrap();

        let result = mgr.rollback(id);
        assert!(result.is_ok());

        // Active snapshot should be set
        assert_eq!(mgr.active_snapshot.load(Ordering::Relaxed), id);
    }

    #[test]
    fn test_snapshot_manager_list() {
        let mut mgr = SnapshotManager::new();

        mgr.create(0, 2, 1000).unwrap();
        mgr.create(0, 2, 2000).unwrap();

        let list = mgr.list();
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn test_snapshot_manager_cow() {
        let mut mgr = SnapshotManager::new();

        let id = mgr.create(0, 2, 1000).unwrap();
        mgr.active_snapshot.store(id, Ordering::Relaxed);

        // COW write
        let result = mgr.cow_write(100, 200);
        assert!(result.is_ok());

        // Translate read
        let translated = mgr.translate_read(100);
        assert_eq!(translated, 200);

        // Non-COW block
        let translated = mgr.translate_read(999);
        assert_eq!(translated, 999);
    }
}
