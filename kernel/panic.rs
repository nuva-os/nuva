/*
 * Nuva OS - Kernel - Panic.Rs
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


use core::panic::PanicInfo;
use core::sync::atomic::{AtomicBool, Ordering};

/// Panic Flag
static PANIC_OCCURRED: AtomicBool = AtomicBool::new(false);

/// Panic Context
pub struct PanicContext {
 pub message: &'static str,
 pub file: &'static str,
 pub line: u32,
 pub column: u32,
}

/// Current Panic Context
static mut PANIC_CONTEXT: Option<PanicContext> = None;

/// Panic HandleFunction
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
 // Checkifalready panic (avoidRecursion panic)
 if PANIC_OCCURRED.load(Ordering::Relaxed) {
 loop {
 // SAFETY: inline assembly required for hardware instruction
 unsafe { asm!("wfi"); }
 }
 }
 
 PANIC_OCCURRED.store(true, Ordering::Relaxed);
 
 // Save panic Context
 let context = PanicContext {
 message: info.message().as_str().unwrap_or("unknown"),
 file: info.location().map(|l| l.file()).unwrap_or("unknown"),
 line: info.location().map(|l| l.line()).unwrap_or(0),
 column: info.location().map(|l| l.column()).unwrap_or(0),
 };
 
 // SAFETY: unsafe block required for low-level memory or hardware access
 unsafe {
 PANIC_CONTEXT = Some(context);
 }
 
 // Output panic Info
 print_panic_info(info);
 
 // StopSystem
 halt()
}

/// printstamp panic Info
fn print_panic_info(info: &PanicInfo) {
 use crate::kernel::debug::printk::*;
 
 log_emerg!("========================================");
 log_emerg!("KERNEL PANIC");
 log_emerg!("========================================");
 
 // Output panic Message
 if let Some(message) = info.message().as_str() {
 log_emerg!("Message: {}", message);
 }
 
 // OutputPositionInfo
 if let Some(location) = info.location() {
 log_emerg!("Location: {}:{}:{}",
 location.file(),
 location.line(),
 location.column()
 );
 }
 
 log_emerg!("========================================");
 
 // OutputRegisterInfo
 log_emerg!("Register State:");
 #[cfg(target_arch = "aarch64")]
 crate::kernel::arch::arm64::boot::early_console::print_registers();
 
 // OutputtuneuseStack
 log_emerg!("Call Stack:");
 print_stack_trace();
 
 log_emerg!("========================================");
 log_emerg!("System halted.");
}

/// printstamptuneuseStack
fn print_stack_trace() {
 use crate::kernel::debug::printk::*;
 
 // SAFETY: unsafe block required for low-level memory or hardware access
 unsafe {
 let mut fp: u64;
 asm!("mov {}, x29", out(reg) fp);
 
 log_emerg!("Stack trace:");
 
 for i in 0..16 {
 if fp == 0 {
 break;
 }
 
 let lr = *(fp as *const u64).offset(1);
 log_emerg!(" #{}: {:#018x}", i, lr);
 
 fp = *(fp as *const u64);
 }
 }
}

/// StopSystem
fn halt() -> ! {
 // DisableInterrupt
 // SAFETY: inline assembly required for hardware instruction
 unsafe {
 asm!("msr daifset, #0xF");
 }
 
 // infinitelimitRing
 loop {
 // SAFETY: unsafe block required for low-level memory or hardware access
 unsafe {
 // WaitInterrupt (lowWorkconsume)
 asm!("wfi");
 }
 }
}

/// Trigger panic
pub fn panic_manual(message: &'static str) -> ! {
 panic!("{}", message);
}

/// Checkifalready panic
pub fn is_panicking() -> bool {
 PANIC_OCCURRED.load(Ordering::Relaxed)
}

/// Get panic Context
pub fn get_panic_context() -> Option<&'static PanicContext> {
 // SAFETY: unsafe block required for low-level memory or hardware access
 unsafe { PANIC_CONTEXT.as_ref() }
}

/// breaklanguageMacro
#[macro_export]
macro_rules! assert {
 ($cond:expr) => {
 if !$cond {
 panic!("assertion failed: {}", stringify!($cond));
 }
 };
 ($cond:expr, $($arg:tt)*) => {
 if !$cond {
 panic!("assertion failed: {}", format_args!($($arg)*));
 }
 };
}

/// encodingtranslatetimebreaklanguage
#[macro_export]
macro_rules! static_assert {
 ($cond:expr) => {
 const _: () = assert!($cond);
 };
}

/// Trigger a kernel bug check. Renamed from BUG! (Linux kernel style) to nuva_bug! (Nuva OS style).
#[macro_export]
macro_rules! nuva_bug {
 () => {
 panic!("BUG at {}:{}", file!(), line!());
 };
 ($msg:expr) => {
 panic!("BUG: {} at {}:{}", $msg, file!(), line!());
 };
}

/// Emit a kernel warning conditionally. Renamed from WARN! (Linux kernel style) to nuva_warn! (Nuva OS style).
#[macro_export]
macro_rules! nuva_warn {
 ($cond:expr) => {
 if $cond {
 $crate::kernel::debug::printk::log_warn!("WARNING at {}:{}", file!(), line!());
 }
 };
 ($cond:expr, $($arg:tt)*) => {
 if $cond {
 $crate::kernel::debug::printk::log_warn!("WARNING: {} at {}:{}",
 format_args!($($arg)*), file!(), line!());
 }
 };
}

/// Deprecated alias for nuva_bug!. Use nuva_bug! instead.
#[deprecated(since = "0.2.0", note = "Use nuva_bug! instead")]
#[macro_export]
macro_rules! BUG {
 () => {
 $crate::nuva_bug!();
 };
 ($msg:expr) => {
 $crate::nuva_bug!($msg);
 };
}

/// Deprecated alias for nuva_warn!. Use nuva_warn! instead.
#[deprecated(since = "0.2.0", note = "Use nuva_warn! instead")]
#[macro_export]
macro_rules! WARN {
 ($cond:expr) => {
 $crate::nuva_warn!($cond);
 };
 ($cond:expr, $($arg:tt)*) => {
 $crate::nuva_warn!($cond, $($arg)*);
 };
}

/// ImplementationMacro
#[macro_export]
macro_rules! unimplemented {
 () => {
 panic!("unimplemented at {}:{}", file!(), line!());
 };
 ($msg:expr) => {
 panic!("unimplemented: {} at {}:{}", $msg, file!(), line!());
 };
}

/// notcanreachMacro
#[macro_export]
macro_rules! unreachable {
 () => {
 panic!("unreachable at {}:{}", file!(), line!());
 };
 ($msg:expr) => {
 panic!("unreachable: {} at {}:{}", $msg, file!(), line!());
 };
}

/// Memory Barrierbreaklanguage
#[macro_export]
macro_rules! assert_barrier {
 () => {
 $crate::static_assert!(core::sync::atomic::Ordering::SeqCst as usize >= 
 core::sync::atomic::Ordering::AcqRel as usize);
 };
}

#[cfg(test)]
mod tests {
 use super::*;
use core::arch::asm;
 
 #[test]
 #[should_panic]
 fn test_panic() {
 panic!("test panic");
 }
 
 #[test]
 fn test_assert() {
 assert!(true);
 assert!(1 + 1 == 2, "math is broken");
 }
}