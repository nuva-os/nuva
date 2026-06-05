/*
 * Nuva OS - HAL - Gpu - GART (Graphics Address Remapping Table)
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

/* GART Page Table Management - GPU Address Remapping Table with IOMMU isolation.
 *
 * The GART provides address translation from GPU virtual addresses to system
 * physical addresses, enabling zero-copy sharing of buffers between CPU and GPU.
 *
 * IOMMU double isolation: Each GPU context gets its own GART page table,
 * preventing a malicious or buggy GPU context from accessing another context's
 * memory. This is enforced by hardware IOMMU context switching.
 */

use core::ptr::{read_volatile, write_volatile};
use core::sync::atomic::{AtomicU32, AtomicU64, AtomicBool, Ordering};

use super::GpuError;

// ============================================================================
// GART Constants
// ============================================================================

/// GART page size (4 KB, same as system page size)
pub const GART_PAGE_SIZE: u64 = 4096;

/// GART page shift
pub const GART_PAGE_SHIFT: u32 = 12;

/// Maximum entries per GART table level
pub const GART_ENTRIES_PER_TABLE: u32 = 512;

/// GART PTE valid bit
pub const GART_PTE_VALID: u64 = 1 << 0;

/// GART PTE writable bit
pub const GART_PTE_WRITABLE: u64 = 1 << 1;

/// GART PTE readable bit
pub const GART_PTE_READABLE: u64 = 1 << 2;

/// GART PTE executable bit (shader code)
pub const GART_PTE_EXECUTABLE: u64 = 1 << 3;

/// GART PTE cache coherent bit
pub const GART_PTE_COHERENT: u64 = 1 << 4;

/// GART PTE IOMMU isolated bit (double isolation marker)
pub const GART_PTE_IOMMU_ISOLATED: u64 = 1 << 5;

/// GART PTE system bit (indicates system memory vs VRAM)
pub const GART_PTE_SYSTEM_MEM: u64 = 1 << 6;

/// Address mask for PTE (physical page frame number)
pub const GART_PTE_ADDR_MASK: u64 = 0x000F_FFFF_FFFF_F000;

// ============================================================================
// GART Page Table Entry
// ============================================================================

/// GART page table entry (64-bit format compatible with GPU hardware)
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct GartPte {
    /// Raw 64-bit PTE value
    pub raw: u64,
}

impl GartPte {
    /// Create a null/invalid PTE
    pub const fn null() -> Self {
        GartPte { raw: 0 }
    }

    /// Create a new PTE with given physical address and flags
    pub const fn new(phys_addr: u64, flags: u64) -> Self {
        GartPte {
            raw: (phys_addr & GART_PTE_ADDR_MASK) | (flags & !GART_PTE_ADDR_MASK) | GART_PTE_VALID,
        }
    }

    /// Check if PTE is valid
    pub fn is_valid(&self) -> bool {
        (self.raw & GART_PTE_VALID) != 0
    }

    /// Get physical address from PTE
    pub fn phys_addr(&self) -> u64 {
        self.raw & GART_PTE_ADDR_MASK
    }

    /// Get flags from PTE
    pub fn flags(&self) -> u64 {
        self.raw & !GART_PTE_ADDR_MASK
    }

    /// Check if PTE is writable
    pub fn is_writable(&self) -> bool {
        (self.raw & GART_PTE_WRITABLE) != 0
    }

    /// Check if PTE is readable
    pub fn is_readable(&self) -> bool {
        (self.raw & GART_PTE_READABLE) != 0
    }

    /// Check if PTE is IOMMU isolated
    pub fn is_iommu_isolated(&self) -> bool {
        (self.raw & GART_PTE_IOMMU_ISOLATED) != 0
    }

    /// Check if PTE points to system memory
    pub fn is_system_mem(&self) -> bool {
        (self.raw & GART_PTE_SYSTEM_MEM) != 0
    }
}

// ============================================================================
// GART Mapping Flags
// ============================================================================

