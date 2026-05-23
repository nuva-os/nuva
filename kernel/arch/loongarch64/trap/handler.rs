/*
* Nuva OS - Kernel - LoongArch64 Trap Handler
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

/// LoongArch64 exception type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExceptionType {
    /// Integer overflow
    IntOverflow = 0,
    /// Page modified exception
    Pme = 1,
    /// Page not readable
    Pnr = 2,
    /// Page not executable
    Pnx = 3,
    /// Page not writable
    Pnw = 4,
    /// Privilege error
    Ppi = 5,
    /// Page invalid exception
    Pif = 6,
    /// Stack overflow
    Ss = 7,
    /// FPU exception
    Fpe = 8,
    /// Breakpoint (BRK instruction)
    Brk = 9,
    /// System call (SYSCALL instruction)
    Syscall = 11,
    /// Timer interrupt
    Timer = 12,
    /// IPI interrupt
    Ipi = 13,
    /// External interrupt
    External = 14,
}

/// Exception context
#[repr(C)]
pub struct ExceptionContext {
    /// General registers $r0-$r31
    pub regs: [u64; 32],
    /// Stack pointer
    pub sp: u64,
    /// Exception return address (ERA)
    pub era: u64,
    /// Bad virtual address (BADV)
    pub badv: u64,
    /// Exception status (ESTAT)
    pub estat: u32,
    /// Exception config (ECFG)
    pub ecfg: u32,
    /// Current mode register (CRMD)
    pub crmd: u32,
    /// Previous mode register (PRMD)
    pub prmd: u32,
}

impl ExceptionContext {
    /// Create new exception context
    pub const fn new() -> Self {
        ExceptionContext {
            regs: [0; 32],
            sp: 0,
            era: 0,
            badv: 0,
            estat: 0,
            ecfg: 0,
            crmd: 0,
            prmd: 0,
        }
    }
}

/// ESTAT register parser
#[derive(Debug, Clone, Copy)]
pub struct Estat {
    pub value: u32,
}

impl Estat {
    /// Create from ESTAT register value
    pub fn from_raw(value: u32) -> Self {
        Estat { value }
    }

    /// Get exception code (bits 16..=22)
    pub fn ecode(&self) -> u32 {
        (self.value >> 16) & 0x3F
    }

    /// Get interrupt pending bits
    pub fn is(&self) -> u32 {
        self.value & 0xFFF_FFFF
    }

    /// Check if system call (ecode == 11)
    pub fn is_syscall(&self) -> bool {
        self.ecode() == 11
    }

    /// Check if page fault (ecode 2..=6)
    pub fn is_page_fault(&self) -> bool {
        let ecode = self.ecode();
        ecode >= 2 && ecode <= 6
    }

    /// Check if breakpoint (ecode == 9)
    pub fn is_breakpoint(&self) -> bool {
        self.ecode() == 9
    }
}

/// Exception handler entry
/// @param exc_type: Exception type code
/// @param ctx: Pointer to exception context
#[no_mangle]
pub extern "C" fn handle_exception(exc_type: u64, ctx: &mut ExceptionContext) {
    let exc_type = match exc_type {
        0 => ExceptionType::IntOverflow,
        1 => ExceptionType::Pme,
        2 => ExceptionType::Pnr,
        3 => ExceptionType::Pnx,
        4 => ExceptionType::Pnw,
        5 => ExceptionType::Ppi,
        6 => ExceptionType::Pif,
        7 => ExceptionType::Ss,
        8 => ExceptionType::Fpe,
        9 => ExceptionType::Brk,
        11 => ExceptionType::Syscall,
        12 => ExceptionType::Timer,
        13 => ExceptionType::Ipi,
        14 => ExceptionType::External,
        _ => {
            log_emerg!("Unknown exception type: {}", exc_type);
            return;
        }
    };

    // Read BADV and ESTAT
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!("csrrd {}, 0x7", out(reg) ctx.badv);
    }
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        let estat: u32;
        asm!("csrrd {}, 0x5", out(reg) estat);
        ctx.estat = estat;
    }

    match exc_type {
        ExceptionType::Syscall => {
            handle_syscall(ctx);
        }
        ExceptionType::Pnr
        | ExceptionType::Pnx
        | ExceptionType::Pnw
        | ExceptionType::Pme
        | ExceptionType::Pif => {
            handle_page_fault(exc_type, ctx);
        }
        ExceptionType::Brk => {
            handle_breakpoint(ctx);
        }
        ExceptionType::Timer => {
            handle_timer_interrupt(ctx);
        }
        ExceptionType::Ipi => {
            handle_ipi_interrupt(ctx);
        }
        ExceptionType::External => {
            handle_external_interrupt(ctx);
        }
        _ => {
            log_emerg!("Unhandled exception: {:?}", exc_type);
            log_emerg!("  ERA: {:#018x}", ctx.era);
            log_emerg!("  BADV: {:#018x}", ctx.badv);
            log_emerg!("  ESTAT: {:#010x}", ctx.estat);
        }
    }
}

/// Handle page fault
fn handle_page_fault(exc_type: ExceptionType, ctx: &mut ExceptionContext) {
    let estat = Estat::from_raw(ctx.estat);
    let is_write = exc_type == ExceptionType::Pnw;
    let is_user = (ctx.crmd & 0x3) == 3;

    log_debug!("Page fault at {:#018x}", ctx.badv);
    log_debug!(
        "  Type: {:?}, Write: {}, User: {}",
        exc_type,
        is_write,
        is_user
    );

    // Implementation: Invoke page fault handler to resolve demand paging or COW
    let _ = estat;
}

/// Handle breakpoint
fn handle_breakpoint(ctx: &mut ExceptionContext) {
    log_info!("Breakpoint hit at {:#018x}", ctx.era);
    // Skip breakpoint instruction
}

/// Handle system call
#[no_mangle]
pub extern "C" fn handle_syscall(ctx: &mut ExceptionContext) {
    let syscall_num = ctx.regs[11]; // $a7 = $r11

    let args = [
        ctx.regs[4], // $a0
        ctx.regs[5], // $a1
        ctx.regs[6], // $a2
        ctx.regs[7], // $a3
        ctx.regs[8], // $a4
        ctx.regs[9], // $a5
    ];

    log_debug!("Syscall {} called", syscall_num);

    // Call system call dispatcher
    let result = crate::kernel::syscall::dispatch(syscall_num, &args);
    ctx.regs[4] = result as u64; // Return in $a0
}

/// Handle timer interrupt
fn handle_timer_interrupt(_ctx: &mut ExceptionContext) {
    // Clear timer interrupt
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!("csrwr $r0, 0x44");
    }
    // Implementation: Invoke scheduler tick
}

/// Handle IPI interrupt
fn handle_ipi_interrupt(_ctx: &mut ExceptionContext) {
    // Implementation: Process IPI messages
}

/// Handle external interrupt
fn handle_external_interrupt(_ctx: &mut ExceptionContext) {
    // Implementation: Dispatch through EIOINTC handler
}

/// Enable interrupt
pub fn enable_irq() {
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!("csrrd $t0, 0x0", "ori $t0, $t0, 1", "csrwr $t0, 0x0",);
    }
}

/// Disable interrupt
pub fn disable_irq() {
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!("csrrd $t0, 0x0", "andi $t0, $t0, ~1", "csrwr $t0, 0x0",);
    }
}

/// Initialize exception handler
pub fn init_handler() {
    // Set exception entry address (CSR EENTRY)
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!("csrwr $r0, 0xc",);
    }

    log_info!("LoongArch64 exception handlers installed");
}

/// Check if IRQs are enabled
pub fn irqs_enabled() -> bool {
    // SAFETY: reading CRMD register
    unsafe {
        let crmd: u32;
        asm!("csrrd {}, 0x0", out(reg) crmd);
        // IE bit is bit 0 of CRMD
        (crmd & 1) != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_estat_parsing() {
        let estat = Estat::from_raw(0x000B_0000);
        assert_eq!(estat.ecode(), 0x0B);
        assert!(estat.is_syscall());
    }
}
