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


/// Local APIC Register
pub mod lapic {
    pub const ID: usize = 0x020;
    pub const VERSION: usize = 0x030;
    pub const TPR: usize = 0x080;
    pub const APR: usize = 0x090;
    pub const PPR: usize = 0x0A0;
    pub const EOI: usize = 0x0B0;
    pub const LDR: usize = 0x0D0;
    pub const DFR: usize = 0x0E0;
    pub const SVR: usize = 0x0F0;
    pub const ISR: usize = 0x100;
    pub const TMR: usize = 0x180;
    pub const IRR: usize = 0x200;
    pub const ESR: usize = 0x280;
    pub const ICR_LOW: usize = 0x300;
    pub const ICR_HIGH: usize = 0x310;
    pub const TIMER_LVT: usize = 0x320;
    pub const THERMAL_LVT: usize = 0x330;
    pub const PMC_LVT: usize = 0x340;
    pub const LINT0_LVT: usize = 0x350;
    pub const LINT1_LVT: usize = 0x360;
    pub const ERROR_LVT: usize = 0x370;
    pub const TIMER_ICR: usize = 0x380;
    pub const TIMER_CCR: usize = 0x390;
    pub const TIMER_DCR: usize = 0x3E0;
}

/// I/O APIC Register
pub mod ioapic {
    pub const IND: usize = 0x00;
    pub const DAT: usize = 0x10;
    pub const IRQ_PIN: usize = 0x20;
}

/// LAPIC MMIO register offsets (u32)
pub const LAPIC_ID: u32 = 0x020;
pub const LAPIC_VERSION: u32 = 0x030;
pub const LAPIC_TPR: u32 = 0x080;
pub const LAPIC_EOI: u32 = 0x0B0;
pub const LAPIC_SVR: u32 = 0x0F0;
pub const LAPIC_TIMER_LVT: u32 = 0x100;
pub const LAPIC_TIMER_ICR: u32 = 0x380;
pub const LAPIC_TIMER_CCR: u32 = 0x390;
pub const LAPIC_TIMER_DCR: u32 = 0x3E0;
pub const LAPIC_ICR_LOW: u32 = 0x300;
pub const LAPIC_ICR_HIGH: u32 = 0x310;
pub const LAPIC_ERROR: u32 = 0x280;

/// I/O APIC register constants
pub const IOAPIC_REG_INDEX: u32 = 0x00;
pub const IOAPIC_REG_DATA: u32 = 0x10;
pub const IOAPIC_REG_ID: u32 = 0x00;
pub const IOAPIC_REG_VERSION: u32 = 0x01;
pub const IOAPIC_REG_REDIR_BASE: u32 = 0x10;

/// Read LAPIC MMIO register
#[inline]
pub fn read_lapic(base: u64, reg: u32) -> u32 {
    // SAFETY: Reading from a valid MMIO address; caller guarantees base is a mapped LAPIC base.
    unsafe { core::ptr::read_volatile((base + reg as u64) as *const u32) }
}

/// Write LAPIC MMIO register
#[inline]
pub fn write_lapic(base: u64, reg: u32, val: u32) {
    // SAFETY: Writing to a valid MMIO address; caller guarantees base is a mapped LAPIC base.
    unsafe { core::ptr::write_volatile((base + reg as u64) as *mut u32, val) }
}

/// Read I/O APIC register (indirect access via index/data pair)
#[inline]
pub fn read_ioapic(base: u64, reg: u32) -> u32 {
    // SAFETY: Writing index register then reading data register; caller guarantees base is a mapped I/O APIC base.
    unsafe {
        core::ptr::write_volatile((base + IOAPIC_REG_INDEX as u64) as *mut u32, reg);
        core::ptr::read_volatile((base + IOAPIC_REG_DATA as u64) as *const u32)
    }
}

/// Write I/O APIC register (indirect access via index/data pair)
#[inline]
pub fn write_ioapic(base: u64, reg: u32, val: u32) {
    // SAFETY: Writing index register then data register; caller guarantees base is a mapped I/O APIC base.
    unsafe {
        core::ptr::write_volatile((base + IOAPIC_REG_INDEX as u64) as *mut u32, reg);
        core::ptr::write_volatile((base + IOAPIC_REG_DATA as u64) as *mut u32, val);
    }
}

/// APIC ID
pub type ApicId = u32;

/// Local APIC
pub struct LocalApic {
    /// Base address
    pub base: usize,
    /// APIC ID
    pub id: ApicId,
}

impl LocalApic {
    /// Create Local APIC instance
    pub fn new(base: usize) -> Self {
        LocalApic {
            base,
            id: 0,
        }
    }

