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


use core::arch::asm;
use super::exception;
use super::gdt;

/// Interrupt vector number
pub mod vector {
    pub const DIVIDE_ERROR: u8 = 0;
    pub const DEBUG: u8 = 1;
    pub const NMI: u8 = 2;
    pub const BREAKPOINT: u8 = 3;
    pub const OVERFLOW: u8 = 4;
    pub const BOUND_RANGE: u8 = 5;
    pub const INVALID_OPCODE: u8 = 6;
    pub const DEVICE_NOT_AVAILABLE: u8 = 7;
    pub const DOUBLE_FAULT: u8 = 8;
    pub const COPROCESSOR_SEGMENT: u8 = 9;
    pub const INVALID_TSS: u8 = 10;
    pub const SEGMENT_NOT_PRESENT: u8 = 11;
    pub const STACK_SEGMENT: u8 = 12;
    pub const GENERAL_PROTECTION: u8 = 13;
    pub const PAGE_FAULT: u8 = 14;
    pub const RESERVED: u8 = 15;
    pub const FPU_ERROR: u8 = 16;
    pub const ALIGNMENT_CHECK: u8 = 17;
    pub const MACHINE_CHECK: u8 = 18;
    pub const SIMD_EXCEPTION: u8 = 19;
    pub const VIRTUALIZATION: u8 = 20;

    // User-defined vectors
    pub const TIMER: u8 = 32;
    pub const KEYBOARD: u8 = 33;
    pub const CASCADE: u8 = 34;
    pub const COM1: u8 = 35;
    pub const COM2: u8 = 36;
    pub const FLOPPY: u8 = 37;
    pub const PARALLEL: u8 = 38;
    pub const RTC: u8 = 39;
    pub const ACPI: u8 = 40;
    pub const SYSCALL: u8 = 0x80;
}

/// Interrupt gate type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GateType {
    /// Interrupt gate
    Interrupt = 0xE,
    /// Trap gate
    Trap = 0xF,
}

/// IDT descriptor
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct IdtEntry {
    /// Low 16 bits of offset
    offset_low: u16,
    /// Segment selector
    selector: u16,
    /// Reserved
    ist: u8,
    /// Type and attributes
    flags: u8,
    /// Middle 16 bits of offset
    offset_mid: u16,
    /// High 32 bits of offset
    offset_high: u32,
    /// Reserved
    reserved: u32,
}

impl IdtEntry {
    /// Create null descriptor
    pub const fn new() -> Self {
        IdtEntry {
            offset_low: 0,
            selector: 0,
            ist: 0,
            flags: 0,
            offset_mid: 0,
            offset_high: 0,
            reserved: 0,
        }
    }

    /// Create interrupt gate
    pub fn create_interrupt(handler: u64, selector: u16, dpl: u8) -> Self {
        IdtEntry {
            offset_low: handler as u16,
            selector,
            ist: 0,
            flags: 0x80 | ((dpl & 3) << 5) | (GateType::Interrupt as u8),
            offset_mid: (handler >> 16) as u16,
            offset_high: (handler >> 32) as u32,
            reserved: 0,
        }
    }

    /// Create trap gate
    pub fn create_trap(handler: u64, selector: u16, dpl: u8) -> Self {
        IdtEntry {
            offset_low: handler as u16,
            selector,
            ist: 0,
            flags: 0x80 | ((dpl & 3) << 5) | (GateType::Trap as u8),
            offset_mid: (handler >> 16) as u16,
            offset_high: (handler >> 32) as u32,
            reserved: 0,
        }
    }
}

/// IDT pointer
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct IdtPointer {
    /// Limit
    pub limit: u16,
    /// Base address
    pub base: u64,
}

/// IDT
pub struct Idt {
    /// Descriptor table
    pub entries: [IdtEntry; 256],
}

impl Idt {
    /// Create new IDT
    pub const fn new() -> Self {
        Idt {
            entries: [IdtEntry::new(); 256],
        }
    }

    /// Set interrupt gate
    pub fn set_interrupt(&mut self, vector: u8, handler: u64, selector: u16, dpl: u8) {
        self.entries[vector as usize] = IdtEntry::create_interrupt(handler, selector, dpl);
    }

    /// Set trap gate
    pub fn set_trap(&mut self, vector: u8, handler: u64, selector: u16, dpl: u8) {
        self.entries[vector as usize] = IdtEntry::create_trap(handler, selector, dpl);
    }

    /// Get IDT pointer
    pub fn get_pointer(&self) -> IdtPointer {
        IdtPointer {
            limit: (core::mem::size_of::<Idt>() - 1) as u16,
            base: self as *const _ as u64,
        }
    }

    /// Load IDT
    pub fn load(&self) {
        let ptr = self.get_pointer();
        // SAFETY: inline assembly required for hardware instruction
        unsafe {
            asm!(
                "lidt [{}]",
                in(reg) &ptr,
            );
        }
    }
}

/// Global IDT
static IDT: core::sync::OnceLock<Idt> = core::sync::OnceLock::new();

/// Get IDT
pub fn idt() -> &'static Idt {
    IDT.get_or_init(Idt::new)
}

/// Initialize IDT
pub fn init_idt() {
    let idt = get_idt();
    let sel = gdt::KERNEL_CODE;

    macro_rules! set_handler {
        ($vec:expr, $handler:expr) => {
            idt.set_interrupt($vec, $handler as u64, sel, 0);
        };
    }

    set_handler!(0, exception::divide_error);
    set_handler!(1, exception::debug);
    set_handler!(2, exception::nmi);
    set_handler!(3, exception::breakpoint);
    set_handler!(4, exception::overflow);
    set_handler!(5, exception::bound_range);
    set_handler!(6, exception::invalid_opcode);
    set_handler!(7, exception::device_not_available);
    set_handler!(8, exception::double_fault);
    set_handler!(10, exception::invalid_tss);
    set_handler!(11, exception::segment_not_present);
    set_handler!(12, exception::stack_segment_fault);
    set_handler!(13, exception::general_protection);
    set_handler!(14, exception::page_fault);
    set_handler!(16, exception::x87_fpu_error);
    set_handler!(17, exception::alignment_check);
    set_handler!(18, exception::machine_check);
    set_handler!(19, exception::simd_exception);
    set_handler!(20, exception::virtualization_exception);

    for vec in 32..=255u8 {
        idt.set_interrupt(vec, exception::generic_interrupt_handler as u64, sel, 0);
    }

    idt.load();

    log_info!("IDT initialized");
}
