/*
 * Nuva OS - Kernel - Arch - x64 - Exception Handlers
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

//! x86_64 CPU exception handlers

use crate::{pr_emerg, pr_info, pr_warn};
use core::arch::asm;

/// Page fault error code bits
#[repr(transparent)]
pub struct PageFaultErrorCode(pub u64);

impl PageFaultErrorCode {
    pub fn present(&self) -> bool {
        self.0 & (1 << 0) != 0
    }
    pub fn write(&self) -> bool {
        self.0 & (1 << 1) != 0
    }
    pub fn user(&self) -> bool {
        self.0 & (1 << 2) != 0
    }
    pub fn reserved_write(&self) -> bool {
        self.0 & (1 << 3) != 0
    }
    pub fn instruction_fetch(&self) -> bool {
        self.0 & (1 << 4) != 0
    }
}

/// Interrupt stack frame (pushed by CPU on exception)
#[repr(C)]
pub struct InterruptStackFrame {
    pub instruction_pointer: u64,
    pub code_segment: u64,
    pub cpu_flags: u64,
    pub stack_pointer: u64,
    pub stack_segment: u64,
}

/// Read CR2 (page fault linear address)
fn read_cr2() -> u64 {
    let val: u64;
    // SAFETY: mov cr2 reads the CR2 control register which holds the
    // page fault linear address. This is a read-only CPU register access.
    unsafe {
        asm!("mov {}, cr2", out(reg) val);
    }
    // SAFETY: CR2 is a readable CPU register
    val
}

/// #DE - Divide Error
pub extern "x86-interrupt" fn divide_error(_frame: &mut InterruptStackFrame) {
    log_emerg!("#DE: Divide Error");
}

/// #DB - Debug Exception
pub extern "x86-interrupt" fn debug(_frame: &mut InterruptStackFrame) {
    log_warn!("#DB: Debug Exception");
}

/// #NMI - Non-Maskable Interrupt
pub extern "x86-interrupt" fn nmi(_frame: &mut InterruptStackFrame) {
    log_warn!("#NMI: Non-Maskable Interrupt");
}

/// #BP - Breakpoint
pub extern "x86-interrupt" fn breakpoint(_frame: &mut InterruptStackFrame) {
    log_info!("#BP: Breakpoint at {:#x}", _frame.instruction_pointer);
}

/// #OF - Overflow
pub extern "x86-interrupt" fn overflow(_frame: &mut InterruptStackFrame) {
    log_emerg!("#OF: Overflow");
}

/// #BR - BOUND Range Exceeded
pub extern "x86-interrupt" fn bound_range(_frame: &mut InterruptStackFrame) {
    log_emerg!("#BR: Bound Range Exceeded");
}

/// #UD - Invalid Opcode
pub extern "x86-interrupt" fn invalid_opcode(_frame: &mut InterruptStackFrame) {
    log_emerg!("#UD: Invalid Opcode at {:#x}", _frame.instruction_pointer);
}

/// #NM - Device Not Available
pub extern "x86-interrupt" fn device_not_available(_frame: &mut InterruptStackFrame) {
    log_emerg!("#NM: Device Not Available");
}

/// #DF - Double Fault
pub extern "x86-interrupt" fn double_fault(
    _frame: &mut InterruptStackFrame,
    _error_code: u64,
) -> ! {
    log_emerg!("#DF: Double Fault (error_code={:#x})", _error_code);
    loop {
        core::hint::spin_loop();
    }
}

/// #TS - Invalid TSS
pub extern "x86-interrupt" fn invalid_tss(_frame: &mut InterruptStackFrame, error_code: u64) {
    log_emerg!("#TS: Invalid TSS (error_code={:#x})", error_code);
}

/// #NP - Segment Not Present
pub extern "x86-interrupt" fn segment_not_present(
    _frame: &mut InterruptStackFrame,
    error_code: u64,
) {
    log_emerg!("#NP: Segment Not Present (error_code={:#x})", error_code);
}

/// #SS - Stack Segment Fault
pub extern "x86-interrupt" fn stack_segment_fault(
    _frame: &mut InterruptStackFrame,
    error_code: u64,
) {
    log_emerg!("#SS: Stack Segment Fault (error_code={:#x})", error_code);
}

/// #GP - General Protection
pub extern "x86-interrupt" fn general_protection(
    _frame: &mut InterruptStackFrame,
    error_code: u64,
) {
    log_emerg!(
        "#GP: General Protection Fault at {:#x} (error_code={:#x})",
        _frame.instruction_pointer,
        error_code
    );
}

/// #PF - Page Fault
pub extern "x86-interrupt" fn page_fault(
    _frame: &mut InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    let fault_addr = read_cr2();
    // SAFETY: CR2 contains the linear address that caused the page fault per x86-64 spec
    let access_type = if error_code.write() { "write" } else { "read" };
    let privilege = if error_code.user() { "user" } else { "kernel" };
    log_emerg!(
        "#PF: Page Fault at {:#x} ({}) ({})",
        fault_addr,
        access_type,
        privilege
    );

    // TODO: Dispatch to page fault handler for demand paging / COW
}

/// #MF - x87 FPU Error
pub extern "x86-interrupt" fn x87_fpu_error(_frame: &mut InterruptStackFrame) {
    log_emerg!("#MF: x87 FPU Error");
}

/// #AC - Alignment Check
pub extern "x86-interrupt" fn alignment_check(_frame: &mut InterruptStackFrame, _error_code: u64) {
    log_emerg!("#AC: Alignment Check");
}

/// #MC - Machine Check
pub extern "x86-interrupt" fn machine_check(_frame: &mut InterruptStackFrame) -> ! {
    log_emerg!("#MC: Machine Check");
    loop {
        core::hint::spin_loop();
    }
}

/// #XM - SIMD Exception
pub extern "x86-interrupt" fn simd_exception(_frame: &mut InterruptStackFrame) {
    log_emerg!("#XM: SIMD Exception");
}

/// #VE - Virtualization Exception
pub extern "x86-interrupt" fn virtualization_exception(_frame: &mut InterruptStackFrame) {
    log_emerg!("#VE: Virtualization Exception");
}

/// Generic interrupt handler stub (for IRQ vectors 32-255)
pub extern "x86-interrupt" fn generic_interrupt_handler(_frame: &mut InterruptStackFrame) {
    // Dispatch to APIC / IRQ controller
}
