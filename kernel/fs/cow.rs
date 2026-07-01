/*
 * Nuva OS - Kernel - Fs - Cow
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
/*
 * Nuva OS - Kernel - Copy-on-Write (COW) Mechanism
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * COW page management for NuvaFS snapshots.
 * Pages are shared until modified, at which point
 * a private copy is created for the writer.
 */

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

use crate::kernel::error::{KernelError, KernelResult};

/// Maximum COW pages tracked
pub const MAX_COW_PAGES: usize = 4096;

/// COW page entry
#[derive(Clone, Debug)]
pub struct CowPage {
    /// Physical page number
    pub pfn: u64,
    /// Reference count (shared snapshots)
    pub ref_count: AtomicU32,
    /// Whether page is frozen (snapshot-protected)
    pub frozen: bool,
    /// Original page before COW (for snapshot rollback)
    pub original_pfn: u64,
}

impl CowPage {
    /// Create a new COW page
    pub const fn new(pfn: u64) -> Self {
        CowPage {
            pfn,
            ref_count: AtomicU32::new(1),
            frozen: false,
            original_pfn: 0,
        }
    }

    /// Increment reference count
    pub fn inc_ref(&self) -> u32 {
        self.ref_count.fetch_add(1, Ordering::Relaxed) + 1
    }

    /// Decrement reference count, return new count
    pub fn dec_ref(&self) -> u32 {
        let prev = self.ref_count.load(Ordering::Acquire);
        if prev > 0 {
            self.ref_count.fetch_sub(1, Ordering::Relaxed);
            prev - 1
        } else {
            0
        }
    }

    /// Get current reference count
    pub fn ref_count(&self) -> u32 {
        self.ref_count.load(Ordering::Acquire)
    }

    /// Check if page is shared (ref_count > 1)
    pub fn is_shared(&self) -> bool {
        self.ref_count() > 1
    }
}

/// COW fault result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CowFaultResult {
    /// Page was already private, no COW needed
    AlreadyPrivate,
    /// COW triggered: new page allocated, old page preserved
    Copied { new_pfn: u64, old_pfn: u64 },
    /// COW failed: out of memory
    Oom,
}

/// CowManager: Copy-on-Write page manager
///
/// Manages COW pages for filesystem snapshots:
/// - Shared pages have ref_count > 1
/// - On write fault, shared pages are copied
/// - Frozen pages are never modified (snapshot-protected)
pub struct CowManager {
    /// Total COW faults handled
    cow_faults: AtomicU64,
    /// Total pages currently shared
    shared_pages: AtomicU64,
    /// Total COW copies made
    cow_copies: AtomicU64,
}

impl CowManager {
    /// Create a new COW manager
    pub const fn new() -> Self {
        CowManager {
            cow_faults: AtomicU64::new(0),
            shared_pages: AtomicU64::new(0),
            cow_copies: AtomicU64::new(0),
        }
    }

    /// Handle a write fault on a COW page
    ///
    /// If the page is shared (ref_count > 1), creates a
    /// private copy for the writer and preserves the
    /// original for snapshots.
    ///
    /// @param page: COW page being written
    /// @return: COW fault result
    pub fn handle_write_fault(&self, page: &mut CowPage) -> CowFaultResult {
        self.cow_faults.fetch_add(1, Ordering::Relaxed);

        if !page.is_shared() && !page.frozen {
            return CowFaultResult::AlreadyPrivate;
        }

        // Save original PFN for snapshot
        let old_pfn = page.pfn;
        page.original_pfn = old_pfn;

        // TODO: Allocate new physical page and copy contents
        // For now, assign a new PFN (placeholder)
        let new_pfn = old_pfn + 1000;

        // Decrement old page ref count
        page.dec_ref();

        // Set up new private page
        page.pfn = new_pfn;
        page.ref_count.store(1, Ordering::Release);
        page.frozen = false;

        self.cow_copies.fetch_add(1, Ordering::Relaxed);

        CowFaultResult::Copied { new_pfn, old_pfn }
    }

    /// Freeze a page for snapshot protection
    ///
    /// Frozen pages trigger COW on any write attempt.
    pub fn freeze_page(&self, page: &mut CowPage) {
        page.frozen = true;
        page.inc_ref();
        self.shared_pages.fetch_add(1, Ordering::Relaxed);
    }

    /// Unfreeze a page (snapshot released)
    pub fn unfreeze_page(&self, page: &mut CowPage) {
        if page.frozen {
            page.frozen = false;
            page.dec_ref();
            self.shared_pages.fetch_sub(1, Ordering::Relaxed);
        }
    }

    /// Get statistics
    pub fn stats(&self) -> (u64, u64, u64) {
        (
            self.cow_faults.load(Ordering::Acquire),
            self.shared_pages.load(Ordering::Acquire),
            self.cow_copies.load(Ordering::Acquire),
        )
    }
}

/// Global COW manager
static COW_MANAGER: crate::sync_oncelock::OnceLock<CowManager> = crate::sync_oncelock::OnceLock::new();

/// Get global COW manager
pub fn get_cow_manager() -> &'static CowManager {
    COW_MANAGER.get_or_init(CowManager::new)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cow_page_ref_count() {
        let page = CowPage::new(100);
        assert_eq!(page.ref_count(), 1);
        assert!(!page.is_shared());

        page.inc_ref();
        assert_eq!(page.ref_count(), 2);
        assert!(page.is_shared());

        page.dec_ref();
        assert_eq!(page.ref_count(), 1);
    }

    #[test]
    fn test_cow_fault_private() {
        let mgr = CowManager::new();
        let mut page = CowPage::new(100);
        let result = mgr.handle_write_fault(&mut page);
        assert_eq!(result, CowFaultResult::AlreadyPrivate);
    }

    #[test]
    fn test_cow_fault_shared() {
        let mgr = CowManager::new();
        let mut page = CowPage::new(100);
        page.inc_ref(); // Make shared
        let result = mgr.handle_write_fault(&mut page);
        match result {
            CowFaultResult::Copied { old_pfn, .. } => assert_eq!(old_pfn, 100),
            _ => panic!("Expected Copied"),
        }
    }

    #[test]
    fn test_freeze_unfreeze() {
        let mgr = CowManager::new();
        let mut page = CowPage::new(100);
        mgr.freeze_page(&mut page);
        assert!(page.frozen);
        assert!(page.is_shared());

        mgr.unfreeze_page(&mut page);
        assert!(!page.frozen);
    }
}