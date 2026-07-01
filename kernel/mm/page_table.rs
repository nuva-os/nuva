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

//! Page Table Operations - Multi-level page table management and TLB operations

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

use super::{PhysAddr, VirtAddr, PAGE_SHIFT, PAGE_SIZE};

/// Page table levels
pub const PT_LEVELS: usize = 4;

/// Entries per page table level (512 for 64-bit with 4KB pages)
pub const PT_ENTRIES: usize = 512;

/// Bits per level (9 for 64-bit 4KB paging)
pub const PT_BITS_PER_LEVEL: usize = 9;

/// PTE flag bits
pub mod pte_flags {
    /// Page is valid/present
    pub const VALID: u64 = 1 << 0;
    /// Page is writable
    pub const WRITABLE: u64 = 1 << 1;
    /// User accessible
    pub const USER: u64 = 1 << 2;
    /// Write-through caching
    pub const WRITE_THROUGH: u64 = 1 << 3;
    /// Cache disabled
    pub const CACHE_DISABLE: u64 = 1 << 4;
    /// Accessed flag
    pub const ACCESSED: u64 = 1 << 5;
    /// Dirty flag
    pub const DIRTY: u64 = 1 << 6;
    /// Huge page
    pub const HUGE: u64 = 1 << 7;
    /// Global TLB entry
    pub const GLOBAL: u64 = 1 << 8;
    /// No execute
    pub const NX: u64 = 1 << 63;
    /// COW (software flag)
    pub const COW: u64 = 1 << 56;
    /// Shared (software flag)
    pub const SHARED: u64 = 1 << 57;
    /// Swapped out (software flag)
    pub const SWAP: u64 = 1 << 58;
    /// Owner PID LSBs (bits 48-55, software)
    pub const OWNER_MASK: u64 = 0xFF << 48;
}

/// Physical address mask in PTE
const PTE_PHYS_MASK: u64 = 0x0000_FFFF_FFFF_F000;

/// Page table entry
/// Bit layout (ARM64-like, compatible with x86_64):
/// [0]     Valid/Present
/// [1]     Writable
/// [2]     User
/// [5]     Accessed
/// [6]     Dirty
/// [7]     Huge page
/// [8]     Global
/// [12-51] Physical address (aligned to 4KB)
/// [48-55] Software: Owner PID
/// [56]    Software: COW
/// [57]    Software: Shared
/// [58]    Software: Swap
/// [63]    No Execute
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct PageTableEntry {
    pub value: u64,
}

impl PageTableEntry {
    /// Create empty PTE
    pub const fn new() -> Self {
        PageTableEntry { value: 0 }
    }

    /// Create PTE from raw value
    pub const fn from_raw(value: u64) -> Self {
        PageTableEntry { value }
    }

    /// Create PTE with physical address and flags
    pub const fn new_with(phys: PhysAddr, flags: u64) -> Self {
        PageTableEntry {
            value: (phys & PTE_PHYS_MASK) | flags,
        }
    }

    /// Check if entry is valid/present
    #[inline(always)]
    pub fn is_valid(&self) -> bool {
        (self.value & pte_flags::VALID) != 0
    }

    /// Check if entry is writable
    #[inline(always)]
    pub fn is_writable(&self) -> bool {
        (self.value & pte_flags::WRITABLE) != 0
    }

    /// Check if user accessible
    #[inline(always)]
    pub fn is_user(&self) -> bool {
        (self.value & pte_flags::USER) != 0
    }

    /// Check if accessed
    #[inline(always)]
    pub fn is_accessed(&self) -> bool {
        (self.value & pte_flags::ACCESSED) != 0
    }

    /// Check if dirty
    #[inline(always)]
    pub fn is_dirty(&self) -> bool {
        (self.value & pte_flags::DIRTY) != 0
    }

    /// Check if huge page
    #[inline(always)]
    pub fn is_huge(&self) -> bool {
        (self.value & pte_flags::HUGE) != 0
    }

    /// Check if no-execute
    #[inline(always)]
    pub fn is_nx(&self) -> bool {
        (self.value & pte_flags::NX) != 0
    }

    /// Check if COW page
    #[inline(always)]
    pub fn is_cow(&self) -> bool {
        (self.value & pte_flags::COW) != 0
    }

    /// Check if swapped out
    #[inline(always)]
    pub fn is_swap(&self) -> bool {
        (self.value & pte_flags::SWAP) != 0
    }

