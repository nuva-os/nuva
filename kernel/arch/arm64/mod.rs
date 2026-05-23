/*
 * Nuva OS - Kernel - ARM64 Architecture
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

// Re-export print macros from crate root
pub use crate::{pr_emerg, pr_alert, pr_crit, pr_err, pr_warn, pr_notice, pr_info, pr_debug};
use core::arch::asm;

pub mod mmu;
pub mod gic;
pub mod timer;
pub mod context;
pub mod arch_impl;
pub mod boot;
pub mod trap;

// Re-export architecture implementation
pub use arch_impl::*;

// ARM64 System Register Operations

/// Macro to read a system register
#[macro_export]
macro_rules! read_sysreg {
    ($reg:tt) => {
        {
            let val: u64;
            // SAFETY: inline assembly required for hardware instruction
            unsafe {
                asm!(
                    concat!("mrs {}, ", $reg),
                    out(reg) val,
                );
            }
            val
        }
    };
}

/// Macro to write a system register
#[macro_export]
macro_rules! write_sysreg {
    ($reg:tt, $val:expr) => {
        // SAFETY: inline assembly required for hardware instruction
        unsafe {
            asm!(
                concat!("msr ", $reg, ", {}"),
                in(reg) $val as u64,
            );
        }
    };
}

/// Exception Level enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExceptionLevel {
    /// EL0: User mode
    EL0 = 0,
    /// EL1: Kernel mode
    EL1 = 1,
    /// EL2: Hypervisor
    EL2 = 2,
    /// EL3: Secure Monitor
    EL3 = 3,
}

/// Get current exception level
/// @return Current exception level
pub fn current_el() -> ExceptionLevel {
    let el_raw: u64;
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!(
            "mrs {}, CurrentEL",
            out(reg) el_raw,
        );
    }
    let el = el_raw >> 2;

    match el {
        0 => ExceptionLevel::EL0,
        1 => ExceptionLevel::EL1,
        2 => ExceptionLevel::EL2,
        3 => ExceptionLevel::EL3,
        _ => ExceptionLevel::EL0,
    }
}

/// Get CPU ID
/// @return CPU ID from MPIDR register
pub fn cpu_id() -> u64 {
    let mpidr: u64;
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!(
            "mrs {}, mpidr_el1",
            out(reg) mpidr,
        );
    }
    mpidr & 0xFF
}

/// Data Memory Barrier
#[inline(always)]
pub fn dmb() {
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!("dmb sy");
    }
}

/// Data Synchronization Barrier
#[inline(always)]
pub fn dsb() {
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!("dsb sy");
    }
}

/// Instruction Synchronization Barrier
#[inline(always)]
pub fn isb() {
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!("isb");
    }
}

/// No Operation
#[inline(always)]
pub fn nop() {
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!("nop");
    }
}

/// Wait For Interrupt
#[inline(always)]
pub fn wfi() {
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!("wfi");
    }
}

/// Wait For Event
#[inline(always)]
pub fn wfe() {
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!("wfe");
    }
}

/// Send Event
#[inline(always)]
pub fn sev() {
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!("sev");
    }
}

/// Send Event Local
#[inline(always)]
pub fn sevl() {
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!("sevl");
    }
}

/// Enable IRQ interrupts
#[inline(always)]
pub fn enable_irq() {
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!(
            "msr daifclr, #2",
        );
    }
}

/// Disable IRQ interrupts
#[inline(always)]
pub fn disable_irq() {
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!(
            "msr daifset, #2",
        );
    }
}

/// Enable FIQ interrupts
#[inline(always)]
pub fn enable_fiq() {
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!(
            "msr daifclr, #1",
        );
    }
}

/// Disable FIQ interrupts
#[inline(always)]
pub fn disable_fiq() {
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!(
            "msr daifset, #1",
        );
    }
}

/// Enable all interrupts
#[inline(always)]
pub fn enable_all_irq() {
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!(
            "msr daifclr, #15",
        );
    }
}

/// Disable all interrupts
#[inline(always)]
pub fn disable_all_irq() {
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!(
            "msr daifset, #15",
        );
    }
}

/// Get DAIF register value
/// @return Current DAIF register value
pub fn get_daif() -> u64 {
    let daif: u64;
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!(
            "mrs {}, daif",
            out(reg) daif,
        );
    }
    daif
}

/// Set DAIF register value
/// @param daif: New DAIF value
pub fn set_daif(daif: u64) {
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!(
            "msr daif, {}",
            in(reg) daif,
        );
    }
}

/// IRQ save guard
/// Saves interrupt state and disables IRQs.
/// Restores state when dropped.
pub struct IrqSave {
    daif: u64,
}

impl IrqSave {
    /// Save current interrupt state and disable IRQs
    pub fn save_disable() -> Self {
        let daif = get_daif();
        disable_irq();
        IrqSave { daif }
    }
}

impl Drop for IrqSave {
    fn drop(&mut self) {
        set_daif(self.daif);
    }
}

/// Flush all TLB entries
#[inline(always)]
pub fn tlb_flush_all() {
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!(
            "dsb ishst",
            "tlbi vmalle1is",
            "dsb ish",
            "isb",
        );
    }
}

/// Flush TLB entry for a specific address
/// @param addr: Virtual address to flush
#[inline(always)]
pub fn tlb_flush_addr(addr: u64) {
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!(
            "dsb ishst",
            "tlbi vae1is, {}",
            "dsb ish",
            "isb",
            in(reg) addr >> 12,
        );
    }
}

/// Flush all instruction cache
#[inline(always)]
pub fn icache_flush_all() {
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!(
            "ic ialluis",
            "dsb ish",
            "isb",
        );
    }
}

/// Flush data cache for a range of addresses
/// @param addr: Start address
/// @param size: Size in bytes
#[inline(always)]
pub fn dcache_flush_addr(addr: u64, size: usize) {
    let cache_line_size = 64;  /* Assume 64-byte cache line size */
    let start = addr & !(cache_line_size - 1);
    let end = (addr + size as u64 + cache_line_size - 1) & !(cache_line_size - 1);

    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe {
        for addr in (start..end).step_by(cache_line_size as usize) {
            asm!(
                "dc civac, {}",
                in(reg) addr,
            );
        }
        asm!(
            "dsb ish",
        );
    }
}

