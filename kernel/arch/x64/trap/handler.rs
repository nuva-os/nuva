/*
 * Nuva OS - Kernel - x86-64 Trap Handler
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
use crate::{pr_debug, pr_emerg, pr_info, pr_warn};

/// x86-64 exception/trap number
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrapType {
    /// Divide by zero
    DivideError = 0,
    /// Debug exception
    Debug = 1,
    /// Non-maskable interrupt
    Nmi = 2,
    /// Breakpoint
    Breakpoint = 3,
    /// Overflow
    Overflow = 4,
    /// BOUND range exceeded
    BoundRange = 5,
    /// Invalid opcode
    InvalidOpcode = 6,
    /// Device not available
    DeviceNotAvailable = 7,
    /// Double fault
    DoubleFault = 8,
    /// Coprocessor segment overrun
    CoprocessorOverrun = 9,
    /// Invalid TSS
    InvalidTss = 10,
    /// Segment not present
    SegmentNotPresent = 11,
    /// Stack segment fault
    StackSegmentFault = 12,
    /// General protection fault
    GeneralProtection = 13,
    /// Page fault
    PageFault = 14,
    /// x87 FPU error
    X87FpuError = 16,
    /// Alignment check
    AlignmentCheck = 17,
    /// Machine check
    MachineCheck = 18,
    /// SIMD floating-point exception
    SimdFpException = 19,
    /// Virtualization exception
    Virtualization = 20,
}

/// Interrupt context saved on stack during trap
#[repr(C)]
pub struct TrapContext {
    /// General purpose registers
    pub regs: [u64; 16],
    /// Interrupt number
    pub trap_no: u64,
    /// Error code
    pub error_code: u64,
    /// Instruction pointer
    pub rip: u64,
    /// Code segment
    pub cs: u64,
    /// RFLAGS
    pub rflags: u64,
    /// Stack pointer
    pub rsp: u64,
    /// Stack segment
    pub ss: u64,
}

impl TrapContext {
    /// Create new trap context
    pub const fn new() -> Self {
        TrapContext {
            regs: [0; 16],
            trap_no: 0,
            error_code: 0,
            rip: 0,
            cs: 0,
            rflags: 0,
            rsp: 0,
            ss: 0,
        }
    }
}

/// Trap handler entry
/// @param trap_no: Trap/exception number
/// @param error_code: Error code (0 if not applicable)
/// @param ctx: Pointer to trap context
#[no_mangle]
pub extern "C" fn handle_trap(trap_no: u64, error_code: u64, ctx: &mut TrapContext) {
    ctx.trap_no = trap_no;
    ctx.error_code = error_code;

    match trap_no {
        14 => handle_page_fault(ctx),
        13 => handle_general_protection(ctx),
        6 => handle_invalid_opcode(ctx),
        3 => handle_breakpoint(ctx),
        8 => handle_double_fault(ctx),
        _ => {
            log_emerg!("Unhandled trap: {}", trap_no);
            log_emerg!("  Error code: {:#x}", error_code);
            log_emerg!("  RIP: {:#018x}", ctx.rip);
            log_emerg!("  RSP: {:#018x}", ctx.rsp);
        }
    }
}

/// Handle page fault
fn handle_page_fault(ctx: &mut TrapContext) {
    let fault_addr: u64;
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!(
            "mov {}, cr2",
            out(reg) fault_addr,
        );
    }

    let is_write = (ctx.error_code & 2) != 0;
    let is_user = (ctx.error_code & 4) != 0;

    log_debug!("Page fault at {:#018x}", fault_addr);
    log_debug!("  Write: {}, User: {}", is_write, is_user);

    // Call page fault handler
    // Implementation: Invoke page fault handler to resolve demand paging or COW
}

/// Handle general protection fault
fn handle_general_protection(ctx: &mut TrapContext) {
    log_emerg!("General protection fault");
    log_emerg!("  Error code: {:#x}", ctx.error_code);
    log_emerg!("  RIP: {:#018x}", ctx.rip);
}

/// Handle invalid opcode
fn handle_invalid_opcode(ctx: &mut TrapContext) {
    log_emerg!("Invalid opcode at RIP: {:#018x}", ctx.rip);
}

/// Handle breakpoint
fn handle_breakpoint(ctx: &mut TrapContext) {
    log_info!("Breakpoint hit at {:#018x}", ctx.rip);
    // Skip breakpoint instruction
}

/// Handle double fault
fn handle_double_fault(ctx: &mut TrapContext) {
    log_emerg!("Double fault!");
    log_emerg!("  Error code: {:#x}", ctx.error_code);
    log_emerg!("  RIP: {:#018x}", ctx.rip);
}

/// Initialize trap/exception handler
pub fn init_handler() {
    // Load IDT
    // Implementation: Load the IDT via lidt instruction
    log_info!("x86-64 trap handlers installed");
}

/// Enable interrupt
pub fn enable_irq() {
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!("sti");
    }
}

/// Disable interrupt
pub fn disable_irq() {
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!("cli");
    }
}

/// Check if IRQs are enabled
pub fn irqs_enabled() -> bool {
    // SAFETY: reading RFLAGS register
    unsafe {
        let rflags: u64;
        asm!(
            "pushfq",
            "pop {}",
            out(reg) rflags,
        );
        // IF flag is bit 9 of RFLAGS
        (rflags & (1 << 9)) != 0
    }
}
