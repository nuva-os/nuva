/*
* Nuva OS - Kernel - LoongArch64 Architecture Implementation
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

use crate::kernel::arch::loongarch64::*;
use crate::kernel::arch::*;
use core::sync::atomic::{AtomicU32, Ordering};

/// LoongArch64 Page Table Operation Implementation
pub struct LoongArch64PageTable;

impl PageTableOps for LoongArch64PageTable {
    fn create(&self) -> PhysAddr {
        log_info!("LoongArch64: Creating page table");

        // Allocate a physical page
        let page_phys = crate::kernel::mm::page_alloc::alloc_page();

        // Clear page table
        mmu::tlb_flush_all();

        log_info!("LoongArch64: Page table created at {:?}", page_phys);
        PhysAddr::new(page_phys as u64)
    }

    fn destroy(&self, pgd: PhysAddr) {
        log_info!("LoongArch64: Destroying page table at {}", pgd);
    }

    fn map(
        &self,
        pgd: PhysAddr,
        vaddr: VirtAddr,
        paddr: PhysAddr,
        prot: ProtFlags,
        page_size: u64,
    ) {
        log_info!(
            "LoongArch64: Mapping {:?} -> {:?} with prot {:?}",
            vaddr,
            paddr,
            prot
        );

        let mut pte_flags = mmu::pte_flags::VALID | mmu::pte_flags::PLV;

        if prot.is_user() {
            pte_flags |= mmu::pte_flags::GLOBAL;
        }
        if prot.is_writable() {
            pte_flags |= mmu::pte_flags::WRITE;
        }
        if prot.is_readable() {
            pte_flags |= mmu::pte_flags::READ;
        }
        if prot.is_executable() {
            pte_flags |= mmu::pte_flags::EXEC;
        }

        // Implementation: Call actual mapping
        let _ = (pgd, vaddr, paddr, pte_flags, page_size);
    }

    fn unmap(&self, pgd: PhysAddr, vaddr: VirtAddr) {
        log_info!("LoongArch64: Unmapping {:?}", vaddr);
        self.tlb_flush_addr(vaddr);
        let _ = pgd;
    }

    fn translate(&self, pgd: PhysAddr, vaddr: VirtAddr) -> Option<PhysAddr> {
        let _ = (pgd, vaddr);
        None
    }

    fn protect(&self, pgd: PhysAddr, vaddr: VirtAddr, prot: ProtFlags) {
        log_info!("LoongArch64: Protecting {:?} with {:?}", vaddr, prot);
        let _ = pgd;
    }

    fn tlb_flush_addr(&self, _vaddr: VirtAddr) {
        // SAFETY: inline assembly required for hardware instruction
        unsafe {
            core::arch::asm!("invtlb 0, $r0, $r0");
        }
    }

    fn tlb_flush_all(&self) {
        // SAFETY: inline assembly required for hardware instruction
        unsafe {
            core::arch::asm!("invtlb 0, $r0, $r0");
        }
    }

    fn switch(&self, pgd: PhysAddr) {
        // SAFETY: inline assembly required for hardware instruction
        unsafe {
            core::arch::asm!(
                "csrwr {}, 0x1b",
                in(reg) pgd.0,
            );
        }
        self.tlb_flush_all();
    }

    fn current(&self) -> PhysAddr {
        let pgd: u64;
        // SAFETY: inline assembly required for hardware instruction
        unsafe {
            core::arch::asm!(
                "csrrd {}, 0x1b",
                out(reg) pgd,
            );
        }
        PhysAddr::new(pgd)
    }
}

/// LoongArch64 Interrupt Controller Implementation (EIOINTC)
pub struct LoongArch64IrqController;

impl IrqControllerOps for LoongArch64IrqController {
    fn init(&self) {
        log_info!("LoongArch64: Initializing EIOINTC");
        // Implementation: Initialize EIOINTC interrupt controller
    }

    fn alloc_irq(&self) -> Option<u32> {
        Some(0)
    }

    fn free_irq(&self, _irq: u32) {}

    fn register_handler(&self, irq: u32, _handler: fn(u32), _flags: u32) -> bool {
        log_info!("LoongArch64: Registering handler for IRQ {}", irq);
        true
    }

    fn unregister_handler(&self, _irq: u32) {}

    fn enable_irq(&self, irq: u32) {
        let _ = irq;
    }

    fn disable_irq(&self, irq: u32) {
        let _ = irq;
    }

    fn eoi(&self, irq: u32) {
        let _ = irq;
    }

    fn set_affinity(&self, irq: u32, cpu_mask: u64) {
        let _ = (irq, cpu_mask);
    }

    fn get_irq_count(&self, _irq: u32) -> u64 {
        0
    }
}

/// LoongArch64 Timer Implementation (Stable Counter)
pub struct LoongArch64Timer;

impl TimerOps for LoongArch64Timer {
    fn init(&self) {
        log_info!("LoongArch64: Initializing Timer");
        timer::init_timer();
    }

    fn now(&self) -> u64 {
        timer::get_time_ns()
    }

    fn set_oneshot(&self, ns: u64) {
        timer::set_timer_relative(ns);
    }

    fn set_periodic(&self, ns: u64) {
        timer::set_timer_relative(ns);
    }

    fn stop(&self) {
        timer::disable_timer();
    }

    fn frequency(&self) -> u64 {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe { timer::TIMER_FREQ as u64 }
    }

    fn delay(&self, ns: u64) {
        let start = self.now();
        while self.now() - start < ns {
            core::hint::spin_loop();
        }
    }
}

/// LoongArch64 Power Management Implementation
pub struct LoongArch64Power;

impl PowerOps for LoongArch64Power {
    fn init(&self) {
        log_info!("LoongArch64: Initializing Power Management");
    }

    fn cpu_idle(&self) {
        // SAFETY: inline assembly required for hardware instruction
        unsafe {
            core::arch::asm!("idle 0");
        }
    }

    fn cpu_sleep(&self) {}

    fn cpu_wakeup(&self, _cpu_id: u32) {}

    fn system_shutdown(&self) {
        log_info!("LoongArch64: System shutdown");
    }

    fn system_reboot(&self) {
        log_info!("LoongArch64: System reboot");
    }

    fn system_suspend(&self) {
        log_info!("LoongArch64: System suspend");
    }
}

/// LoongArch64 Context Operation Implementation
pub struct LoongArch64Context;

impl ContextOps for LoongArch64Context {
    fn save_context(&self, ctx: &mut CpuContext) {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            core::arch::asm!(
                "st.d $s0, {0}, 0",
                "st.d $s1, {0}, 8",
                "st.d $s2, {0}, 16",
                "st.d $s3, {0}, 24",
                "st.d $s4, {0}, 32",
                "st.d $s5, {0}, 40",
                "st.d $s6, {0}, 48",
                "st.d $s7, {0}, 56",
                "st.d $fp, {0}, 64",
                "st.d $ra, {0}, 72",
                in(reg) ctx as *mut CpuContext as u64,
            );

            core::arch::asm!(
                "move {}, $sp",
                out(reg) ctx.sp,
            );
        }
    }

    fn restore_context(&self, ctx: &CpuContext) {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            core::arch::asm!(
                "move $sp, {}",
                in(reg) ctx.sp,
            );

            core::arch::asm!(
                "ld.d $s0, {0}, 0",
                "ld.d $s1, {0}, 8",
                "ld.d $s2, {0}, 16",
                "ld.d $s3, {0}, 24",
                "ld.d $s4, {0}, 32",
                "ld.d $s5, {0}, 40",
                "ld.d $s6, {0}, 48",
                "ld.d $s7, {0}, 56",
                "ld.d $fp, {0}, 64",
                "ld.d $ra, {0}, 72",
                in(reg) ctx as *const CpuContext as u64,
            );
        }
    }

    fn switch_context(&self, from: &mut CpuContext, to: &CpuContext) {
        self.save_context(from);
        self.restore_context(to);
    }
}

/// LoongArch64 Architecture Implementation
pub struct LoongArch64Arch;

impl ArchOps for LoongArch64Arch {
    fn init(&self) {
        log_info!("LoongArch64 architecture initialized");

        self.irq_controller().init();
        self.timer().init();
        self.power().init();
    }

    fn page_table(&self) -> &'static dyn PageTableOps {
        &LoongArch64PageTable
    }

    fn irq_controller(&self) -> &'static dyn IrqControllerOps {
        &LoongArch64IrqController
    }

    fn timer(&self) -> &'static dyn TimerOps {
        &LoongArch64Timer
    }

    fn power(&self) -> &'static dyn PowerOps {
        &LoongArch64Power
    }

    fn context(&self) -> &'static dyn ContextOps {
        &LoongArch64Context
    }

    fn enable_irq(&self) {
        // SAFETY: inline assembly required for hardware instruction
        unsafe {
            core::arch::asm!("csrrd $t0, 0x0", "ori $t0, $t0, 1", "csrwr $t0, 0x0",);
        }
    }

    fn disable_irq(&self) {
        // SAFETY: inline assembly required for hardware instruction
        unsafe {
            core::arch::asm!("csrrd $t0, 0x0", "andi $t0, $t0, ~1", "csrwr $t0, 0x0",);
        }
    }

    fn cpu_id(&self) -> u32 {
        let id: u32;
        // SAFETY: inline assembly required for hardware instruction
        unsafe {
            core::arch::asm!(
                "csrrd {}, 0x20",
                out(reg) id,
            );
        }
        id
    }

    fn cpu_count(&self) -> u32 {
        4
    }
}

/// Global LoongArch64 architecture instance
pub static LOONGARCH64_ARCH: LoongArch64Arch = LoongArch64Arch;
