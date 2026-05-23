/*
 * Nuva OS
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

//! Page Allocation - Integration with Buddy allocator and Per-CPU cache

use core::sync::atomic::{AtomicU64, AtomicU32, AtomicBool, Ordering};

use super::{Page, PhysAddr, VirtAddr, PAGE_SIZE, ZoneType};
use super::percpu_cache::{pcp_alloc_pages, pcp_free_pages, init_pcp_cache};

/// Allocation flags
pub mod gfp_flags {
    /// Normal kernel allocation
    pub const GFP_KERNEL: u32 = 0x00;
    /// Atomic allocation (cannot sleep)
    pub const GFP_ATOMIC: u32 = 0x01;
    /// DMA-able memory
    pub const GFP_DMA: u32 = 0x02;
    /// High memory allocation
    pub const GFP_HIGHMEM: u32 = 0x04;
    /// Zero the page
    pub const GFP_ZERO: u32 = 0x08;
    /// No retry on failure
    pub const GFP_NORETRY: u32 = 0x10;
    /// Nowait semantics
    pub const GFP_NOWAIT: u32 = 0x20;
    /// Reclaim allowed
    pub const GFP_RECLAIM: u32 = 0x40;
    /// Compaction allowed
    pub const GFP_COMP: u32 = 0x80;
}

/// Per-CPU page cache (PCP) watermark levels.
/// Controls when the PCP cache refills from or drains to the buddy allocator.
/// Watermarks are computed adaptively based on total RAM to balance
/// between memory overhead and allocation latency.
#[derive(Debug, Clone, Copy)]
pub struct PcpWatermark {
    /// Minimum number of pages in PCP cache before refill
    pub low: u32,
    /// Target number of pages in PCP cache (refill target)
    pub high: u32,
    /// Batch size for refill/drain operations
    pub batch: u32,
}

/// Minimum watermark base value (pages)
const PCP_WATERMARK_MIN: u64 = 32;
/// Maximum watermark base value (pages)
const PCP_WATERMARK_MAX: u64 = 512;

impl PcpWatermark {
    /// Create watermarks from total RAM pages.
    /// base = (totalram_pages / 1024).clamp(32, 512)
    /// high = base * 2, low = base, batch = base / 4 (min 1)
    pub fn from_totalram(totalram_pages: u64) -> Self {
        let base = (totalram_pages / 1024).clamp(PCP_WATERMARK_MIN, PCP_WATERMARK_MAX);
        let batch = (base / 4).max(1) as u32;
        PcpWatermark {
            low: base as u32,
            high: (base * 2) as u32,
            batch,
        }
    }

    /// Default watermarks for systems where total RAM is unknown
    pub const fn default_watermarks() -> Self {
        PcpWatermark {
            low: 64,
            high: 128,
            batch: 16,
        }
    }

    /// Check if PCP cache needs refill (count < low)
    #[inline(always)]
    pub fn needs_refill(&self, count: u32) -> bool {
        count < self.low
    }

    /// Check if PCP cache needs drain (count > high)
    #[inline(always)]
    pub fn needs_drain(&self, count: u32) -> bool {
        count > self.high
    }
}

/// Page allocation statistics
pub struct PageAllocStats {
    /// Total pages allocated
    pub alloc_total: AtomicU64,
    /// Total pages freed
    pub free_total: AtomicU64,
    /// Per-CPU cache hits
    pub pcp_hits: AtomicU64,
    /// Per-CPU cache misses (fell through to buddy)
    pub pcp_misses: AtomicU64,
    /// Buddy allocations
    pub buddy_allocs: AtomicU64,
    /// Allocation failures
    pub alloc_fails: AtomicU64,
    /// Order-0 allocations
    pub order0_allocs: AtomicU64,
    /// Higher-order allocations
    pub high_order_allocs: AtomicU64,
}

impl PageAllocStats {
    pub const fn new() -> Self {
        PageAllocStats {
            alloc_total: AtomicU64::new(0),
            free_total: AtomicU64::new(0),
            pcp_hits: AtomicU64::new(0),
            pcp_misses: AtomicU64::new(0),
            buddy_allocs: AtomicU64::new(0),
            alloc_fails: AtomicU64::new(0),
            order0_allocs: AtomicU64::new(0),
            high_order_allocs: AtomicU64::new(0),
        }
    }
}

/// Page allocator state
pub struct PageAllocator {
    /// Statistics
    pub stats: PageAllocStats,
    /// Per-CPU cache enabled
    pub pcp_enabled: AtomicBool,
    /// Reclaim on failure enabled
    pub reclaim_enabled: AtomicBool,
    /// PCP watermarks (adaptive)
    pub pcp_watermark: PcpWatermark,
    /// Initialized
    pub initialized: AtomicBool,
}

impl PageAllocator {
    pub const fn new() -> Self {
        PageAllocator {
            stats: PageAllocStats::new(),
            pcp_enabled: AtomicBool::new(true),
            reclaim_enabled: AtomicBool::new(true),
            pcp_watermark: PcpWatermark::default_watermarks(),
            initialized: AtomicBool::new(false),
        }
    }

    /// Initialize page allocator
    /// @param nr_cpus: Number of CPUs for Per-CPU cache
    /// @param totalram_pages: Total RAM pages for adaptive watermark computation
    pub fn init(&self, nr_cpus: u32, totalram_pages: u64) {
        if self.initialized.load(Ordering::Acquire) {
            return;
        }

        init_pcp_cache(nr_cpus);
        self.initialized.store(true, Ordering::Release);
    }

    /// Set PCP watermarks based on total RAM
    pub fn set_pcp_watermarks(&mut self, totalram_pages: u64) {
        self.pcp_watermark = PcpWatermark::from_totalram(totalram_pages);
    }

    /// Allocate a single physical page
    /// Tries Per-CPU cache first, then buddy allocator.
    /// @param flags: Allocation flags (GFP_*)
    /// @return Pointer to Page structure, or null on failure
    #[inline(always)]
    pub fn alloc_page(&self, flags: u32) -> *mut Page {
        self.alloc_pages(flags, 0)
    }

    /// Allocate 2^order contiguous physical pages
    /// @param flags: Allocation flags
    /// @param order: Power-of-two order (0 = single page)
    /// @return Pointer to first Page structure, or null on failure
    pub fn alloc_pages(&self, flags: u32, order: usize) -> *mut Page {
        self.stats.alloc_total.fetch_add(1, Ordering::Relaxed);

        if order == 0 {
            self.stats.order0_allocs.fetch_add(1, Ordering::Relaxed);
        } else {
            self.stats.high_order_allocs.fetch_add(1, Ordering::Relaxed);
        }

        let mut page: *mut Page = core::ptr::null_mut();

        if order == 0 && self.pcp_enabled.load(Ordering::Acquire) {
            page = pcp_alloc_pages(0);
            if !page.is_null() {
                self.stats.pcp_hits.fetch_add(1, Ordering::Relaxed);
                return page;
            }
            self.stats.pcp_misses.fetch_add(1, Ordering::Relaxed);
        }

        page = self.buddy_alloc(order);
        if !page.is_null() {
            self.stats.buddy_allocs.fetch_add(1, Ordering::Relaxed);

            if (flags & gfp_flags::GFP_ZERO) != 0 {
                self.zero_page(page, order);
            }

            return page;
        }

        if self.reclaim_enabled.load(Ordering::Acquire)
            && (flags & gfp_flags::GFP_NORETRY) == 0
        {
            if self.try_reclaim_and_alloc(order) {
                page = self.buddy_alloc(order);
                if !page.is_null() {
                    self.stats.buddy_allocs.fetch_add(1, Ordering::Relaxed);
                    if (flags & gfp_flags::GFP_ZERO) != 0 {
                        self.zero_page(page, order);
                    }
                    return page;
                }
            }
        }

        self.stats.alloc_fails.fetch_add(1, Ordering::Relaxed);
        core::ptr::null_mut()
    }

    /// Free a single physical page
    /// @param page: Pointer to Page structure
    #[inline(always)]
    pub fn free_page(&self, page: *mut Page) {
        self.free_pages(page, 0);
    }

    /// Free 2^order contiguous physical pages
    /// @param page: Pointer to first Page structure
    /// @param order: Power-of-two order
    pub fn free_pages(&self, page: *mut Page, order: usize) {
        if page.is_null() {
            return;
        }

        self.stats.free_total.fetch_add(1, Ordering::Relaxed);

        // SAFETY: page pointer validated by caller (from alloc)
        unsafe {
            let ref_count = (*page).ref_count.fetch_sub(1, Ordering::AcqRel);
            if ref_count > 1 {
                return;
            }
        }

        if order == 0 && self.pcp_enabled.load(Ordering::Acquire) {
            if pcp_free_pages(page, 0) {
                return;
            }
        }

        self.buddy_free(page, order);
    }

    /// Get page reference (increment ref count)
    #[inline(always)]
    pub fn get_page(&self, page: *mut Page) {
        if page.is_null() {
            return;
        }
        // SAFETY: caller ensures page is valid
        unsafe {
            (*page).ref_count.fetch_add(1, Ordering::AcqRel);
        }
    }

    /// Put page reference (decrement ref count, free if last)
    #[inline(always)]
    pub fn put_page(&self, page: *mut Page) {
        if page.is_null() {
            return;
        }
        // SAFETY: caller ensures page is valid
        unsafe {
            let old = (*page).ref_count.fetch_sub(1, Ordering::AcqRel);
            if old == 1 {
                self.free_page(page);
            }
        }
    }

    /// Allocate from buddy allocator
    fn buddy_alloc(&self, order: usize) -> *mut Page {
        super::alloc_pages(order)
    }

    /// Free to buddy allocator
    fn buddy_free(&self, page: *mut Page, order: usize) {
        super::free_pages(page, order);
    }

    /// Zero a page (or pages for order > 0)
    fn zero_page(&self, page: *mut Page, order: usize) {
        if page.is_null() {
            return;
        }

        let nr_pages = 1usize << order;
        let page_size = PAGE_SIZE as usize;

        // SAFETY: page is from allocator, valid for writing
        unsafe {
            for i in 0..nr_pages {
                let p = page.add(i);
                let vaddr = (*p).phys_addr + 0xFFFF_0000_0000_0000;
                let ptr = vaddr as *mut u8;
                core::ptr::write_bytes(ptr, 0u8, page_size);
            }
        }
    }

    /// Try reclaiming pages and retry allocation
    fn try_reclaim_and_alloc(&self, order: usize) -> bool {
        let needed = 1u64 << order;
        let free = super::nr_free_pages();

        if free < needed {
            if let Ok(reclaimed) = super::reclaim::reclaim_pages(needed as usize) {
                return reclaimed >= needed as usize;
            }
        }

        free >= needed
    }

    /// Get total free pages across all zones
    pub fn nr_free_pages(&self) -> u64 {
        super::nr_free_pages()
    }

    /// Get allocation statistics
    pub fn get_stats(&self) -> &PageAllocStats {
        &self.stats
    }
}

/// Global page allocator
static PAGE_ALLOCATOR: PageAllocator = PageAllocator::new();

/// Initialize page allocator
pub fn init_page_alloc(nr_cpus: u32, totalram_pages: u64) {
    PAGE_ALLOCATOR.init(nr_cpus, totalram_pages);
}

/// Allocate a physical page
#[inline(always)]
pub fn alloc_page() -> *mut Page {
    PAGE_ALLOCATOR.alloc_page(gfp_flags::GFP_KERNEL)
}

/// Allocate a physical page with flags
#[inline(always)]
pub fn alloc_page_flags(flags: u32) -> *mut Page {
    PAGE_ALLOCATOR.alloc_page(flags)
}

/// Allocate 2^order contiguous pages
#[inline(always)]
pub fn alloc_pages(order: usize) -> *mut Page {
    PAGE_ALLOCATOR.alloc_pages(gfp_flags::GFP_KERNEL, order)
}

/// Allocate 2^order contiguous pages with flags
#[inline(always)]
pub fn alloc_pages_flags(flags: u32, order: usize) -> *mut Page {
    PAGE_ALLOCATOR.alloc_pages(flags, order)
}

/// Free a physical page
#[inline(always)]
pub fn free_page(page: *mut Page) {
    PAGE_ALLOCATOR.free_page(page);
}

/// Free 2^order contiguous pages
#[inline(always)]
pub fn free_pages(page: *mut Page, order: usize) {
    PAGE_ALLOCATOR.free_pages(page, order);
}

/// Get page reference
#[inline(always)]
pub fn get_page_ref(page: *mut Page) {
    PAGE_ALLOCATOR.get_page(page);
}

/// Put page reference
#[inline(always)]
pub fn put_page_ref(page: *mut Page) {
    PAGE_ALLOCATOR.put_page(page);
}

/// Get page allocator statistics
pub fn get_page_alloc_stats() -> &'static PageAllocStats {
    PAGE_ALLOCATOR.get_stats()
}

/// Get number of free pages
pub fn nr_free_pages() -> u64 {
    PAGE_ALLOCATOR.nr_free_pages()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gfp_flags() {
        assert_eq!(gfp_flags::GFP_KERNEL, 0x00);
        assert_eq!(gfp_flags::GFP_ATOMIC, 0x01);
        assert_eq!(gfp_flags::GFP_DMA, 0x02);
        assert_eq!(gfp_flags::GFP_ZERO, 0x08);
    }

    #[test]
    fn test_page_alloc_stats_new() {
        let stats = PageAllocStats::new();
        assert_eq!(stats.alloc_total.load(Ordering::Relaxed), 0);
        assert_eq!(stats.free_total.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_page_allocator_new() {
        let alloc = PageAllocator::new();
        assert!(!alloc.initialized.load(Ordering::Relaxed));
        assert!(alloc.pcp_enabled.load(Ordering::Relaxed));
    }
}
