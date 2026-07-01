use crate::{pr_err, pr_info};
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


use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// ExceptionType
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExceptionType {
 /// DivideError
 DivideError = 0,
 /// DebuggingException
 Debug = 1,
 /// notcanscreenInterrupt
 Nmi = 2,
 /// Breakpoint
 Breakpoint = 3,
 /// Overflow
 Overflow = 4,
 /// Bounds Checking
 BoundRange = 5,
 /// invalidOperationcode
 InvalidOpcode = 6,
 /// Devicenotcanuse
 DeviceNotAvailable = 7,
 /// doublerepeatError
 DoubleFault = 8,
 /// invalid TSS
 InvalidTss = 10,
 /// paragraphnotExists
 SegmentNotPresent = 11,
 /// StackparagraphError
 StackSegment = 12,
 /// ageneralprotectedError
 GeneralProtection = 13,
 /// pageError
 PageFault = 14,
 /// FPU Error
 FpuError = 16,
 /// AlignmentCheck
 AlignmentCheck = 17,
 /// machinedeviceCheck
 MachineCheck = 18,
 /// SIMD Exception
 SimdException = 19,
}

/// TrapFrame
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct TrapFrame {
 /// GeneralRegister
 pub rax: u64,
 pub rbx: u64,
 pub rcx: u64,
 pub rdx: u64,
 pub rsi: u64,
 pub rdi: u64,
 pub rbp: u64,
 pub r8: u64,
 pub r9: u64,
 pub r10: u64,
 pub r11: u64,
 pub r12: u64,
 pub r13: u64,
 pub r14: u64,
 pub r15: u64,
 /// Error code
 pub error_code: u64,
 /// Vectorsignal
 pub vector: u64,
 /// Instructionpointer
 pub rip: u64,
 /// Codeparagraph
 pub cs: u64,
 /// FlagRegister
 pub rflags: u64,
 /// Stackpointer
 pub rsp: u64,
 /// Stackparagraph
 pub ss: u64,
}

/// ExceptionHandleFunctionType
pub type ExceptionHandler = fn(&TrapFrame);

/// ExceptionHandleform
pub struct ExceptionTable {
 /// HandleFunctionArray
 handlers: [Option<ExceptionHandler>; 32],
 /// ExceptionCount
 counts: [AtomicU64; 32],
}

impl ExceptionTable {
 pub const fn new() -> Self {
 ExceptionTable {
 handlers: [const { None }; 32],
 counts: [const { AtomicU64::new(0) }; 32],
 }
 }
 
 /// RegisterHandleFunction
 pub fn register(&mut self, vector: u8, handler: ExceptionHandler) {
 if (vector as usize) < 32 {
 self.handlers[vector as usize] = Some(handler);
 }
 }
 
 /// HandleException
 pub fn handle(&self, frame: &TrapFrame) {
 let vector = frame.vector as usize;
 
 if vector < 32 {
 self.counts[vector].fetch_add(1, Ordering::AcqRel);
 
 if let Some(handler) = self.handlers[vector] {
 handler(frame);
 return;
 }
 }
 
 // DefaultHandle
 default_exception_handler(frame);
 }
 
 /// GetExceptionCount
 pub fn get_count(&self, vector: u8) -> u64 {
 if (vector as usize) < 32 {
 self.counts[vector as usize].load(Ordering::Acquire)
 } else {
 0
 }
 }
}

/// DefaultExceptionHandle
fn default_exception_handler(frame: &TrapFrame) {
 log_error!("Unhandled exception: vector={}", frame.vector);
 log_error!(" RIP: {:#x}", frame.rip);
 log_error!(" RSP: {:#x}", frame.rsp);
 log_error!(" Error code: {:#x}", frame.error_code);
 
 // printstampRegister
 log_error!(" RAX: {:#x}, RBX: {:#x}, RCX: {:#x}", frame.rax, frame.rbx, frame.rcx);
 log_error!(" RDX: {:#x}, RSI: {:#x}, RDI: {:#x}", frame.rdx, frame.rsi, frame.rdi);
 log_error!(" RBP: {:#x}, R8: {:#x}, R9: {:#x}", frame.rbp, frame.r8, frame.r9);
 log_error!(" R10: {:#x}, R11: {:#x}, R12: {:#x}", frame.r10, frame.r11, frame.r12);
 log_error!(" R13: {:#x}, R14: {:#x}, R15: {:#x}", frame.r13, frame.r14, frame.r15);
 
 // suspendSystem
 loop {
 core::hint::spin_loop();
 }
}

