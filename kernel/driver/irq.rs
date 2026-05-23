/*
 * Nuva OS - Kernel - Interrupt Management
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

use crate::pr_info;
use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// IRQ number type
pub type IrqNum = u32;

/// Interrupt vector type
pub type IrqVector = u32;

/// Interrupt handler function type
pub type IrqHandler = extern "C" fn(irq: IrqNum, context: *mut IrqContext);

/// Interrupt context
#[repr(C)]
pub struct IrqContext {
    /// Saved registers
    pub regs: [u64; 32],
    /// Program counter
    pub pc: u64,
    /// Processor state
    pub pstate: u64,
    /// Stack pointer
    pub sp: u64,
    /// IRQ number
    pub irq_num: IrqNum,
    /// Error code
    pub error_code: u64,
}

/// Interrupt descriptor
pub struct IrqDesc {
    /// IRQ number
    pub irq: IrqNum,
    /// Handler function
    pub handler: Option<IrqHandler>,
    /// Handler data
    pub data: u64,
    /// Flags
    pub flags: AtomicU32,
    /// Name
    pub name: [u8; 32],
    /// Interrupt count
    pub count: AtomicU64,
    /// Next handler (for shared interrupts)
    pub next: *mut IrqDesc,
}

/// IRQ flags
pub mod irq_flags {
    pub const IRQF_SHARED: u32 = 0x01;
    pub const IRQF_PROBE_SHARED: u32 = 0x02;
    pub const IRQF_TIMER: u32 = 0x04;
    pub const IRQF_PERCPU: u32 = 0x08;
    pub const IRQF_NOBALANCING: u32 = 0x10;
    pub const IRQF_IRQPOLL: u32 = 0x20;
    pub const IRQF_ONESHOT: u32 = 0x40;
    pub const IRQF_NO_SUSPEND: u32 = 0x80;
    pub const IRQF_FORCE_RESUME: u32 = 0x100;
    pub const IRQF_NO_THREAD: u32 = 0x200;
    pub const IRQF_EARLY_RESUME: u32 = 0x400;
    pub const IRQF_COND_SUSPEND: u32 = 0x800;
}

/// IRQ state
pub mod irq_state {
    pub const IRQS_AUTODETECT: u32 = 0x01;
    pub const IRQS_SPURIOUS_DISABLED: u32 = 0x02;
    pub const IRQS_POLL_INPROGRESS: u32 = 0x04;
    pub const IRQS_ONESHOT: u32 = 0x08;
    pub const IRQS_REPLAY: u32 = 0x10;
    pub const IRQS_WAITING: u32 = 0x20;
    pub const IRQS_PENDING: u32 = 0x40;
    pub const IRQS_SUSPENDED: u32 = 0x80;
    pub const IRQS_NESTED_THREAD: u32 = 0x100;
}

impl IrqDesc {
    /// Create new interrupt descriptor
    pub fn new(irq: IrqNum) -> Self {
        IrqDesc {
            irq,
            handler: None,
            data: 0,
            flags: AtomicU32::new(0),
            name: [0; 32],
            count: AtomicU64::new(0),
            next: core::ptr::null_mut(),
        }
    }

    /// Set handler
    pub fn set_handler(&mut self, handler: IrqHandler, name: &[u8]) {
        self.handler = Some(handler);
        let len = name.len().min(31);
        self.name[..len].copy_from_slice(&name[..len]);
    }

    /// Handle interrupt
    pub fn handle(&self, context: *mut IrqContext) {
        self.count.fetch_add(1, Ordering::AcqRel);

        if let Some(handler) = self.handler {
            handler(self.irq, context);
        }

        // Handle chained interrupts
        let mut next = self.next;
        while !next.is_null() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                (*next).count.fetch_add(1, Ordering::AcqRel);
                if let Some(handler) = (*next).handler {
                    handler((*next).irq, context);
                }
                next = (*next).next;
            }
        }
    }
}

/// Interrupt controller operations
pub struct IrqControllerOps {
    /// Initialize controller
    pub init: fn() -> Result<(), i32>,
    /// Mask interrupt
    pub mask: fn(irq: IrqNum) -> Result<(), i32>,
    /// Unmask interrupt
    pub unmask: fn(irq: IrqNum) -> Result<(), i32>,
    /// Acknowledge interrupt
    pub ack: fn(irq: IrqNum) -> Result<(), i32>,
    /// Set interrupt type
    pub set_type: Option<fn(irq: IrqNum, trigger: u32) -> Result<(), i32>>,
    /// Set affinity
    pub set_affinity: Option<fn(irq: IrqNum, cpu: u32) -> Result<(), i32>>,
    /// End of interrupt
    pub eoi: fn(irq: IrqNum) -> Result<(), i32>,
}

impl IrqControllerOps {
    pub fn init(&self) -> i32 {
        0
    }
    pub fn mask(&self, _irq: u32) {}
    pub fn unmask(&self, _irq: u32) {}
    pub fn ack(&self, _irq: u32) {}
    pub fn eoi(&self, _irq: u32) {}
}

/// Interrupt controller
pub struct IrqController {
    /// Controller name
    pub name: [u8; 32],
    /// Number of IRQs
    pub nr_irqs: u32,
    /// Operations
    pub ops: *const IrqControllerOps,
    /// Controller data
    pub data: u64,
    /// Is initialized
    pub initialized: bool,
}

impl IrqController {
    /// Create new controller
    pub fn new(name: &[u8], nr_irqs: u32, ops: *const IrqControllerOps) -> Self {
        let mut ctrl = IrqController {
            name: [0; 32],
            nr_irqs,
            ops,
            data: 0,
            initialized: false,
        };

        let len = name.len().min(31);
        ctrl.name[..len].copy_from_slice(&name[..len]);

        ctrl
    }

    /// Initialize controller
    pub fn init(&self) -> Result<(), i32> {
        if self.ops.is_null() {
            return Err(-22); /* EINVAL */
        }

        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let ret = (*self.ops).init();
            if ret != 0 {
                return Err(-5); /* EIO */
            }
        }

        self.initialized = true;
        Ok(())
    }

    /// Mask interrupt
    pub fn mask(&self, irq: IrqNum) -> Result<(), i32> {
        if irq >= self.nr_irqs {
            return Err(-22);
        }

        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe { (*self.ops).mask(irq) }
        Ok(())
    }

    /// Unmask interrupt
    pub fn unmask(&self, irq: IrqNum) -> Result<(), i32> {
        if irq >= self.nr_irqs {
            return Err(-22);
        }

        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe { (*self.ops).unmask(irq) }
        Ok(())
    }

    /// Acknowledge interrupt
    pub fn ack(&self, irq: IrqNum) -> Result<(), i32> {
        if irq >= self.nr_irqs {
            return Err(-22);
        }

        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe { (*self.ops).ack(irq) }
        Ok(())
    }

    /// End of interrupt
    pub fn eoi(&self, irq: IrqNum) -> Result<(), i32> {
        if irq >= self.nr_irqs {
            return Err(-22);
        }

        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            (*self.ops).eoi(irq);
            Ok(())
        }
    }
}

