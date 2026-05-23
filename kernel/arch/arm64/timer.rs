/*
 * Nuva OS - Kernel - Kernel
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


use core::arch::asm;
use crate::{pr_info};

/// Timer frequency
pub static mut TIMER_FREQ: u32 = 0;

/// Read CNTPCT (Physical Counter)
#[inline(always)]
pub fn read_cntpct() -> u64 {
    let val: u64;
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!(
            "mrs {}, cntpct_el0",
            out(reg) val,
        );
    }
    val
}

/// Read CNTVCT (Virtual Counter)
#[inline(always)]
pub fn read_cntvct() -> u64 {
    let val: u64;
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!(
            "mrs {}, cntvct_el0",
            out(reg) val,
        );
    }
    val
}

/// Read CNTFRQ (Counter Frequency)
#[inline(always)]
pub fn read_cntfrq() -> u32 {
    let val: u32;
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!(
            "mrs {}, cntfrq_el0",
            out(reg) val,
        );
    }
    val
}

/// Write CNTFRQ (Counter Frequency)
#[inline(always)]
pub fn write_cntfrq(val: u32) {
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!(
            "msr cntfrq_el0, {}",
            in(reg) val,
        );
    }
}

/// Read CNTP_TVAL (Physical Timer value)
#[inline(always)]
pub fn read_cntp_tval() -> u32 {
    let val: u32;
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!(
            "mrs {}, cntp_tval_el0",
            out(reg) val,
        );
    }
    val
}

/// Write CNTP_TVAL (Physical Timer value)
#[inline(always)]
pub fn write_cntp_tval(val: u32) {
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!(
            "msr cntp_tval_el0, {}",
            in(reg) val,
        );
    }
}

/// Read CNTP_CVAL (Physical Timer Compare value)
#[inline(always)]
pub fn read_cntp_cval() -> u64 {
    let val: u64;
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!(
            "mrs {}, cntp_cval_el0",
            out(reg) val,
        );
    }
    val
}

/// Write CNTP_CVAL (Physical Timer Compare value)
#[inline(always)]
pub fn write_cntp_cval(val: u64) {
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!(
            "msr cntp_cval_el0, {}",
            in(reg) val,
        );
    }
}

/// Read CNTP_CTL (Physical Timer Control)
#[inline(always)]
pub fn read_cntp_ctl() -> u32 {
    let val: u32;
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!(
            "mrs {}, cntp_ctl_el0",
            out(reg) val,
        );
    }
    val
}

/// Write CNTP_CTL (Physical Timer Control)
#[inline(always)]
pub fn write_cntp_ctl(val: u32) {
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!(
            "msr cntp_ctl_el0, {}",
            in(reg) val,
        );
    }
}

/// Enable physical timer
#[inline(always)]
pub fn enable_timer() {
    write_cntp_ctl(1);
}

/// Disable physical timer
#[inline(always)]
pub fn disable_timer() {
    write_cntp_ctl(0);
}

/// Set timer (relative time)
pub fn set_timer_relative(us: u64) {
    // SAFETY: unsafe block required for low-level memory or hardware access
    let freq = unsafe { TIMER_FREQ } as u64;
    let ticks = (us * freq) / 1_000_000;
    write_cntp_tval(ticks as u32);
    enable_timer();
}

/// Set timer (absolute time)
pub fn set_timer_absolute(ticks: u64) {
    write_cntp_cval(ticks);
    enable_timer();
}

/// Get current time (microseconds)
pub fn get_time_us() -> u64 {
    let cnt = read_cntpct();
    // SAFETY: unsafe block required for low-level memory or hardware access
    let freq = unsafe { TIMER_FREQ } as u64;
    (cnt * 1_000_000) / freq
}

/// Get current time (milliseconds)
pub fn get_time_ms() -> u64 {
    let cnt = read_cntpct();
    // SAFETY: unsafe block required for low-level memory or hardware access
    let freq = unsafe { TIMER_FREQ } as u64;
    (cnt * 1_000) / freq
}

/// Get current time (nanoseconds)
pub fn get_time_ns() -> u64 {
    let cnt = read_cntpct();
    // SAFETY: unsafe block required for low-level memory or hardware access
    let freq = unsafe { TIMER_FREQ } as u64;
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
    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe {
        TIMER_FREQ = read_cntfrq();
    }

    // Disable timer
    disable_timer();

    log_info!("ARM Generic Timer initialized");
    // SAFETY: unsafe block required for low-level memory or hardware access
    log_info!("  Frequency: {} Hz", unsafe { TIMER_FREQ });
}
