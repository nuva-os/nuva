/*
* Nuva OS - Kernel - LoongArch64 Timer (Stable Counter)
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

/// Timer frequency (typically 100 MHz for Loongson 3A6000)
pub static mut TIMER_FREQ: u32 = 0;

/// Read stable counter
#[inline(always)]
pub fn read_stable_counter() -> u64 {
    let count: u64;
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!(
            "rdtime.d {}, $r0",
            out(reg) count,
        );
    }
    count
}

/// Read timer counter value (CSR TVAL)
#[inline(always)]
pub fn read_tval() -> u64 {
    let val: u64;
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!(
            "csrrd {}, 0x42",
            out(reg) val,
        );
    }
    val
}

/// Write timer config (CSR TCFG)
/// @param val: Timer config value
#[inline(always)]
pub fn write_tcfg(val: u64) {
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!(
            "csrwr {}, 0x41",
            in(reg) val,
        );
    }
}

/// Write timer interrupt clear (CSR TICLR)
#[inline(always)]
pub fn write_ticlr(val: u64) {
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!(
            "csrwr {}, 0x44",
            in(reg) val,
        );
    }
}

/// Enable timer (set TCFG en bit)
pub fn enable_timer() {
    let tcfg: u64;
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!(
            "csrrd {}, 0x41",
            out(reg) tcfg,
        );
    }
    // Set bit 0 (en) and bit 1 (periodic=0, one-shot)
    write_tcfg(tcfg | 1);
}

/// Disable timer
pub fn disable_timer() {
    let tcfg: u64;
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!(
            "csrrd {}, 0x41",
            out(reg) tcfg,
        );
    }
    // Clear bit 0 (en)
    write_tcfg(tcfg & !1u64);
}

/// Set timer (relative time in nanoseconds)
pub fn set_timer_relative(ns: u64) {
    // SAFETY: unsafe block required for low-level memory or hardware access
    let freq = unsafe { TIMER_FREQ } as u64;
    if freq == 0 {
        return;
    }
    let ticks = ns * freq / 1_000_000_000;
    // TCFG: bit 0 = en, bit 1 = periodic, bits 2..=63 = initval
    write_tcfg((ticks << 2) | 1);
}

/// Set timer (absolute time in counter ticks)
pub fn set_timer_absolute(ticks: u64) {
    write_tcfg((ticks << 2) | 1);
}

/// Clear timer interrupt
pub fn clear_timer_irq() {
    // Write 1 to TICLR to clear timer interrupt
    write_ticlr(1);
}

/// Get current time (microseconds)
pub fn get_time_us() -> u64 {
    let cnt = read_stable_counter();
    // SAFETY: unsafe block required for low-level memory or hardware access
    let freq = unsafe { TIMER_FREQ } as u64;
    if freq == 0 {
        return 0;
    }
    (cnt * 1_000_000) / freq
}

/// Get current time (milliseconds)
pub fn get_time_ms() -> u64 {
    let cnt = read_stable_counter();
    // SAFETY: unsafe block required for low-level memory or hardware access
    let freq = unsafe { TIMER_FREQ } as u64;
    if freq == 0 {
        return 0;
    }
    (cnt * 1_000) / freq
}

/// Get current time (nanoseconds)
pub fn get_time_ns() -> u64 {
    let cnt = read_stable_counter();
    // SAFETY: unsafe block required for low-level memory or hardware access
    let freq = unsafe { TIMER_FREQ } as u64;
    if freq == 0 {
        return 0;
    }
    (cnt * 1_000_000_000) / freq
}

/// Busy wait (microseconds)
pub fn udelay(us: u64) {
    let start = get_time_us();
    while get_time_us() - start < us {
        core::hint::spin_loop();
    }
}

/// Busy wait (milliseconds)
pub fn mdelay(ms: u64) {
    udelay(ms * 1000);
}

/// Initialize timer
pub fn init_timer() {
    // Default frequency for Loongson 3A6000: 100 MHz
    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe {
        TIMER_FREQ = 100_000_000;
    }

    // Disable timer initially
    disable_timer();

    log_info!("LoongArch64 Timer initialized (stable counter)");
    // SAFETY: unsafe block required for low-level memory or hardware access
    log_info!("  Frequency: {} Hz", unsafe { TIMER_FREQ });
}
