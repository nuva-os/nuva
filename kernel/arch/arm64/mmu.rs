/*
 * Nuva OS - Kernel - ARM64 MMU
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

/// Page size in bytes
pub const PAGE_SIZE: u64 = 4096;

/// Page shift (log2 of page size)
pub const PAGE_SHIFT: u64 = 12;

/// Number of page table levels
pub const PAGE_LEVELS: usize = 4;

/// Page table entry flags
pub mod pte_flags {
    pub const VALID: u64 = 1 << 0;
    pub const TABLE: u64 = 1 << 1;
    pub const USER: u64 = 1 << 6;
    pub const READONLY: u64 = 1 << 7;
    pub const SHARED: u64 = 3 << 8;
    pub const ACCESSED: u64 = 1 << 10;
    pub const DIRTY: u64 = 1 << 51;
    pub const NX: u64 = 1 << 54;
    pub const PXN: u64 = 1 << 53;
}

/// Memory attribute indices
pub mod mair {
    pub const NORMAL: u64 = 0;
    pub const NORMAL_NC: u64 = 1;
    pub const DEVICE: u64 = 2;
    pub const DEVICE_NC: u64 = 3;
}

/// Page Table Entry
/// Represents a single entry in an ARM64 page table.
#[derive(Debug, Clone, Copy)]
pub struct PageTableEntry {
    pub value: u64,
}

impl PageTableEntry {
    pub const fn new() -> Self {
        PageTableEntry { value: 0 }
    }

    pub const fn from(value: u64) -> Self {
        PageTableEntry { value }
    }

    /// Check if entry is valid
    pub fn is_valid(&self) -> bool {
        self.value & pte_flags::VALID != 0
    }

    /// Check if entry is a table (points to next level)
    pub fn is_table(&self) -> bool {
        self.value & pte_flags::TABLE != 0
    }

    /// Check if entry is a page (leaf entry)
    pub fn is_page(&self) -> bool {
        self.is_valid() && !self.is_table()
    }

    /// Get physical address from entry
    /// @return Physical address (page frame number)
    pub fn get_phys(&self) -> u64 {
        self.value & 0x0000_FFFF_FFFF_F000
    }

    /// Set physical address in entry
    /// @param phys: Physical address to set
    pub fn set_phys(&mut self, phys: u64) {
        self.value = (self.value & 0xFFF) | (phys & 0x0000_FFFF_FFFF_F000);
    }

    /// Set flags in entry
    /// @param flags: Flags to set
    pub fn set_flags(&mut self, flags: u64) {
        self.value |= flags;
    }

    /// Clear flags in entry
    /// @param flags: Flags to clear
    pub fn clear_flags(&mut self, flags: u64) {
        self.value &= !flags;
    }

    /// Create a table entry
    /// @param phys: Physical address of next level table
    /// @return New PageTableEntry pointing to next level
    pub fn create_table(phys: u64) -> Self {
        PageTableEntry {
            value: phys | pte_flags::VALID | pte_flags::TABLE,
        }
    }

    /// Create a page entry
    /// @param phys: Physical address of page
    /// @param flags: Additional flags
    /// @return New PageTableEntry for a page mapping
    pub fn create_page(phys: u64, flags: u64) -> Self {
        PageTableEntry {
            value: phys | flags | pte_flags::VALID | pte_flags::ACCESSED,
        }
    }
}

/// Page Table
/// A single level of the page table hierarchy.
pub struct PageTable {
    /// Array of page table entries
    pub entries: [PageTableEntry; 512],
}

impl PageTable {
    pub const fn new() -> Self {
        PageTable {
            entries: [PageTableEntry::new(); 512],
        }
    }

    /// Get entry at index
    /// @param index: Entry index (0-511)
    /// @return Reference to entry
    pub fn get_entry(&self, index: usize) -> &PageTableEntry {
        &self.entries[index]
    }

    /// Get mutable entry at index
    /// @param index: Entry index (0-511)
    /// @return Mutable reference to entry
    pub fn get_entry_mut(&mut self, index: usize) -> &mut PageTableEntry {
        &mut self.entries[index]
    }

    /// Set entry at index
    /// @param index: Entry index (0-511)
    /// @param entry: New entry value
    pub fn set_entry(&mut self, index: usize, entry: PageTableEntry) {
        self.entries[index] = entry;
    }
}

/// Calculate page table index for a virtual address
/// @param virt: Virtual address
/// @param level: Page table level (0-3)
/// @return Index into page table (0-511)
pub fn get_pgt_index(virt: u64, level: usize) -> usize {
    let shift = PAGE_SHIFT as usize + (3 - level) * 9;
    ((virt >> shift) & 0x1FF) as usize
}

/// TCR (Translation Control Register) configuration
pub struct TcrConfig {
    /// T0SZ: TTBR0 address size
    pub t0sz: u64,
    /// T1SZ: TTBR1 address size
    pub t1sz: u64,
    /// IRGN0: TTBR0 inner cacheability
    pub irgn0: u64,
    /// IRGN1: TTBR1 inner cacheability
    pub irgn1: u64,
    /// ORGN0: TTBR0 outer cacheability
    pub orgn0: u64,
    /// ORGN1: TTBR1 outer cacheability
    pub orgn1: u64,
    /// SH0: TTBR0 shareability
    pub sh0: u64,
    /// SH1: TTBR1 shareability
    pub sh1: u64,
    /// TG0: TTBR0 page granularity
    pub tg0: u64,
    /// TG1: TTBR1 page granularity
    pub tg1: u64,
    /// IPS: Intermediate physical address size
    pub ips: u64,
}

impl TcrConfig {
    /// Get default TCR configuration
    pub fn default() -> Self {
        TcrConfig {
            t0sz: 25,   /* 39-bit virtual address */
            t1sz: 25,
            irgn0: 1,   /* Write-back */
            irgn1: 1,
            orgn0: 1,
            orgn1: 1,
            sh0: 3,     /* Inner shareable */
            sh1: 3,
            tg0: 0,     /* 4KB pages */
            tg1: 1,     /* 4KB pages */
            ips: 5,     /* 48-bit physical address */
        }
    }

    /// Convert configuration to TCR register value
    /// @return TCR_EL1 value
    pub fn to_tcr(&self) -> u64 {
        (self.t0sz) |
        (self.t1sz << 16) |
        (self.irgn0 << 8) |
        (self.irgn1 << 24) |
        (self.orgn0 << 10) |
        (self.orgn1 << 26) |
        (self.sh0 << 12) |
        (self.sh1 << 28) |
        (self.tg0 << 14) |
        (self.tg1 << 30) |
        (self.ips << 32)
    }
}

