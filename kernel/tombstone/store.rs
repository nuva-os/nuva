/*
 * Nuva OS - Kernel - Tombstone - Store
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

//! Tombstone persistent storage management.
/*!*/
//! Manages tombstone file writing with atomic rename, FIFO auto-pruning,
//! in-memory ring buffer fallback when the filesystem is unavailable,
//! and an in-memory index for fast queries.

use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use super::config::TombstoneStoreConfig;
use super::record::{
    ArchId, CrashReason, TombstoneError, TombstoneRecord, TOMBSTONE_MAX_COUNT,
    TOMBSTONE_MAX_FILE_SIZE,
};
use super::stats::TombstoneStats;

// ---------------------------------------------------------------------------
// TombstoneIndexEntry
// ---------------------------------------------------------------------------

/** In-memory index entry for fast tombstone queries */
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct TombstoneIndexEntry {
    /** File number (0-99, maps to tombstone_XX.pb) */
    pub file_number: u8,
    /** Crash timestamp (nanoseconds since boot) */
    pub timestamp: u64,
    /** Process ID */
    pub pid: u32,
    /** Thread ID */
    pub tid: u32,
    /** Crash reason */
    pub crash_reason: CrashReason,
    /** Architecture identifier */
    pub arch_id: ArchId,
}

impl TombstoneIndexEntry {
    /** Create a TombstoneIndexEntry from a TombstoneRecord */
    pub fn from_record(file_number: u8, record: &TombstoneRecord) -> Self {
        TombstoneIndexEntry {
            file_number,
            timestamp: record.timestamp,
            pid: record.pid,
            tid: record.tid,
            crash_reason: record.crash_reason,
            arch_id: record.arch_id,
        }
    }
}

// ---------------------------------------------------------------------------
// MemoryCache
// ---------------------------------------------------------------------------

/** Ring buffer capacity for in-memory tombstone cache */
const MEMORY_CACHE_SIZE: usize = 4;

/** In-memory ring buffer for tombstone records when FS is unavailable */
pub struct MemoryCache {
    /** Ring buffer slots */
    buffer: [Option<TombstoneRecord>; MEMORY_CACHE_SIZE],
    /** Next write position */
    write_pos: AtomicU32,
    /** Number of records in the buffer */
    count: AtomicU32,
}

impl MemoryCache {
    /** Create an empty MemoryCache */
    pub const fn new() -> Self {
        MemoryCache {
            buffer: [None, None, None, None],
            write_pos: AtomicU32::new(0),
            count: AtomicU32::new(0),
        }
    }

    /** Push a tombstone record into the ring buffer.
     *  Overwrites the oldest entry if the buffer is full. */
    pub fn push(&self, record: TombstoneRecord) {
        let pos = self.write_pos.fetch_add(1, Ordering::Relaxed) as usize;
        let idx = pos % MEMORY_CACHE_SIZE;
        // SAFETY: idx is always in bounds (0..MEMORY_CACHE_SIZE).
        // We use a single-producer model protected by the manager's lock.
        unsafe {
            let buf_ptr = &self.buffer as *const [Option<TombstoneRecord>; MEMORY_CACHE_SIZE]
                as *mut [Option<TombstoneRecord>; MEMORY_CACHE_SIZE];
            (*buf_ptr)[idx] = Some(record);
        }
        let old_count = self.count.load(Ordering::Relaxed);
        if (old_count as usize) < MEMORY_CACHE_SIZE {
            self.count.fetch_add(1, Ordering::Relaxed);
        }
    }

    /** Pop the oldest record from the cache (FIFO order).
     *  Returns None if the cache is empty. */
    pub fn pop(&self) -> Option<TombstoneRecord> {
        let count = self.count.load(Ordering::Relaxed);
        if count == 0 {
            return None;
        }
        let wp = self.write_pos.load(Ordering::Relaxed) as usize;
        let oldest = if (count as usize) < MEMORY_CACHE_SIZE {
            0
        } else {
            wp % MEMORY_CACHE_SIZE
        };
        // SAFETY: oldest is always in bounds.
        let result = unsafe {
            let buf_ptr = &self.buffer as *const [Option<TombstoneRecord>; MEMORY_CACHE_SIZE]
                as *mut [Option<TombstoneRecord>; MEMORY_CACHE_SIZE];
            (*buf_ptr)[oldest].take()
        };
        if result.is_some() {
            self.count.fetch_sub(1, Ordering::Relaxed);
        }
        result
    }