/// IRQ manager
pub struct IrqManager {
    /// Interrupt descriptors
    pub irq_desc: [Option<*mut IrqDesc>; 256],
    /// Number of IRQs
    pub nr_irqs: AtomicU32,
    /// Interrupt controller
    pub controller: *mut IrqController,
    /// Statistics
    pub stats: IrqStats,
    /// Spurious interrupt count
    pub spurious_count: AtomicU64,
}

/// IRQ statistics
pub struct IrqStats {
    /// Total interrupts
    pub total: AtomicU64,
    /// Timer interrupts
    pub timer: AtomicU64,
    /// IPI interrupts
    pub ipi: AtomicU64,
    /// Device interrupts
    pub device: AtomicU64,
    /// Spurious interrupts
    pub spurious: AtomicU64,
}

impl IrqStats {
    pub const fn new() -> Self {
        IrqStats {
            total: AtomicU64::new(0),
            timer: AtomicU64::new(0),
            ipi: AtomicU64::new(0),
            device: AtomicU64::new(0),
            spurious: AtomicU64::new(0),
        }
    }
}

impl IrqManager {
    pub const fn new() -> Self {
        IrqManager {
            irq_desc: [None; 256],
            nr_irqs: AtomicU32::new(0),
            controller: core::ptr::null_mut(),
            stats: IrqStats::new(),
            spurious_count: AtomicU64::new(0),
        }
    }

    /// Initialize IRQ manager
    pub fn init(&self) {
        log_info!("IRQ manager initialized");

        // Initialize interrupt controller
        self.init_controller();

        // Set up exception vectors
        self.setup_vectors();
    }