    /// Initialize
    pub fn init(&mut self) {
        // Get APIC ID
        self.id = self.read(lapic::ID) >> 24;

        // Enable APIC
        let svr = self.read(lapic::SVR);
        self.write(lapic::SVR, svr | 0x100);  // Set enable bit

        // Set task priority
        self.write(lapic::TPR, 0);

        // Set error handler
        self.write(lapic::ERROR_LVT, 1 << 16);  // Mask

        log_info!("Local APIC initialized");
        log_info!("  ID: {}", self.id);
    }

    /// Read register
    pub fn read(&self, reg: usize) -> u32 {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            core::ptr::read_volatile((self.base + reg) as *const u32)
        }
    }

    /// Write register
    pub fn write(&self, reg: usize, val: u32) {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            core::ptr::write_volatile((self.base + reg) as *mut u32, val);
        }
    }

    /// Send EOI
    pub fn eoi(&self) {
        self.write(lapic::EOI, 0);
    }

    /// Get APIC ID
    pub fn get_id(&self) -> ApicId {
        self.id
    }

    /// Set timer
    pub fn set_timer(&self, vector: u8, initial: u32, divide: u32) {
        self.write(lapic::TIMER_LVT, vector as u32);
        self.write(lapic::TIMER_DCR, divide);
        self.write(lapic::TIMER_ICR, initial);
    }

    /// Send IPI (Inter-Processor Interrupt)
    pub fn send_ipi(&self, dest: ApicId, vector: u8) {
        self.write(lapic::ICR_HIGH, dest << 24);
        self.write(lapic::ICR_LOW, vector as u32);
    }

    /// Broadcast IPI
    pub fn broadcast_ipi(&self, vector: u8) {
        self.write(lapic::ICR_HIGH, 0xFF << 24);
        self.write(lapic::ICR_LOW, vector as u32 | 0x000C4000);
    }
}

/// I/O APIC
pub struct IoApic {
    /// Base address
    pub base: usize,
    /// Max interrupt count
    pub max_irq: u32,
}

impl IoApic {
    /// Create I/O APIC instance
    pub fn new(base: usize) -> Self {
        IoApic {
            base,
            max_irq: 24,
        }
    }

    /// Initialize
    pub fn init(&mut self) {
        // Get max interrupt count
        self.write(ioapic::IND, 1);
        self.max_irq = (self.read(ioapic::DAT) >> 16) + 1;

        // Mask all interrupts
        for i in 0..self.max_irq {
            self.set_irq_mask(i, true);
        }

        log_info!("I/O APIC initialized");
        log_info!("  Max IRQ: {}", self.max_irq);
    }

    /// Read register
    pub fn read(&self, reg: usize) -> u32 {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            core::ptr::write_volatile((self.base + ioapic::IND) as *mut u32, reg as u32);
            core::ptr::read_volatile((self.base + ioapic::DAT) as *const u32)
        }
    }

    /// Write register
    pub fn write(&self, reg: usize, val: u32) {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            core::ptr::write_volatile((self.base + ioapic::IND) as *mut u32, reg as u32);
            core::ptr::write_volatile((self.base + ioapic::DAT) as *mut u32, val);
        }
    }

    /// Set interrupt redirection
    pub fn set_irq(&self, irq: u32, vector: u8, dest: ApicId, mask: bool) {
        let reg = 0x10 + irq * 2;
        let low = vector as u32 | (0 << 8) | (0 << 11) | ((mask as u32) << 16);
        let high = dest << 24;

        self.write(reg as usize, low);
        self.write((reg + 1) as usize, high);
    }

    /// Mask interrupt
    pub fn set_irq_mask(&self, irq: u32, mask: bool) {
        let reg = 0x10 + irq * 2;
        let mut val = self.read(reg as usize);

        if mask {
            val |= 1 << 16;
        } else {
            val &= !(1 << 16);
        }

        self.write(reg as usize, val);
    }

    /// Enable interrupt
    pub fn enable_irq(&self, irq: u32, vector: u8, dest: ApicId) {
        self.set_irq(irq, vector, dest, false);
    }

    /// Disable interrupt
    pub fn disable_irq(&self, irq: u32) {
        self.set_irq_mask(irq, true);
    }
}

/// Global Local APIC
static mut LOCAL_APIC: Option<LocalApic> = None;

/// Global I/O APIC
static mut IO_APIC: Option<IoApic> = None;

/// Get Local APIC
pub fn get_lapic() -> Option<&'static mut LocalApic> {
    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe { LOCAL_APIC.as_mut() }
}

/// Get I/O APIC
pub fn get_ioapic() -> Option<&'static mut IoApic> {
    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe { IO_APIC.as_mut() }
}

/// Initialize APIC
pub fn init_apic(lapic_base: usize, ioapic_base: usize) {
    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe {
        LOCAL_APIC = Some(LocalApic::new(lapic_base));
        if let Some(ref mut lapic) = LOCAL_APIC {
            lapic.init();
        }

        IO_APIC = Some(IoApic::new(ioapic_base));
        if let Some(ref mut ioapic) = IO_APIC {
            ioapic.init();
        }
    }
}
