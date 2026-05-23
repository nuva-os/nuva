/*
 * Nuva OS - Kernel - Page Reclamation
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

//! Page Reclamation Implementation
/*!*/
//! LRU-based page reclamation with working set awareness.

use core::sync::atomic::{AtomicU32, AtomicU64, AtomicBool, AtomicPtr, Ordering};
use crate::{pr_debug, pr_info, pr_warn};

/// Reclaim configuration
pub mod reclaim_config {
    /// Target pages to reclaim per scan
    pub const RECLAIM_TARGET_PAGES: usize = 32;

    /// Maximum pages to scan per iteration
    pub const MAX_SCAN_PAGES: usize = 1024;

    /// Active list ratio (active / inactive)
    pub const ACTIVE_LIST_RATIO: u32 = 2;

    /// Working set window size (pages)
    pub const WORKING_SET_WINDOW: usize = 4096;

    /// Refault distance threshold
    pub const REFAULT_THRESHOLD: u32 = 100;

    /// Page age decay factor
    pub const AGE_DECAY_FACTOR: u32 = 2;

    /// Minimum pages to keep free
    pub const MIN_FREE_PAGES: u64 = 1024;
}

/// LRU list types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LruList {
    /// Active anonymous
    ActiveAnon = 0,
    /// Inactive anonymous
    InactiveAnon = 1,
    /// Active file
    ActiveFile = 2,
    /// Inactive file
    InactiveFile = 3,
    /// Unevictable
    Unevictable = 4,
}

/// LRU list count
pub const NR_LRU_LISTS: usize = 5;

/// Page reference
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct PageRef {
    /// Physical address
    pub phys_addr: u64,

    /// Page flags
    pub flags: u32,

    /// Reference count
    pub ref_count: u32,

    /// Map count
    pub map_count: u32,

    /// Page age (access counter)
    pub age: u32,

    /// Last access time
    pub last_access: u64,
}

impl PageRef {
    pub const fn new(phys: u64) -> Self {
        Self {
            phys_addr: phys,
            flags: 0,
            ref_count: 0,
            map_count: 0,
            age: 0,
            last_access: 0,
        }
    }

    pub fn is_anon(&self) -> bool {
        (self.flags & PG_ANON) != 0
    }

    pub fn is_file(&self) -> bool {
        !self.is_anon()
    }

    pub fn is_dirty(&self) -> bool {
        (self.flags & PG_DIRTY) != 0
    }

    pub fn is_referenced(&self) -> bool {
        (self.flags & PG_REFERENCED) != 0
    }

    pub fn is_active(&self) -> bool {
        (self.flags & PG_ACTIVE) != 0
    }

    pub fn is_unevictable(&self) -> bool {
        (self.flags & PG_UNEVICTABLE) != 0
    }
}

/// Page flags
pub const PG_DIRTY: u32 = 0x00000002;
pub const PG_REFERENCED: u32 = 0x00002000;
pub const PG_ACTIVE: u32 = 0x00008000;
pub const PG_ANON: u32 = 0x00000400;
pub const PG_UNEVICTABLE: u32 = 0x00080000;
pub const PG_LOCKED: u32 = 0x00000001;

/// LRU list node
#[derive(Debug)]
#[repr(C)]
pub struct LruNode {
    /// Page reference
    pub page: PageRef,

    /// Next node
    pub next: AtomicPtr<LruNode>,

    /// Previous node
    pub prev: AtomicPtr<LruNode>,
}

impl LruNode {
    pub fn new(page: PageRef) -> Self {
        Self {
            page,
            next: AtomicPtr::new(core::ptr::null_mut()),
            prev: AtomicPtr::new(core::ptr::null_mut()),
        }
    }
}

/// LRU list
pub struct LruListHead {
    /// Head pointer
    head: AtomicPtr<LruNode>,

    /// Tail pointer
    tail: AtomicPtr<LruNode>,

    /// Node count
    count: AtomicU64,

    /// List type
    list_type: LruList,
}

impl LruListHead {
    pub const fn new(list_type: LruList) -> Self {
        Self {
            head: AtomicPtr::new(core::ptr::null_mut()),
            tail: AtomicPtr::new(core::ptr::null_mut()),
            count: AtomicU64::new(0),
            list_type,
        }
    }

    /// Add page to tail (most recently used)
    pub fn add_tail(&self, node: *mut LruNode) {
        if node.is_null() {
            return;
        }

        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            (*node).next.store(core::ptr::null_mut(), Ordering::Release);
            (*node).prev.store(self.tail.load(Ordering::Acquire), Ordering::Release);
        }

