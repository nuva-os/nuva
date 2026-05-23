/*
 * Nuva OS - HAL - LoongArch64 MMU
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */

// LoongArch64 MMU and page table management

/// Page size (4KB)
pub const PAGE_SIZE: u64 = 4096;

/// Page shift
pub const PAGE_SHIFT: u64 = 12;

/// Number of page table entries per table
pub const PTE_COUNT: usize = 512;

/// Number of page table levels (3-level: L0 -> L1 -> L2)
pub const PAGE_LEVELS: usize = 3;

// ============================================================================
// Page Table Entry (PTE) bit constants for LoongArch64
// ============================================================================

/// PTE: Valid bit (bit 0)
pub const PTE_V: u64 = 1 << 0;
/// PTE: Readable bit (bit 1)
pub const PTE_R: u64 = 1 << 1;
/// PTE: Writable bit (bit 2)
pub const PTE_W: u64 = 1 << 2;
/// PTE: Executable bit (bit 3)
pub const PTE_X: u64 = 1 << 3;
/// PTE: Global bit (bit 4) - not flushed on ASID change
pub const PTE_G: u64 = 1 << 4;
/// PTE: Dirty (modified) bit (bit 5)
pub const PTE_D: u64 = 1 << 5;
/// PTE: PLV (Privilege Level) bits (bits 7:6)
pub const PTE_PLV_SHIFT: u64 = 6;
/// PTE: PLV mask
pub const PTE_PLV_MASK: u64 = 0b11 << PTE_PLV_SHIFT;
/// PTE: PLV0 (kernel mode)
pub const PTE_PLV0: u64 = 0 << PTE_PLV_SHIFT;
/// PTE: PLV3 (user mode)
pub const PTE_PLV3: u64 = 3 << PTE_PLV_SHIFT;

/// PTE: Physical page number shift (bits 12:63 contain the physical page number)
pub const PTE_PPN_SHIFT: u64 = 12;
/// PTE: Physical page number mask
pub const PTE_PPN_MASK: u64 = !0 << PTE_PPN_SHIFT;

/// PTE: Huge page bit (bit 6 for LoongArch64 large page)
pub const PTE_HUGE: u64 = 1 << 6;

/// LoongArch64 Page Table Entry
#[derive(Debug, Clone, Copy)]
pub struct Pte {
    /// Raw PTE value
    pub value: u64,
}

impl Pte {
    /// Create an empty (invalid) PTE
    pub const fn new() -> Self {
        Pte { value: 0 }
    }

    /// Create a PTE from a raw value
    pub const fn from(value: u64) -> Self {
        Pte { value }
    }

    /// Check if the PTE is valid (present)
    pub fn is_valid(&self) -> bool {
        self.value & PTE_V != 0
    }

    /// Check if readable
    pub fn is_readable(&self) -> bool {
        self.value & PTE_R != 0
    }

    /// Check if writable
    pub fn is_writable(&self) -> bool {
        self.value & PTE_W != 0
    }

    /// Check if executable
    pub fn is_executable(&self) -> bool {
        self.value & PTE_X != 0
    }

    /// Check if global
    pub fn is_global(&self) -> bool {
        self.value & PTE_G != 0
    }

    /// Check if dirty (modified)
    pub fn is_dirty(&self) -> bool {
        self.value & PTE_D != 0
    }

    /// Check if huge page
    pub fn is_huge(&self) -> bool {
        self.value & PTE_HUGE != 0
    }

    /// Get physical page number (PPN)
    pub fn get_ppn(&self) -> u64 {
        (self.value & PTE_PPN_MASK) >> PTE_PPN_SHIFT
    }

    /// Get physical address from PTE
    pub fn get_phys(&self) -> u64 {
        self.value & PTE_PPN_MASK
    }

    /// Set physical address in PTE
    pub fn set_phys(&mut self, phys: u64) {
        self.value = (self.value & !PTE_PPN_MASK) | (phys & PTE_PPN_MASK);
    }

    /// Create a leaf PTE mapping a physical page with given flags
    pub fn create_page(phys: u64, readable: bool, writable: bool, executable: bool, user: bool) -> Self {
        let mut val = PTE_V | PTE_D;
        if readable { val |= PTE_R; }
        if writable { val |= PTE_W; }
        if executable { val |= PTE_X; }
        if user { val |= PTE_PLV3; } else { val |= PTE_PLV0; }
        val |= phys & PTE_PPN_MASK;
        Pte { value: val }
    }

