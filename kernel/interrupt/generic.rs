/* * Nuva OS - Kernel - InterruptmanagementadministrationsystemaInterface
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

// ! InterruptmanagementadministrationsystemaInterface
/*!*/
// ! systema InterruptsumExceptionHandleInterface,integercombine:
// ! - hardcaseInterruptHandle
//! - ExceptionHandle
// ! - Systemtuneuseenterport
// ! - softInterruptmachinecontrol

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use crate::{pr_debug, pr_err, pr_info, pr_warn};

/// InterruptsignalType
pub type IrqNumber = u32;

/// InterruptHandleFunctionType
pub type IrqHandler = extern "C" fn(IrqNumber, *mut core::ffi::c_void);

/// InterruptReturn Value
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IrqReturn {
 /// Handle
 None = 0,
 /// alreadyHandle
 Handled = 1,
 /// needWakeThread
 WakeThread = 2,
}

/// InterruptFlag
#[derive(Debug, Clone, Copy)]
pub struct IrqFlags(pub u32);

impl IrqFlags {
 pub const NONE: IrqFlags = IrqFlags(0);
 pub const SHARED: IrqFlags = IrqFlags(1 << 0); // SharedInterrupt
 pub const PERCPU: IrqFlags = IrqFlags(1 << 1); // PerCPUInterrupt
 pub const NOTHREAD: IrqFlags = IrqFlags(1 << 2); // notThread
 pub const NOAUTOEN: IrqFlags = IrqFlags(1 << 3); // notselfdynamicmakecan
 pub const NOSUSPEND: IrqFlags = IrqFlags(1 << 4); // notsuspend
 
 pub fn contains(&self, flag: IrqFlags) -> bool {
 (self.0 & flag.0) != 0
 }
}

/// ExceptionType
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExceptionType {
 /// DivideError
 DivideError,
 /// DebuggingException
 Debug,
 /// Breakpoint
 Breakpoint,
 /// Overflow
 Overflow,
 /// invalidOperationcode
 InvalidOpcode,
 /// doublerepeatError
 DoubleFault,
 /// ageneralprotectedError
 GeneralProtection,
 /// pageError
 PageFault,
 /// FPUError
 FpuError,
 /// AlignmentCheck
 AlignmentCheck,
 /// machinedeviceCheck
 MachineCheck,
 /// SIMDException
 SimdException,
 /// Systemcall
 SystemCall,
 /// Exception
 Unknown,
}

/// ExceptionHandleFunctionType
pub type ExceptionHandler = fn(&ExceptionContext);

/// ExceptionContext
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct ExceptionContext {
 /// ExceptionType
 pub exc_type: ExceptionType,
 /// Error code
 pub error_code: u64,
 /// incidentfaultAddress
 pub fault_addr: u64,
 /// Program counter
 pub pc: u64,
 /// Stackpointer
 pub sp: u64,
 /// FlagRegister
 pub flags: u64,
 /// GeneralRegister
 pub regs: [u64; 16],
}

/// softInterruptType
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoftIrqType {
 /// highPriorityTimer
 HiTimer = 0,
 /// Timer
 Timer = 1,
 /// NetworkSend
 NetTx = 2,
 /// NetworkReceive
 NetRx = 3,
 /// BlockDevice
 Block = 4,
 /// TaskEnd
 Tasklet = 5,
 /// RCU
 Rcu = 6,
 /// Maxvalue
 Max = 7,
}

/// softInterruptHandleFunctionType
pub type SoftIrqHandler = fn();

/// InterruptDescriptor
pub struct IrqDesc {
 /// Interruptsignal
 pub irq: IrqNumber,
 /// HandleFunctionlinkform(SupportSharedInterrupt)
 pub handlers: *mut IrqAction,
 /// StateFlag
 pub status: AtomicU32,
 /// DisabledeepDegree
 pub depth: AtomicU32,
 /// TriggerCount
 pub count: AtomicU64,
 /// CPUAffinity
 pub affinity: AtomicU64,
}

