/*
 * Nuva OS - Kernel - Tombstone - Crash Context Collector
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

//! Crash context collection from HAL layer.
/*!*/
//! Collects CPU registers, exception information, and stack backtrace
//! from the architecture adapter and assembles them into a CrashContext.

use super::arch_adapter::{current_arch_adapter, ArchExceptionInfo};
use super::record::{
    ArchId, CrashReason, RegisterArray, StackFrameArray, TombstoneError, MAX_REGISTERS,
    MAX_STACK_FRAMES,
};

// ---------------------------------------------------------------------------
// CrashContext
// ---------------------------------------------------------------------------

/** Intermediate crash context before assembling a full TombstoneRecord */
#[repr(C)]
#[derive(Debug, Clone)]
pub struct CrashContext {
    /** General-purpose registers at crash time */
    pub registers: RegisterArray,
    /** Stack pointer */
    pub sp: u64,
    /** Program counter */
    pub pc: u64,
    /** Faulting address */
    pub fault_addr: u64,
    /** Exception syndrome register */
    pub esr: u64,
    /** Processor state register */
    pub pstate: u64,
    /** Stack backtrace */
    pub stack_frames: StackFrameArray,
    /** Architecture identifier */
    pub arch_id: ArchId,
    /** Whether context collection was incomplete */
    pub context_incomplete: bool,
}

impl CrashContext {
    /** Create a minimal CrashContext indicating collection failure */
    pub fn minimal() -> Self {
        CrashContext {
            registers: RegisterArray::new(),
            sp: 0,
            pc: 0,
            fault_addr: 0,
            esr: 0,
            pstate: 0,
            stack_frames: StackFrameArray::new(),
            arch_id: ArchId::current(),
            context_incomplete: true,
        }
    }
}

// ---------------------------------------------------------------------------
// CrashSource
// ---------------------------------------------------------------------------

/** Origin of the crash event */
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrashSource {
    /** Task transitioned to Zombie/Dead due to kernel fault */
    TaskCrash,
    /** Fatal signal delivered to the process */
    FatalSignal,
    /** Watchdog detected task timeout */
    Watchdog,
}

// ---------------------------------------------------------------------------
// collect_crash_context
// ---------------------------------------------------------------------------

/** Collect the full crash context from the current architecture adapter.
 *  On any step failure, returns a partially-filled CrashContext with
 *  context_incomplete = true. Never panics. */
pub fn collect_crash_context(
    crash_source: CrashSource,
    _pid: u32,
    _tid: u32,
    _signal: Option<u8>,
) -> Result<CrashContext, TombstoneError> {
    let adapter = current_arch_adapter();
    let mut context_incomplete = false;

    // 1. Save CPU context from HAL
    let cpu_ctx = crate::kernel::arch::current_arch().context().save_context();

    // 2. Read architecture-specific exception information
    let exc_info: ArchExceptionInfo = match adapter.read_exception_info() {
        Ok(info) => info,
        Err(_) => {
            context_incomplete = true;
            ArchExceptionInfo::new()
        }
    };

    // 3. Build register array from CpuContext
    let mut registers = RegisterArray::new();
    let reg_count = if cpu_ctx.regs.len() > MAX_REGISTERS {
        MAX_REGISTERS
    } else {
        cpu_ctx.regs.len()
    };
    registers.set_regs(&cpu_ctx.regs[..reg_count]);

    // 4. Perform stack backtrace
    let fp = extract_frame_pointer(&cpu_ctx);
    let sp = cpu_ctx.sp;
    let stack_frames: StackFrameArray = match adapter.unwind_stack(fp, sp, MAX_STACK_FRAMES) {
        Ok(result) => result.frames,
        Err(_) => {
            context_incomplete = true;
            StackFrameArray::new()
        }
    };

    // 5. Determine PC from CPU context
    let pc = cpu_ctx.pc;

    // 6. Assemble CrashContext
    let ctx = CrashContext {
        registers,
        sp,
        pc,
        fault_addr: exc_info.fault_addr,
        esr: exc_info.esr,
        pstate: exc_info.pstate,
        stack_frames,
        arch_id: adapter.arch_id(),
        context_incomplete,
    };

    if context_incomplete {
        log_warn!(
            "Crash context collection incomplete for source={:?}",
            crash_source
        );
    }

    Ok(ctx)
}

/** Extract the frame pointer from CpuContext for stack unwinding.
 *  Architecture-specific: ARM64=x29, x86-64=rbp, LoongArch64=$r22 */
fn extract_frame_pointer(ctx: &crate::kernel::arch::CpuContext) -> u64 {
    #[cfg(target_arch = "aarch64")]
    {
        ctx.regs[29]
    }
    #[cfg(target_arch = "x86_64")]
    {
        ctx.regs[10]
    }
    #[cfg(target_arch = "loongarch64")]
    {
        ctx.regs[22]
    }
}

// ---------------------------------------------------------------------------
// mask_sensitive_registers
// ---------------------------------------------------------------------------

/** Mask potentially sensitive callee-saved registers that may contain
 *  cryptographic keys, tokens, or other secrets.
 *  Only argument registers and frame-related registers are preserved. */
pub fn mask_sensitive_registers(regs: &mut RegisterArray, arch_id: ArchId) {
    match arch_id {
        ArchId::Arm64 => {
            // ARM64: preserve x0-x7 (args), x29 (FP), x30 (LR)
            // Mask x8-x28 (callee-saved may hold secrets)
            for i in 8..29 {
                if i < MAX_REGISTERS {
                    regs.regs[i] = 0;
                }
            }
        }
        ArchId::X64 => {
            // x86-64: preserve rdi,rsi,rdx,rcx,r8,r9 (args), rbp(frame)
            // Mapping: rdi=regs[0], rsi=regs[1], rdx=regs[2], rcx=regs[3],
            //          r8=regs[4], r9=regs[5], rbp=regs[10]
            let preserve: [usize; 7] = [0, 1, 2, 3, 4, 5, 10];
            for i in 0..MAX_REGISTERS {
                if !preserve.contains(&i) {
                    regs.regs[i] = 0;
                }
            }
        }
        ArchId::LoongArch64 => {
            // LoongArch64: preserve $a0-$a7 (args, r4-r11), $fp(r22), $ra(r1)
            let preserve: [usize; 10] = [1, 4, 5, 6, 7, 8, 9, 10, 11, 22];
            for i in 0..MAX_REGISTERS {
                if !preserve.contains(&i) {
                    regs.regs[i] = 0;
                }
            }
        }
    }
}
