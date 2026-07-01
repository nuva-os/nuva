/*
 * Nuva OS - Kernel - Memory Management - Per-CPU Page Cache
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

use super::{Page, PhysAddr, PAGE_SIZE, ZoneType};

/** Per-CPU page cache configuration */
pub mod pcp_cache_config {
    /** Number of pages per CPU cache (per order) */
    pub const PCP_CACHE_SIZE: usize = 128;

    /** High watermark - when to return pages to buddy allocator */
    pub const PCP_HIGH: u32 = 32;

    /** Low watermark - when to replenish from buddy allocator */
    pub const PCP_LOW: u32 = 8;

    /** Batch size for bulk refill/drain operations */
    pub const PCP_BATCH: u32 = 16;

    /** Number of orders to cache (0, 1, 2 = 4KB, 8KB, 16KB) */
    pub const PCP_NR_ORDERS: usize = 3;

    /** L1 cache line size for alignment optimization */
    pub const CACHE_LINE_SIZE: usize = 64;
}

/**
 * Per-CPU page cache for a specific order.
 *
 * Provides fast allocation without global locks by maintaining
 * a per-CPU cache of free pages. When the cache falls below
 * the low watermark, it is replenished from the buddy allocator
 * in batch. When it exceeds the high watermark, pages are
 * drained back to buddy.
 */
#[repr(C, align(64))]
pub struct PerCpuPageCache {
    /** Cached pages */
    pub pages: [*mut Page; pcp_cache_config::PCP_CACHE_SIZE],

    /** Number of cached pages */
    pub count: AtomicU32,

    /** High watermark - drain when count >= high */
    pub high: u32,

    /** Low watermark - refill when count < low */
    pub low: u32,

    /** Batch size for bulk operations */
    pub batch: u32,

    /** Cache statistics */
    pub stats: PcpStats,
}

/// Per-CPU page cache statistics
pub struct PcpStats {
    /// Number of allocations from cache
    pub alloc_count: AtomicU64,
    
    /// Number of allocations that missed cache
    pub miss_count: AtomicU64,
    
    /// Number of pages returned to buddy
    pub drain_count: AtomicU64,
    
    /// Number of bulk allocations
    pub bulk_alloc_count: AtomicU64,
}

impl PerCpuPageCache {
    /** Create a new per-CPU page cache */
    pub const fn new() -> Self {
        PerCpuPageCache {
            pages: [ptr::null_mut(); pcp_cache_config::PCP_CACHE_SIZE],
            count: AtomicU32::new(0),
            high: pcp_cache_config::PCP_HIGH,
            low: pcp_cache_config::PCP_LOW,
            batch: pcp_cache_config::PCP_BATCH,
            stats: PcpStats {
                alloc_count: AtomicU64::new(0),
                miss_count: AtomicU64::new(0),
                drain_count: AtomicU64::new(0),
                bulk_alloc_count: AtomicU64::new(0),
            },
        }
    }
    
    /// Allocate a page from the cache
    /// Returns null if cache is empty
    #[inline(always)]
    pub fn alloc(&mut self) -> *mut Page {
        let count = self.count.load(Ordering::Acquire);
        
        if count == 0 {
            self.stats.miss_count.fetch_add(1, Ordering::Relaxed);
            return ptr::null_mut();
        }
        
        // Fast path: pop from cache
        let idx = count - 1;
        let page = self.pages[idx as usize];
        
        self.pages[idx as usize] = ptr::null_mut();
        self.count.store(idx, Ordering::Release);
        self.stats.alloc_count.fetch_add(1, Ordering::Relaxed);
        
        page
    }
    
    /// Free a page to the cache
    /// Returns true if page was cached, false if cache is full
    #[inline(always)]
    pub fn free(&mut self, page: *mut Page) -> bool {
        let count = self.count.load(Ordering::Acquire);
        
        // Don't cache if above high watermark
        if count >= self.high {
            return false;
        }
        
        // Add to cache
        self.pages[count as usize] = page;
        self.count.store(count + 1, Ordering::Release);
        
        true
    }
    
    /// Bulk allocate pages from cache
    /// Returns number of pages allocated
    pub fn bulk_alloc(&mut self, pages: &mut [*mut Page]) -> usize {
        let count = self.count.load(Ordering::Acquire);
        let to_alloc = core::cmp::min(count as usize, pages.len());
        
        if to_alloc == 0 {
            return 0;
        }
        
        let new_count = count - to_alloc as u32;
        
        // Copy pages from cache
        for i in 0..to_alloc {
            pages[i] = self.pages[(new_count as usize) + i];
            self.pages[(new_count as usize) + i] = ptr::null_mut();
        }
        
        self.count.store(new_count, Ordering::Release);
        self.stats.bulk_alloc_count.fetch_add(1, Ordering::Relaxed);
        
        to_alloc
    }
    
