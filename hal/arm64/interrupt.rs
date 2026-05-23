/*
 * Nuva OS - HAL - ARM64 Interrupt
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */

// ARM64 interrupt and exception handling

/// GICv3 Distributor base address (default for QEMU virt)
const GICD_BASE: u64 = 0x0800_0000;

/// GICv3 Redistributor base address (default for QEMU virt)
const GICR_BASE: u64 = 0x080A_0000;

/// GICD_CTLR: Enable Group0 (bit 0) and Group1 (bit 1) interrupts
const GICD_CTLR_ENABLE_BOTH: u32 = 0x3;

/// GICD_CTLR offset
const GICD_CTLR: u64 = 0x0000;

/// GICD_SCTLR offset (ARE enable)
const GICD_SCTLR_ARE: u32 = 0x1 << 4;

/// GICR_CTLR offset
const GICR_CTLR: u64 = 0x0000;

/// GICR_SGI offset base
const GICR_IGROUPR0: u64 = 0x0080;

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

/// Initialize ARM64 interrupt controller (GICv3)
pub fn init_interrupt() {
    // SAFETY: GICv3 distributor and redistributor initialization via MMIO.
    // The base addresses are platform-specific constants. This operation
    // is required during early boot before any interrupts are delivered.
    unsafe {
        // Enable GICv3 ARE (Affinity Routing Enable) in Distributor
        let ctlr = mmio_read32(GICD_BASE + GICD_CTLR);
        mmio_write32(GICD_BASE + GICD_CTLR, ctlr | GICD_SCTLR_ARE | GICD_CTLR_ENABLE_BOTH);

        // Enable Group0 and Group1 in Redistributor for current CPU
        let gicr_ctlr = mmio_read32(GICR_BASE + GICR_CTLR);
        mmio_write32(GICR_BASE + GICR_CTLR, gicr_ctlr | 0x1);

        // Enable SGI (Software Generated Interrupts) for Group0
        mmio_write32(GICR_BASE + GICR_IGROUPR0, 0xFFFF_FFFF);
    }
}

/// Enable interrupts (clear DAIF.I bit)
pub fn enable_irq() {
    // SAFETY: DAIFClr #2 clears the IRQ mask bit (DAIF.I) in DAIF register.
    // This is a standard ARM64 system instruction with no memory side effects.
    unsafe {
        core::arch::asm!("msr DAIFClr, #2");
    }
}

/// Disable interrupts (set DAIF.I bit)
pub fn disable_irq() {
    // SAFETY: DAIFSet #2 sets the IRQ mask bit (DAIF.I) in DAIF register.
    // This is a standard ARM64 system instruction with no memory side effects.
    unsafe {
        core::arch::asm!("msr DAIFSet, #2");
    }
}
