/*
 * Nuva OS - HAL - ARM64 (AArch64)
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

// ARM64 CPU abstraction
pub mod cpu;

// ARM64 MMU and page table management
pub mod mmu;

// ARM64 interrupt and exception handling
pub mod interrupt;

/// Exception vector base address (VBAR_EL1) - set at runtime via init_arm64_hal
pub static mut VBAR_EL1: u64 = 0;

/// Set the exception vector base address
fn set_vbar(addr: u64) {
    // SAFETY: Writing to VBAR_EL1 sets the exception vector table base.
    // The caller must ensure addr points to a properly aligned vector table.
    unsafe {
        VBAR_EL1 = addr;
        core::arch::asm!("msr VBAR_EL1, {}", in(reg) addr);
        core::arch::asm!("isb");
    }
}

/// UART0 base address for QEMU virt machine
pub const UART0_BASE: u64 = 0x0900_0000;

/// Initialize ARM64 HAL
pub fn init_arm64_hal() {
    // Initialize MMU first (MAIR, TCR, TTBR, enable MMU)
    mmu::init_mmu();
    // Initialize interrupt controller (GICv3 Distributor + Redistributor)
    interrupt::init_interrupt();
    // Set exception vector base (0 = to be set by kernel trap setup)
    set_vbar(0);
}