    /** Return the number of cached records */
    pub fn len(&self) -> u32 {
        self.count.load(Ordering::Relaxed)
    }

    /** Check if the cache is empty */
    pub fn is_empty(&self) -> bool {
        self.count.load(Ordering::Relaxed) == 0
    }
}

// ---------------------------------------------------------------------------
// TombstoneStore
// ---------------------------------------------------------------------------

/** Maximum number of index entries (matches TOMBSTONE_MAX_COUNT) */
const MAX_INDEX_ENTRIES: usize = TOMBSTONE_MAX_COUNT as usize;

/** Persistent tombstone storage manager.
 *  Handles atomic file writes, FIFO auto-pruning, FS degradation,
 *  and in-memory index maintenance. */
pub struct TombstoneStore {
    /** Storage configuration */
    pub config: TombstoneStoreConfig,
    /** In-memory ring buffer for FS-degraded mode */
    pub memory_cache: MemoryCache,
    /** Next file number (0-99, cyclic) */
    file_counter: AtomicU32,
    /** In-memory index for fast queries */
    index: [Option<TombstoneIndexEntry>; MAX_INDEX_ENTRIES],
    /** Number of valid index entries */
    index_count: AtomicU32,
    /** Whether the filesystem is currently available */
    fs_available: AtomicBool,
    /** Set of file numbers currently being written (bitmask) */
    writing_set: AtomicU32,
}

impl TombstoneStore {
    /** Create a TombstoneStore with the given configuration */
    pub fn new(config: TombstoneStoreConfig) -> Self {
        let index = [None; MAX_INDEX_ENTRIES];
        TombstoneStore {
            config,
            memory_cache: MemoryCache::new(),
            file_counter: AtomicU32::new(0),
            index,
            index_count: AtomicU32::new(0),
            fs_available: AtomicBool::new(true),
            writing_set: AtomicU32::new(0),
        }
    }

    /** Write a tombstone record to persistent storage.
     *  On FS failure, falls back to the in-memory cache.
     *  On capacity overflow, auto-prunes the oldest record first. */
    pub fn write(
        &mut self,
        record: &TombstoneRecord,
        stats: &TombstoneStats,
    ) -> Result<(), TombstoneError> {
        let count = self.index_count.load(Ordering::Relaxed);
        if count >= self.config.max_count {
            if self.config.auto_prune_enabled {
                self.prune_oldest(stats);
            } else {
                return Err(TombstoneError::CapacityExceeded);
            }
        }

        let file_num = self.file_counter.fetch_add(1, Ordering::Relaxed) % 100;
        let file_num_u8 = file_num as u8;

        // Mark file as being written
        self.writing_set
            .fetch_or(1u32 << file_num, Ordering::Relaxed);

        if self.fs_available.load(Ordering::Relaxed) {
            match self.write_file(file_num_u8, record) {
                Ok(()) => {
                    self.writing_set
                        .fetch_and(!(1u32 << file_num), Ordering::Relaxed);
                    let entry = TombstoneIndexEntry::from_record(file_num_u8, record);
                    self.add_index(entry);
                    stats.increment_generated();
                    Ok(())
                }
                Err(_) => {
                    self.writing_set
                        .fetch_and(!(1u32 << file_num), Ordering::Relaxed);
                    self.fs_available.store(false, Ordering::Relaxed);
                    self.memory_cache.push(record.clone());
                    stats.increment_mem_cache();
                    stats.increment_generated();
                    log_warn!("FS write failed, tombstone cached in memory");
                    Ok(())
                }
            }
        } else {
            self.writing_set
                .fetch_and(!(1u32 << file_num), Ordering::Relaxed);
            self.memory_cache.push(record.clone());
            stats.increment_mem_cache();
            stats.increment_generated();
            Ok(())
        }
    }

