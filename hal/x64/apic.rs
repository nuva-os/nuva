/*
 * Nuva OS - HAL - X64
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



use core::sync::atomic::{AtomicU32, Ordering};

/// APIC register addresses
pub mod apic {
    /// Local APIC base address
    pub const LOCAL_APIC_BASE: u64 = 0xFEE0_0000;

    /// Local APIC registers
    pub const LAPIC_ID: u64 = LOCAL_APIC_BASE + 0x0020;
    pub const LAPIC_VERSION: u64 = LOCAL_APIC_BASE + 0x0030;
    pub const LAPIC_TPR: u64 = LOCAL_APIC_BASE + 0x0080;
    pub const LAPIC_APR: u64 = LOCAL_APIC_BASE + 0x0090;
    pub const LAPIC_PPR: u64 = LOCAL_APIC_BASE + 0x00A0;
    pub const LAPIC_EOI: u64 = LOCAL_APIC_BASE + 0x00B0;
    pub const LAPIC_LDR: u64 = LOCAL_APIC_BASE + 0x00D0;
    pub const LAPIC_DFR: u64 = LOCAL_APIC_BASE + 0x00E0;
    pub const LAPIC_SVR: u64 = LOCAL_APIC_BASE + 0x00F0;
    pub const LAPIC_ISR: u64 = LOCAL_APIC_BASE + 0x0100;
    pub const LAPIC_TMR: u64 = LOCAL_APIC_BASE + 0x0180;
    pub const LAPIC_IRR: u64 = LOCAL_APIC_BASE + 0x0200;
    pub const LAPIC_ESR: u64 = LOCAL_APIC_BASE + 0x0280;
    pub const LAPIC_ICR_LOW: u64 = LOCAL_APIC_BASE + 0x0300;
    pub const LAPIC_ICR_HIGH: u64 = LOCAL_APIC_BASE + 0x0310;
    pub const LAPIC_LVT_TIMER: u64 = LOCAL_APIC_BASE + 0x0320;
    pub const LAPIC_LVT_THERMAL: u64 = LOCAL_APIC_BASE + 0x0330;
    pub const LAPIC_LVT_PERF: u64 = LOCAL_APIC_BASE + 0x0340;
    pub const LAPIC_LVT_LINT0: u64 = LOCAL_APIC_BASE + 0x0350;
    pub const LAPIC_LVT_LINT1: u64 = LOCAL_APIC_BASE + 0x0360;
    pub const LAPIC_LVT_ERROR: u64 = LOCAL_APIC_BASE + 0x0370;
    pub const LAPIC_TIMER_ICR: u64 = LOCAL_APIC_BASE + 0x0380;
    pub const LAPIC_TIMER_CCR: u64 = LOCAL_APIC_BASE + 0x0390;
    pub const LAPIC_TIMER_DCR: u64 = LOCAL_APIC_BASE + 0x03E0;

    /// I/O APIC base address
    pub const IO_APIC_BASE: u64 = 0xFEC0_0000;

    /// I/O APIC registers
    pub const IOAPIC_ADDRESS: u64 = IO_APIC_BASE + 0x0000;
    pub const IOAPIC_DATA: u64 = IO_APIC_BASE + 0x0010;
    pub const IOAPIC_EOI: u64 = IO_APIC_BASE + 0x0040;
}

/// Interrupt vectors
pub mod vectors {
    /// Divide error
    pub const DIVIDE_ERROR: u8 = 0;
    /// Debug exception
    pub const DEBUG: u8 = 1;
    /// Non-maskable interrupt
    pub const NMI: u8 = 2;
    /// Breakpoint
    pub const BREAKPOINT: u8 = 3;
    /// Overflow
    pub const OVERFLOW: u8 = 4;
    /// BOUND range exceeded
    pub const BOUND: u8 = 5;
    /// Invalid opcode
    pub const INVALID_OPCODE: u8 = 6;
    /// Device not available
    pub const DEVICE_NOT_AVAILABLE: u8 = 7;
    /// Double fault
    pub const DOUBLE_FAULT: u8 = 8;
    /// Coprocessor segment overrun
    pub const COPROCESSOR_SEGMENT_OVERRUN: u8 = 9;
    /// Invalid TSS
    pub const INVALID_TSS: u8 = 10;
    /// Segment not present
    pub const SEGMENT_NOT_PRESENT: u8 = 11;
    /// Stack segment fault
    pub const STACK_SEGMENT: u8 = 12;
    /// General protection fault
    pub const GENERAL_PROTECTION: u8 = 13;
    /// Page fault
    pub const PAGE_FAULT: u8 = 14;
    /// x87 FPU floating point error
    pub const X87_FPU_ERROR: u8 = 16;
    /// Alignment check
    pub const ALIGNMENT_CHECK: u8 = 17;
    /// Machine check
    pub const MACHINE_CHECK: u8 = 18;
    /// SIMD floating point exception
    pub const SIMD_EXCEPTION: u8 = 19;
    /// Virtualization exception
    pub const VIRTUALIZATION: u8 = 20;

    /// Timer interrupt
    pub const TIMER: u8 = 32;
    /// Keyboard interrupt
    pub const KEYBOARD: u8 = 33;
    /// Mouse interrupt
    pub const MOUSE: u8 = 44;
    /// System call
    pub const SYSCALL: u8 = 0x80;

    /// APIC spurious
    pub const SPURIOUS: u8 = 255;
}

/// APIC mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApicMode {
    /// Disabled
    Disabled,
    /// PIC mode
    Pic,
    /// APIC mode
    Apic,
    /// x2APIC mode
    X2Apic,
}

/// Local APIC
pub struct LocalApic {
    /// APIC ID
    pub id: u32,
    /// APIC version
    pub version: u32,
    /// If enabled
    pub enabled: bool,
    /// Mode
    pub mode: ApicMode,
}

impl LocalApic {
    pub fn new() -> Self {
        LocalApic {
            id: 0,
            version: 0,
            enabled: false,
            mode: ApicMode::Disabled,
        }
    }

    /// Initialize
    pub fn init(&mut self) {
        // Read APIC ID
        self.id = self.read_id();
        self.version = self.read_version();
        self.enabled = true;
        self.mode = ApicMode::Apic;

        log_info!("Local APIC initialized");
        log_info!("  ID: {}", self.id);
        log_info!("  Version: {}", self.version);
    }

    /// Read APIC ID
    fn read_id(&self) -> u32 {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let value = read_u32(apic::LAPIC_ID);
            (value >> 24) & 0xFF
        }
    }

    /// Read version
    fn read_version(&self) -> u32 {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            read_u32(apic::LAPIC_VERSION) & 0xFF
        }
    }

    /// Send EOI
    pub fn send_eoi(&self) {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            write_u32(apic::LAPIC_EOI, 0);
        }
    }

    /// Set task priority
    pub fn set_tpr(&self, priority: u32) {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let value = (priority & 0xFF) << 4;
            write_u32(apic::LAPIC_TPR, value);
        }
    }

    /// Enable
    pub fn enable(&mut self) {
        // Set SVR register
        self.enabled = true;
    }

    /// Disable
    pub fn disable(&mut self) {
        self.enabled = false;
    }
}

/// I/O APIC
pub struct IoApic {
    /// I/O APIC ID
    pub id: u32,
    /// Version
    pub version: u32,
    /// Number of interrupt inputs
    pub num_inputs: u32,
    /// Base address
    pub base: u64,
}

impl IoApic {
    pub fn new(base: u64) -> Self {
        IoApic {
            id: 0,
            version: 0,
            num_inputs: 24,
            base,
        }
    }

    /// Initialize
    pub fn init(&mut self) {
        self.id = self.read_id();
        self.version = self.read_version();

        log_info!("I/O APIC initialized");
        log_info!("  ID: {}", self.id);
        log_info!("  Version: {}", self.version);
        log_info!("  Inputs: {}", self.num_inputs);
    }

    /// Read ID
    fn read_id(&self) -> u32 {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            // I/O APIC ID is in bits 24-27 of index 0x00
            write_u32(self.base + apic::IOAPIC_ADDRESS, 0x00);
            let value = read_u32(self.base + apic::IOAPIC_DATA);
            (value >> 24) & 0x0F
        }
    }

    /// Read version
    fn read_version(&self) -> u32 {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            // I/O APIC version is at index 0x01
            write_u32(self.base + apic::IOAPIC_ADDRESS, 0x01);
            let value = read_u32(self.base + apic::IOAPIC_DATA);
            self.num_inputs = ((value >> 16) & 0xFF) + 1;
            value & 0xFF
        }
    }

    /// Set redirection table
    pub fn set_redirect(&self, irq: u8, vector: u8, delivery_mode: u8, dest: u8) {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            // I/O APIC redirection table starts at index 0x10, each IRQ occupies 2 32-bit registers
            let index = 0x10 + (irq as u64) * 2;

            // Low 32 bits
            let low = (vector as u32) |
                     ((delivery_mode as u32) << 8) |
                     (0 << 11) | // Delivery Status (read-only)
                     (1 << 12) | // Polarity: Active High
                     (0 << 13) | // Trigger Mode: Edge
                     (0 << 14) | // Interrupt Mask: Unmasked
                     (0 << 16); // Destination Mode: Physical

            // High 32 bits
            let high = (dest as u32) << 24;

            // Write low 32 bits
            write_u32(self.base + apic::IOAPIC_ADDRESS, index as u32);
            write_u32(self.base + apic::IOAPIC_DATA, low);

            // Write high 32 bits
            write_u32(self.base + apic::IOAPIC_ADDRESS, (index + 1) as u32);
            write_u32(self.base + apic::IOAPIC_DATA, high);
        }
    }

    /// Mask IRQ
    pub fn mask_irq(&self, irq: u8, mask: bool) {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let index = 0x10 + (irq as u64) * 2;

            // Read current value
            write_u32(self.base + apic::IOAPIC_ADDRESS, index as u32);
            let mut low = read_u32(self.base + apic::IOAPIC_DATA);

            // Set or clear mask bit
            if mask {
                low |= (1 << 16);
            } else {
                low &= !(1 << 16);
            }

            // Write back
            write_u32(self.base + apic::IOAPIC_ADDRESS, index as u32);
            write_u32(self.base + apic::IOAPIC_DATA, low);
        }
    }
}

/// APIC controller
pub struct ApicController {
    /// Local APIC
    pub local: LocalApic,
    /// I/O APIC list
    pub io_apics: [Option<IoApic>; 4],
    /// Current mode
    pub mode: ApicMode,
}

impl ApicController {
    pub fn new() -> Self {
        ApicController {
            local: LocalApic::new(),
            io_apics: [None; 4],
            mode: ApicMode::Disabled,
        }
    }

    /// Initialize
    pub fn init(&mut self) {
        log_info!("APIC controller initializing...");

        // Initialize Local APIC
        self.local.init();

        // Initialize I/O APIC
        self.io_apics[0] = Some(IoApic::new(apic::IO_APIC_BASE));
        if let Some(ref mut ioapic) = self.io_apics[0] {
            ioapic.init();
        }

        self.mode = ApicMode::Apic;

        log_info!("APIC controller initialized");
    }

    /// Send EOI
    pub fn send_eoi(&self) {
        self.local.send_eoi();
    }

    /// Mask IRQ
    pub fn mask_irq(&self, irq: u8) {
        if let Some(ref ioapic) = self.io_apics[0] {
            ioapic.mask_irq(irq, true);
        }
    }

    /// Unmask IRQ
    pub fn unmask_irq(&self, irq: u8) {
        if let Some(ref ioapic) = self.io_apics[0] {
            ioapic.mask_irq(irq, false);
        }
    }
}

/// Global APIC controller
static mut APIC_CONTROLLER: Option<ApicController> = None;

pub fn get_apic() -> &'static mut ApicController {
    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe {
        if APIC_CONTROLLER.is_none() {
            APIC_CONTROLLER = Some(ApicController::new());
        }
        APIC_CONTROLLER.as_mut().unwrap()
    }
}

pub fn init_apic() {
    let apic = get_apic();
    apic.init();
}

/// Read 32-bit MMIO register
#[inline]
pub unsafe fn read_u32(addr: u64) -> u32 {
    let ptr = addr as *const u32;
    ptr.read_volatile()
}

/// Write 32-bit MMIO register
#[inline]
pub unsafe fn write_u32(addr: u64, value: u32) {
    let ptr = addr as *mut u32;
    ptr.write_volatile(value);
}

/// Read 64-bit MMIO register
#[inline]
pub unsafe fn read_u64(addr: u64) -> u64 {
    let ptr = addr as *const u64;
    ptr.read_volatile()
}

/// Write 64-bit MMIO register
#[inline]
pub unsafe fn write_u64(addr: u64, value: u64) {
    let ptr = addr as *mut u64;
    ptr.write_volatile(value);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apic_mode() {
        assert_eq!(ApicMode::Disabled as i32, 0);
        assert_eq!(ApicMode::Pic as i32, 1);
        assert_eq!(ApicMode::Apic as i32, 2);
        assert_eq!(ApicMode::X2Apic as i32, 3);
    }

    #[test]
    fn test_local_apic() {
        let apic = LocalApic::new();
        assert_eq!(apic.id, 0);
        assert_eq!(apic.version, 0);
        assert!(!apic.enabled);
    }

    #[test]
    fn test_io_apic() {
        let ioapic = IoApic::new(0xFEC0_0000);
        assert_eq!(ioapic.id, 0);
        assert_eq!(ioapic.version, 0);
        assert_eq!(ioapic.num_inputs, 24);
    }

    #[test]
    fn test_apic_controller() {
        let controller = ApicController::new();
        assert_eq!(controller.mode, ApicMode::Disabled);
    }

    #[test]
    fn test_interrupt_vectors() {
        assert_eq!(vectors::DIVIDE_ERROR, 0);
        assert_eq!(vectors::DEBUG, 1);
        assert_eq!(vectors::NMI, 2);
        assert_eq!(vectors::BREAKPOINT, 3);
        assert_eq!(vectors::TIMER, 32);
        assert_eq!(vectors::KEYBOARD, 33);
        assert_eq!(vectors::SYSCALL, 0x80);
    }
}