/// GART mapping permission flags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GartMapFlags(u64);

impl GartMapFlags {
    /// Read-only mapping
    pub const READ_ONLY: Self = GartMapFlags(GART_PTE_READABLE);
    /// Read-write mapping
    pub const READ_WRITE: Self = GartMapFlags(GART_PTE_READABLE | GART_PTE_WRITABLE);
    /// Read-write-execute mapping (for shader code)
    pub const READ_WRITE_EXEC: Self = GartMapFlags(GART_PTE_READABLE | GART_PTE_WRITABLE | GART_PTE_EXECUTABLE);
    /// Cache coherent mapping
    pub const COHERENT: Self = GartMapFlags(GART_PTE_COHERENT);
    /// IOMMU isolated mapping
    pub const IOMMU_ISOLATED: Self = GartMapFlags(GART_PTE_IOMMU_ISOLATED);
    /// System memory mapping (vs VRAM)
    pub const SYSTEM_MEM: Self = GartMapFlags(GART_PTE_SYSTEM_MEM);

    /// Create empty flags
    pub const fn empty() -> Self {
        GartMapFlags(0)
    }

    /// Combine flags
    pub const fn union(self, other: Self) -> Self {
        GartMapFlags(self.0 | other.0)
    }

    /// Get raw value
    pub const fn bits(&self) -> u64 {
        self.0
    }
}

// ============================================================================
// GART Table
// ============================================================================

/// Maximum number of GART page tables (one per GPU context for isolation)
pub const MAX_GART_TABLES: usize = 16;

/// Maximum mappings tracked per table (for software-side bookkeeping)
pub const MAX_GART_MAPPINGS: usize = 2048;

/// GART mapping record (software-side tracking)
#[derive(Debug, Clone, Copy)]
pub struct GartMapping {
    /// GPU virtual address (start)
    pub gpu_addr: u64,
    /// System physical address (start)
    pub sys_addr: u64,
    /// Size in bytes
    pub size: u64,
    /// Mapping flags
    pub flags: GartMapFlags,
    /// Whether this mapping is valid
    pub valid: bool,
    /// IOMMU context ID that owns this mapping
    pub iommu_context: u32,
}

/// GART Table - GPU Address Remapping Table with IOMMU double isolation
///
/// Each GART table provides address translation for a single GPU context.
/// IOMMU double isolation ensures that GPU contexts cannot access each
/// other's memory, even if the GPU is compromised.
pub struct GartTable {
    /// Table ID
    pub table_id: u32,
    /// IOMMU context ID for double isolation
    pub iommu_context: u32,
    /// Base address of the page table in system memory (GPU-accessible)
    pt_base: u64,
    /// GPU virtual address range start
    va_start: u64,
    /// GPU virtual address range size
    va_size: u64,
    /// Number of page table levels (1=flat, 2=two-level, 3=three-level)
    pt_levels: u32,
    /// Software-side mapping records
    mappings: [GartMapping; MAX_GART_MAPPINGS],
    /// Number of active mappings
    mapping_count: AtomicU32,
    /// Total mapped bytes
    mapped_bytes: AtomicU64,
    /// Table is valid/active
    active: AtomicBool,
}

impl GartTable {
    /// Create a new GART table
    pub const fn new(table_id: u32, iommu_context: u32, pt_base: u64,
                     va_start: u64, va_size: u64, pt_levels: u32) -> Self {
        GartTable {
            table_id,
            iommu_context,
            pt_base,
            va_start,
            va_size,
            pt_levels,
            mappings: [GartMapping {
                gpu_addr: 0,
                sys_addr: 0,
                size: 0,
                flags: GartMapFlags::empty(),
                valid: false,
                iommu_context: 0,
            }; MAX_GART_MAPPINGS],
            mapping_count: AtomicU32::new(0),
            mapped_bytes: AtomicU64::new(0),
            active: AtomicBool::new(false),
        }
    }

