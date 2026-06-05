/*
 * Nuva OS - Kernel - RISC-V 64 Architecture Implementation
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

//! RISC-V 64 architecture implementation of ArchOps trait.
//! Integrates all sub-operations (page table, IRQ, timer, power, context).

use core::arch::asm;

use crate::kernel::arch::*;
use super::*;

/// RISC-V 64 power management implementation via SBI.
pub struct RiscV64Power;

impl PowerOps for RiscV64Power {
    fn init(&self) {
        log_info!("RISC-V: Initializing power management (SBI)");
    }

    fn cpu_idle(&self) {
        // SAFETY: wfi is a standard RISC-V instruction for low-power wait.
        unsafe { asm!("wfi"); }
    }

    fn cpu_sleep(&self) {
        // Use SBI HSM suspend with non-retentive suspend type
        let _ = sbi::hart_suspend(1, 0, 0);
    }

    fn cpu_wakeup(&self, cpu_id: u32) {
        // Use SBI HSM to start a halted hart
        let _ = sbi::hart_start(cpu_id as u64, 0, 0);
    }

    fn system_shutdown(&self) {
        log_info!("RISC-V: System shutdown via SBI");
        let _ = sbi::system_reset(sbi::SBI_RESET_TYPE_SHUTDOWN, 0);
    }

    fn system_reboot(&self) {
        log_info!("RISC-V: System reboot via SBI");
        let _ = sbi::system_reset(sbi::SBI_RESET_TYPE_COLD_REBOOT, 0);
    }

    fn system_suspend(&self) {
        log_info!("RISC-V: System suspend via SBI");
        let _ = sbi::system_reset(sbi::SBI_RESET_TYPE_WARM_REBOOT, 0);
    }
}

/// RISC-V 64 architecture implementation.
pub struct RiscV64Arch;

impl ArchOps for RiscV64Arch {
    fn init(&self) {
        log_info!("RISC-V 64 architecture initialized");

        // Initialize subsystems
        self.irq_controller().init();
        self.timer().init();
        self.power().init();

        // Initialize trap handling
        trap::init_trap();
    }

    fn page_table(&self) -> &'static dyn PageTableOps {
        &mmu::RiscV64PageTable
    }

    fn irq_controller(&self) -> &'static dyn IrqControllerOps {
        &plic::RiscV64IrqController
    }

    fn timer(&self) -> &'static dyn TimerOps {
        &timer::RiscV64Timer
    }

    fn power(&self) -> &'static dyn PowerOps {
        &RiscV64Power
    }

    fn context(&self) -> &'static dyn ContextOps {
        &context::RiscV64Context
    }

    fn enable_irq(&self) {
        // SAFETY: csrs sets the SIE bit in sstatus to enable S-mode interrupts.
        unsafe { asm!("csrs sstatus, 2"); }
    }

    fn disable_irq(&self) {
        // SAFETY: csrc clears the SIE bit in sstatus to disable S-mode interrupts.
        unsafe { asm!("csrc sstatus, 2"); }
    }

    fn cpu_id(&self) -> u32 {
        let hartid: u64;
        // SAFETY: mhartid is a read-only CSR.
        unsafe { asm!("csrr {}, mhartid", out(reg) hartid); }
        hartid as u32
    }

    fn cpu_count(&self) -> u32 {
        // TODO: Read from FDT /cpus node
        1
    }
}

/// Global RISC-V 64 architecture instance.
pub static RISCV64_ARCH: RiscV64Arch = RiscV64Arch;
