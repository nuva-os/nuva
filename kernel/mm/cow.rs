/*
* Nuva OS - Kernel - Copy-on-Write Implementation
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

//! Copy-on-Write (COW) Implementation
/*!*/
//! Complete COW support for memory sharing and fork optimization.

use crate::{pr_debug, pr_info};
use core::sync::atomic::{AtomicBool, AtomicPtr, AtomicU32, AtomicU64, Ordering};

/// COW configuration
pub mod cow_config {
    /// Maximum COW chains
    pub const MAX_COW_CHAINS: usize = 16;

    /// COW break batch size
    pub const COW_BREAK_BATCH: usize = 32;

    /// Enable COW statistics
    pub const ENABLE_STATS: bool = true;
}

/// COW page flags
pub mod cow_flags {
    /// Page is COW
    pub const COW_PAGE: u32 = 0x00000001;

    /// COW pending (write fault occurred)
    pub const COW_PENDING: u32 = 0x00000002;

    /// COW broken (copy made)
    pub const COW_BROKEN: u32 = 0x00000004;

    /// COW shared (multiple mappings)
    pub const COW_SHARED: u32 = 0x00000008;
}

/// Physical address type
pub type PhysAddr = u64;

/// Virtual address type
pub type VirtAddr = u64;

/// Page size
pub const PAGE_SIZE: u64 = 4096;

/// COW page entry
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct CowEntry {
    /// Physical address of shared page
    pub phys_addr: PhysAddr,

    /// Reference count
    pub ref_count: u32,

    /// Flags
    pub flags: u32,

    /// Owner process ID
    pub owner_pid: u32,

    /// Original virtual address in owner
    pub orig_vaddr: VirtAddr,
}

impl CowEntry {
    pub const fn new(phys: PhysAddr, pid: u32, vaddr: VirtAddr) -> Self {
        Self {
            phys_addr: phys,
            ref_count: 1,
            flags: cow_flags::COW_PAGE,
            owner_pid: pid,
            orig_vaddr: vaddr,
        }
    }

    pub fn is_cow(&self) -> bool {
        (self.flags & cow_flags::COW_PAGE) != 0
    }

    pub fn is_pending(&self) -> bool {
        (self.flags & cow_flags::COW_PENDING) != 0
    }

    pub fn is_broken(&self) -> bool {
        (self.flags & cow_flags::COW_BROKEN) != 0
    }

    pub fn is_shared(&self) -> bool {
        self.ref_count > 1
    }
}

/// COW statistics
pub struct CowStats {
    /// Total COW pages created
    pub pages_created: AtomicU64,

    /// COW breaks (copies made)
    pub breaks: AtomicU64,

    /// COW faults handled
    pub faults_handled: AtomicU64,

    /// COW pages shared
    pub pages_shared: AtomicU64,

    /// COW pages freed
    pub pages_freed: AtomicU64,

    /// Break failures
    pub break_failures: AtomicU64,

    /// Memory saved by COW
    pub memory_saved: AtomicU64,
}

impl CowStats {
    pub const fn new() -> Self {
        Self {
            pages_created: AtomicU64::new(0),
            breaks: AtomicU64::new(0),
            faults_handled: AtomicU64::new(0),
            pages_shared: AtomicU64::new(0),
            pages_freed: AtomicU64::new(0),
            break_failures: AtomicU64::new(0),
            memory_saved: AtomicU64::new(0),
        }
    }
}

/// COW manager
pub struct CowManager {
    /// Statistics
    stats: CowStats,

    /// COW enabled
    enabled: AtomicBool,

    /// Break batch size
    break_batch: AtomicU32,
}

impl CowManager {
    pub const fn new() -> Self {
        Self {
            stats: CowStats::new(),
            enabled: AtomicBool::new(true),
            break_batch: AtomicU32::new(cow_config::COW_BREAK_BATCH as u32),
        }
    }

    /// Initialize COW manager
    pub fn init(&self) {
        log_info!("COW manager initialized");
        log_info!("  COW enabled: {}", self.enabled.load(Ordering::Relaxed));
        log_info!(
            "  Break batch size: {}",
            self.break_batch.load(Ordering::Relaxed)
        );
    }