    /// Activate the GART table (program hardware registers)
    pub fn activate(&self, gart_base_reg: u64, gart_size_reg: u64) -> Result<(), GpuError> {
        // SAFETY: writing GART base and size to GPU control registers
        unsafe {
            write_volatile(gart_base_reg as *mut u32, self.pt_base as u32);
            write_volatile(gart_size_reg as *mut u32,
                (self.va_size / GART_PAGE_SIZE) as u32);
        }
        self.active.store(true, Ordering::Release);
        log_info!("GART table {}: activated (ctx={}, pt_base=0x{:X}, va=0x{:X}-0x{:X})",
            self.table_id, self.iommu_context, self.pt_base,
            self.va_start, self.va_start + self.va_size);
        Ok(())
    }

    /// Deactivate the GART table
    pub fn deactivate(&self) {
        self.active.store(false, Ordering::Release);
        log_info!("GART table {}: deactivated", self.table_id);
    }

    /// Check if table is active
    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::Acquire)
    }

    /// Map a GPU virtual address range to a system physical address range
    pub fn map(&mut self, gpu_addr: u64, sys_addr: u64, size: u64,
               flags: GartMapFlags) -> Result<(), GpuError> {
        if !self.active.load(Ordering::Acquire) {
            return Err(GpuError::NotInitialized);
        }

        // Validate address range
        if gpu_addr < self.va_start || gpu_addr + size > self.va_start + self.va_size {
            log_warn!("GART table {}: map address out of range (gpu=0x{:X}, size=0x{:X})",
                self.table_id, gpu_addr, size);
            return Err(GpuError::InvalidArg);
        }

        // Check alignment
        if (gpu_addr & (GART_PAGE_SIZE - 1)) != 0 || (sys_addr & (GART_PAGE_SIZE - 1)) != 0 {
            return Err(GpuError::InvalidArg);
        }

        // Find a free mapping slot
        let mut slot_idx: Option<usize> = None;
        for (i, mapping) in self.mappings.iter().enumerate() {
            if !mapping.valid {
                slot_idx = Some(i);
                break;
            }
        }
        let idx = slot_idx.ok_or(GpuError::OutOfMemory)?;

        // Write PTEs into hardware page table
        let num_pages = (size + GART_PAGE_SIZE - 1) / GART_PAGE_SIZE;
        let pte_flags = flags.bits() | GART_PTE_IOMMU_ISOLATED | GART_PTE_VALID;

        for page in 0..num_pages {
            let pte = GartPte::new(sys_addr + page * GART_PAGE_SIZE, pte_flags);
            let pte_index = ((gpu_addr - self.va_start) / GART_PAGE_SIZE + page) as u64;

            // SAFETY: writing PTE into GPU page table memory
            unsafe {
                let pte_ptr = (self.pt_base + pte_index * 8) as *mut u64;
                write_volatile(pte_ptr, pte.raw);
            }
        }

        // Record mapping in software table
        self.mappings[idx] = GartMapping {
            gpu_addr,
            sys_addr,
            size,
            flags,
            valid: true,
            iommu_context: self.iommu_context,
        };
        self.mapping_count.fetch_add(1, Ordering::Release);
        self.mapped_bytes.fetch_add(size, Ordering::Release);

        log_debug!("GART table {}: mapped gpu=0x{:X} -> sys=0x{:X} ({} KB, flags=0x{:X})",
            self.table_id, gpu_addr, sys_addr, size / 1024, flags.bits());

        Ok(())
    }

    /// Unmap a GPU virtual address range
    pub fn unmap(&mut self, gpu_addr: u64, size: u64) -> Result<(), GpuError> {
        if !self.active.load(Ordering::Acquire) {
            return Err(GpuError::NotInitialized);
        }

        // Find the mapping record
        let mut found_idx: Option<usize> = None;
        for (i, mapping) in self.mappings.iter().enumerate() {
            if mapping.valid && mapping.gpu_addr == gpu_addr {
                found_idx = Some(i);
                break;
            }
        }
        let idx = found_idx.ok_or(GpuError::InvalidArg)?;

        let mapping = &self.mappings[idx];
        let actual_size = if size == 0 { mapping.size } else { size };

        // Clear PTEs in hardware page table
        let num_pages = (actual_size + GART_PAGE_SIZE - 1) / GART_PAGE_SIZE;
        for page in 0..num_pages {
            let pte_index = ((gpu_addr - self.va_start) / GART_PAGE_SIZE + page) as u64;

            // SAFETY: clearing PTE in GPU page table memory
            unsafe {
                let pte_ptr = (self.pt_base + pte_index * 8) as *mut u64;
                write_volatile(pte_ptr, 0); // Invalid PTE
            }
        }

        // Invalidate mapping record
        self.mapped_bytes.fetch_sub(mapping.size, Ordering::Release);
        self.mappings[idx].valid = false;
        self.mapping_count.fetch_sub(1, Ordering::Release);

        log_debug!("GART table {}: unmapped gpu=0x{:X} ({} KB)",
            self.table_id, gpu_addr, actual_size / 1024);

        Ok(())
    }

    /// Translate a GPU virtual address to a system physical address
    pub fn translate(&self, gpu_addr: u64) -> Result<u64, GpuError> {
        if !self.active.load(Ordering::Acquire) {
            return Err(GpuError::NotInitialized);
        }

        // Check address range
        if gpu_addr < self.va_start || gpu_addr >= self.va_start + self.va_size {
            return Err(GpuError::InvalidArg);
        }

        // Look up in software mapping records first (fast path)
        for mapping in &self.mappings {
            if mapping.valid && gpu_addr >= mapping.gpu_addr
                && gpu_addr < mapping.gpu_addr + mapping.size
            {
                let offset = gpu_addr - mapping.gpu_addr;
                return Ok(mapping.sys_addr + offset);
            }
        }

        // Slow path: walk hardware page table
        let pte_index = ((gpu_addr - self.va_start) / GART_PAGE_SIZE) as u64;

        // SAFETY: reading PTE from GPU page table memory
        let pte_raw = unsafe {
            read_volatile((self.pt_base + pte_index * 8) as *const u64)
        };

        let pte = GartPte { raw: pte_raw };
        if !pte.is_valid() {
            return Err(GpuError::InvalidArg);
        }

        // Check IOMMU isolation
        if !pte.is_iommu_isolated() {
            log_warn!("GART table {}: PTE at index {} not IOMMU isolated!",
                self.table_id, pte_index);
            // Still translate but warn - hardware IOMMU will enforce
        }

        let page_offset = gpu_addr & (GART_PAGE_SIZE - 1);
        Ok(pte.phys_addr() + page_offset)
    }

    /// Get the number of active mappings
    pub fn mapping_count(&self) -> u32 {
        self.mapping_count.load(Ordering::Acquire)
    }

    /// Get total mapped bytes
    pub fn mapped_bytes(&self) -> u64 {
        self.mapped_bytes.load(Ordering::Acquire)
    }

    /// Flush TLB for this GART table (invalidate GPU TLB entries)
    pub fn flush_tlb(&self, tlb_invalidate_reg: u64) {
        // SAFETY: writing to GPU TLB invalidation register
        unsafe {
            // Write table ID to trigger TLB invalidation
            write_volatile(tlb_invalidate_reg as *mut u32, self.table_id);
        }
        log_debug!("GART table {}: TLB flushed", self.table_id);
    }

    /// Reset the GART table (clear all mappings)
    pub fn reset(&mut self) {
        for mapping in &mut self.mappings {
            mapping.valid = false;
        }
        self.mapping_count.store(0, Ordering::Release);
        self.mapped_bytes.store(0, Ordering::Release);
        self.active.store(false, Ordering::Release);
        log_info!("GART table {}: reset", self.table_id);
    }
}

