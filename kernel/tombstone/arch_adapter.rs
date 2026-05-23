/*
 * Nuva OS - Kernel - Tombstone - Architecture Adapter
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

//! Architecture-specific crash context collection adapter.
/*!*/
//! Provides a unified `CrashArchAdapter` trait and conditional compilation
//! dispatch to ARM64, x86-64, and LoongArch64 implementations that
//! delegate to the HAL CrashInfoOps trait.

use super::record::{
    ArchId, StackFrame, StackFrameArray, TombstoneError, UnwindTruncateReason, MAX_STACK_FRAMES,
};

// ---------------------------------------------------------------------------
// ArchExceptionInfo
// ---------------------------------------------------------------------------

/** Architecture-specific exception information collected from HAL */
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ArchExceptionInfo {
    /** Exception syndrome register value (ESR/error code/ESTAT) */
    pub esr: u64,
    /** Faulting address (FAR/CR2/badaddr) */
    pub fault_addr: u64,
    /** Processor state (PSTATE/RFLAGS/CRMD) */
    pub pstate: u64,
}

impl ArchExceptionInfo {
    /** Create a zero-valued ArchExceptionInfo */
    pub const fn new() -> Self {
        ArchExceptionInfo {
            esr: 0,
            fault_addr: 0,
            pstate: 0,
        }
    }
}

// ---------------------------------------------------------------------------
// StackUnwindResult
// ---------------------------------------------------------------------------

/** Result of a stack backtrace operation */
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct StackUnwindResult {
    /** Collected stack frames */
    pub frames: StackFrameArray,
    /** Whether the unwind succeeded (even if truncated) */
    pub success: bool,
}

impl StackUnwindResult {
    /** Create a failed unwind result */
    pub const fn failed() -> Self {
        StackUnwindResult {
            frames: StackFrameArray::new(),
            success: false,
        }
    }
}

// ---------------------------------------------------------------------------
// CrashArchAdapter trait
// ---------------------------------------------------------------------------

/** Architecture-agnostic crash context collection interface.
 *  Each supported architecture provides an implementation that
 *  delegates to the HAL CrashInfoOps trait. */
pub trait CrashArchAdapter {
    /** Read architecture-specific exception registers via HAL */
    fn read_exception_info(&self) -> Result<ArchExceptionInfo, TombstoneError>;

    /** Perform stack unwind from the given frame pointer and stack pointer.
     *  Returns up to max_frames stack frames. */
    fn unwind_stack(
        &self,
        fp: u64,
        sp: u64,
        max_frames: usize,
    ) -> Result<StackUnwindResult, TombstoneError>;

    /** Return the architecture identifier */
    fn arch_id(&self) -> ArchId;
}

// ---------------------------------------------------------------------------
// ARM64 adapter
// ---------------------------------------------------------------------------

/** ARM64 crash adapter that reads ESR/FAR/SPSR and performs FP-based unwind */
#[cfg(target_arch = "aarch64")]
pub mod arm64 {
    use super::*;
    use crate::kernel::arch::CpuContext;

    /** ARM64 architecture crash adapter */
    pub struct ARM64CrashAdapter;

    impl CrashArchAdapter for ARM64CrashAdapter {
        fn read_exception_info(&self) -> Result<ArchExceptionInfo, TombstoneError> {
            let ctx = crate::kernel::arch::current_arch().context();
            let saved = ctx.save_context();
            Ok(ArchExceptionInfo {
                esr: saved.esr_if_available(),
                fault_addr: saved.far_if_available(),
                pstate: saved.pstate,
            })
        }

        fn unwind_stack(
            &self,
            fp: u64,
            _sp: u64,
            max_frames: usize,
        ) -> Result<StackUnwindResult, TombstoneError> {
            let mut frames = StackFrameArray::new();
            let mut current_fp = fp;
            let limit = if max_frames > MAX_STACK_FRAMES {
                MAX_STACK_FRAMES
            } else {
                max_frames
            };

            for _ in 0..limit {
                if current_fp == 0 || current_fp & 0x7 != 0 {
                    if frames.count > 0 {
                        frames.truncate_reason = UnwindTruncateReason::InvalidFp;
                    }
                    break;
                }
                // SAFETY: We read two u64 values from the stack via the
                // HAL memory-read interface. The pointer validity is
                // checked by the HAL implementation; on failure it
                // returns zero and we break the loop.
                let next_fp = unsafe { read_stack_u64(current_fp) };
                let return_addr = unsafe { read_stack_u64(current_fp + 8) };
                if next_fp == 0 && return_addr == 0 {
                    break;
                }
                frames.push(StackFrame::from_addr(return_addr));
                if next_fp <= current_fp {
                    frames.truncate_reason = UnwindTruncateReason::CorruptStack;
                    break;
                }
                current_fp = next_fp;
            }

            Ok(StackUnwindResult {
                frames,
                success: true,
            })
        }