    /// Create a non-leaf (page directory) PTE pointing to the next level table
    pub fn create_table(next_table_phys: u64) -> Self {
        Pte {
            value: PTE_V | PTE_R | PTE_W | (next_table_phys & PTE_PPN_MASK),
        }
    }
}

/// LoongArch64 Page Table (array of 512 PTEs)
pub struct PageTable {
    /// Page table entries
    pub entries: [Pte; PTE_COUNT],
}

impl PageTable {
    /// Create an empty page table
    pub const fn new() -> Self {
        PageTable {
            entries: [Pte::new(); PTE_COUNT],
        }
    }

    /// Get PTE at index
    pub fn get_entry(&self, index: usize) -> &Pte {
        &self.entries[index]
    }

    /// Get mutable PTE at index
    pub fn get_entry_mut(&mut self, index: usize) -> &mut Pte {
        &mut self.entries[index]
    }

    /// Set PTE at index
    pub fn set_entry(&mut self, index: usize, entry: Pte) {
        self.entries[index] = entry;
    }
}

/// Calculate page table index for a given level (LoongArch64 3-level)
/// Level 0 (L0): bits 38:30 (9 bits)
/// Level 1 (L1): bits 29:21 (9 bits)
/// Level 2 (L2): bits 20:12 (9 bits)
pub fn get_pt_index(vaddr: u64, level: usize) -> usize {
    let shift = PAGE_SHIFT as usize + (PAGE_LEVELS - 1 - level) * 9;
    ((vaddr >> shift) & 0x1FF) as usize
}

/// Parse virtual address into 3-level page table indices
pub fn parse_vaddr(vaddr: u64) -> (usize, usize, usize) {
    let l0_idx = get_pt_index(vaddr, 0);
    let l1_idx = get_pt_index(vaddr, 1);
    let l2_idx = get_pt_index(vaddr, 2);
    (l0_idx, l1_idx, l2_idx)
}

/// Get mutable reference to page table from physical address
pub fn get_page_table_mut(phys: u64) -> &'static mut PageTable {
    // SAFETY: The caller guarantees that phys points to a valid, mapped PageTable
    // structure aligned to PAGE_SIZE. The returned reference is valid for the
    // lifetime of the page table mapping.
    unsafe { &mut *(phys as *mut PageTable) }
}

/// Get immutable reference to page table from physical address
pub fn get_page_table(phys: u64) -> &'static PageTable {
    // SAFETY: The caller guarantees that phys points to a valid, mapped PageTable
    // structure aligned to PAGE_SIZE.
    unsafe { &*(phys as *const PageTable) }
}

/// Zero all entries in a page table
pub fn clear_page_table(phys: u64) {
    let table = get_page_table_mut(phys);
    for entry in table.entries.iter_mut() {
        entry.value = 0;
    }
}

/// Flush a single TLB entry for the given virtual address
fn flush_tlb_addr(vaddr: u64) {
    // SAFETY: INVTLB with operand 0x1 invalidates a single TLB entry
    // matching the given virtual address and ASID.
    unsafe {
        core::arch::asm!("invtlb 0x1, $r0, {}", in(reg) vaddr);
    }
}

// ============================================================================
// LoongArch64 Page Table Operations
// ============================================================================

/// LoongArch64 3-level page table
pub struct LoongArch64PageTable;

impl LoongArch64PageTable {
    /// Create a new page table (allocate L0 root directory)
    pub fn create() -> u64 {
        // SAFETY: Allocating a page for the L0 page directory; the returned
        // physical address will be zeroed and used as the page table root.
        let page_phys = crate::kernel::mm::page_alloc::alloc_page() as u64;
        if page_phys == 0 {
            return 0;
        }
        clear_page_table(page_phys);
        page_phys
    }

    /// Map a virtual page to a physical page
    pub fn map(l0: u64, vaddr: u64, paddr: u64, readable: bool, writable: bool, executable: bool, user: bool) {
        let (l0_idx, l1_idx, l2_idx) = parse_vaddr(vaddr);

        // Walk/create L0 -> L1 -> L2
        let l0_table = get_page_table_mut(l0);
        let l0_entry = l0_table.get_entry_mut(l0_idx);
        let l1_phys = if l0_entry.is_valid() {
            l0_entry.get_phys()
        } else {
            // SAFETY: Allocating a new L1 page table page.
            let new_phys = crate::kernel::mm::page_alloc::alloc_page() as u64;
            if new_phys == 0 { return; }
            clear_page_table(new_phys);
            *l0_entry = Pte::create_table(new_phys);
            new_phys
        };

        let l1_table = get_page_table_mut(l1_phys);
        let l1_entry = l1_table.get_entry_mut(l1_idx);
        let l2_phys = if l1_entry.is_valid() {
            l1_entry.get_phys()
        } else {
            // SAFETY: Allocating a new L2 page table page.
            let new_phys = crate::kernel::mm::page_alloc::alloc_page() as u64;
            if new_phys == 0 { return; }
            clear_page_table(new_phys);
            *l1_entry = Pte::create_table(new_phys);
            new_phys
        };

        // Set leaf PTE in L2
        let l2_table = get_page_table_mut(l2_phys);
        let l2_entry = l2_table.get_entry_mut(l2_idx);
        *l2_entry = Pte::create_page(paddr, readable, writable, executable, user);
    }

