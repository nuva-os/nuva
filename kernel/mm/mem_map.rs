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

//! Memory Map - Page frame number to Page structure mapping

use core::sync::atomic::{AtomicU64, AtomicBool, Ordering};

use super::{Page, PhysAddr, VirtAddr, PAGE_SIZE, PAGE_SHIFT};

/// Kernel virtual address offset for direct mapping
const PAGE_OFFSET: u64 = 0xFFFF_0000_0000_0000;

/// Maximum number of page frames (1GB / 4KB = 262144)
pub const MAX_MEM_MAP_PAGES: usize = 262144;

/// MemMap state
/// Tracks the mem_map array metadata without owning the array directly
/// (the actual Page array is allocated in PhysMemManager).
pub struct MemMapState {
    /// Number of valid page entries
    pub nr_pages: AtomicU64,

    /// Base physical address of the mem_map region
    pub phys_base: AtomicU64,

    /// Highest valid pfn + 1
    pub max_pfn: AtomicU64,

    /// Virtual address of the Page array
    pub page_array: AtomicU64,

    /// Whether mem_map is initialized
    pub initialized: AtomicBool,
}

impl MemMapState {
    /// Create uninitialized mem_map state
    pub const fn new() -> Self {
        MemMapState {
            nr_pages: AtomicU64::new(0),
            phys_base: AtomicU64::new(0),
            max_pfn: AtomicU64::new(0),
            page_array: AtomicU64::new(0),
            initialized: AtomicBool::new(false),
        }
    }

    /// Initialize mem_map state
    /// @param phys_start: Start physical address
    /// @param nr_pages: Number of page frames
    /// @param page_array_ptr: Virtual address of the Page array
    pub fn init(&self, phys_start: PhysAddr, nr_pages: u64, page_array_ptr: u64) {
        self.phys_base.store(phys_start, Ordering::Release);
        self.max_pfn.store(nr_pages, Ordering::Release);
        self.nr_pages.store(nr_pages, Ordering::Release);
        self.page_array.store(page_array_ptr, Ordering::Release);
        self.initialized.store(true, Ordering::Release);
    }

    /// Get Page structure for a physical address
    #[inline(always)]
    pub fn get_page(&self, paddr: PhysAddr) -> *mut Page {
        if !self.initialized.load(Ordering::Acquire) {
            return core::ptr::null_mut();
        }

        let base = self.phys_base.load(Ordering::Acquire);
        if paddr < base {
            return core::ptr::null_mut();
        }

        let pfn = (paddr - base) / PAGE_SIZE;
        if pfn >= self.nr_pages.load(Ordering::Acquire) {
            return core::ptr::null_mut();
        }

        let array = self.page_array.load(Ordering::Acquire) as *mut Page;
        if array.is_null() {
            return core::ptr::null_mut();
        }

        // SAFETY: bounds checked above, array is valid after init
        unsafe { array.add(pfn as usize) }
    }

    /// Get Page structure by page frame number (absolute)
    #[inline(always)]
    pub fn get_page_by_pfn(&self, pfn: u64) -> *mut Page {
        if !self.initialized.load(Ordering::Acquire) {
            return core::ptr::null_mut();
        }

        let base_pfn = self.phys_base.load(Ordering::Acquire) / PAGE_SIZE;
        if pfn < base_pfn {
            return core::ptr::null_mut();
        }

        let idx = pfn - base_pfn;
        if idx >= self.nr_pages.load(Ordering::Acquire) {
            return core::ptr::null_mut();
        }

        let array = self.page_array.load(Ordering::Acquire) as *mut Page;
        if array.is_null() {
            return core::ptr::null_mut();
        }

        // SAFETY: bounds checked above
        unsafe { array.add(idx as usize) }
    }