// ============================================================================
// GART Manager (multi-table with IOMMU isolation)
// ============================================================================

/// GART manager - manages multiple GART tables with IOMMU double isolation
pub struct GartManager {
    /// GART table slots
    tables: [Option<GartTable>; MAX_GART_TABLES],
    /// Number of active tables
    num_tables: AtomicU32,
    /// Next IOMMU context ID
    next_iommu_context: AtomicU32,
}

impl GartManager {
    /// Create a new GART manager
    pub const fn new() -> Self {
        GartManager {
            tables: [None; MAX_GART_TABLES],
            num_tables: AtomicU32::new(0),
            next_iommu_context: AtomicU32::new(1), // 0 = kernel context
        }
    }

    /// Create a new GART table for a GPU context (with IOMMU isolation)
    pub fn create_table(&mut self, pt_base: u64, va_start: u64,
                        va_size: u64, pt_levels: u32) -> Result<u32, GpuError> {
        let iommu_ctx = self.next_iommu_context.fetch_add(1, Ordering::AcqRel);

        for (i, slot) in self.tables.iter_mut().enumerate() {
            if slot.is_none() {
                let table = GartTable::new(
                    i as u32, iommu_ctx, pt_base, va_start, va_size, pt_levels
                );
                *slot = Some(table);
                self.num_tables.fetch_add(1, Ordering::Release);
                log_info!("GART: created table {} (IOMMU context={})", i, iommu_ctx);
                return Ok(i as u32);
            }
        }

        Err(GpuError::OutOfMemory)
    }

