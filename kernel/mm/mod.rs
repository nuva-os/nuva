/*
 * Nuva OS - Kernel - Memory Management
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

// Memory management submodules
// Re-export print macros from crate root
pub use crate::{pr_emerg, pr_alert, pr_crit, pr_err, pr_warn, pr_notice, pr_info, pr_debug};
pub mod percpu_cache;
pub mod huge_page;
pub mod numa;
pub mod compaction;
pub mod api;
pub mod stats;
pub mod reclaim;
pub mod cow;
pub mod mmap;
pub mod oom;
pub mod mm_tests;
pub mod page_alloc;
pub mod mem_map;
pub mod page_table;
pub mod mempool_opt;
pub mod npu_mem;
pub mod region;
pub mod buddy;
pub mod address_space;
pub mod vma;
pub mod slab;
pub mod allocator;
pub mod fault;
pub mod memory;
pub mod complete_mem_map;
pub mod complete_features;
pub mod advanced_memory;
pub mod advanced_features;

// Re-export key types
pub use percpu_cache::{PerCpuPageCache, PerCpuPageCacheManager, init_pcp_cache as init_percpu_cache};
pub use huge_page::{HugePageSize, HugePagePool, ThpManager, init_huge_pages};
pub use numa::{NumaNode, NumaTopology, init_numa, cpu_to_node};
pub use compaction::{MemoryCompactor, CompactResult, init_memory_compaction};
pub use stats::{MemoryStats, MemoryMonitor, MemoryPressure, memory_monitor, init_memory_monitoring};
pub use reclaim::{PageReclaimer, ReclaimError, init_reclaimer, reclaim_pages};
pub use cow::{CowManager, CowEntry, CowError, init_cow, create_cow_page};
pub use region::{NvMemoryRegion, NvMemoryType};

// Re-export unified API
pub use api::{
    MemoryManager, GfpFlags,
    PhysicalMemoryAllocator, VirtualMemoryAllocator,
    SlabAllocatorOps, NumaMemoryOps,
    kmalloc, kfree,
};

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// Page size (4KB)
pub const PAGE_SIZE: u64 = 4096;

/// Page shift
pub const PAGE_SHIFT: u64 = 12;

/// Page mask
pub const PAGE_MASK: u64 = !(PAGE_SIZE - 1);

/// Physical address type
pub type PhysAddr = u64;

/// Virtual address type
pub type VirtAddr = u64;

/// Page flags
pub mod page_flags {
    pub const PG_PRESENT: u64 = 1 << 0;
    pub const PG_WRITABLE: u64 = 1 << 1;
    pub const PG_USER: u64 = 1 << 2;
    pub const PG_WRITETHROUGH: u64 = 1 << 3;
    pub const PG_CACHE_DISABLE: u64 = 1 << 4;
    pub const PG_ACCESSED: u64 = 1 << 5;
    pub const PG_DIRTY: u64 = 1 << 6;
    pub const PG_HUGE: u64 = 1 << 7;
    pub const PG_GLOBAL: u64 = 1 << 8;
    pub const PG_NO_EXECUTE: u64 = 1 << 63;
    
    // Page structure flags
    pub const PG_LOCKED: u32 = 0;
    pub const PG_ERROR: u32 = 1;
    pub const PG_REFERENCED: u32 = 2;
    pub const PG_UPTODATE: u32 = 3;
    pub const PG_DIRTY_FLAG: u32 = 4;
    pub const PG_LRU: u32 = 5;
    pub const PG_ACTIVE: u32 = 6;
    pub const PG_SLAB: u32 = 7;
    pub const PG_OWNER_PRIV_1: u32 = 8;
    pub const PG_ARCH_1: u32 = 9;
    pub const PG_RESERVED: u32 = 10;
    pub const PG_PRIVATE: u32 = 11;
    pub const PG_PRIVATE_2: u32 = 12;
    pub const PG_WRITEBACK: u32 = 13;
    pub const PG_HEAD: u32 = 14;
    pub const PG_SWAPCACHE: u32 = 15;
    pub const PG_SWAPBACKED: u32 = 16;
    pub const PG_UNEVICTABLE: u32 = 17;
    pub const PG_MLOCKED: u32 = 18;
    pub const PG_SWAP_USED: u32 = 19;
    pub const PG_UNCACHED: u32 = 20;
}

/// Memory zone types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZoneType {
    /// DMA zone (for ISA DMA)
    Dma = 0,
    /// DMA32 zone (for 32-bit DMA)
    Dma32 = 1,
    /// Normal zone
    Normal = 2,
    /// High memory zone
    HighMem = 3,
    /// Movable zone
    Movable = 4,
}

/// Page structure
/// One page structure per physical page frame.
pub struct Page {
    /// Page flags
    pub flags: AtomicU32,
    /// Reference count
    pub ref_count: AtomicU32,
    /// Map count
    pub map_count: AtomicU32,
    /// Physical address
    pub phys_addr: PhysAddr,
    /// Index in mem_map
    pub index: u64,
    /// LRU list pointers
    pub lru: [u64; 2],
    /// Private data
    pub private: u64,
    /// Mapping
    pub mapping: u64,
}

/// Handle page fault
pub fn handle_page_fault(_addr: u64, _flags: u32) -> i32 { 0 }

impl Page {
    /// Create new page structure
    pub fn new(phys_addr: PhysAddr, index: u64) -> Self {
        Page {
            flags: AtomicU32::new(0),
            ref_count: AtomicU32::new(1),
            map_count: AtomicU32::new(0),
            phys_addr,
            index,
            lru: [0; 2],
            private: 0,
            mapping: 0,
        }
    }
    
    /// Get page from physical address
    pub fn phys_to_page(phys: PhysAddr) -> *mut Page {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let base = MEM_MAP_BASE as *mut Page;
            let idx = phys / PAGE_SIZE;
            base.add(idx as usize)
        }
    }
    
    /// Get physical address from page
    pub fn page_to_phys(page: *const Page) -> PhysAddr {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe { (*page).phys_addr }
    }
    
    /// Get virtual address from page
    pub fn page_to_virt(page: *const Page) -> VirtAddr {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe { (*page).phys_addr + PAGE_OFFSET }
    }
    
    /// Increment reference count
    pub fn get_page(&self) {
        self.ref_count.fetch_add(1, Ordering::AcqRel);
    }
    
    /// Decrement reference count
    pub fn put_page(&self) -> u32 {
        self.ref_count.fetch_sub(1, Ordering::AcqRel)
    }
    
    /// Check if page is reserved
    pub fn is_reserved(&self) -> bool {
        (self.flags.load(Ordering::Acquire) & (1 << page_flags::PG_RESERVED)) != 0
    }
    
    /// Set page reserved
    pub fn set_reserved(&self) {
        self.flags.fetch_or(1 << page_flags::PG_RESERVED, Ordering::AcqRel);
    }
    
    /// Clear reserved flag
    pub fn clear_reserved(&self) {
        self.flags.fetch_and(!(1 << page_flags::PG_RESERVED), Ordering::AcqRel);
    }
}

/// Memory map base address (virtual)
static mut MEM_MAP_BASE: u64 = 0;

/// Page offset for kernel virtual address
const PAGE_OFFSET: u64 = 0xFFFF_0000_0000_0000;

/// Memory zone
pub struct Zone {
    /// Zone type
    pub zone_type: ZoneType,
    /// Zone name
    pub name: [u8; 16],
    /// Start pfn
    pub start_pfn: u64,
    /// End pfn
    pub end_pfn: u64,
    /// Number of pages
    pub nr_pages: u64,
    /// Number of free pages
    pub free_pages: AtomicU64,
    /// Number of managed pages
    pub managed_pages: AtomicU64,
    /// Watermarks
    pub watermarks: [AtomicU64; 3],
    /// Per-cpu pageset
    pub pageset: [PerCpuPageset; 16],
    /// Buddy allocator
    pub buddy: BuddyAllocator,
}

/// Watermark indices
pub mod watermark {
    pub const MIN: usize = 0;
    pub const LOW: usize = 1;
    pub const HIGH: usize = 2;
}

/// Per-CPU pageset
pub struct PerCpuPageset {
    /// Number of pages
    pub count: u32,
    /// High watermark
    pub high: u32,
    /// Batch size
    pub batch: u32,
    /// Page list
    pub list: [*mut Page; 128],
}

impl PerCpuPageset {
    pub const fn new() -> Self {
        PerCpuPageset {
            count: 0,
            high: 22,
            batch: 7,
            list: [core::ptr::null_mut(); 128],
        }
    }
}

/// Buddy allocator
pub struct BuddyAllocator {
    /// Free lists for each order
    pub free_lists: [FreeList; 11],
    /// Number of free pages
    pub nr_free: AtomicU64,
}

/// Maximum order (2^10 = 1024 pages = 4MB)
pub const MAX_ORDER: usize = 11;

/// Free list for a specific order
pub struct FreeList {
    /// List head
    pub head: *mut Page,
    /// List tail
    pub tail: *mut Page,
    /// Number of free blocks
    pub nr_free: AtomicU32,
}

impl FreeList {
    pub const fn new() -> Self {
        FreeList {
            head: core::ptr::null_mut(),
            tail: core::ptr::null_mut(),
            nr_free: AtomicU32::new(0),
        }
    }
    
    /// Add page to free list
    pub fn add(&mut self, page: *mut Page) {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            (*page).lru[0] = self.head as u64;
            (*page).lru[1] = 0;
            
            if !self.head.is_null() {
                (*self.head).lru[1] = page as u64;
            }
            self.head = page;
            
            if self.tail.is_null() {
                self.tail = page;
            }
            
            self.nr_free.fetch_add(1, Ordering::AcqRel);
        }
    }
    
    /// Remove page from free list
    pub fn remove(&mut self, page: *mut Page) {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let prev = (*page).lru[1] as *mut Page;
            let next = (*page).lru[0] as *mut Page;
            
            if !prev.is_null() {
                (*prev).lru[0] = next as u64;
            } else {
                self.head = next;
            }
            
            if !next.is_null() {
                (*next).lru[1] = prev as u64;
            } else {
                self.tail = prev;
            }
            
            self.nr_free.fetch_sub(1, Ordering::AcqRel);
        }
    }
}

impl BuddyAllocator {
    pub const fn new() -> Self {
        BuddyAllocator {
            free_lists: [const { FreeList::new() }; MAX_ORDER],
            nr_free: AtomicU64::new(0),
        }
    }
    
    /// Initialize buddy allocator
    pub fn init(&mut self, start_pfn: u64, end_pfn: u64) {
        let mut pfn = start_pfn;
        
        while pfn < end_pfn {
            // Find largest order that fits
            let mut order = MAX_ORDER - 1;
            while order > 0 {
                // Check if pfn is aligned for this order
                if pfn % (1 << order) == 0 && pfn + (1 << order) <= end_pfn {
                    break;
                }
                order -= 1;
            }
            
            // Add to free list
            let page = Page::phys_to_page(pfn * PAGE_SIZE);
            self.free_lists[order].add(page);
            
            pfn += 1 << order;
        }
        
        self.nr_free.store(end_pfn - start_pfn, Ordering::Release);
    }
    
    /// Allocate pages
    pub fn alloc_pages(&mut self, order: usize) -> *mut Page {
        if order >= MAX_ORDER {
            return core::ptr::null_mut();
        }
        
        // Find a free block of at least the requested order
        let mut current_order = order;
        while current_order < MAX_ORDER {
            if self.free_lists[current_order].nr_free.load(Ordering::Acquire) > 0 {
                break;
            }
            current_order += 1;
        }
        
        if current_order >= MAX_ORDER {
            return core::ptr::null_mut();
        }
        
        // Remove block from free list
        let page = self.free_lists[current_order].head;
        if page.is_null() {
            return core::ptr::null_mut();
        }
        
        self.free_lists[current_order].remove(page);
        
        // Split block if necessary
        let mut current = current_order;
        while current > order {
            current -= 1;
            
            // Get buddy of first half
            // SAFETY: unsafe block required for low-level memory or hardware access
            let buddy_pfn = unsafe { (*page).index + (1 << current) };
            let buddy = Page::phys_to_page(buddy_pfn * PAGE_SIZE);
            
            // Add buddy to free list
            self.free_lists[current].add(buddy);
        }
        
        self.nr_free.fetch_sub(1 << order, Ordering::AcqRel);
        
        page
    }
    
    /// Free pages
    pub fn free_pages(&mut self, page: *mut Page, order: usize) {
        if page.is_null() || order >= MAX_ORDER {
            return;
        }
        
        let mut current_order = order;
        let mut current_page = page;
        
        // Try to merge with buddy
        while current_order < MAX_ORDER - 1 {
            // Find buddy
            // SAFETY: unsafe block required for low-level memory or hardware access
            let pfn = unsafe { (*current_page).index };
            let buddy_pfn = pfn ^ (1 << current_order);
            let buddy = Page::phys_to_page(buddy_pfn * PAGE_SIZE);
            
            // Check if buddy is free
            if !self.is_buddy_free(buddy, current_order) {
                break;
            }
            
            // Remove buddy from free list
            self.free_lists[current_order].remove(buddy);
            
            // Merge: use lower pfn as new page
            current_page = if buddy_pfn < pfn { buddy } else { current_page };
            current_order += 1;
        }
        
        // Add merged block to free list
        self.free_lists[current_order].add(current_page);
        self.nr_free.fetch_add(1 << order, Ordering::AcqRel);
    }
    
    /// Check if buddy is free
    fn is_buddy_free(&self, buddy: *mut Page, order: usize) -> bool {
        if buddy.is_null() {
            return false;
        }
        
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            // Check if buddy is in free list
            let ref_count = (*buddy).ref_count.load(Ordering::Acquire);
            let flags = (*buddy).flags.load(Ordering::Acquire);
            
            ref_count == 0 && (flags & (1 << page_flags::PG_LRU)) != 0
        }
    }
}

/// Physical memory manager
pub struct PhysMemManager {
    /// Total memory (bytes)
    pub total_memory: u64,
    /// Total pages
    pub total_pages: u64,
    /// Free pages
    pub free_pages: AtomicU64,
    /// Memory zones
    pub zones: [Option<ZoneType>; 5],
    /// mem_map array
    pub mem_map: *mut Page,
    /// Number of zones
    pub nr_zones: u32,
}

impl PhysMemManager {
    pub const fn new() -> Self {
        PhysMemManager {
            total_memory: 0,
            total_pages: 0,
            free_pages: AtomicU64::new(0),
            zones: [None, None, None, None, None],
            mem_map: core::ptr::null_mut(),
            nr_zones: 0,
        }
    }
    
    /// Initialize physical memory manager
    pub fn init(&mut self, total_memory: u64) {
        self.total_memory = total_memory;
        self.total_pages = total_memory / PAGE_SIZE;
        
        log_info!("Physical memory: {} MB", total_memory / (1024 * 1024));
        log_info!("Total pages: {}", self.total_pages);
        
        // Initialize mem_map array
        self.init_mem_map();
        
        // Initialize zones
        self.init_zones();
    }
    
    /// Initialize mem_map array
    fn init_mem_map(&mut self) {
        // mem_map should be at a known virtual address
        // For now, use a static buffer
        static mut MEM_MAP: [Page; 262144] = [const { Page {
            flags: AtomicU32::new(0),
            ref_count: AtomicU32::new(0),
            map_count: AtomicU32::new(0),
            phys_addr: 0,
            index: 0,
            lru: [0; 2],
            private: 0,
            mapping: 0,
        } }; 262144];  /* 1GB worth of pages */
        
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            self.mem_map = MEM_MAP.as_mut_ptr();
            MEM_MAP_BASE = self.mem_map as u64;
            
            // Initialize each page
            for i in 0..self.total_pages as usize {
                MEM_MAP[i].phys_addr = (i as u64) * PAGE_SIZE;
                MEM_MAP[i].index = i as u64;
                MEM_MAP[i].ref_count.store(0, Ordering::Release);
            }

            // Register with mem_map subsystem
            mem_map::init_mem_map(
                0,
                self.total_pages,
                MEM_MAP.as_mut_ptr() as u64,
            );
        }
    }
    
    /// Initialize memory zones
    fn init_zones(&mut self) {
        // Create normal zone
        let mut zone = Zone {
            zone_type: ZoneType::Normal,
            name: *b"Normal\0\0\0\0\0\0\0\0\0\0",
            start_pfn: 0,
            end_pfn: self.total_pages,
            nr_pages: self.total_pages,
            free_pages: AtomicU64::new(self.total_pages),
            managed_pages: AtomicU64::new(self.total_pages),
            watermarks: [
                AtomicU64::new(self.total_pages / 256),
                AtomicU64::new(self.total_pages / 128),
                AtomicU64::new(self.total_pages / 64),
            ],
            pageset: core::array::from_fn(|_| PerCpuPageset::new()),
            buddy: BuddyAllocator::new(),
        };
        
        // Initialize buddy allocator
        zone.buddy.init(0, self.total_pages);
        
        self.zones[ZoneType::Normal as usize] = Some(zone);
        self.nr_zones = 1;
        
        self.free_pages.store(self.total_pages, Ordering::Release);
    }
    
    /// Allocate pages
    pub fn alloc_pages(&mut self, order: usize) -> *mut Page {
        // Try each zone
        for i in 0..self.nr_zones as usize {
            if let Some(ref mut zone) = self.zones[i] {
                let page = zone.buddy.alloc_pages(order);
                if !page.is_null() {
                    self.free_pages.fetch_sub(1 << order, Ordering::AcqRel);
                    zone.free_pages.fetch_sub(1 << order, Ordering::AcqRel);
                    
                    // SAFETY: unsafe block required for low-level memory or hardware access
                    unsafe {
                        (*page).ref_count.store(1, Ordering::Release);
                    }
                    
                    return page;
                }
            }
        }
        
        core::ptr::null_mut()
    }
    
    /// Free pages
    pub fn free_pages(&mut self, page: *mut Page, order: usize) {
        if page.is_null() {
            return;
        }
        
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let ref_count = (*page).ref_count.fetch_sub(1, Ordering::AcqRel);
            if ref_count > 1 {
                return;
            }
            
            // Find zone
            let pfn = (*page).index;
            for i in 0..self.nr_zones as usize {
                if let Some(ref mut zone) = self.zones[i] {
                    if pfn >= zone.start_pfn && pfn < zone.end_pfn {
                        zone.buddy.free_pages(page, order);
                        zone.free_pages.fetch_add(1 << order, Ordering::AcqRel);
                        self.free_pages.fetch_add(1 << order, Ordering::AcqRel);
                        break;
                    }
                }
            }
        }
    }
    
    /// Get free page count
    pub fn get_free_pages(&self) -> u64 {
        self.free_pages.load(Ordering::Acquire)
    }
    
    /// Get total page count
    pub fn get_total_pages(&self) -> u64 {
        self.total_pages
    }
    
    /// Print memory statistics
    pub fn print_stats(&self) {
        log_info!("Memory Statistics:");
        log_info!("  Total: {} MB", self.total_memory / (1024 * 1024));
        log_info!("  Total pages: {}", self.total_pages);
        log_info!("  Free pages: {}", self.free_pages.load(Ordering::Acquire));
        log_info!("  Used: {} MB", 
                 (self.total_pages - self.free_pages.load(Ordering::Acquire)) * PAGE_SIZE / (1024 * 1024));
    }
}