    /// Get Page structure by index in mem_map array
    #[inline(always)]
    pub fn get_page_by_idx(&self, idx: usize) -> *mut Page {
        if !self.initialized.load(Ordering::Acquire) {
            return core::ptr::null_mut();
        }

        if idx as u64 >= self.nr_pages.load(Ordering::Acquire) {
            return core::ptr::null_mut();
        }

        let array = self.page_array.load(Ordering::Acquire) as *mut Page;
        if array.is_null() {
            return core::ptr::null_mut();
        }

        // SAFETY: bounds checked above
        unsafe { array.add(idx) }
    }

    /// Get total number of pages
    #[inline(always)]
    pub fn get_page_count(&self) -> usize {
        self.nr_pages.load(Ordering::Acquire) as usize
    }

    /// Get maximum pfn
    #[inline(always)]
    pub fn get_max_pfn(&self) -> u64 {
        self.max_pfn.load(Ordering::Acquire)
    }

    /// Check if mem_map is initialized
    #[inline(always)]
    pub fn is_initialized(&self) -> bool {
        self.initialized.load(Ordering::Acquire)
    }
}

/// Global mem_map state
static MEM_MAP_STATE: MemMapState = MemMapState::new();

/// Initialize mem_map subsystem
/// @param phys_start: Start of physical memory
/// @param nr_pages: Number of page frames
/// @param page_array_ptr: Virtual address of the Page array
pub fn init_mem_map(phys_start: PhysAddr, nr_pages: u64, page_array_ptr: u64) {
    MEM_MAP_STATE.init(phys_start, nr_pages, page_array_ptr);
}

/// Get Page structure for physical address
#[inline(always)]
pub fn get_page(paddr: PhysAddr) -> *mut Page {
    MEM_MAP_STATE.get_page(paddr)
}

/// Get Page structure by page frame number
#[inline(always)]
pub fn get_page_by_pfn(pfn: u64) -> *mut Page {
    MEM_MAP_STATE.get_page_by_pfn(pfn)
}

/// Get page count
#[inline(always)]
pub fn get_page_count() -> usize {
    MEM_MAP_STATE.get_page_count()
}

/// Convert physical address to virtual address (direct mapping)
#[inline(always)]
pub fn phys_to_virt(paddr: PhysAddr) -> VirtAddr {
    paddr + PAGE_OFFSET
}

/// Convert virtual address to physical address (direct mapping)
#[inline(always)]
pub fn virt_to_phys(vaddr: VirtAddr) -> PhysAddr {
    if vaddr >= PAGE_OFFSET {
        vaddr - PAGE_OFFSET
    } else {
        vaddr
    }
}

/// Get maximum pfn
#[inline(always)]
pub fn get_max_pfn() -> u64 {
    MEM_MAP_STATE.get_max_pfn()
}

/// Check if mem_map is initialized
#[inline(always)]
pub fn is_initialized() -> bool {
    MEM_MAP_STATE.is_initialized()
}

/// Get mem_map state reference
pub fn get_mem_map_state() -> &'static MemMapState {
    &MEM_MAP_STATE
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phys_to_virt() {
        let vaddr = phys_to_virt(0x1000);
        assert_eq!(vaddr, 0x1000 + PAGE_OFFSET);
    }

    #[test]
    fn test_virt_to_phys() {
        let paddr = virt_to_phys(0x1000 + PAGE_OFFSET);
        assert_eq!(paddr, 0x1000);
    }

    #[test]
    fn test_pfn_conversion() {
        assert_eq!(0x1000u64 >> PAGE_SHIFT, 1);
        assert_eq!(1u64 << PAGE_SHIFT, 0x1000);
    }

    #[test]
    fn test_mem_map_state_new() {
        let state = MemMapState::new();
        assert_eq!(state.get_page_count(), 0);
        assert!(!state.is_initialized());
    }

    #[test]
    fn test_mem_map_state_uninit_returns_null() {
        let state = MemMapState::new();
        assert!(state.get_page(0x1000).is_null());
        assert!(state.get_page_by_pfn(1).is_null());
        assert!(state.get_page_by_idx(0).is_null());
    }
}
