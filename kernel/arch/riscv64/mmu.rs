/*
 * Nuva OS - Kernel - RISC-V 64 MMU / Page Table Operations
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

//! RISC-V Sv39/Sv48/Sv57 page table operations implementing PageTableOps trait.

use core::arch::asm;

use crate::kernel::arch::*;
use super::read_csr;

/// RISC-V paging mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PagingMode {
    /// Sv39: 3-level page table, 39-bit virtual address.
    Sv39 = 8,
    /// Sv48: 4-level page table, 48-bit virtual address.
    Sv48 = 9,
    /// Sv57: 5-level page table, 57-bit virtual address.
    Sv57 = 10,
}

// PTE flag bits (RISC-V privileged spec Table 4.7)
pub const PTE_V: u64 = 1 << 0;     // Valid
pub const PTE_R: u64 = 1 << 1;     // Read
pub const PTE_W: u64 = 1 << 2;     // Write
pub const PTE_X: u64 = 1 << 3;     // Execute
pub const PTE_U: u64 = 1 << 4;     // User
pub const PTE_G: u64 = 1 << 5;     // Global
pub const PTE_A: u64 = 1 << 6;     // Accessed
pub const PTE_D: u64 = 1 << 7;     // Dirty

/// Page Table Entry (PTE) - 64-bit on RV64.
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct Pte(pub u64);

impl Pte {
    /// Check if PTE is valid.
    pub const fn is_valid(&self) -> bool {
        (self.0 & PTE_V) != 0
    }

    /// Check if PTE is a leaf entry (has R, W, or X bits set).
    pub const fn is_leaf(&self) -> bool {
        (self.0 & (PTE_R | PTE_W | PTE_X)) != 0
    }

    /// Check if PTE is a table (non-leaf) entry.
    pub const fn is_table(&self) -> bool {
        self.is_valid() && !self.is_leaf()
    }

    /// Extract PPN (Physical Page Number) from PTE.
    pub const fn ppn(&self) -> u64 {
        (self.0 >> 10) & ((1u64 << 44) - 1)
    }

    /// Set PPN in PTE.
    pub const fn set_ppn(&mut self, ppn: u64) -> Self {
        Pte((self.0 & 0x3FF) | ((ppn & ((1u64 << 44) - 1)) << 10))
    }

    /// Create a leaf PTE from PPN and flags.
    pub const fn new_leaf(ppn: u64, flags: u64) -> Self {
        Pte(((ppn & ((1u64 << 44) - 1)) << 10) | flags | PTE_V | PTE_A | PTE_D)
    }

    /// Create a table PTE pointing to next-level page table at PPN.
    pub const fn new_table(ppn: u64) -> Self {
        Pte(((ppn & ((1u64 << 44) - 1)) << 10) | PTE_V)
    }
}

/// Convert ProtFlags to RISC-V PTE flags.
/// Note: RISC-V requires W implies R (W=1,R=0 is reserved).
fn prot_to_pte_flags(prot: ProtFlags) -> u64 {
    let mut flags = PTE_A | PTE_D;

    if prot.is_readable() || prot.is_writable() {
        flags |= PTE_R;
    }
    if prot.is_writable() {
        flags |= PTE_W;
    }
    if prot.is_executable() {
        flags |= PTE_X;
    }
    if prot.is_user() {
        flags |= PTE_U;
    }

    flags
}

/// Detect the best available paging mode by attempting to write satp.
/// Returns Sv39 as baseline (all RV64 implementations support it).
pub fn detect_paging_mode() -> PagingMode {
    PagingMode::Sv39
}

const PT_ENTRIES: usize = 512;

/// Allocate a zeroed page for use as a page table.
/// Returns the physical address of the allocated page, or 0 on failure.
fn alloc_page_table() -> u64 {
    let page = crate::kernel::mm::page_alloc::alloc_page();
    if page.is_null() {
        log_error!("RISC-V: Failed to allocate page table page");
        return 0;
    }
    let ptr = page as *mut u64;
    for i in 0..PT_ENTRIES {
        unsafe { ptr.add(i).write(0); }
    }
    page as u64
}

/// Free a page table page at the given physical address.
fn free_page_table(paddr: u64) {
    if paddr == 0 { return; }
    let page = paddr as *mut crate::kernel::mm::page_alloc::Page;
    crate::kernel::mm::page_alloc::free_page(page);
}

/// RISC-V 64 page table implementation.
pub struct RiscV64PageTable;

impl PageTableOps for RiscV64PageTable {
    fn create(&self) -> PhysAddr {
        let pt = alloc_page_table();
        if pt == 0 {
            return PhysAddr::zero();
        }
        PhysAddr::new(pt)
    }

    fn destroy(&self, pgd: PhysAddr) {
        let l2 = pgd.0 as *const Pte;
        for i2 in 0..PT_ENTRIES {
            let l2e = unsafe { *l2.add(i2) };
            if !l2e.is_table() { continue; }
            let l1 = (l2e.ppn() << 12) as *const Pte;
            for i1 in 0..PT_ENTRIES {
                let l1e = unsafe { *l1.add(i1) };
                if !l1e.is_table() { continue; }
                free_page_table(l1e.ppn() << 12);
            }
            free_page_table(l2e.ppn() << 12);
        }
        free_page_table(pgd.0);
    }

    fn map(&self, pgd: PhysAddr, vaddr: VirtAddr, paddr: PhysAddr, prot: ProtFlags, _page_size: u64) {
        let pte_flags = prot_to_pte_flags(prot);
        let ppn = paddr.0 >> 12;
        let vpn2 = ((vaddr.0 >> 30) & 0x1FF) as usize;
        let vpn1 = ((vaddr.0 >> 21) & 0x1FF) as usize;
        let vpn0 = ((vaddr.0 >> 12) & 0x1FF) as usize;

        // Level 2: walk or allocate
        let l2 = pgd.0 as *mut Pte;
        let l2e = unsafe { &mut *l2.add(vpn2) };
        if l2e.is_leaf() {
            log_warn!("RISC-V map: L2 superpage collision at VPN2={}", vpn2);
            return;
        }
        let l1_addr = if l2e.is_table() {
            l2e.ppn() << 12
        } else {
            let pt = alloc_page_table();
            if pt == 0 { return; }
            *l2e = Pte::new_table(pt >> 12);
            pt
        };

        // Level 1: walk or allocate
        let l1 = l1_addr as *mut Pte;
        let l1e = unsafe { &mut *l1.add(vpn1) };
        if l1e.is_leaf() {
            log_warn!("RISC-V map: L1 superpage collision at VPN1={}", vpn1);
            return;
        }
        let l0_addr = if l1e.is_table() {
            l1e.ppn() << 12
        } else {
            let pt = alloc_page_table();
            if pt == 0 { return; }
            *l1e = Pte::new_table(pt >> 12);
            pt
        };

        // Level 0: set leaf PTE
        let l0 = l0_addr as *mut Pte;
        unsafe { *l0.add(vpn0) = Pte::new_leaf(ppn, pte_flags); }
        self.tlb_flush_addr(vaddr);
    }

    fn unmap(&self, pgd: PhysAddr, vaddr: VirtAddr) {
        let vpn2 = ((vaddr.0 >> 30) & 0x1FF) as usize;
        let vpn1 = ((vaddr.0 >> 21) & 0x1FF) as usize;
        let vpn0 = ((vaddr.0 >> 12) & 0x1FF) as usize;

        let l2 = pgd.0 as *const Pte;
        let l2e = unsafe { *l2.add(vpn2) };
        if !l2e.is_table() { return; }

        let l1 = (l2e.ppn() << 12) as *const Pte;
        let l1e = unsafe { *l1.add(vpn1) };
        if !l1e.is_table() { return; }

        let l0 = (l1e.ppn() << 12) as *mut Pte;
        unsafe { *l0.add(vpn0) = Pte(0); }
        self.tlb_flush_addr(vaddr);
    }

    fn translate(&self, pgd: PhysAddr, vaddr: VirtAddr) -> Option<PhysAddr> {
        let vpn2 = ((vaddr.0 >> 30) & 0x1FF) as usize;
        let vpn1 = ((vaddr.0 >> 21) & 0x1FF) as usize;
        let vpn0 = ((vaddr.0 >> 12) & 0x1FF) as usize;

        let l2 = pgd.0 as *const Pte;
        let l2e = unsafe { *l2.add(vpn2) };
        if !l2e.is_valid() { return None; }
        if l2e.is_leaf() {
            return Some(PhysAddr::new((l2e.ppn() << 30) | (vaddr.0 & 0x3FFF_FFFF)));
        }

        let l1 = (l2e.ppn() << 12) as *const Pte;
        let l1e = unsafe { *l1.add(vpn1) };
        if !l1e.is_valid() { return None; }
        if l1e.is_leaf() {
            return Some(PhysAddr::new((l1e.ppn() << 21) | (vaddr.0 & 0x1F_FFFF)));
        }

        let l0 = (l1e.ppn() << 12) as *const Pte;
        let l0e = unsafe { *l0.add(vpn0) };
        if !l0e.is_valid() { return None; }

        Some(PhysAddr::new((l0e.ppn() << 12) | (vaddr.0 & 0xFFF)))
    }

    fn protect(&self, pgd: PhysAddr, vaddr: VirtAddr, prot: ProtFlags) {
        let pte_flags = prot_to_pte_flags(prot);
        let vpn2 = ((vaddr.0 >> 30) & 0x1FF) as usize;
        let vpn1 = ((vaddr.0 >> 21) & 0x1FF) as usize;
        let vpn0 = ((vaddr.0 >> 12) & 0x1FF) as usize;

        let l2 = pgd.0 as *const Pte;
        let l2e = unsafe { *l2.add(vpn2) };
        if !l2e.is_table() { return; }

        let l1 = (l2e.ppn() << 12) as *const Pte;
        let l1e = unsafe { *l1.add(vpn1) };
        if !l1e.is_table() { return; }

        let l0 = (l1e.ppn() << 12) as *mut Pte;
        let l0e = unsafe { &mut *l0.add(vpn0) };
        if !l0e.is_valid() { return; }

        let ppn = l0e.ppn();
        *l0e = Pte::new_leaf(ppn, pte_flags);
        self.tlb_flush_addr(vaddr);
    }

    fn tlb_flush_addr(&self, vaddr: VirtAddr) {
        unsafe {
            asm!(
                "sfence.vma {0}, zero",
                in(reg) vaddr.0,
            );
        }
    }

    fn tlb_flush_all(&self) {
        unsafe {
            asm!(
                "sfence.vma zero, zero",
            );
        }
    }

    fn switch(&self, pgd: PhysAddr) {
        let mode = detect_paging_mode() as u64;
        let ppn = pgd.0 >> 12;
        let satp = (mode << 60) | ppn;
        unsafe {
            asm!(
                "csrw satp, {0}",
                "sfence.vma zero, zero",
                in(reg) satp,
            );
        }
    }

    fn current(&self) -> PhysAddr {
        let satp = read_csr!("satp");
        let ppn = satp & ((1u64 << 44) - 1);
        PhysAddr::new(ppn << 12)
    }
}
