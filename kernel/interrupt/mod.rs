/*
 * Nuva OS - Kernel - Interrupt
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


// Re-export print macros from crate root
use crate::posix::errno::Errno;

pub use crate::{pr_emerg, pr_alert, pr_crit, pr_err, pr_warn, pr_notice, pr_info, pr_debug};
pub mod generic;
pub mod gic;

// TODO: irqchip module not yet in driver; re-export when available
// pub use crate::kernel::driver::irqchip::gic;

// Re-export generic interrupt types
pub use generic::{
    IrqReturn, ExceptionType, ExceptionContext, ExceptionHandler,
    SoftIrqHandler, IrqNumber, IrqFlags, IrqAction, InterruptManager,
    get_interrupt_manager, init_interrupt,
};

/// InterruptsignalDefinition
pub mod irq {
 /// SGI (Software Generated Interrupts): 0-15
 pub const SGI_START: u32 = 0;
 pub const SGI_END: u32 = 15;
 
 /// PPI (Private Peripheral Interrupts): 16-31
 pub const PPI_START: u32 = 16;
 pub const PPI_END: u32 = 31;
 
 /// SPI (Shared Peripheral Interrupts): 32-1019
 pub const SPI_START: u32 = 32;
 pub const SPI_END: u32 = 1019;
 
 // constantuseInterruptsignal
 pub const TIMER_IRQ: u32 = 30; // GeneralTimer
 pub const UART0_IRQ: u32 = 33; // UART0
 pub const UART1_IRQ: u32 = 34; // UART1
 pub const GPU_IRQ: u32 = 100; // GPU
 pub const NPU_IRQ: u32 = 101; // NPU
}

/// InterruptFlag
pub mod irq_flags {
 pub const IRQF_TRIGGER_RISING: u32 = 0x0001; // Rising EdgeTrigger
 pub const IRQF_TRIGGER_FALLING: u32 = 0x0002; // Falling EdgeTrigger
 pub const IRQF_TRIGGER_HIGH: u32 = 0x0004; // High LevelTrigger
 pub const IRQF_TRIGGER_LOW: u32 = 0x0008; // Low LevelTrigger
 pub const IRQF_SHARED: u32 = 0x0010; // SharedInterrupt
 pub const IRQF_PROBE_SHARED: u32 = 0x0020; // ProbeShared
 pub const IRQF_TIMER: u32 = 0x0040; // TimerInterrupt
 pub const IRQF_PERCPU: u32 = 0x0080; // Per CPU Interrupt
 pub const IRQF_NOBALANCING: u32 = 0x0100; // DisableLoad Balancing
 pub const IRQF_IRQPOLL: u32 = 0x0200; // InterruptPolling
 pub const IRQF_ONESHOT: u32 = 0x0400; // One-shotTrigger
 pub const IRQF_NO_SUSPEND: u32 = 0x0800; // DisableSuspend
 pub const IRQF_FORCE_RESUME: u32 = 0x1000; // ForceRecovery
 pub const IRQF_NO_THREAD: u32 = 0x2000; // DisableThread
 pub const IRQF_EARLY_RESUME: u32 = 0x4000; // EarlyRecovery
}

/// InterruptHandleFunctionType
pub type IrqHandler = extern "C" fn(irq: u32, arg: *mut u8);

/// InterruptDescriptor
pub struct IrqDesc {
 pub irq: u32,
 pub name: &'static str,
 pub handler: IrqHandler,
 pub flags: u32,
 pub arg: *mut u8,
 pub count: u64,
}

/// Interruptstatistics
pub struct IrqStats {
 pub total: u64,
 pub per_irq: [u64; 1024],
}

impl IrqStats {
 pub const fn new() -> Self {
 IrqStats {
 total: 0,
 per_irq: [0; 1024],
 }
 }
 
 pub fn record(&mut self, irq: u32) {
 self.total += 1;
 if (irq as usize) < self.per_irq.len() {
 self.per_irq[irq as usize] += 1;
 }
 }
}

/// GlobalInterruptStatistics
static IRQ_STATS: core::sync::OnceLock<IrqStats> = core::sync::OnceLock::new();

/// InitializeInterruptSystem
pub fn init_interrupts() {
 log_info!("Initializing interrupt system...");
 
 // Initialize GIC
// TODO: gic module stub
mod gic {
    pub fn gic_init() {}
    pub fn gic_enable_irq(_: u32) {}
    pub fn gic_disable_irq(_: u32) {}
    pub fn gic_ack_irq() -> u32 { 0 }
    pub fn gic_eoi_irq(_: u32) {}
    pub fn gic_set_priority(_: u32, _: u32) {}
    pub fn gic_get_priority(_: u32) -> u32 { 0 }
}
 gic::gic_init();
 
 // InitializeExceptionHandle
 #[cfg(target_arch = "aarch64")]
 crate::kernel::arch::arm64::trap::handler::init_exceptions();
 
 log_info!("Interrupt system initialized");
}

/// RegisterInterruptHandleFunction
pub fn request_irq(
 irq: u32,
 handler: crate::kernel::driver::irq::IrqHandler,
 flags: u32,
 name: &'static str,
 arg: *mut crate::kernel::driver::irq::IrqContext,
) -> i32 {
 // Registerto GIC
 if !gic::register_irq(irq, handler, arg) {
 log_error!("Failed to register IRQ {}", irq);
 return Errno::Eperm.to_ret_i32();
 }
 
 // SetInterruptType
 if flags & irq_flags::IRQF_TRIGGER_RISING != 0 ||
 flags & irq_flags::IRQF_TRIGGER_FALLING != 0 {
 if let Some(gic) = gic::get_gic() {
 gic.set_irq_type(irq, true); // Edge-triggered
 }
 }
 
 // makecanInterrupt
 gic::enable_irq(irq);
 
 log_info!("IRQ {} ({}) registered", irq, name);
 0
}

/// FreeInterrupt
pub fn free_irq(irq: u32) {
 gic::disable_irq(irq);
 log_info!("IRQ {} freed", irq);
}

/// makecanInterrupt
pub fn enable_irq(irq: u32) {
 gic::enable_irq(irq);
}

/// DisableInterrupt
pub fn disable_irq(irq: u32) {
 gic::disable_irq(irq);
}

/// RecordInterruptStatistics
pub fn record_irq(irq: u32) {
 // SAFETY: unsafe block required for low-level memory or hardware access
 unsafe {
 IRQ_STATS.record(irq);
 }
}

/// GetInterruptstatistics
pub fn get_irq_stats() -> &'static IrqStats {
 // SAFETY: unsafe block required for low-level memory or hardware access
 unsafe { &IRQ_STATS }
}

/// printstampInterruptStatistics
pub fn print_irq_stats() {
 let stats = get_irq_stats();
 
 log_info!("=== IRQ Statistics ===");
 log_info!("Total: {}", stats.total);
 
 // printstampprefix 10 itemmostactive Interrupt
 log_info!("Top 10 active IRQs:");
 
 let mut top_irqs: [(u32, u64); 10] = [(0, 0); 10];
 
 for (irq, &count) in stats.per_irq.iter().enumerate() {
 if count == 0 {
 continue;
 }
 
 // Insertto top List
 for i in 0..10 {
 if count > top_irqs[i].1 {
 // MovethenFace prime
 for j in (i + 1..10).rev() {
 top_irqs[j] = top_irqs[j - 1];
 }
 top_irqs[i] = (irq as u32, count);
 break;
 }
 }
 }
 
 for (irq, count) in top_irqs.iter() {
 if *count > 0 {
 log_info!(" IRQ {}: {}", irq, count);
 }
 }
}

/// InterruptprotectedRange
pub struct IrqSave {
 was_enabled: bool,
}

impl IrqSave {
 /// DisableInterruptparallelSaveState
 pub fn new() -> Self {
 #[cfg(target_arch = "aarch64")]
 let was_enabled = crate::kernel::arch::arm64::trap::handler::irqs_enabled();
 #[cfg(not(target_arch = "aarch64"))]
 let was_enabled = false;
 if was_enabled {
     #[cfg(target_arch = "aarch64")]
     crate::kernel::arch::arm64::trap::handler::disable_irq();
 }
 IrqSave { was_enabled }
 }
}

impl Drop for IrqSave {
 /// RecoveryInterruptState
 fn drop(&mut self) {
 if self.was_enabled {
     #[cfg(target_arch = "aarch64")]
     crate::kernel::arch::arm64::trap::handler::enable_irq();
 }
 }
}

/// DisableInterrupt Macro
#[macro_export]
macro_rules! irq_save {
 () => {
 $crate::interrupt::IrqSave::new()
 };
}

/// CheckInterruptifmakecan
pub fn irqs_enabled() -> bool {
    #[cfg(target_arch = "aarch64")]
    {
        crate::kernel::arch::arm64::trap::handler::irqs_enabled()
    }
    #[cfg(not(target_arch = "aarch64"))]
    {
        false
    }
}

/// inInterruptContextinfixexecuteclosedPackage
pub fn with_irqs_disabled<F, R>(f: F) -> R
where
 F: FnOnce() -> R,
{
 let _save = IrqSave::new();
 f()
}

#[cfg(test)]
mod tests {
 use super::*;
 
 extern "C" fn test_handler(_irq: u32, _arg: *mut u8) {
 // TestHandleFunction
 }
 
 #[test]
 fn test_irq_registration() {
 let result = request_irq(
 100,
 test_handler,
 irq_flags::IRQF_SHARED,
 "test",
 core::ptr::null_mut(),
 );
 assert_eq!(result, 0);
 }
}