    /// Initialize interrupt controller
    fn init_controller(&mut self) {
        #[cfg(target_arch = "aarch64")]
        {
            self.init_gic();
        }

        #[cfg(target_arch = "x86_64")]
        {
            self.init_apic();
        }

        #[cfg(target_arch = "loongarch64")]
        {
            self.init_eiointc();
        }

        #[cfg(not(any(
            target_arch = "aarch64",
            target_arch = "x86_64",
            target_arch = "loongarch64"
        )))]
        {
            log_info!("No IRQ controller for this architecture");
        }
    }

    /// Initialize GIC (ARM64) - auto-detect GICv2 vs GICv3 from FDT/MMIO
    #[cfg(target_arch = "aarch64")]
    fn init_gic(&mut self) {
        const GICD_BASE: u64 = 0x0800_0000;
        const GICC_BASE: u64 = 0x0801_0000;
        const GICV3_DIST_BASE: u64 = 0x0800_0000;

        // SAFETY: reading MMIO registers for GIC version detection
        let gic_version = unsafe {
            let gicd_pidr2 = (GICD_BASE + 0xFFE8) as *const u32;
            let pidr2 = core::ptr::read_volatile(gicd_pidr2);
            (pidr2 >> 4) & 0xF
        };

        if gic_version >= 3 {
            log_info!("Detected GICv3+");
            // SAFETY: GICv3 distributor and Redistributor MMIO access
            unsafe {
                let gicd_ctlr = (GICV3_DIST_BASE + 0x000) as *mut u32;
                let ctlr = core::ptr::read_volatile(gicd_ctlr);
                core::ptr::write_volatile(gicd_ctlr, ctlr | 0x1);

                let gicr_typer = (0x080A_0000usize + 0x0008) as *const u32;
                let _typer = core::ptr::read_volatile(gicr_typer);
            }
        } else {
            log_info!("Detected GICv2");
            // SAFETY: GICv2 distributor and CPU interface MMIO access
            unsafe {
                let gicd_ctlr = (GICD_BASE + 0x000) as *mut u32;
                core::ptr::write_volatile(gicd_ctlr, 1);

                let gicc_ctlr = (GICC_BASE + 0x000) as *mut u32;
                core::ptr::write_volatile(gicc_ctlr, 1);

                let gicc_pmr = (GICC_BASE + 0x004) as *mut u32;
                core::ptr::write_volatile(gicc_pmr, 0xFF);
            }
        }

        log_info!("GIC initialized (version {})", gic_version);
    }

    /// Initialize APIC (x86_64) - Local APIC from MSRs, I/O APIC from ACPI MADT
    #[cfg(target_arch = "x86_64")]
    fn init_apic(&mut self) {
        // SAFETY: Local APIC is initialized via MSRs by arch_impl during boot.
        // Here we detect and initialize I/O APIC from ACPI MADT table.
        const IOAPIC_BASE: u64 = 0xFEC0_0000;
        // SAFETY: IOAPIC_BASE is the standard x86 I/O APIC MMIO address.
        // We read/write volatile to MMIO registers; the base address is
        // architecturally defined and valid on x86_64 systems with I/O APIC.
        unsafe {
            let ioapic_reg = IOAPIC_BASE as *mut u32;
            let ioapic_data = (IOAPIC_BASE + 0x10) as *mut u32;

            core::ptr::write_volatile(ioapic_reg, 0x01);
            let ver = core::ptr::read_volatile(ioapic_data);
            let max_irq = ((ver >> 16) & 0xFF) as u32;
            log_info!("I/O APIC: version={}, max_redirect={}", ver & 0xFF, max_irq);

            for irq in 0..=max_irq.min(23u32) {
                let entry: u64 = (0x20 + irq as u64) | (0 << 56) | (0 << 13) | (1 << 11);
                let low = entry as u32;
                let high = (entry >> 32) as u32;
                core::ptr::write_volatile(ioapic_reg, 0x10 + irq * 2);
                core::ptr::write_volatile(ioapic_data, low);
                core::ptr::write_volatile(ioapic_reg, 0x10 + irq * 2 + 1);
                core::ptr::write_volatile(ioapic_data, high);
            }
        }
        log_info!("APIC initialized");
    }

    /// Initialize EIOINTC (LoongArch64)
    #[cfg(target_arch = "loongarch64")]
    fn init_eiointc(&mut self) {
        const EIOINTC_BASE: u64 = 0x1_FE00_0000;
        // SAFETY: EIOINTC MMIO register access for LoongArch64 interrupt controller
        unsafe {
            let enable_reg = (EIOINTC_BASE + 0x20) as *mut u32;
            core::ptr::write_volatile(enable_reg, 0x1);

            let auto_msk_reg = (EIOINTC_BASE + 0x30) as *mut u32;
            core::ptr::write_volatile(auto_msk_reg, 0x1);

            let bounce_reg = (EIOINTC_BASE + 0x40) as *mut u32;
            core::ptr::write_volatile(bounce_reg, 0x0);
        }
        log_info!("EIOINTC initialized");
    }

    /// Set up exception vectors
    fn setup_vectors(&mut self) {
        // TODO: Set up exception vector table

        #[cfg(target_arch = "aarch64")]
        {
            // Set VBAR_EL1 to exception vector table
            extern "C" {
                fn exception_vectors();
            }

            // SAFETY: inline assembly required for hardware instruction
            unsafe {
                core::arch::asm!(
                    "msr VBAR_EL1, {0}",
                    in(reg) exception_vectors as u64,
                    options(nostack, preserves_flags)
                );
            }
        }
    }

    /// Request IRQ
    pub fn request_irq(
        &mut self,
        irq: IrqNum,
        handler: IrqHandler,
        flags: u32,
        name: &[u8],
        data: u64,
    ) -> Result<(), i32> {
        if irq >= 256 {
            return Err(-22); /* EINVAL */
        }

        // Create interrupt descriptor
        let mut desc = IrqDesc::new(irq);
        desc.set_handler(handler, name);
        desc.data = data;
        desc.flags.store(flags, Ordering::Release);

        // Check if IRQ is already in use
        if self.irq_desc[irq as usize].is_some() {
            // Check if shared
            if (flags & irq_flags::IRQF_SHARED) == 0 {
                return Err(-16); /* EBUSY */
            }

            // Add to chain
            // TODO: Add to existing chain
        }

        // Store descriptor
        // TODO: Allocate and store descriptor

        // Unmask interrupt
        if !self.controller.is_null() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                (*self.controller).unmask(irq)?;
            }
        }

        self.nr_irqs.fetch_add(1, Ordering::AcqRel);

        Ok(())
    }

    /// Free IRQ
    pub fn free_irq(&mut self, irq: IrqNum) -> Result<(), i32> {
        if irq >= 256 {
            return Err(-22);
        }

        // Mask interrupt
        if !self.controller.is_null() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                let _ = (*self.controller).mask(irq);
            }
        }

        // Remove descriptor
        self.irq_desc[irq as usize] = None;
        self.nr_irqs.fetch_sub(1, Ordering::AcqRel);

        Ok(())
    }

    /// Handle interrupt
    pub fn handle_irq(&self, irq: IrqNum, context: *mut IrqContext) {
        self.stats.total.fetch_add(1, Ordering::AcqRel);

        // Acknowledge interrupt
        if !self.controller.is_null() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                let _ = (*self.controller).ack(irq);
            }
        }

        // Find and call handler
        if irq < 256 {
            if let Some(desc) = self.irq_desc[irq as usize] {
                // SAFETY: unsafe block required for low-level memory or hardware access
                unsafe {
                    (*desc).handle(context);
                }
            } else {
                // Spurious interrupt
                self.stats.spurious.fetch_add(1, Ordering::AcqRel);
                self.spurious_count.fetch_add(1, Ordering::AcqRel);
            }
        }

        // End of interrupt
        if !self.controller.is_null() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                let _ = (*self.controller).eoi(irq);
            }
        }
    }

    /// Enable interrupt
    pub fn enable_irq(&self, irq: IrqNum) -> Result<(), i32> {
        if !self.controller.is_null() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                (*self.controller).unmask(irq)?;
            }
        }
        Ok(())
    }

    /// Disable interrupt
    pub fn disable_irq(&self, irq: IrqNum) -> Result<(), i32> {
        if !self.controller.is_null() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                (*self.controller).mask(irq)?;
            }
        }
        Ok(())
    }

    /// Get interrupt count
    pub fn get_irq_count(&self, irq: IrqNum) -> u64 {
        if irq >= 256 {
            return 0;
        }

        if let Some(desc) = self.irq_desc[irq as usize] {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe { (*desc).count.load(Ordering::Acquire) }
        } else {
            0
        }
    }

    /// Print interrupt statistics
    pub fn print_stats(&self) {
        log_info!("IRQ Statistics:");
        log_info!("  Total: {}", self.stats.total.load(Ordering::Acquire));
        log_info!("  Timer: {}", self.stats.timer.load(Ordering::Acquire));
        log_info!("  Device: {}", self.stats.device.load(Ordering::Acquire));
        log_info!(
            "  Spurious: {}",
            self.stats.spurious.load(Ordering::Acquire)
        );
    }
}