/// InterruptAction
#[repr(C)]
pub struct IrqAction {
 /// HandleFunction
 pub handler: IrqHandler,
 /// DeviceName
 pub name: [u8; 32],
 /// privatefiniteData
 pub dev_id: *mut core::ffi::c_void,
 /// NextAction(SharedInterrupt)
 pub next: *mut IrqAction,
 /// Flag
 pub flags: IrqFlags,
}

/// InterruptManager
pub struct InterruptManager {
 /// InterruptDescriptorArray
 irq_descs: [Option<IrqDesc>; 256],
 /// ExceptionHandleform
 exc_handlers: [Option<ExceptionHandler>; 32],
 /// softInterruptHandleform
 softirq_handlers: [Option<SoftIrqHandler>; 8],
 /// softInterruptHandleBitGraph
 softirq_pending: AtomicU64,
 /// InterruptNesteddeepDegree
 irq_depth: AtomicU32,
}

impl InterruptManager {
 /// Create new InterruptManager
 pub const fn new() -> Self {
 InterruptManager {
 irq_descs: [const { None }; 256],
 exc_handlers: [const { None }; 32],
 softirq_handlers: [const { None }; 8],
 softirq_pending: AtomicU64::new(0),
 irq_depth: AtomicU32::new(0),
 }
 }
 
 /// InitializeInterruptManager
 pub fn init(&self) {
 log_info!("Initializing interrupt management");
 
 // InitializeArchitectureCorrelation InterruptControldevice
 crate::kernel::arch::current_arch().irq_controller().init();
 
 // RegisterDefaultExceptionHandleFunction
 self.register_default_exception_handlers();
 
 log_info!("Interrupt management initialized");
 }
 
 /// RegisterInterruptHandleFunction
 /// # Parameter
 /// - `irq`: Interruptsignal
 /// - `handler`: HandleFunction
 /// -
 /// ame`: DeviceName
 /// - `dev_id`: privatefiniteData
 /// - `flags`: InterruptFlag
 /// # return
 /// Successreturntrue,Failurereturnfalse
 pub fn request_irq(
 &mut self,
 irq: IrqNumber,
 handler: IrqHandler,
 name: &[u8],
 dev_id: *mut core::ffi::c_void,
 flags: IrqFlags,
 ) -> bool {
 if irq as usize >= 256 {
 log_warn!("request_irq: invalid irq number {}", irq);
 return false;
 }
 
 log_info!("request_irq: irq={}, name={:?}", irq, name);
 
 // Create new Action
 let mut action = IrqAction {
 handler,
 name: [0; 32],
 dev_id,
 next: core::ptr::null_mut(),
 flags,
 };
 
 let len = name.len().min(31);
 action.name[..len].copy_from_slice(&name[..len]);
 
 // TODO: addPlustoInterruptDescriptor HandleFunctionlinkform
 // ifisSharedInterrupt,addPlustolinkformfinalTail
 // whetherprincipledirectacceptSet
 
 // inInterruptControldeviceinfixmakecanInterrupt
 if !flags.contains(IrqFlags::NOAUTOEN) {
 crate::kernel::arch::current_arch().irq_controller().enable_irq(irq);
 }
 
 true
 }
 
 /// UnregisterInterruptHandleFunction
 pub fn free_irq(&mut self, irq: IrqNumber, dev_id: *mut core::ffi::c_void) {
 if irq as usize >= 256 {
 return;
 }
 
 log_info!("free_irq: irq={}", irq);
 
 // TODO: secondaryHandleFunctionlinkforminfixDivide
 // iflinkformasempty,DisableInterrupt
 
 crate::kernel::arch::current_arch().irq_controller().disable_irq(irq);
 }
 
 /// makecanInterrupt
 pub fn enable_irq(&self, irq: IrqNumber) {
 crate::kernel::arch::current_arch().irq_controller().enable_irq(irq);
 }
 
