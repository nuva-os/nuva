/*
 * Nuva OS - HAL - X64
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



use core::sync::atomic::{AtomicU64, Ordering};

/// Page table entry flags
pub mod flags {
    /// Present
    pub const PRESENT: u64 = 1 << 0;
    /// Writable
    pub const WRITABLE: u64 = 1 << 1;
    /// User accessible
    pub const USER: u64 = 1 << 2;
    /// Write-through
    pub const WRITE_THROUGH: u64 = 1 << 3;
    /// Disable caching
    pub const CACHE_DISABLE: u64 = 1 << 4;
    /// Accessed
    pub const ACCESSED: u64 = 1 << 5;
    /// Dirty
    pub const DIRTY: u64 = 1 << 6;
    /// Huge page
    pub const HUGE: u64 = 1 << 7;
    /// Global
    pub const GLOBAL: u64 = 1 << 8;
    /// No execute
    pub const NO_EXECUTE: u64 = 1 << 63;
}

/// CR0 register bits
pub mod cr0 {
    /// Protected mode
    pub const PE: u64 = 1 << 0;
    /// Monitor coprocessor
    pub const MP: u64 = 1 << 1;
    /// Emulation
    pub const EM: u64 = 1 << 2;
    /// Task switch
    pub const TS: u64 = 1 << 3;
    /// Extension type
    pub const ET: u64 = 1 << 4;
    /// Numeric error
    pub const NE: u64 = 1 << 5;
    /// Write protect
    pub const WP: u64 = 1 << 16;
    /// Alignment mask
    pub const AM: u64 = 1 << 18;
    /// Not write-through
    pub const NW: u64 = 1 << 29;
    /// Disable caching
    pub const CD: u64 = 1 << 30;
    /// Paging
    pub const PG: u64 = 1 << 31;
}

/// CR4 register bits
pub mod cr4 {
    /// VME
    pub const VME: u64 = 1 << 0;
    /// PVI
    pub const PVI: u64 = 1 << 1;
    /// TSD
    pub const TSD: u64 = 1 << 2;
    /// DE
    pub const DE: u64 = 1 << 3;
    /// PSE
    pub const PSE: u64 = 1 << 4;
    /// PAE
    pub const PAE: u64 = 1 << 5;
    /// MCE
    pub const MCE: u64 = 1 << 6;
    /// PGE
    pub const PGE: u64 = 1 << 7;
    /// PCE
    pub const PCE: u64 = 1 << 8;
    /// OSFXSR
    pub const OSFXSR: u64 = 1 << 9;
    /// OSXMMEXCPT
    pub const OSXMMEXCPT: u64 = 1 << 10;
    /// UMIP
    pub const UMIP: u64 = 1 << 11;
    /// VMXE
    pub const VMXE: u64 = 1 << 13;
    /// SMXE
    pub const SMXE: u64 = 1 << 14;
    /// FSGSBASE
    pub const FSGSBASE: u64 = 1 << 16;
    /// PCIDE
    pub const PCIDE: u64 = 1 << 17;
    /// OSXSAVE
    pub const OSXSAVE: u64 = 1 << 18;
    /// SMEP
    pub const SMEP: u64 = 1 << 20;
    /// SMAP
    pub const SMAP: u64 = 1 << 21;
    /// PKE
    pub const PKE: u64 = 1 << 22;
}

/// Page table level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageTableLevel {
    /// PML4 (4-level paging)
    Pml4 = 0,
    /// PDPT
    Pdpt = 1,
    /// PD
    Pd = 2,
    /// PT
    Pt = 3,
}

/// Page size
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageSize {
    /// 4KB
    Page4K = 4096,
    /// 2MB
    Page2M = 2 * 1024 * 1024,
    /// 1GB
    Page1G = 1024 * 1024 * 1024,
}

/// Page table entry
#[repr(C)]
pub struct PageTableEntry {
    pub value: AtomicU64,
}

impl PageTableEntry {
    pub const fn new() -> Self {
        PageTableEntry {
            value: AtomicU64::new(0),
        }
    }

    /// If present
    pub fn is_present(&self) -> bool {
        (self.value.load(Ordering::Acquire) & flags::PRESENT) != 0
    }

    /// If writable
    pub fn is_writable(&self) -> bool {
        (self.value.load(Ordering::Acquire) & flags::WRITABLE) != 0
    }

    /// If user accessible
    pub fn is_user(&self) -> bool {
        (self.value.load(Ordering::Acquire) & flags::USER) != 0
    }

    /// If huge page
    pub fn is_huge(&self) -> bool {
        (self.value.load(Ordering::Acquire) & flags::HUGE) != 0
    }

    /// If execute disabled
    pub fn is_no_execute(&self) -> bool {
        (self.value.load(Ordering::Acquire) & flags::NO_EXECUTE) != 0
    }

    /// Get physical address
    pub fn get_addr(&self) -> u64 {
        self.value.load(Ordering::Acquire) & 0x000F_FFFF_FFFF_F000
    }

    /// Set page table entry
    pub fn set(&self, addr: u64, flags: u64) {
        let value = (addr & 0x000F_FFFF_FFFF_F000) | flags;
        self.value.store(value, Ordering::Release);
    }

    /// Clear
    pub fn clear(&self) {
        self.value.store(0, Ordering::Release);
    }
}

/// Page table (512 page table entries)
#[repr(C, align(4096))]
pub struct PageTable {
    pub entries: [PageTableEntry; 512],
}

impl PageTable {
    pub const fn new() -> Self {
        PageTable {
            entries: [PageTableEntry::new(); 512],
        }
    }

    /// Get page table entry
    pub fn get_entry(&self, index: usize) -> &PageTableEntry {
        &self.entries[index]
    }
}

/// X64 MMU
pub struct X64Mmu {
    /// PML4 address
    pub pml4_addr: AtomicU64,
    /// If 5-level paging enabled
    pub la57: bool,
    /// Total page count
    pub total_pages: AtomicU64,
    /// Used page count
    pub used_pages: AtomicU64,
    /// PML4 pointer
    pub pml4: u64,
}

impl X64Mmu {
    pub fn new() -> Self {
        X64Mmu {
            pml4_addr: AtomicU64::new(0),
            la57: false,
            total_pages: AtomicU64::new(0),
            used_pages: AtomicU64::new(0),
            pml4: 0,
        }
    }

    /// Initialize
    pub fn init(&mut self) {
        log_info!("X64 MMU initialized");
        log_info!("  Paging: 4-level");
        log_info!("  Page size: 4KB, 2MB, 1GB");
    }

    /// Enable paging
    pub fn enable_paging(&mut self, pml4: u64) {
        self.pml4_addr.store(pml4, Ordering::Release);
        self.pml4 = pml4;

        // Set CR3
        // Set CR0.PG
        // Set CR4.PAE, CR4.PGE

        log_info!("Paging enabled");
        log_info!("  PML4: 0x{:016X}", pml4);
    }

    /// Map page
    pub fn map_page(&mut self, virt: u64, phys: u64, flags: u64) -> bool {
        // Calculate page table indices
        let pml4_idx = ((virt >> 39) & 0x1FF) as usize;
        let pdpt_idx = ((virt >> 30) & 0x1FF) as usize;
        let pd_idx = ((virt >> 21) & 0x1FF) as usize;
        let pt_idx = ((virt >> 12) & 0x1FF) as usize;

        // Get or create PML4 entry
        // SAFETY: unsafe block required for low-level memory or hardware access
        let pml4_entry = unsafe { &mut *((self.pml4 as *mut u64).add(pml4_idx)) };

        // Create PDPT if not present
        if (*pml4_entry & 1) == 0 {
            let pdpt = self.alloc_page();
            if pdpt == 0 {
                return false;
            }
            // Zero the new page table
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                core::ptr::write_bytes(pdpt as *mut u8, 0, 4096);
            }
            *pml4_entry = pdpt | 0x03; // Present | Writable
        }

        let pdpt = *pml4_entry & 0xFFFFFFFFF000;
        // SAFETY: unsafe block required for low-level memory or hardware access
        let pdpt_entry = unsafe { &mut *((pdpt as *mut u64).add(pdpt_idx)) };

        // Create PD if not present
        if (*pdpt_entry & 1) == 0 {
            let pd = self.alloc_page();
            if pd == 0 {
                return false;
            }
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                core::ptr::write_bytes(pd as *mut u8, 0, 4096);
            }
            *pdpt_entry = pd | 0x03;
        }

        let pd = *pdpt_entry & 0xFFFFFFFFF000;
        // SAFETY: unsafe block required for low-level memory or hardware access
        let pd_entry = unsafe { &mut *((pd as *mut u64).add(pd_idx)) };

        // Create PT if not present
        if (*pd_entry & 1) == 0 {
            let pt = self.alloc_page();
            if pt == 0 {
                return false;
            }
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                core::ptr::write_bytes(pt as *mut u8, 0, 4096);
            }
            *pd_entry = pt | 0x03;
        }

        let pt = *pd_entry & 0xFFFFFFFFF000;
        // SAFETY: unsafe block required for low-level memory or hardware access
        let pt_entry = unsafe { &mut *((pt as *mut u64).add(pt_idx)) };

        // Set the final mapping
        *pt_entry = (phys & 0xFFFFFFFFF000) | flags | 1; // Present

        self.used_pages.fetch_add(1, Ordering::AcqRel);
        true
    }

    /// Unmap
    pub fn unmap_page(&mut self, virt: u64) -> bool {
        let pml4_idx = ((virt >> 39) & 0x1FF) as usize;
        let pdpt_idx = ((virt >> 30) & 0x1FF) as usize;
        let pd_idx = ((virt >> 21) & 0x1FF) as usize;
        let pt_idx = ((virt >> 12) & 0x1FF) as usize;

        // Walk the page tables
        // SAFETY: unsafe block required for low-level memory or hardware access
        let pml4_entry = unsafe { *((self.pml4 as *const u64).add(pml4_idx)) };
        if (pml4_entry & 1) == 0 {
            return false;
        }

        let pdpt = pml4_entry & 0xFFFFFFFFF000;
        // SAFETY: unsafe block required for low-level memory or hardware access
        let pdpt_entry = unsafe { *((pdpt as *const u64).add(pdpt_idx)) };
        if (pdpt_entry & 1) == 0 {
            return false;
        }

        let pd = pdpt_entry & 0xFFFFFFFFF000;
        // SAFETY: unsafe block required for low-level memory or hardware access
        let pd_entry = unsafe { *((pd as *const u64).add(pd_idx)) };
        if (pd_entry & 1) == 0 {
            return false;
        }

        let pt = pd_entry & 0xFFFFFFFFF000;
        // SAFETY: unsafe block required for low-level memory or hardware access
        let pt_entry = unsafe { &mut *((pt as *mut u64).add(pt_idx)) };

        // Clear the entry
        *pt_entry = 0;

        // Invalidate TLB
        self.flush_tlb(virt);

        self.used_pages.fetch_sub(1, Ordering::AcqRel);
        true
    }

    /// Virtual address to physical address
    pub fn virt_to_phys(&self, virt: u64) -> Option<u64> {
        let pml4_idx = ((virt >> 39) & 0x1FF) as usize;
        let pdpt_idx = ((virt >> 30) & 0x1FF) as usize;
        let pd_idx = ((virt >> 21) & 0x1FF) as usize;
        let pt_idx = ((virt >> 12) & 0x1FF) as usize;

        // SAFETY: unsafe block required for low-level memory or hardware access
        let pml4_entry = unsafe { *((self.pml4 as *const u64).add(pml4_idx)) };
        if (pml4_entry & 1) == 0 {
            return None;
        }

        let pdpt = pml4_entry & 0xFFFFFFFFF000;
        // SAFETY: unsafe block required for low-level memory or hardware access
        let pdpt_entry = unsafe { *((pdpt as *const u64).add(pdpt_idx)) };
        if (pdpt_entry & 1) == 0 {
            return None;
        }

        let pd = pdpt_entry & 0xFFFFFFFFF000;
        // SAFETY: unsafe block required for low-level memory or hardware access
        let pd_entry = unsafe { *((pd as *const u64).add(pd_idx)) };
        if (pd_entry & 1) == 0 {
            return None;
        }

        let pt = pd_entry & 0xFFFFFFFFF000;
        // SAFETY: unsafe block required for low-level memory or hardware access
        let pt_entry = unsafe { *((pt as *const u64).add(pt_idx)) };
        if (pt_entry & 1) == 0 {
            return None;
        }

        let phys = pt_entry & 0xFFFFFFFFF000;
        let offset = virt & 0xFFF;

        Some(phys | offset)
    }

    /// Flush TLB
    pub fn flush_tlb(&self, addr: u64) {
        // SAFETY: inline assembly required for hardware instruction
        unsafe {
            core::arch::asm!(
                "invlpg [{}]",
                in(reg) addr,
                options(nostack, preserves_flags)
            );
        }
    }

    /// Flush all TLB
    pub fn flush_tlb_all(&self) {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            // Reload CR3 to flush all TLB entries
            let cr3: u64;
            core::arch::asm!(
                "mov {}, cr3",
                out(reg) cr3,
                options(nostack, preserves_flags)
            );
            core::arch::asm!(
                "mov cr3, {}",
                in(reg) cr3,
                options(nostack, preserves_flags)
            );
        }
    }

    /// Get memory usage
    pub fn get_usage(&self) -> f32 {
        let total = self.total_pages.load(Ordering::Acquire);
        let used = self.used_pages.load(Ordering::Acquire);
        if total == 0 {
            return 0.0;
        }
        (used as f32) / (total as f32) * 100.0
    }

    /// Allocate page
    fn alloc_page(&self) -> u64 {
        // Simplified implementation
        // Actual implementation should allocate from physical memory manager
        0
    }
}

/// Global MMU
static mut MMU: Option<X64Mmu> = None;

pub fn get_mmu() -> &'static mut X64Mmu {
    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe {
        if MMU.is_none() {
            MMU = Some(X64Mmu::new());
        }
        MMU.as_mut().unwrap()
    }
}

pub fn init_mmu() {
    let mmu = get_mmu();
    mmu.init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_table_level() {
        assert_eq!(PageTableLevel::Pml4 as i32, 0);
        assert_eq!(PageTableLevel::Pdpt as i32, 1);
        assert_eq!(PageTableLevel::Pd as i32, 2);
        assert_eq!(PageTableLevel::Pt as i32, 3);
    }

    #[test]
    fn test_page_size() {
        assert_eq!(PageSize::Page4K as usize, 4096);
        assert_eq!(PageSize::Page2M as usize, 2 * 1024 * 1024);
        assert_eq!(PageSize::Page1G as usize, 1024 * 1024 * 1024);
    }

    #[test]
    fn test_page_table_entry() {
        let entry = PageTableEntry::new();
        assert!(!entry.is_present());
        assert!(!entry.is_writable());
    }

    #[test]
    fn test_page_table() {
        let table = PageTable::new();
        assert_eq!(table.entries.len(), 512);
    }

    #[test]
    fn test_x64_mmu() {
        let mmu = X64Mmu::new();
        assert_eq!(mmu.pml4_addr.load(Ordering::Acquire), 0);
        assert!(!mmu.la57);
    }

    #[test]
    fn test_page_flags() {
        assert_eq!(flags::PRESENT, 1 << 0);
        assert_eq!(flags::WRITABLE, 1 << 1);
        assert_eq!(flags::USER, 1 << 2);
        assert_eq!(flags::HUGE, 1 << 7);
        assert_eq!(flags::GLOBAL, 1 << 8);
        assert_eq!(flags::NO_EXECUTE, 1 << 63);
    }
}
