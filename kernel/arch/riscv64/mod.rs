/*
 * Nuva OS - Kernel - RISC-V 64 Architecture
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

//! RISC-V 64-bit (RV64G) architecture support for Nuva OS.
//! Implements the ArchOps trait hierarchy for RISC-V S-mode operation.

// Re-export print macros from crate root
pub use crate::{pr_emerg, pr_alert, pr_crit, pr_err, pr_warn, pr_notice, pr_info, pr_debug};
use core::arch::asm;

pub mod mmu;
pub mod plic;
pub mod timer;
pub mod context;
pub mod sbi;
pub mod arch_impl;
pub mod boot;
pub mod trap;
pub mod plugin;
pub mod mm;

// Re-export architecture implementation
pub use arch_impl::*;

// ============================================================================
// CSR Register Access Macros
// ============================================================================

/// Macro to read a CSR register.
#[macro_export]
macro_rules! read_csr {
    ($csr:tt) => {
        {
            let val: u64;
            // SAFETY: CSR read is a benign hardware register read.
            unsafe {
                asm!(
                    concat!("csrr {}, ", $csr),
                    out(reg) val,
                );
            }
            val
        }
    };
}

/// Macro to write a CSR register.
#[macro_export]
macro_rules! write_csr {
    ($csr:tt, $val:expr) => {
        // SAFETY: CSR write is required for hardware configuration.
        unsafe {
            asm!(
                concat!("csrw ", $csr, ", {}"),
                in(reg) $val as u64,
            );
        }
    };
}

/// Macro to swap a CSR register (write new, return old).
#[macro_export]
macro_rules! swap_csr {
    ($csr:tt, $val:expr) => {
        {
            let old: u64;
            // SAFETY: CSR swap is required for atomic hardware register update.
            unsafe {
                asm!(
                    concat!("csrrw {}, ", $csr, ", {}"),
                    out(reg) old,
                    in(reg) $val as u64,
                );
            }
            old
        }
    };
}

// ============================================================================
// RISC-V Constants
// ============================================================================

/// Page size (4 KiB).
pub const PAGE_SIZE: u64 = 4096;

/// Page table entries per page table (512 for RV64).
pub const PTE_PER_PT: u64 = 512;

/// Page shift (log2(4096) = 12).
pub const PAGE_SHIFT: u64 = 12;

/// Physical address width (56 bits on RV64).
pub const PADDR_WIDTH: u64 = 56;

/// Number of general-purpose registers (x0-x31).
pub const NUM_GPRS: usize = 32;

/// Number of floating-point registers (f0-f31).
pub const NUM_FPRS: usize = 32;

// ============================================================================
// RISC-V CSR Addresses (for reference; use read_csr!/write_csr! macros)
// ============================================================================

/// SSTATUS - Supervisor Status Register.
pub const CSR_SSTATUS: u64 = 0x100;
/// SEPC - Supervisor Exception Program Counter.
pub const CSR_SEPC: u64 = 0x141;
/// SCAUSE - Supervisor Cause Register.
pub const CSR_SCAUSE: u64 = 0x142;
/// STVAL - Supervisor Trap Value Register.
pub const CSR_STVAL: u64 = 0x143;
/// SATP - Supervisor Address Translation and Protection.
pub const CSR_SATP: u64 = 0x180;
/// STVEC - Supervisor Trap Vector Base Address.
pub const CSR_STVEC: u64 = 0x105;
/// SIE - Supervisor Interrupt Enable.
pub const CSR_SIE: u64 = 0x104;
/// SIP - Supervisor Interrupt Pending.
pub const CSR_SIP: u64 = 0x144;
/// SSCRATCH - Supervisor Scratch Register.
pub const CSR_SSCRATCH: u64 = 0x140;

// ============================================================================
// Barrier Instructions
// ============================================================================

/// FENCE - Memory ordering barrier.
#[inline(always)]
pub fn fence() {
    // SAFETY: FENCE is a standard RISC-V memory ordering instruction.
    unsafe { asm!("fence"); }
}

/// FENCE.I - Instruction memory ordering barrier.
#[inline(always)]
pub fn fence_i() {
    // SAFETY: FENCE.I ensures instruction fetch ordering.
    unsafe { asm!("fence.i"); }
}

/// SFENCE.VMA - TLB flush for a single address.
#[inline(always)]
pub fn sfence_vma(vaddr: u64, asid: u64) {
    // SAFETY: sfence.vma is the standard TLB maintenance instruction.
    unsafe {
        asm!(
            "sfence.vma {0}, {1}",
            in(reg) vaddr,
            in(reg) asid,
        );
    }
}

/// SFENCE.VMA ALL - Flush entire TLB.
#[inline(always)]
pub fn sfence_vma_all() {
    // SAFETY: sfence.vma with zero args flushes all TLB entries.
    unsafe { asm!("sfence.vma zero, zero"); }
}

// ============================================================================
// Interrupt Control
// ============================================================================

/// Enable S-mode interrupts (sets sstatus.SIE).
#[inline(always)]
pub fn enable_irq() {
    // SAFETY: csrs sets the SIE bit in sstatus.
    unsafe { asm!("csrs sstatus, 2"); }
}

/// Disable S-mode interrupts (clears sstatus.SIE).
#[inline(always)]
pub fn disable_irq() {
    // SAFETY: csrc clears the SIE bit in sstatus.
    unsafe { asm!("csrc sstatus, 2"); }
}

/// Wait For Interrupt (low-power idle).
#[inline(always)]
pub fn wfi() {
    // SAFETY: WFI is a standard RISC-V instruction.
    unsafe { asm!("wfi"); }
}

/// No Operation.
#[inline(always)]
pub fn nop() {
    // SAFETY: NOP is a benign instruction.
    unsafe { asm!("nop"); }
}

/// Get the current hart ID.
#[inline(always)]
pub fn hart_id() -> u64 {
    let hartid: u64;
    // SAFETY: mhartid is a read-only CSR.
    unsafe { asm!("csrr {}, mhartid", out(reg) hartid); }
    hartid
}
