/*
 * Nuva OS - Kernel - Debug
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


use core::fmt;
use core::fmt::Write;
use core::sync::atomic::{AtomicU32, Ordering};

/// LogLevel
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
 Emergency = 0, // System is unusable
 Alert = 1, // Must be handled immediately
 Critical = 2, // Critical error
 Error = 3, // Error
 Warning = 4, // Warning
 Notice = 5, // Normal but significant
 Info = 6, // Informational
 Debug = 7, // Debug-level messages
}

impl LogLevel {
 /// Get log level prefix
 pub fn prefix(&self) -> &'static str {
 match self {
 LogLevel::Emergency => "EMERG",
 LogLevel::Alert => "ALERT",
 LogLevel::Critical => "CRIT",
 LogLevel::Error => "ERROR",
 LogLevel::Warning => "WARN",
 LogLevel::Notice => "NOTICE",
 LogLevel::Info => "INFO",
 LogLevel::Debug => "DEBUG",
 }
 }
 
 /// GetLogLevelColor (ANSI)
 pub fn color(&self) -> &'static str {
 match self {
 LogLevel::Emergency => "\x1b[31m", // ㈣
 LogLevel::Alert => "\x1b[31m", // Red
 LogLevel::Critical => "\x1b[31m", // Red
 LogLevel::Error => "\x1b[31m", // Red
 LogLevel::Warning => "\x1b[33m", // Yellow
 LogLevel::Notice => "\x1b[36m", // Cyan
 LogLevel::Info => "\x1b[32m", // Green
 LogLevel::Debug => "\x1b[34m", // Blue
 }
 }
}

/// CurrentLogLevel
static CURRENT_LOG_LEVEL: AtomicU32 = AtomicU32::new(LogLevel::Info as u32);

/// GetCurrentLogLevel
pub fn get_log_level() -> LogLevel {
 match CURRENT_LOG_LEVEL.load(Ordering::Relaxed) {
 0 => LogLevel::Emergency,
 1 => LogLevel::Alert,
 2 => LogLevel::Critical,
 3 => LogLevel::Error,
 4 => LogLevel::Warning,
 5 => LogLevel::Notice,
 6 => LogLevel::Info,
 7 => LogLevel::Debug,
 _ => LogLevel::Info,
 }
}

/// SetLogLevel
pub fn set_log_level(level: LogLevel) {
 CURRENT_LOG_LEVEL.store(level as u32, Ordering::Relaxed);
}

/// printk writer
struct PrintkWriter;

impl fmt::Write for PrintkWriter {
 fn write_str(&mut self, s: &str) -> fmt::Result {
 // Use early console output
 #[cfg(target_arch = "aarch64")]
 crate::kernel::arch::arm64::boot::early_console::get_early_console().puts(s);
 #[cfg(not(target_arch = "aarch64"))]
 {
     // For non-aarch64 targets, output is platform-specific
     let _ = s;
 }
 Ok(())
 }
}

/// Initialize printk subsystem
pub fn init_printk() {
    // Set default log level
    set_log_level(LogLevel::Info);
}

/// Kernel print function
pub fn printk(level: LogLevel, args: fmt::Arguments) {
 // Check log level
 if level > get_log_level() {
 return;
 }
 
 let mut writer = PrintkWriter;
 
 // Output timestamp
 // SAFETY: unsafe block required for low-level memory or hardware access
 let timestamp_ms = unsafe { crate::kernel::time::get_time_ms() };
 let secs = timestamp_ms / 1000;
 let msecs = timestamp_ms % 1000;
 let _ = fmt::write(
 &mut writer,
 format_args!("[{}.{:03}] ", secs, msecs),
 );
 
 // Output log level
 let _ = fmt::write(
 &mut writer,
 format_args!(
 "{}[{}]{} ",
 level.color(),
 level.prefix(),
 "\x1b[0m" // Reset color
 ),
 );
 
 // Output message
 let _ = fmt::write(&mut writer, args);
 
 // Output newline
 let _ = writer.write_str("
");
}

/// Emergency print (no condition output)
pub fn printk_emerg(args: fmt::Arguments) {
 printk(LogLevel::Emergency, args);
}

/// Alert print
pub fn printk_alert(args: fmt::Arguments) {
 printk(LogLevel::Alert, args);
}

/// Critical error print
pub fn printk_crit(args: fmt::Arguments) {
 printk(LogLevel::Critical, args);
}

/// Error print
pub fn printk_err(args: fmt::Arguments) {
 printk(LogLevel::Error, args);
}

/// Warning print
pub fn printk_warn(args: fmt::Arguments) {
 printk(LogLevel::Warning, args);
}

/// Notice print
pub fn printk_notice(args: fmt::Arguments) {
 printk(LogLevel::Notice, args);
}

/// Info print
pub fn printk_info(args: fmt::Arguments) {
 printk(LogLevel::Info, args);
}

/// Debug print
pub fn printk_debug(args: fmt::Arguments) {
 printk(LogLevel::Debug, args);
}

/// printk macro
#[macro_export]
macro_rules! printk {
 ($level:expr, $($arg:tt)*) => {
 $crate::kernel::debug::printk::printk($level, format_args!($($arg)*))
 };
}


/// Hex dump
pub fn hex_dump(prefix: &str, data: &[u8]) {
 for (i, chunk) in data.chunks(16).enumerate() {
 log_info!("{}{:08x}: ", prefix, i * 16);
 
 // Output hexadecimal
 for (j, byte) in chunk.iter().enumerate() {
 if j == 8 {
 log_info!(" ");
 }
 log_info!("{:02x} ", byte);
 }
 
 // Padding for alignment
 for j in chunk.len()..16 {
 if j == 8 {
 log_info!(" ");
 }
 log_info!(" ");
 }
 
 log_info!(" |");
 
 // Output ASCII
 for byte in chunk {
 if byte.is_ascii_graphic() || *byte == b' ' {
 log_info!("{}", *byte as char);
 } else {
 log_info!(".");
 }
 }
 
 log_info!("|");
 }
}

/// Print stack trace
pub fn print_stack_trace() {
 log_info!("=== Stack Trace ===");
 
 // SAFETY: unsafe block required for low-level memory or hardware access
 #[cfg(target_arch = "aarch64")]
 unsafe {
 let mut fp: u64;
 core::arch::asm!("mov {}, x29", out(reg) fp);
 
 for i in 0..16 {
 if fp == 0 {
 break;
 }
 
 let lr = *(fp as *const u64).offset(1);
 log_info!("#{}: {:016x}", i, lr);
 
 fp = *(fp as *const u64);
 }
 }
 #[cfg(not(target_arch = "aarch64"))]
 {
     // Stack trace not yet implemented for this architecture
 }
}

#[cfg(test)]
mod tests {
 use super::*;
 
 #[test]
 fn test_printk() {
 log_info!("Hello, Nuva OS!");
 log_warn!("This is a warning");
 log_error!("This is an error");
 }
}