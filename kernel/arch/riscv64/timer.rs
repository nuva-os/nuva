/*
 * Nuva OS - Kernel - RISC-V 64 Timer Operations
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

//! RISC-V timer operations implementing TimerOps trait.
//! Uses SBI timer extension for setting timers and mtime for reading time.

use core::arch::asm;
use core::sync::atomic::{AtomicU64, Ordering};

use crate::kernel::arch::*;
use super::sbi;

/// Default timer frequency for QEMU virt (10 MHz).
const DEFAULT_FREQ: u64 = 10_000_000;

/// QEMU virt ACLINT mtime register address.
const MTIME_ADDR_DEFAULT: usize = 0x0200_BFF8;

/// Timer frequency in Hz (read from FDT or default).
static TIMER_FREQ: AtomicU64 = AtomicU64::new(DEFAULT_FREQ);

/// mtime MMIO address.
static MTIME_ADDR: AtomicU64 = AtomicU64::new(MTIME_ADDR_DEFAULT as u64);

/// Read the mtime register (64-bit memory-mapped timer).
fn read_mtime() -> u64 {
    let addr = MTIME_ADDR.load(Ordering::SeqCst);
    // SAFETY: mtime is a memory-mapped register that is always readable.
    unsafe { (addr as *const u64).read_volatile() }
}

/// Convert mtime ticks to nanoseconds.
fn ticks_to_ns(ticks: u64) -> u64 {
    let freq = TIMER_FREQ.load(Ordering::SeqCst);
    if freq == 0 {
        return 0;
    }
    // ns = ticks * 1_000_000_000 / freq
    // Use 64-bit arithmetic; avoid overflow for large tick values
    ticks / freq * 1_000_000_000 + (ticks % freq) * 1_000_000_000 / freq
}

/// RISC-V timer implementation.
pub struct RiscV64Timer;

impl TimerOps for RiscV64Timer {
    fn init(&self) {
        log_info!("RISC-V: Initializing timer (freq={} Hz)", DEFAULT_FREQ);
        TIMER_FREQ.store(DEFAULT_FREQ, Ordering::SeqCst);
        MTIME_ADDR.store(MTIME_ADDR_DEFAULT as u64, Ordering::SeqCst);
        // TODO: Read timebase-frequency from FDT
    }

    fn now(&self) -> u64 {
        ticks_to_ns(read_mtime())
    }

    fn set_oneshot(&self, ns: u64) {
        let freq = TIMER_FREQ.load(Ordering::SeqCst);
        if freq == 0 {
            return;
        }
        let delta_ticks = ns * freq / 1_000_000_000;
        let next = read_mtime().wrapping_add(delta_ticks);
        let _ = sbi::timer_set(next);
    }

    fn set_periodic(&self, ns: u64) {
        // RISC-V has no hardware periodic timer; emulate via one-shot
        self.set_oneshot(ns);
    }

    fn stop(&self) {
        // Disable timer by setting mtimecmp far in the future
        let _ = sbi::timer_set(u64::MAX);
    }

    fn frequency(&self) -> u64 {
        TIMER_FREQ.load(Ordering::SeqCst)
    }

    fn delay(&self, ns: u64) {
        let start = self.now();
        while self.now().wrapping_sub(start) < ns {
            core::hint::spin_loop();
        }
    }
}