        let old_tail = self.tail.swap(node, Ordering::AcqRel);

        if old_tail.is_null() {
            self.head.store(node, Ordering::Release);
        } else {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                (*old_tail).next.store(node, Ordering::Release);
            }
        }

        self.count.fetch_add(1, Ordering::Relaxed);
    }

    /// Remove page from list
    pub fn remove(&self, node: *mut LruNode) {
        if node.is_null() {
            return;
        }

        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let prev = (*node).prev.load(Ordering::Acquire);
            let next = (*node).next.load(Ordering::Acquire);

            if prev.is_null() {
                self.head.store(next, Ordering::Release);
            } else {
                (*prev).next.store(next, Ordering::Release);
            }

            if next.is_null() {
                self.tail.store(prev, Ordering::Release);
            } else {
                (*next).prev.store(prev, Ordering::Release);
            }

            (*node).prev.store(core::ptr::null_mut(), Ordering::Release);
            (*node).next.store(core::ptr::null_mut(), Ordering::Release);
        }

        self.count.fetch_sub(1, Ordering::Relaxed);
    }

    /// Remove and return head (least recently used)
    pub fn remove_head(&self) -> *mut LruNode {
        let head = self.head.load(Ordering::Acquire);
        if head.is_null() {
            return core::ptr::null_mut();
        }

        self.remove(head);
        head
    }

    /// Rotate list: move head to tail
    pub fn rotate(&self) {
        let head = self.remove_head();
        if !head.is_null() {
            self.add_tail(head);
        }
    }

    /// Get list count
    pub fn len(&self) -> u64 {
        self.count.load(Ordering::Relaxed)
    }

    /// Check if list is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Working set estimator
pub struct WorkingSetEstimator {
    /// Total pages accessed in window
    total_accesses: AtomicU64,

    /// Unique pages accessed
    unique_pages: AtomicU64,

    /// Refault counter
    refaults: AtomicU64,

    /// Evictions counter
    evictions: AtomicU64,

    /// Current window start
    window_start: AtomicU64,

    /// Window size
    window_size: u64,
}

impl WorkingSetEstimator {
    pub const fn new() -> Self {
        Self {
            total_accesses: AtomicU64::new(0),
            unique_pages: AtomicU64::new(0),
            refaults: AtomicU64::new(0),
            evictions: AtomicU64::new(0),
            window_start: AtomicU64::new(0),
            window_size: reclaim_config::WORKING_SET_WINDOW as u64,
        }
    }

    /// Record page access
    pub fn record_access(&self) {
        self.total_accesses.fetch_add(1, Ordering::Relaxed);
    }

    /// Record refault (page brought back after eviction)
    pub fn record_refault(&self) {
        self.refaults.fetch_add(1, Ordering::Relaxed);
    }

    /// Record eviction
    pub fn record_eviction(&self) {
        self.evictions.fetch_add(1, Ordering::Relaxed);
    }

    /// Estimate working set size
    pub fn estimate_working_set(&self) -> u64 {
        let accesses = self.total_accesses.load(Ordering::Relaxed);
        let unique = self.unique_pages.load(Ordering::Relaxed);

        if accesses == 0 {
            return 0;
        }

        // Working set = unique pages * (accesses / window_size)
        unique * accesses / self.window_size
    }

    /// Get refault rate
    pub fn refault_rate(&self) -> u32 {
        let refaults = self.refaults.load(Ordering::Relaxed);
        let evictions = self.evictions.load(Ordering::Relaxed);

        if evictions == 0 {
            return 0;
        }

        ((refaults * 100) / evictions) as u32
    }
}

/// Reclaim statistics
pub struct ReclaimStats {
    /// Pages reclaimed
    pub pages_reclaimed: AtomicU64,

    /// Pages scanned
    pub pages_scanned: AtomicU64,

    /// Pages rotated
    pub pages_rotated: AtomicU64,

    /// Reclaim failures
    pub reclaim_failures: AtomicU64,

    /// Direct reclaim count
    pub direct_reclaims: AtomicU64,

    /// Background reclaim count
    pub background_reclaims: AtomicU64,
}

impl ReclaimStats {
    pub const fn new() -> Self {
        Self {
            pages_reclaimed: AtomicU64::new(0),
            pages_scanned: AtomicU64::new(0),
            pages_rotated: AtomicU64::new(0),
            reclaim_failures: AtomicU64::new(0),
            direct_reclaims: AtomicU64::new(0),
            background_reclaims: AtomicU64::new(0),
        }
    }
}