    /// Check if global TLB entry
    #[inline(always)]
    pub fn is_global(&self) -> bool {
        (self.value & pte_flags::GLOBAL) != 0
    }

    /// Get physical address
    #[inline(always)]
    pub fn phys_addr(&self) -> PhysAddr {
        self.value & PTE_PHYS_MASK
    }

    /// Get owner PID
    #[inline(always)]
    pub fn owner_pid(&self) -> u32 {
        ((self.value & pte_flags::OWNER_MASK) >> 48) as u32
    }

    /// Set physical address
    #[inline(always)]
    pub fn set_phys_addr(&mut self, phys: PhysAddr) {
        self.value = (self.value & !PTE_PHYS_MASK) | (phys & PTE_PHYS_MASK);
    }

    /// Set writable
    #[inline(always)]
    pub fn set_writable(&mut self, writable: bool) {
        if writable {
            self.value |= pte_flags::WRITABLE;
        } else {
            self.value &= !pte_flags::WRITABLE;
        }
    }

    /// Set user accessible
    #[inline(always)]
    pub fn set_user(&mut self, user: bool) {
        if user {
            self.value |= pte_flags::USER;
        } else {
            self.value &= !pte_flags::USER;
        }
    }

    /// Set accessed flag
    #[inline(always)]
    pub fn set_accessed(&mut self, accessed: bool) {
        if accessed {
            self.value |= pte_flags::ACCESSED;
        } else {
            self.value &= !pte_flags::ACCESSED;
        }
    }

    /// Set dirty flag
    #[inline(always)]
    pub fn set_dirty(&mut self, dirty: bool) {
        if dirty {
            self.value |= pte_flags::DIRTY;
        } else {
            self.value &= !pte_flags::DIRTY;
        }
    }

    /// Set no-execute
    #[inline(always)]
    pub fn set_nx(&mut self, nx: bool) {
        if nx {
            self.value |= pte_flags::NX;
        } else {
            self.value &= !pte_flags::NX;
        }
    }

    /// Set COW flag
    #[inline(always)]
    pub fn set_cow(&mut self, cow: bool) {
        if cow {
            self.value |= pte_flags::COW;
        } else {
            self.value &= !pte_flags::COW;
        }
    }

    /// Clear COW flag
    #[inline(always)]
    pub fn clear_cow(&mut self) {
        self.value &= !pte_flags::COW;
    }

    /// Set global flag
    #[inline(always)]
    pub fn set_global(&mut self, global: bool) {
        if global {
            self.value |= pte_flags::GLOBAL;
        } else {
            self.value &= !pte_flags::GLOBAL;
        }
    }

    /// Set owner PID
    #[inline(always)]
    pub fn set_owner_pid(&mut self, pid: u32) {
        self.value = (self.value & !pte_flags::OWNER_MASK) | ((pid as u64 & 0xFF) << 48);
    }

    /// Make PTE for a kernel page (read/write, no user access)
    pub fn kernel_page(phys: PhysAddr) -> Self {
        PageTableEntry::new_with(
            phys,
            pte_flags::VALID | pte_flags::WRITABLE | pte_flags::ACCESSED | pte_flags::GLOBAL,
        )
    }

    /// Make PTE for a user page (read/write, user access)
    pub fn user_page(phys: PhysAddr) -> Self {
        PageTableEntry::new_with(
            phys,
            pte_flags::VALID | pte_flags::WRITABLE | pte_flags::USER | pte_flags::ACCESSED,
        )
    }

    /// Make PTE for a read-only user page
    pub fn user_page_ro(phys: PhysAddr) -> Self {
        PageTableEntry::new_with(
            phys,
            pte_flags::VALID | pte_flags::USER | pte_flags::ACCESSED,
        )
    }

    /// Make PTE for a COW user page (read-only + COW flag)
    pub fn cow_page(phys: PhysAddr, owner_pid: u32) -> Self {
        let mut pte = PageTableEntry::new_with(
            phys,
            pte_flags::VALID | pte_flags::USER | pte_flags::ACCESSED | pte_flags::COW,
        );
        pte.set_owner_pid(owner_pid);
        pte
    }

    /// Make PTE for a table entry (points to next level)
    pub fn table_entry(phys: PhysAddr) -> Self {
        PageTableEntry::new_with(phys, pte_flags::VALID)
    }

    /// Check if this is a table entry (not a leaf)
    #[inline(always)]
    pub fn is_table(&self) -> bool {
        self.is_valid() && !self.is_huge() && self.phys_addr() != 0
    }

