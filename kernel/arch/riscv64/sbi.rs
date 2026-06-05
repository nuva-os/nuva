/*
 * Nuva OS - Kernel - RISC-V 64 SBI Interface
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

//! SBI (Supervisor Binary Interface) ecall wrappers for RISC-V S-mode.
//! Provides access to timer, reset, HSM, and console extensions.

use core::arch::asm;

/// SBI return value structure.
#[derive(Debug, Clone, Copy)]
pub struct SbiRet {
    /// Error code (0 = success).
    pub error: i64,
    /// Return value.
    pub value: i64,
}

// SBI error codes
pub const SBI_SUCCESS: i64 = 0;
pub const SBI_ERR_FAILED: i64 = -1;
pub const SBI_ERR_NOT_SUPPORTED: i64 = -2;
pub const SBI_ERR_INVALID_PARAM: i64 = -3;
pub const SBI_ERR_DENIED: i64 = -4;
pub const SBI_ERR_INVALID_ADDRESS: i64 = -5;

// SBI extension IDs
pub const SBI_EXT_BASE: u64 = 0x10;
pub const SBI_EXT_TIMER: u64 = 0x00;
pub const SBI_EXT_IPI: u64 = 0x00;
pub const SBI_EXT_RESET: u64 = 0x01;
pub const SBI_EXT_HSM: u64 = 0x02;
pub const SBI_EXT_DBCN: u64 = 0x44;

// SBI base function IDs
pub const SBI_BASE_GET_SPEC_VERSION: u64 = 0x00;
pub const SBI_BASE_GET_IMPL_ID: u64 = 0x01;
pub const SBI_BASE_GET_IMPL_VERSION: u64 = 0x02;
pub const SBI_BASE_PROBE_EXTENSION: u64 = 0x03;
pub const SBI_BASE_GET_MVENDORID: u64 = 0x04;
pub const SBI_BASE_GET_MARCHID: u64 = 0x05;
pub const SBI_BASE_GET_MIMPID: u64 = 0x06;

// SBI reset types
pub const SBI_RESET_TYPE_SHUTDOWN: u32 = 0x00;
pub const SBI_RESET_TYPE_COLD_REBOOT: u32 = 0x01;
pub const SBI_RESET_TYPE_WARM_REBOOT: u32 = 0x02;

/// Generic SBI ecall with up to 6 parameters.
///
/// # Safety
/// This function uses inline assembly to perform an ecall instruction.
#[inline(always)]
pub fn sbi_call(eid: u64, fid: u64, arg0: u64, arg1: u64, arg2: u64) -> SbiRet {
    let error: i64;
    let value: i64;
    unsafe {
        asm!(
            "ecall",
            in("a7") eid,
            in("a6") fid,
            inlateout("a0") arg0 => error,
            inlateout("a1") arg1 => value,
            in("a2") arg2,
        );
    }
    SbiRet { error, value }
}

/// SBI ecall with 0 additional arguments.
#[inline(always)]
pub fn sbi_call_0(eid: u64, fid: u64) -> SbiRet {
    sbi_call(eid, fid, 0, 0, 0)
}

/// SBI ecall with 1 additional argument.
#[inline(always)]
pub fn sbi_call_1(eid: u64, fid: u64, arg0: u64) -> SbiRet {
    sbi_call(eid, fid, arg0, 0, 0)
}

/// SBI ecall with 2 additional arguments.
#[inline(always)]
pub fn sbi_call_2(eid: u64, fid: u64, arg0: u64, arg1: u64) -> SbiRet {
    sbi_call(eid, fid, arg0, arg1, 0)
}

/// Output a character via SBI debug console extension (eid=0x44, fid=0x02).
pub fn console_putchar(ch: u8) {
    let _ = sbi_call_1(SBI_EXT_DBCN, 0x02, ch as u64);
}

/// Set the next timer event via SBI timer extension (eid=0x00, fid=0x00).
pub fn timer_set(next_value: u64) -> SbiRet {
    sbi_call_1(SBI_EXT_TIMER, 0x00, next_value)
}

/// System reset via SBI reset extension (eid=0x01, fid=0x00).
pub fn system_reset(reset_type: u32, reason: u32) -> SbiRet {
    sbi_call_2(SBI_EXT_RESET, 0x00, reset_type as u64, reason as u64)
}

/// Start a halted hart via SBI HSM extension (eid=0x02, fid=0x00).
pub fn hart_start(hartid: u64, start_addr: u64, opaque: u64) -> SbiRet {
    sbi_call(SBI_EXT_HSM, 0x00, hartid, start_addr, opaque)
}

/// Suspend a hart via SBI HSM extension (eid=0x02, fid=0x01).
pub fn hart_suspend(suspend_type: u32, resume_addr: u64, opaque: u64) -> SbiRet {
    sbi_call(SBI_EXT_HSM, 0x01, suspend_type as u64, resume_addr, opaque)
}

/// Send IPI to target hart via SBI IPI extension (eid=0x00, fid=0x00).
pub fn send_ipi(hart_mask: u64) -> SbiRet {
    sbi_call_1(SBI_EXT_IPI, 0x00, hart_mask)
}

/// Probe whether an SBI extension is available.
pub fn probe_extension(eid: u64) -> SbiRet {
    sbi_call_1(SBI_EXT_BASE, SBI_BASE_PROBE_EXTENSION, eid)
}

/// Get SBI specification version.
pub fn get_spec_version() -> SbiRet {
    sbi_call_0(SBI_EXT_BASE, SBI_BASE_GET_SPEC_VERSION)
}
