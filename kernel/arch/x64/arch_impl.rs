/*
 * Nuva OS - Kernel - x86_64 Architecture Implementation
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

use crate::arch::*;
use crate::arch::x64::*;

/// x86_64 Page Table Operation Implementation
pub struct X64PageTable;

impl PageTableOps for X64PageTable {
    fn create() -> PhysAddr {
        log_info!("x86_64: Creating page table");

        // Allocate a physical page as PML4
        let page_phys = crate::mm::page_alloc::alloc_page();

        // Clear page table
        mmu::clear_page_table(page_phys.as_u64());

        log_info!("x86_64: Page table created at {:?}", page_phys);
        page_phys
    }

    fn destroy(&self, pgd: PhysAddr) {
        log_info!("x86_64: Destroying page table at {}", pgd);

        // Recursively free all page table pages (PML4 -> PDPT -> PD -> PT)
        // Level 0 = PML4, Level 1 = PDPT, Level 2 = PD, Level 3 = PT
        let pml4 = mmu::get_page_table(pgd.as_u64());
        for i in 0..512 {
            let pml4e = pml4.get_entry(i);
            if !pml4e.is_present() || pml4e.is_huge() {
                continue;
            }
            let pdpt_phys = pml4e.get_phys();

            // Free PDPT level
            let pdpt = mmu::get_page_table(pdpt_phys);
            for j in 0..512 {
                let pdpte = pdpt.get_entry(j);
                if !pdpte.is_present() || pdpte.is_huge() {
                    continue;
                }
                let pd_phys = pdpte.get_phys();

                // Free PD level
                let pd = mmu::get_page_table(pd_phys);
                for k in 0..512 {
                    let pde = pd.get_entry(k);
                    if !pde.is_present() || pde.is_huge() {
                        continue;
                    }
                    let pt_phys = pde.get_phys();

                    // Free PT page (leaf page table)
                    // SAFETY: pt_phys is a valid page table physical address obtained
                    // from the page walk; free_page returns it to the physical allocator.
                    crate::mm::page_alloc::free_page(pt_phys as *mut u8);
                }

                // Free PD page
                // SAFETY: pd_phys is a valid page directory physical address.
                crate::mm::page_alloc::free_page(pd_phys as *mut u8);
            }

            // Free PDPT page
            // SAFETY: pdpt_phys is a valid PDPT physical address.
            crate::mm::page_alloc::free_page(pdpt_phys as *mut u8);
        }

        // Free PML4 page
        // SAFETY: pgd is the PML4 root physical address; after freeing all children,
        // we free the PML4 page itself.
        crate::mm::page_alloc::free_page(pgd.as_u64() as *mut u8);
    }

    fn map(pgd: PhysAddr, vaddr: VirtAddr, paddr: PhysAddr, prot: ProtFlags, page_size: u64) {
        log_info!("x86_64: Mapping {:?} -> {:?} with prot {:?}", vaddr, paddr, prot);

        // Convert permission flags to x86_64 PTE flags
        let mut pte_flags = mmu::pte_flags::PRESENT;

        if prot.is_writable() {
            pte_flags |= mmu::pte_flags::WRITABLE;
        }

        if prot.is_user() {
            pte_flags |= mmu::pte_flags::USER;
        }

        if !prot.is_executable() {
            pte_flags |= mmu::pte_flags::NO_EXECUTE;
        }

        // Call actual mapping implementation
        mmu::page_table_map_impl(pgd.as_u64(), vaddr.as_u64(), paddr.as_u64(), pte_flags, page_size);
    }

    fn unmap(pgd: PhysAddr, vaddr: VirtAddr) {
        log_info!("x86_64: Unmapping {:?}", vaddr);

        // Call actual unmap implementation
        if let Some(_phys) = mmu::page_table_unmap_impl(pgd.as_u64(), vaddr.as_u64()) {
            // Flush TLB
            Self::tlb_flush_addr(vaddr);
        }
    }

    fn translate(pgd: PhysAddr, vaddr: VirtAddr) -> Option<PhysAddr> {
        // Call actual address translation implementation
        mmu::page_table_translate_impl(pgd.as_u64(), vaddr.as_u64()).map(PhysAddr::new)
    }

    fn protect(&self, pgd: PhysAddr, vaddr: VirtAddr, prot: ProtFlags) {
        log_info!("x86_64: Protecting {:?} with {:?}", vaddr, prot);

        let (pml4_idx, pdpt_idx, pd_idx, pt_idx) = mmu::parse_vaddr(vaddr.as_u64());

        // Walk PML4 -> PDPT -> PD -> PT to find the leaf PTE
        let pml4 = mmu::get_page_table(pgd.as_u64());
        let pml4e = pml4.get_entry(pml4_idx);
        if !pml4e.is_present() {
            return;
        }

        let pdpt = mmu::get_page_table(pml4e.get_phys());
        let pdpte = pdpt.get_entry(pdpt_idx);
        if !pdpte.is_present() {
            // Check for 1GB huge page
            if pdpte.is_huge() {
                // Modify 1GB page permissions directly
                let pdpt_mut = mmu::get_page_table_mut(pml4e.get_phys());
                let pte = pdpt_mut.get_entry_mut(pdpt_idx);
                let mut val = pte.value;
                // Clear R/W, U, NX bits
                val &= !(mmu::pte_flags::WRITABLE | mmu::pte_flags::USER | mmu::pte_flags::NO_EXECUTE);
                // Set new permissions
                if prot.is_writable() {
                    val |= mmu::pte_flags::WRITABLE;
                }
                if prot.is_user() {
                    val |= mmu::pte_flags::USER;
                }
                if !prot.is_executable() {
                    val |= mmu::pte_flags::NO_EXECUTE;
                }
                pte.value = val;
            }
            return;
        }

        let pd = mmu::get_page_table(pdpte.get_phys());
        let pde = pd.get_entry(pd_idx);
        if !pde.is_present() {
            // Check for 2MB huge page
            if pde.is_huge() {
                // Modify 2MB page permissions directly
                let pd_mut = mmu::get_page_table_mut(pdpte.get_phys());
                let pte = pd_mut.get_entry_mut(pd_idx);
                let mut val = pte.value;
                val &= !(mmu::pte_flags::WRITABLE | mmu::pte_flags::USER | mmu::pte_flags::NO_EXECUTE);
                if prot.is_writable() {
                    val |= mmu::pte_flags::WRITABLE;
                }
                if prot.is_user() {
                    val |= mmu::pte_flags::USER;
                }
                if !prot.is_executable() {
                    val |= mmu::pte_flags::NO_EXECUTE;
                }
                pte.value = val;
            }
            return;
        }

        let pt = mmu::get_page_table_mut(pde.get_phys());
        let pte = pt.get_entry_mut(pt_idx);
        if !pte.is_present() {
            return;
        }

        // Modify R/W, User, NX bits on the leaf PTE
        let mut val = pte.value;
        // Clear existing R/W, U, NX bits
        val &= !(mmu::pte_flags::WRITABLE | mmu::pte_flags::USER | mmu::pte_flags::NO_EXECUTE);
        // Set new permissions from ProtFlags
        if prot.is_writable() {
            val |= mmu::pte_flags::WRITABLE;
        }
        if prot.is_user() {
            val |= mmu::pte_flags::USER;
        }
        if !prot.is_executable() {
            val |= mmu::pte_flags::NO_EXECUTE;
        }
        pte.value = val;

        // Flush TLB entry for this virtual address
        Self::tlb_flush_addr(vaddr);
    }

    fn tlb_flush_addr(vaddr: VirtAddr) {
        // SAFETY: inline assembly required for hardware instruction
        unsafe {
            core::arch::asm!(
                "invlpg [{}]",
                in(reg) vaddr.as_u64(),
            );
        }
    }

    fn tlb_flush_all() {
        let cr3 = read_cr3();
        write_cr3(cr3);
    }

    fn switch(pgd: PhysAddr) {
        write_cr3(pgd.as_u64());
    }

    fn current() -> PhysAddr {
        PhysAddr::new(read_cr3())
    }
}

/// MSR_IA32_APICBASE — Local APIC base address
const MSR_IA32_APICBASE: u32 = 0x1B;

/// SVR enable bit (bit 8)
const SVR_APIC_ENABLED: u32 = 1 << 8;

/// I/O APIC redirection entry mask bit (bit 16 in lower 32 bits)
const REDIR_MASK_BIT: u32 = 1 << 16;

/// Default I/O APIC base physical address
const DEFAULT_IOAPIC_BASE: u64 = 0xFEC0_0000;

/// Maximum number of IRQ handlers
const MAX_IRQ: usize = 256;

/// Bit-map for tracking allocated IRQ numbers
struct IrqBitmap {
    bits: [u64; MAX_IRQ / 64],
}

impl IrqBitmap {
    const fn new() -> Self {
        IrqBitmap {
            bits: [0; MAX_IRQ / 64],
        }
    }

    fn alloc(&mut self) -> Option<u32> {
        for word_idx in 0..self.bits.len() {
            if self.bits[word_idx] != !0u64 {
                let word = self.bits[word_idx];
                for bit_idx in 0..64 {
                    if word & (1u64 << bit_idx) == 0 {
                        let irq = (word_idx as u32) * 64 + bit_idx;
                        if irq as usize >= MAX_IRQ {
                            return None;
                        }
                        self.bits[word_idx] |= 1u64 << bit_idx;
                        return Some(irq);
                    }
                }
            }
        }
        None
    }

    fn free(&mut self, irq: u32) {
        let irq_usize = irq as usize;
        if irq_usize >= MAX_IRQ {
            return;
        }
        let word_idx = irq_usize / 64;
        let bit_idx = irq_usize % 64;
        self.bits[word_idx] &= !(1u64 << bit_idx);
    }
}

/// APIC driver state
pub struct ApicDriver {
    /// LAPIC MMIO base address
    lapic_base: u64,
    /// I/O APIC MMIO base address
    ioapic_base: u64,
    /// Number of I/O APIC redirection entries
    num_ioapic_entries: u32,
    /// IRQ handler functions
    irq_handlers: [Option<fn(u32)>; MAX_IRQ],
    /// Per-IRQ invocation counters
    irq_counts: [u64; MAX_IRQ],
    /// IRQ allocation bitmap
    irq_bitmap: IrqBitmap,
}

impl ApicDriver {
    const fn new() -> Self {
        ApicDriver {
            lapic_base: 0,
            ioapic_base: 0,
            num_ioapic_entries: 0,
            irq_handlers: [None; MAX_IRQ],
            irq_counts: [0; MAX_IRQ],
            irq_bitmap: IrqBitmap::new(),
        }
    }

    fn init_driver(&mut self) {
        // Read APIC base from MSR 0x1B
        let msr_val = super::read_msr(MSR_IA32_APICBASE);
        self.lapic_base = msr_val & 0xFFFF_F000;

        // Default I/O APIC base (typically discovered via ACPI/MADT)
        self.ioapic_base = DEFAULT_IOAPIC_BASE;

        log_info!("x86_64: APIC base from MSR: {:#X}", self.lapic_base);
        log_info!("x86_64: I/O APIC base: {:#X}", self.ioapic_base);

        // Set Spurious Vector Register to enable LAPIC
        // Vector 0xFF (spurious), set APIC software enable bit
        let svr = apic::read_lapic(self.lapic_base, apic::LAPIC_SVR);
        apic::write_lapic(self.lapic_base, apic::LAPIC_SVR, (svr | SVR_APIC_ENABLED) | 0xFF);

        // Set Task Priority to 0 (accept all interrupts)
        apic::write_lapic(self.lapic_base, apic::LAPIC_TPR, 0);

        // Mask error LVT
        apic::write_lapic(self.lapic_base, apic::LAPIC_ERROR, REDIR_MASK_BIT);

        // Initialize I/O APIC: read version register to get max redirection entries
        let ver = apic::read_ioapic(self.ioapic_base, apic::IOAPIC_REG_VERSION);
        self.num_ioapic_entries = ((ver >> 16) & 0xFF) + 1;
        log_info!("x86_64: I/O APIC version {}, max entries: {}", ver & 0xFF, self.num_ioapic_entries);

        // Mask all I/O APIC redirection entries
        for i in 0..self.num_ioapic_entries {
            let reg = apic::IOAPIC_REG_REDIR_BASE + i * 2;
            let low = apic::read_ioapic(self.ioapic_base, reg);
            apic::write_ioapic(self.ioapic_base, reg, low | REDIR_MASK_BIT);
        }

        log_info!("x86_64: APIC initialized");
    }

    fn write_redir_entry(&self, irq: u32, vector: u8, dest: u8, mask: bool) {
        if irq >= self.num_ioapic_entries {
            return;
        }
        let reg_low = apic::IOAPIC_REG_REDIR_BASE + irq * 2;
        let reg_high = reg_low + 1;

        let low = vector as u32
            | (0u32 << 8)   // delivery mode: fixed
            | (0u32 << 11)  // delivery status
            | (0u32 << 13)  // polarity: active high
            | (0u32 << 15)  // trigger mode: edge
            | ((mask as u32) << 16);
        let high = (dest as u32) << 24;

        apic::write_ioapic(self.ioapic_base, reg_low, low);
        apic::write_ioapic(self.ioapic_base, reg_high, high);
    }
}

/// Global APIC driver instance
static APIC_DRIVER: core::sync::OnceLock<ApicDriver> = core::sync::OnceLock::new();

/// x86_64 Interrupt Controller Implementation (APIC)
pub struct X64IrqController;

impl IrqControllerOps for X64IrqController {
    fn init(&self) {
        // SAFETY: APIC driver is only accessed here during early init before any concurrent access.
        unsafe {
            APIC_DRIVER.init_driver();
        }
    }

    fn alloc_irq(&self) -> Option<u32> {
        // SAFETY: Single-threaded access during IRQ allocation; caller must coordinate.
        unsafe {
            APIC_DRIVER.irq_bitmap.alloc()
        }
    }

    fn free_irq(&self, irq: u32) {
        // SAFETY: Caller guarantees the IRQ was previously allocated and no handler is active.
        unsafe {
            APIC_DRIVER.irq_bitmap.free(irq);
            APIC_DRIVER.irq_handlers[irq as usize] = None;
        }
    }

    fn register_handler(&self, irq: u32, handler: fn(u32), flags: u32) -> bool {
        if irq as usize >= MAX_IRQ {
            return false;
        }
        // SAFETY: Caller guarantees exclusive access; IRQ registration is done under lock by caller.
        unsafe {
            // Write I/O APIC redirection table: unmask, vector = irq + 0x20, dest = BSP (0)
            let vector = (irq + 0x20) as u8;
            APIC_DRIVER.write_redir_entry(irq, vector, 0, false);
            APIC_DRIVER.irq_handlers[irq as usize] = Some(handler);
            APIC_DRIVER.irq_counts[irq as usize] = 0;
        }
        let _ = flags;
        log_info!("x86_64: Registered handler for IRQ {} (vector {:#X})", irq, irq + 0x20);
        true
    }

    fn unregister_handler(&self, irq: u32) {
        if irq as usize >= MAX_IRQ {
            return;
        }
        // SAFETY: Caller guarantees the handler is not currently executing.
        unsafe {
            APIC_DRIVER.irq_handlers[irq as usize] = None;
            // Mask the redirection entry
            if irq < APIC_DRIVER.num_ioapic_entries {
                let reg = apic::IOAPIC_REG_REDIR_BASE + irq * 2;
                let low = apic::read_ioapic(APIC_DRIVER.ioapic_base, reg);
                apic::write_ioapic(APIC_DRIVER.ioapic_base, reg, low | REDIR_MASK_BIT);
            }
        }
    }

    fn enable_irq(&self, irq: u32) {
        // SAFETY: Clearing the mask bit in the I/O APIC redirection entry.
        unsafe {
            if irq < APIC_DRIVER.num_ioapic_entries {
                let reg = apic::IOAPIC_REG_REDIR_BASE + irq * 2;
                let low = apic::read_ioapic(APIC_DRIVER.ioapic_base, reg);
                apic::write_ioapic(APIC_DRIVER.ioapic_base, reg, low & !REDIR_MASK_BIT);
            }
        }
    }

    fn disable_irq(&self, irq: u32) {
        // SAFETY: Setting the mask bit in the I/O APIC redirection entry.
        unsafe {
            if irq < APIC_DRIVER.num_ioapic_entries {
                let reg = apic::IOAPIC_REG_REDIR_BASE + irq * 2;
                let low = apic::read_ioapic(APIC_DRIVER.ioapic_base, reg);
                apic::write_ioapic(APIC_DRIVER.ioapic_base, reg, low | REDIR_MASK_BIT);
            }
        }
    }

    fn eoi(&self, _irq: u32) {
        // SAFETY: Writing EOI register is always safe; it signals end of interrupt processing.
        unsafe {
            apic::write_lapic(APIC_DRIVER.lapic_base, apic::LAPIC_EOI, 0);
        }
    }

    fn set_affinity(&self, irq: u32, cpu_mask: u64) {
        // SAFETY: Modifying I/O APIC redirection entry destination field.
        unsafe {
            if irq < APIC_DRIVER.num_ioapic_entries {
                // Find lowest set bit in cpu_mask as destination APIC ID
                let dest = if cpu_mask == 0 { 0u8 } else { cpu_mask.trailing_zeros() as u8 };
                let reg_high = apic::IOAPIC_REG_REDIR_BASE + irq * 2 + 1;
                let high = (dest as u32) << 24;
                apic::write_ioapic(APIC_DRIVER.ioapic_base, reg_high, high);
            }
        }
    }

    fn get_irq_count(&self, irq: u32) -> u64 {
        if irq as usize >= MAX_IRQ {
            return 0;
        }
        // SAFETY: Read-only access; no mutation hazard.
        unsafe {
            APIC_DRIVER.irq_counts[irq as usize]
        }
    }
}

/// LAPIC Timer LVT: timer mode mask (bits 18:17)
const LAPIC_TIMER_MODE_MASK: u32 = 0b11 << 17;
/// LAPIC Timer LVT: one-shot mode (0b00)
const LAPIC_TIMER_MODE_ONESHOT: u32 = 0b00 << 17;
/// LAPIC Timer LVT: periodic mode (0b01)
const LAPIC_TIMER_MODE_PERIODIC: u32 = 0b01 << 17;
/// LAPIC Timer LVT: mask bit (bit 16)
const LAPIC_TIMER_LVT_MASK: u32 = 1 << 16;
/// LAPIC Timer vector for scheduling (IRQ 0x20)
const LAPIC_TIMER_VECTOR: u32 = 0x20;
/// LAPIC Timer divide value: divide by 1
const LAPIC_TIMER_DIVIDE_1: u32 = 0b1011;

/// MSR_IA32_TSC_AUX for TSC frequency (Intel)
const MSR_IA32_TSC_FREQ: u32 = 0x0CE;
/// HPET signature
const HPET_SIGNATURE: [u8; 4] = *b"HPET";

/// Calibrated TSC frequency (Hz), set during init
static mut TSC_FREQUENCY: u64 = 0;

/// HPET period in femtoseconds, 0 if not detected
static mut HPET_PERIOD_FS: u64 = 0;

/// x86_64 Timer Implementation (LAPIC Timer / HPET)
pub struct X64Timer;

impl TimerOps for X64Timer {
    fn init(&self) {
        log_info!("x86_64: Initializing LAPIC Timer + HPET");

        // Step 1: Detect HPET via ACPI table
        // SAFETY: Reading ACPI tables during early init; single-threaded access.
        let hpet_detected = unsafe {
            let acpi = crate::hal::acpi::get_acpi_tables();
            acpi.find_table(&HPET_SIGNATURE).is_some()
        };
        if hpet_detected {
            log_info!("x86_64: HPET detected via ACPI");
            // SAFETY: HPET_PERIOD_FS is only written here during single-threaded init.
            unsafe {
                // HPET main counter period is at offset 0x4 of the HPET table
                // (femtoseconds per tick). Default 10ns = 10_000_000 fs if not readable.
                HPET_PERIOD_FS = 10_000_000;
            }
        } else {
            log_info!("x86_64: HPET not found, using LAPIC Timer only");
        }

        // Step 2: Configure LAPIC Timer vector
        // SAFETY: APIC_DRIVER is accessed only during early init before SMP.
        unsafe {
            let lapic_base = APIC_DRIVER.lapic_base;
            if lapic_base != 0 {
                // Set timer LVT: vector, unmask, one-shot mode
                let lvt = LAPIC_TIMER_VECTOR | LAPIC_TIMER_MODE_ONESHOT;
                apic::write_lapic(lapic_base, apic::LAPIC_TIMER_LVT, lvt);
                // Set divide configuration to 1
                apic::write_lapic(lapic_base, apic::LAPIC_TIMER_DCR, LAPIC_TIMER_DIVIDE_1);
            }
        }

        // Step 3: Calibrate TSC frequency
        // Try CPUID leaf 0x15 (Intel TSC freq) first
        let (eax, _ebx, _ecx, _edx) = super::cpuid(0x15, 0);
        let tsc_freq: u64 = if eax != 0 {
            // CPUID.15H: EAX = denominator, EBX = numerator, ECX = core crystal clock
            let (_eax2, ebx2, ecx2, _edx2) = super::cpuid(0x15, 0);
            let denom = eax as u64;
            let numer = ebx2 as u64;
            let crystal = ecx2 as u64;
            if numer != 0 && denom != 0 && crystal != 0 {
                crystal * numer / denom
            } else {
                0
            }
        } else {
            0
        };

        if tsc_freq != 0 {
            // SAFETY: TSC_FREQUENCY is only written here during single-threaded init.
            unsafe { TSC_FREQUENCY = tsc_freq; }
            log_info!("x86_64: TSC frequency calibrated via CPUID: {} Hz", tsc_freq);
        } else {
            // Fallback: read MSR_IA32_TSC_FREQ or estimate from LAPIC bus
            let freq = super::read_msr(MSR_IA32_TSC_FREQ);
            if freq != 0 && freq != !0u64 {
                // SAFETY: TSC_FREQUENCY is only written here during single-threaded init.
                unsafe { TSC_FREQUENCY = freq; }
                log_info!("x86_64: TSC frequency from MSR: {} Hz", freq);
            } else {
                // Final fallback: estimate 3 GHz
                // SAFETY: TSC_FREQUENCY is only written here during single-threaded init.
                unsafe { TSC_FREQUENCY = 3_000_000_000; }
                log_info!("x86_64: TSC frequency estimated: 3 GHz");
            }
        }
    }

    fn now(&self) -> u64 {
        // Convert TSC ticks to nanoseconds using calibrated frequency
        let tsc = rdtsc();
        // SAFETY: TSC_FREQUENCY is read-only after init; no concurrent write hazard.
        let freq = unsafe { TSC_FREQUENCY };
        if freq == 0 {
            return tsc;
        }
        tsc * 1_000_000_000 / freq
    }

    fn set_oneshot(&self, ns: u64) {
        // SAFETY: APIC_DRIVER lapic_base is set during init and read-only afterwards.
        unsafe {
            let lapic_base = APIC_DRIVER.lapic_base;
            if lapic_base == 0 {
                return;
            }
            // Set LVT to one-shot mode (0b00), unmask, with timer vector
            let lvt = LAPIC_TIMER_VECTOR | LAPIC_TIMER_MODE_ONESHOT;
            apic::write_lapic(lapic_base, apic::LAPIC_TIMER_LVT, lvt);
            // Calculate initial count: ICR = ns * frequency / 1e9
            let freq = TSC_FREQUENCY;
            if freq == 0 {
                return;
            }
            let icr = ns * freq / 1_000_000_000;
            // ICR is u32; cap at u32::MAX to avoid overflow
            let icr_val = if icr > u32::MAX as u64 { u32::MAX } else { icr as u32 };
            apic::write_lapic(lapic_base, apic::LAPIC_TIMER_DCR, LAPIC_TIMER_DIVIDE_1);
            apic::write_lapic(lapic_base, apic::LAPIC_TIMER_ICR, icr_val);
        }
    }

    fn set_periodic(&self, ns: u64) {
        // SAFETY: APIC_DRIVER lapic_base is set during init and read-only afterwards.
        unsafe {
            let lapic_base = APIC_DRIVER.lapic_base;
            if lapic_base == 0 {
                return;
            }
            // Set LVT to periodic mode (0b01), unmask, with timer vector
            let lvt = LAPIC_TIMER_VECTOR | LAPIC_TIMER_MODE_PERIODIC;
            apic::write_lapic(lapic_base, apic::LAPIC_TIMER_LVT, lvt);
            // Calculate initial count and divider
            let freq = TSC_FREQUENCY;
            if freq == 0 {
                return;
            }
            let icr = ns * freq / 1_000_000_000;
            // For large intervals, use DCR divider to reduce ICR
            let (icr_val, dcr_val) = if icr <= u32::MAX as u64 {
                (icr as u32, LAPIC_TIMER_DIVIDE_1)
            } else {
                // Divide by 16 (DCR = 0b0010)
                let dcr: u32 = 0b0010;
                let divided = icr / 16;
                let val = if divided > u32::MAX as u64 { u32::MAX } else { divided as u32 };
                (val, dcr)
            };
            apic::write_lapic(lapic_base, apic::LAPIC_TIMER_DCR, dcr_val);
            apic::write_lapic(lapic_base, apic::LAPIC_TIMER_ICR, icr_val);
        }
    }

    fn stop(&self) {
        // SAFETY: APIC_DRIVER lapic_base is set during init and read-only afterwards.
        unsafe {
            let lapic_base = APIC_DRIVER.lapic_base;
            if lapic_base == 0 {
                return;
            }
            // Mask the timer LVT entry (bit 16)
            let lvt = apic::read_lapic(lapic_base, apic::LAPIC_TIMER_LVT);
            apic::write_lapic(lapic_base, apic::LAPIC_TIMER_LVT, lvt | LAPIC_TIMER_LVT_MASK);
            // Zero the initial count
            apic::write_lapic(lapic_base, apic::LAPIC_TIMER_ICR, 0);
        }
    }

    fn frequency(&self) -> u64 {
        // SAFETY: TSC_FREQUENCY is read-only after init; no concurrent write hazard.
        unsafe { TSC_FREQUENCY }
    }

    fn delay(&self, ns: u64) {
        let start = Self::now();
        let deadline = start + ns;

        while Self::now() < deadline {
            core::hint::spin_loop();
        }
    }
}

/// IPI wakeup vector
const WAKEUP_IPI_VECTOR: u32 = 0x30;

/// x86_64 Power Management Implementation (ACPI)
pub struct X64Power;

impl PowerOps for X64Power {
    fn init(&self) {
        log_info!("x86_64: Initializing ACPI power management");
        // SAFETY: init_acpi_power is called only during early single-threaded init.
        if !crate::hal::acpi::init_acpi_power() {
            log_warn!("x86_64: ACPI power driver init failed, using fallback");
        }
    }

    fn cpu_idle(&self) {
        // SAFETY: inline assembly required for hardware instruction
        unsafe {
            core::arch::asm!("hlt");
        }
    }

    fn cpu_sleep(&self) {
        // Execute MWAIT with C3 hint (eax=0x10, ecx=0)
        // SAFETY: MWAIT instruction puts the CPU into a low-power C-state;
        // the CPU wakes on any interrupt. eax=0x10 selects C3 (deep sleep).
        unsafe {
            let eax: u32 = 0x10;
            let ecx: u32 = 0;
            core::arch::asm!("mwait", in("eax") eax, in("ecx") ecx, options(nomem, nostack));
        }
    }

    fn cpu_wakeup(&self, cpu_id: u32) {
        // Send IPI to target CPU via LAPIC ICR to wake from C-state
        // SAFETY: APIC_DRIVER lapic_base is set during init and read-only afterwards.
        // Writing to LAPIC ICR sends an inter-processor interrupt, which wakes the
        // target CPU from any MWAIT/WAIT-for-interrupt state.
        unsafe {
            let lapic_base = APIC_DRIVER.lapic_base;
            if lapic_base == 0 {
                return;
            }
            // Write ICR high: destination APIC ID in bits 24-31
            apic::write_lapic(lapic_base, apic::LAPIC_ICR_HIGH, cpu_id << 24);
            // Write ICR low: delivery mode=fixed (000), destination mode=physical (0),
            // level=assert (1), trigger=edge (0), vector=WAKEUP_IPI_VECTOR
            let icr_low = WAKEUP_IPI_VECTOR | (1 << 14);
            apic::write_lapic(lapic_base, apic::LAPIC_ICR_LOW, icr_low);
        }
    }

    fn system_shutdown(&self) {
        log_info!("x86_64: System shutdown (ACPI S5)");
        // Write SLP_TYP S5 to PM1a/b_CNT to enter soft-off state
        crate::hal::acpi::enter_sleep_state(crate::hal::acpi::sleep_type::S5);
    }

    fn system_reboot(&self) {
        log_info!("x86_64: System reboot via FADT reset register");
        // Use FADT reset register for system reset
        crate::hal::acpi::get_acpi_power_driver().system_reset();
    }

    fn system_suspend(&self) {
        log_info!("x86_64: System suspend (ACPI S3)");
        // Write SLP_TYP S3 to PM1a/b_CNT to enter suspend-to-RAM
        crate::hal::acpi::enter_sleep_state(crate::hal::acpi::sleep_type::S3);
    }
}

/// x86_64 Context Operation Implementation
pub struct X64Context;

impl ContextOps for X64Context {
    fn save_context(ctx: &mut CpuContext) {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            // Save general registers
            core::arch::asm!(
                "mov [{0}], rax",
                "mov [{0} + 8], rbx",
                "mov [{0} + 16], rcx",
                "mov [{0} + 24], rdx",
                "mov [{0} + 32], rsi",
                "mov [{0} + 40], rdi",
                "mov [{0} + 48], rbp",
                "mov [{0} + 56], r8",
                "mov [{0} + 64], r9",
                "mov [{0} + 72], r10",
                "mov [{0} + 80], r11",
                "mov [{0} + 88], r12",
                "mov [{0} + 96], r13",
                "mov [{0} + 104], r14",
                "mov [{0} + 112], r15",
                in(reg) ctx.regs.as_mut_ptr() as *mut u8,
            );

            // Save stack pointer and RFLAGS
            core::arch::asm!(
                "mov {0}, rsp",
                "pushfq",
                "pop {1}",
                out(reg) ctx.sp,
                out(reg) ctx.pstate,
            );
        }
    }

    fn restore_context(ctx: &CpuContext) {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            // Restore stack pointer and RFLAGS
            core::arch::asm!(
                "mov rsp, {0}",
                "push {1}",
                "popfq",
                in(reg) ctx.sp,
                in(reg) ctx.pstate,
            );

            // Restore general registers
            core::arch::asm!(
                "mov rax, [{0}]",
                "mov rbx, [{0} + 8]",
                "mov rcx, [{0} + 16]",
                "mov rdx, [{0} + 24]",
                "mov rsi, [{0} + 32]",
                "mov rdi, [{0} + 40]",
                "mov rbp, [{0} + 48]",
                "mov r8, [{0} + 56]",
                "mov r9, [{0} + 64]",
                "mov r10, [{0} + 72]",
                "mov r11, [{0} + 80]",
                "mov r12, [{0} + 88]",
                "mov r13, [{0} + 96]",
                "mov r14, [{0} + 104]",
                "mov r15, [{0} + 112]",
                in(reg) ctx.regs.as_ptr() as *const u8,
            );
        }
    }

    fn switch_context(from: &mut CpuContext, to: &CpuContext) {
        Self::save_context(from);
        Self::restore_context(to);
    }
}

/// x86_64 Architecture Implementation
pub struct X64Arch;

impl ArchOps for X64Arch {
    fn init() {
        let vendor = get_cpu_vendor();
        let vendor_str = core::str::from_utf8(&vendor).unwrap_or("Unknown");

        log_info!("x86_64 architecture initialized");
        log_info!("  CPU Vendor: {}", vendor_str);

        // Initialize subsystems
        Self::irq_controller().init();
        Self::timer().init();
        Self::power().init();
    }

    fn page_table() -> &'static dyn PageTableOps {
        &X64PageTable
    }

    fn irq_controller() -> &'static dyn IrqControllerOps {
        &X64IrqController
    }

    fn timer() -> &'static dyn TimerOps {
        &X64Timer
    }

    fn power() -> &'static dyn PowerOps {
        &X64Power
    }

    fn context() -> &'static dyn ContextOps {
        &X64Context
    }

    fn enable_irq() {
        // SAFETY: inline assembly required for hardware instruction
        unsafe {
            core::arch::asm!("sti");
        }
    }

    fn disable_irq() {
        // SAFETY: inline assembly required for hardware instruction
        unsafe {
            core::arch::asm!("cli");
        }
    }

    fn cpu_id() -> u32 {
        // TODO: Read Local APIC ID
        0
    }

    fn cpu_count() -> u32 {
        // TODO: Read CPU count from ACPI or MP table
        1
    }
}

/// Global x86_64 architecture instance
pub static X64_ARCH: X64Arch = X64Arch;