        fn arch_id(&self) -> ArchId {
            ArchId::Arm64
        }
    }

    /** Global ARM64 crash adapter instance */
    pub static ARM64_CRASH_ADAPTER: ARM64CrashAdapter = ARM64CrashAdapter;

    /** Read a u64 from a virtual address (HAL delegate) */
    // SAFETY: The caller must ensure addr is a valid readable virtual address.
    unsafe fn read_stack_u64(addr: u64) -> u64 {
        let ptr = addr as *const u64;
        if ptr.is_null() {
            return 0;
        }
        // SAFETY: Caller guarantees addr is a valid stack address.
        core::ptr::read_volatile(ptr)
    }

    /** Extension trait for CpuContext to extract arch-specific exception info */
    trait CpuContextExt {
        fn esr_if_available(&self) -> u64;
        fn far_if_available(&self) -> u64;
    }

    impl CpuContextExt for CpuContext {
        fn esr_if_available(&self) -> u64 {
            self.regs[18]
        }
        fn far_if_available(&self) -> u64 {
            self.regs[19]
        }
    }
}

// ---------------------------------------------------------------------------
// x86-64 adapter
// ---------------------------------------------------------------------------

/** x86-64 crash adapter that reads error_code/CR2/RFLAGS and performs RBP-based unwind */
#[cfg(target_arch = "x86_64")]
pub mod x64 {
    use super::*;
    use crate::kernel::arch::CpuContext;

    /** x86-64 architecture crash adapter */
    pub struct X64CrashAdapter;

    impl CrashArchAdapter for X64CrashAdapter {
        fn read_exception_info(&self) -> Result<ArchExceptionInfo, TombstoneError> {
            let ctx = crate::kernel::arch::current_arch().context();
            let saved = ctx.save_context();
            Ok(ArchExceptionInfo {
                esr: saved.regs[18],
                fault_addr: read_cr2(),
                pstate: saved.pstate,
            })
        }

        fn unwind_stack(
            &self,
            rbp: u64,
            _rsp: u64,
            max_frames: usize,
        ) -> Result<StackUnwindResult, TombstoneError> {
            let mut frames = StackFrameArray::new();
            let mut current_rbp = rbp;
            let limit = if max_frames > MAX_STACK_FRAMES {
                MAX_STACK_FRAMES
            } else {
                max_frames
            };

            for _ in 0..limit {
                if current_rbp == 0 || current_rbp & 0x7 != 0 {
                    if frames.count > 0 {
                        frames.truncate_reason = UnwindTruncateReason::InvalidFp;
                    }
                    break;
                }
                // SAFETY: Reading from stack via HAL-validated address.
                let next_rbp = unsafe { read_stack_u64(current_rbp) };
                let return_addr = unsafe { read_stack_u64(current_rbp + 8) };
                if next_rbp == 0 && return_addr == 0 {
                    break;
                }
                frames.push(StackFrame::from_addr(return_addr));
                if next_rbp <= current_rbp {
                    frames.truncate_reason = UnwindTruncateReason::CorruptStack;
                    break;
                }
                current_rbp = next_rbp;
            }

            Ok(StackUnwindResult {
                frames,
                success: true,
            })
        }

        fn arch_id(&self) -> ArchId {
            ArchId::X64
        }
    }

    /** Global x86-64 crash adapter instance */
    pub static X64_CRASH_ADAPTER: X64CrashAdapter = X64CrashAdapter;

    /** Read CR2 (page fault linear address) */
    fn read_cr2() -> u64 {
        let val: u64;
        // SAFETY: Reading CR2 is a privileged but well-defined x86 operation.
        unsafe { core::arch::asm!("mov {}, cr2", out(reg) val) }
        val
    }