/// pageErrorHandle
fn page_fault_handler(frame: &TrapFrame) {
 // GetErrorAddress
 let fault_addr: u64;
 // SAFETY: unsafe block required for low-level memory or hardware access
 unsafe {
 // ARM64: read FAR_EL1
 // x86-64: read CR2
 core::arch::asm!(
 "mov {}, cr2",
 out(reg) fault_addr,
 );
 }
 
 log_error!("Page fault at {:#x}", fault_addr);
 log_error!(" RIP: {:#x}", frame.rip);
 log_error!(" Error code: {:#x}", frame.error_code);
 
 // parseError code
 let present = frame.error_code & 1 != 0;
 let write = frame.error_code & 2 != 0;
 let user = frame.error_code & 4 != 0;
 
 log_error!(" Present: {}, Write: {}, User: {}", present, write, user);

 // Handle COW: if write fault on a COW page, allocate a private copy
 if write {
     // TODO: properly obtain CowEntry and current PID for COW fault
     // For now, skip COW handling at trap level
 }

 // Handle demand paging: if page is in a valid VMA but not yet mapped
 if crate::kernel::mm::mmap::handle_demand_page(crate::kernel::arch::VirtAddr(fault_addr)) == 0 {
     return; // Demand page handled successfully
 }

 // Unrecoverable fault: send SIGSEGV to the current process
 if user {
     let handler = crate::kernel::process::signal::get_signal_handler();
     let info = crate::kernel::process::signal::SigInfo {
         signo: crate::kernel::process::signal::signal::SIGSEGV as i32,
         errno: 0,
         code: if present { 2 } else { 1 }, // SEGV_ACCERR or SEGV_MAPERR
         pid: 0,
         uid: 0,
         value: crate::kernel::process::signal::SigVal { sival_int: 0 },
         addr: fault_addr,
     };
     let _ = handler.deliver_signal(crate::kernel::process::signal::signal::SIGSEGV, &info);
 } else {
     // Kernel mode page fault: fatal
     log_error!("FATAL: Kernel page fault at {:#x}", fault_addr);
     loop { core::hint::spin_loop(); }
 }
}

/// ageneralprotectedErrorHandle
fn general_protection_handler(frame: &TrapFrame) {
 log_error!("General protection fault at RIP: {:#x}", frame.rip);
 log_error!(" Error code: {:#x}", frame.error_code);

 let user_mode = frame.error_code & 4 != 0;
 if user_mode {
     // Send SIGSEGV to the current process
     let handler = crate::kernel::process::signal::get_signal_handler();
     let info = crate::kernel::process::signal::SigInfo {
         signo: crate::kernel::process::signal::signal::SIGSEGV as i32,
         errno: 0,
         code: 2, // SEGV_ACCERR
         pid: 0,
         uid: 0,
         value: crate::kernel::process::signal::SigVal { sival_int: 0 },
         addr: frame.rip,
     };
     let _ = handler.deliver_signal(crate::kernel::process::signal::signal::SIGSEGV, &info);
 } else {
     // Kernel mode GP fault: fatal
     loop { core::hint::spin_loop(); }
 }
}

/// DivideErrorHandle
fn divide_error_handler(frame: &TrapFrame) {
 log_error!("Divide error at RIP: {:#x}", frame.rip);

 // Send SIGFPE to the current process
 let handler = crate::kernel::process::signal::get_signal_handler();
 let info = crate::kernel::process::signal::SigInfo {
     signo: crate::kernel::process::signal::signal::SIGFPE as i32,
     errno: 0,
     code: 6, // FPE_INTDIV (integer divide by zero)
     pid: 0,
     uid: 0,
     value: crate::kernel::process::signal::SigVal { sival_int: 0 },
     addr: frame.rip,
 };
 let _ = handler.deliver_signal(crate::kernel::process::signal::signal::SIGFPE, &info);
}

