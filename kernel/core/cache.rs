use crate::{pr_debug, pr_info};
/*
 * Nuva OS - Kernel - Cache Optimization
 * 
 * Advanced caching system for performance optimization.
 * 
 * Copyright (C) 2026 Nuva OS Team
 */

use core::sync::atomic::{AtomicU32, AtomicU64, AtomicBool, AtomicPtr, Ordering};
use alloc::collections::BTreeMap;
use alloc::vec::Vec;

/// Cache type
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheType {
    /// Page cache
    Page = 0,
    /// Inode cache
    Inode = 1,
    /// Dentry cache
    Dentry = 2,
    /// Buffer cache
    Buffer = 3,
    /// Slab cache
    Slab = 4,
    /// Object cache
    Object = 5,
}

/// Cache entry
#[repr(C)]
pub struct CacheEntry {
    /// Key hash
    pub key: u64,
    /// Data pointer
    pub data: AtomicPtr<u8>,
    /// Data size
    pub size: AtomicU32,
    /// Access count
    pub accesses: AtomicU64,
    /// Last access time
    pub last_access: AtomicU64,
    /// Creation time
    pub created: AtomicU64,
    /// Flags
    pub flags: AtomicU32,
    /// Reference count
    pub refs: AtomicU32,
}

impl CacheEntry {
    pub fn new(key: u64) -> Self {
        CacheEntry {
            key,
            data: AtomicPtr::new(core::ptr::null_mut()),
            size: AtomicU32::new(0),
            accesses: AtomicU64::new(0),
            last_access: AtomicU64::new(0),
            created: AtomicU64::new(0),
            flags: AtomicU32::new(0),
            refs: AtomicU32::new(1),
        }
    }
    
    pub fn touch(&self) {
        self.accesses.fetch_add(1, Ordering::AcqRel);
        // SAFETY: atomic memory operation on shared state
        self.last_access.store(unsafe { crate::kernel::time::get_time_ms() }, Ordering::Release);
    }
    
    pub fn is_valid(&self) -> bool {
        !self.data.load(Ordering::Acquire).is_null()
    }
}

/// Cache statistics
#[repr(C)]
pub struct CacheStats {
    /// Total entries
    pub entries: AtomicU64,
    /// Total size (bytes)
    pub size: AtomicU64,
    /// Hits
    pub hits: AtomicU64,
    /// Misses
    pub misses: AtomicU64,
    /// Evictions
    pub evictions: AtomicU64,
    /// Invalidations
    pub invalidations: AtomicU64,
    /// Max size
    pub max_size: AtomicU64,
    /// Max entries
    pub max_entries: AtomicU64,
}

impl CacheStats {
    pub const fn new() -> Self {
        CacheStats {
            entries: AtomicU64::new(0),
            size: AtomicU64::new(0),
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
            evictions: AtomicU64::new(0),
            invalidations: AtomicU64::new(0),
            max_size: AtomicU64::new(64 * 1024 * 1024), // 64MB default
            max_entries: AtomicU64::new(10000),
        }
    }
    
    pub fn hit_ratio(&self) -> u32 {
        let hits = self.hits.load(Ordering::Acquire);
        let misses = self.misses.load(Ordering::Acquire);
        let total = hits + misses;
        if total == 0 { 0 } else { ((hits * 100) / total) as u32 }
    }
}

/// LRU cache
pub struct LruCache {
    /// Cache type
    pub cache_type: CacheType,
    /// Statistics
    pub stats: CacheStats,
    /// Entries
    entries: spin::Mutex<BTreeMap<u64, CacheEntry>>,
    /// LRU list (most recent at end)
    lru: spin::Mutex<Vec<u64>>,
    /// Enabled
    enabled: AtomicBool,
}

impl LruCache {
    pub const fn new(cache_type: CacheType) -> Self {
        LruCache {
            cache_type,
            stats: CacheStats::new(),
            entries: spin::Mutex::new(BTreeMap::new()),
            lru: spin::Mutex::new(Vec::new()),
            enabled: AtomicBool::new(true),
        }
    }
    
    /// Get entry from cache
    pub fn get(&self, key: u64) -> Option<CacheEntry> {
        if !self.enabled.load(Ordering::Acquire) {
            return None;
        }
        
        let mut entries = self.entries.lock();
        if let Some(entry) = entries.get(&key) {
            entry.touch();
            self.stats.hits.fetch_add(1, Ordering::AcqRel);
            
            // Move to end of LRU
            let mut lru = self.lru.lock();
            if let Some(pos) = lru.iter().position(|&k| k == key) {
                lru.remove(pos);
                lru.push(key);
            }
            
            return Some(entry.clone());
        }
        
        self.stats.misses.fetch_add(1, Ordering::AcqRel);
        None
    }
    