    /** Atomically write a tombstone file: write temp → fsync → rename */
    fn write_file(&self, file_num: u8, record: &TombstoneRecord) -> Result<(), TombstoneError> {
        let mut buf: [u8; TOMBSTONE_MAX_FILE_SIZE as usize] =
            [0u8; TOMBSTONE_MAX_FILE_SIZE as usize];
        let len = record.serialize_into(&mut buf);
        if len == 0 {
            return Err(TombstoneError::SerializeError);
        }
        if len > self.config.max_file_size as usize {
            return Err(TombstoneError::CapacityExceeded);
        }

        // Build file paths
        let dir = self.config.store_dir_str();

        // Attempt to write via VFS
        let vfs = crate::kernel::fs::vfs::vfs_core();

        // Write temp file
        let tmp_name = alloc::format!(".tombstone_{:02}.pb.tmp", file_num);
        let tmp_path = alloc::format!(
            "{}{}",
            core::str::from_utf8(dir).unwrap_or("/data/tombstones/"),
            tmp_name
        );
        match vfs.create(&tmp_path) {
            Ok(_) => {}
            Err(_) => return Err(TombstoneError::IoError),
        }
        match vfs.write(&tmp_path, &buf[..len]) {
            Ok(_) => {}
            Err(_) => {
                let _ = vfs.unlink(&tmp_path);
                return Err(TombstoneError::IoError);
            }
        }

        // Atomic rename
        let final_name = alloc::format!("tombstone_{:02}.pb", file_num);
        let final_path = alloc::format!(
            "{}{}",
            core::str::from_utf8(dir).unwrap_or("/data/tombstones/"),
            final_name
        );
        match vfs.rename(&tmp_path, &final_path) {
            Ok(_) => Ok(()),
            Err(_) => {
                let _ = vfs.unlink(&tmp_path);
                Err(TombstoneError::IoError)
            }
        }
    }

    /** Prune the oldest tombstone record from storage */
    fn prune_oldest(&mut self, stats: &TombstoneStats) {
        let count = self.index_count.load(Ordering::Relaxed);
        if count == 0 {
            return;
        }

        let mut oldest_idx: usize = 0;
        let mut oldest_ts: u64 = u64::MAX;

        for i in 0..(count as usize) {
            if let Some(ref entry) = self.index[i] {
                if entry.timestamp < oldest_ts {
                    oldest_ts = entry.timestamp;
                    oldest_idx = i;
                }
            }
        }

        if let Some(entry) = self.index[oldest_idx].take() {
            let writing = self.writing_set.load(Ordering::Relaxed);
            if (writing >> entry.file_number) & 1 == 0 {
                let dir = self.config.store_dir_str();
                let name = alloc::format!("tombstone_{:02}.pb", entry.file_number);
                let path = alloc::format!(
                    "{}{}",
                    core::str::from_utf8(dir).unwrap_or("/data/tombstones/"),
                    name
                );
                let vfs = crate::kernel::fs::vfs::vfs_core();
                let _ = vfs.unlink(&path);
            }
            stats.decrement_count(1);
            self.compact_index(oldest_idx);
        }
    }

    /** Add an entry to the in-memory index */
    fn add_index(&mut self, entry: TombstoneIndexEntry) {
        let count = self.index_count.load(Ordering::Relaxed) as usize;
        if count < MAX_INDEX_ENTRIES {
            self.index[count] = Some(entry);
            self.index_count.fetch_add(1, Ordering::Relaxed);
        }
    }

    /** Compact the index by shifting entries after a removal */
    fn compact_index(&mut self, removed_idx: usize) {
        let count = self.index_count.load(Ordering::Relaxed) as usize;
        if removed_idx < count - 1 {
            self.index.copy_within(removed_idx + 1..count, removed_idx);
        }
        self.index[count - 1] = None;
        self.index_count.fetch_sub(1, Ordering::Relaxed);
    }

    /** Query the index for entries matching a predicate.
     *  Returns up to limit matching index entries. */
    pub fn query_index<F>(&self, predicate: F, limit: u32) -> alloc::vec::Vec<TombstoneIndexEntry>
    where
        F: Fn(&TombstoneIndexEntry) -> bool,
    {
        let mut results = alloc::vec::Vec::new();
        let count = self.index_count.load(Ordering::Relaxed) as usize;
        for i in 0..count {
            if let Some(ref entry) = self.index[i] {
                if predicate(entry) {
                    results.push(*entry);
                    if results.len() as u32 >= limit {
                        break;
                    }
                }
            }
        }
        results
    }

    /** Read a tombstone record by file number */
    pub fn read_record(&self, file_number: u8) -> Result<TombstoneRecord, TombstoneError> {
        let dir = self.config.store_dir_str();
        let name = alloc::format!("tombstone_{:02}.pb", file_number);
        let path = alloc::format!(
            "{}{}",
            core::str::from_utf8(dir).unwrap_or("/data/tombstones/"),
            name
        );

        let vfs = crate::kernel::fs::vfs::vfs_core();
        let mut buf: [u8; TOMBSTONE_MAX_FILE_SIZE as usize] =
            [0u8; TOMBSTONE_MAX_FILE_SIZE as usize];
        match vfs.read(&path, &mut buf) {
            Ok(len) => TombstoneRecord::deserialize_from(&buf[..len]),
            Err(_) => Err(TombstoneError::NotFound),
        }
    }

