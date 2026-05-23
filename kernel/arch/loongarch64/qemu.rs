/*
 * Nuva OS - Kernel - Arch - LoongArch64 - QEMU Support
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

//! LoongArch64 QEMU virt machine support

use crate::pr_info;

// ============================================================================
// QEMU Virt Machine Constants
// ============================================================================

/// QEMU serial port base address
pub const QEMU_UART_BASE: u64 = 0x1FE0_0000;

/// QEMU UART register offsets
pub const UART_THR: u64 = 0x00;
pub const UART_LSR: u64 = 0x05;

/// UART LSR: Transmitter Holding Register Empty bit
const UART_LSR_THRE: u8 = 1 << 5;

/// EIOINTC (Extended I/O Interrupt Controller) base address
pub const EIOINTC_BASE: u64 = 0x1000_0000;

/// EIOINTC register offsets
const EIOINTC_ENABLE: u64 = 0x04;
const EIOINTC_BOUNCE: u64 = 0x08;
const EIOINTC_ISR: u64 = 0x40;
const EIOINTC_AUTO_CTRL0: u64 = 0xC0;

/// QEMU timer frequency (100 MHz stable counter)
const QEMU_TIMER_FREQ: u64 = 100_000_000;

/// QEMU memory layout
pub const QEMU_RAM_BASE: u64 = 0x0000_0000;
pub const QEMU_RAM_SIZE: u64 = 0x4000_0000;
pub const QEMU_KERNEL_ENTRY: u64 = 0x9000_0000_0020_0000;

// ============================================================================
// QEMU Serial Port
// ============================================================================

/// Initialize QEMU serial port for early console output
pub fn qemu_uart_init() {
    log_info!("QEMU UART initialized at {:#x}", QEMU_UART_BASE);
}

/// Write a byte to QEMU serial port
pub fn qemu_uart_putc(c: u8) {
    // SAFETY: Writing to MMIO UART register at known QEMU-defined address.
    // The QEMU virt machine maps the UART at QEMU_UART_BASE. This is a
    // side-effect-only write with no memory safety implications.
    unsafe {
        let lsr_ptr = (QEMU_UART_BASE + UART_LSR) as *mut u8;
        while core::ptr::read_volatile(lsr_ptr) & UART_LSR_THRE == 0 {
            core::hint::spin_loop();
        }
        let thr_ptr = (QEMU_UART_BASE + UART_THR) as *mut u8;
        core::ptr::write_volatile(thr_ptr, c);
    }
}

/// Write a string to QEMU serial port
pub fn qemu_uart_puts(s: &str) {
    for &b in s.as_bytes() {
        qemu_uart_putc(b);
    }
}

// ============================================================================
// EIOINTC (Extended I/O Interrupt Controller)
// ============================================================================

/// Initialize EIOINTC for QEMU virt machine
pub fn eiointc_init() {
    // SAFETY: Writing to EIOINTC MMIO registers at known QEMU-defined address.
    // These writes configure the interrupt controller; no memory safety issues.
    unsafe {
        let enable_ptr = (EIOINTC_BASE + EIOINTC_ENABLE) as *mut u64;
        core::ptr::write_volatile(enable_ptr, 0);

        let bounce_ptr = (EIOINTC_BASE + EIOINTC_BOUNCE) as *mut u64;
        core::ptr::write_volatile(bounce_ptr, 0);

        let auto_ctrl_ptr = (EIOINTC_BASE + EIOINTC_AUTO_CTRL0) as *mut u64;
        core::ptr::write_volatile(auto_ctrl_ptr, 0);
    }

    log_info!("EIOINTC initialized at {:#x}", EIOINTC_BASE);
}

/// Enable a specific interrupt in EIOINTC
pub fn eiointc_enable_irq(irq: u32) {
    if irq >= 256 {
        return;
    }
    // SAFETY: Writing to EIOINTC enable register. Only affects the specified
    // interrupt bit; no memory safety implications.
    unsafe {
        let enable_ptr = (EIOINTC_BASE + EIOINTC_ENABLE) as *mut u64;
        let word = (irq / 64) as usize;
        let bit = irq % 64;
        let current = core::ptr::read_volatile(enable_ptr.add(word));
        core::ptr::write_volatile(enable_ptr.add(word), current | (1u64 << bit));
    }
}

/// Disable a specific interrupt in EIOINTC
pub fn eiointc_disable_irq(irq: u32) {
    if irq >= 256 {
        return;
    }
    // SAFETY: Writing to EIOINTC enable register. Only affects the specified
    // interrupt bit; no memory safety implications.
    unsafe {
        let enable_ptr = (EIOINTC_BASE + EIOINTC_ENABLE) as *mut u64;
        let word = (irq / 64) as usize;
        let bit = irq % 64;
        let current = core::ptr::read_volatile(enable_ptr.add(word));
        core::ptr::write_volatile(enable_ptr.add(word), current & !(1u64 << bit));
    }
}

/// Get pending interrupt status from EIOINTC
pub fn eiointc_get_pending() -> u64 {
    // SAFETY: Reading EIOINTC ISR register. Read-only operation with no side effects.
    unsafe {
        let isr_ptr = (EIOINTC_BASE + EIOINTC_ISR) as *const u64;
        core::ptr::read_volatile(isr_ptr)
    }
}

// ============================================================================
// QEMU Timer Configuration
// ============================================================================

/// Initialize QEMU timer (stable counter based)
pub fn qemu_timer_init() {
    // SAFETY: Writing CSR TCFG to configure the timer. The timer compare
    // register is a per-CPU CSR with no cross-CPU side effects.
    unsafe {
        let tcfg_val: u32 = (QEMU_TIMER_FREQ / 100) as u32 | (1u32 << 0);
        core::arch::asm!("csrwr {}, 0x41", in(reg) tcfg_val);
    }
    log_info!("QEMU timer initialized, freq={} Hz", QEMU_TIMER_FREQ);
}

/// Read QEMU stable counter
pub fn qemu_timer_read() -> u64 {
    let count: u64;
    // SAFETY: Reading the stable counter CSR (0x4). Read-only operation.
    unsafe {
        core::arch::asm!("csrrd {}, 0x4", out(reg) count);
    }
    count
}

/// Get QEMU timer frequency
pub const fn qemu_timer_freq() -> u64 {
    QEMU_TIMER_FREQ
}

// ============================================================================
// QEMU Early Boot
// ============================================================================

/// QEMU virt machine early initialization
pub fn qemu_early_init() {
    qemu_uart_init();
    eiointc_init();
    qemu_timer_init();
    log_info!("QEMU LoongArch64 virt machine initialized");
}

/// QEMU virt machine memory layout information
#[derive(Debug, Clone, Copy)]
pub struct QemuMemLayout {
    /// RAM base physical address
    pub ram_base: u64,
    /// RAM size in bytes
    pub ram_size: u64,
    /// Kernel entry point virtual address
    pub kernel_entry: u64,
}

/// Get QEMU virt machine memory layout
pub const fn qemu_mem_layout() -> QemuMemLayout {
    QemuMemLayout {
        ram_base: QEMU_RAM_BASE,
        ram_size: QEMU_RAM_SIZE,
        kernel_entry: QEMU_KERNEL_ENTRY,
    }
}