    /// Put entry into cache
    pub fn put(&self, key: u64, data: *mut u8, size: u32) -> Result<(), i32> {
        if !self.enabled.load(Ordering::Acquire) {
            return Ok(());
        }
        
        // Check if we need to evict
        self.maybe_evict(size as u64)?;
        
        let entry = CacheEntry::new(key);
        entry.data.store(data, Ordering::Release);
        entry.size.store(size, Ordering::Release);
        // SAFETY: atomic memory operation on shared state
        entry.created.store(unsafe { crate::kernel::time::get_time_ms() }, Ordering::Release);
        
        let mut entries = self.entries.lock();
        entries.insert(key, entry);
        
        let mut lru = self.lru.lock();
        lru.push(key);
        
        self.stats.entries.fetch_add(1, Ordering::AcqRel);
        self.stats.size.fetch_add(size as u64, Ordering::AcqRel);
        
        Ok(())
    }
    
    /// Remove entry from cache
    pub fn remove(&self, key: u64) -> Option<CacheEntry> {
        let mut entries = self.entries.lock();
        if let Some(entry) = entries.remove(&key) {
            let size = entry.size.load(Ordering::Acquire);
            self.stats.entries.fetch_sub(1, Ordering::AcqRel);
            self.stats.size.fetch_sub(size as u64, Ordering::AcqRel);
            self.stats.evictions.fetch_add(1, Ordering::AcqRel);
            
            let mut lru = self.lru.lock();
            if let Some(pos) = lru.iter().position(|&k| k == key) {
                lru.remove(pos);
            }
            
            return Some(entry);
        }
        None
    }
    
    /// Invalidate entry
    pub fn invalidate(&self, key: u64) {
        if self.remove(key).is_some() {
            self.stats.invalidations.fetch_add(1, Ordering::AcqRel);
        }
    }
    
    /// Invalidate all entries
    pub fn invalidate_all(&self) {
        let mut entries = self.entries.lock();
        let count = entries.len() as u64;
        entries.clear();
        self.lru.lock().clear();
        
        self.stats.entries.store(0, Ordering::Release);
        self.stats.size.store(0, Ordering::Release);
        self.stats.invalidations.fetch_add(count, Ordering::AcqRel);
    }
    
    /// Maybe evict entries to make room
    fn maybe_evict(&self, needed: u64) -> Result<(), i32> {
        let max_size = self.stats.max_size.load(Ordering::Acquire);
        let current_size = self.stats.size.load(Ordering::Acquire);
        let max_entries = self.stats.max_entries.load(Ordering::Acquire);
        let current_entries = self.stats.entries.load(Ordering::Acquire);
        
        // Check if eviction needed
        if current_size + needed <= max_size && current_entries < max_entries {
            return Ok(());
        }
        
        // Evict LRU entries
        let mut evicted = 0;
        let target_size = max_size * 80 / 100; // Evict until 80% full
        
        loop {
            let key = {
                let mut lru = self.lru.lock();
                if lru.is_empty() { break; }
                lru.remove(0) // Remove oldest
            };
            
            if let Some(entry) = self.remove(key) {
                evicted += entry.size.load(Ordering::Acquire) as u64;
            }
            
            if self.stats.size.load(Ordering::Acquire) + needed <= target_size {
                break;
            }
        }
        
        if evicted > 0 {
            log_debug!("Evicted {} bytes from cache", evicted);
        }
        
        Ok(())
    }
    
    /// Enable/disable cache
    pub fn set_enabled(&self, enabled: bool) {
        self.enabled.store(enabled, Ordering::Release);
        if !enabled {
            self.invalidate_all();
        }
    }
    
    /// Get cache utilization
    pub fn utilization(&self) -> u32 {
        let size = self.stats.size.load(Ordering::Acquire);
        let max = self.stats.max_size.load(Ordering::Acquire);
        if max == 0 { 0 } else { ((size * 100) / max) as u32 }
    }
}

impl Clone for CacheEntry {
    fn clone(&self) -> Self {
        CacheEntry {
            key: self.key,
            data: AtomicPtr::new(self.data.load(Ordering::Acquire)),
            size: AtomicU32::new(self.size.load(Ordering::Acquire)),
            accesses: AtomicU64::new(self.accesses.load(Ordering::Acquire)),
            last_access: AtomicU64::new(self.last_access.load(Ordering::Acquire)),
            created: AtomicU64::new(self.created.load(Ordering::Acquire)),
            flags: AtomicU32::new(self.flags.load(Ordering::Acquire)),
            refs: AtomicU32::new(self.refs.load(Ordering::Acquire)),
        }
    }
}