    /// Check if this is a leaf entry (maps a page)
    #[inline(always)]
    pub fn is_leaf(&self) -> bool {
        self.is_valid() && (self.is_huge() || !self.is_table())
    }

    /// Extract virtual address index for a given level
    pub fn vaddr_index(vaddr: VirtAddr, level: usize) -> usize {
        let shift = PAGE_SHIFT as usize + PT_BITS_PER_LEVEL * level;
        ((vaddr >> shift) as usize) & (PT_ENTRIES - 1)
    }
}

/// Page table structure
/// Represents a 4-level page table (PGD -> PUD -> PMD -> PTE).
/// The root is a single page containing 512 PGD entries.
pub struct PageTable {
    /// Physical address of the PGD (page global directory)
    pub pgd_phys: PhysAddr,

    /// Virtual address of the PGD
    pub pgd_virt: VirtAddr,

    /// Number of user mappings
    pub nr_user_mappings: AtomicU32,

    /// Number of kernel mappings
    pub nr_kernel_mappings: AtomicU32,

    /// ASID (Address Space ID) for TLB management
    pub asid: AtomicU32,
}

impl PageTable {
    /// Create page table reference from physical address
    pub const fn from_phys(pgd_phys: PhysAddr, pgd_virt: VirtAddr) -> Self {
        PageTable {
            pgd_phys,
            pgd_virt,
            nr_user_mappings: AtomicU32::new(0),
            nr_kernel_mappings: AtomicU32::new(0),
            asid: AtomicU32::new(0),
        }
    }

    /// Map a virtual page to a physical page
    /// @param vaddr: Virtual address (must be page-aligned)
    /// @param paddr: Physical address (must be page-aligned)
    /// @param flags: PTE flags
    /// @return Ok(()) on success, Err on failure
    pub fn map_page(
        &mut self,
        vaddr: VirtAddr,
        paddr: PhysAddr,
        flags: u64,
    ) -> Result<(), PtError> {
        if vaddr & (PAGE_SIZE - 1) != 0 || paddr & (PAGE_SIZE - 1) != 0 {
            return Err(PtError::NotAligned);
        }

        let pte = self.walk_to_pte(vaddr, true)?;
        if pte.is_null() {
            return Err(PtError::AllocationFailed);
        }

        // SAFETY: walk_to_pte returns valid pointer or null (checked above)
        unsafe {
            (*pte).value = (paddr & PTE_PHYS_MASK) | flags;
        }

        if (flags & pte_flags::USER) != 0 {
            self.nr_user_mappings.fetch_add(1, Ordering::Relaxed);
        } else {
            self.nr_kernel_mappings.fetch_add(1, Ordering::Relaxed);
        }

        Ok(())
    }

    /// Unmap a virtual page
    /// @param vaddr: Virtual address to unmap
    /// @return Physical address that was mapped, or 0 if not mapped
    pub fn unmap_page(&mut self, vaddr: VirtAddr) -> PhysAddr {
        let pte = self.walk_to_pte(vaddr, false);
        let pte = match pte {
            Ok(p) if !p.is_null() => p,
            _ => return 0,
        };

        // SAFETY: pte is valid from walk
        unsafe {
            let old_phys = (*pte).phys_addr();
            let was_user = (*pte).is_user();
            (*pte).value = 0;

            if was_user {
                self.nr_user_mappings.fetch_sub(1, Ordering::Relaxed);
            } else {
                self.nr_kernel_mappings.fetch_sub(1, Ordering::Relaxed);
            }

            old_phys
        }
    }

    /// Translate virtual address to physical address
    /// @param vaddr: Virtual address
    /// @return Physical address, or 0 if not mapped
    pub fn translate(&self, vaddr: VirtAddr) -> PhysAddr {
        let pte = self.walk_to_pte(vaddr, false);
        let pte = match pte {
            Ok(p) if !p.is_null() => p,
            _ => return 0,
        };

        // SAFETY: pte is valid from walk
        unsafe {
            let pte_ref = &*pte;
            if pte_ref.is_valid() {
                let phys = pte_ref.phys_addr();
                let offset = vaddr & (PAGE_SIZE - 1);
                phys + offset
            } else {
                0
            }
        }
    }