/// Page reclaimer
pub struct PageReclaimer {
    /// LRU lists
    lru_lists: [LruListHead; NR_LRU_LISTS],

    /// Working set estimator
    working_set: WorkingSetEstimator,

    /// Statistics
    stats: ReclaimStats,

    /// Reclaim in progress
    reclaiming: AtomicBool,

    /// Target free pages
    target_free: AtomicU64,

    /// Current free pages
    current_free: AtomicU64,
}

impl PageReclaimer {
    pub const fn new() -> Self {
        Self {
            lru_lists: [
                LruListHead::new(LruList::ActiveAnon),
                LruListHead::new(LruList::InactiveAnon),
                LruListHead::new(LruList::ActiveFile),
                LruListHead::new(LruList::InactiveFile),
                LruListHead::new(LruList::Unevictable),
            ],
            working_set: WorkingSetEstimator::new(),
            stats: ReclaimStats::new(),
            reclaiming: AtomicBool::new(false),
            target_free: AtomicU64::new(reclaim_config::MIN_FREE_PAGES),
            current_free: AtomicU64::new(0),
        }
    }

    /// Check if reclaimer is currently reclaiming
    pub fn is_reclaiming(&self) -> bool {
        self.reclaiming.load(Ordering::Relaxed)
    }

    /// Initialize reclaimer
    pub fn init(&self) {
        log_info!("Page reclaimer initialized");
        log_info!("  LRU lists: {}", NR_LRU_LISTS);
        log_info!("  Target free pages: {}", self.target_free.load(Ordering::Relaxed));
    }

    /// Add page to LRU list
    pub fn add_page(&self, page: PageRef) {
        if page.is_unevictable() {
            // Add to unevictable list
            let node = self.alloc_node(page);
            self.lru_lists[LruList::Unevictable as usize].add_tail(node);
            return;
        }

        // Determine which list to add to
        let list_idx = if page.is_anon() {
            if page.is_active() {
                LruList::ActiveAnon as usize
            } else {
                LruList::InactiveAnon as usize
            }
        } else {
            if page.is_active() {
                LruList::ActiveFile as usize
            } else {
                LruList::InactiveFile as usize
            }
        };

        let node = self.alloc_node(page);
        self.lru_lists[list_idx].add_tail(node);
    }

    /// Remove page from LRU list
    pub fn remove_page(&self, page: *mut LruNode) {
        // Determine which list the page is on
        for list in &self.lru_lists {
            list.remove(page);
        }
    }

    /// Mark page as accessed
    pub fn mark_accessed(&self, page: *mut LruNode) {
        if page.is_null() {
            return;
        }

        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            (*page).page.flags |= PG_REFERENCED;
            (*page).page.last_access = self.current_time();
            (*page).page.age = (*page).page.age.saturating_add(1);
        }