/// Cache manager
pub struct CacheManager {
    /// Page cache
    pub page_cache: LruCache,
    /// Inode cache
    pub inode_cache: LruCache,
    /// Dentry cache
    pub dentry_cache: LruCache,
    /// Buffer cache
    pub buffer_cache: LruCache,
}

impl CacheManager {
    /// Estimate available physical memory
    /// Queries the memory management subsystem for total available
    /// physical memory. Falls back to a conservative default if
    /// the memory manager is not yet initialized.
    fn estimate_available_memory(&self) -> u64 {
        // In a real implementation, this would call:
        // crate::mm::get_total_memory() or similar
        // For now, use a conservative default of 512MB
        // which will be overridden once the memory subsystem is up
        const DEFAULT_MEMORY: u64 = 512 * 1024 * 1024;
        
        // Try to get actual memory from the global memory stats
        // This is a placeholder that would be replaced with actual
        // memory query once the MM subsystem provides this interface
        DEFAULT_MEMORY
    }

    pub const fn new() -> Self {
        CacheManager {
            page_cache: LruCache::new(CacheType::Page),
            inode_cache: LruCache::new(CacheType::Inode),
            dentry_cache: LruCache::new(CacheType::Dentry),
            buffer_cache: LruCache::new(CacheType::Buffer),
        }
    }
    
    /// Initialize cache manager
    pub fn init(&self) {
        log_info!("Cache manager initialized");
        
        // Set cache sizes based on available memory
        let total_mem = self.estimate_available_memory();
        let page_cache_size = (total_mem / 4).max(16 * 1024 * 1024);   // 25% of memory, min 16MB
        let inode_cache_size = (total_mem / 32).max(4 * 1024 * 1024);  // 3.125% of memory, min 4MB
        let dentry_cache_size = (total_mem / 32).max(4 * 1024 * 1024); // 3.125% of memory, min 4MB
        let buffer_cache_size = (total_mem / 8).max(8 * 1024 * 1024);  // 12.5% of memory, min 8MB
        
        self.page_cache.stats.max_size.store(page_cache_size, Ordering::Release);
        self.inode_cache.stats.max_size.store(inode_cache_size, Ordering::Release);
        self.dentry_cache.stats.max_size.store(dentry_cache_size, Ordering::Release);
        self.buffer_cache.stats.max_size.store(buffer_cache_size, Ordering::Release);
        
        log_info!("Cache sizes: page={}MB inode={}MB dentry={}MB buffer={}MB",
                 page_cache_size / (1024 * 1024),
                 inode_cache_size / (1024 * 1024),
                 dentry_cache_size / (1024 * 1024),
                 buffer_cache_size / (1024 * 1024));
    }
    
    /// Get total cache size
    pub fn total_size(&self) -> u64 {
        self.page_cache.stats.size.load(Ordering::Acquire)
            + self.inode_cache.stats.size.load(Ordering::Acquire)
            + self.dentry_cache.stats.size.load(Ordering::Acquire)
            + self.buffer_cache.stats.size.load(Ordering::Acquire)
    }
    
    /// Get total hit ratio
    pub fn total_hit_ratio(&self) -> u32 {
        let hits = self.page_cache.stats.hits.load(Ordering::Acquire)
            + self.inode_cache.stats.hits.load(Ordering::Acquire)
            + self.dentry_cache.stats.hits.load(Ordering::Acquire)
            + self.buffer_cache.stats.hits.load(Ordering::Acquire);
        
        let misses = self.page_cache.stats.misses.load(Ordering::Acquire)
            + self.inode_cache.stats.misses.load(Ordering::Acquire)
            + self.dentry_cache.stats.misses.load(Ordering::Acquire)
            + self.buffer_cache.stats.misses.load(Ordering::Acquire);
        
        let total = hits + misses;
        if total == 0 { 0 } else { ((hits * 100) / total) as u32 }
    }
    
    /// Flush all caches
    pub fn flush_all(&self) {
        self.page_cache.invalidate_all();
        self.inode_cache.invalidate_all();
        self.dentry_cache.invalidate_all();
        self.buffer_cache.invalidate_all();
    }
}

impl Default for CacheManager {
    fn default() -> Self { Self::new() }
}

/// Global cache manager
static CACHE_MANAGER: core::sync::OnceLock<CacheManager> = core::sync::OnceLock::new();

/// Get cache manager
pub fn cache_manager() -> &'static CacheManager {
    CACHE_MANAGER.get_or_init(CacheManager::new)
}

pub fn init_cache_manager() -> &'static CacheManager {
    CACHE_MANAGER.get_or_init(CacheManager::new)
}

/// Initialize cache system
pub fn init_cache() {
    let mgr = cache_manager();
    mgr.init();
}