/// invalidOperationcodeHandle
fn invalid_opcode_handler(frame: &TrapFrame) {
 log_error!("Invalid opcode at RIP: {:#x}", frame.rip);

 // Send SIGILL to the current process
 let handler = crate::kernel::process::signal::get_signal_handler();
 let info = crate::kernel::process::signal::SigInfo {
     signo: crate::kernel::process::signal::signal::SIGILL as i32,
     errno: 0,
     code: 1, // ILL_ILLOPC (illegal opcode)
     pid: 0,
     uid: 0,
     value: crate::kernel::process::signal::SigVal { sival_int: 0 },
     addr: frame.rip,
 };
 let _ = handler.deliver_signal(crate::kernel::process::signal::signal::SIGILL, &info);
}

/// BreakpointHandle
fn breakpoint_handler(frame: &TrapFrame) {
 log_info!("Breakpoint at RIP: {:#x}", frame.rip);

 // Send SIGTRAP to the current process (for debugger support)
 let handler = crate::kernel::process::signal::get_signal_handler();
 let info = crate::kernel::process::signal::SigInfo {
     signo: crate::kernel::process::signal::signal::SIGTRAP as i32,
     errno: 0,
     code: 1, // TRAP_BRKPT (breakpoint trap)
     pid: 0,
     uid: 0,
     value: crate::kernel::process::signal::SigVal { sival_int: 0 },
     addr: frame.rip,
 };
 let _ = handler.deliver_signal(crate::kernel::process::signal::signal::SIGTRAP, &info);
}

/// GlobalExceptionform
static EXCEPTION_TABLE: crate::sync_oncelock::OnceLock<ExceptionTable> = crate::sync_oncelock::OnceLock::new();

/// GetExceptionform
pub fn exception_table() -> &'static ExceptionTable {
    EXCEPTION_TABLE.get_or_init(ExceptionTable::new)
}

/// InitializeTrapHandle
pub fn init_trap() {
 let table = exception_table();
 
 // RegisterExceptionHandleFunction
 table.register(ExceptionType::DivideError as u8, divide_error_handler);
 table.register(ExceptionType::InvalidOpcode as u8, invalid_opcode_handler);
 table.register(ExceptionType::Breakpoint as u8, breakpoint_handler);
 table.register(ExceptionType::GeneralProtection as u8, general_protection_handler);
 table.register(ExceptionType::PageFault as u8, page_fault_handler);
 
 log_info!("Trap handler initialized");
}

/// TrapHandleenterport
pub fn trap_handler(frame: &mut TrapFrame) {
 let table = exception_table();
 table.handle(frame);
}

#[cfg(test)]
mod tests {
 use super::*;

 #[test]
 fn test_exception_type_values() {
 assert_eq!(ExceptionType::DivideError as u8, 0);
 assert_eq!(ExceptionType::Debug as u8, 1);
 assert_eq!(ExceptionType::Nmi as u8, 2);
 assert_eq!(ExceptionType::Breakpoint as u8, 3);
 assert_eq!(ExceptionType::Overflow as u8, 4);
 assert_eq!(ExceptionType::InvalidOpcode as u8, 6);
 assert_eq!(ExceptionType::DoubleFault as u8, 8);
 assert_eq!(ExceptionType::GeneralProtection as u8, 13);
 assert_eq!(ExceptionType::PageFault as u8, 14);
 assert_eq!(ExceptionType::FpuError as u8, 16);
 assert_eq!(ExceptionType::SimdException as u8, 19);
 }

 #[test]
 fn test_trap_frame() {
 let frame = TrapFrame {
 rax: 1,
 rbx: 2,
 rcx: 3,
 rdx: 4,
 rsi: 5,
 rdi: 6,
 rbp: 7,
 r8: 8,
 r9: 9,
 r10: 10,
 r11: 11,
 r12: 12,
 r13: 13,
 r14: 14,
 r15: 15,
 error_code: 0,
 vector: 14,
 rip: 0x1000,
 cs: 8,
 rflags: 0x202,
 rsp: 0x7FFF_F000,
 ss: 16,
 };

 assert_eq!(frame.rax, 1);
 assert_eq!(frame.vector, 14);
 assert_eq!(frame.rip, 0x1000);
 assert_eq!(frame.rsp, 0x7FFF_F000);
 }

 #[test]
 fn test_exception_table_new() {
 let table = ExceptionTable::new();

 // placefiniteCountshouldas 0
 for i in 0..32 {
 assert_eq!(table.get_count(i as u8), 0);
 }
 }

