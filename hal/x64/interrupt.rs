/*
 * Nuva OS - HAL - x86-64 Interrupt
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

// x86-64 interrupt and exception handling via APIC

/// Local APIC base MSR
const IA32_APIC_BASE: u32 = 0x1B;

/// Local APIC default physical address
const APIC_DEFAULT_BASE: u64 = 0xFEE0_0000;

/// APIC register offsets
mod apic_reg {
    pub const ID: u64 = 0x020;
    pub const VERSION: u64 = 0x030;
    pub const TPR: u64 = 0x080;
    pub const PPR: u64 = 0x0A0;
    pub const EOI: u64 = 0x0B0;
    pub const LDR: u64 = 0x0D0;
    pub const DFR: u64 = 0x0E0;
    pub const SPIV: u64 = 0x0F0;
    pub const ICR_LOW: u64 = 0x300;
    pub const ICR_HIGH: u64 = 0x310;
    pub const LVT_TIMER: u64 = 0x320;
    pub const LVT_LINT0: u64 = 0x350;
    pub const LVT_LINT1: u64 = 0x360;
    pub const LVT_ERROR: u64 = 0x370;
    pub const TIMER_ICR: u64 = 0x380;
    pub const TIMER_CCR: u64 = 0x390;
    pub const TIMER_DCR: u64 = 0x3E0;
    pub const ERROR_STATUS: u64 = 0x280;
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

/// Read MSR
unsafe fn rdmsr(reg: u32) -> u64 {
    let (low, high): (u32, u32);
    // SAFETY: inline assembly required for hardware instruction
    core::arch::asm!(
        "rdmsr",
        in("ecx") reg,
        lateout("eax") low,
        lateout("edx") high,
    );
    ((high as u64) << 32) | (low as u64)
}

/// Get APIC base address
fn get_apic_base() -> u64 {
    // SAFETY: Reading IA32_APIC_BASE MSR to get APIC base address
    unsafe {
        let val = rdmsr(IA32_APIC_BASE);
        val & 0xFFFF_F000
    }
}

/// Initialize x86-64 interrupt controller (Local APIC)
pub fn init_interrupt() {
    let apic_base = get_apic_base();
    if apic_base == 0 {
        return;
    }

    // SAFETY: Local APIC initialization via MMIO.
    // The base address is read from MSR IA32_APIC_BASE.
    // This operation is required during early boot before any interrupts are delivered.
    unsafe {
        // Set Spurious Interrupt Vector Register (enable APIC, set vector 0xFF)
        mmio_write32(apic_base + apic_reg::SPIV, 0x1FF);

        // Mask all LVT entries initially
        mmio_write32(apic_base + apic_reg::LVT_TIMER, 0x10000);
        mmio_write32(apic_base + apic_reg::LVT_LINT0, 0x10000);
        mmio_write32(apic_base + apic_reg::LVT_LINT1, 0x10000);
        mmio_write32(apic_base + apic_reg::LVT_ERROR, 0x10000);
    }
}

/// Enable interrupts (STI)
pub fn enable_irq() {
    // SAFETY: STI sets the IF flag in RFLAGS, enabling maskable interrupts.
    unsafe {
        core::arch::asm!("sti");
    }
}

/// Disable interrupts (CLI)
pub fn disable_irq() {
    // SAFETY: CLI clears the IF flag in RFLAGS, disabling maskable interrupts.
    unsafe {
        core::arch::asm!("cli");
    }
}

/// Send EOI (End of Interrupt) to Local APIC
pub fn send_eoi() {
    let apic_base = get_apic_base();
    if apic_base == 0 {
        return;
    }
    // SAFETY: Writing 0 to EOI register signals end of interrupt handling
    unsafe {
        mmio_write32(apic_base + apic_reg::EOI, 0);
    }
}