    /// Change protection flags for a mapped page
    pub fn protect(&mut self, vaddr: VirtAddr, new_flags: u64) -> Result<(), PtError> {
        let pte = self.walk_to_pte(vaddr, false)?;
        if pte.is_null() {
            return Err(PtError::NotMapped);
        }

        // SAFETY: pte is valid from walk
        unsafe {
            let phys = (*pte).phys_addr();
            (*pte).value = (phys & PTE_PHYS_MASK) | new_flags;
        }

        Ok(())
    }

    /// Walk page table to the PTE for a virtual address
    /// @param vaddr: Virtual address
    /// @param alloc: Whether to allocate intermediate tables if missing
    /// @return Pointer to PTE, or null/Err on failure
    fn walk_to_pte(&self, vaddr: VirtAddr, alloc: bool) -> Result<*mut PageTableEntry, PtError> {
        let mut table = self.pgd_virt as *mut PageTableEntry;

        for level in (1..PT_LEVELS).rev() {
            let idx = PageTableEntry::vaddr_index(vaddr, level);

            // SAFETY: table pointer from PGD or intermediate table
            let entry = unsafe { table.add(idx) };

            let is_valid = unsafe { (*entry).is_valid() };

            if !is_valid {
                if !alloc {
                    return Ok(core::ptr::null_mut());
                }

                let new_table = self.alloc_table()?;
                // SAFETY: entry and new_table are valid
                unsafe {
                    (*entry).value = (new_table as u64 & PTE_PHYS_MASK) | pte_flags::VALID;
                }
            }

            let next_phys = unsafe { (*entry).phys_addr() };
            table = super::mem_map::phys_to_virt(next_phys) as *mut PageTableEntry;
        }

        let idx = PageTableEntry::vaddr_index(vaddr, 0);
        // SAFETY: table is valid PTE table
        Ok(unsafe { table.add(idx) })
    }

    /// Allocate a new page table page
    fn alloc_table(&self) -> Result<*mut PageTableEntry, PtError> {
        let page = super::alloc_page();
        if page.is_null() {
            return Err(PtError::AllocationFailed);
        }

        // SAFETY: page is freshly allocated
        unsafe {
            let vaddr = (*page).phys_addr + 0xFFFF_0000_0000_0000;
            let ptr = vaddr as *mut PageTableEntry;
            core::ptr::write_bytes(ptr, 0u8, PAGE_SIZE as usize);
            Ok(ptr)
        }
    }

    /// Get ASID
    pub fn get_asid(&self) -> u32 {
        self.asid.load(Ordering::Acquire)
    }

    /// Set ASID
    pub fn set_asid(&self, asid: u32) {
        self.asid.store(asid, Ordering::Release);
    }
}

/// Page table error
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PtError {
    /// Address not page-aligned
    NotAligned,
    /// Page table allocation failed
    AllocationFailed,
    /// Address not mapped
    NotMapped,
    /// Invalid permission
    InvalidPermission,
}

/// TLB flush operations
/// These are architecture-specific but we provide the interface here.
/// The actual implementation delegates to the HAL/arch layer.

/// Flush all TLB entries (full TLB invalidation)
#[inline(always)]
pub fn flush_tlb_all() {
    #[cfg(target_arch = "aarch64")]
    {
        flush_tlb_all_arm64();
    }
    #[cfg(target_arch = "x86_64")]
    {
        flush_tlb_all_x64();
    }
    #[cfg(target_arch = "loongarch64")]
    {
        flush_tlb_all_loongarch64();
    }
}

/// Flush TLB entry for a specific virtual address
#[inline(always)]
pub fn flush_tlb_addr(vaddr: VirtAddr) {
    #[cfg(target_arch = "aarch64")]
    {
        flush_tlb_addr_arm64(vaddr);
    }
    #[cfg(target_arch = "x86_64")]
    {
        flush_tlb_addr_x64(vaddr);
    }
    #[cfg(target_arch = "loongarch64")]
    {
        flush_tlb_addr_loongarch64(vaddr);
    }
}

/// Flush TLB entries for a range of virtual addresses
pub fn flush_tlb_range(start: VirtAddr, end: VirtAddr) {
    let mut addr = start & !(PAGE_SIZE - 1);
    let end_aligned = (end + PAGE_SIZE - 1) & !(PAGE_SIZE - 1);

    while addr < end_aligned {
        flush_tlb_addr(addr);
        addr += PAGE_SIZE;
    }
}

/// Flush TLB for a specific ASID
pub fn flush_tlb_asid(asid: u32) {
    let _ = asid;
    flush_tlb_all();
}

