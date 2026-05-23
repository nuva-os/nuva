/*
 * Nuva OS - Kernel - Kernel
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


/// Page size
pub const PAGE_SIZE: u64 = 4096;

/// Page offset bits
pub const PAGE_SHIFT: u64 = 12;

/// Page table levels
pub const PAGE_LEVELS: usize = 4;

/// Page table entry flags
pub mod pte_flags {
    pub const PRESENT: u64 = 1 << 0;
    pub const WRITABLE: u64 = 1 << 1;
    pub const USER: u64 = 1 << 2;
    pub const WRITE_THROUGH: u64 = 1 << 3;
    pub const CACHE_DISABLE: u64 = 1 << 4;
    pub const ACCESSED: u64 = 1 << 5;
    pub const DIRTY: u64 = 1 << 6;
    pub const HUGE: u64 = 1 << 7;
    pub const GLOBAL: u64 = 1 << 8;
    pub const NO_EXECUTE: u64 = 1 << 63;
}

/// Page table entry
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

    /// Check if present
    pub fn is_present(&self) -> bool {
        self.value & pte_flags::PRESENT != 0
    }

    /// Check if writable
    pub fn is_writable(&self) -> bool {
        self.value & pte_flags::WRITABLE != 0
    }

    /// Check if user accessible
    pub fn is_user(&self) -> bool {
        self.value & pte_flags::USER != 0
    }

    /// Check if huge page
    pub fn is_huge(&self) -> bool {
        self.value & pte_flags::HUGE != 0
    }

    /// Get physical address
    pub fn get_phys(&self) -> u64 {
        self.value & 0x000F_FFFF_FFFF_F000
    }

    /// Set physical address
    pub fn set_phys(&mut self, phys: u64) {
        self.value = (self.value & 0xFFF) | (phys & 0x000F_FFFF_FFFF_F000);
    }

    /// Set flags
    pub fn set_flags(&mut self, flags: u64) {
        self.value |= flags;
    }

    /// Clear flags
    pub fn clear_flags(&mut self, flags: u64) {
        self.value &= !flags;
    }

    /// Create page table entry
    pub fn create_table(phys: u64) -> Self {
        PageTableEntry {
            value: phys | pte_flags::PRESENT | pte_flags::WRITABLE | pte_flags::USER,
        }
    }

    /// Create page entry
    pub fn create_page(phys: u64, flags: u64) -> Self {
        PageTableEntry {
            value: phys | flags | pte_flags::PRESENT,
        }
    }
}

/// Page table
pub struct PageTable {
    /// Page table entry array
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

    /// Get mutable page table entry
    pub fn get_entry_mut(&mut self, index: usize) -> &mut PageTableEntry {
        &mut self.entries[index]
    }

    /// Set page table entry
    pub fn set_entry(&mut self, index: usize, entry: PageTableEntry) {
        self.entries[index] = entry;
    }
}

/// Calculate page table index
pub fn get_pgt_index(virt: u64, level: usize) -> usize {
    let shift = PAGE_SHIFT + (3 - level) * 9;
    ((virt >> shift) & 0x1FF) as usize
}

/// Parse virtual address to get page table indices
/// @param vaddr: Virtual address
/// @return Tuple of (PML4 index, PDPT index, PD index, PT index)
pub fn parse_vaddr(vaddr: u64) -> (usize, usize, usize, usize) {
    let pml4_idx = get_pgt_index(vaddr, 0);
    let pdpt_idx = get_pgt_index(vaddr, 1);
    let pd_idx = get_pgt_index(vaddr, 2);
    let pt_idx = get_pgt_index(vaddr, 3);
    (pml4_idx, pdpt_idx, pd_idx, pt_idx)
}

/// Get page table from physical address
/// @param phys: Physical address of page table
/// @return Mutable reference to PageTable
pub fn get_page_table_mut(phys: u64) -> &'static mut PageTable {
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
    if entry.is_present() {
        entry.get_phys()
    } else {
        let next_phys = crate::mm::page_alloc::alloc_page().as_u64();
        clear_page_table(next_phys);
        *entry = PageTableEntry::create_table(next_phys);
        next_phys
    }
}

/// Map a page in the page table
/// @param pml4: Physical address of PML4
/// @param vaddr: Virtual address
/// @param paddr: Physical address
/// @param flags: Page table entry flags
/// @param page_size: Page size (currently only 4KB supported)
pub fn page_table_map_impl(pml4: u64, vaddr: u64, paddr: u64, flags: u64, page_size: u64) {
    let (pml4_idx, pdpt_idx, pd_idx, pt_idx) = parse_vaddr(vaddr);

    let pml4_table = get_page_table_mut(pml4);
    let pml4_entry = pml4_table.get_entry_mut(pml4_idx);
    let pdpt_phys = create_next_level(pml4_entry);

    let pdpt_table = get_page_table_mut(pdpt_phys);
    let pdpt_entry = pdpt_table.get_entry_mut(pdpt_idx);
    let pd_phys = create_next_level(pdpt_entry);

    let pd_table = get_page_table_mut(pd_phys);
    let pd_entry = pd_table.get_entry_mut(pd_idx);
    let pt_phys = create_next_level(pd_entry);

    let pt_table = get_page_table_mut(pt_phys);
    let pt_entry = pt_table.get_entry_mut(pt_idx);

    *pt_entry = PageTableEntry::create_page(paddr, flags);
}

/// Unmap a page from the page table
/// @param pml4: Physical address of PML4
/// @param vaddr: Virtual address
/// @return Option<PhysAddr> Physical address of the unmapped page, or None if not mapped
pub fn page_table_unmap_impl(pml4: u64, vaddr: u64) -> Option<u64> {
    let (pml4_idx, pdpt_idx, pd_idx, pt_idx) = parse_vaddr(vaddr);

    let pml4_table = get_page_table(pml4);
    let pml4_entry = pml4_table.get_entry(pml4_idx);
    if !pml4_entry.is_present() {
        return None;
    }

    let pdpt_table = get_page_table(pml4_entry.get_phys());
    let pdpt_entry = pdpt_table.get_entry(pdpt_idx);
    if !pdpt_entry.is_present() {
        return None;
    }

    let pd_table = get_page_table(pdpt_entry.get_phys());
    let pd_entry = pd_table.get_entry(pd_idx);
    if !pd_entry.is_present() {
        return None;
    }

    let pt_table = get_page_table_mut(pd_entry.get_phys());
    let pt_entry = pt_table.get_entry(pt_idx);
    if !pt_entry.is_present() {
        return None;
    }

    let phys = pt_entry.get_phys();
    pt_table.set_entry(pt_idx, PageTableEntry::new());

    Some(phys)
}

/// Translate virtual address to physical address
/// @param pml4: Physical address of PML4
/// @param vaddr: Virtual address
/// @return Option<PhysAddr> Physical address, or None if not mapped
pub fn page_table_translate_impl(pml4: u64, vaddr: u64) -> Option<u64> {
    let (pml4_idx, pdpt_idx, pd_idx, pt_idx) = parse_vaddr(vaddr);

    let pml4_table = get_page_table(pml4);
    let pml4_entry = pml4_table.get_entry(pml4_idx);
    if !pml4_entry.is_present() {
        return None;
    }

    let pdpt_table = get_page_table(pml4_entry.get_phys());
    let pdpt_entry = pdpt_table.get_entry(pdpt_idx);
    if !pdpt_entry.is_present() {
        return None;
    }

    let pd_table = get_page_table(pdpt_entry.get_phys());
    let pd_entry = pd_table.get_entry(pd_idx);
    if !pd_entry.is_present() {
        return None;
    }

    let pt_table = get_page_table(pd_entry.get_phys());
    let pt_entry = pt_table.get_entry(pt_idx);
    if !pt_entry.is_present() {
        return None;
    }

    Some(pt_entry.get_phys())
}

/// Initialize MMU
pub fn init_mmu() {
    log_info!("x86-64 MMU initialized");
}
