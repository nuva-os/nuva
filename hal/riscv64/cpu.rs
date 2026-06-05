/*
 * Nuva OS - HAL - RISC-V 64 CPU
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

//! RISC-V 64 CPU HAL: Hart management and CSR access wrappers.

use core::arch::asm;

/// Initialize CPU HAL.
pub fn init_cpu_hal() {
    log_info!("RISC-V: CPU HAL init (Hart {})", hart_id());
}

/// Get current hart ID.
pub fn hart_id() -> u64 {
    let id: u64;
    // SAFETY: mhartid is a read-only CSR.
    unsafe { asm!("csrr {}, mhartid", out(reg) id); }
    id
}

/// Read a CSR register by address.
///
/// # Safety
/// The caller must ensure the CSR address is valid.
pub unsafe fn read_csr_raw(csr: u64) -> u64 {
    let val: u64;
    asm!("csrr {0}, {1}", out(reg) val, in(reg) csr);
    val
}

/// Write a CSR register by address.
///
/// # Safety
/// The caller must ensure the CSR address is valid and the write is safe.
pub unsafe fn write_csr_raw(csr: u64, val: u64) {
    asm!("csrw {0}, {1}", in(reg) csr, in(reg) val);
}

/// Start a secondary hart via SBI HSM.
pub fn start_hart(hartid: u64, start_addr: u64) -> bool {
    use crate::kernel::arch::riscv64::sbi;
    let ret = sbi::hart_start(hartid, start_addr, 0);
    ret.error == sbi::SBI_SUCCESS
}

/// Suspend the current hart.
pub fn suspend_hart() {
    use crate::kernel::arch::riscv64::sbi;
    let _ = sbi::hart_suspend(0, 0, 0);
}
