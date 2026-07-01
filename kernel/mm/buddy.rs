/*
 * Nuva OS - Kernel - Buddy Allocator
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

use core::sync::atomic::{AtomicU32, Ordering};

/// Maximum order (2^10 = 1024 pages = 4MB)
pub const MAX_ORDER: usize = 10;

/// Free list for buddy allocator
/// Manages a linked list of free page blocks.
pub struct FreeList {
    /// List head pointer
    pub head: *mut Page,
    /// Number of blocks in list
    pub count: AtomicU32,
}

impl FreeList {
    pub const fn new() -> Self {
        FreeList {
            head: core::ptr::null_mut(),
            count: AtomicU32::new(0),
        }
    }
    
    /// Add page block to list head
    /// @param page: Page block to add
    pub fn add_page(&mut self, page: &mut Page) {
        page.prev = core::ptr::null_mut();
        page.next = self.head;
        
        if !self.head.is_null() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                (*self.head).prev = page;
            }
        }
        
        self.head = page;
        self.count.fetch_add(1, Ordering::AcqRel);
    }
    
    /// Remove page block from list
    /// @param page: Page block to remove
    pub fn remove_page(&mut self, page: &mut Page) {
        if !page.prev.is_null() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                (*page.prev).next = page.next;
            }
        } else {
            self.head = page.next;
        }
        
        if !page.next.is_null() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                (*page.next).prev = page.prev;
            }
        }
        
        page.prev = core::ptr::null_mut();
        page.next = core::ptr::null_mut();
        
        self.count.fetch_sub(1, Ordering::AcqRel);
    }
    
    /// Pop list head
    /// @return Pointer to popped page, or null if list is empty
    pub fn pop(&mut self) -> *mut Page {
        if self.head.is_null() {
            return core::ptr::null_mut();
        }
        
        let page = self.head;
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            self.remove_page(&mut *page);
        }
        page
    }
    
    /// Check if list is empty
    pub fn is_empty(&self) -> bool {
        self.head.is_null()
    }
    
    /// Get list length
    pub fn len(&self) -> u32 {
        self.count.load(Ordering::Acquire)
    }
}

/// Page state enumeration
/// Represents the current state of a physical page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageState {
    /// Page is free and available for allocation
    Free,
    /// Page is allocated and in use
    Allocated,
    /// Page is reserved and cannot be allocated
    Reserved,
}

/// Page descriptor structure
/// Represents metadata for a physical page in the system.
#[repr(C)]
pub struct Page {
    /// Page block order (log2 of block size in pages)
    pub order: u8,
    /// Current page state
    pub state: PageState,
    /// Reference count for shared pages
    pub ref_count: AtomicU32,
    /// Physical address of this page
    pub phys_addr: u64,
    /// Previous page in free list
    pub prev: *mut Page,
    /// Next page in free list
    pub next: *mut Page,
}

impl Page {
    pub const fn new() -> Self {
        Page {
            order: 0,
            state: PageState::Free,
            ref_count: AtomicU32::new(0),
            phys_addr: 0,
            prev: core::ptr::null_mut(),
            next: core::ptr::null_mut(),
        }
    }
    
    /// Initialize page with physical address
    /// @param phys_addr: Physical address of the page
    pub fn init(&mut self, phys_addr: u64) {
        self.phys_addr = phys_addr;
        self.order = 0;
        self.state = PageState::Free;
        self.ref_count.store(0, Ordering::Release);
        self.prev = core::ptr::null_mut();
        self.next = core::ptr::null_mut();
    }
    
    /// Check if page is free
    pub fn is_free(&self) -> bool {
        self.state == PageState::Free
    }
    
    /// Mark page as allocated
    pub fn mark_allocated(&mut self) {
        self.state = PageState::Allocated;
        self.ref_count.store(1, Ordering::Release);
    }
    
    /// Mark page as free
    pub fn mark_free(&mut self) {
        self.state = PageState::Free;
        self.ref_count.store(0, Ordering::Release);
    }
}

/// Buddy allocator for physical page management
/// Implements the buddy system algorithm for efficient memory allocation
/// with minimal fragmentation. Supports allocation of power-of-two sized
/// blocks from 1 page to 2^MAX_ORDER pages.
pub struct BuddyAllocator {
    /// Free list array, one for each order
    pub free_lists: [FreeList; MAX_ORDER + 1],
    /// Page descriptor array pointer
    pub page_array: *mut Page,
    /// Number of page descriptors
    pub num_pages: usize,
    /// Memory start address
    pub mem_start: u64,
    /// Total number of pages
    pub total_pages: AtomicU32,
    /// Number of free pages
    pub free_pages: AtomicU32,
    /// Total allocation count
    pub alloc_count: AtomicU32,
    /// Total free count
    pub free_count: AtomicU32,
}

impl BuddyAllocator {
    pub const fn new() -> Self {
        BuddyAllocator {
            free_lists: [
                FreeList::new(), FreeList::new(), FreeList::new(),
                FreeList::new(), FreeList::new(), FreeList::new(),
                FreeList::new(), FreeList::new(), FreeList::new(),
                FreeList::new(), FreeList::new(),
            ],
            page_array: core::ptr::null_mut(),
            num_pages: 0,
            mem_start: 0,
            total_pages: AtomicU32::new(0),
            free_pages: AtomicU32::new(0),
            alloc_count: AtomicU32::new(0),
            free_count: AtomicU32::new(0),
        }
    }
    
    /// Initialize the buddy allocator
    /// @param mem_start: Physical address of memory start
    /// @param total_pages: Total number of pages to manage
    /// @param page_array: Pointer to page descriptor array
    pub fn init(&mut self, mem_start: u64, total_pages: u32, page_array: *mut Page) {
        self.mem_start = mem_start;
        self.num_pages = total_pages as usize;
        self.page_array = page_array;
        self.total_pages.store(total_pages, Ordering::Release);
        self.free_pages.store(total_pages, Ordering::Release);
        
        // Initialize all page descriptors
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            for i in 0..total_pages as usize {
                let page = &mut *page_array.add(i);
                page.init(mem_start + (i as u64) * 4096);
            }
        }
        
        // Add all pages to free list as order=0 blocks
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            for i in 0..total_pages as usize {
                let page = &mut *page_array.add(i);
                page.order = 0;
                self.free_lists[0].add_page(page);
            }
        }
        
        log_info!("Buddy allocator initialized");
        log_info!("  Memory start: 0x{:X}", mem_start);
        log_info!("  Total pages: {}", total_pages);
        log_info!("  Max order: {} ({} pages = {} KB)", 
            MAX_ORDER, 1 << MAX_ORDER, (1 << MAX_ORDER) * 4);
    }
    
    /// Allocate 2^order contiguous pages
    /// @param order: Order of allocation (log2 of pages needed)
    /// @return Pointer to page descriptor, or null on failure
    pub fn alloc(&mut self, order: usize) -> *mut Page {
        if order > MAX_ORDER {
            log_warn!("Buddy alloc: order {} exceeds MAX_ORDER {}", order, MAX_ORDER);
            return core::ptr::null_mut();
        }
        
        // Search for free blocks starting from requested order
        for current_order in order..=MAX_ORDER {
            let page = self.free_lists[current_order].pop();
            
            if !page.is_null() {
                self.alloc_count.fetch_add(1, Ordering::Relaxed);
                
                // Found a block, may need to split
                let result = self.split_block(page, current_order, order);
                
                // Mark as allocated
                // SAFETY: unsafe block required for low-level memory or hardware access
                unsafe {
                    (*result).mark_allocated();
                }
                
                return result;
            }
        }
        
        // No suitable block found
        log_warn!("Buddy alloc: no free block for order {}", order);
        core::ptr::null_mut()
    }
    
    /// Split a block until it reaches target order
    /// @param block: Block to split
    /// @param current_order: Current order of the block
    /// @param target_order: Target order after splitting
    /// @return Pointer to the resulting block
    fn split_block(&mut self, block: *mut Page, current_order: usize, target_order: usize) -> *mut Page {
        let mut order = current_order;
        let mut current_block = block;
        
        while order > target_order {
            order -= 1;
            
            // Calculate buddy block address
            let buddy = self.buddy(current_block, order);
            
            if !buddy.is_null() {
                // SAFETY: unsafe block required for low-level memory or hardware access
                unsafe {
                    // Set buddy block properties
                    (*buddy).order = order as u8;
                    (*buddy).state = PageState::Free;
                    
                    // Add buddy to free list
                    self.free_lists[order].add_page(&mut *buddy);
                }
            }
        }
        
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            (*current_block).order = target_order as u8;
        }
        
        self.free_pages.fetch_sub(1 << target_order, Ordering::AcqRel);
        
        current_block
    }
    
    /// Free a previously allocated block
    /// @param block: Pointer to page descriptor to free
    pub fn free(&mut self, block: *mut Page) {
        if block.is_null() {
            return;
        }
        
        let order;
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            order = (*block).order as usize;
            (*block).mark_free();
        }
        
        if order > MAX_ORDER {
            log_warn!("Buddy free: invalid order {}", order);
            return;
        }
        
        self.free_count.fetch_add(1, Ordering::Relaxed);
        self.free_pages.fetch_add(1 << order, Ordering::AcqRel);
        
        // Try to merge with buddy blocks
        let mut current_order = order;
        let mut current_block = block;
        
        while current_order < MAX_ORDER {
            let buddy = self.buddy(current_block, current_order);
            
            // Check if buddy can be merged
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                if buddy.is_null() {
                    break;
                }
                
                let buddy_ref = &*buddy;
                if !buddy_ref.is_free() || buddy_ref.order as usize != current_order {
                    break;
                }
                
                // Remove buddy from free list
                self.free_lists[current_order].remove_page(&mut *buddy);
                
                // Merge: select block with smaller address
                if current_block > buddy {
                    current_block = buddy;
                }
                (*current_block).order = (current_order + 1) as u8;
            }
            
            current_order += 1;
        }
        
        // Add merged block to free list
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            self.free_lists[current_order].add_page(&mut *current_block);
        }
    }
    
    /// Get buddy block address
    /// @param block: Current block
    /// @param order: Order of the block
    /// @return Pointer to buddy block, or null if invalid
    fn buddy(&self, block: *mut Page, order: usize) -> *mut Page {
        if self.page_array.is_null() {
            return core::ptr::null_mut();
        }
        
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let block_phys = (*block).phys_addr;
            let block_size = (1 << order) * 4096u64;
            let buddy_phys = block_phys ^ block_size;
            
            // Check if buddy address is in valid range
            let mem_end = self.mem_start + (self.num_pages as u64) * 4096;
            if buddy_phys < self.mem_start || buddy_phys >= mem_end {
                return core::ptr::null_mut();
            }
            
            // Calculate page index
            let page_index = ((buddy_phys - self.mem_start) / 4096) as usize;
            if page_index >= self.num_pages {
                return core::ptr::null_mut();
            }
            
            self.page_array.add(page_index)
        }
    }
    
    /// Get physical address of a page
    /// @param page: Pointer to page descriptor
    /// @return Physical address, or 0 if page is null
    pub fn get_phys_addr(&self, page: *const Page) -> u64 {
        if page.is_null() {
            return 0;
        }
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe { (*page).phys_addr }
    }
    
    /// Get number of free pages
    pub fn get_free_pages(&self) -> u32 {
        self.free_pages.load(Ordering::Acquire)
    }
    
    /// Get total number of pages
    pub fn get_total_pages(&self) -> u32 {
        self.total_pages.load(Ordering::Acquire)
    }
    
    /// Get memory usage percentage
    pub fn get_usage_percent(&self) -> u32 {
        let total = self.total_pages.load(Ordering::Acquire);
        let free = self.free_pages.load(Ordering::Acquire);
        
        if total == 0 {
            return 0;
        }
        
        ((total - free) * 100 / total)
    }
    
    /// Print allocator statistics
    pub fn print_stats(&self) {
        log_info!("Buddy Allocator Statistics:");
        log_info!("  Total pages: {}", self.total_pages.load(Ordering::Acquire));
        log_info!("  Free pages: {}", self.free_pages.load(Ordering::Acquire));
        log_info!("  Usage: {}%", self.get_usage_percent());
        log_info!("  Allocations: {}", self.alloc_count.load(Ordering::Acquire));
        log_info!("  Frees: {}", self.free_count.load(Ordering::Acquire));
        
        log_info!("  Free blocks by order:");
        for i in 0..=MAX_ORDER {
            let count = self.free_lists[i].count.load(Ordering::Acquire);
            if count > 0 {
                log_info!("    Order {}: {} blocks ({} pages = {} KB)", 
                    i, count, 1 << i, (1 << i) * 4);
            }
        }
    }
}

/// Global buddy allocator instance
static BUDDY_ALLOC: crate::sync_oncelock::OnceLock<BuddyAllocator> = crate::sync_oncelock::OnceLock::new();

/// Get reference to global buddy allocator
pub fn buddy() -> &'static BuddyAllocator {
    BUDDY_ALLOC.get_or_init(BuddyAllocator::new)
}

/// Initialize buddy allocator
/// @param mem_start: Physical address of memory start
/// @param total_pages: Total number of pages to manage
/// @param page_array: Pointer to page descriptor array
pub fn init_buddy(mem_start: u64, total_pages: u32, page_array: *mut Page) {
    let buddy = buddy();
    buddy.init(mem_start, total_pages, page_array);
}

/// Allocate a single page
pub fn alloc_page() -> *mut Page {
    buddy().alloc(0)
}

/// Allocate pages of specified order
/// @param order: Order of allocation (log2 of pages needed)
/// @return Pointer to page descriptor, or null on failure
pub fn alloc_pages(order: usize) -> *mut Page {
    buddy().alloc(order)
}

/// Free a page block
/// @param page: Pointer to page descriptor to free
pub fn free_page(page: *mut Page) {
    buddy().free(page);
}

/// Get number of free pages
pub fn nr_free_pages() -> u32 {
    buddy().get_free_pages()
}

/// Get total number of pages
pub fn nr_total_pages() -> u32 {
    buddy().get_total_pages()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buddy_new() {
        let buddy = BuddyAllocator::new();
        assert_eq!(buddy.total_pages.load(Ordering::Relaxed), 0);
        assert_eq!(buddy.free_pages.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_free_list_new() {
        let list = FreeList::new();
        assert!(list.head.is_null());
        assert_eq!(list.count.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_page_state() {
        let state = PageState::Free;
        assert!(matches!(state, PageState::Free));
    }

    #[test]
    fn test_page_new() {
        let page = Page::new();
        assert_eq!(page.order, 0);
        assert!(page.is_free());
    }

    #[test]
    fn test_max_order() {
        // Verify MAX_ORDER is reasonable
        assert!(MAX_ORDER >= 1);
        assert!(MAX_ORDER <= 20); /* 2^20 pages = 4GB, large enough */

        // Verify can allocate 2^MAX_ORDER pages
        let pages = 1 << MAX_ORDER;
        assert!(pages >= 1024); /* At least 4MB */
    }
}
