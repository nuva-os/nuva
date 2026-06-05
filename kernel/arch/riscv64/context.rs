/*
 * Nuva OS - Kernel - RISC-V 64 Context Switch Operations
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

//! RISC-V 64 context switch operations implementing ContextOps trait.
//! Saves/restores general-purpose, floating-point, and CSR registers.

use core::arch::asm;

use crate::kernel::arch::*;
use super::{read_csr, write_csr};

/// RISC-V 64 context operations.
pub struct RiscV64Context;

impl ContextOps for RiscV64Context {
    fn save_context(&self, ctx: &mut CpuContext) {
        // SAFETY: Inline assembly required for register access.
        unsafe {
            // Save general-purpose registers x1-x31 (x0 is hardwired to 0)
            asm!(
                "sd ra, 8*1({0})",
                "sd sp, 8*2({0})",
                "sd gp, 8*3({0})",
                "sd tp, 8*4({0})",
                "sd t0, 8*5({0})",
                "sd t1, 8*6({0})",
                "sd t2, 8*7({0})",
                "sd s0, 8*8({0})",
                "sd s1, 8*9({0})",
                "sd a0, 8*10({0})",
                "sd a1, 8*11({0})",
                "sd a2, 8*12({0})",
                "sd a3, 8*13({0})",
                "sd a4, 8*14({0})",
                "sd a5, 8*15({0})",
                "sd a6, 8*16({0})",
                "sd a7, 8*17({0})",
                "sd s2, 8*18({0})",
                "sd s3, 8*19({0})",
                "sd s4, 8*20({0})",
                "sd s5, 8*21({0})",
                "sd s6, 8*22({0})",
                "sd s7, 8*23({0})",
                "sd s8, 8*24({0})",
                "sd s9, 8*25({0})",
                "sd s10, 8*26({0})",
                "sd s11, 8*27({0})",
                "sd t3, 8*28({0})",
                "sd t4, 8*29({0})",
                "sd t5, 8*30({0})",
                "sd t6, 8*31({0})",
                in(reg) ctx.regs.as_mut_ptr() as *mut u8,
            );

            // Save SP and PC
            asm!("mv {0}, sp", out(reg) ctx.sp);
            ctx.pc = read_csr!("sepc");
            ctx.pstate = read_csr!("sstatus");

            // Save TLS pointer from tp register
            asm!("mv {0}, tp", out(reg) ctx.tls_base);

            // Save FPU state only if FS != 0 (FPU was active)
            let sstatus = ctx.pstate;
            let fs = (sstatus >> 13) & 0x3;
            if fs != 0 {
                // Save floating-point registers f0-f31 into fpsimd
                // RV64D: 32 x 64-bit registers, stored in fpsimd[0..32]
                for i in 0..32 {
                    asm!(
                        "fmv.x.d {0}, f{1}",
                        out(reg) ctx.fpsimd[i],
                        in(reg) i,
                    );
                }
                // Save fcsr
                asm!("frcsr {0}", out(reg) ctx.fpcr);
            }
        }
    }

    fn restore_context(&self, ctx: &CpuContext) {
        // SAFETY: Inline assembly required for register access.
        unsafe {
            // Restore SP and PC
            asm!("mv sp, {0}", in(reg) ctx.sp);
            write_csr!("sepc", ctx.pc);

            // Restore TLS pointer
            asm!("mv tp, {0}", in(reg) ctx.tls_base);

            // Restore FPU state if it was saved
            let fs = (ctx.pstate >> 13) & 0x3;
            if fs != 0 {
                // Restore floating-point registers f0-f31
                for i in 0..32 {
                    asm!(
                        "fmv.d.x f{1}, {0}",
                        in(reg) ctx.fpsimd[i],
                        in(reg) i,
                    );
                }
                // Restore fcsr
                asm!("fscsr {0}", in(reg) ctx.fpcr);
            }

            // Restore sstatus last
            write_csr!("sstatus", ctx.pstate);

            // Restore general-purpose registers
            asm!(
                "ld ra, 8*1({0})",
                "ld gp, 8*3({0})",
                // Skip tp (already restored for TLS)
                "ld t0, 8*5({0})",
                "ld t1, 8*6({0})",
                "ld t2, 8*7({0})",
                "ld s0, 8*8({0})",
                "ld s1, 8*9({0})",
                "ld a0, 8*10({0})",
                "ld a1, 8*11({0})",
                "ld a2, 8*12({0})",
                "ld a3, 8*13({0})",
                "ld a4, 8*14({0})",
                "ld a5, 8*15({0})",
                "ld a6, 8*16({0})",
                "ld a7, 8*17({0})",
                "ld s2, 8*18({0})",
                "ld s3, 8*19({0})",
                "ld s4, 8*20({0})",
                "ld s5, 8*21({0})",
                "ld s6, 8*22({0})",
                "ld s7, 8*23({0})",
                "ld s8, 8*24({0})",
                "ld s9, 8*25({0})",
                "ld s10, 8*26({0})",
                "ld s11, 8*27({0})",
                "ld t3, 8*28({0})",
                "ld t4, 8*29({0})",
                "ld t5, 8*30({0})",
                "ld t6, 8*31({0})",
                in(reg) ctx.regs.as_ptr() as *const u8,
            );
        }
    }

    fn switch_context(&self, from: &mut CpuContext, to: &CpuContext) {
        // Disable interrupts during context switch
        unsafe { asm!("csrc sstatus, 2"); }
        self.save_context(from);
        self.restore_context(to);
        // Interrupts will be restored via sstatus in restore_context
    }
}
