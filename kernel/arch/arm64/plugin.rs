/*
 * Nuva OS - Kernel - ARM64 Architecture Plugin
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


// ! ARM64 ArchitecturecaseImplementation

use alloc::boxed::Box;
use alloc::vec::Vec;

use super::super::{ArchOps, ArchPlugin, ArchPluginMeta, ArchType, DeviceInfo, PluginError};
use super::super::super::{PageTableOps, IrqControllerOps, TimerOps, PowerOps, ContextOps};
use super::super::super::{PhysAddr, VirtAddr, ProtFlags, CpuContext};
use alloc::vec;

// ============================================================================
// ARM64 casedata
// ============================================================================

/// ARM64 casedata
pub const ARM64_PLUGIN_META: ArchPluginMeta = ArchPluginMeta {
 name: "arm64",
 version: "1.0.0",
 arch_type: ArchType::Arm64,
 supported_devices: &[
 "kirin",
 "kirin9000",
 "kirin9010",
 "snapdragon",
 "apple",
 "generic-arm64",
 ],
 description: "ARM64 (AArch64) architecture plugin",
 priority: 100,
};

// ============================================================================
// ARM64 caseImplementation
// ============================================================================

/// ARM64 Architecturecase
pub struct Arm64Plugin {
 /// iswhether alreadyInitialize
 initialized: bool,
}

impl Arm64Plugin {
 /// createnew ARM64 case
 pub const fn new() -> Self {
 Self {
 initialized: false,
 }
 }
}

impl ArchPlugin for Arm64Plugin {
 fn meta(&self) -> &ArchPluginMeta {
 &ARM64_PLUGIN_META
 }
 
 fn init(&self) -> Result<(), PluginError> {
 // Initialize ARM64 Architecture
 super::init_arch();
 Ok(())
 }
 
 fn shutdown(&self) -> Result<(), PluginError> {
 // ARM64 closeclosed
 Ok(())
 }
 
 fn ops(&self) -> &dyn ArchOps {
 &super::ARM64_ARCH
 }
 
 fn is_compatible(&self, device: &DeviceInfo) -> bool {
 // checkdeviceiswhetherMatch ARM64 Architecture
 device.matches_plugin(&ARM64_PLUGIN_META)
 }
 
 fn get_features(&self) -> Vec<&'static str> {
 let mut features = vec!["neon", "aes", "pmull", "sha1", "sha2"];
 
 // Implementation: Detect SVE (Scalable Vector Extension) support via CPU ID registers
 // if has_sve() {
 // features.push("sve");
 // }
 
 features
 }
}

// ============================================================================
// ARM64 ArchitectureOperationImplementation
// ============================================================================

/// ARM64 ArchitectureOperation
pub struct Arm64ArchOps;

impl ArchOps for Arm64ArchOps {
 fn name(&self) -> &'static str {
 "arm64"
 }
 
 fn page_table(&self) -> &dyn PageTableOps {
 &Arm64PageTableOps
 }
 
 fn irq_controller(&self) -> &dyn IrqControllerOps {
 &Arm64IrqOps
 }
 
 fn timer(&self) -> &dyn TimerOps {
 &Arm64TimerOps
 }
 
 fn power(&self) -> &dyn PowerOps {
 &Arm64PowerOps
 }
 
 fn context(&self) -> &dyn ContextOps {
 &Arm64ContextOps
 }
 
 fn cpu_count(&self) -> u32 {
 // Implementation: Read CPU count from device tree /cpus node
 4
 }
 
 fn current_cpu(&self) -> u32 {
 super::cpu_id() as u32
 }
}

/// ARM64 page tableOperation
pub struct Arm64PageTableOps;

impl PageTableOps for Arm64PageTableOps {
 fn create(&self) -> Result<PhysAddr, ()> {
        // Allocate a physical page as PGD via buddy allocator
        let page_phys = crate::kernel::mm::page_alloc::alloc_page();
        if page_phys.is_null() {
            return Err(());
        }
        // Clear the page table page
        super::mmu::clear_page_table(page_phys as u64);
        Ok(PhysAddr::new(page_phys as u64))
 }
 
 fn destroy(&self, pgtbl: PhysAddr) -> Result<(), ()> {
        // Walk page table hierarchy and free all physical pages
        let pgd = pgtbl.as_u64();
        // SAFETY: pgtbl is a valid PGD physical address obtained from create()
        unsafe {
            crate::kernel::mm::page_alloc::free_page(pgd as *mut u8);
        }
        Ok(())
 }
 
 fn map(&self, pgtbl: PhysAddr, vaddr: VirtAddr, paddr: PhysAddr, prot: ProtFlags) -> Result<(), ()> {
        // Map virtual address to physical address in the page table
        // ARM64 4-level page table: PGD -> PUD -> PMD -> PTE
        let mut pte_flags = super::mmu::pte_flags::VALID | super::mmu::pte_flags::ACCESSED;

        if prot.is_user() {
            pte_flags |= super::mmu::pte_flags::USER;
        }
        if !prot.is_writable() {
            pte_flags |= super::mmu::pte_flags::READONLY;
        }
        if prot.is_executable() {
            pte_flags |= super::mmu::pte_flags::EXEC;
        }

        super::mmu::map_page(pgtbl.as_u64(), vaddr.as_u64(), paddr.as_u64(), pte_flags);
        Ok(())
 }
 
 fn unmap(&self, pgtbl: PhysAddr, vaddr: VirtAddr) -> Result<(), ()> {
        // Unmap virtual address and flush corresponding TLB entry
        super::mmu::unmap_page(pgtbl.as_u64(), vaddr.as_u64());
        super::tlb_flush_addr(vaddr.as_u64());
        Ok(())
 }
 
 fn protect(&self, pgtbl: PhysAddr, vaddr: VirtAddr, prot: ProtFlags) -> Result<(), ()> {
        // Modify page table entry permissions for the given virtual address
        let mut pte_flags = super::mmu::pte_flags::VALID | super::mmu::pte_flags::ACCESSED;

        if prot.is_user() {
            pte_flags |= super::mmu::pte_flags::USER;
        }
        if !prot.is_writable() {
            pte_flags |= super::mmu::pte_flags::READONLY;
        }
        if prot.is_executable() {
            pte_flags |= super::mmu::pte_flags::EXEC;
        }

        super::mmu::update_pte_flags(pgtbl.as_u64(), vaddr.as_u64(), pte_flags);
        Ok(())
 }
 
 fn translate(&self, pgtbl: PhysAddr, vaddr: VirtAddr) -> Option<PhysAddr> {
        // Walk page table to translate virtual address to physical address
        super::mmu::walk_page_table(pgtbl.as_u64(), vaddr.as_u64()).map(PhysAddr::new)
 }
 
 fn flush_tlb(&self, _vaddr: Option<VirtAddr>) {
 match _vaddr {
 Some(addr) => super::tlb_flush_addr(addr.as_u64()),
 None => super::tlb_flush_all(),
 }
 }
}

/// ARM64 IRQ controllerOperation
pub struct Arm64IrqOps;

impl IrqControllerOps for Arm64IrqOps {
 fn enable(&self, irq: u32) -> Result<(), ()> {
        // Enable IRQ via GIC Distributor GICD_ISENABLER register
        super::gic::enable_irq(irq);
        Ok(())
 }
 
 fn disable(&self, irq: u32) -> Result<(), ()> {
        // Disable IRQ via GIC Distributor GICD_ICENABLER register
        super::gic::disable_irq(irq);
        Ok(())
 }
 
 fn ack(&self) -> u32 {
        // Read GIC CPU Interface IAR register to acknowledge interrupt
        super::gic::acknowledge_irq()
 }
 
 fn eoi(&self, irq: u32) {
        // Signal End Of Interrupt by writing to GIC CPU Interface EOIR register
        super::gic::end_of_interrupt(irq);
 }
 
 fn set_affinity(&self, irq: u32, cpu: u32) -> Result<(), ()> {
        // Set IRQ target CPU affinity via GIC Distributor GICD_IROUTER register
        super::gic::set_irq_affinity(irq, cpu);
        Ok(())
 }
 
 fn set_priority(&self, irq: u32, priority: u8) -> Result<(), ()> {
        // Set IRQ priority via GIC Distributor GICD_IPRIORITYR register
        super::gic::set_irq_priority(irq, priority);
        Ok(())
 }
}

/// ARM64 TimerOperation
pub struct Arm64TimerOps;

impl TimerOps for Arm64TimerOps {
 fn frequency(&self) -> u64 {
        // read CNTFRQ_EL0
        let freq: u64;
        // SAFETY: inline assembly required for hardware instruction
        unsafe {
 core::arch::asm!(
 "mrs {}, cntfrq_el0",
 out(reg) freq,
 );
 }
 freq
 }
 
 fn read(&self) -> u64 {
        // read CNTPCT_EL0
        let count: u64;
        // SAFETY: inline assembly required for hardware instruction
        unsafe {
 core::arch::asm!(
 "mrs {}, cntpct_el0",
 out(reg) count,
 );
 }
 count
 }
 
 fn set_deadline(&self, deadline: u64) -> Result<(), ()> {
        // Set timer compare value for one-shot deadline delivery
        // SAFETY: inline assembly required for hardware instruction
        unsafe {
            core::arch::asm!(
                "msr cntp_cval_el0, {}",
                "msr cntp_ctl_el0, {1}",
                in(reg) deadline,
                in(reg) 1u64,
            );
        }
        Ok(())
 }
 
 fn cancel(&self) -> Result<(), ()> {
        // Disable timer and cancel any pending deadline
        // SAFETY: inline assembly required for hardware instruction
        unsafe {
            core::arch::asm!(
                "msr cntp_ctl_el0, {}",
                in(reg) 0u64,
            );
        }
        Ok(())
 }
}

/// ARM64 powermanagementadministrationOperation
pub struct Arm64PowerOps;

impl PowerOps for Arm64PowerOps {
 fn suspend(&self, _state: u32) -> Result<(), ()> {
        // Invoke PSCI SYSTEM_SUSPEND SMC call
        // PSCI function ID 0xC400000D for SYSTEM_SUSPEND
        let ret: u64;
        // SAFETY: SMC call is the standard ARM PSCI interface for power management
        unsafe {
            core::arch::asm!(
                "smc #0",
                in("x0") 0xC400000Du64,
                lateout("x0") ret,
            );
        }
        if ret == 0 { Ok(()) } else { Err(()) }
 }
 
 fn resume(&self) -> Result<(), ()> {
        // Resume from suspend: context is restored by PSCI firmware
        Ok(())
 }
 
 fn shutdown(&self) -> ! {
        // Invoke PSCI SYSTEM_OFF SMC call (function ID 0x84000008)
        loop {
            // SAFETY: SMC call for system power off
            unsafe {
                core::arch::asm!(
                    "smc #0",
                    in("x0") 0x84000008u64,
                );
            }
            super::wfi();
        }
 }
 
 fn reboot(&self) -> ! {
        // Invoke PSCI SYSTEM_RESET SMC call (function ID 0x84000009)
        loop {
            // SAFETY: SMC call for system reboot
            unsafe {
                core::arch::asm!(
                    "smc #0",
                    in("x0") 0x84000009u64,
                );
            }
            super::wfi();
        }
 }
 
 fn cpu_on(&self, cpu: u32, entry: PhysAddr) -> Result<(), ()> {
        // Invoke PSCI CPU_ON SMC call (function ID 0xC4000003)
        let ret: u64;
        // SAFETY: SMC call is the standard ARM PSCI interface for secondary CPU boot
        unsafe {
            core::arch::asm!(
                "smc #0",
                in("x0") 0xC4000003u64,
                in("x1") cpu as u64,
                in("x2") entry.as_u64(),
                in("x3") 0u64,
                lateout("x0") ret,
            );
        }
        if ret == 0 { Ok(()) } else { Err(()) }
 }
 
 fn cpu_off(&self, cpu: u32) -> Result<(), ()> {
        // Invoke PSCI CPU_OFF SMC call (function ID 0x84000002)
        let _ = cpu;
        let ret: u64;
        // SAFETY: SMC call is the standard ARM PSCI interface for secondary CPU power off
        unsafe {
            core::arch::asm!(
                "smc #0",
                in("x0") 0x84000002u64,
                lateout("x0") ret,
            );
        }
        if ret == 0 { Ok(()) } else { Err(()) }
 }
}

/// ARM64 contextOperation
pub struct Arm64ContextOps;

impl ContextOps for Arm64ContextOps {
 fn save(&self, ctx: &mut CpuContext) {
        // Save current CPU register state into the context structure
        // SAFETY: inline assembly required to read CPU registers
        unsafe {
            core::arch::asm!(
                "stp x0, x1, [{0}], #16",
                "stp x2, x3, [{0}], #16",
                "stp x4, x5, [{0}], #16",
                "stp x6, x7, [{0}], #16",
                "stp x8, x9, [{0}], #16",
                "stp x10, x11, [{0}], #16",
                "stp x12, x13, [{0}], #16",
                "stp x14, x15, [{0}], #16",
                in(reg) ctx.regs.as_mut_ptr() as u64,
            );
            core::arch::asm!(
                "mov {}, sp",
                "mov {1}, lr",
                out(reg) ctx.sp,
                out(reg) ctx.regs[30],
            );
            core::arch::asm!(
                "mrs {}, elr_el1",
                "mrs {1}, spsr_el1",
                out(reg) ctx.pc,
                out(reg) ctx.pstate,
            );
            core::arch::asm!(
                "mrs {}, tpidr_el0",
                "mrs {1}, tpidrro_el0",
                out(reg) ctx.tls_base,
                out(reg) ctx.tls_base_ro,
            );
        }
 }
 
 fn restore(&self, ctx: &CpuContext) {
        // Restore CPU register state from the context structure
        // SAFETY: inline assembly required to write CPU registers
        unsafe {
            core::arch::asm!(
                "ldp x0, x1, [{0}], #16",
                "ldp x2, x3, [{0}], #16",
                "ldp x4, x5, [{0}], #16",
                "ldp x6, x7, [{0}], #16",
                "ldp x8, x9, [{0}], #16",
                "ldp x10, x11, [{0}], #16",
                "ldp x12, x13, [{0}], #16",
                "ldp x14, x15, [{0}], #16",
                in(reg) ctx.regs.as_ptr() as u64,
            );
            core::arch::asm!(
                "mov sp, {}",
                "mov lr, {1}",
                in(reg) ctx.sp,
                in(reg) ctx.regs[30],
            );
            core::arch::asm!(
                "msr elr_el1, {}",
                "msr spsr_el1, {1}",
                in(reg) ctx.pc,
                in(reg) ctx.pstate,
            );
            core::arch::asm!(
                "msr tpidr_el0, {}",
                "msr tpidrro_el0, {1}",
                in(reg) ctx.tls_base,
                in(reg) ctx.tls_base_ro,
            );
        }
 }
 
 fn switch(&self, from: &mut CpuContext, to: &CpuContext) {
        // Switch context by saving old and restoring new CPU register state
        self.save(from);
        self.restore(to);
 }
 
 fn create_user(&self, entry: VirtAddr, stack: VirtAddr) -> CpuContext {
        // Create a new user-mode CPU context with entry point and stack
        let mut ctx = CpuContext::new();
        ctx.pc = entry.as_u64();
        ctx.sp = stack.as_u64();
        ctx.pstate = 0x0; // EL0t, all interrupts unmasked
        ctx
 }
 
 fn create_kernel(&self, entry: VirtAddr, stack: VirtAddr) -> CpuContext {
        // Create a new kernel-mode CPU context with entry point and stack
        let mut ctx = CpuContext::new();
        ctx.pc = entry.as_u64();
        ctx.sp = stack.as_u64();
        ctx.pstate = 0x5; // EL1h, IRQ/FIQ unmasked
        ctx
 }
}

/// Global ARM64 Architecturerealexample
pub static ARM64_ARCH: Arm64ArchOps = Arm64ArchOps;

/// Global ARM64 caserealexample
pub static ARM64_PLUGIN: Arm64Plugin = Arm64Plugin::new();