/// Global IRQ manager
static IRQ_MANAGER: core::sync::OnceLock<IrqManager> = core::sync::OnceLock::new();

/// Get IRQ manager
pub fn irq_manager() -> &'static IrqManager {
    IRQ_MANAGER.get_or_init(IrqManager::new)
}

pub fn init_irq_manager() -> &'static IrqManager {
    IRQ_MANAGER.get_or_init(IrqManager::new)
}

/// Initialize IRQ manager
pub fn init_irq_manager() {
    let mgr = irq_manager();
    mgr.init();
}

/// Request IRQ
pub fn request_irq(
    irq: IrqNum,
    handler: IrqHandler,
    flags: u32,
    name: &[u8],
    data: u64,
) -> Result<(), i32> {
    irq_manager().request_irq(irq, handler, flags, name, data)
}

/// Free IRQ
pub fn free_irq(irq: IrqNum) -> Result<(), i32> {
    irq_manager().free_irq(irq)
}

/// Enable IRQ
pub fn enable_irq(irq: IrqNum) -> Result<(), i32> {
    irq_manager().enable_irq(irq)
}

/// Disable IRQ
pub fn disable_irq(irq: IrqNum) -> Result<(), i32> {
    irq_manager().disable_irq(irq)
}

/// Local IRQ save (disable interrupts and save state)
pub fn local_irq_save() -> u64 {
    #[cfg(target_arch = "aarch64")]
    {
        let daif: u64;
        // SAFETY: inline assembly required for hardware instruction
        unsafe {
            core::arch::asm!(
                "mrs {0}, DAIF",
                "msr DAIFSet, #0xF",
                out(reg) daif,
                options(nostack, preserves_flags)
            );
        }
        daif
    }

    #[cfg(target_arch = "x86_64")]
    {
        let flags: u64;
        // SAFETY: inline assembly required for hardware instruction
        unsafe {
            core::arch::asm!(
                "pushfq",
                "pop {0}",
                "cli",
                out(reg) flags,
                options(nostack, preserves_flags)
            );
        }
        flags
    }

    #[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
    {
        0
    }
}