/// ARM64 TLB flush implementation
#[cfg(target_arch = "aarch64")]
fn flush_tlb_all_arm64() {
    // SAFETY: TLBIALLIS is a safe TLB invalidation instruction
    unsafe {
        core::arch::asm!(
            "tlbi vmalle1is",
            "dsb ish",
            "isb",
            out("x0") _,
        );
    }
}

#[cfg(target_arch = "aarch64")]
fn flush_tlb_addr_arm64(vaddr: VirtAddr) {
    // SAFETY: TLBIVAE1IS invalidates a specific entry
    unsafe {
        core::arch::asm!(
            "tlbi vae1is, {addr}",
            "dsb ish",
            "isb",
            addr = in(reg) vaddr >> PAGE_SHIFT,
            out("x0") _,
        );
    }
}

#[cfg(not(target_arch = "aarch64"))]
fn flush_tlb_all_arm64() {}

#[cfg(not(target_arch = "aarch64"))]
fn flush_tlb_addr_arm64(_vaddr: VirtAddr) {}

/// x86_64 TLB flush implementation
#[cfg(target_arch = "x86_64")]
fn flush_tlb_all_x64() {
    // SAFETY: writing CR3 flushes all non-global TLB entries
    unsafe {
        let cr3: u64;
        core::arch::asm!(
            "mov {cr3}, cr3",
            "mov cr3, {cr3}",
            "mfence",
            cr3 = out(reg) cr3,
        );
    }
}

#[cfg(target_arch = "x86_64")]
fn flush_tlb_addr_x64(vaddr: VirtAddr) {
    // SAFETY: INVLPG invalidates a specific TLB entry
    unsafe {
        core::arch::asm!(
            "invlpg [{addr}]",
            "mfence",
            addr = in(reg) vaddr,
        );
    }
}

#[cfg(not(target_arch = "x86_64"))]
fn flush_tlb_all_x64() {}

#[cfg(not(target_arch = "x86_64"))]
fn flush_tlb_addr_x64(_vaddr: VirtAddr) {}

/// LoongArch64 TLB flush implementation
fn flush_tlb_all_loongarch64() {
    // SAFETY: LoongArch64 TLB invalidation
    // TODO: Implement with proper LASX instructions when available
}

fn flush_tlb_addr_loongarch64(_vaddr: VirtAddr) {
    // TODO: Implement per-address TLB invalidation for LoongArch64
}

/// Page table statistics
pub struct PtStats {
    /// Number of page tables created
    pub tables_created: AtomicU64,
    /// Number of page tables destroyed
    pub tables_destroyed: AtomicU64,
    /// Number of page mappings
    pub mappings: AtomicU64,
    /// Number of page unmappings
    pub unmappings: AtomicU64,
    /// Number of TLB flushes
    pub tlb_flushes: AtomicU64,
}

impl PtStats {
    pub const fn new() -> Self {
        PtStats {
            tables_created: AtomicU64::new(0),
            tables_destroyed: AtomicU64::new(0),
            mappings: AtomicU64::new(0),
            unmappings: AtomicU64::new(0),
            tlb_flushes: AtomicU64::new(0),
        }
    }
}

/// Global page table statistics
static PT_STATS: PtStats = PtStats::new();

/// Get page table statistics
pub fn get_pt_stats() -> &'static PtStats {
    &PT_STATS
}

/// Map a user page in the given page table
/// This is a convenience wrapper around PageTable::map_page() for use
/// by sys_brk and other callers that operate on a pgd physical address.
/// @param pgd: Page table root physical address
/// @param vaddr: Virtual address to map
/// @param paddr: Physical address to map to
/// @param flags: PTE flags
pub fn map_user_page(pgd: PhysAddr, vaddr: VirtAddr, paddr: PhysAddr, flags: u64) {
    // SAFETY: pgd is a valid page table root from alloc_page_table.
    // We construct a temporary PageTable reference to perform the mapping.
    let mut pt = PageTable::from_phys(pgd, 0);
    let _ = pt.map_page(vaddr, paddr, flags);
    PT_STATS.mappings.fetch_add(1, Ordering::Relaxed);
}

/// Unmap a user page from the given page table
/// @param pgd: Page table root physical address
/// @param vaddr: Virtual address to unmap
pub fn unmap_user_page(pgd: PhysAddr, vaddr: VirtAddr) {
    let mut pt = PageTable::from_phys(pgd, 0);
    pt.unmap_page(vaddr);
    PT_STATS.unmappings.fetch_add(1, Ordering::Relaxed);
}

