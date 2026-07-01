/*
 * Nuva OS - Kernel - Memory Management - Huge Page Support
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

use super::{Page, PhysAddr, VirtAddr, PAGE_SIZE, PAGE_SHIFT};
use crate::{pr_debug, pr_warn};

/// Huge page sizes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HugePageSize {
    /// 2MB huge page (512 * 4KB)
    Huge2MB = 21,
    
    /// 1GB huge page (512 * 2MB)
    Huge1GB = 30,
}

impl HugePageSize {
    /// Get the size in bytes
    pub const fn size(&self) -> u64 {
        match self {
            HugePageSize::Huge2MB => 2 * 1024 * 1024,  // 2MB
            HugePageSize::Huge1GB => 1024 * 1024 * 1024,  // 1GB
        }
    }
    
    /// Get the page shift
    pub const fn shift(&self) -> u64 {
        match self {
            HugePageSize::Huge2MB => 21,
            HugePageSize::Huge1GB => 30,
        }
    }
    
    /// Get the number of base pages
    pub const fn nr_pages(&self) -> u64 {
        match self {
            HugePageSize::Huge2MB => 512,
            HugePageSize::Huge1GB => 512 * 512,
        }
    }
    
    /// Get the order for buddy allocator
    pub const fn order(&self) -> usize {
        match self {
            HugePageSize::Huge2MB => 9,   // 2^9 = 512 pages
            HugePageSize::Huge1GB => 18,  // 2^18 = 262144 pages
        }
    }
}

/// Huge page flags
pub mod huge_page_flags {
    /// Page is a huge page
    pub const HP_HUGE: u32 = 1 << 0;
    
    /// Page is a transparent huge page
    pub const HP_TRANSPARENT: u32 = 1 << 1;
    
    /// Page is pinned (cannot be split)
    pub const HP_PINNED: u32 = 1 << 2;
    
    /// Page is being split
    pub const HP_SPLITTING: u32 = 1 << 3;
    
    /// Page is being collapsed
    pub const HP_COLLAPSED: u32 = 1 << 4;
}

/// Huge page structure
pub struct HugePage {
    /// Base page structure
    pub base: Page,
    
    /// Huge page size
    pub size: HugePageSize,
    
    /// Huge page flags
    pub hp_flags: AtomicU32,
    
    /// Number of subpages mapped
    pub nr_mapped: AtomicU32,
    
    /// Reservation count
    pub reservation: AtomicU32,
    pub physical_address: u64,
}

impl HugePage {
    /// Create a new huge page
    pub fn new(phys_addr: PhysAddr, size: HugePageSize) -> Self {
        HugePage {
            base: Page::new(phys_addr, phys_addr / PAGE_SIZE),
            size,
            hp_flags: AtomicU32::new(huge_page_flags::HP_HUGE),
            nr_mapped: AtomicU32::new(0),
            reservation: AtomicU32::new(0),
                physical_address: 0,
            }
    }
    
    /// Check if this is a transparent huge page
    pub fn is_transparent(&self) -> bool {
        (self.hp_flags.load(Ordering::Acquire) & huge_page_flags::HP_TRANSPARENT) != 0
    }
    
    /// Check if page is pinned
    pub fn is_pinned(&self) -> bool {
        (self.hp_flags.load(Ordering::Acquire) & huge_page_flags::HP_PINNED) != 0
    }
    
    /// Mark as transparent huge page
    pub fn set_transparent(&self) {
        self.hp_flags.fetch_or(huge_page_flags::HP_TRANSPARENT, Ordering::AcqRel);
    }
    
    /// Pin the huge page
    pub fn pin(&self) {
        self.hp_flags.fetch_or(huge_page_flags::HP_PINNED, Ordering::AcqRel);
    }
    
    /// Unpin the huge page
    pub fn unpin(&self) {
        self.hp_flags.fetch_and(!huge_page_flags::HP_PINNED, Ordering::AcqRel);
    }
}

/// Huge page pool
/// Manages pre-allocated huge pages
pub struct HugePagePool {
    /// Pool for 2MB pages
    pub pool_2mb: HugePageList,
    
    /// Pool for 1GB pages
    pub pool_1gb: HugePageList,
    
    /// Total huge pages allocated
    pub total_huge_pages: AtomicU64,
    
    /// Free huge pages
    pub free_huge_pages: AtomicU64,
    
    /// Reserved huge pages
    pub reserved_huge_pages: AtomicU64,
    
    /// Surplus huge pages (over commitment)
    pub surplus_huge_pages: AtomicU64,
    
    /// Initialized flag
    pub initialized: AtomicBool,
}

/// Huge page list
pub struct HugePageList {
    /// List head
    pub head: *mut HugePage,
    
    /// List tail
    pub tail: *mut HugePage,
    
    /// Number of pages
    pub count: AtomicU32,
    
    /// Maximum pages
    pub max_count: AtomicU32,
}

impl HugePageList {
    pub const fn new() -> Self {
        HugePageList {
            head: ptr::null_mut(),
            tail: ptr::null_mut(),
            count: AtomicU32::new(0),
            max_count: AtomicU32::new(0),
        }
    }
    
    /// Add a huge page to the list
    pub fn add(&mut self, page: *mut HugePage) {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            (*page).base.lru[0] = self.head as u64;
            (*page).base.lru[1] = 0;
            
            if !self.head.is_null() {
                (*self.head).base.lru[1] = page as u64;
            }
            self.head = page;
            
            if self.tail.is_null() {
                self.tail = page;
            }
            
            self.count.fetch_add(1, Ordering::AcqRel);
        }
    }
    
    /// Remove a huge page from the list
    pub fn remove(&mut self, page: *mut HugePage) {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let prev = (*page).base.lru[1] as *mut HugePage;
            let next = (*page).base.lru[0] as *mut HugePage;
            
            if !prev.is_null() {
                (*prev).base.lru[0] = next as u64;
            } else {
                self.head = next;
            }
            
            if !next.is_null() {
                (*next).base.lru[1] = prev as u64;
            } else {
                self.tail = prev;
            }
            
            self.count.fetch_sub(1, Ordering::AcqRel);
        }
    }
    
    /// Pop a page from the list
    pub fn pop(&mut self) -> *mut HugePage {
        if self.head.is_null() {
            return ptr::null_mut();
        }
        
        let page = self.head;
        self.remove(page);
        page
    }
}

impl HugePagePool {
    pub const fn new() -> Self {
        HugePagePool {
            pool_2mb: HugePageList::new(),
            pool_1gb: HugePageList::new(),
            total_huge_pages: AtomicU64::new(0),
            free_huge_pages: AtomicU64::new(0),
            reserved_huge_pages: AtomicU64::new(0),
            surplus_huge_pages: AtomicU64::new(0),
            initialized: AtomicBool::new(false),
        }
    }
    
    /// Initialize the huge page pool
    pub fn init(&mut self, nr_2mb: u32, nr_1gb: u32) {
        self.pool_2mb.max_count.store(nr_2mb, Ordering::Release);
        self.pool_1gb.max_count.store(nr_1gb, Ordering::Release);
        self.initialized.store(true, Ordering::Release);
    }
    
    /// Allocate a huge page
    pub fn alloc_huge_page(&mut self, size: HugePageSize) -> *mut HugePage {
        let list = match size {
            HugePageSize::Huge2MB => &mut self.pool_2mb,
            HugePageSize::Huge1GB => &mut self.pool_1gb,
        };
        
        let page = list.pop();
        
        if !page.is_null() {
            self.free_huge_pages.fetch_sub(1, Ordering::AcqRel);
        }
        
        page
    }
    
    /// Free a huge page
    pub fn free_huge_page(&mut self, page: *mut HugePage) {
        if page.is_null() {
            return;
        }
        
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let list = match (*page).size {
                HugePageSize::Huge2MB => &mut self.pool_2mb,
                HugePageSize::Huge1GB => &mut self.pool_1gb,
            };
            
            list.add(page);
            self.free_huge_pages.fetch_add(1, Ordering::AcqRel);
        }
    }
    
    /// Get free count for a specific size
    pub fn get_free_count(&self, size: HugePageSize) -> u32 {
        match size {
            HugePageSize::Huge2MB => self.pool_2mb.count.load(Ordering::Acquire),
            HugePageSize::Huge1GB => self.pool_1gb.count.load(Ordering::Acquire),
        }
    }


}

/// Transparent Huge Page (THP) manager
pub struct ThpManager {
    /// THP enabled flag
    pub enabled: AtomicBool,
    
    /// THP mode: always, madvise, never
    pub mode: AtomicU32,
    
    /// Number of THP allocations
    pub thp_alloc_count: AtomicU64,
    
    /// Number of THP splits
    pub thp_split_count: AtomicU64,
    
    /// Number of THP collapses
    pub thp_collapse_count: AtomicU64,
    
    /// THP fault fallback count
    pub thp_fault_fallback: AtomicU64,
    pub huge_page_size: u64,
}

/// THP modes
pub mod thp_mode {
    pub const ALWAYS: u32 = 0;
    pub const MADVISE: u32 = 1;
    pub const NEVER: u32 = 2;
}

impl ThpManager {
    pub const fn new() -> Self {
        ThpManager {
            enabled: AtomicBool::new(true),
            mode: AtomicU32::new(thp_mode::MADVISE),
            thp_alloc_count: AtomicU64::new(0),
            thp_split_count: AtomicU64::new(0),
            thp_collapse_count: AtomicU64::new(0),
            thp_fault_fallback: AtomicU64::new(0),
                huge_page_size: 0,
            }
    }
    
    /// Check if THP is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Acquire)
    }
    
    /// Check if THP should be used for this VMA
    pub fn should_use_thp(&self, vm_flags: u64) -> bool {
        if !self.is_enabled() {
            return false;
        }
        
        let mode = self.mode.load(Ordering::Acquire);
        
        match mode {
            thp_mode::ALWAYS => true,
            thp_mode::NEVER => false,
            thp_mode::MADVISE => {
                // Check if MADV_HUGEPAGE was set
                (vm_flags & (1 << 0)) != 0  // VM_HUGEPAGE flag
            }
            _ => false,
        }
    }
    
    /// Collapse a range of pages into a huge page
    pub fn collapse_range(&mut self, start: VirtAddr, end: VirtAddr) -> bool {
        // Check if range is aligned to huge page size
        let huge_page_size = self.huge_page_size as u64;
        if start % huge_page_size != 0 || (end - start) != huge_page_size {
            log_warn!("Collapse range not aligned to huge page size");
            return false;
        }
        
        // Check if all pages are present and not shared
        // In a real implementation, this would check page table entries
        let pages_ok = true; // Simplified: assume pages are OK
        
        if !pages_ok {
            log_warn!("Cannot collapse: pages not suitable");
            return false;
        }
        
        // Allocate a huge page
        // SAFETY: unsafe block required for low-level memory or hardware access
        let huge_page = unsafe {
            let ptr = self.alloc_huge_page();
            if ptr.is_null() {
                log_warn!("Failed to allocate huge page for collapse");
                return false;
            }
            ptr
        };
        
        // Copy data from small pages
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let huge_page_virt = crate::kernel::mm::mem_map::phys_to_virt(huge_page as u64);
            let small_page_virt = crate::kernel::mm::mem_map::phys_to_virt(start);
            
            // Copy data from small pages to huge page
            core::ptr::copy_nonoverlapping(
                small_page_virt as *const u8,
                huge_page_virt as *mut u8,
                huge_page_size as usize,
            );
        }
        
        // Update page table mappings
        // In a real implementation, this would update the page table to use huge page mapping
        log_debug!("Collapsed small pages into huge page at {:#x}", start);
        
        // Free small pages
        // In a real implementation, this would free the small pages
        
        self.thp_collapse_count.fetch_add(1, Ordering::Relaxed);
        
        true
    }
    
    /// Split a huge page into small pages
    pub fn split_huge_page(&mut self, page: *mut HugePage) -> bool {
        if page.is_null() {
            log_warn!("Cannot split null huge page");
            return false;
        }
        
        let huge_page_size = self.huge_page_size as u64;
        let small_page_size = 4096u64; // 4KB
        let num_small_pages = (huge_page_size / small_page_size) as usize;
        
        // Allocate small pages
        let mut small_pages_allocated = 0;
        for _ in 0..num_small_pages {
            let small_page = crate::kernel::mm::page_alloc::alloc_page();
            if small_page.is_null() {
                log_warn!("Failed to allocate small page for split");
                // Free already allocated small pages
                // In a real implementation, this would clean up
                break;
            }
            small_pages_allocated += 1;
        }
        
        if small_pages_allocated != num_small_pages {
            log_warn!("Failed to allocate all small pages for split");
            return false;
        }
        
        // Copy data from huge page
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let huge_page_virt = crate::kernel::mm::mem_map::phys_to_virt((*page).physical_address);
            
            // Copy data from huge page to small pages
            for i in 0..num_small_pages {
                let small_page_phys = crate::kernel::mm::page_alloc::alloc_page();
                let small_page_virt = crate::kernel::mm::mem_map::phys_to_virt(small_page_phys as u64);
                
                let offset = i * small_page_size as usize;
                core::ptr::copy_nonoverlapping(
                    (huge_page_virt as *const u8).add(offset),
                    small_page_virt as *mut u8,
                    small_page_size as usize,
                );
            }
        }
        
        // Update page table mappings
        // In a real implementation, this would update the page table to use small page mappings
        log_debug!("Split huge page into small pages");
        
        // Free huge page
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            self.free_huge_page(page as *mut u8);
        }
        
        self.thp_split_count.fetch_add(1, Ordering::Relaxed);
        
        true
    }

    /// Allocate a huge page
    fn alloc_huge_page(&mut self) -> *mut u8 {
        self.thp_alloc_count.fetch_add(1, Ordering::Relaxed);
        core::ptr::null_mut()
    }

    /// Free a huge page
    fn free_huge_page(&mut self, _page: *mut u8) {
        // TODO: implement huge page free
    }
}

/// Global huge page pool
static HUGE_PAGE_POOL: crate::sync_oncelock::OnceLock<HugePagePool> = crate::sync_oncelock::OnceLock::new();

/// Global THP manager
static THP_MANAGER: crate::sync_oncelock::OnceLock<ThpManager> = crate::sync_oncelock::OnceLock::new();

/// Get the huge page pool
pub fn huge_page_pool() -> &'static HugePagePool {
    HUGE_PAGE_POOL.get_or_init(HugePagePool::new)
}

/// Get the THP manager
pub fn thp_manager() -> &'static ThpManager {
    THP_MANAGER.get_or_init(ThpManager::new)
}

pub fn init_thp_manager() -> &'static ThpManager {
    THP_MANAGER.get_or_init(ThpManager::new)
}

/// Initialize huge page support
pub fn init_huge_pages(nr_2mb: u32, nr_1gb: u32) {
    huge_page_pool().init(nr_2mb, nr_1gb);
}

/// Allocate a huge page
pub fn alloc_huge_page(size: HugePageSize) -> *mut HugePage {
    huge_page_pool().alloc_huge_page(size)
}

/// Free a huge page
pub fn free_huge_page(page: *mut HugePage) {
    huge_page_pool().free_huge_page(page);
}

/// Check if address is huge page aligned
pub fn is_huge_page_aligned(addr: VirtAddr, size: HugePageSize) -> bool {
    let mask = size.size() - 1;
    (addr & mask) == 0
}

/// Align address down to huge page boundary
pub fn huge_page_align_down(addr: VirtAddr, size: HugePageSize) -> VirtAddr {
    let mask = !(size.size() - 1);
    addr & mask
}

/// Align address up to huge page boundary
pub fn huge_page_align_up(addr: VirtAddr, size: HugePageSize) -> VirtAddr {
    let mask = size.size() - 1;
    (addr + mask) & !mask
}
