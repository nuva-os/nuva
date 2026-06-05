/*
 * Nuva OS - Kernel - RISC-V 64 PLIC Interrupt Controller
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

//! PLIC (Platform-Level Interrupt Controller) driver for RISC-V.
//! Implements IrqControllerOps trait.

use core::sync::atomic::{AtomicU32, Ordering};

use crate::kernel::arch::*;

/// QEMU virt PLIC default base address.
const PLIC_BASE_DEFAULT: usize = 0x0C000000;

/// Maximum number of IRQ sources on QEMU virt.
const PLIC_MAX_IRQS: u32 = 128;

// PLIC register offsets (relative to base)
const PLIC_PRIORITY: usize = 0x0000;
const PLIC_PENDING: usize = 0x1000;
const PLIC_ENABLE: usize = 0x2000;
const PLIC_THRESHOLD: usize = 0x500000;
const PLIC_CLAIM: usize = 0x500004;
const PLIC_COMPLETE: usize = 0x500004;

/// PLIC driver structure.
pub struct Plic {
    /// MMIO base address.
    base: usize,
    /// Number of IRQ sources.
    num_irqs: u32,
}

impl Plic {
    /// Create a new PLIC instance with the given base address.
    pub const fn new(base: usize, num_irqs: u32) -> Self {
        Plic { base, num_irqs }
    }

    /// Read a PLIC register.
    fn read_reg(&self, offset: usize) -> u32 {
        unsafe { ((self.base + offset) as *const u32).read_volatile() }
    }

    /// Write a PLIC register.
    fn write_reg(&self, offset: usize, val: u32) {
        unsafe { ((self.base + offset) as *mut u32).write_volatile(val) }
    }

    /// Claim the highest-priority pending interrupt for context 0 (Hart 0, S-mode).
    pub fn claim(&self) -> u32 {
        self.read_reg(PLIC_CLAIM)
    }

    /// Complete (EOI) an interrupt for context 0.
    pub fn complete(&self, irq: u32) {
        self.write_reg(PLIC_COMPLETE, irq);
    }

    /// Set the priority threshold for context 0.
    pub fn set_threshold(&self, priority: u32) {
        self.write_reg(PLIC_THRESHOLD, priority);
    }

    /// Enable an IRQ source for context 0.
    pub fn set_enable(&self, irq: u32, enable: bool) {
        if irq == 0 || irq >= self.num_irqs {
            return;
        }
        let reg_offset = PLIC_ENABLE + ((irq / 32) as usize) * 4;
        let bit = 1u32 << (irq % 32);
        let old = self.read_reg(reg_offset);
        if enable {
            self.write_reg(reg_offset, old | bit);
        } else {
            self.write_reg(reg_offset, old & !bit);
        }
    }

    /// Set priority for an IRQ source.
    pub fn set_priority(&self, irq: u32, priority: u32) {
        if irq == 0 || irq >= self.num_irqs {
            return;
        }
        self.write_reg(PLIC_PRIORITY + (irq as usize) * 4, priority);
    }
}

/// Global PLIC instance.
static mut PLIC: Plic = Plic::new(PLIC_BASE_DEFAULT, PLIC_MAX_IRQS);

/// Next available IRQ number for allocation.
static NEXT_IRQ: AtomicU32 = AtomicU32::new(1);

/// RISC-V 64 interrupt controller implementation.
pub struct RiscV64IrqController;

impl IrqControllerOps for RiscV64IrqController {
    fn init(&self) {
        log_info!("RISC-V: Initializing PLIC at {:#x}", PLIC_BASE_DEFAULT);
        // SAFETY: PLIC is accessed only during single-threaded init.
        unsafe {
            PLIC.set_threshold(0);
            // Disable all interrupts
            for irq in 1..PLIC_MAX_IRQS {
                PLIC.set_enable(irq, false);
                PLIC.set_priority(irq, 0);
            }
        }
    }

    fn alloc_irq(&self) -> Option<u32> {
        let irq = NEXT_IRQ.fetch_add(1, Ordering::SeqCst);
        if irq < PLIC_MAX_IRQS {
            Some(irq)
        } else {
            None
        }
    }

    fn free_irq(&self, _irq: u32) {
        // TODO: Return IRQ to allocation pool
    }

    fn register_handler(&self, irq: u32, _handler: fn(u32), _flags: u32) -> bool {
        log_info!("RISC-V: Registering handler for IRQ {}", irq);
        // SAFETY: PLIC enable is a simple MMIO write.
        unsafe {
            PLIC.set_enable(irq, true);
            PLIC.set_priority(irq, 1);
        }
        true
    }

    fn unregister_handler(&self, irq: u32) {
        // SAFETY: PLIC disable is a simple MMIO write.
        unsafe {
            PLIC.set_enable(irq, false);
        }
    }

    fn enable_irq(&self, irq: u32) {
        unsafe { PLIC.set_enable(irq, true); }
    }

    fn disable_irq(&self, irq: u32) {
        unsafe { PLIC.set_enable(irq, false); }
    }

    fn eoi(&self, irq: u32) {
        unsafe { PLIC.complete(irq); }
    }

    fn set_affinity(&self, _irq: u32, _cpu_mask: u64) {
        // PLIC on QEMU virt supports only Hart 0 for S-mode
    }

    fn get_irq_count(&self, _irq: u32) -> u64 {
        0
    }
}

/// Claim the highest-priority pending IRQ.
pub fn plic_claim() -> u32 {
    unsafe { PLIC.claim() }
}

/// Complete (EOI) an IRQ.
pub fn plic_complete(irq: u32) {
    unsafe { PLIC.complete(irq); }
}