    /// Create COW page (for fork)
    pub fn create_cow_page(
        &self,
        src_phys: PhysAddr,
        src_vaddr: VirtAddr,
        owner_pid: u32,
    ) -> Result<CowEntry, CowError> {
        if !self.enabled.load(Ordering::Relaxed) {
            return Err(CowError::Disabled);
        }

        // Create COW entry
        let entry = CowEntry::new(src_phys, owner_pid, src_vaddr);

        // Increment reference count on source page
        self.inc_page_ref(src_phys);

        // Update statistics
        self.stats.pages_created.fetch_add(1, Ordering::Relaxed);
        self.stats
            .memory_saved
            .fetch_add(PAGE_SIZE, Ordering::Relaxed);

        log_debug!(
            "COW page created: phys={:#x}, owner={}, vaddr={:#x}",
            src_phys,
            owner_pid,
            src_vaddr
        );

        Ok(entry)
    }

    /// Handle write fault on COW page
    pub fn handle_cow_fault(
        &self,
        entry: &mut CowEntry,
        fault_vaddr: VirtAddr,
        current_pid: u32,
    ) -> Result<PhysAddr, CowError> {
        if !entry.is_cow() {
            return Err(CowError::NotCowPage);
        }

        if entry.is_broken() {
            // Already broken, return the physical address
            return Ok(entry.phys_addr);
        }

        // Update statistics
        self.stats.faults_handled.fetch_add(1, Ordering::Relaxed);

        // Check if we're the only reference
        if entry.ref_count == 1 {
            // We own the page, no need to copy
            entry.flags &= !cow_flags::COW_PAGE;
            entry.flags |= cow_flags::COW_BROKEN;
            return Ok(entry.phys_addr);
        }

        // Need to make a copy
        let new_phys = self.break_cow(entry, current_pid)?;

        log_debug!(
            "COW break: old={:#x}, new={:#x}, vaddr={:#x}",
            entry.phys_addr,
            new_phys,
            fault_vaddr
        );

        Ok(new_phys)
    }

    /// Break COW (make a copy)
    fn break_cow(&self, entry: &mut CowEntry, current_pid: u32) -> Result<PhysAddr, CowError> {
        // Allocate new page
        let new_phys = self.alloc_page();
        if new_phys == 0 {
            self.stats.break_failures.fetch_add(1, Ordering::Relaxed);
            return Err(CowError::OutOfMemory);
        }

        // Copy content
        self.copy_page(new_phys, entry.phys_addr);

        // Decrement reference on old page
        self.dec_page_ref(entry.phys_addr);

        // Update entry
        let old_phys = entry.phys_addr;
        entry.phys_addr = new_phys;
        entry.ref_count = 1;
        entry.flags &= !cow_flags::COW_PAGE;
        entry.flags |= cow_flags::COW_BROKEN;
        entry.owner_pid = current_pid;

        // Update statistics
        self.stats.breaks.fetch_add(1, Ordering::Relaxed);
        self.stats
            .memory_saved
            .fetch_sub(PAGE_SIZE, Ordering::Relaxed);

        log_debug!("COW broken: {:#x} -> {:#x}", old_phys, new_phys);

        Ok(new_phys)
    }

    /// Break COW for a range of pages
    pub fn break_cow_range(
        &self,
        entries: &mut [CowEntry],
        start_vaddr: VirtAddr,
        current_pid: u32,
    ) -> Result<usize, CowError> {
        let mut broken = 0;
        let batch = self.break_batch.load(Ordering::Relaxed) as usize;

        for (i, entry) in entries.iter_mut().enumerate() {
            if !entry.is_cow() || entry.is_broken() {
                continue;
            }

            let vaddr = start_vaddr + (i as u64) * PAGE_SIZE;
            self.handle_cow_fault(entry, vaddr, current_pid)?;
            broken += 1;

            // Yield after batch
            if broken % batch == 0 {
                // TODO: Check for need_resched
            }
        }

        Ok(broken)
    }

    /// Share COW page (add reference)
    pub fn share_cow_page(&self, entry: &mut CowEntry) {
        entry.ref_count += 1;
        entry.flags |= cow_flags::COW_SHARED;

        self.stats.pages_shared.fetch_add(1, Ordering::Relaxed);
        self.stats
            .memory_saved
            .fetch_add(PAGE_SIZE, Ordering::Relaxed);
    }