/// Global physical memory manager
static PHYS_MEM_MANAGER: crate::sync_oncelock::OnceLock<PhysMemManager> = crate::sync_oncelock::OnceLock::new();

/// Get physical memory manager
pub fn phys_mem_manager() -> &'static PhysMemManager {
    PHYS_MEM_MANAGER.get_or_init(PhysMemManager::new)
}

pub fn init_phys_mem_manager() -> &'static PhysMemManager {
    PHYS_MEM_MANAGER.get_or_init(PhysMemManager::new)
}

/// Initialize physical memory
pub fn init_phys_mem(total_memory: u64) {
    let mgr = phys_mem_manager();
    mgr.init(total_memory);
}

/// Initialize buddy allocator
pub fn init_buddy(nr_pages: u32) {
    let _ = nr_pages;
    // Already initialized in init_phys_mem
}

/// Allocate pages
pub fn alloc_pages(order: usize) -> *mut Page {
    phys_mem_manager().alloc_pages(order)
}

/// Free pages
pub fn free_pages(page: *mut Page, order: usize) {
    phys_mem_manager().free_pages(page, order);
}

/// Allocate a single page
pub fn alloc_page() -> *mut Page {
    alloc_pages(0)
}

/// Free a single page
pub fn free_page(page: *mut Page) {
    free_pages(page, 0);
}

/// Get free page count
pub fn nr_free_pages() -> u64 {
    phys_mem_manager().get_free_pages()
}

/// Convert physical address to virtual address
pub fn phys_to_virt(phys: PhysAddr) -> VirtAddr {
    phys + PAGE_OFFSET
}

/// Convert virtual address to physical address
pub fn virt_to_phys(virt: VirtAddr) -> PhysAddr {
    virt - PAGE_OFFSET
}

/// Convert physical address to page
pub fn phys_to_page(phys: PhysAddr) -> *mut Page {
    Page::phys_to_page(phys)
}

/// Convert page to physical address
pub fn page_to_phys(page: *const Page) -> PhysAddr {
    Page::page_to_phys(page)
}
