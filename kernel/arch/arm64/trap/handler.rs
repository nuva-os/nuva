/*
* Nuva OS - Kernel - Arch
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

use crate::{pr_debug, pr_emerg, pr_info, pr_warn};
use core::arch::asm;

/// Exception type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExceptionType {
    /// EL0 synchronous exception
    SyncEL0 = 0,
    /// EL0 IRQ
    IrqEL0 = 1,
    /// EL0 FIQ
    FiqEL0 = 2,
    /// EL0 SError
    SErrorEL0 = 3,
    /// EL1 synchronous exception
    SyncEL1 = 4,
    /// EL1 IRQ
    IrqEL1 = 5,
    /// EL1 FIQ
    FiqEL1 = 6,
    /// EL1 SError
    SErrorEL1 = 7,
}

/// Exception context
#[repr(C)]
pub struct ExceptionContext {
    /// General registers x0-x29
    pub regs: [u64; 30],
    /// Stack pointer
    pub sp: u64,
    /// Exception return address
    pub elr: u64,
    /// Saved program status register
    pub spsr: u64,
    /// Exception syndrome register
    pub esr: u64,
    /// Fault address register
    pub far: u64,
}

impl ExceptionContext {
    /// Create new exception context
    pub const fn new() -> Self {
        ExceptionContext {
            regs: [0; 30],
            sp: 0,
            elr: 0,
            spsr: 0,
            esr: 0,
            far: 0,
        }
    }
}

/// ESR register parser
#[derive(Debug, Clone, Copy)]
pub struct EsrElx {
    pub value: u64,
}

impl EsrElx {
    /// Create from ESR_ELx register
    pub fn from_raw(value: u64) -> Self {
        EsrElx { value }
    }

    /// Get exception class
    pub fn ec(&self) -> u8 {
        ((self.value >> 26) & 0x3F) as u8
    }

    /// Get instruction length (0=32-bit, 1=16-bit)
    pub fn il(&self) -> bool {
        (self.value & (1 << 25)) != 0
    }

    /// Get ISS (Instruction Specific Syndrome)
    pub fn iss(&self) -> u32 {
        (self.value & 0x1FFFFFF) as u32
    }

    /// Check if system call
    pub fn is_syscall(&self) -> bool {
        self.ec() == 0x15 // SVC instruction execution
    }

    /// Check if page fault
    pub fn is_page_fault(&self) -> bool {
        let ec = self.ec();
        ec == 0x20 ||  // Instruction abort from lower EL
        ec == 0x21 ||  // Instruction abort from same EL
        ec == 0x22 ||  // Data abort from lower EL
        ec == 0x23 // Data abort from same EL
    }

    /// Check if breakpoint
    pub fn is_breakpoint(&self) -> bool {
        self.ec() == 0x30 // Breakpoint from lower EL
    }
}

/// Exception handler entry
#[no_mangle]
pub extern "C" fn handle_exception(exc_type: u64, ctx: &mut ExceptionContext) {
    let exc_type = match exc_type {
        0 => ExceptionType::SyncEL0,
        1 => ExceptionType::IrqEL0,
        2 => ExceptionType::FiqEL0,
        3 => ExceptionType::SErrorEL0,
        4 => ExceptionType::SyncEL1,
        5 => ExceptionType::IrqEL1,
        6 => ExceptionType::FiqEL1,
        7 => ExceptionType::SErrorEL1,
        _ => {
            log_emerg!("Unknown exception type: {}", exc_type);
            return;
        }
    };

    // Read ESR and FAR
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!("mrs {}, esr_el1", out(reg) ctx.esr);
        asm!("mrs {}, far_el1", out(reg) ctx.far);
    }

    match exc_type {
        ExceptionType::SyncEL0 | ExceptionType::SyncEL1 => {
            handle_sync_exception(exc_type, ctx);
        }
        ExceptionType::IrqEL0 | ExceptionType::IrqEL1 => {
            handle_irq_exception(exc_type, ctx);
        }
        ExceptionType::FiqEL0 | ExceptionType::FiqEL1 => {
            handle_fiq_exception(exc_type, ctx);
        }
        ExceptionType::SErrorEL0 | ExceptionType::SErrorEL1 => {
            handle_serror_exception(exc_type, ctx);
        }
    }
}

/// Handle synchronous exception
fn handle_sync_exception(exc_type: ExceptionType, ctx: &mut ExceptionContext) {
    let esr = EsrElx::from_raw(ctx.esr);

    if esr.is_syscall() {
        // System call
        handle_syscall(ctx);
    } else if esr.is_page_fault() {
        // Page fault
        handle_page_fault(exc_type, ctx, &esr);
    } else if esr.is_breakpoint() {
        // Breakpoint
        handle_breakpoint(exc_type, ctx);
    } else {
        // Unknown exception
        log_emerg!("Unhandled sync exception");
        log_emerg!("  Type: {:?}", exc_type);
        log_emerg!("  ESR: {:#018x}", ctx.esr);
        log_emerg!("  FAR: {:#018x}", ctx.far);
        log_emerg!("  ELR: {:#018x}", ctx.elr);

        // If user mode exception, terminate process
        if exc_type == ExceptionType::SyncEL0 {
            log_emerg!("Terminating user process");
            // Implementation: Terminate current process via scheduler exit
        } else {
            // Kernel mode exception, panic
            // PANIC: Unrecoverable kernel sync exception. The kernel cannot
            // continue execution after an unhandled synchronous exception in
            // kernel mode. This is analogous to Linux's die().
            panic!("Unhandled kernel sync exception");
        }
    }
}

/// Handle IRQ interrupt
fn handle_irq_exception(_exc_type: ExceptionType, _ctx: &mut ExceptionContext) {
    // Call interrupt controller handler
    // Implementation: Dispatch IRQ through the interrupt controller handler
    crate::kernel::driver::r#impl::irqchip::gic::handle_irq();
}

/// Handle FIQ interrupt
fn handle_fiq_exception(_exc_type: ExceptionType, _ctx: &mut ExceptionContext) {
    // FIQ handler (usually for high priority interrupts)
    log_warn!("FIQ received (not implemented)");
}

/// Handle SError
fn handle_serror_exception(_exc_type: ExceptionType, ctx: &mut ExceptionContext) {
    log_emerg!("SError exception");
    log_emerg!("  ESR: {:#018x}", ctx.esr);
    log_emerg!("  FAR: {:#018x}", ctx.far);
    log_emerg!("  ELR: {:#018x}", ctx.elr);

    // SError is usually a serious hardware error
    // PANIC: Unrecoverable SError (System Error). This indicates a severe
    // hardware error such as an uncorrectable memory error. The kernel
    // cannot safely continue execution.
    panic!("SError exception");
}

/// Handle page fault
fn handle_page_fault(exc_type: ExceptionType, ctx: &mut ExceptionContext, esr: &EsrElx) {
    let iss = esr.iss();

    // Parse ISS
    let is_write = (iss & (1 << 6)) != 0; // WnR bit
    let is_user = exc_type == ExceptionType::SyncEL0;

    log_debug!("Page fault at {:#018x}", ctx.far);
    log_debug!("  Write: {}, User: {}", is_write, is_user);

    // Call page fault handler
    // Implementation: Invoke page fault handler to resolve demand paging or COW
    // if !crate::mm::fault::handle_page_fault(ctx.far, is_write, is_user) {
    //     log_error!("Page fault handling failed");
    //     if is_user {
    //         // Terminate user process
    //     } else {
    //         panic!("Kernel page fault");
    //     }
    // }
}

/// Handle breakpoint
fn handle_breakpoint(_exc_type: ExceptionType, ctx: &mut ExceptionContext) {
    log_info!("Breakpoint hit at {:#018x}", ctx.elr);

    // Skip breakpoint instruction
    // Implementation: Skip breakpoint instruction and invoke debugger if attached
}

/// System call handler
#[no_mangle]
pub extern "C" fn handle_syscall(ctx: &mut ExceptionContext) {
    // System call number in x8
    let syscall_num = ctx.regs[8];

    // Parameters in x0-x5
    let args = [
        ctx.regs[0],
        ctx.regs[1],
        ctx.regs[2],
        ctx.regs[3],
        ctx.regs[4],
        ctx.regs[5],
    ];

    log_debug!("Syscall {} called", syscall_num);

    // Call system call dispatcher
    // Implementation: Dispatch system call via system call number table lookup
    let result = crate::kernel::syscall::dispatch(syscall_num, &args);

    // Return value in x0
    ctx.regs[0] = result as u64;
}

/// Enable interrupt
pub fn enable_irq() {
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!("msr daifclr, #2");
    }
}

/// Disable interrupt
pub fn disable_irq() {
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!("msr daifset, #2");
    }
}

/// Enable all exceptions
pub fn enable_exceptions() {
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!("msr daifclr, #0xF");
    }
}

/// Disable all exceptions
pub fn disable_exceptions() {
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!("msr daifset, #0xF");
    }
}

/// Initialize exception handler
pub fn init_exceptions() {
    // Set exception vector table
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!("ldr x0, =exception_vectors", "msr vbar_el1, x0", "isb");
    }

    log_info!("Exception vectors installed");
}

/// Alias for init_exceptions (used by trap module)
pub fn init_handler() {
    init_exceptions();
}

/// Check if IRQs are enabled
pub fn irqs_enabled() -> bool {
    // SAFETY: reading DAIF register
    unsafe {
        let daif: u64;
        asm!("mrs {}, daif", out(reg) daif);
        // IRQ bit is bit 1 of DAIF (0 = enabled, 1 = masked)
        (daif & 2) == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_esr_parsing() {
        let esr = EsrElx::from_raw(0x58000000);
        assert_eq!(esr.ec(), 0x16);
        assert!(esr.is_syscall());
    }
}