/// Get default MAIR (Memory Attribute Indirection Register) value
/// @return MAIR_EL1 value
pub fn get_default_mair() -> u64 {
    // Attr0: Normal memory, Inner WB, Outer WB
    // Attr1: Normal memory, Non-cacheable
    // Attr2: Device memory, nGnRnE
    // Attr3: Device memory, nGnRE

    let attr0 = 0xFF;  /* Normal, WB */
    let attr1 = 0x44;  /* Normal, NC */
    let attr2 = 0x00;  /* Device, nGnRnE */
    let attr3 = 0x04;  /* Device, nGnRE */

    attr0 | (attr1 << 8) | (attr2 << 16) | (attr3 << 24)
}

/// Parse virtual address to get page table indices
/// @param vaddr: Virtual address
/// @return Tuple of (PGD index, PUD index, PMD index, PTE index)
pub fn parse_vaddr(vaddr: u64) -> (usize, usize, usize, usize) {
    let pgd_idx = get_pgt_index(vaddr, 0);
    let pud_idx = get_pgt_index(vaddr, 1);
    let pmd_idx = get_pgt_index(vaddr, 2);
    let pte_idx = get_pgt_index(vaddr, 3);
    (pgd_idx, pud_idx, pmd_idx, pte_idx)
}

/// Get page table from physical address
/// @param phys: Physical address of page table
/// @return Mutable reference to PageTable
pub fn get_page_table_mut(phys: u64) -> &'static mut PageTable {
    // TODO: Use correct virtual address mapping
    // Assume physical address maps directly to virtual address
    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe {
        &mut *(phys as *mut PageTable)
    }
}

/// Get page table from physical address (immutable)
/// @param phys: Physical address of page table
/// @return Reference to PageTable
pub fn get_page_table(phys: u64) -> &'static PageTable {
    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe {
        &*(phys as *const PageTable)
    }
}

/// Clear page table (zero all entries)
/// @param phys: Physical address of page table
pub fn clear_page_table(phys: u64) {
    let table = get_page_table_mut(phys);
    for entry in table.entries.iter_mut() {
        entry.value = 0;
    }
}