/// Get TTBR0 (user page table base)
/// @return TTBR0_EL1 value
pub fn get_ttbr0() -> u64 {
    let ttbr0: u64;
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!(
            "mrs {}, ttbr0_el1",
            out(reg) ttbr0,
        );
    }
    ttbr0
}

/// Set TTBR0 (user page table base)
/// @param ttbr0: New TTBR0 value
pub fn set_ttbr0(ttbr0: u64) {
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!(
            "msr ttbr0_el1, {}",
            "isb",
            in(reg) ttbr0,
        );
    }
}

/// Get TTBR1 (kernel page table base)
/// @return TTBR1_EL1 value
pub fn get_ttbr1() -> u64 {
    let ttbr1: u64;
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!(
            "mrs {}, ttbr1_el1",
            out(reg) ttbr1,
        );
    }
    ttbr1
}

/// Set TTBR1 (kernel page table base)
/// @param ttbr1: New TTBR1 value
pub fn set_ttbr1(ttbr1: u64) {
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!(
            "msr ttbr1_el1, {}",
            "isb",
            in(reg) ttbr1,
        );
    }
}

/// Get TCR (Translation Control Register)
/// @return TCR_EL1 value
pub fn get_tcr() -> u64 {
    let tcr: u64;
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!(
            "mrs {}, tcr_el1",
            out(reg) tcr,
        );
    }
    tcr
}

/// Set TCR (Translation Control Register)
/// @param tcr: New TCR value
pub fn set_tcr(tcr: u64) {
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!(
            "msr tcr_el1, {}",
            "isb",
            in(reg) tcr,
        );
    }
}

/// Get MAIR (Memory Attribute Indirection Register)
/// @return MAIR_EL1 value
pub fn get_mair() -> u64 {
    let mair: u64;
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!(
            "mrs {}, mair_el1",
            out(reg) mair,
        );
    }
    mair
}

/// Set MAIR (Memory Attribute Indirection Register)
/// @param mair: New MAIR value
pub fn set_mair(mair: u64) {
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!(
            "msr mair_el1, {}",
            "isb",
            in(reg) mair,
        );
    }
}

/// Get SCTLR (System Control Register)
/// @return SCTLR_EL1 value
pub fn get_sctlr() -> u64 {
    let sctlr: u64;
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!(
            "mrs {}, sctlr_el1",
            out(reg) sctlr,
        );
    }
    sctlr
}