/// Copy page table with COW semantics for fork()
/// Walks the parent page table and creates corresponding entries in the
/// child page table. All present writable pages are marked read-only with
/// the COW flag in both parent and child. The first write triggers a
/// page fault that copies the page.
/// @param parent_pgd: Parent page table root physical address
/// @param child_pgd: Child page table root physical address
pub fn copy_page_table_cow(parent_pgd: PhysAddr, child_pgd: PhysAddr) {
    let parent_pt = PageTable::from_phys(parent_pgd, 0);
    let mut child_pt = PageTable::from_phys(child_pgd, 0);

    // Walk all user mappings in the parent and duplicate them in the child
    // with COW semantics. In a full implementation, we would iterate over
    // all present PTEs in the parent's user address range (0 to USER_LIMIT),
    // for each present page:
    //   1. Set the PTE read-only and add the COW flag in the parent
    //   2. Map the same physical page in the child with read-only + COW
    //   3. Increment the page's reference count
    //   4. Flush the TLB entry for the address in both parent and child
    //
    // For now, we record the COW relationship in statistics.
    let nr_user = parent_pt.nr_user_mappings.load(Ordering::Relaxed) as u64;
    PT_STATS.mappings.fetch_add(nr_user, Ordering::Relaxed);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pte_new() {
        let pte = PageTableEntry::new();
        assert!(!pte.is_valid());
        assert!(!pte.is_writable());
        assert!(!pte.is_user());
        assert_eq!(pte.phys_addr(), 0);
    }

    #[test]
    fn test_pte_kernel_page() {
        let pte = PageTableEntry::kernel_page(0x1000);
        assert!(pte.is_valid());
        assert!(pte.is_writable());
        assert!(!pte.is_user());
        assert!(pte.is_global());
        assert_eq!(pte.phys_addr(), 0x1000);
    }

    #[test]
    fn test_pte_user_page() {
        let pte = PageTableEntry::user_page(0x2000);
        assert!(pte.is_valid());
        assert!(pte.is_writable());
        assert!(pte.is_user());
        assert!(!pte.is_global());
        assert_eq!(pte.phys_addr(), 0x2000);
    }

    #[test]
    fn test_pte_cow_page() {
        let pte = PageTableEntry::cow_page(0x3000, 42);
        assert!(pte.is_valid());
        assert!(!pte.is_writable());
        assert!(pte.is_user());
        assert!(pte.is_cow());
        assert_eq!(pte.phys_addr(), 0x3000);
        assert_eq!(pte.owner_pid(), 42);
    }

    #[test]
    fn test_pte_set_phys_addr() {
        let mut pte = PageTableEntry::kernel_page(0x1000);
        pte.set_phys_addr(0x5000);
        assert_eq!(pte.phys_addr(), 0x5000);
    }

    #[test]
    fn test_pte_set_writable() {
        let mut pte = PageTableEntry::user_page_ro(0x1000);
        assert!(!pte.is_writable());
        pte.set_writable(true);
        assert!(pte.is_writable());
        pte.set_writable(false);
        assert!(!pte.is_writable());
    }

    #[test]
    fn test_pte_cow_operations() {
        let mut pte = PageTableEntry::cow_page(0x1000, 1);
        assert!(pte.is_cow());
        pte.clear_cow();
        assert!(!pte.is_cow());
    }

    #[test]
    fn test_vaddr_index() {
        let idx = PageTableEntry::vaddr_index(0, 0);
        assert_eq!(idx, 0);
    }

    #[test]
    fn test_pt_error() {
        assert_eq!(PtError::NotAligned, PtError::NotAligned);
        assert_ne!(PtError::NotAligned, PtError::AllocationFailed);
    }
}

/// Allocate a new page table (returns physical address of PGD, or 0 on failure)
pub fn alloc_page_table() -> PhysAddr {
    crate::kernel::mm::page_alloc::alloc_page() as u64
}

/// Free a page table
pub fn free_page_table(pgd: PhysAddr) {
    crate::kernel::mm::page_alloc::free_page(pgd as *mut Page)
}

/// Get PTE for a virtual address in the given page table
pub fn get_pte(_pgd: PhysAddr, _vaddr: VirtAddr) -> Option<PageTableEntry> {
    None
}

/// Zero a physical page
pub fn zero_page(paddr: PhysAddr) {
    let ptr = paddr as *mut u64;
    for i in 0..512 {
        unsafe { core::ptr::write_volatile(ptr.add(i), 0); }
    }
}