    /// Destroy a GART table
    pub fn destroy_table(&mut self, table_id: u32) -> Result<(), GpuError> {
        if (table_id as usize) >= MAX_GART_TABLES {
            return Err(GpuError::InvalidArg);
        }

        if let Some(ref mut table) = self.tables[table_id as usize] {
            table.reset();
        }
        self.tables[table_id as usize] = None;
        self.num_tables.fetch_sub(1, Ordering::Release);
        Ok(())
    }

    /// Get a GART table by ID
    pub fn get_table(&self, table_id: u32) -> Option<&GartTable> {
        if (table_id as usize) >= MAX_GART_TABLES {
            return None;
        }
        self.tables[table_id as usize].as_ref()
    }

    /// Get a mutable GART table by ID
    pub fn get_table_mut(&mut self, table_id: u32) -> Option<&mut GartTable> {
        if (table_id as usize) >= MAX_GART_TABLES {
            return None;
        }
        self.tables[table_id as usize].as_mut()
    }

    /// Get number of active tables
    pub fn num_tables(&self) -> u32 {
        self.num_tables.load(Ordering::Acquire)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gart_pte_null() {
        let pte = GartPte::null();
        assert!(!pte.is_valid());
        assert_eq!(pte.phys_addr(), 0);
    }

    #[test]
    fn test_gart_pte_new() {
        let pte = GartPte::new(0x1000_0000, Gart_PTE_READABLE | Gart_PTE_WRITABLE);
        assert!(pte.is_valid());
        assert!(pte.is_readable());
        assert!(pte.is_writable());
        assert_eq!(pte.phys_addr(), 0x1000_0000);
    }

    #[test]
    fn test_gart_map_flags() {
        let rw = GartMapFlags::READ_WRITE;
        let exec = GartMapFlags::READ_WRITE_EXEC;
        assert!(exec.bits() > rw.bits());
    }

    #[test]
    fn test_gart_table_creation() {
        let table = GartTable::new(0, 1, 0x2000_0000, 0x0000_0000, 0x4000_0000, 2);
        assert_eq!(table.table_id, 0);
        assert_eq!(table.iommu_context, 1);
        assert!(!table.is_active());
    }

    #[test]
    fn test_gart_manager() {
        let mut mgr = GartManager::new();
        assert_eq!(mgr.num_tables(), 0);

        let result = mgr.create_table(0x2000_0000, 0, 0x4000_0000, 2);
        assert!(result.is_ok());
        assert_eq!(mgr.num_tables(), 1);
    }
}