    /// Unmap a virtual page
    pub fn unmap(l0: u64, vaddr: u64) -> Option<u64> {
        let (l0_idx, l1_idx, l2_idx) = parse_vaddr(vaddr);

        let l0_table = get_page_table(l0);
        let l0_entry = l0_table.get_entry(l0_idx);
        if !l0_entry.is_valid() {
            return None;
        }

        let l1_table = get_page_table(l0_entry.get_phys());
        let l1_entry = l1_table.get_entry(l1_idx);
        if !l1_entry.is_valid() {
            return None;
        }

        let l2_table = get_page_table_mut(l1_entry.get_phys());
        let l2_entry = l2_table.get_entry(l2_idx);
        if !l2_entry.is_valid() {
            return None;
        }

        let phys = l2_entry.get_phys();
        l2_table.set_entry(l2_idx, Pte::new());
        flush_tlb_addr(vaddr);
        Some(phys)
    }

    /// Translate a virtual address to a physical address
    pub fn translate(l0: u64, vaddr: u64) -> Option<u64> {
        let (l0_idx, l1_idx, l2_idx) = parse_vaddr(vaddr);

        let l0_table = get_page_table(l0);
        let l0_entry = l0_table.get_entry(l0_idx);
        if !l0_entry.is_valid() {
            return None;
        }

        let l1_table = get_page_table(l0_entry.get_phys());
        let l1_entry = l1_table.get_entry(l1_idx);
        if !l1_entry.is_valid() {
            // Check for huge page at L1 (1GB page)
            if l1_entry.is_huge() {
                // For huge pages, the physical address includes lower bits of vaddr
                let offset = vaddr & ((1 << 21) - 1);
                return Some(l1_entry.get_phys() + offset);
            }
            return None;
        }

        let l2_table = get_page_table(l1_entry.get_phys());
        let l2_entry = l2_table.get_entry(l2_idx);
        if !l2_entry.is_valid() {
            return None;
        }

        // For regular 4KB pages, add the page offset from the virtual address
        let offset = vaddr & (PAGE_SIZE - 1);
        Some(l2_entry.get_phys() + offset)
    }
}

// ============================================================================
// MMU Initialization and Control
// ============================================================================

/// Configure page table base address (PGDL/PGDH)
fn configure_page_table_base(pgdl: u64, pgdh: u64) {
    // SAFETY: Writing to PGDL (Direct Mapping Low) and PGDH (Direct Mapping High)
    // CSR registers sets the page table root pointers. The caller must ensure
    // pgdl and pgdh point to valid page directories.
    unsafe {
        core::arch::asm!("csrwr {}, 0x81", in(reg) pgdl);
        core::arch::asm!("csrwr {}, 0x82", in(reg) pgdh);
    }
}

/// Flush all TLB entries (INVTLB all)
fn flush_tlb_all() {
    // SAFETY: INVTLB with operand 0x0 invalidates all TLB entries.
    // This is required after modifying page tables to prevent stale translations.
    unsafe {
        core::arch::asm!("invtlb 0x0, $r0, $r0");
    }
}

/// Enable MMU by setting CRMD.DA=0 and CRMD.PG=1
fn enable_mmu() {
    let crmd: u32;
    // SAFETY: Reading and writing CRMD (Current Mode Definition) CSR
    // to enable paging (PG bit) and disable direct address mode (DA bit).
    unsafe {
        core::arch::asm!("csrrd {}, 0x0", out(reg) crmd);
        // Clear DA bit (bit 3), set PG bit (bit 4)
        let crmd_new = (crmd & !(1 << 3)) | (1 << 4);
        core::arch::asm!("csrwr {}, 0x0", in(reg) crmd_new);
    }
}

/// Initialize LoongArch64 MMU
pub fn init_mmu() {
    // Set page table base (0 as placeholder; platform code sets real values)
    configure_page_table_base(0, 0);
    // Flush TLB to remove any stale entries
    flush_tlb_all();
    // Enable MMU paging
    enable_mmu();
}