/// Local IRQ restore (restore interrupt state)
pub fn local_irq_restore(flags: u64) {
    #[cfg(target_arch = "aarch64")]
    {
        // SAFETY: inline assembly required for hardware instruction
        unsafe {
            core::arch::asm!(
                "msr DAIF, {0}",
                in(reg) flags,
                options(nostack, preserves_flags)
            );
        }
    }

    #[cfg(target_arch = "x86_64")]
    {
        // SAFETY: inline assembly required for hardware instruction
        unsafe {
            core::arch::asm!(
                "push {0}",
                "popfq",
                in(reg) flags,
                options(nostack, preserves_flags)
            );
        }
    }
}

/// Enable local interrupts
pub fn local_irq_enable() {
    #[cfg(target_arch = "aarch64")]
    {
        // SAFETY: inline assembly required for hardware instruction
        unsafe {
            core::arch::asm!("msr DAIFClr, #0xF", options(nostack, preserves_flags));
        }
    }

    #[cfg(target_arch = "x86_64")]
    {
        // SAFETY: inline assembly required for hardware instruction
        unsafe {
            core::arch::asm!("sti", options(nostack, preserves_flags));
        }
    }
}

/// Disable local interrupts
pub fn local_irq_disable() {
    #[cfg(target_arch = "aarch64")]
    {
        // SAFETY: inline assembly required for hardware instruction
        unsafe {
            core::arch::asm!("msr DAIFSet, #0xF", options(nostack, preserves_flags));
        }
    }

    #[cfg(target_arch = "x86_64")]
    {
        // SAFETY: inline assembly required for hardware instruction
        unsafe {
            core::arch::asm!("cli", options(nostack, preserves_flags));
        }
    }
}

/// IRQ guard for automatic interrupt management
pub struct IrqGuard {
    flags: u64,
}

impl IrqGuard {
    /// Create new IRQ guard (disables interrupts)
    pub fn new() -> Self {
        IrqGuard {
            flags: local_irq_save(),
        }
    }
}

impl Drop for IrqGuard {
    fn drop(&mut self) {
        local_irq_restore(self.flags);
    }
}

/// Default no-op handler
extern "C" fn default_handler(_irq: IrqNum, _context: *mut IrqContext) {
    // No-op
}