    /// Release COW page
    pub fn release_cow_page(&self, entry: &mut CowEntry) -> bool {
        if entry.ref_count > 0 {
            entry.ref_count -= 1;
        }

        if entry.ref_count == 0 {
            // Last reference, free the page
            self.free_page(entry.phys_addr);
            self.stats.pages_freed.fetch_add(1, Ordering::Relaxed);
            return true;
        }

        // Update shared flag
        if entry.ref_count == 1 {
            entry.flags &= !cow_flags::COW_SHARED;
        }

        false
    }

    /// Merge COW pages (KSM-style)
    pub fn try_merge_cow(&self, entry1: &CowEntry, entry2: &CowEntry) -> Result<bool, CowError> {
        // Check if pages can be merged
        if entry1.phys_addr == entry2.phys_addr {
            return Ok(false); // Already same page
        }

        // Compare page contents
        if !self.pages_equal(entry1.phys_addr, entry2.phys_addr) {
            return Ok(false); // Contents differ
        }

        // Merge: point both to same physical page
        // This would require updating page tables
        // For now, just return false
        Ok(false)
    }

    /// Check if COW is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    /// Enable/disable COW
    pub fn set_enabled(&self, enabled: bool) {
        self.enabled.store(enabled, Ordering::Release);
    }

    /// Get statistics
    pub fn get_stats(&self) -> &CowStats {
        &self.stats
    }

    /// Print statistics
    pub fn print_stats(&self) {
        log_info!("COW Statistics:");
        log_info!(
            "  Pages created: {}",
            self.stats.pages_created.load(Ordering::Relaxed)
        );
        log_info!(
            "  COW breaks: {}",
            self.stats.breaks.load(Ordering::Relaxed)
        );
        log_info!(
            "  Faults handled: {}",
            self.stats.faults_handled.load(Ordering::Relaxed)
        );
        log_info!(
            "  Pages shared: {}",
            self.stats.pages_shared.load(Ordering::Relaxed)
        );
        log_info!(
            "  Pages freed: {}",
            self.stats.pages_freed.load(Ordering::Relaxed)
        );
        log_info!(
            "  Break failures: {}",
            self.stats.break_failures.load(Ordering::Relaxed)
        );
        log_info!(
            "  Memory saved: {} KB",
            self.stats.memory_saved.load(Ordering::Relaxed) / 1024
        );
    }

    /// Allocate a page
    fn alloc_page(&self) -> PhysAddr {
        let p = super::alloc_page();
        if p.is_null() {
            0
        } else {
            p as PhysAddr
        }
    }

    /// Free a page
    fn free_page(&self, phys: PhysAddr) {
        let page = super::phys_to_page(phys);
        super::free_page(page);
    }

