/*
 * Nuva OS - Kernel - x86-64 Timer (Local APIC Timer)
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
use crate::pr_info;

/// Timer frequency (detected at init)
pub static mut TIMER_FREQ: u32 = 0;

/// Read timestamp counter
#[inline(always)]
pub fn rdtsc() -> u64 {
    let low: u32;
    let high: u32;
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!(
            "rdtsc",
            lateout("eax") low,
            lateout("edx") high,
        );
    }
    ((high as u64) << 32) | (low as u64)
}

/// Read timestamp counter (serialized)
#[inline(always)]
pub fn rdtscp() -> u64 {
    let low: u32;
    let high: u32;
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!(
            "rdtscp",
            lateout("eax") low,
            lateout("edx") high,
        );
    }
    ((high as u64) << 32) | (low as u64)
}

/// Get current time (microseconds based on TSC)
pub fn get_time_us() -> u64 {
    let tsc = rdtsc();
    // SAFETY: unsafe block required for low-level memory or hardware access
    let freq = unsafe { TIMER_FREQ } as u64;
    if freq == 0 {
        return 0;
    }
    (tsc * 1_000_000) / freq
}

/// Get current time (milliseconds based on TSC)
pub fn get_time_ms() -> u64 {
    let tsc = rdtsc();
    // SAFETY: unsafe block required for low-level memory or hardware access
    let freq = unsafe { TIMER_FREQ } as u64;
    if freq == 0 {
        return 0;
    }
    (tsc * 1_000) / freq
}

/// Get current time (nanoseconds based on TSC)
pub fn get_time_ns() -> u64 {
    let tsc = rdtsc();
    // SAFETY: unsafe block required for low-level memory or hardware access
    let freq = unsafe { TIMER_FREQ } as u64;
    if freq == 0 {
        return 0;
    }
    (tsc * 1_000_000_000) / freq
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
    // Use TSC frequency from CPUID leaf 0x15 (if available)
    // For now, estimate from CPUID leaf 0x16
    let (_, ebx, _, _) = super::cpuid(0x15, 0);
    if ebx != 0 {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            TIMER_FREQ = ebx;
        }
    } else {
        // Fallback: estimate TSC frequency from CPUID 0x16
        let (_, _, ecx, _) = super::cpuid(0x16, 0);
        // ecx = core crystal frequency in MHz
        if ecx != 0 {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                TIMER_FREQ = ecx * 1_000_000;
            }
        }
    }

    log_info!("x86-64 Timer initialized (TSC-based)");
    // SAFETY: unsafe block required for low-level memory or hardware access
    log_info!("  Frequency: {} Hz", unsafe { TIMER_FREQ });
}
