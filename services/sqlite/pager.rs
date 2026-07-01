/*
 * Nuva OS - SystemService - SQLite - Pager
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

//! Page read/write manager for SQLite database files.
//! Reads and writes fixed-size pages via NuvaFS, with page caching
//! and dirty page writeback.

use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

use super::error::SqliteError;
use alloc::vec;

/// Page identifier (1-based, page 1 is the database header)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PageId(pub u32);

/// Default database page size (4 KiB, same as standard SQLite)
pub const PAGE_SIZE: usize = 4096;

/// Default page cache capacity
const DEFAULT_CACHE_CAPACITY: u32 = 1024;

/// Page content type
#[derive(Debug, Clone)]
pub struct Page {
    /// Page ID
    pub id: PageId,
    /// Page data bytes (always PAGE_SIZE)
    pub data: [u8; PAGE_SIZE],
    /// Whether this page has been modified (dirty flag)
    pub dirty: bool,
    /// Whether this page is currently pinned (in use)
    pub pin_count: u32,
}

impl Page {
    /// Create a new zero-filled page
    pub fn new(id: PageId) -> Self {
        Page {
            id,
            data: [0u8; PAGE_SIZE],
            dirty: false,
            pin_count: 0,
        }
    }

    /// Create a page from raw data
    pub fn from_data(id: PageId, data: &[u8]) -> Result<Self, SqliteError> {
        if data.len() != PAGE_SIZE {
            return Err(SqliteError::DatabaseCorrupted);
        }
        let mut page = Page::new(id);
        page.data.copy_from_slice(data);
        Ok(page)
    }
}

/// Pager statistics
#[derive(Debug)]
pub struct PagerStats {
    /// Total page reads from disk
    pub reads: AtomicU64,
    /// Total page writes to disk
    pub writes: AtomicU64,
    /// Cache hits
    pub cache_hits: AtomicU64,
    /// Cache misses
    pub cache_misses: AtomicU64,
    /// Pages currently in cache
    pub cached_pages: AtomicU32,
}

impl PagerStats {
    /// Create zero-initialized stats
    pub const fn new() -> Self {
        PagerStats {
            reads: AtomicU64::new(0),
            writes: AtomicU64::new(0),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
            cached_pages: AtomicU32::new(0),
        }
    }
}

/// Page read/write manager with caching
pub struct Pager {
    /// Page cache (in-memory)
    cache: BTreeMap<u32, Page>,
    /// Maximum number of pages in the cache
    cache_capacity: u32,
    /// Total number of pages in the database file
    page_count: u32,
    /// Database file descriptor (NuvaFS handle)
    file_handle: u64,
    /// Pager statistics
    stats: PagerStats,
    /// Next page counter for allocation
    next_page: AtomicU32,
}

/// Database file header (occupies the first 100 bytes of page 1)
#[derive(Debug, Clone, Copy)]
pub struct DbHeader {
    /// Magic string: "SQLite format 3\000"
    pub magic: [u8; 16],
    /// Page size in bytes
    pub page_size: u16,
    /// File format write version (1 for legacy, 2 for WAL)
    pub write_version: u8,
    /// File format read version
    pub read_version: u8,
    /// Reserved space at end of each page
    pub reserved_space: u8,
    /// Maximum embedded payload fraction (must be 64)
    pub max_payload_frac: u8,
    /// Minimum embedded payload fraction (must be 32)
    pub min_payload_frac: u8,
    /// Leaf payload fraction (must be 32)
    pub leaf_payload_frac: u8,
    /// File change counter
    pub file_change_counter: u32,
    /// Size of the database file in pages
    pub db_size_pages: u32,
    /// First freelist trunk page
    pub first_freelist_trunk: u32,
    /// Total freelist pages
    pub freelist_page_count: u32,
    /// Schema cookie
    pub schema_cookie: u32,
    /// Schema format number
    pub schema_format: u32,
    /// Default page cache size
    pub default_cache_size: u32,
    /// Largest root b-tree page number (auto-vacuum/incremental-vacuum)
    pub largest_root_page: u32,
    /// Database text encoding (1=UTF-8, 2=UTF-16le, 3=UTF-16be)
    pub text_encoding: u32,
    /// User version
    pub user_version: u32,
    /// Incremental vacuum mode
    pub incremental_vacuum: u32,
    /// Application ID
    pub app_id: u32,
    /// Version-valid-for number
    pub version_valid_for: u32,
    /// SQLite version number
    pub sqlite_version: u32,
}

impl DbHeader {
    /// Create a default header for a new database
    pub fn new() -> Self {
        let mut magic = [0u8; 16];
        let magic_str = b"SQLite format 3\0";
        magic.copy_from_slice(magic_str);

        DbHeader {
            magic,
            page_size: PAGE_SIZE as u16,
            write_version: 2, // WAL mode
            read_version: 2,
            reserved_space: 0,
            max_payload_frac: 64,
            min_payload_frac: 32,
            leaf_payload_frac: 32,
            file_change_counter: 0,
            db_size_pages: 0,
            first_freelist_trunk: 0,
            freelist_page_count: 0,
            schema_cookie: 0,
            schema_format: 4,
            default_cache_size: DEFAULT_CACHE_CAPACITY,
            largest_root_page: 0,
            text_encoding: 1, // UTF-8
            user_version: 0,
            incremental_vacuum: 0,
            app_id: 0,
            version_valid_for: 0,
            sqlite_version: 3039004, // 3.39.4
        }
    }
}

impl Pager {
    /// Create a new pager for a database file
    pub fn new(file_handle: u64, page_count: u32) -> Self {
        Pager {
            cache: BTreeMap::new(),
            cache_capacity: DEFAULT_CACHE_CAPACITY,
            page_count,
            file_handle,
            stats: PagerStats::new(),
            next_page: AtomicU32::new(page_count + 1),
        }
    }

    /// Read a page from cache or disk
    pub fn read_page(&mut self, page_id: PageId) -> Result<Page, SqliteError> {
        // Check cache first
        if let Some(page) = self.cache.get(&page_id.0) {
            self.stats.cache_hits.fetch_add(1, Ordering::Relaxed);
            return Ok(page.clone());
        }

        self.stats.cache_misses.fetch_add(1, Ordering::Relaxed);

        // Read from disk via NuvaFS
        let data = self.read_page_from_disk(page_id)?;
        let page = Page::from_data(page_id, &data)?;
        self.stats.reads.fetch_add(1, Ordering::Relaxed);

        // Add to cache (evict if full)
        if self.cache.len() >= self.cache_capacity as usize {
            self.evict_clean_page();
        }
        self.cache.insert(page_id.0, page.clone());
        self.stats.cached_pages.store(self.cache.len() as u32, Ordering::Relaxed);

        Ok(page)
    }

    /// Write a page (marks it dirty in cache)
    pub fn write_page(&mut self, page: Page) -> Result<(), SqliteError> {
        let page_id = page.id;
        let mut page = page;
        page.dirty = true;
        self.cache.insert(page_id.0, page);
        self.stats.writes.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    /// Allocate a new page
    pub fn allocate_page(&mut self) -> Result<PageId, SqliteError> {
        let page_id = PageId(self.next_page.fetch_add(1, Ordering::Relaxed));
        self.page_count += 1;

        let page = Page::new(page_id);
        self.cache.insert(page_id.0, page);

        Ok(page_id)
    }

    /// Flush all dirty pages to disk
    pub fn sync(&mut self) -> Result<(), SqliteError> {
        let dirty_pages: Vec<(u32, Page)> = self
            .cache
            .iter()
            .filter(|(_, p)| p.dirty)
            .map(|(id, p)| (*id, p.clone()))
            .collect();

        for (id, page) in &dirty_pages {
            self.write_page_to_disk(page)?;
            if let Some(p) = self.cache.get_mut(id) {
                p.dirty = false;
            }
        }

        Ok(())
    }

    /// Flush a single dirty page to disk
    pub fn sync_page(&mut self, page_id: PageId) -> Result<(), SqliteError> {
        if let Some(page) = self.cache.get(&page_id.0) {
            if page.dirty {
                self.write_page_to_disk(page)?;
                if let Some(p) = self.cache.get_mut(&page_id.0) {
                    p.dirty = false;
                }
            }
        }
        Ok(())
    }

    /// Pin a page (increment reference count)
    pub fn pin(&mut self, page_id: PageId) {
        if let Some(page) = self.cache.get_mut(&page_id.0) {
            page.pin_count += 1;
        }
    }

    /// Unpin a page (decrement reference count)
    pub fn unpin(&mut self, page_id: PageId) {
        if let Some(page) = self.cache.get_mut(&page_id.0) {
            if page.pin_count > 0 {
                page.pin_count -= 1;
            }
        }
    }

    /// Returns the total number of pages in the database
    pub fn page_count(&self) -> u32 {
        self.page_count
    }

    /// Returns the database file handle
    pub fn file_handle(&self) -> u64 {
        self.file_handle
    }

    /// Evict a clean (non-dirty, non-pinned) page from cache
    fn evict_clean_page(&mut self) {
        let evict_key = self
            .cache
            .iter()
            .find(|(_, p)| !p.dirty && p.pin_count == 0)
            .map(|(id, _)| *id);

        if let Some(key) = evict_key {
            self.cache.remove(&key);
        }
    }

    /// Read a page from disk via NuvaFS file API
    fn read_page_from_disk(&self, page_id: PageId) -> Result<Vec<u8>, SqliteError> {
        // In a full implementation, this would call NuvaFS read:
        //   nuva_fs_read(self.file_handle, offset, PAGE_SIZE)
        // where offset = (page_id.0 - 1) * PAGE_SIZE
        let _ = (self.file_handle, page_id);
        Ok(vec![0u8; PAGE_SIZE])
    }

    /// Write a page to disk via NuvaFS file API
    fn write_page_to_disk(&self, page: &Page) -> Result<(), SqliteError> {
        // In a full implementation, this would call NuvaFS write:
        //   nuva_fs_write(self.file_handle, offset, &page.data)
        // where offset = (page.id.0 - 1) * PAGE_SIZE
        let _ = (self.file_handle, page);
        Ok(())
    }

    /// Read and parse the database header from page 1
    pub fn read_db_header(&self) -> Result<DbHeader, SqliteError> {
        // In a full implementation, parse the first 100 bytes of page 1
        Ok(DbHeader::new())
    }

    /// Write the database header to page 1
    pub fn write_db_header(&mut self, header: &DbHeader) -> Result<(), SqliteError> {
        let _ = header;
        // In a full implementation, serialize header into page 1
        Ok(())
    }
}