    /// Copy page content
    fn copy_page(&self, dst: PhysAddr, src: PhysAddr) {
        // TODO: Use optimized copy
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            core::ptr::copy_nonoverlapping(src as *const u8, dst as *mut u8, PAGE_SIZE as usize);
        }
    }

    /// Compare pages
    fn pages_equal(&self, phys1: PhysAddr, phys2: PhysAddr) -> bool {
        // TODO: Use optimized comparison
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let ptr1 = phys1 as *const u8;
            let ptr2 = phys2 as *const u8;
            for i in 0..PAGE_SIZE as usize {
                if *ptr1.add(i) != *ptr2.add(i) {
                    return false;
                }
            }
        }
        true
    }

    /// Increment page reference
    fn inc_page_ref(&self, phys: PhysAddr) {
        let page = super::phys_to_page(phys);
        if !page.is_null() {
            // SAFETY: page is a valid non-null pointer returned by
            // phys_to_page. The page structure is valid for the lifetime
            // of the physical page. fetch_add is an atomic operation.
            unsafe {
                (*page).ref_count.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    /// Decrement page reference
    fn dec_page_ref(&self, phys: PhysAddr) {
        let page = super::phys_to_page(phys);
        if !page.is_null() {
            let old = unsafe { (*page).ref_count.fetch_sub(1, Ordering::Relaxed) };
            if old == 1 {
                // Last reference dropped, free the page
                self.free_page(phys);
            }
        }
    }
}

/// COW error
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CowError {
    Disabled,
    NotCowPage,
    OutOfMemory,
    InvalidAddress,
    AlreadyBroken,
}

/// COW fault handler for page fault
pub struct CowFaultHandler {
    /// COW manager
    cow: &'static CowManager,
}

impl CowFaultHandler {
    pub fn new() -> Self {
        Self { cow: cow_manager() }
    }

    /// Handle page fault
    pub fn handle_fault(
        &self,
        vaddr: VirtAddr,
        pte: &mut PageTableEntry,
        write: bool,
        pid: u32,
    ) -> Result<FaultAction, CowError> {
        // Check if this is a COW page
        if !pte.is_cow() {
            return Ok(FaultAction::None);
        }

        // Read fault on COW page is OK
        if !write {
            return Ok(FaultAction::None);
        }

        // Write fault: need to break COW
        let entry = CowEntry::new(pte.phys_addr(), pte.owner_pid(), vaddr);
        let mut entry = entry;

        let new_phys = self.cow.handle_cow_fault(&mut entry, vaddr, pid)?;

        // Update page table entry
        pte.set_phys_addr(new_phys);
        pte.set_writable(true);
        pte.clear_cow();

        Ok(FaultAction::UpdatePte)
    }
}

/// Fault action
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaultAction {
    None,
    UpdatePte,
    Signal,
    Oom,
}

/// Page table entry (simplified)
#[derive(Debug, Clone, Copy)]
pub struct PageTableEntry {
    pub value: u64,
}

impl PageTableEntry {
    pub fn is_cow(&self) -> bool {
        (self.value & (1 << 51)) != 0
    }

    pub fn phys_addr(&self) -> PhysAddr {
        (self.value & 0x000FFFFFFFFFF000) >> 12 << 12
    }

    pub fn owner_pid(&self) -> u32 {
        ((self.value >> 48) & 0xFFF) as u32
    }

    pub fn set_phys_addr(&mut self, phys: PhysAddr) {
        self.value = (self.value & !0x000FFFFFFFFFF000) | (phys & 0x000FFFFFFFFFF000);
    }

    pub fn set_writable(&mut self, writable: bool) {
        if writable {
            self.value |= 1 << 1;
        } else {
            self.value &= !(1 << 1);
        }
    }

    pub fn clear_cow(&mut self) {
        self.value &= !(1 << 51);
    }
}

/// Global COW manager
static COW_MANAGER: CowManager = CowManager::new();

/// Get COW manager
pub fn cow_manager() -> &'static CowManager {
    &COW_MANAGER
}

/// Initialize COW
pub fn init_cow() {
    cow_manager().init();
}

/// Create COW page
pub fn create_cow_page(
    src_phys: PhysAddr,
    src_vaddr: VirtAddr,
    owner_pid: u32,
) -> Result<CowEntry, CowError> {
    cow_manager().create_cow_page(src_phys, src_vaddr, owner_pid)
}

/// Handle COW fault
pub fn handle_cow_fault(
    entry: &mut CowEntry,
    fault_vaddr: VirtAddr,
    current_pid: u32,
) -> Result<PhysAddr, CowError> {
    cow_manager().handle_cow_fault(entry, fault_vaddr, current_pid)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cow_entry() {
        let entry = CowEntry::new(0x1000, 1, 0x400000);
        assert!(entry.is_cow());
        assert!(!entry.is_broken());
        assert!(!entry.is_shared());
        assert_eq!(entry.ref_count, 1);
    }

    #[test]
    fn test_cow_stats() {
        let stats = CowStats::new();
        assert_eq!(stats.pages_created.load(Ordering::Relaxed), 0);
        assert_eq!(stats.breaks.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_cow_manager() {
        let manager = CowManager::new();
        assert!(manager.is_enabled());
    }

    #[test]
    fn test_page_table_entry() {
        let mut pte = PageTableEntry { value: 0 };
        pte.set_phys_addr(0x1000);
        assert_eq!(pte.phys_addr(), 0x1000);

        pte.set_writable(true);
        assert!((pte.value & (1 << 1)) != 0);
    }
}