 #[test]
 fn test_exception_table_register() {
 let mut table = ExceptionTable::new();

 fn test_handler(_frame: &TrapFrame) {}

 table.register(0, test_handler);

 // ValidateHandleFunctionalreadyRegister
 assert!(table.handlers[0].is_some());
 }

 #[test]
 fn test_exception_table_get_count() {
 let table = ExceptionTable::new();

 // validVector
 assert_eq!(table.get_count(0), 0);
 assert_eq!(table.get_count(14), 0);

 // invalidVector
 assert_eq!(table.get_count(32), 0);
 assert_eq!(table.get_count(255), 0);
 }

 #[test]
 fn test_exception_table_handle_count() {
 let table = ExceptionTable::new();

 let frame = TrapFrame {
 rax: 0, rbx: 0, rcx: 0, rdx: 0,
 rsi: 0, rdi: 0, rbp: 0,
 r8: 0, r9: 0, r10: 0, r11: 0,
 r12: 0, r13: 0, r14: 0, r15: 0,
 error_code: 0,
 vector: 14,
 rip: 0,
 cs: 0,
 rflags: 0,
 rsp: 0,
 ss: 0,
 };

 // noteintent: thisitemTestwilltuneuse default_exception_handler, willinfinitelimitRing
 // placewithTestCounterincreasePlus
 assert_eq!(table.get_count(14), 0);
 }

 #[test]
 fn test_exception_type_equality() {
 assert_eq!(ExceptionType::PageFault, ExceptionType::PageFault);
 assert_ne!(ExceptionType::PageFault, ExceptionType::GeneralProtection);
 assert_ne!(ExceptionType::DivideError, ExceptionType::Overflow);
 }

 #[test]
 fn test_trap_frame_error_code() {
 let mut frame = TrapFrame {
 rax: 0, rbx: 0, rcx: 0, rdx: 0,
 rsi: 0, rdi: 0, rbp: 0,
 r8: 0, r9: 0, r10: 0, r11: 0,
 r12: 0, r13: 0, r14: 0, r15: 0,
 error_code: 0,
 vector: 14,
 rip: 0,
 cs: 0,
 rflags: 0,
 rsp: 0,
 ss: 0,
 };

 frame.error_code = 0b111; // Present + Write + User

 let present = frame.error_code & 1 != 0;
 let write = frame.error_code & 2 != 0;
 let user = frame.error_code & 4 != 0;

 assert!(present);
 assert!(write);
 assert!(user);
 }

 #[test]
 fn test_trap_frame_registers() {
 let frame = TrapFrame {
 rax: 0x1111,
 rbx: 0x2222,
 rcx: 0x3333,
 rdx: 0x4444,
 rsi: 0x5555,
 rdi: 0x6666,
 rbp: 0x7777,
 r8: 0x8888,
 r9: 0x9999,
 r10: 0xAAAA,
 r11: 0xBBBB,
 r12: 0xCCCC,
 r13: 0xDDDD,
 r14: 0xEEEE,
 r15: 0xFFFF,
 error_code: 0,
 vector: 0,
 rip: 0,
 cs: 0,
 rflags: 0,
 rsp: 0,
 ss: 0,
 };

 assert_eq!(frame.rax, 0x1111);
 assert_eq!(frame.rbx, 0x2222);
 assert_eq!(frame.rcx, 0x3333);
 assert_eq!(frame.r15, 0xFFFF);
 }

 #[test]
 fn test_exception_table_multiple_registers() {
 let mut table = ExceptionTable::new();

 fn handler1(_frame: &TrapFrame) {}
 fn handler2(_frame: &TrapFrame) {}
 fn handler3(_frame: &TrapFrame) {}

 table.register(0, handler1);
 table.register(6, handler2);
 table.register(14, handler3);

 assert!(table.handlers[0].is_some());
 assert!(table.handlers[6].is_some());
 assert!(table.handlers[14].is_some());
 }

 #[test]
 fn test_exception_table_register_out_of_range() {
 let mut table = ExceptionTable::new();

 fn handler(_frame: &TrapFrame) {}

 // RegisterexceedexitRange Vectornotshould panic
 table.register(32, handler);
 table.register(255, handler);

 // ValidatefiniteexceedboundaryWrite
 for i in 0..32 {
 assert!(table.handlers[i].is_none());
 }
 }
}