 /// DisableInterrupt
 pub fn disable_irq(&self, irq: IrqNumber) {
 crate::kernel::arch::current_arch().irq_controller().disable_irq(irq);
 }
 
 /// InterruptHandleenterport
 /// # Parameter
 /// - `irq`: Interruptsignal
 pub fn handle_irq(&mut self, irq: IrqNumber) {
 // increasePlusNesteddeepDegree
 self.irq_depth.fetch_add(1, Ordering::AcqRel);
 
 // FindInterruptDescriptor
 if irq as usize >= 256 {
 log_warn!("handle_irq: invalid irq number {}", irq);
 self.irq_depth.fetch_sub(1, Ordering::AcqRel);
 return;
 }
 
 // tuneuseHandleFunctionlinkform
 // TODO: traverseHandleFunctionlinkform,tuneusePeritemHandleFunction
 
 // SendEOI
 crate::kernel::arch::current_arch().irq_controller().eoi(irq);
 
 // MinusfewNesteddeepDegree
 self.irq_depth.fetch_sub(1, Ordering::AcqRel);
 
 // ifExitmostoutsideSheafInterrupt,HandlesoftInterrupt
 if self.irq_depth.load(Ordering::Acquire) == 0 {
 self.do_softirq();
 }
 }
 
 /// RegisterExceptionHandleFunction
 pub fn register_exception_handler(&mut self, exc_type: ExceptionType, handler: ExceptionHandler) {
 let vector = self.exception_to_vector(exc_type);
 if vector < 32 {
 self.exc_handlers[vector] = Some(handler);
 }
 }
 
 /// ExceptionHandleenterport
 pub fn handle_exception(&self, ctx: &ExceptionContext) {
 let vector = self.exception_to_vector(ctx.exc_type);
 
 if vector < 32 {
 if let Some(handler) = self.exc_handlers[vector] {
 handler(ctx);
 return;
 }
 }
 
 // DefaultHandle
 self.default_exception_handler(ctx);
 }
 
 /// RegistersoftInterruptHandleFunction
 pub fn register_softirq(&mut self, softirq_type: SoftIrqType, handler: SoftIrqHandler) {
 let index = softirq_type as usize;
 if index < 8 {
 self.softirq_handlers[index] = Some(handler);
 }
 }
 
 /// TriggersoftInterrupt
 pub fn raise_softirq(&self, softirq_type: SoftIrqType) {
 let index = softirq_type as usize;
 if index < 8 {
 self.softirq_pending.fetch_or(1 << index, Ordering::AcqRel);
 }
 }
 
 /// HandlesoftInterrupt
 pub fn do_softirq(&mut self) {
 let pending = self.softirq_pending.swap(0, Ordering::AcqRel);
 
 if pending == 0 {
 return;
 }
 
 // byPriorityHandlesoftInterrupt
 for i in 0..8 {
 if pending & (1 << i) != 0 {
 if let Some(handler) = self.softirq_handlers[i] {
 handler();
 }
 }
 }
 }
 
 /// RegisterDefaultExceptionHandleFunction
 fn register_default_exception_handlers(&mut self) {
 // pageErrorHandle
 self.register_exception_handler(ExceptionType::PageFault, Self::handle_page_fault);
 
 // ageneralprotectedErrorHandle
 self.register_exception_handler(ExceptionType::GeneralProtection, Self::handle_general_protection);
 
 // BreakpointHandle
 self.register_exception_handler(ExceptionType::Breakpoint, Self::handle_breakpoint);
 }
 
 /// DefaultExceptionHandleFunction
 fn default_exception_handler(&self, ctx: &ExceptionContext) {
 log_error!("Unhandled exception: {:?}", ctx.exc_type);
 log_error!(" Error code: {:#x}", ctx.error_code);
 log_error!(" Fault address: {:#x}", ctx.fault_addr);
 log_error!(" PC: {:#x}, SP: {:#x}", ctx.pc, ctx.sp);
 
 // TODO: TerminateCurrent process
 // process::current().kill();
 }
 
