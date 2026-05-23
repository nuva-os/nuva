/*
 * Nuva OS - Kernel - Drivers
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

use crate::{pr_debug, pr_err, pr_info, pr_warn};
use core::ptr;
use core::sync::atomic::{AtomicUsize, Ordering};

// GIC Distributor base address
const GICD_BASE: u64 = 0x08000000;

// GIC CPU Interface base address
const GICC_BASE: u64 = 0x08010000;

// GIC Redistributor base address (GICv3)
const GICR_BASE: u64 = 0x080A0000;

/// GIC Distributor RegisterOffset
const GICD_CTLR: u64 = 0x000; // Control Register
const GICD_TYPER: u64 = 0x004; // Type Register
const GICD_ISENABLER: u64 = 0x100; // Interrupt Set-Enable Registers
const GICD_ICENABLER: u64 = 0x180; // Interrupt Clear-Enable Registers
const GICD_ISPENDR: u64 = 0x200; // Interrupt Set-Pending Registers
const GICD_ICPENDR: u64 = 0x280; // Interrupt Clear-Pending Registers
const GICD_IPRIORITYR: u64 = 0x400; // Interrupt Priority Registers
const GICD_ITARGETSR: u64 = 0x800; // Interrupt Processor Targets Registers
const GICD_ICFGR: u64 = 0xC00; // Interrupt Configuration Registers

/// GIC CPU Interface RegisterOffset
const GICC_CTLR: u64 = 0x000; // CPU Interface Control Register
const GICC_PMR: u64 = 0x004; // Interrupt Priority Mask Register
const GICC_BPR: u64 = 0x008; // Binary Point Register
const GICC_IAR: u64 = 0x00C; // Interrupt Acknowledge Register
const GICC_EOIR: u64 = 0x010; // End of Interrupt Register
const GICC_RPR: u64 = 0x014; // Running Priority Register
const GICC_HPPIR: u64 = 0x018; // Highest Priority Pending Interrupt Register

// Interrupt count
const MAX_IRQS: usize = 1024;

/// InterruptHandleFunctionType
type IrqHandler = extern "C" fn(irq_num: u32, arg: *mut u8);

/// InterruptDescriptor
#[derive(Clone, Copy)]
struct IrqDescriptor {
    handler: Option<IrqHandler>,
    arg: *mut u8,
    enabled: bool,
}

// GIC Driver structure
pub struct GicDriver {
    gicd_base: u64,
    gicc_base: u64,
    gicr_base: u64,
    irq_count: usize,
    is_gicv3: bool,
}

// Global GIC Driver instance
static mut GIC_DRIVER: Option<GicDriver> = None;

// Interrupt descriptor table
static mut IRQ_DESCRIPTORS: [IrqDescriptor; MAX_IRQS] = [IrqDescriptor {
    handler: None,
    arg: ptr::null_mut(),
    enabled: false,
}; MAX_IRQS];

/// CurrentInterruptCount
static IRQ_COUNT: AtomicUsize = AtomicUsize::new(0);

impl GicDriver {
    // Create new GIC Driver instance
    pub const fn new() -> Self {
        GicDriver {
            gicd_base: GICD_BASE,
            gicc_base: GICC_BASE,
            gicr_base: GICR_BASE,
            irq_count: 0,
            is_gicv3: false,
        }
    }

    /// Initialize GIC
    pub fn init(&mut self) {
        log_info!("Initializing GIC...");

        // Read GICD_TYPER to get interrupt count
        let typer = self.read_gicd(GICD_TYPER);
        self.irq_count = ((typer & 0x1F) as usize + 1) * 32;

        log_info!("GIC: {} IRQs detected", self.irq_count);

        // Detect GIC version
        self.detect_gic_version();

        // Disable all interrupts
        self.disable_all_irqs();

        // Initialize Distributor
        self.init_distributor();

        // Initialize CPU Interface
        self.init_cpu_interface();

        log_info!("GIC initialized successfully");
    }

    // Detect GIC version
    fn detect_gic_version(&mut self) {
        // Simplified implementation: assume GICv2
        // Actually need to read GICD_PIDR2 register
        self.is_gicv3 = false;
        log_info!("GIC version: GICv2");
    }

    /// Initialize Distributor
    fn init_distributor(&mut self) {
        // Set all interrupts to lowest priority
        for i in 0..self.irq_count {
            self.set_irq_priority(i as u32, 0xFF);
        }

        // Set all interrupts to Group 0 (IRQ)
        // GICv2: Group 0 = Secure IRQ, Group 1 = Non-secure IRQ

        // Enable distributor
        self.write_gicd(GICD_CTLR, 0x1);
    }

    /// Initialize CPU Interface
    fn init_cpu_interface(&mut self) {
        // Set priority mask (accept all priorities)
        self.write_gicc(GICC_PMR, 0xFF);

        // Set binary point
        self.write_gicc(GICC_BPR, 0x0);

        // Enable CPU interface
        self.write_gicc(GICC_CTLR, 0x1);
    }

    // Disable all interrupts
    fn disable_all_irqs(&mut self) {
        for i in (0..self.irq_count).step_by(32) {
            let reg_offset = GICD_ICENABLER + (i as u64 / 32) * 4;
            self.write_gicd(reg_offset, 0xFFFFFFFF);
        }
    }

    /// RegisterInterruptHandleFunction
    pub fn register_irq(&mut self, irq: u32, handler: IrqHandler, arg: *mut u8) -> bool {
        if irq as usize >= self.irq_count {
            log_error!("Invalid IRQ number: {}", irq);
            return false;
        }

        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            IRQ_DESCRIPTORS[irq as usize] = IrqDescriptor {
                handler: Some(handler),
                arg,
                enabled: false,
            };
        }

        log_info!("IRQ {} registered", irq);
        true
    }

    // Enable interrupt
    pub fn enable_irq(&mut self, irq: u32) {
        if irq as usize >= self.irq_count {
            return;
        }

        // Set interrupt to enabled state
        let reg_offset = GICD_ISENABLER + (irq as u64 / 32) * 4;
        let bit = 1 << (irq % 32);
        self.write_gicd(reg_offset, bit);

        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            if let Some(desc) = IRQ_DESCRIPTORS.get_mut(irq as usize) {
                desc.enabled = true;
            }
        }

        log_debug!("IRQ {} enabled", irq);
    }

    /// DisableInterrupt
    pub fn disable_irq(&mut self, irq: u32) {
        if irq as usize >= self.irq_count {
            return;
        }

        let reg_offset = GICD_ICENABLER + (irq as u64 / 32) * 4;
        let bit = 1 << (irq % 32);
        self.write_gicd(reg_offset, bit);

        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            if let Some(desc) = IRQ_DESCRIPTORS.get_mut(irq as usize) {
                desc.enabled = false;
            }
        }

        log_debug!("IRQ {} disabled", irq);
    }

    /// SetInterruptPriority
    pub fn set_irq_priority(&mut self, irq: u32, priority: u8) {
        let reg_offset = GICD_IPRIORITYR + (irq as u64 / 4) * 4;
        let shift = (irq % 4) * 8;

        let mut value = self.read_gicd(reg_offset);
        value &= !(0xFF << shift);
        value |= (priority as u64) << shift;

        self.write_gicd(reg_offset, value);
    }

    /// SetInterrupttarget CPU
    pub fn set_irq_target(&mut self, irq: u32, cpu: u8) {
        if irq < 32 {
            // SGI and PPI are per-CPU private
            return;
        }

        let reg_offset = GICD_ITARGETSR + (irq as u64 / 4) * 4;
        let shift = (irq % 4) * 8;

        let mut value = self.read_gicd(reg_offset);
        value &= !(0xFF << shift);
        value |= ((1 << cpu) as u64) << shift;

        self.write_gicd(reg_offset, value);
    }

    // Set interrupt trigger type
    pub fn set_irq_type(&mut self, irq: u32, is_edge: bool) {
        let reg_offset = GICD_ICFGR + (irq as u64 / 16) * 4;
        let shift = (irq % 16) * 2;

        let mut value = self.read_gicd(reg_offset);
        value &= !(0x3 << shift);

        if is_edge {
            value |= 0x2 << shift; // Edge-triggered
        } else {
            value |= 0x0 << shift; // Level-sensitive
        }

        self.write_gicd(reg_offset, value);
    }

    // Acknowledge interrupt
    pub fn acknowledge_irq(&mut self) -> u32 {
        let iar = self.read_gicc(GICC_IAR);
        let irq = (iar & 0x3FF) as u32;

        IRQ_COUNT.fetch_add(1, Ordering::Relaxed);

        irq
    }

    /// EndInterrupt
    pub fn end_of_interrupt(&mut self, irq: u32) {
        self.write_gicc(GICC_EOIR, irq as u64);
    }

    /// Read GICD Register
    fn read_gicd(&self, offset: u64) -> u64 {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe { ptr::read_volatile((self.gicd_base + offset) as *const u32) as u64 }
    }

    /// Write GICD Register
    fn write_gicd(&self, offset: u64, value: u64) {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            ptr::write_volatile((self.gicd_base + offset) as *mut u32, value as u32);
        }
    }

    /// Read GICC Register
    fn read_gicc(&self, offset: u64) -> u64 {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe { ptr::read_volatile((self.gicc_base + offset) as *const u32) as u64 }
    }

    /// Write GICC Register
    fn write_gicc(&self, offset: u64, value: u64) {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            ptr::write_volatile((self.gicc_base + offset) as *mut u32, value as u32);
        }
    }
}

/// Initialize GIC
pub fn init_gic() {
    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe {
        let mut gic = GicDriver::new();
        gic.init();
        GIC_DRIVER = Some(gic);
    }
}

/// Get GIC DriverInstance
pub fn get_gic() -> Option<&'static mut GicDriver> {
    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe { GIC_DRIVER.as_mut() }
}

/// Handle IRQ
pub fn handle_irq() {
    let gic = match get_gic() {
        Some(g) => g,
        None => {
            log_error!("GIC not initialized");
            return;
        }
    };

    // Acknowledge interrupt
    let irq = gic.acknowledge_irq();

    if irq >= 1020 {
        // Special interrupt numbers (1020-1023)
        log_debug!("Spurious IRQ: {}", irq);
        return;
    }

    // Call interrupt handler
    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe {
        if let Some(desc) = IRQ_DESCRIPTORS.get(irq as usize) {
            if let Some(handler) = desc.handler {
                handler(irq, desc.arg);
            } else {
                log_warn!("Unhandled IRQ: {}", irq);
            }
        }
    }

    // EndInterrupt
    gic.end_of_interrupt(irq);
}

/// RegisterInterruptHandleFunction
pub fn register_irq(irq: u32, handler: IrqHandler, arg: *mut u8) -> bool {
    match get_gic() {
        Some(gic) => gic.register_irq(irq, handler, arg),
        None => false,
    }
}

// Enable interrupt
pub fn enable_irq(irq: u32) {
    if let Some(gic) = get_gic() {
        gic.enable_irq(irq);
    }
}

/// DisableInterrupt
pub fn disable_irq(irq: u32) {
    if let Some(gic) = get_gic() {
        gic.disable_irq(irq);
    }
}

/// GetInterruptCount
pub fn get_irq_count() -> usize {
    IRQ_COUNT.load(Ordering::Relaxed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gic_init() {
        init_gic();
        assert!(get_gic().is_some());
    }
}