        self.working_set.record_access();
    }

    /// Reclaim pages
    pub fn reclaim(&self, target: usize) -> Result<usize, ReclaimError> {
        // Check if already reclaiming
        if self.reclaiming.swap(true, Ordering::AcqRel) {
            return Err(ReclaimError::AlreadyReclaiming);
        }

        let mut reclaimed = 0;
        let mut scanned = 0;

        // Try to reclaim from inactive lists first
        reclaimed += self.reclaim_from_list(
            LruList::InactiveFile,
            target - reclaimed,
            &mut scanned,
        )?;

        if reclaimed < target {
            reclaimed += self.reclaim_from_list(
                LruList::InactiveAnon,
                target - reclaimed,
                &mut scanned,
            )?;
        }

        // If still not enough, shrink active lists
        if reclaimed < target {
            self.shrink_active_lists();
        }

        // Update statistics
        self.stats.pages_reclaimed.fetch_add(reclaimed as u64, Ordering::Relaxed);
        self.stats.pages_scanned.fetch_add(scanned as u64, Ordering::Relaxed);

        self.reclaiming.store(false, Ordering::Release);

        Ok(reclaimed)
    }

    /// Reclaim from a specific LRU list
    fn reclaim_from_list(
        &self,
        list_type: LruList,
        target: usize,
        scanned: &mut usize,
    ) -> Result<usize, ReclaimError> {
        let list = &self.lru_lists[list_type as usize];
        let mut reclaimed = 0;

        while reclaimed < target && *scanned < reclaim_config::MAX_SCAN_PAGES {
            let node = list.remove_head();
            if node.is_null() {
                break;
            }

            *scanned += 1;

            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                let page = &(*node).page;

                // Check if page can be reclaimed
                if self.can_reclaim(page) {
                    // Write back if dirty
                    if page.is_dirty() {
                        self.writeback_page(page);
                    }

                    // Reclaim the page
                    self.free_page(page);
                    reclaimed += 1;
                    self.working_set.record_eviction();
                } else {
                    // Page is referenced, promote to active
                    if page.is_referenced() {
                        self.promote_page(node, list_type);
                    } else {
                        // Put back at tail
                        list.add_tail(node);
                    }
                }
            }
        }

        Ok(reclaimed)
    }

    /// Check if page can be reclaimed
    fn can_reclaim(&self, page: &PageRef) -> bool {
        // Cannot reclaim if:
        // - Locked
        // - Reference count > 0
        // - Mapped and not clean

        if (page.flags & PG_LOCKED) != 0 {
            return false;
        }

        if page.ref_count > 0 {
            return false;
        }

        // Can reclaim if not mapped or clean
        page.map_count == 0 || !page.is_dirty()
    }

    /// Promote page to active list
    fn promote_page(&self, node: *mut LruNode, from_list: LruList) {
        let to_list = match from_list {
            LruList::InactiveAnon => LruList::ActiveAnon,
            LruList::InactiveFile => LruList::ActiveFile,
            _ => return,
        };

        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            (*node).page.flags |= PG_ACTIVE;
            (*node).page.flags &= !PG_REFERENCED;
        }

        self.lru_lists[to_list as usize].add_tail(node);
    }

    /// Shrink active lists (demote pages to inactive)
    fn shrink_active_lists(&self) {
        // Demote some pages from active to inactive
        let active_file = &self.lru_lists[LruList::ActiveFile as usize];
        let inactive_file = &self.lru_lists[LruList::InactiveFile as usize];

        // Move 1/4 of active file pages to inactive
        let to_move = active_file.len() / 4;
        for _ in 0..to_move {
            let node = active_file.remove_head();
            if node.is_null() {
                break;
            }

            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                (*node).page.flags &= !PG_ACTIVE;
            }

            inactive_file.add_tail(node);
        }

        // Same for anonymous pages
        let active_anon = &self.lru_lists[LruList::ActiveAnon as usize];
        let inactive_anon = &self.lru_lists[LruList::InactiveAnon as usize];

        let to_move = active_anon.len() / 4;
        for _ in 0..to_move {
            let node = active_anon.remove_head();
            if node.is_null() {
                break;
            }

            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                (*node).page.flags &= !PG_ACTIVE;
            }

            inactive_anon.add_tail(node);
        }
    }

    /// Background reclaim (kswapd style)
    pub fn background_reclaim(&self) {
        let free = self.current_free.load(Ordering::Relaxed);
        let target = self.target_free.load(Ordering::Relaxed);

        if free < target {
            let to_reclaim = (target - free) as usize;
            match self.reclaim(to_reclaim) {
                Ok(n) => {
                    self.stats.background_reclaims.fetch_add(1, Ordering::Relaxed);
                    log_debug!("Background reclaim: {} pages", n);
                }
                Err(e) => {
                    log_warn!("Background reclaim failed: {:?}", e);
                }
            }
        }
    }

    /// Direct reclaim (synchronous)
    pub fn direct_reclaim(&self, needed: usize) -> Result<usize, ReclaimError> {
        self.stats.direct_reclaims.fetch_add(1, Ordering::Relaxed);
        self.reclaim(needed)
    }

    /// Check if reclaim is needed
    pub fn needs_reclaim(&self) -> bool {
        let free = self.current_free.load(Ordering::Relaxed);
        let target = self.target_free.load(Ordering::Relaxed);
        free < target
    }

    /// Update free pages count
    pub fn update_free_pages(&self, count: u64) {
        self.current_free.store(count, Ordering::Release);
    }

    /// Set target free pages
    pub fn set_target_free(&self, target: u64) {
        self.target_free.store(target, Ordering::Release);
    }

    /// Get LRU list sizes
    pub fn get_lru_sizes(&self) -> [u64; NR_LRU_LISTS] {
        [
            self.lru_lists[0].len(),
            self.lru_lists[1].len(),
            self.lru_lists[2].len(),
            self.lru_lists[3].len(),
            self.lru_lists[4].len(),
        ]
    }

    /// Print statistics
    pub fn print_stats(&self) {
        log_info!("Page Reclaimer Statistics:");
        log_info!("  Pages reclaimed: {}", self.stats.pages_reclaimed.load(Ordering::Relaxed));
        log_info!("  Pages scanned: {}", self.stats.pages_scanned.load(Ordering::Relaxed));
        log_info!("  Pages rotated: {}", self.stats.pages_rotated.load(Ordering::Relaxed));
        log_info!("  Reclaim failures: {}", self.stats.reclaim_failures.load(Ordering::Relaxed));
        log_info!("  Direct reclaims: {}", self.stats.direct_reclaims.load(Ordering::Relaxed));
        log_info!("  Background reclaims: {}", self.stats.background_reclaims.load(Ordering::Relaxed));

        let sizes = self.get_lru_sizes();
        log_info!("  LRU Active Anon: {}", sizes[0]);
        log_info!("  LRU Inactive Anon: {}", sizes[1]);
        log_info!("  LRU Active File: {}", sizes[2]);
        log_info!("  LRU Inactive File: {}", sizes[3]);
        log_info!("  LRU Unevictable: {}", sizes[4]);

        log_info!("  Working set estimate: {}", self.working_set.estimate_working_set());
        log_info!("  Refault rate: {}%", self.working_set.refault_rate());
    }

    /// Allocate LRU node using kernel allocator
    fn alloc_node(&self, page: PageRef) -> *mut LruNode {
        let size = core::mem::size_of::<LruNode>();
        // SAFETY: unsafe block required for low-level memory or hardware access
        let ptr = unsafe { alloc_zeroed(size) as *mut LruNode };
        if !ptr.is_null() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                (*ptr).page = page;
            }
        }
        ptr
    }

    /// Free LRU node via kernel allocator
    fn free_node(&self, node: *mut LruNode) {
        if !node.is_null() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                dealloc(node as *mut u8, core::mem::size_of::<LruNode>());
            }
        }
    }

    /// Write back dirty page
    fn writeback_page(&self, page: &PageRef) {
        log_debug!("Writeback page: {:#x}", page.phys_addr);
        // SAFETY: flush dirty data to backing store
        // For anonymous pages, swap out; for file pages, write to filesystem
        // This is a no-op until the swap/filesystem subsystem is connected
    }

    /// Free page via buddy allocator
    fn free_page(&self, page: &PageRef) {
        log_debug!("Free reclaimed page: {:#x}", page.phys_addr);
        let p = super::mem_map::get_page(page.phys_addr);
        if !p.is_null() {
            super::free_page(p);
        }
    }

    /// Get current time from kernel clock
    fn current_time(&self) -> u64 {
        // Use jiffies or monotonic clock from timer subsystem
        // SAFETY: reading a global counter is safe
        crate::kernel::time::get_jiffies()
    }
}