 /// pageErrorHandle
 fn handle_page_fault(ctx: &ExceptionContext) {
 log_debug!("Page fault at {:#x}, error={:#x}", ctx.fault_addr, ctx.error_code);
 
 // tuneuseMemoryManager defectpageHandle
 if !crate::kernel::mm::handle_page_fault(
 ctx.fault_addr,
 ctx.error_code as u32,
 ) == 0 {
 log_error!("Page fault handling failed at {:#x}", ctx.fault_addr);
 // TODO: SendSIGSEGVgiveCurrent process
 }
 }
 
 /// ageneralprotectedErrorHandle
 fn handle_general_protection(ctx: &ExceptionContext) {
 log_error!("General protection fault at {:#x}", ctx.pc);
 log_error!(" Error code: {:#x}", ctx.error_code);
 
 // TODO: SendSIGSEGVgiveCurrent process
 }
 
 /// BreakpointHandle
 fn handle_breakpoint(ctx: &ExceptionContext) {
 log_debug!("Breakpoint at {:#x}", ctx.pc);
 
 // TODO: NotificationDebuggingdevice
 }
 
 /// ExceptionTypebranchVectorsignal
 fn exception_to_vector(&self, exc_type: ExceptionType) -> usize {
 match exc_type {
 ExceptionType::DivideError => 0,
 ExceptionType::Debug => 1,
 ExceptionType::Breakpoint => 3,
 ExceptionType::Overflow => 4,
 ExceptionType::InvalidOpcode => 6,
 ExceptionType::DoubleFault => 8,
 ExceptionType::GeneralProtection => 13,
 ExceptionType::PageFault => 14,
 ExceptionType::FpuError => 16,
 ExceptionType::AlignmentCheck => 17,
 ExceptionType::MachineCheck => 18,
 ExceptionType::SimdException => 19,
 ExceptionType::SystemCall => 0x80,
 ExceptionType::Unknown => 31,
 }
 }
}

/// GlobalInterruptManager
static INTERRUPT_MANAGER: core::sync::OnceLock<InterruptManager> = core::sync::OnceLock::new();

/// GetInterruptManager
pub fn interrupt_manager() -> &'static InterruptManager {
    INTERRUPT_MANAGER.get_or_init(InterruptManager::new)
}

pub fn init_interrupt_manager() -> &'static InterruptManager {
    INTERRUPT_MANAGER.get_or_init(InterruptManager::new)
}

/// InitializeInterruptmanagementadministration
pub fn init_interrupt() {
 let manager = interrupt_manager();
 manager.init();
}

/// Function: RegisterInterrupt
pub fn request_irq(irq: IrqNumber, handler: IrqHandler, name: &[u8], dev_id: *mut core::ffi::c_void, flags: IrqFlags) -> bool {
 interrupt_manager().request_irq(irq, handler, name, dev_id, flags)
}

/// Function: UnregisterInterrupt
pub fn free_irq(irq: IrqNumber, dev_id: *mut core::ffi::c_void) {
 interrupt_manager().free_irq(irq, dev_id);
}

/// Function: makecanInterrupt
pub fn enable_irq(irq: IrqNumber) {
 interrupt_manager().enable_irq(irq);
}

/// Function: DisableInterrupt
pub fn disable_irq(irq: IrqNumber) {
 interrupt_manager().disable_irq(irq);
}

/// InterruptSaveguard
pub struct IrqSave {
 was_enabled: bool,
}

impl IrqSave {
 /// SaveparallelDisableInterrupt
 pub fn save_disable() -> Self {
 let was_enabled = crate::kernel::arch::current_arch().irq_controller().get_irq_count(0) != 0;
 crate::kernel::arch::current_arch().disable_irq();
 IrqSave { was_enabled }
 }
}

impl Drop for IrqSave {
 fn drop(&mut self) {
 if self.was_enabled {
 crate::kernel::arch::current_arch().enable_irq();
 }
 }
}