/*
 * Nuva OS - HAL - ARM64 MMU
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */

// ARM64 MMU and page table management

/// Page size (4KB)
pub const PAGE_SIZE: u64 = 4096;

/// Page shift
pub const PAGE_SHIFT: u64 = 12;

/// MAIR_EL1 attributes: Normal Non-Cacheable (index 0), Normal Cacheable (index 1), Device nGnRnE (index 2)
const MAIR_EL1_VALUE: u64 = (0x00 << 0) | (0xFF << 8) | (0x00 << 16);

/// TCR_EL1: 4KB granule, VA 48-bit, ASID 16-bit, inner shareable
const TCR_EL1_VALUE: u64 = (0b00 << 30)   // TG1 = 4KB
    | (25 << 16)                            // T1SZ = 25 (VA=39-bit for TTBR1)
    | (0b01 << 12)                          // SH1 = Inner Shareable
    | (0b01 << 10)                          // ORGN1 = Write-Back Cacheable
    | (0b01 << 8)                           // IRGN1 = Write-Back Cacheable
    | (0b00 << 6)                           // TG0 = 4KB
    | (25 << 0)                             // T0SZ = 25 (VA=39-bit for TTBR0)
    | (0b01 << 22);                         // IPS = 4TB (36-bit PA)

/// Configure MAIR_EL1 (Memory Attribute Indirection Register)
fn configure_mair() {
    // SAFETY: Writing to MAIR_EL1 system register configures memory attributes.
    // This is a system register write with no memory safety implications
    // beyond the intended MMU configuration.
    unsafe {
        core::arch::asm!("msr MAIR_EL1, {}", in(reg) MAIR_EL1_VALUE);
    }
}

/// Configure TCR_EL1 (Translation Control Register)
fn configure_tcr() {
    // SAFETY: Writing to TCR_EL1 configures translation control parameters.
    // This is a system register write required for MMU setup.
    unsafe {
        core::arch::asm!("msr TCR_EL1, {}", in(reg) TCR_EL1_VALUE);
    }
}

/// Set TTBR0_EL1 (Translation Table Base Register 0)
fn set_ttbr0(ttbr0: u64) {
    // SAFETY: Writing to TTBR0_EL1 sets the page table base for lower VA space.
    // The caller must ensure ttbr0 points to a valid page table.
    unsafe {
        core::arch::asm!("msr TTBR0_EL1, {}", in(reg) ttbr0);
    }
}

/// Set TTBR1_EL1 (Translation Table Base Register 1)
fn set_ttbr1(ttbr1: u64) {
    // SAFETY: Writing to TTBR1_EL1 sets the page table base for upper VA space.
    // The caller must ensure ttbr1 points to a valid page table.
    unsafe {
        core::arch::asm!("msr TTBR1_EL1, {}", in(reg) ttbr1);
    }
}

/// Invalidate all TLB entries
fn invalidate_tlb_all() {
    // SAFETY: TLBI ALL invalidates all TLB entries. This is required
    // when changing page tables to prevent stale translations.
    unsafe {
        core::arch::asm!("tlbi vmalle1is");
        core::arch::asm!("dsb ish");
        core::arch::asm!("isb");
    }
}

/// Enable MMU by setting SCTLR_EL1.M bit
fn enable_mmu() {
    let sctlr: u64;
    // SAFETY: Reading SCTLR_EL1 to preserve existing configuration bits
    // before enabling MMU (M bit).
    unsafe {
        core::arch::asm!("mrs {}, SCTLR_EL1", out(reg) sctlr);
        // Set M bit (bit 0) to enable MMU, C bit (bit 2) for data cache
        let sctlr_new = sctlr | (1 << 0) | (1 << 2);
        core::arch::asm!("msr SCTLR_EL1, {}", in(reg) sctlr_new);
        core::arch::asm!("isb");
    }
}

/// Initialize ARM64 MMU
pub fn init_mmu() {
    configure_mair();
    configure_tcr();
    // Page table base addresses must be set by platform-specific code
    // before calling init_mmu(). Using 0 as placeholder; real boot
    // code will set these via set_ttbr0/set_ttbr1.
    set_ttbr0(0);
    set_ttbr1(0);
    invalidate_tlb_all();
    enable_mmu();
}
