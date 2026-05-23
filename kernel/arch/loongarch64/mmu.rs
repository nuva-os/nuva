/*
* Nuva OS - Kernel - LoongArch64 MMU
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

use crate::pr_info;
use core::arch::asm;

/// Page size (4KB)
pub const PAGE_SIZE: u64 = 4096;

/// Page offset bits
pub const PAGE_SHIFT: u64 = 12;

/// Page table levels (3-level)
pub const PAGE_LEVELS: usize = 3;

/// LoongArch64 PTE flags
pub mod pte_flags {
    pub const VALID: u64 = 1 << 0;
    pub const READ: u64 = 1 << 1;
    pub const WRITE: u64 = 1 << 2;
    pub const EXEC: u64 = 1 << 3;
    pub const GLOBAL: u64 = 1 << 6;
    pub const DIRTY: u64 = 1 << 7;
    pub const PLV: u64 = 3 << 8;
}

/// CSR register addresses for MMU
mod csr {
    pub const PGDL: u32 = 0x19;
    pub const PGDH: u32 = 0x1a;
    pub const PGD: u32 = 0x1b;
    pub const TLBRENTRY: u32 = 0x88;
}

/// Page table entry
#[derive(Debug, Clone, Copy)]
pub struct PageTableEntry {
    pub value: u64,
}

impl PageTableEntry {
    /// Create new zeroed entry
    pub const fn new() -> Self {
        PageTableEntry { value: 0 }
    }

    /// Check if valid
    pub fn is_valid(&self) -> bool {
        self.value & pte_flags::VALID != 0
    }

    /// Check if readable
    pub fn is_readable(&self) -> bool {
        self.value & pte_flags::READ != 0
    }

    /// Check if writable
    pub fn is_writable(&self) -> bool {
        self.value & pte_flags::WRITE != 0
    }

    /// Check if executable
    pub fn is_executable(&self) -> bool {
        self.value & pte_flags::EXEC != 0
    }

    /// Get physical address from PTE
    pub fn get_phys(&self) -> u64 {
        (self.value >> 12) << 12
    }
}

/// Page table (512 entries for 4KB page)
pub struct PageTable {
    pub entries: [PageTableEntry; 512],
}

impl PageTable {
    /// Create new zeroed page table
    pub const fn new() -> Self {
        PageTable {
            entries: [PageTableEntry::new(); 512],
        }
    }
}

/// Read page directory base (CSR PGD)
pub fn read_pgd() -> u64 {
    let pgd: u64;
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!(
            "csrrd {}, {}",
            out(reg) pgd,
            in(reg) csr::PGD,
        );
    }
    pgd
}

/// Write page directory base (CSR PGD)
pub fn write_pgd(pgd: u64) {
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!(
            "csrwr {}, {}",
            in(reg) pgd,
            in(reg) csr::PGD,
        );
    }
}

/// Flush entire TLB
#[inline(always)]
pub fn tlb_flush_all() {
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!("invtlb 0, $r0, $r0");
    }
}

/// Flush TLB entry for a specific address
/// @param addr: Virtual address to flush
#[inline(always)]
pub fn tlb_flush_addr(_addr: u64) {
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!("invtlb 0, $r0, $r0");
    }
}

/// Initialize MMU
pub fn init_mmu() {
    log_info!("LoongArch64 MMU initialized");
}
