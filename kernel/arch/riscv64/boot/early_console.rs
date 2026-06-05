/*
 * Nuva OS - Kernel - RISC-V 64 Boot Early Console
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

//! SBI early console output for boot-time debugging.
//! Uses the SBI debug console extension (eid=0x44, fid=0x02) or
//! falls back to the legacy putchar (eid=0x01).

use core::arch::asm;

/// Initialize early console (no-op for SBI console).
pub fn init_early_console() {
    // SBI console is always available; no hardware init needed.
}

/// Output a single character via SBI console putchar.
pub fn early_putchar(ch: u8) {
    // SAFETY: SBI ecall is the specified interface for console output
    // in S-mode RISC-V firmware environments.
    unsafe {
        asm!(
            "li a7, 1",         // Legacy console putchar EID
            "mv a0, {0}",
            "ecall",
            in(reg) ch as u64,
            out("a0") _,
            out("a7") _,
        );
    }
}

/// Output a byte slice via early console.
pub fn early_print(s: &[u8]) {
    for &b in s {
        early_putchar(b);
    }
}