    /// Drain cache to buddy allocator
    /// Returns array of pages to free and count
    pub fn drain(&mut self) -> ([*mut Page; pcp_cache_config::PCP_CACHE_SIZE], u32) {
        let count = self.count.load(Ordering::Acquire);
        
        if count == 0 {
            return ([ptr::null_mut(); pcp_cache_config::PCP_CACHE_SIZE], 0);
        }
        
        let mut pages = [ptr::null_mut(); pcp_cache_config::PCP_CACHE_SIZE];
        
        for i in 0..count as usize {
            pages[i] = self.pages[i];
            self.pages[i] = ptr::null_mut();
        }
        
        self.count.store(0, Ordering::Release);
        self.stats.drain_count.fetch_add(count as u64, Ordering::Relaxed);
        
        (pages, count)
    }
    
    /// Get cache statistics
    pub fn get_stats(&self) -> (u64, u64, u64, u64) {
        (
            self.stats.alloc_count.load(Ordering::Relaxed),
            self.stats.miss_count.load(Ordering::Relaxed),
            self.stats.drain_count.load(Ordering::Relaxed),
            self.stats.bulk_alloc_count.load(Ordering::Relaxed),
        )
    }
}

/// Per-CPU page cache set (one cache per order)
pub struct PerCpuPageCacheSet {
    /// Caches for each order
    pub caches: [PerCpuPageCache; pcp_cache_config::PCP_NR_ORDERS],
    
    /// CPU ID
    pub cpu_id: u32,
    
    /// Initialized flag
    pub initialized: AtomicBool,
}

impl PerCpuPageCacheSet {
    pub const fn new() -> Self {
        PerCpuPageCacheSet {
            caches: [
                PerCpuPageCache::new(),
                PerCpuPageCache::new(),
                PerCpuPageCache::new(),
            ],
            cpu_id: 0,
            initialized: AtomicBool::new(false),
        }
    }
    
    /// Initialize the cache set for a specific CPU
    pub fn init(&mut self, cpu_id: u32) {
        self.cpu_id = cpu_id;
        self.initialized.store(true, Ordering::Release);
    }
    
    /// Allocate a page of the given order
    #[inline(always)]
    pub fn alloc_page(&mut self, order: usize) -> *mut Page {
        if order >= pcp_cache_config::PCP_NR_ORDERS {
            return ptr::null_mut();
        }
        
        self.caches[order].alloc()
    }
    
    /// Free a page of the given order
    #[inline(always)]
    pub fn free_page(&mut self, page: *mut Page, order: usize) -> bool {
        if order >= pcp_cache_config::PCP_NR_ORDERS {
            return false;
        }
        
        self.caches[order].free(page)
    }
    
    /// Drain all caches
    pub fn drain_all(&mut self) -> [([*mut Page; pcp_cache_config::PCP_CACHE_SIZE], u32); pcp_cache_config::PCP_NR_ORDERS] {
        let mut result = [([ptr::null_mut(); pcp_cache_config::PCP_CACHE_SIZE], 0); pcp_cache_config::PCP_NR_ORDERS];
        
        for i in 0..pcp_cache_config::PCP_NR_ORDERS {
            result[i] = self.caches[i].drain();
        }
        
        result
    }
}

/// Global per-CPU page cache manager
pub struct PerCpuPageCacheManager {
    /// Per-CPU cache sets
    pub cache_sets: [PerCpuPageCacheSet; 16],  // Support up to 16 CPUs
    
    /// Number of CPUs
    pub nr_cpus: u32,
    
    /// Initialized flag
    pub initialized: AtomicBool,
    
    /// Global statistics
    pub global_stats: GlobalPcpStats,
}

/// Global per-CPU page cache statistics
pub struct GlobalPcpStats {
    /// Total allocations from cache
    pub total_alloc: AtomicU64,
    
    /// Total cache misses
    pub total_miss: AtomicU64,
    
    /// Total drains
    pub total_drain: AtomicU64,
    
    /// Cache hit rate (scaled by 1000)
    pub hit_rate: AtomicU32,
}

impl PerCpuPageCacheManager {
    pub const fn new() -> Self {
        PerCpuPageCacheManager {
            cache_sets: [
                PerCpuPageCacheSet::new(),
                PerCpuPageCacheSet::new(),
                PerCpuPageCacheSet::new(),
                PerCpuPageCacheSet::new(),
                PerCpuPageCacheSet::new(),
                PerCpuPageCacheSet::new(),
                PerCpuPageCacheSet::new(),
                PerCpuPageCacheSet::new(),
                PerCpuPageCacheSet::new(),
                PerCpuPageCacheSet::new(),
                PerCpuPageCacheSet::new(),
                PerCpuPageCacheSet::new(),
                PerCpuPageCacheSet::new(),
                PerCpuPageCacheSet::new(),
                PerCpuPageCacheSet::new(),
                PerCpuPageCacheSet::new(),
            ],
            nr_cpus: 0,
            initialized: AtomicBool::new(false),
            global_stats: GlobalPcpStats {
                total_alloc: AtomicU64::new(0),
                total_miss: AtomicU64::new(0),
                total_drain: AtomicU64::new(0),
                hit_rate: AtomicU32::new(0),
            },
        }
    }
    
