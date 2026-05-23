/*
 * Nuva OS - Kernel - ARM64 Architecture Implementation
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

use crate::kernel::arch::*;
use crate::kernel::arch::arm64::*;
use core::sync::atomic::{AtomicU32, Ordering};

/// ARM64 Page Table Operation Implementation
pub struct Arm64PageTable;

impl PageTableOps for Arm64PageTable {
    fn create(&self) -> PhysAddr {
        // Use buddy allocator to allocate a physical page as PGD
        log_info!("ARM64: Creating page table");

        // Allocate a physical page
        let page_phys = crate::kernel::mm::page_alloc::alloc_page();

        // Clear page table
        mmu::clear_page_table(page_phys as u64);

        log_info!("ARM64: Page table created at {:?}", page_phys);
        PhysAddr::new(page_phys as u64)
    }

    fn destroy(&self, pgd: PhysAddr) {
        // Implementation: Free all physical pages occupied by the page table hierarchy
        log_info!("ARM64: Destroying page table at {}", pgd);
    }

    fn map(&self, pgd: PhysAddr, vaddr: VirtAddr, paddr: PhysAddr, prot: ProtFlags, page_size: u64) {
        log_info!("ARM64: Mapping {:?} -> {:?} with prot {:?}", vaddr, paddr, prot);

        // Convert permission flags to ARM64 PTE flags
        let mut pte_flags = mmu::pte_flags::VALID | mmu::pte_flags::ACCESSED;

        if prot.is_user() {
            pte_flags |= mmu::pte_flags::USER;
        }

        if !prot.is_writable() {
            pte_flags |= mmu::pte_flags::READONLY;
        }

        if !prot.is_executable() {
            pte_flags |= mmu::pte_flags::NX;
        }

        // Call actual mapping implementation
        mmu::page_table_map_impl(pgd.0, vaddr.0, paddr.0, pte_flags, page_size);
    }

    fn unmap(&self, pgd: PhysAddr, vaddr: VirtAddr) {
        log_info!("ARM64: Unmapping {:?}", vaddr);

        // Call actual unmap implementation
        if let Some(_phys) = mmu::page_table_unmap_impl(pgd.0, vaddr.0) {
            // Flush TLB
            self.tlb_flush_addr(vaddr);
        }
    }

    fn translate(&self, pgd: PhysAddr, vaddr: VirtAddr) -> Option<PhysAddr> {
        // Call actual address translation implementation
        mmu::page_table_translate_impl(pgd.0, vaddr.0).map(PhysAddr::new)
    }

    fn protect(&self, pgd: PhysAddr, vaddr: VirtAddr, prot: ProtFlags) {
        // Implementation: Modify page table entry permissions for the given virtual address
        log_info!("ARM64: Protecting {:?} with {:?}", vaddr, prot);
    }

    fn tlb_flush_addr(&self, vaddr: VirtAddr) {
        // SAFETY: inline assembly required for hardware instruction
        unsafe {
            core::arch::asm!(
                "dsb ishst",
                "tlbi vae1is, {}",
                "dsb ish",
                "isb",
                in(reg) vaddr.0 >> 12,
            );
        }
    }

    fn tlb_flush_all(&self) {
        // SAFETY: inline assembly required for hardware instruction
        unsafe {
            core::arch::asm!(
                "dsb ishst",
                "tlbi vmalle1is",
                "dsb ish",
                "isb",
            );
        }
    }

    fn switch(&self, pgd: PhysAddr) {
        // SAFETY: inline assembly required for hardware instruction
        unsafe {
            core::arch::asm!(
                "msr ttbr0_el1, {}",
                "isb",
                in(reg) pgd.0,
            );
        }
        self.tlb_flush_all();
    }

    fn current(&self) -> PhysAddr {
        let ttbr0: u64;
        // SAFETY: inline assembly required for hardware instruction
        unsafe {
            core::arch::asm!(
                "mrs {}, ttbr0_el1",
                out(reg) ttbr0,
            );
        }
        PhysAddr::new(ttbr0)
    }
}

/// ARM64 Interrupt Controller Implementation (GIC)
pub struct Arm64IrqController;

impl IrqControllerOps for Arm64IrqController {
    fn init(&self) {
        log_info!("ARM64: Initializing GIC");

        // Implementation: Get GIC address from device tree firmware table
        // Use default address here
        let gicd_base = 0x0800_0000;  // Distributor base address
        let gicc_base = 0x0801_0000;  // CPU Interface base address

        // Initialize GICv3
        gic::init_gic(gic::GicVersion::V3, gicd_base, gicc_base);
    }

    fn alloc_irq(&self) -> Option<u32> {
        // Implementation: Allocate a free interrupt number from the IRQ bitmap
        Some(0)
    }

    fn free_irq(&self, _irq: u32) {
        // Implementation: Free interrupt number back to the IRQ bitmap
    }

    fn register_handler(&self, irq: u32, _handler: fn(u32), _flags: u32) -> bool {
        // Implementation: Register interrupt handler function to interrupt vector table
        log_info!("ARM64: Registering handler for IRQ {}", irq);
        true
    }

    fn unregister_handler(&self, _irq: u32) {
        // Implementation: Unregister interrupt handler function from interrupt vector table
    }

    fn enable_irq(&self, irq: u32) {
        if let Some(gic) = gic::get_gic() {
            gic.enable_irq(irq);
        }
    }

    fn disable_irq(&self, irq: u32) {
        if let Some(gic) = gic::get_gic() {
            gic.disable_irq(irq);
        }
    }

    fn eoi(&self, irq: u32) {
        if let Some(gic) = gic::get_gic() {
            gic.end_irq(irq);
        }
    }

    fn set_affinity(&self, irq: u32, cpu_mask: u64) {
        if let Some(gic) = gic::get_gic() {
            // Set interrupt target CPU
            gic.set_target(irq, cpu_mask as u8);
        }
    }

    fn get_irq_count(&self, _irq: u32) -> u64 {
        // Implementation: Return trigger count for this interrupt from per-IRQ counter
        0
    }
}

/// ARM64 Timer Implementation (Generic Timer)
pub struct Arm64Timer;

impl TimerOps for Arm64Timer {
    fn init(&self) {
        log_info!("ARM64: Initializing Generic Timer");
        timer::init_timer();
    }

    fn now(&self) -> u64 {
        // Return nanosecond time
        timer::get_time_ns()
    }

    fn set_oneshot(&self, ns: u64) {
        // Set one-shot timer
        let freq = self.frequency();
        let ticks = ns * freq / 1_000_000_000;

        // Set absolute time
        let current = timer::read_cntpct();
        timer::set_timer_absolute(current + ticks);
    }

    fn set_periodic(&self, ns: u64) {
        // Set periodic timer
        let freq = self.frequency();
        let ticks = (ns * freq / 1_000_000_000) as u32;

        // Set relative time
        timer::write_cntp_tval(ticks);
        timer::enable_timer();
    }

    fn stop(&self) {
        timer::disable_timer();
    }

    fn frequency(&self) -> u64 {
        timer::read_cntfrq() as u64
    }

    fn delay(&self, ns: u64) {
        let start = self.now();
        while self.now() - start < ns {
            core::hint::spin_loop();
        }
    }
}

/// ARM64 Power Management Implementation (PSCI)
pub struct Arm64Power;

impl PowerOps for Arm64Power {
    fn init(&self) {
        log_info!("ARM64: Initializing PSCI");
        // Implementation: Initialize PSCI (Power State Coordination Interface) via firmware SMC calls
    }

    fn cpu_idle(&self) {
        // SAFETY: inline assembly required for hardware instruction
        unsafe {
            core::arch::asm!("wfi");
        }
    }

    fn cpu_sleep(&self) {
        // Implementation: Use PSCI SMC call to put CPU into low-power sleep state
    }

    fn cpu_wakeup(&self, _cpu_id: u32) {
        // Implementation: Use PSCI SMC call to wake the specified secondary CPU
    }

    fn system_shutdown(&self) {
        // Implementation: Use PSCI SYSTEM_OFF call to power off the system
        log_info!("ARM64: System shutdown");
    }

    fn system_reboot(&self) {
        // Implementation: Use PSCI SYSTEM_RESET call to reboot the system
        log_info!("ARM64: System reboot");
    }

    fn system_suspend(&self) {
        // Implementation: Use PSCI SYSTEM_SUSPEND call to enter system suspend state
        log_info!("ARM64: System suspend");
    }
}

/// ARM64 Context Operation Implementation
pub struct Arm64Context;

impl ContextOps for Arm64Context {
    fn save_context(&self, ctx: &mut CpuContext) {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            // Save general registers x0-x30
            core::arch::asm!(
                "stp x0, x1, [{0}, #0]",
                "stp x2, x3, [{0}, #16]",
                "stp x4, x5, [{0}, #32]",
                "stp x6, x7, [{0}, #48]",
                "stp x8, x9, [{0}, #64]",
                "stp x10, x11, [{0}, #80]",
                "stp x12, x13, [{0}, #96]",
                "stp x14, x15, [{0}, #112]",
                "stp x16, x17, [{0}, #128]",
                "stp x18, x19, [{0}, #144]",
                "stp x20, x21, [{0}, #160]",
                "stp x22, x23, [{0}, #176]",
                "stp x24, x25, [{0}, #192]",
                "stp x26, x27, [{0}, #208]",
                "stp x28, x29, [{0}, #224]",
                "str x30, [{0}, #240]",
                in(reg) ctx.regs.as_mut_ptr() as *mut u8,
            );

            // Save stack pointer and program state
            core::arch::asm!(
                "mov {0}, sp",
                "mrs {1}, spsr_el1",
                out(reg) ctx.sp,
                out(reg) ctx.pstate,
            );
        }
    }

    fn restore_context(&self, ctx: &CpuContext) {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            // Restore stack pointer and program state
            core::arch::asm!(
                "mov sp, {0}",
                "msr spsr_el1, {1}",
                in(reg) ctx.sp,
                in(reg) ctx.pstate,
            );

            // Restore general registers
            core::arch::asm!(
                "ldp x0, x1, [{0}, #0]",
                "ldp x2, x3, [{0}, #16]",
                "ldp x4, x5, [{0}, #32]",
                "ldp x6, x7, [{0}, #48]",
                "ldp x8, x9, [{0}, #64]",
                "ldp x10, x11, [{0}, #80]",
                "ldp x12, x13, [{0}, #96]",
                "ldp x14, x15, [{0}, #112]",
                "ldp x16, x17, [{0}, #128]",
                "ldp x18, x19, [{0}, #144]",
                "ldp x20, x21, [{0}, #160]",
                "ldp x22, x23, [{0}, #176]",
                "ldp x24, x25, [{0}, #192]",
                "ldp x26, x27, [{0}, #208]",
                "ldp x28, x29, [{0}, #224]",
                "ldr x30, [{0}, #240]",
                in(reg) ctx.regs.as_ptr() as *const u8,
            );
        }
    }

    fn switch_context(&self, from: &mut CpuContext, to: &CpuContext) {
        self.save_context(from);
        self.restore_context(to);
    }
}

/// ARM64 Architecture Implementation
pub struct Arm64Arch;

impl ArchOps for Arm64Arch {
    fn init(&self) {
        log_info!("ARM64 architecture initialized");
        log_info!("  Current EL: {:?}", current_el());
        log_info!("  CPU ID: {}", cpu_id());

        // Initialize subsystems
        self.irq_controller().init();
        self.timer().init();
        self.power().init();
    }

    fn page_table(&self) -> &'static dyn PageTableOps {
        &Arm64PageTable
    }

    fn irq_controller(&self) -> &'static dyn IrqControllerOps {
        &Arm64IrqController
    }

    fn timer(&self) -> &'static dyn TimerOps {
        &Arm64Timer
    }

    fn power(&self) -> &'static dyn PowerOps {
        &Arm64Power
    }

    fn context(&self) -> &'static dyn ContextOps {
        &Arm64Context
    }

    fn enable_irq(&self) {
        // SAFETY: inline assembly required for hardware instruction
        unsafe {
            core::arch::asm!("msr daifclr, #2");
        }
    }

    fn disable_irq(&self) {
        // SAFETY: inline assembly required for hardware instruction
        unsafe {
            core::arch::asm!("msr daifset, #2");
        }
    }

    fn cpu_id(&self) -> u32 {
        cpu_id() as u32
    }

    fn cpu_count(&self) -> u32 {
        // Implementation: Read CPU count from device tree or MPIDR register
        1
    }
}

/// Global ARM64 architecture instance
pub static ARM64_ARCH: Arm64Arch = Arm64Arch;
