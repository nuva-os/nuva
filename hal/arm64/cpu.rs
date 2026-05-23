/*
 * Nuva OS - HAL - ARM64 CPU
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */

// ARM64 CPU abstraction

/// Read MIDR_EL1 to identify CPU
pub fn read_midr() -> u64 {
    let midr: u64;
    // SAFETY: MRS instruction reads a system register. It has no side effects
    // on memory and does not violate Rust's safety guarantees.
    unsafe {
        core::arch::asm!("mrs {}, MIDR_EL1", out(reg) midr);
    }
    midr
}

/// Get current core ID (MPIDR_EL1)
pub fn current_core_id() -> u32 {
    let mpidr: u64;
    // SAFETY: MRS instruction reads MPIDR_EL1 system register.
    // No memory side effects, safe to call from any context.
    unsafe {
        core::arch::asm!("mrs {}, MPIDR_EL1", out(reg) mpidr);
    }
    // Extract Aff0 (bits [7:0]) as the core ID
    (mpidr & 0xFF) as u32
}

/// Wait for interrupt (WFI)
pub fn wfi() {
    // SAFETY: WFI is a hint instruction that suspends execution until
    // an interrupt occurs. It has no memory safety implications.
    unsafe {
        core::arch::asm!("wfi");
    }
}

/// Wait for event (WFE)
pub fn wfe() {
    // SAFETY: WFE is a hint instruction similar to WFI.
    // It has no memory safety implications.
    unsafe {
        core::arch::asm!("wfe");
    }
}

/// Send event (SEV)
pub fn sev() {
    // SAFETY: SEV is a hint instruction that sends an event
    // to all cores. It has no memory safety implications.
    unsafe {
        core::arch::asm!("sev");
    }
}
