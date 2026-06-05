/*
 * Nuva OS - Kernel - RISC-V 64 Trap Handler
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

//! RISC-V trap/exception handler with scause-based dispatch.
//! Handles synchronous exceptions and interrupts in S-mode.

use core::arch::asm;

/// Trap context saved on exception entry.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct TrapContext {
    /// General-purpose registers x0-x31 (x0 is always 0).
    pub regs: [u64; 32],
    /// Saved SEPC (exception program counter).
    pub sepc: u64,
    /// Saved SSTATUS.
    pub sstatus: u64,
    /// SCAUSE value (exception code).
    pub scause: u64,
    /// STVAL value (trap value / fault address).
    pub stval: u64,
}

// Exception codes (scause when bit63=0)
const EXC_INST_MISALIGNED: u64 = 0;
const EXC_INST_ACCESS: u64 = 1;
const EXC_ILLEGAL_INST: u64 = 2;
const EXC_BREAKPOINT: u64 = 3;
const EXC_LOAD_MISALIGNED: u64 = 4;
const EXC_LOAD_ACCESS: u64 = 5;
const EXC_STORE_MISALIGNED: u64 = 6;
const EXC_STORE_ACCESS: u64 = 7;
const EXC_ECALL_U: u64 = 8;
const EXC_ECALL_S: u64 = 9;
const EXC_INST_PAGE_FAULT: u64 = 12;
const EXC_LOAD_PAGE_FAULT: u64 = 13;
const EXC_STORE_PAGE_FAULT: u64 = 15;

// Interrupt codes (scause when bit63=1)
const IRQ_SSOFT: u64 = 1;
const IRQ_STIMER: u64 = 5;
const IRQ_SEXT: u64 = 9;

/// Initialize trap handling: set stvec to trap entry.
pub fn init_trap() {
    // SAFETY: stvec write sets the trap vector; inline asm required.
    unsafe {
        asm!(
            "la t0, _trap_entry",
            "csrw stvec, t0",
        );
    }
    log_info!("RISC-V: Trap vector initialized");
}

/// Main trap dispatch handler called from assembly _trap_entry.
///
/// # Safety
/// This function is called from assembly with a valid TrapContext pointer.
#[no_mangle]
pub unsafe extern "C" fn trap_handler(ctx: &mut TrapContext) {
    let scause = ctx.scause;
    let is_interrupt = (scause >> 63) & 1 == 1;
    let code = scause & !(1u64 << 63);

    if is_interrupt {
        handle_interrupt(ctx, code);
    } else {
        handle_exception(ctx, code);
    }
}

/// Handle an interrupt (scause bit63=1).
fn handle_interrupt(_ctx: &mut TrapContext, code: u64) {
    match code {
        IRQ_SSOFT => {
            // Software interrupt / IPI
            // Acknowledge IPI by clearing sip.SSIP
            unsafe { asm!("csrw sip, zero"); }
            log_info!("RISC-V: Software interrupt (IPI)");
        }
        IRQ_STIMER => {
            // Timer interrupt
            log_info!("RISC-V: Timer interrupt");
            // Timer handler will be invoked via registered callback
        }
        IRQ_SEXT => {
            // External interrupt (PLIC)
            log_info!("RISC-V: External interrupt");
            // PLIC handler will claim and dispatch the IRQ
        }
        _ => {
            log_warn!("RISC-V: Unknown interrupt code: {}", code);
        }
    }
}

/// Handle a synchronous exception (scause bit63=0).
fn handle_exception(ctx: &mut TrapContext, code: u64) {
    match code {
        EXC_ECALL_U => {
            // System call from user mode
            // a7 = syscall number, a0-a5 = arguments
            let _sysno = ctx.regs[17]; // a7
            let _arg0 = ctx.regs[10]; // a0
            let _arg1 = ctx.regs[11]; // a1
            let _arg2 = ctx.regs[12]; // a2
            let _arg3 = ctx.regs[13]; // a3
            let _arg4 = ctx.regs[14]; // a4
            let _arg5 = ctx.regs[15]; // a5

            // Advance SEPC past the ecall instruction (4 bytes)
            ctx.sepc = ctx.sepc.wrapping_add(4);

            // TODO: Dispatch to syscall handler
            // ctx.regs[10] = syscall_dispatch(sysno, arg0..arg5);
            ctx.regs[10] = 0; // Return 0 for unimplemented syscalls
        }
        EXC_ECALL_S => {
            // S-mode ecall (should not happen in kernel)
            log_warn!("RISC-V: S-mode ecall at SEPC={:#x}", ctx.sepc);
            ctx.sepc = ctx.sepc.wrapping_add(4);
        }
        EXC_INST_PAGE_FAULT | EXC_LOAD_PAGE_FAULT | EXC_STORE_PAGE_FAULT => {
            // Page fault: stval contains fault address
            log_warn!(
                "RISC-V: Page fault (code={}) at SEPC={:#x}, STVAL={:#x}",
                code, ctx.sepc, ctx.stval
            );
            // TODO: Dispatch to page fault handler
        }
        EXC_ILLEGAL_INST => {
            log_warn!(
                "RISC-V: Illegal instruction at SEPC={:#x}, STVAL={:#x}",
                ctx.sepc, ctx.stval
            );
        }
        EXC_BREAKPOINT => {
            log_info!("RISC-V: Breakpoint at SEPC={:#x}", ctx.sepc);
        }
        _ => {
            log_warn!(
                "RISC-V: Unhandled exception code={} at SEPC={:#x}, STVAL={:#x}",
                code, ctx.sepc, ctx.stval
            );
        }
    }
}
