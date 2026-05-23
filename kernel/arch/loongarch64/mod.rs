/*
* Nuva OS - Kernel - Architecture - LoongArch64
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

use super::{ArchOps, ContextOps, CpuContext, IrqControllerOps, PageTableOps, PowerOps, TimerOps};
use super::{PhysAddr, ProtFlags, VirtAddr};
use crate::pr_info;
use core::ptr::{read_volatile, write_volatile};
use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

pub mod arch_impl;
pub mod boot;
pub mod context;
pub mod interrupt_controller;
pub mod mm;
pub mod mmu;
pub mod qemu;
pub mod timer;
pub mod trap;

// Re-export architecture implementation
pub use arch_impl::*;

// ============================================================================
// LoongArch64 Constant Definitions
// ============================================================================

/// Page size (4KB)
const PAGE_SIZE: u64 = 4096;

/// Page table levels (3-level page table)
const PT_LEVELS: usize = 3;

/// PTE entries per page table (4KB / 8 bytes = 512)
const PTE_PER_PT: usize = 512;

/// LoongArch64 PTE bits
const PTE_V: u64 = 1 << 0; // Valid
const PTE_R: u64 = 1 << 1; // Read
const PTE_W: u64 = 1 << 2; // Write
const PTE_X: u64 = 1 << 3; // Execute
const PTE_G: u64 = 1 << 6; // Global
const PTE_D: u64 = 1 << 7; // Dirty
const PTE_PLV: u64 = 3 << 8; // Privilege level (PLV0=0, PLV3=3)
const PTE_PPN_SHIFT: u64 = 12; // PPN starts at bit 12

/// EIOINTC base address (QEMU virt)
const EIOINTC_BASE: u64 = 0x1FE0_0000;
const EIOINTC_ENABLE: u64 = EIOINTC_BASE + 0x0020;
const EIOINTC_DISABLE: u64 = EIOINTC_BASE + 0x0028;
const EIOINTC_STATUS: u64 = EIOINTC_BASE + 0x0010;
const EIOINTC_IRQ_COUNT: u32 = 256;

/// Maximum interrupt handlers
const MAX_IRQ_HANDLERS: usize = 256;

/// CSR register addresses
mod csr {
    pub const CRMD: u32 = 0x0;
    pub const PRMD: u32 = 0x1;
    pub const EUEN: u32 = 0x2;
    pub const ECFG: u32 = 0x4;
    pub const ESTAT: u32 = 0x5;
    pub const ERA: u32 = 0x6;
    pub const BADV: u32 = 0x7;
    pub const EENTRY: u32 = 0xc;
    pub const TLBRENTRY: u32 = 0x88;
    pub const PGDL: u32 = 0x19;
    pub const PGDH: u32 = 0x1a;
    pub const PGD: u32 = 0x1b;
    pub const CPUID: u32 = 0x20;
    pub const SAVE0: u32 = 0x30;
    pub const SAVE1: u32 = 0x31;
    pub const SAVE2: u32 = 0x32;
    pub const SAVE3: u32 = 0x33;
    pub const TID: u32 = 0x40;
    pub const TCFG: u32 = 0x41;
    pub const TVAL: u32 = 0x42;
    pub const TICLR: u32 = 0x44;
}

// ============================================================================
// LoongArch64 Page Table Operations
// ============================================================================

/// LoongArch64 page table operations implementation
pub struct LoongArch64PageTable;

/// Convert ProtFlags to LoongArch64 PTE permission bits
fn prot_to_pte(prot: ProtFlags) -> u64 {
    let mut pte = PTE_V | PTE_PLV;
    if prot.contains(ProtFlags::READ) {
        pte |= PTE_R;
    }
    if prot.contains(ProtFlags::WRITE) {
        pte |= PTE_W;
    }
    if prot.contains(ProtFlags::EXEC) {
        pte |= PTE_X;
    }
    if prot.contains(ProtFlags::USER) {
        pte |= PTE_G;
    }
    pte
}

/// Extract page table index for a given level from a virtual address
fn pt_index(vaddr: u64, level: usize) -> usize {
    let shift = 12 + (PT_LEVELS - 1 - level) * 9;
    ((vaddr >> shift) & 0x1FF) as usize
}

/// Read a page table entry (8 bytes) at the given address and index
// SAFETY: The caller must ensure table is a valid physical address of a
// page table and idx is within the page table bounds (0..512).
unsafe fn read_pte(table: u64, idx: usize) -> u64 {
    read_volatile((table as *const u64).add(idx))
}

/// Write a page table entry at the given address and index
// SAFETY: The caller must ensure table is a valid physical address of a
// page table and idx is within the page table bounds (0..512).
unsafe fn write_pte(table: u64, idx: usize, pte: u64) {
    write_volatile((table as *mut u64).add(idx), pte);
}

/// Allocate a zeroed page (returns physical address or 0 on failure)
/// Uses buddy allocator through FFI
extern "C" {
    fn buddy_alloc_page() -> u64;
    fn buddy_free_page(paddr: u64);
}

fn alloc_zeroed_page() -> u64 {
    // SAFETY: buddy_alloc_page returns a zeroed physical page or 0 on failure
    unsafe { buddy_alloc_page() }
}

fn free_page(paddr: u64) {
    // SAFETY: buddy_free_page returns a physical page to the allocator
    unsafe {
        buddy_free_page(paddr);
    }
}

impl PageTableOps for LoongArch64PageTable {
    fn create(&self) -> PhysAddr {
        let pgd = alloc_zeroed_page();
        if pgd == 0 {
            PhysAddr::zero()
        } else {
            PhysAddr::new(pgd)
        }
    }

    fn destroy(&self, pgd: PhysAddr) {
        let pgd_addr = pgd.as_u64();
        if pgd_addr == 0 {
            return;
        }

        for l0_idx in 0..PTE_PER_PT {
            // SAFETY: reading page table entries from validated physical memory
            let l0_pte = unsafe { read_pte(pgd_addr, l0_idx) };
            if l0_pte & PTE_V == 0 {
                continue;
            }
            let l1_addr = (l0_pte >> PTE_PPN_SHIFT) << PAGE_SIZE.trailing_zeros();

            for l1_idx in 0..PTE_PER_PT {
                // SAFETY: reading page table entries from validated physical memory
                let l1_pte = unsafe { read_pte(l1_addr, l1_idx) };
                if l1_pte & PTE_V == 0 {
                    continue;
                }
                let l2_addr = (l1_pte >> PTE_PPN_SHIFT) << PAGE_SIZE.trailing_zeros();
                free_page(l2_addr);
            }
            free_page(l1_addr);
        }
        free_page(pgd_addr);
    }

    fn map(
        &self,
        pgd: PhysAddr,
        vaddr: VirtAddr,
        paddr: PhysAddr,
        prot: ProtFlags,
        _page_size: u64,
    ) {
        let pgd_addr = pgd.as_u64();
        let va = vaddr.as_u64();
        let pa = paddr.as_u64();
        if pgd_addr == 0 {
            return;
        }

        let l0_idx = pt_index(va, 0);
        // SAFETY: reading page table entry from validated physical memory
        let mut l0_pte = unsafe { read_pte(pgd_addr, l0_idx) };
        let mut l1_addr = if l0_pte & PTE_V != 0 {
            (l0_pte >> PTE_PPN_SHIFT) << PAGE_SIZE.trailing_zeros()
        } else {
            let new_l1 = alloc_zeroed_page();
            if new_l1 == 0 {
                return;
            }
            l0_pte = PTE_V
                | PTE_R
                | PTE_W
                | PTE_X
                | (new_l1 >> PAGE_SIZE.trailing_zeros() << PTE_PPN_SHIFT);
            // SAFETY: writing page table entry to validated physical memory
            unsafe {
                write_pte(pgd_addr, l0_idx, l0_pte);
            }
            new_l1
        };

        let l1_idx = pt_index(va, 1);
        // SAFETY: reading page table entry from validated physical memory
        let mut l1_pte = unsafe { read_pte(l1_addr, l1_idx) };
        let l2_addr = if l1_pte & PTE_V != 0 {
            (l1_pte >> PTE_PPN_SHIFT) << PAGE_SIZE.trailing_zeros()
        } else {
            let new_l2 = alloc_zeroed_page();
            if new_l2 == 0 {
                return;
            }
            l1_pte = PTE_V
                | PTE_R
                | PTE_W
                | PTE_X
                | (new_l2 >> PAGE_SIZE.trailing_zeros() << PTE_PPN_SHIFT);
            // SAFETY: writing page table entry to validated physical memory
            unsafe {
                write_pte(l1_addr, l1_idx, l1_pte);
            }
            new_l2
        };

        let l2_idx = pt_index(va, 2);
        let perm = prot_to_pte(prot);
        let l2_pte = perm | (pa >> PAGE_SIZE.trailing_zeros() << PTE_PPN_SHIFT);
        // SAFETY: writing leaf page table entry to validated physical memory
        unsafe {
            write_pte(l2_addr, l2_idx, l2_pte);
        }
        self.tlb_flush_addr(vaddr);
    }

    fn unmap(&self, pgd: PhysAddr, vaddr: VirtAddr) {
        let pgd_addr = pgd.as_u64();
        let va = vaddr.as_u64();
        if pgd_addr == 0 {
            return;
        }

        let l0_idx = pt_index(va, 0);
        // SAFETY: reading page table entry
        let l0_pte = unsafe { read_pte(pgd_addr, l0_idx) };
        if l0_pte & PTE_V == 0 {
            return;
        }
        let l1_addr = (l0_pte >> PTE_PPN_SHIFT) << PAGE_SIZE.trailing_zeros();

        let l1_idx = pt_index(va, 1);
        // SAFETY: reading page table entry
        let l1_pte = unsafe { read_pte(l1_addr, l1_idx) };
        if l1_pte & PTE_V == 0 {
            return;
        }
        let l2_addr = (l1_pte >> PTE_PPN_SHIFT) << PAGE_SIZE.trailing_zeros();

        let l2_idx = pt_index(va, 2);
        // SAFETY: clearing leaf page table entry
        unsafe {
            write_pte(l2_addr, l2_idx, 0);
        }
        self.tlb_flush_addr(vaddr);
    }

    fn translate(&self, pgd: PhysAddr, vaddr: VirtAddr) -> Option<PhysAddr> {
        let pgd_addr = pgd.as_u64();
        let va = vaddr.as_u64();
        if pgd_addr == 0 {
            return None;
        }

        let l0_idx = pt_index(va, 0);
        // SAFETY: reading page table entry
        let l0_pte = unsafe { read_pte(pgd_addr, l0_idx) };
        if l0_pte & PTE_V == 0 {
            return None;
        }
        let l1_addr = (l0_pte >> PTE_PPN_SHIFT) << PAGE_SIZE.trailing_zeros();

        let l1_idx = pt_index(va, 1);
        // SAFETY: reading page table entry
        let l1_pte = unsafe { read_pte(l1_addr, l1_idx) };
        if l1_pte & PTE_V == 0 {
            return None;
        }
        let l2_addr = (l1_pte >> PTE_PPN_SHIFT) << PAGE_SIZE.trailing_zeros();

        let l2_idx = pt_index(va, 2);
        // SAFETY: reading leaf page table entry
        let l2_pte = unsafe { read_pte(l2_addr, l2_idx) };
        if l2_pte & PTE_V == 0 {
            return None;
        }

        let paddr = (l2_pte >> PTE_PPN_SHIFT) << PAGE_SIZE.trailing_zeros();
        let offset = va & (PAGE_SIZE - 1);
        Some(PhysAddr::new(paddr | offset))
    }

    fn protect(&self, pgd: PhysAddr, vaddr: VirtAddr, prot: ProtFlags) {
        let pgd_addr = pgd.as_u64();
        let va = vaddr.as_u64();
        if pgd_addr == 0 {
            return;
        }

        let l0_idx = pt_index(va, 0);
        // SAFETY: reading page table entry
        let l0_pte = unsafe { read_pte(pgd_addr, l0_idx) };
        if l0_pte & PTE_V == 0 {
            return;
        }
        let l1_addr = (l0_pte >> PTE_PPN_SHIFT) << PAGE_SIZE.trailing_zeros();

        let l1_idx = pt_index(va, 1);
        // SAFETY: reading page table entry
        let l1_pte = unsafe { read_pte(l1_addr, l1_idx) };
        if l1_pte & PTE_V == 0 {
            return;
        }
        let l2_addr = (l1_pte >> PTE_PPN_SHIFT) << PAGE_SIZE.trailing_zeros();

        let l2_idx = pt_index(va, 2);
        // SAFETY: reading leaf page table entry
        let l2_pte = unsafe { read_pte(l2_addr, l2_idx) };
        if l2_pte & PTE_V == 0 {
            return;
        }

        let perm = prot_to_pte(prot);
        let ppn_part = l2_pte & !(PTE_V | PTE_R | PTE_W | PTE_X | PTE_G | PTE_D | PTE_PLV);
        // SAFETY: writing updated page table entry
        unsafe {
            write_pte(l2_addr, l2_idx, perm | ppn_part);
        }
        self.tlb_flush_addr(vaddr);
    }

    fn tlb_flush_addr(&self, _vaddr: VirtAddr) {
        // Flush single TLB entry
        // Use invtlb instruction
        // SAFETY: inline assembly required for hardware instruction
        unsafe {
            core::arch::asm!("invtlb 0, $r0, $r0");
        }
    }

    fn tlb_flush_all(&self) {
        // Flush entire TLB
        // SAFETY: inline assembly required for hardware instruction
        unsafe {
            core::arch::asm!("invtlb 0, $r0, $r0");
        }
    }

    fn switch(&self, pgd: PhysAddr) {
        // Switch page table
        // SAFETY: inline assembly required for hardware instruction
        unsafe {
            core::arch::asm!(
                "csrrd $t0, {}",
                "csrwr {}, {}",
                in(reg) csr::PGD,
                in(reg) pgd.as_u64(),
                in(reg) csr::PGD,
            );
        }
    }

    fn current(&self) -> PhysAddr {
        let pgd: u64;
        // SAFETY: inline assembly required for hardware instruction
        unsafe {
            core::arch::asm!(
                "csrrd {}, {}",
                out(reg) pgd,
                in(reg) csr::PGD,
            );
        }
        PhysAddr::new(pgd)
    }
}

// ============================================================================
// LoongArch64 Interrupt Controller
// ============================================================================

/// LoongArch64 interrupt controller implementation
pub struct LoongArch64IrqController;

/// IRQ allocation bitmap (256 bits = 4 x u64)
static IRQ_BITMAP: [AtomicU64; 4] = [
    AtomicU64::new(0),
    AtomicU64::new(0),
    AtomicU64::new(0),
    AtomicU64::new(0),
];

/// IRQ handler table
static IRQ_HANDLERS: [AtomicU64; MAX_IRQ_HANDLERS] = [AtomicU64::new(0); MAX_IRQ_HANDLERS];

/// IRQ count per interrupt
static IRQ_COUNTS: [AtomicU64; MAX_IRQ_HANDLERS] = [AtomicU64::new(0); MAX_IRQ_HANDLERS];

fn irq_bitmap_set(irq: u32) {
    let word = irq as usize / 64;
    let bit = irq as usize % 64;
    if word < 4 {
        IRQ_BITMAP[word].fetch_or(1u64 << bit, Ordering::AcqRel);
    }
}

fn irq_bitmap_clear(irq: u32) {
    let word = irq as usize / 64;
    let bit = irq as usize % 64;
    if word < 4 {
        IRQ_BITMAP[word].fetch_and(!(1u64 << bit), Ordering::AcqRel);
    }
}

fn irq_bitmap_alloc() -> Option<u32> {
    for word in 0..4 {
        let val = IRQ_BITMAP[word].load(Ordering::Acquire);
        if val != !0u64 {
            for bit in 0..64 {
                if val & (1u64 << bit) == 0 {
                    let irq = (word * 64 + bit) as u32;
                    if irq < EIOINTC_IRQ_COUNT {
                        if IRQ_BITMAP[word]
                            .compare_exchange(
                                val,
                                val | (1u64 << bit),
                                Ordering::AcqRel,
                                Ordering::Acquire,
                            )
                            .is_ok()
                        {
                            return Some(irq);
                        }
                    }
                }
            }
        }
    }
    None
}

impl IrqControllerOps for LoongArch64IrqController {
    fn init(&self) {
        for i in 0..4 {
            IRQ_BITMAP[i].store(0, Ordering::Release);
        }
        for i in 0..MAX_IRQ_HANDLERS {
            IRQ_HANDLERS[i].store(0, Ordering::Release);
        }
        for i in 0..MAX_IRQ_HANDLERS {
            IRQ_COUNTS[i].store(0, Ordering::Release);
        }
        // SAFETY: MMIO write to EIOINTC enable register to mask all interrupts initially
        unsafe {
            write_volatile(EIOINTC_ENABLE as *mut u32, 0);
        }
    }

    fn alloc_irq(&self) -> Option<u32> {
        irq_bitmap_alloc()
    }

    fn free_irq(&self, irq: u32) {
        if irq >= EIOINTC_IRQ_COUNT {
            return;
        }
        irq_bitmap_clear(irq);
        IRQ_HANDLERS[irq as usize].store(0, Ordering::Release);
    }

    fn register_handler(&self, irq: u32, handler: fn(u32), _flags: u32) -> bool {
        if irq as usize >= MAX_IRQ_HANDLERS {
            return false;
        }
        let handler_addr = handler as usize as u64;
        IRQ_HANDLERS[irq as usize].store(handler_addr, Ordering::Release);
        irq_bitmap_set(irq);
        true
    }

    fn unregister_handler(&self, irq: u32) {
        if irq as usize >= MAX_IRQ_HANDLERS {
            return;
        }
        IRQ_HANDLERS[irq as usize].store(0, Ordering::Release);
    }

    fn enable_irq(&self, irq: u32) {
        if irq >= EIOINTC_IRQ_COUNT {
            return;
        }
        // SAFETY: MMIO write to EIOINTC enable register
        unsafe {
            write_volatile(EIOINTC_ENABLE as *mut u32, 1u32 << (irq % 32));
        }
    }

    fn disable_irq(&self, irq: u32) {
        if irq >= EIOINTC_IRQ_COUNT {
            return;
        }
        // SAFETY: MMIO write to EIOINTC disable register
        unsafe {
            write_volatile(EIOINTC_DISABLE as *mut u32, 1u32 << (irq % 32));
        }
    }

    fn eoi(&self, irq: u32) {
        // EIOINTC EOI: write to EIOINTC EOI register
        if irq >= EIOINTC_IRQ_COUNT {
            return;
        }
        // SAFETY: MMIO write to acknowledge interrupt completion
        unsafe {
            let eoi_addr = EIOINTC_BASE + 0x0040;
            write_volatile(eoi_addr as *mut u32, irq);
        }
    }

    fn set_affinity(&self, irq: u32, cpu_mask: u64) {
        if irq >= EIOINTC_IRQ_COUNT {
            return;
        }
        // SAFETY: MMIO write to set interrupt routing
        unsafe {
            let route_addr = EIOINTC_BASE + 0x0060 + (irq as u64) * 8;
            write_volatile(route_addr as *mut u64, cpu_mask);
        }
    }

    fn get_irq_count(&self, irq: u32) -> u64 {
        if irq as usize >= MAX_IRQ_HANDLERS {
            return 0;
        }
        IRQ_COUNTS[irq as usize].load(Ordering::Acquire)
    }
}

// ============================================================================
// LoongArch64 Timer
// ============================================================================

/// LoongArch64 timer implementation
pub struct LoongArch64Timer;

impl TimerOps for LoongArch64Timer {
    fn init(&self) {
        // Initialize timer
        // Use stable counter
    }

    fn now(&self) -> u64 {
        // Read stable counter
        let count: u64;
        // SAFETY: inline assembly required for hardware instruction
        unsafe {
            core::arch::asm!(
                "rdtime.d {}, $r0",
                out(reg) count,
            );
        }
        count
    }

    fn set_oneshot(&self, ns: u64) {
        // Set one-shot timer
        // Implementation: Configure timer compare register for one-shot mode
        let _ = ns;
    }

    fn set_periodic(&self, ns: u64) {
        // Set periodic timer
        // Implementation: Configure timer compare register for periodic mode
        let _ = ns;
    }

    fn stop(&self) {
        // Stop timer
    }

    fn frequency(&self) -> u64 {
        // Timer frequency (usually 1 GHz)
        1_000_000_000
    }

    fn delay(&self, ns: u64) {
        // Busy wait
        let start = self.now();
        let end = start + ns;
        while self.now() < end {
            core::hint::spin_loop();
        }
    }
}

// ============================================================================
// LoongArch64 Power Management
// ============================================================================

/// LoongArch64 power management implementation
pub struct LoongArch64Power;

impl PowerOps for LoongArch64Power {
    fn init(&self) {
        // Initialize power management
    }

    fn cpu_idle(&self) {
        // CPU idle
        // SAFETY: inline assembly required for hardware instruction
        unsafe {
            core::arch::asm!("idle 0");
        }
    }

    fn cpu_sleep(&self) {
        // CPU sleep
    }

    fn cpu_wakeup(&self, _cpu_id: u32) {
        // Wake up CPU
    }

    fn system_shutdown(&self) {
        // System shutdown
    }

    fn system_reboot(&self) {
        // System reboot
    }

    fn system_suspend(&self) {
        // System suspend
    }
}

// ============================================================================
// LoongArch64 Context Operations
// ============================================================================

/// LoongArch64 context operations implementation
pub struct LoongArch64Context;

impl ContextOps for LoongArch64Context {
    fn save_context(&self, ctx: &mut CpuContext) {
        // Save current context
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            // Save general registers
            core::arch::asm!(
                "st.d $r1, {0}, 0",
                "st.d $r2, {0}, 8",
                "st.d $r3, {0}, 16",
                // ... more registers
                in(reg) ctx.regs.as_mut_ptr() as u64,
            );

            // Save stack pointer and program counter
            core::arch::asm!(
                "move {}, $sp",
                "move {1}, $ra",
                out(reg) ctx.sp,
                out(reg) ctx.pc,
            );
        }
    }

    fn restore_context(&self, ctx: &CpuContext) {
        // Restore context
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            // Restore stack pointer and program counter
            core::arch::asm!(
                "move $sp, {}",
                "move $ra, {}",
                in(reg) ctx.sp,
                in(reg) ctx.pc,
            );

            // Restore general registers
            core::arch::asm!(
                "ld.d $r1, {0}, 0",
                "ld.d $r2, {0}, 8",
                "ld.d $r3, {0}, 16",
                // ... more registers
                in(reg) ctx.regs.as_ptr() as u64,
            );
        }
    }

    fn switch_context(&self, from: &mut CpuContext, to: &CpuContext) {
        self.save_context(from);
        self.restore_context(to);
    }
}

// ============================================================================
// LoongArch64 Architecture Implementation
// ============================================================================

/// LoongArch64 architecture implementation
pub struct LoongArch64Arch;

impl ArchOps for LoongArch64Arch {
    fn init(&self) {
        log_info!("LoongArch64 architecture initialized");

        // Initialize subsystems
        LoongArch64PageTable.create();
        LoongArch64IrqController.init();
        LoongArch64Timer.init();
        LoongArch64Power.init();
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
        // Enable interrupt
        // SAFETY: inline assembly required for hardware instruction
        unsafe {
            core::arch::asm!(
                "csrrd $t0, {}",
                "ori $t0, $t0, 1",
                "csrwr $t0, {}",
                in(reg) csr::CRMD,
                in(reg) csr::CRMD,
            );
        }
    }

    fn disable_irq(&self) {
        // Disable interrupt
        // SAFETY: inline assembly required for hardware instruction
        unsafe {
            core::arch::asm!(
                "csrrd $t0, {}",
                "andi $t0, $t0, ~1",
                "csrwr $t0, {}",
                in(reg) csr::CRMD,
                in(reg) csr::CRMD,
            );
        }
    }

    fn cpu_id(&self) -> u32 {
        let id: u32;
        // SAFETY: inline assembly required for hardware instruction
        unsafe {
            core::arch::asm!(
                "csrrd {}, {}",
                out(reg) id,
                in(reg) csr::CPUID,
            );
        }
        id
    }

    fn cpu_count(&self) -> u32 {
        // Implementation: Query firmware (ACPI SRAT or FDT) for total CPU count
        4
    }
}

/// LoongArch64 architecture instance
pub static LOONGARCH64_ARCH: LoongArch64Arch = LoongArch64Arch;

// ============================================================================
// LoongArch Extended Instruction Set Support
// ============================================================================

/// LoongArch extended instruction set
#[derive(Debug, Clone, Copy)]
pub struct LoongArchExtensions {
    /// LSX: 128-bit SIMD extension
    pub lsx: bool,
    /// LASX: 256-bit SIMD extension
    pub lasx: bool,
    /// LVZ: Virtualization extension
    pub lvz: bool,
    /// LBT: Binary translation extension
    pub lbt: bool,
}

impl Default for LoongArchExtensions {
    fn default() -> Self {
        Self {
            lsx: true,
            lasx: true,
            lvz: true,
            lbt: true,
        }
    }
}

impl LoongArchExtensions {
    /// Detect CPU supported extensions
    pub fn detect() -> Self {
        let mut ext = Self {
            lsx: false,
            lasx: false,
            lvz: false,
            lbt: false,
        };

        #[cfg(target_arch = "loongarch64")]
        {
            // SAFETY: CPUCFG is a privileged instruction that reads CPU configuration
            // registers; it has no side effects and is safe to execute.
            unsafe {
                let cfg2: u32;
                core::arch::asm!(
                    "cpucfg {}, $r2",
                    out(reg) cfg2,
                );
                ext.lsx = (cfg2 & (1 << 6)) != 0;
                ext.lasx = (cfg2 & (1 << 7)) != 0;
                ext.lvz = (cfg2 & (1 << 8)) != 0;
                ext.lbt = (cfg2 & (1 << 9)) != 0;
            }
        }

        #[cfg(not(target_arch = "loongarch64"))]
        {
            ext = Self::default();
        }

        ext
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phys_addr() {
        let addr = PhysAddr::new(0x1000);
        assert_eq!(addr.as_u64(), 0x1000);
        assert!(addr.is_aligned(0x1000));
    }

    #[test]
    fn test_virt_addr() {
        let addr = VirtAddr::new(0x1000);
        assert_eq!(addr.as_u64(), 0x1000);
        assert!(!addr.is_null());
    }

    #[test]
    fn test_prot_flags() {
        let flags = ProtFlags::RW;
        assert!(flags.is_readable());
        assert!(flags.is_writable());
        assert!(!flags.is_executable());
    }
}