    /// Initialize the per-CPU page cache manager
    pub fn init(&mut self, nr_cpus: u32) {
        self.nr_cpus = nr_cpus;
        
        for i in 0..nr_cpus as usize {
            self.cache_sets[i].init(i as u32);
        }
        
        self.initialized.store(true, Ordering::Release);
    }
    
    /// Get the current CPU ID
    /// This should be replaced with actual CPU ID retrieval
    #[inline(always)]
    fn get_current_cpu(&self) -> u32 {
        // SAFETY: reading CPU ID is safe on SMP systems
        crate::hal::cpu::smp_processor_id()
    }
    
    /// Allocate a page from the per-CPU cache
    #[inline(always)]
    pub fn alloc_pages(&mut self, order: usize) -> *mut Page {
        let cpu = self.get_current_cpu() as usize;
        
        if cpu >= self.nr_cpus as usize {
            return ptr::null_mut();
        }
        
        let page = self.cache_sets[cpu].alloc_page(order);
        
        if !page.is_null() {
            self.global_stats.total_alloc.fetch_add(1, Ordering::Relaxed);
            return page;
        }
        
        self.global_stats.total_miss.fetch_add(1, Ordering::Relaxed);
        ptr::null_mut()
    }
    
    /// Free a page to the per-CPU cache
    #[inline(always)]
    pub fn free_pages(&mut self, page: *mut Page, order: usize) -> bool {
        let cpu = self.get_current_cpu() as usize;
        
        if cpu >= self.nr_cpus as usize {
            return false;
        }
        
        self.cache_sets[cpu].free_page(page, order)
    }
    
    /// Drain all per-CPU caches
    /// This is called when memory is low
    pub fn drain_all_cpus(&mut self) -> u32 {
        let mut total_drained = 0u32;
        
        for i in 0..self.nr_cpus as usize {
            let drained = self.cache_sets[i].drain_all();
            
            for j in 0..pcp_cache_config::PCP_NR_ORDERS {
                total_drained += drained[j].1;
            }
        }
        
        self.global_stats.total_drain.fetch_add(total_drained as u64, Ordering::Relaxed);
        total_drained
    }
    
    /// Update hit rate statistics
    pub fn update_stats(&self) {
        let alloc = self.global_stats.total_alloc.load(Ordering::Relaxed);
        let miss = self.global_stats.total_miss.load(Ordering::Relaxed);
        let total = alloc + miss;
        
        if total > 0 {
            let rate = ((alloc * 1000) / total) as u32;
            self.global_stats.hit_rate.store(rate, Ordering::Relaxed);
        }
    }
    
    /// Print statistics
    pub fn print_stats(&self) {
        // Use log_info! macro if available
        // log_info!("Per-CPU Page Cache Statistics:");
        // log_info!("  Total alloc: {}", self.global_stats.total_alloc.load(Ordering::Relaxed));
        // log_info!("  Total miss: {}", self.global_stats.total_miss.load(Ordering::Relaxed));
        // log_info!("  Hit rate: {}%", self.global_stats.hit_rate.load(Ordering::Relaxed) as f64 / 10.0);
    }
}

/// Global per-CPU page cache manager
static PCP_MANAGER: crate::sync_oncelock::OnceLock<PerCpuPageCacheManager> = crate::sync_oncelock::OnceLock::new();

/// Get the per-CPU page cache manager
pub fn pcp_manager() -> &'static PerCpuPageCacheManager {
    PCP_MANAGER.get_or_init(PerCpuPageCacheManager::new)
}

pub fn init_pcp_manager() -> &'static PerCpuPageCacheManager {
    PCP_MANAGER.get_or_init(PerCpuPageCacheManager::new)
}

/// Initialize per-CPU page cache
pub fn init_pcp_cache(nr_cpus: u32) {
    pcp_manager().init(nr_cpus);
}

/// Allocate a page from per-CPU cache
#[inline(always)]
pub fn pcp_alloc_pages(order: usize) -> *mut Page {
    pcp_manager().alloc_pages(order)
}

/// Free a page to per-CPU cache
#[inline(always)]
pub fn pcp_free_pages(page: *mut Page, order: usize) -> bool {
    pcp_manager().free_pages(page, order)
}