    /** Delete a tombstone file by file number.
     *  Returns Err if the file is currently being written. */
    pub fn delete_file(&self, file_number: u8) -> Result<(), TombstoneError> {
        let writing = self.writing_set.load(Ordering::Relaxed);
        if (writing >> file_number) & 1 != 0 {
            return Err(TombstoneError::IoError);
        }
        let dir = self.config.store_dir_str();
        let name = alloc::format!("tombstone_{:02}.pb", file_number);
        let path = alloc::format!(
            "{}{}",
            core::str::from_utf8(dir).unwrap_or("/data/tombstones/"),
            name
        );
        let vfs = crate::kernel::fs::vfs::vfs_core();
        match vfs.unlink(&path) {
            Ok(_) => Ok(()),
            Err(_) => Err(TombstoneError::IoError),
        }
    }

    /** Remove an index entry by file number */
    pub fn remove_index_by_file_number(&mut self, file_number: u8) {
        let count = self.index_count.load(Ordering::Relaxed) as usize;
        for i in 0..count {
            if let Some(ref entry) = self.index[i] {
                if entry.file_number == file_number {
                    self.index[i] = None;
                    self.compact_index(i);
                    return;
                }
            }
        }
    }

    /** Flush all cached records to the filesystem */
    pub fn flush_memory_cache(&mut self, stats: &TombstoneStats) -> u32 {
        if !self.fs_available.load(Ordering::Relaxed) {
            let vfs = crate::kernel::fs::vfs::vfs_core();
            let dir = self.config.store_dir_str();
            let path = core::str::from_utf8(dir).unwrap_or("/data/tombstones/");
            match vfs.mkdir(path) {
                Ok(_) => self.fs_available.store(true, Ordering::Relaxed),
                Err(_) => return 0,
            }
        }

        let mut flushed: u32 = 0;
        loop {
            let record = self.memory_cache.pop();
            match record {
                Some(rec) => {
                    let file_num = self.file_counter.fetch_add(1, Ordering::Relaxed) % 100;
                    match self.write_file(file_num as u8, &rec) {
                        Ok(()) => {
                            let entry = TombstoneIndexEntry::from_record(file_num as u8, &rec);
                            self.add_index(entry);
                            flushed += 1;
                        }
                        Err(_) => {
                            self.memory_cache.push(rec);
                            break;
                        }
                    }
                }
                None => break,
            }
        }

        if flushed > 0 {
            stats.record_flush(flushed);
        }
        flushed
    }

    /** Scan the store directory at startup and rebuild the in-memory index */
    pub fn rebuild_index(&mut self) {
        let dir = self.config.store_dir_str();
        let path = core::str::from_utf8(dir).unwrap_or("/data/tombstones/");
        let vfs = crate::kernel::fs::vfs::vfs_core();

        // Try to create the tombstone directory if it does not exist
        match vfs.mkdir(path) {
            Ok(_) => self.fs_available.store(true, Ordering::Relaxed),
            Err(_) => {
                self.fs_available.store(false, Ordering::Relaxed);
                log_warn!("Tombstone store directory creation failed, operating in degraded mode");
                return;
            }
        }

        // Scan existing tombstone files
        for file_num in 0u8..100 {
            let name = alloc::format!("tombstone_{:02}.pb", file_num);
            let fpath = alloc::format!("{}{}", path, name);
            let mut buf: [u8; TOMBSTONE_MAX_FILE_SIZE as usize] =
                [0u8; TOMBSTONE_MAX_FILE_SIZE as usize];
            match vfs.read(&fpath, &mut buf) {
                Ok(len) => match TombstoneRecord::deserialize_from(&buf[..len]) {
                    Ok(record) => {
                        let entry = TombstoneIndexEntry::from_record(file_num, &record);
                        self.add_index(entry);
                        self.file_counter.fetch_add(1, Ordering::Relaxed);
                    }
                    Err(_) => {
                        log_warn!("Corrupt tombstone file skipped: {}", fpath);
                    }
                },
                Err(_) => continue,
            }
        }
    }

    /** Return the current index count */
    pub fn count(&self) -> u32 {
        self.index_count.load(Ordering::Relaxed)
    }

    /** Return whether the store is in degraded (memory-only) mode */
    pub fn is_degraded(&self) -> bool {
        !self.fs_available.load(Ordering::Relaxed)
    }
}