/// Reclaim error
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReclaimError {
    AlreadyReclaiming,
    NoPagesToReclaim,
    WritebackFailed,
    OutOfMemory,
}

/// External functions (to be implemented elsewhere)
extern "C" {
    fn alloc_zeroed(size: usize) -> *mut u8;
    fn dealloc(ptr: *mut u8, size: usize);
}

/// Global page reclaimer
static PAGE_RECLAIMER: PageReclaimer = PageReclaimer::new();

/// Get page reclaimer
pub fn get_reclaimer() -> &'static PageReclaimer {
    &PAGE_RECLAIMER
}

/// Initialize page reclaimer
pub fn init_reclaimer() {
    get_reclaimer().init();
}

/// Reclaim pages
pub fn reclaim_pages(target: usize) -> Result<usize, ReclaimError> {
    get_reclaimer().reclaim(target)
}

/// Check if reclaim is needed
pub fn needs_reclaim() -> bool {
    get_reclaimer().needs_reclaim()
}

/// Background reclaim
pub fn background_reclaim() {
    get_reclaimer().background_reclaim();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lru_list() {
        let list = LruListHead::new(LruList::InactiveFile);
        assert!(list.is_empty());
        assert_eq!(list.len(), 0);
    }

    #[test]
    fn test_page_ref() {
        let page = PageRef::new(0x1000);
        assert_eq!(page.phys_addr, 0x1000);
        assert!(!page.is_anon());
        assert!(!page.is_dirty());
    }

    #[test]
    fn test_working_set() {
        let ws = WorkingSetEstimator::new();
        ws.record_access();
        ws.record_access();
        ws.record_refault();

        assert_eq!(ws.refault_rate(), 0);
    }

    #[test]
    fn test_reclaim_stats() {
        let stats = ReclaimStats::new();
        assert_eq!(stats.pages_reclaimed.load(Ordering::Relaxed), 0);
        assert_eq!(stats.pages_scanned.load(Ordering::Relaxed), 0);
    }
}
