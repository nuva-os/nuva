/*
 * Nuva OS - Kernel - File Page Cache
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

use core::sync::atomic::{AtomicU32, AtomicU64, AtomicBool, Ordering};
use core::ptr;

/// Page cache configuration
pub mod page_cache_config {
    /// Maximum pages in cache
    pub const MAX_CACHE_PAGES: usize = 65536;  // 256MB for 4KB pages
    
    /// Hash table size (must be power of 2)
    pub const HASH_SIZE: usize = 4096;
    
    /// Read-ahead size in pages
    pub const RA_SIZE: u32 = 16;
    
    /// LRU active/inactive ratio
    pub const ACTIVE_RATIO: u32 = 2;  // active = inactive / 2
}

/// Page cache entry key
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PageCacheKey {
    /// Inode number
    pub ino: u64,
    /// Page index (offset / PAGE_SIZE)
    pub index: u64,
}

impl PageCacheKey {
    pub const fn new(ino: u64, index: u64) -> Self {
        PageCacheKey { ino, index }
    }
    
    /// Calculate hash value
    pub fn hash(&self) -> usize {
        let mut h = self.ino;
        h ^= self.index.wrapping_mul(0x9e3779b97f4a7c15);
        h ^= h >> 32;
        (h as usize) & (page_cache_config::HASH_SIZE - 1)
    }
}

/// Page cache entry flags
pub mod page_flags {
    /// Page is uptodate (valid data)
    pub const PG_UPTODATE: u32 = 1 << 0;
    
    /// Page is dirty (modified)
    pub const PG_DIRTY: u32 = 1 << 1;
    
    /// Page is locked
    pub const PG_LOCKED: u32 = 1 << 2;
    
    /// Page is in writeback
    pub const PG_WRITEBACK: u32 = 1 << 3;
    
    /// Page is in active list
    pub const PG_ACTIVE: u32 = 1 << 4;
    
    /// Page is referenced recently
    pub const PG_REFERENCED: u32 = 1 << 5;
    
    /// Page is mapped
    pub const PG_MAPPED: u32 = 1 << 6;
}

/// Page cache entry
pub struct PageCacheEntry {
    /// Entry key
    pub key: PageCacheKey,
    
    /// Physical page address
    pub phys_addr: u64,
    
    /// Entry flags
    pub flags: AtomicU32,
    
    /// Reference count
    pub ref_count: AtomicU32,
    
    /// LRU list pointers
    pub lru_prev: *mut PageCacheEntry,
    pub lru_next: *mut PageCacheEntry,
    
    /// Hash chain next
    pub hash_next: *mut PageCacheEntry,
}

impl PageCacheEntry {
    pub const fn new(key: PageCacheKey, phys_addr: u64) -> Self {
        PageCacheEntry {
            key,
            phys_addr,
            flags: AtomicU32::new(0),
            ref_count: AtomicU32::new(1),
            lru_prev: ptr::null_mut(),
            lru_next: ptr::null_mut(),
            hash_next: ptr::null_mut(),
        }
    }
    
    #[inline]
    pub fn is_uptodate(&self) -> bool {
        (self.flags.load(Ordering::Acquire) & page_flags::PG_UPTODATE) != 0
    }
    
    #[inline]
    pub fn is_dirty(&self) -> bool {
        (self.flags.load(Ordering::Acquire) & page_flags::PG_DIRTY) != 0
    }
    
    #[inline]
    pub fn is_locked(&self) -> bool {
        (self.flags.load(Ordering::Acquire) & page_flags::PG_LOCKED) != 0
    }
    
    #[inline]
    pub fn set_uptodate(&self) {
        self.flags.fetch_or(page_flags::PG_UPTODATE, Ordering::AcqRel);
    }
    
    #[inline]
    pub fn set_dirty(&self) {
        self.flags.fetch_or(page_flags::PG_DIRTY, Ordering::AcqRel);
    }
    
    #[inline]
    pub fn clear_dirty(&self) {
        self.flags.fetch_and(!page_flags::PG_DIRTY, Ordering::AcqRel);
    }
    
    #[inline]
    pub fn set_active(&self) {
        self.flags.fetch_or(page_flags::PG_ACTIVE, Ordering::AcqRel);
    }
    
    #[inline]
    pub fn clear_active(&self) {
        self.flags.fetch_and(!page_flags::PG_ACTIVE, Ordering::AcqRel);
    }
}

/// LRU list
pub struct LruList {
    /// List head
    pub head: *mut PageCacheEntry,
    /// List tail
    pub tail: *mut PageCacheEntry,
    /// Number of entries
    pub count: AtomicU32,
}

impl LruList {
    pub const fn new() -> Self {
        LruList {
            head: ptr::null_mut(),
            tail: ptr::null_mut(),
            count: AtomicU32::new(0),
        }
    }
    
    /// Add entry to tail (most recently used)
    pub fn add_tail(&mut self, entry: *mut PageCacheEntry) {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            (*entry).lru_prev = self.tail;
            (*entry).lru_next = ptr::null_mut();
            
            if !self.tail.is_null() {
                (*self.tail).lru_next = entry;
            } else {
                self.head = entry;
            }
            self.tail = entry;
            
            self.count.fetch_add(1, Ordering::AcqRel);
        }
    }
    
    /// Remove entry from list
    pub fn remove(&mut self, entry: *mut PageCacheEntry) {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            if !(*entry).lru_prev.is_null() {
                (*(*entry).lru_prev).lru_next = (*entry).lru_next;
            } else {
                self.head = (*entry).lru_next;
            }
            
            if !(*entry).lru_next.is_null() {
                (*(*entry).lru_next).lru_prev = (*entry).lru_prev;
            } else {
                self.tail = (*entry).lru_prev;
            }
            
            (*entry).lru_prev = ptr::null_mut();
            (*entry).lru_next = ptr::null_mut();
            
            self.count.fetch_sub(1, Ordering::AcqRel);
        }
    }
    
    /// Move entry to tail (mark as recently used)
    pub fn move_to_tail(&mut self, entry: *mut PageCacheEntry) {
        self.remove(entry);
        self.add_tail(entry);
    }
    
    /// Pop from head (least recently used)
    pub fn pop_head(&mut self) -> *mut PageCacheEntry {
        let entry = self.head;
        if !entry.is_null() {
            self.remove(entry);
        }
        entry
    }
}

/// Page cache
pub struct PageCache {
    /// Hash table
    pub hash_table: [*mut PageCacheEntry; page_cache_config::HASH_SIZE],
    
    /// Active LRU list
    pub active_list: LruList,
    
    /// Inactive LRU list
    pub inactive_list: LruList,
    
    /// Total pages in cache
    pub nr_pages: AtomicU32,
    
    /// Maximum pages
    pub max_pages: u32,
    
    /// Cache statistics
    pub stats: PageCacheStats,
    
    /// Initialized flag
    pub initialized: AtomicBool,
}

/// Page cache statistics
pub struct PageCacheStats {
    /// Cache hits
    pub hits: AtomicU64,
    
    /// Cache misses
    pub misses: AtomicU64,
    
    /// Pages read
    pub pages_read: AtomicU64,
    
    /// Pages written
    pub pages_written: AtomicU64,
    
    /// Pages evicted
    pub pages_evicted: AtomicU64,
}

impl PageCache {
    pub const fn new() -> Self {
        PageCache {
            hash_table: [ptr::null_mut(); page_cache_config::HASH_SIZE],
            active_list: LruList::new(),
            inactive_list: LruList::new(),
            nr_pages: AtomicU32::new(0),
            max_pages: page_cache_config::MAX_CACHE_PAGES as u32,
            stats: PageCacheStats {
                hits: AtomicU64::new(0),
                misses: AtomicU64::new(0),
                pages_read: AtomicU64::new(0),
                pages_written: AtomicU64::new(0),
                pages_evicted: AtomicU64::new(0),
            },
            initialized: AtomicBool::new(false),
        }
    }
    
    /// Initialize page cache
    pub fn init(&mut self) {
        self.initialized.store(true, Ordering::Release);
    }
    
    /// Look up a page in the cache
    pub fn lookup(&mut self, key: &PageCacheKey) -> *mut PageCacheEntry {
        let hash = key.hash();
        let mut entry = self.hash_table[hash];
        
        while !entry.is_null() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                if (*entry).key == *key {
                    // Cache hit
                    self.stats.hits.fetch_add(1, Ordering::Relaxed);
                    
                    // Move to active list
                    (*entry).set_active();
                    self.active_list.move_to_tail(entry);
                    
                    // Increment reference count
                    (*entry).ref_count.fetch_add(1, Ordering::AcqRel);
                    
                    return entry;
                }
                entry = (*entry).hash_next;
            }
        }
        
        // Cache miss
        self.stats.misses.fetch_add(1, Ordering::Relaxed);
        ptr::null_mut()
    }
    
    /// Add a page to the cache
    pub fn add(&mut self, entry: *mut PageCacheEntry) -> bool {
        if entry.is_null() {
            return false;
        }
        
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let key = (*entry).key;
            let hash = key.hash();
            
            // Check if already exists
            let mut existing = self.hash_table[hash];
            while !existing.is_null() {
                if (*existing).key == key {
                    return false;  // Already exists
                }
                existing = (*existing).hash_next;
            }
            
            // Add to hash table
            (*entry).hash_next = self.hash_table[hash];
            self.hash_table[hash] = entry;
            
            // Add to inactive list
            self.inactive_list.add_tail(entry);
            
            self.nr_pages.fetch_add(1, Ordering::AcqRel);
            
            // Check if we need to evict
            if self.nr_pages.load(Ordering::Acquire) > self.max_pages {
                self.evict_pages();
            }
        }
        
        true
    }
    
    /// Remove a page from the cache
    pub fn remove(&mut self, entry: *mut PageCacheEntry) {
        if entry.is_null() {
            return;
        }
        
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let key = (*entry).key;
            let hash = key.hash();
            
            // Remove from hash table
            let mut prev: *mut PageCacheEntry = ptr::null_mut();
            let mut current = self.hash_table[hash];
            
            while !current.is_null() {
                if current == entry {
                    if prev.is_null() {
                        self.hash_table[hash] = (*current).hash_next;
                    } else {
                        (*prev).hash_next = (*current).hash_next;
                    }
                    break;
                }
                prev = current;
                current = (*current).hash_next;
            }
            
            // Remove from LRU list
            if (*entry).flags.load(Ordering::Acquire) & page_flags::PG_ACTIVE != 0 {
                self.active_list.remove(entry);
            } else {
                self.inactive_list.remove(entry);
            }
            
            self.nr_pages.fetch_sub(1, Ordering::AcqRel);
        }
    }
    
    /// Evict pages from cache
    fn evict_pages(&mut self) {
        // Evict from inactive list first
        let target = self.nr_pages.load(Ordering::Acquire) - self.max_pages;
        
        for _ in 0..target {
            let entry = self.inactive_list.pop_head();
            if entry.is_null() {
                break;
            }
            
            // Check if page is dirty or locked
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                if (*entry).is_dirty() || (*entry).is_locked() {
                    // Can't evict, move to tail
                    self.inactive_list.add_tail(entry);
                    continue;
                }
                
                // Remove from hash table
                self.remove(entry);
                
                // Free the page
                // TODO: Call page allocator to free
                
                self.stats.pages_evicted.fetch_add(1, Ordering::Relaxed);
            }
        }
    }
    
    /// Read a page from file
    /// Returns the page entry, reading from disk if necessary
    pub fn read_page(&mut self, key: &PageCacheKey, read_fn: fn(u64, u64, u64) -> i32) -> *mut PageCacheEntry {
        // Check cache first
        let entry = self.lookup(key);
        if !entry.is_null() {
            return entry;
        }
        
        // Allocate a new page
        // TODO: Call page allocator
        let phys_addr = 0u64;  // Placeholder
        
        // Create new entry
        // TODO: Allocate from slab
        // For now, return null as we can't allocate
        let new_entry = ptr::null_mut::<PageCacheEntry>();
        
        if new_entry.is_null() {
            return ptr::null_mut();
        }
        
        // Read from disk
        let ret = read_fn(key.ino, key.index, phys_addr);
        if ret < 0 {
            // Read failed
            // TODO: Free page and entry
            return ptr::null_mut();
        }
        
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            (*new_entry).set_uptodate();
        }
        
        // Add to cache
        self.add(new_entry);
        self.stats.pages_read.fetch_add(1, Ordering::Relaxed);
        
        new_entry
    }
    
    /// Mark a page as dirty
    pub fn mark_dirty(&mut self, entry: *mut PageCacheEntry) {
        if !entry.is_null() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                (*entry).set_dirty();
            }
            self.stats.pages_written.fetch_add(1, Ordering::Relaxed);
        }
    }
    
    /// Get cache hit rate
    pub fn get_hit_rate(&self) -> u32 {
        let hits = self.stats.hits.load(Ordering::Relaxed);
        let misses = self.stats.misses.load(Ordering::Relaxed);
        let total = hits + misses;
        
        if total == 0 {
            return 0;
        }
        
        ((hits * 1000) / total) as u32
    }
}

/// Global page cache
static PAGE_CACHE: core::sync::OnceLock<PageCache> = core::sync::OnceLock::new();

/// Get the page cache
pub fn page_cache() -> &'static PageCache {
    PAGE_CACHE.get_or_init(PageCache::new)
}

/// Initialize page cache
pub fn init_page_cache() {
    get_page_cache().init();
}

/// Look up a page in cache
pub fn page_cache_lookup(ino: u64, index: u64) -> *mut PageCacheEntry {
    let key = PageCacheKey::new(ino, index);
    get_page_cache().lookup(&key)
}
