/*
 * Nuva OS - HAL - LoongArch64 Interrupt
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

// LoongArch64 interrupt and exception handling via EIOINTC

/// EIOINTC base address (QEMU virt)
const EIOINTC_BASE: u64 = 0x1FE0_0000;

/// EIOINTC register offsets
mod eiointc_reg {
    pub const CTLR: u64 = 0x0000;
    pub const STATUS: u64 = 0x0010;
    pub const ENABLE: u64 = 0x0020;
    pub const DISABLE: u64 = 0x0028;
    pub const EOI: u64 = 0x0040;
    pub const AUTO_EOI: u64 = 0x0050;
    pub const ROUTE: u64 = 0x0060;
}

/// CSR register addresses for interrupt control
mod csr {
    pub const CRMD: u32 = 0x0;
    pub const ECFG: u32 = 0x4;
    pub const ESTAT: u32 = 0x5;
    pub const EENTRY: u32 = 0xc;
    pub const ECLR: u32 = 0x16;
}

/// Write to MMIO register
unsafe fn mmio_write32(addr: u64, val: u32) {
    // SAFETY: Caller ensures addr is a valid MMIO address.
    core::ptr::write_volatile(addr as *mut u32, val);
}

/// Read from MMIO register
unsafe fn mmio_read32(addr: u64) -> u32 {
    // SAFETY: Caller ensures addr is a valid MMIO address.
    core::ptr::read_volatile(addr as *const u32)
}

/// Initialize LoongArch64 interrupt controller (EIOINTC)
pub fn init_interrupt() {
    // SAFETY: EIOINTC initialization via MMIO.
    // The base address is a platform-specific constant.
    // This operation is required during early boot before any interrupts are delivered.
    unsafe {
        // Disable all interrupts initially
        mmio_write32(EIOINTC_BASE + eiointc_reg::DISABLE, 0xFFFF_FFFF);

        // Enable EIOINTC controller
        mmio_write32(EIOINTC_BASE + eiointc_reg::CTLR, 1);
    }

    // Set exception entry address
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        core::arch::asm!(
            "csrwr $r0, 0xc",
        );
    }

    // Enable external interrupts in ECFG (set interrupt line enable bits)
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        let ecfg: u32;
        core::arch::asm!(
            "csrrd {}, 0x4",
            out(reg) ecfg,
        );
        // Enable LIE (Local Interrupt Enable) bits for external interrupts
        core::arch::asm!(
            "csrwr {}, 0x4",
            in(reg) ecfg | 0xFFFF_F000,
        );
    }
}

/// Enable interrupts (set IE bit in CRMD)
pub fn enable_irq() {
    // SAFETY: Setting IE bit in CRMD register enables interrupts.
    unsafe {
        core::arch::asm!(
            "csrrd $t0, 0x0",
            "ori $t0, $t0, 1",
            "csrwr $t0, 0x0",
        );
    }
}

/// Disable interrupts (clear IE bit in CRMD)
pub fn disable_irq() {
    // SAFETY: Clearing IE bit in CRMD register disables interrupts.
    unsafe {
        core::arch::asm!(
            "csrrd $t0, 0x0",
            "andi $t0, $t0, ~1",
            "csrwr $t0, 0x0",
        );
    }
}

/// Send EOI (End of Interrupt) to EIOINTC
/// @param irq: Interrupt number to acknowledge
pub fn send_eoi(irq: u32) {
    // SAFETY: Writing to EOI register signals end of interrupt handling
    unsafe {
        mmio_write32(EIOINTC_BASE + eiointc_reg::EOI, irq);
    }
}

/// Clear interrupt pending in ECLR
pub fn clear_interrupt_pending() {
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        core::arch::asm!(
            "csrwr $r0, 0x16",
        );
    }
}