/// Create next level page table if not exists
/// @param entry: Current page table entry
/// @return Physical address of next level table
pub fn create_next_level(entry: &mut PageTableEntry) -> u64 {
    if entry.is_valid() && entry.is_table() {
        // Already exists, return directly
        entry.get_phys()
    } else {
        // Allocate new page table
        let next_phys = crate::kernel::mm::page_alloc::alloc_page() as u64;
        clear_page_table(next_phys);

        // Set to table entry
        *entry = PageTableEntry::create_table(next_phys);
        next_phys
    }
}

/// Map a page in the page table
/// @param pgd: Physical address of PGD
/// @param vaddr: Virtual address
/// @param paddr: Physical address
/// @param flags: Page table entry flags
/// @param page_size: Page size (currently only 4KB supported)
pub fn page_table_map_impl(pgd: u64, vaddr: u64, paddr: u64, flags: u64, page_size: u64) {
    // Parse virtual address
    let (pgd_idx, pud_idx, pmd_idx, pte_idx) = parse_vaddr(vaddr);

    // Get PGD
    let pgd_table = get_page_table_mut(pgd);
    let pgd_entry = pgd_table.get_entry_mut(pgd_idx);

    // Create or get PUD
    let pud_phys = create_next_level(pgd_entry);

    // Get PUD
    let pud_table = get_page_table_mut(pud_phys);
    let pud_entry = pud_table.get_entry_mut(pud_idx);

    // Create or get PMD
    let pmd_phys = create_next_level(pud_entry);

    // Get PMD
    let pmd_table = get_page_table_mut(pmd_phys);
    let pmd_entry = pmd_table.get_entry_mut(pmd_idx);

    // Create or get PTE
    let pte_phys = create_next_level(pmd_entry);

    // Get PTE
    let pte_table = get_page_table_mut(pte_phys);
    let pte_entry = pte_table.get_entry_mut(pte_idx);

    // Set final page table entry
    *pte_entry = PageTableEntry::create_page(paddr, flags);
}

/// Unmap a page from the page table
/// @param pgd: Physical address of PGD
/// @param vaddr: Virtual address
/// @return Option<PhysAddr> Physical address of the unmapped page, or None if not mapped
pub fn page_table_unmap_impl(pgd: u64, vaddr: u64) -> Option<u64> {
    // Parse virtual address
    let (pgd_idx, pud_idx, pmd_idx, pte_idx) = parse_vaddr(vaddr);

    // Get PGD
    let pgd_table = get_page_table(pgd);
    let pgd_entry = pgd_table.get_entry(pgd_idx);

    if !pgd_entry.is_valid() {
        return None;
    }

    // Get PUD
    let pud_table = get_page_table(pgd_entry.get_phys());
    let pud_entry = pud_table.get_entry(pud_idx);

    if !pud_entry.is_valid() {
        return None;
    }

    // Get PMD
    let pmd_table = get_page_table(pud_entry.get_phys());
    let pmd_entry = pmd_table.get_entry(pmd_idx);

    if !pmd_entry.is_valid() {
        return None;
    }

    // Get PTE
    let pte_table = get_page_table_mut(pmd_entry.get_phys());
    let pte_entry = pte_table.get_entry(pte_idx);

    if !pte_entry.is_valid() {
        return None;
    }

    // Save physical address
    let phys = pte_entry.get_phys();

    // Clear PTE
    pte_table.set_entry(pte_idx, PageTableEntry::new());

    Some(phys)
}

/// Translate virtual address to physical address
/// @param pgd: Physical address of PGD
/// @param vaddr: Virtual address
/// @return Option<PhysAddr> Physical address, or None if not mapped
pub fn page_table_translate_impl(pgd: u64, vaddr: u64) -> Option<u64> {
    // Parse virtual address
    let (pgd_idx, pud_idx, pmd_idx, pte_idx) = parse_vaddr(vaddr);

    // Get PGD
    let pgd_table = get_page_table(pgd);
    let pgd_entry = pgd_table.get_entry(pgd_idx);

    if !pgd_entry.is_valid() {
        return None;
    }

    // Get PUD
    let pud_table = get_page_table(pgd_entry.get_phys());
    let pud_entry = pud_table.get_entry(pud_idx);

    if !pud_entry.is_valid() {
        return None;
    }

    // Get PMD
    let pmd_table = get_page_table(pud_entry.get_phys());
    let pmd_entry = pmd_table.get_entry(pmd_idx);

    if !pmd_entry.is_valid() {
        return None;
    }

    // Get PTE
    let pte_table = get_page_table(pmd_entry.get_phys());
    let pte_entry = pte_table.get_entry(pte_idx);

    if !pte_entry.is_valid() {
        return None;
    }

    // Return physical address
    Some(pte_entry.get_phys() | (vaddr & 0xFFF))
}
