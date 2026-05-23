/*
 * Nuva OS - Kernel - ARM64 GIC (Generic Interrupt Controller)
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
use crate::{pr_info};

/// GIC version enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GicVersion {
    /// GIC version 2
    V2 = 2,
    /// GIC version 3
    V3 = 3,
    /// GIC version 4
    V4 = 4,
}

/// GIC Distributor register offsets
pub mod gicd {
    /// Control Register
    pub const CTLR: usize = 0x0000;
    /// Interrupt Group Register
    pub const IGROUPR: usize = 0x0080;
    /// Interrupt Set-Enable Register
    pub const ISENABLER: usize = 0x0100;
    /// Interrupt Clear-Enable Register
    pub const ICENABLER: usize = 0x0180;
    /// Interrupt Set-Pending Register
    pub const ISPENDR: usize = 0x0200;
    /// Interrupt Clear-Pending Register
    pub const ICPENDR: usize = 0x0280;
    /// Interrupt Priority Register
    pub const IPRIORITYR: usize = 0x0400;
    /// Interrupt Processor Targets Register
    pub const ITARGETSR: usize = 0x0800;
    /// Interrupt Configuration Register
    pub const ICFGR: usize = 0x0C00;
    /// Software Generated Interrupt Register
    pub const SGIR: usize = 0x0F00;
}

/// GIC CPU Interface register offsets
pub mod gicc {
    /// Control Register
    pub const CTLR: usize = 0x0000;
    /// Interrupt Priority Mask Register
    pub const PMR: usize = 0x0004;
    /// Binary Point Register
    pub const BPR: usize = 0x0008;
    /// Interrupt Acknowledge Register
    pub const IAR: usize = 0x000C;
    /// End of Interrupt Register
    pub const EOIR: usize = 0x0010;
    /// Running Priority Register
    pub const RPR: usize = 0x0014;
    /// Highest Priority Pending Interrupt Register
    pub const HPPIR: usize = 0x0018;
}

/// GIC Redistributor register offsets (GICv3)
pub mod gicr {
    /// Control Register
    pub const CTLR: usize = 0x0000;
    /// Power Management Control Register
    pub const PMR: usize = 0x0004;
    /// Interrupt Set-Enable Register
    pub const ISENABLER: usize = 0x0100;
    /// Interrupt Clear-Enable Register
    pub const ICENABLER: usize = 0x0180;
    /// Interrupt Priority Register
    pub const IPRIORITYR: usize = 0x0400;
}

/// Interrupt ID type
pub type IrqId = u32;

/// Special interrupt IDs
pub mod irq_id {
    use super::IrqId;
    pub const SPURIOUS: IrqId = 1023;
    pub const SGI_BASE: IrqId = 0;
    pub const SGI_MAX: IrqId = 15;
    pub const PPI_BASE: IrqId = 16;
    pub const PPI_MAX: IrqId = 31;
    pub const SPI_BASE: IrqId = 32;
}

/// GIC Controller
/// Manages the Generic Interrupt Controller for ARM platforms.
pub struct Gic {
    /// GIC version
    pub version: GicVersion,
    /// Distributor base address
    pub gicd_base: usize,
    /// CPU Interface base address
    pub gicc_base: usize,
    /// Redistributor base address (GICv3)
    pub gicr_base: usize,
    /// Maximum interrupt ID
    pub max_irq: IrqId,
}

impl Gic {
    /// Create a new GIC instance
    /// @param version: GIC version
    /// @param gicd_base: Distributor base address
    /// @param gicc_base: CPU Interface base address
    /// @return New Gic instance
    pub fn new(version: GicVersion, gicd_base: usize, gicc_base: usize) -> Self {
        Gic {
            version,
            gicd_base,
            gicc_base,
            gicr_base: 0,
            max_irq: 1020,
        }
    }

    /// Initialize the GIC
    pub fn init(&mut self) {
        log_info!("GIC initialized");
        log_info!("  Version: {:?}", self.version);
        log_info!("  Max IRQ: {}", self.max_irq);

        // Disable all interrupts
        self.disable_all();

        // Set all interrupts to group 0 (secure)
        self.set_all_group0();

        // Set default priority
        self.set_all_priority(0xA0);

        // Enable Distributor
        self.enable_distributor();

        // Enable CPU Interface
        self.enable_cpuif();
    }

    /// Enable an interrupt
    /// @param irq: Interrupt ID to enable
    pub fn enable_irq(&self, irq: IrqId) {
        let reg = self.gicd_base + gicd::ISENABLER + ((irq / 32) as usize) * 4;
        let bit = 1 << (irq % 32);

        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            core::ptr::write_volatile(reg as *mut u32, bit);
        }
    }

    /// Disable an interrupt
    /// @param irq: Interrupt ID to disable
    pub fn disable_irq(&self, irq: IrqId) {
        let reg = self.gicd_base + gicd::ICENABLER + ((irq / 32) as usize) * 4;
        let bit = 1 << (irq % 32);

        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            core::ptr::write_volatile(reg as *mut u32, bit);
        }
    }

    /// Set interrupt priority
    /// @param irq: Interrupt ID
    /// @param priority: Priority value (lower = higher priority)
    pub fn set_priority(&self, irq: IrqId, priority: u8) {
        let reg = self.gicd_base + gicd::IPRIORITYR + (irq as usize);

        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            core::ptr::write_volatile(reg as *mut u8, priority);
        }
    }

    /// Set interrupt target CPU
    /// @param irq: Interrupt ID
    /// @param target: Target CPU mask
    pub fn set_target(&self, irq: IrqId, target: u8) {
        let reg = self.gicd_base + gicd::ITARGETSR + (irq as usize);

        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            core::ptr::write_volatile(reg as *mut u8, target);
        }
    }

    /// Set interrupt trigger type
    /// @param irq: Interrupt ID
    /// @param edge: true for edge-triggered, false for level-triggered
    pub fn set_trigger(&self, irq: IrqId, edge: bool) {
        let reg = self.gicd_base + gicd::ICFGR + ((irq / 16) as usize) * 4;
        let shift = (irq % 16) * 2;

        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let mut val = core::ptr::read_volatile(reg as *const u32);
            if edge {
                val |= 2 << shift;
            } else {
                val &= !(2 << shift);
            }
            core::ptr::write_volatile(reg as *mut u32, val);
        }
    }

    /// Get pending interrupt ID
    /// @return Interrupt ID, or SPURIOUS if none pending
    pub fn get_irq(&self) -> IrqId {
        let reg = self.gicc_base + gicc::IAR;

        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            core::ptr::read_volatile(reg as *const u32)
        }
    }

    /// Signal end of interrupt handling
    /// @param irq: Interrupt ID to acknowledge
    pub fn end_irq(&self, irq: IrqId) {
        let reg = self.gicc_base + gicc::EOIR;

        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            core::ptr::write_volatile(reg as *mut u32, irq);
        }
    }

    /// Send Software Generated Interrupt
    /// @param irq: SGI ID (0-15)
    /// @param target: Target CPU mask
    pub fn send_sgi(&self, irq: IrqId, target: u8) {
        let reg = self.gicd_base + gicd::SGIR;
        let val = ((target as u32) << 16) | irq;

        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            core::ptr::write_volatile(reg as *mut u32, val);
        }
    }

    /// Disable all interrupts
    fn disable_all(&self) {
        for i in 0..(self.max_irq / 32) {
            let reg = self.gicd_base + gicd::ICENABLER + (i as usize) * 4;
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                core::ptr::write_volatile(reg as *mut u32, 0xFFFFFFFF);
            }
        }
    }

    /// Set all interrupts to group 0
    fn set_all_group0(&self) {
        for i in 0..(self.max_irq / 32) {
            let reg = self.gicd_base + gicd::IGROUPR + (i as usize) * 4;
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                core::ptr::write_volatile(reg as *mut u32, 0);
            }
        }
    }

    /// Set priority for all interrupts
    /// @param priority: Default priority value
    fn set_all_priority(&self, priority: u8) {
        for i in 0..self.max_irq {
            self.set_priority(i, priority);
        }
    }

    /// Enable Distributor
    fn enable_distributor(&self) {
        let reg = self.gicd_base + gicd::CTLR;
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            core::ptr::write_volatile(reg as *mut u32, 1);
        }
    }

    /// Enable CPU Interface
    fn enable_cpuif(&self) {
        // Set priority mask
        let pmr = self.gicc_base + gicc::PMR;
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            core::ptr::write_volatile(pmr as *mut u32, 0xF0);
        }

        // Enable CPU Interface
        let ctlr = self.gicc_base + gicc::CTLR;
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            core::ptr::write_volatile(ctlr as *mut u32, 1);
        }
    }
}

/// Global GIC instance
static mut GIC: Option<Gic> = None;

/// Get reference to global GIC instance
pub fn get_gic() -> Option<&'static mut Gic> {
    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe { GIC.as_mut() }
}

/// Initialize GIC
/// @param version: GIC version
/// @param gicd_base: Distributor base address
/// @param gicc_base: CPU Interface base address
pub fn init_gic(version: GicVersion, gicd_base: usize, gicc_base: usize) {
    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe {
        GIC = Some(Gic::new(version, gicd_base, gicc_base));
        if let Some(ref mut gic) = GIC {
            gic.init();
        }
    }
}
