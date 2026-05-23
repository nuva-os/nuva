/*
 * Nuva OS - Kernel - Kernel
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


use core::arch::asm;

pub mod mmu;
pub mod apic;
pub mod idt;
pub mod gdt;
pub mod exception;
pub mod arch_impl;
pub mod plugin;
pub mod boot;
pub mod context;
pub mod timer;
pub mod mm;
pub mod trap;

// Re-export architecture implementation
pub use arch_impl::*;

/// CPUID instruction
pub fn cpuid(leaf: u32, subleaf: u32) -> (u32, u32, u32, u32) {
    let eax: u32;
    let ebx: u32;
    let ecx: u32;
    let edx: u32;

    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!(
            "cpuid",
            in("eax") leaf,
            in("ecx") subleaf,
            lateout("eax") eax,
            lateout("ebx") ebx,
            lateout("ecx") ecx,
            lateout("edx") edx,
        );
    }

    (eax, ebx, ecx, edx)
}

/// Get CPU vendor
pub fn get_cpu_vendor() -> [u8; 12] {
    let (_, ebx, ecx, edx) = cpuid(0, 0);

    let mut vendor = [0u8; 12];
    vendor[0..4].copy_from_slice(&ebx.to_le_bytes());
    vendor[4..8].copy_from_slice(&edx.to_le_bytes());
    vendor[8..12].copy_from_slice(&ecx.to_le_bytes());

    vendor
}

/// Get CPU brand string
pub fn get_cpu_brand() -> [u8; 48] {
    let mut brand = [0u8; 48];

    for i in 0..3 {
        let (eax, ebx, ecx, edx) = cpuid(0x80000002 + i, 0);
        let offset = (i as usize) * 16;
        brand[offset..offset+4].copy_from_slice(&eax.to_le_bytes());
        brand[offset+4..offset+8].copy_from_slice(&ebx.to_le_bytes());
        brand[offset+8..offset+12].copy_from_slice(&ecx.to_le_bytes());
        brand[offset+12..offset+16].copy_from_slice(&edx.to_le_bytes());
    }

    brand
}

/// Read CR0
#[inline(always)]
pub fn read_cr0() -> u64 {
    let val: u64;
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!(
            "mov {}, cr0",
            out(reg) val,
        );
    }
    val
}

/// Write CR0
#[inline(always)]
pub fn write_cr0(val: u64) {
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!(
            "mov cr0, {}",
            in(reg) val,
        );
    }
}

/// Read CR2
#[inline(always)]
pub fn read_cr2() -> u64 {
    let val: u64;
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!(
            "mov {}, cr2",
            out(reg) val,
        );
    }
    val
}

/// Read CR3
#[inline(always)]
pub fn read_cr3() -> u64 {
    let val: u64;
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!(
            "mov {}, cr3",
            out(reg) val,
        );
    }
    val
}

/// Write CR3
#[inline(always)]
pub fn write_cr3(val: u64) {
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!(
            "mov cr3, {}",
            in(reg) val,
        );
    }
}

/// Read CR4
#[inline(always)]
pub fn read_cr4() -> u64 {
    let val: u64;
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!(
            "mov {}, cr4",
            out(reg) val,
        );
    }
    val
}

/// Write CR4
#[inline(always)]
pub fn write_cr4(val: u64) {
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!(
            "mov cr4, {}",
            in(reg) val,
        );
    }
}

/// Read MSR
#[inline(always)]
pub fn read_msr(reg: u32) -> u64 {
    let (low, high): (u32, u32);
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!(
            "rdmsr",
            in("ecx") reg,
            lateout("eax") low,
            lateout("edx") high,
        );
    }
    ((high as u64) << 32) | (low as u64)
}

/// Write MSR
#[inline(always)]
pub fn write_msr(reg: u32, val: u64) {
    let low = val as u32;
    let high = (val >> 32) as u32;

    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!(
            "wrmsr",
            in("ecx") reg,
            in("eax") low,
            in("edx") high,
        );
    }
}

/// Enable interrupt
#[inline(always)]
pub fn enable_irq() {
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!("sti");
    }
}

/// Disable interrupt
#[inline(always)]
pub fn disable_irq() {
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!("cli");
    }
}

/// Halt
#[inline(always)]
pub fn hlt() {
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!("hlt");
    }
}

/// Pause
#[inline(always)]
pub fn pause() {
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!("pause");
    }
}

/// No operation
#[inline(always)]
pub fn nop() {
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!("nop");
    }
}

/// Enable MMU (paging)
pub fn enable_paging() {
    let cr0 = read_cr0();
    write_cr0(cr0 | (1 << 31));  // Set PG bit
}

/// Disable MMU (paging)
pub fn disable_paging() {
    let cr0 = read_cr0();
    write_cr0(cr0 & !(1 << 31));  // Clear PG bit
}

/// Enable protected mode
pub fn enable_protected_mode() {
    let cr0 = read_cr0();
    write_cr0(cr0 | 1);  // Set PE bit
}

/// Enable long mode
pub fn enable_long_mode() {
    // Set PAE (Physical Address Extension)
    let cr4 = read_cr4();
    write_cr4(cr4 | (1 << 5));

    // Set LME (Long Mode Enable) in EFER MSR
    let efer = read_msr(0xC0000080);
    write_msr(0xC0000080, efer | (1 << 8));
}

/// Flush TLB
#[inline(always)]
pub fn tlb_flush() {
    let cr3 = read_cr3();
    write_cr3(cr3);
}

/// Flush TLB (single address)
#[inline(always)]
pub fn tlb_flush_addr(addr: u64) {
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!(
            "invlpg [{}]",
            in(reg) addr,
        );
    }
}

/// Flush entire TLB (including global pages)
#[inline(always)]
pub fn tlb_flush_all() {
    let cr4 = read_cr4();

    // Disable global pages
    write_cr4(cr4 & !(1 << 7));
    tlb_flush();

    // Re-enable global pages
    write_cr4(cr4);
}

/// Interrupt save
pub struct IrqSave {
    rflags: u64,
}

impl IrqSave {
    /// Save and disable interrupt
    pub fn save_disable() -> Self {
        let rflags: u64;
        // SAFETY: inline assembly required for hardware instruction
        unsafe {
            asm!(
                "pushfq",
                "pop {}",
                "cli",
                out(reg) rflags,
            );
        }
        IrqSave { rflags }
    }
}

impl Drop for IrqSave {
    fn drop(&mut self) {
        if self.rflags & (1 << 9) != 0 {
            enable_irq();
        }
    }
}

/// Get RFLAGS
pub fn get_rflags() -> u64 {
    let rflags: u64;
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!(
            "pushfq",
            "pop {}",
            out(reg) rflags,
        );
    }
    rflags
}

/// Set RFLAGS
pub fn set_rflags(rflags: u64) {
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!(
            "push {}",
            "popfq",
            in(reg) rflags,
        );
    }
}

/// Read timestamp counter
#[inline(always)]
pub fn rdtsc() -> u64 {
    let low: u32;
    let high: u32;
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!(
            "rdtsc",
            lateout("eax") low,
            lateout("edx") high,
        );
    }
    ((high as u64) << 32) | (low as u64)
}

/// Read timestamp counter (serialized)
#[inline(always)]
pub fn rdtscp() -> u64 {
    let low: u32;
    let high: u32;
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!(
            "rdtscp",
            lateout("eax") low,
            lateout("edx") high,
        );
    }
    ((high as u64) << 32) | (low as u64)
}

/// Initialize x86-64 architecture
pub fn init_arch() {
    let vendor = get_cpu_vendor();
    let vendor_str = core::str::from_utf8(&vendor).unwrap_or("Unknown");

    log_info!("x86-64 architecture initialized");
    log_info!("  CPU Vendor: {}", vendor_str);
}
