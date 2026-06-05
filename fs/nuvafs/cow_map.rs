/*
 * Nuva OS - NuvaFS COW Map Engine
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

//! NuvaFS COW Map Engine
//! BTreeMap-based copy-on-write mapping with O(log n) lookup,
//! replacing the fixed-size array CowTable for scalable snapshots.

use alloc::collections::BTreeMap;
use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// COW mapping entry stored in the BTreeMap value.
#[derive(Debug, Clone, Copy)]
pub struct CowMapEntry {
    /// Snapshot block number (the copied-to block)
    pub snapshot_block: u64,
    /// Generation number for rollback invalidation
    pub generation: u32,
    /// Entry flags
    pub flags: u32,
}

/// COW entry flags
pub const COW_MAP_FLAG_VALID: u32 = 1 << 0;
pub const COW_MAP_FLAG_DELETED: u32 = 1 << 1;

impl CowMapEntry {
    /// Create a new COW map entry
    pub const fn new(snapshot_block: u64, generation: u32) -> Self {
        Self {
            snapshot_block,
            generation,
            flags: COW_MAP_FLAG_VALID,
        }
    }

    /// Check if the entry is valid
    pub fn is_valid(&self) -> bool {
        (self.flags & COW_MAP_FLAG_VALID) != 0
    }

    /// Check if the entry is marked deleted
    pub fn is_deleted(&self) -> bool {
        (self.flags & COW_MAP_FLAG_DELETED) != 0
    }
}

/// BTreeMap-based COW mapping engine.
///
/// Provides O(log n) lookup by original block number, replacing the
/// O(n) linear scan in the fixed-size `CowTable`.
pub struct CowMapEngine {
    /// BTreeMap from original block -> CowMapEntry
    map: BTreeMap<u64, CowMapEntry>,
    /// Current generation (incremented on rollback)
    generation: AtomicU32,
    /// Total number of entries (including deleted)
    total_entries: AtomicU64,
}

impl CowMapEngine {
    /// Create a new empty COW map engine
    pub fn new() -> Self {
        Self {
            map: BTreeMap::new(),
            generation: AtomicU32::new(1),
            total_entries: AtomicU64::new(0),
        }
    }

    /// Lookup snapshot block for an original block.
    /// Returns None if no valid mapping exists at the current generation.
    pub fn lookup(&self, original: u64) -> Option<u64> {
        let gen = self.generation.load(Ordering::Relaxed);
        self.map.get(&original).and_then(|entry| {
            if entry.is_valid() && !entry.is_deleted() && entry.generation <= gen {
                Some(entry.snapshot_block)
            } else {
                None
            }
        })
    }

    /// Insert a COW mapping: original -> snapshot block.
    /// Returns false if the key already exists with a valid entry at
    /// the current generation (use `update` to overwrite).
    pub fn insert(&mut self, original: u64, snapshot_block: u64) -> bool {
        let gen = self.generation.load(Ordering::Relaxed);
        if let Some(existing) = self.map.get(&original) {
            if existing.is_valid() && !existing.is_deleted() && existing.generation <= gen {
                return false; // already mapped
            }
        }
        self.map.insert(original, CowMapEntry::new(snapshot_block, gen));
        self.total_entries.fetch_add(1, Ordering::Relaxed);
        true
    }

    /// Update an existing COW mapping, or insert if absent.
    pub fn update(&mut self, original: u64, snapshot_block: u64) {
        let gen = self.generation.load(Ordering::Relaxed);
        self.map.insert(original, CowMapEntry::new(snapshot_block, gen));
    }

    /// Remove (soft-delete) a COW mapping by marking it deleted.
    /// Returns true if a valid entry was found and marked.
    pub fn remove(&mut self, original: u64) -> bool {
        if let Some(entry) = self.map.get_mut(&original) {
            if entry.is_valid() && !entry.is_deleted() {
                entry.flags |= COW_MAP_FLAG_DELETED;
                return true;
            }
        }
        false
    }

    /// Hard-remove an entry from the map entirely.
    /// Returns true if the entry existed.
    pub fn hard_remove(&mut self, original: u64) -> bool {
        if self.map.remove(&original).is_some() {
            self.total_entries.fetch_sub(1, Ordering::Relaxed);
            true
        } else {
            false
        }
    }

    /// Increment generation (used during rollback to invalidate newer entries).
    pub fn increment_generation(&self) {
        self.generation.fetch_add(1, Ordering::Relaxed);
    }

    /// Get current generation
    pub fn generation(&self) -> u32 {
        self.generation.load(Ordering::Relaxed)
    }

    /// Get number of live (valid, non-deleted) entries
    pub fn live_count(&self) -> u64 {
        let gen = self.generation.load(Ordering::Relaxed);
        let mut count: u64 = 0;
        for (_, entry) in self.map.iter() {
            if entry.is_valid() && !entry.is_deleted() && entry.generation <= gen {
                count += 1;
            }
        }
        count
    }

    /// Get total entries including deleted
    pub fn total_count(&self) -> u64 {
        self.total_entries.load(Ordering::Relaxed)
    }

    /// Clear all entries and reset generation
    pub fn clear(&mut self) {
        self.map.clear();
        self.total_entries.store(0, Ordering::Relaxed);
        self.generation.store(1, Ordering::Relaxed);
    }
}

impl Default for CowMapEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cow_map_entry() {
        let entry = CowMapEntry::new(200, 1);
        assert_eq!(entry.snapshot_block, 200);
        assert!(entry.is_valid());
        assert!(!entry.is_deleted());
    }

    #[test]
    fn test_cow_map_insert_lookup() {
        let mut engine = CowMapEngine::new();
        assert!(engine.insert(100, 200));
        assert_eq!(engine.lookup(100), Some(200));
        assert_eq!(engine.lookup(999), None);
    }

    #[test]
    fn test_cow_map_duplicate_insert() {
        let mut engine = CowMapEngine::new();
        assert!(engine.insert(100, 200));
        assert!(!engine.insert(100, 300)); // duplicate blocked
        assert_eq!(engine.lookup(100), Some(200));
    }

    #[test]
    fn test_cow_map_update() {
        let mut engine = CowMapEngine::new();
        engine.insert(100, 200);
        engine.update(100, 300);
        assert_eq!(engine.lookup(100), Some(300));
    }

    #[test]
    fn test_cow_map_remove() {
        let mut engine = CowMapEngine::new();
        engine.insert(100, 200);
        assert!(engine.remove(100));
        assert_eq!(engine.lookup(100), None); // deleted entries not returned
    }

    #[test]
    fn test_cow_map_generation() {
        let mut engine = CowMapEngine::new();
        assert!(engine.insert(100, 200));
        engine.increment_generation();
        // Entry at gen 1 should still be visible at gen 2
        assert_eq!(engine.lookup(100), Some(200));
        // New insert at gen 2
        assert!(engine.insert(200, 400));
    }

    #[test]
    fn test_cow_map_live_count() {
        let mut engine = CowMapEngine::new();
        engine.insert(100, 200);
        engine.insert(200, 400);
        assert_eq!(engine.live_count(), 2);
        engine.remove(100);
        assert_eq!(engine.live_count(), 1);
    }

    #[test]
    fn test_cow_map_clear() {
        let mut engine = CowMapEngine::new();
        engine.insert(100, 200);
        engine.clear();
        assert_eq!(engine.lookup(100), None);
        assert_eq!(engine.generation(), 1);
    }
}
