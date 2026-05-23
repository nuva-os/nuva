/*
* Nuva OS - Kernel - LoongArch64 EIOINTC (Extended I/O Interrupt Controller)
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

use crate::pr_info;
use core::sync::atomic::{AtomicU32, Ordering};

/// EIOINTC version enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EiointcVersion {
    /// EIOINTC version 1.0
    V1 = 1,
    /// EIOINTC version 2.0 (LIOINTC)
    V2 = 2,
}

/// EIOINTC register offsets
pub mod eiointc_reg {
    pub const STATUS: usize = 0x0010;
    pub const ENABLE: usize = 0x0020;
    pub const DISABLE: usize = 0x0028;
    pub const EOI: usize = 0x0040;
    pub const ROUTE: usize = 0x0060;
    pub const CTLR: usize = 0x0000;
    pub const AUTO_EOI: usize = 0x0050;
}

/// Interrupt ID type
pub type IrqId = u32;

/// Special interrupt IDs
pub mod irq_id {
    use super::IrqId;
    pub const SPURIOUS: IrqId = 255;
    pub const SGI_BASE: IrqId = 0;
    pub const SGI_MAX: IrqId = 15;
    pub const IPI_BASE: IrqId = 16;
    pub const IPI_MAX: IrqId = 31;
    pub const EXT_BASE: IrqId = 32;
}

/// EIOINTC Controller
/// Manages the Extended I/O Interrupt Controller for LoongArch platforms.
pub struct Eiointc {
    /// EIOINTC version
    pub version: EiointcVersion,
    /// Base address
    pub base: usize,
    /// Maximum interrupt ID
    pub max_irq: IrqId,
}

impl Eiointc {
    /// Create a new EIOINTC instance
    /// @param version: EIOINTC version
    /// @param base: Base address
    /// @return New Eiointc instance
    pub fn new(version: EiointcVersion, base: usize) -> Self {
        Eiointc {
            version,
            base,
            max_irq: 256,
        }
    }

    /// Initialize the EIOINTC
    pub fn init(&mut self) {
        log_info!("EIOINTC initialized");
        log_info!("  Version: {:?}", self.version);
        log_info!("  Max IRQ: {}", self.max_irq);

        self.disable_all();
        self.enable_controller();
    }

    /// Enable an interrupt
    /// @param irq: Interrupt ID to enable
    pub fn enable_irq(&self, irq: IrqId) {
        let reg = self.base + eiointc_reg::ENABLE;
        let bit = 1u32 << (irq % 32);

        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            core::ptr::write_volatile(reg as *mut u32, bit);
        }
    }

    /// Disable an interrupt
    /// @param irq: Interrupt ID to disable
    pub fn disable_irq(&self, irq: IrqId) {
        let reg = self.base + eiointc_reg::DISABLE;
        let bit = 1u32 << (irq % 32);

        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            core::ptr::write_volatile(reg as *mut u32, bit);
        }
    }

    /// Signal end of interrupt handling
    /// @param irq: Interrupt ID to acknowledge
    pub fn end_irq(&self, irq: IrqId) {
        let reg = self.base + eiointc_reg::EOI;

        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            core::ptr::write_volatile(reg as *mut u32, irq);
        }
    }

    /// Set interrupt target CPU
    /// @param irq: Interrupt ID
    /// @param target: Target CPU mask
    pub fn set_target(&self, irq: IrqId, target: u8) {
        let reg = self.base + eiointc_reg::ROUTE + (irq as usize) * 8;

        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            core::ptr::write_volatile(reg as *mut u8, target);
        }
    }

    /// Disable all interrupts
    fn disable_all(&self) {
        let reg = self.base + eiointc_reg::DISABLE;
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            core::ptr::write_volatile(reg as *mut u32, 0xFFFF_FFFF);
        }
    }

    /// Enable controller
    fn enable_controller(&self) {
        let reg = self.base + eiointc_reg::CTLR;
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            core::ptr::write_volatile(reg as *mut u32, 1);
        }
    }
}

/// Global EIOINTC instance
static mut EIOINTC: Option<Eiointc> = None;

/// Get reference to global EIOINTC instance
pub fn get_eiointc() -> Option<&'static mut Eiointc> {
    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe { EIOINTC.as_mut() }
}

/// Initialize EIOINTC
/// @param version: EIOINTC version
/// @param base: Base address
pub fn init_eiointc(version: EiointcVersion, base: usize) {
    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe {
        EIOINTC = Some(Eiointc::new(version, base));
        if let Some(ref mut eiointc) = EIOINTC {
            eiointc.init();
        }
    }
}
