/*
 * Nuva OS - HAL - LoongArch64 CPU
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */

// LoongArch64 CPU abstraction

/// Get current core ID
pub fn current_core_id() -> u32 {
    let core_id: u32;
    // SAFETY: CSR read instruction (csrrd) reads the processor ID CSR (0x20).
    // This is a read-only system register operation with no memory side effects.
    unsafe {
        core::arch::asm!("csrrd {}, 0x20", out(reg) core_id);
    }
    core_id
}

/// Wait for interrupt (idle hint)
pub fn idle() {
    // SAFETY: The idle instruction is a hint that suspends the processor
    // until an interrupt or event occurs. No memory safety implications.
    unsafe {
        core::arch::asm!("idle 0");
    }
}