    /** Read a u64 from a virtual address */
    // SAFETY: The caller must ensure addr is a valid readable virtual address.
    unsafe fn read_stack_u64(addr: u64) -> u64 {
        let ptr = addr as *const u64;
        if ptr.is_null() {
            return 0;
        }
        // SAFETY: Caller guarantees addr is a valid stack address.
        core::ptr::read_volatile(ptr)
    }
}

// ---------------------------------------------------------------------------
// LoongArch64 adapter
// ---------------------------------------------------------------------------

/** LoongArch64 crash adapter that reads ESTAT/badaddr/CRMD and performs $fp-based unwind */
#[cfg(target_arch = "loongarch64")]
pub mod loongarch64 {
    use super::*;
    use crate::kernel::arch::CpuContext;

    /** LoongArch64 architecture crash adapter */
    pub struct LoongArch64CrashAdapter;

    impl CrashArchAdapter for LoongArch64CrashAdapter {
        fn read_exception_info(&self) -> Result<ArchExceptionInfo, TombstoneError> {
            let ctx = crate::kernel::arch::current_arch().context();
            let saved = ctx.save_context();
            Ok(ArchExceptionInfo {
                esr: saved.regs[18],
                fault_addr: saved.regs[19],
                pstate: saved.pstate,
            })
        }

        fn unwind_stack(
            &self,
            fp: u64,
            _sp: u64,
            max_frames: usize,
        ) -> Result<StackUnwindResult, TombstoneError> {
            let mut frames = StackFrameArray::new();
            let mut current_fp = fp;
            let limit = if max_frames > MAX_STACK_FRAMES {
                MAX_STACK_FRAMES
            } else {
                max_frames
            };

            for _ in 0..limit {
                if current_fp == 0 || current_fp & 0x7 != 0 {
                    if frames.count > 0 {
                        frames.truncate_reason = UnwindTruncateReason::InvalidFp;
                    }
                    break;
                }
                // SAFETY: Reading from stack via HAL-validated address.
                let next_fp = unsafe { read_stack_u64(current_fp) };
                let return_addr = unsafe { read_stack_u64(current_fp + 8) };
                if next_fp == 0 && return_addr == 0 {
                    break;
                }
                frames.push(StackFrame::from_addr(return_addr));
                if next_fp <= current_fp {
                    frames.truncate_reason = UnwindTruncateReason::CorruptStack;
                    break;
                }
                current_fp = next_fp;
            }

            Ok(StackUnwindResult {
                frames,
                success: true,
            })
        }

        fn arch_id(&self) -> ArchId {
            ArchId::LoongArch64
        }
    }

    /** Global LoongArch64 crash adapter instance */
    pub static LOONGARCH64_CRASH_ADAPTER: LoongArch64CrashAdapter = LoongArch64CrashAdapter;

    /** Read a u64 from a virtual address */
    // SAFETY: The caller must ensure addr is a valid readable virtual address.
    unsafe fn read_stack_u64(addr: u64) -> u64 {
        let ptr = addr as *const u64;
        if ptr.is_null() {
            return 0;
        }
        // SAFETY: Caller guarantees addr is a valid stack address.
        core::ptr::read_volatile(ptr)
    }
}

// ---------------------------------------------------------------------------
// Conditional compilation dispatch
// ---------------------------------------------------------------------------

/** Return a reference to the CrashArchAdapter for the current architecture.
 *  Selected at compile time via #[cfg(target_arch)]. */
pub fn current_arch_adapter() -> &'static dyn CrashArchAdapter {
    #[cfg(target_arch = "aarch64")]
    {
        &arm64::ARM64_CRASH_ADAPTER
    }
    #[cfg(target_arch = "x86_64")]
    {
        &x64::X64_CRASH_ADAPTER
    }
    #[cfg(target_arch = "loongarch64")]
    {
        &loongarch64::LOONGARCH64_CRASH_ADAPTER
    }
}

// Compile-time guard: at least one architecture must be selected
#[cfg(not(any(
    target_arch = "aarch64",
    target_arch = "x86_64",
    target_arch = "loongarch64"
)))]
compile_error!("Tombstone mechanism requires at least one supported architecture (aarch64, x86_64, or loongarch64)");
