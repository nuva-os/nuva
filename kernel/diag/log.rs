/*
 * Nuva OS - Kernel - Kernel Log
 * 
 * Copyright (C) 2026 Nuva OS Team
 * 
 * Kernel logging and printk implementation.
 */

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

use crate::posix::errno::Errno;
/// Log Level
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    /// Emergency - system is unusable
    Emerg = 0,
    /// Alert - action must be taken immediately
    Alert = 1,
    /// Critical - critical conditions
    Crit = 2,
    /// Error - error conditions
    Err = 3,
    /// Warning - warning conditions
    Warning = 4,
    /// Notice - normal but significant condition
    Notice = 5,
    /// Info - informational
    Info = 6,
    /// Debug - debug-level messages
    Debug = 7,
}

impl LogLevel {
    pub fn from_u32(level: u32) -> Self {
        match level {
            0 => LogLevel::Emerg,
            1 => LogLevel::Alert,
            2 => LogLevel::Crit,
            3 => LogLevel::Err,
            4 => LogLevel::Warning,
            5 => LogLevel::Notice,
            6 => LogLevel::Info,
            _ => LogLevel::Debug,
        }
    }
    
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Emerg => "EMERG",
            LogLevel::Alert => "ALERT",
            LogLevel::Crit => "CRIT",
            LogLevel::Err => "ERROR",
            LogLevel::Warning => "WARN",
            LogLevel::Notice => "NOTICE",
            LogLevel::Info => "INFO",
            LogLevel::Debug => "DEBUG",
        }
    }
}

/// Log Record
#[repr(C)]
#[derive(Clone, Copy)]
pub struct LogRecord {
    /// Log level
    pub level: LogLevel,
    /// Timestamp (nanoseconds)
    pub timestamp: u64,
    /// CPU ID
    pub cpu: u32,
    /// Process ID
    pub pid: u32,
    /// Thread ID
    pub tid: u32,
    /// Facility
    pub facility: u8,
    /// Flags
    pub flags: u8,
    /// Message length
    pub msg_len: u16,
    /// Message text
    pub msg: [u8; 256],
}

impl LogRecord {
    pub fn new(level: LogLevel, msg: &[u8]) -> Self {
        let mut msg_arr = [0u8; 256];
        let len = msg.len().min(255);
        msg_arr[..len].copy_from_slice(&msg[..len]);
        
        LogRecord {
            level,
            timestamp: 0, // TODO: Get current time
            cpu: 0,       // TODO: Get current CPU
            pid: 0,       // TODO: Get current PID
            tid: 0,       // TODO: Get current TID
            facility: 0,
            flags: 0,
            msg_len: len as u16,
            msg: msg_arr,
        }
    }
}

/// Log Buffer
pub struct LogBuffer {
    /// Buffer
    pub buffer: [LogRecord; 1024],
    /// Head index
    pub head: AtomicU32,
    /// Tail index
    pub tail: AtomicU32,
    /// Count
    pub count: AtomicU32,
    /// Total logged
    pub total: AtomicU64,
    /// Dropped
    pub dropped: AtomicU64,
}

impl LogBuffer {
    pub const fn new() -> Self {
        LogBuffer {
            buffer: [LogRecord {
                level: LogLevel::Info,
                timestamp: 0,
                cpu: 0,
                pid: 0,
                tid: 0,
                facility: 0,
                flags: 0,
                msg_len: 0,
                msg: [0; 256],
            }; 1024],
            head: AtomicU32::new(0),
            tail: AtomicU32::new(0),
            count: AtomicU32::new(0),
            total: AtomicU64::new(0),
            dropped: AtomicU64::new(0),
        }
    }
    
    /// Write log record
    pub fn write(&mut self, record: &LogRecord) {
        let head = self.head.load(Ordering::Acquire);
        let next = (head + 1) % 1024;
        
        if next == self.tail.load(Ordering::Acquire) {
            // Buffer full, drop oldest
            self.tail.store((self.tail.load(Ordering::Acquire) + 1) % 1024, Ordering::Release);
            self.dropped.fetch_add(1, Ordering::AcqRel);
        }
        
        self.buffer[head as usize] = *record;
        self.head.store(next, Ordering::Release);
        self.count.fetch_add(1, Ordering::AcqRel);
        self.total.fetch_add(1, Ordering::AcqRel);
    }
    
    /// Read log record
    pub fn read(&mut self) -> Option<&LogRecord> {
        let tail = self.tail.load(Ordering::Acquire);
        
        if tail == self.head.load(Ordering::Acquire) {
            return None;
        }
        
        let record = &self.buffer[tail as usize];
        self.tail.store((tail + 1) % 1024, Ordering::Release);
        self.count.fetch_sub(1, Ordering::AcqRel);
        
        Some(record)
    }
    
    /// Clear buffer
    pub fn clear(&mut self) {
        self.head.store(0, Ordering::Release);
        self.tail.store(0, Ordering::Release);
        self.count.store(0, Ordering::Release);
    }
}

/// Console Operations
pub struct ConsoleOps {
    /// Write function
    pub write: Option<unsafe extern "C" fn(*const u8, usize) -> usize>,
    /// Read function
    pub read: Option<unsafe extern "C" fn(*mut u8, usize) -> usize>,
    /// Flush function
    pub flush: Option<unsafe extern "C" fn()>,
}

/// Console
pub struct Console {
    /// Name
    pub name: [u8; 32],
    /// Operations
    pub ops: ConsoleOps,
    /// Index
    pub index: u32,
    /// Flags
    pub flags: AtomicU32,
    /// Next console
    pub next: *mut Console,
}

/// Log Manager
pub struct LogManager {
    /// Log buffer
    pub buffer: LogBuffer,
    /// Console list
    pub consoles: *mut Console,
    /// Console count
    pub console_count: AtomicU32,
    /// Current console
    pub current_console: *mut Console,
    /// Log level
    pub log_level: AtomicU32,
    /// Statistics
    pub stats: LogStats,
}

/// Log Statistics
pub struct LogStats {
    pub messages: AtomicU64,
    pub bytes: AtomicU64,
    pub errors: AtomicU64,
}

impl LogStats {
    pub const fn new() -> Self {
        LogStats {
            messages: AtomicU64::new(0),
            bytes: AtomicU64::new(0),
            errors: AtomicU64::new(0),
        }
    }
}

impl LogManager {
    pub const fn new() -> Self {
        LogManager {
            buffer: LogBuffer::new(),
            consoles: core::ptr::null_mut(),
            console_count: AtomicU32::new(0),
            current_console: core::ptr::null_mut(),
            log_level: AtomicU32::new(LogLevel::Info as u32),
            stats: LogStats::new(),
        }
    }
    
    /// Initialize
    pub fn init(&self) {
        // Register early console
        self.register_early_console();
    }
    
    /// Register early console
    fn register_early_console(&mut self) {
        // TODO: Register UART console
    }
    
    /// Register console
    pub fn register_console(&mut self, console: *mut Console) -> i32 {
        if console.is_null() {
            return Errno::Einval.to_ret_i32();
        }
        
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            (*console).next = self.consoles;
            self.consoles = console;
            (*console).index = self.console_count.load(Ordering::Acquire);
        }
        
        self.console_count.fetch_add(1, Ordering::AcqRel);
        
        // Set as current if first console
        if self.current_console.is_null() {
            self.current_console = console;
        }
        
        0
    }
    
    /// Unregister console
    pub fn unregister_console(&mut self, console: *mut Console) {
        if console.is_null() {
            return;
        }
        
        // TODO: Remove from list
        self.console_count.fetch_sub(1, Ordering::AcqRel);
    }
    
    /// Set log level
    pub fn set_log_level(&mut self, level: LogLevel) {
        self.log_level.store(level as u32, Ordering::Release);
    }
    
    /// Get log level
    pub fn get_log_level(&self) -> LogLevel {
        LogLevel::from_u32(self.log_level.load(Ordering::Acquire))
    }
    
    /// Print message
    pub fn print(&mut self, level: LogLevel, msg: &[u8]) {
        // Check log level
        if level as u32 > self.log_level.load(Ordering::Acquire) {
            return;
        }
        
        // Create log record
        let record = LogRecord::new(level, msg);
        
        // Write to buffer
        self.buffer.write(&record);
        
        // Write to console
        self.console_write(level, msg);
        
        // Update stats
        self.stats.messages.fetch_add(1, Ordering::AcqRel);
        self.stats.bytes.fetch_add(msg.len() as u64, Ordering::AcqRel);
    }
    
    /// Write to console
    fn console_write(&mut self, level: LogLevel, msg: &[u8]) {
        let console = self.current_console;
        if console.is_null() {
            return;
        }
        
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            if let Some(write) = (*console).ops.write {
                // Write level prefix
                let prefix = match level {
                    LogLevel::Emerg => b"[EMERG ] ",
                    LogLevel::Alert => b"[ALERT ] ",
                    LogLevel::Crit => b"[CRIT  ] ",
                    LogLevel::Err => b"[ERROR ] ",
                    LogLevel::Warning => b"[WARN  ] ",
                    LogLevel::Notice => b"[NOTICE] ",
                    LogLevel::Info => b"[INFO  ] ",
                    LogLevel::Debug => b"[DEBUG ] ",
                };
                
                write(prefix.as_ptr(), prefix.len());
                write(msg.as_ptr(), msg.len());
                
                // Write newline if not present
                if !msg.is_empty() && msg[msg.len() - 1] != b'\n' {
                    write(b"\n".as_ptr(), 1);
                }
            }
        }
    }
    
    /// Flush console
    pub fn flush(&mut self) {
        let console = self.current_console;
        if console.is_null() {
            return;
        }
        
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            if let Some(flush) = (*console).ops.flush {
                flush();
            }
        }
    }
    
    /// Dump log buffer
    pub fn dump(&mut self) {
        loop {
            let record = self.buffer.read();
            if let Some(r) = record {
                let level = r.level;
                let msg_len = r.msg_len as usize;
                let mut msg_buf = [0u8; 256];
                msg_buf[..msg_len].copy_from_slice(&r.msg[..msg_len]);
                drop(record);
                self.console_write(level, &msg_buf[..msg_len]);
            } else {
                break;
            }
        }
    }
}

/// Global log manager
static LOG_MANAGER: core::sync::OnceLock<LogManager> = core::sync::OnceLock::new();

/// Get log manager
pub fn log_manager() -> &'static LogManager {
    LOG_MANAGER.get_or_init(LogManager::new)
}

pub fn init_log_manager() -> &'static LogManager {
    LOG_MANAGER.get_or_init(LogManager::new)
}

/// Initialize logging
pub fn init_log() {
    let mgr = log_manager();
    mgr.init();
}

/// Print message
pub fn printk(level: LogLevel, msg: &[u8]) {
    log_manager().print(level, msg);
}

/// Print formatted message
pub fn printk_fmt(level: LogLevel, fmt: &[u8], args: &[u64]) {
    // Simple format implementation
    let mut msg = [0u8; 256];
    let mut pos = 0;
    
    let mut arg_idx = 0;
    let mut i = 0;
    
    while i < fmt.len() && pos < 250 {
        if fmt[i] == b'{' && i + 1 < fmt.len() && fmt[i + 1] == b'}' {
            // Format placeholder
            if arg_idx < args.len() {
                let num = args[arg_idx];
                arg_idx += 1;
                
                // Convert number to string
                let num_str = format_u64(num);
                for c in num_str.iter() {
                    if pos < 250 {
                        msg[pos] = *c;
                        pos += 1;
                    }
                }
            }
            i += 2;
        } else {
            msg[pos] = fmt[i];
            pos += 1;
            i += 1;
        }
    }
    
    log_manager().print(level, &msg[..pos]);
}

/// Format u64 to string
fn format_u64(mut n: u64) -> [u8; 21] {
    let mut result = [0u8; 21];
    let mut pos = 20;
    
    if n == 0 {
        result[20] = b'0';
        return result;
    }
    
    while n > 0 {
        result[pos] = b'0' + (n % 10) as u8;
        n /= 10;
        pos -= 1;
    }
    
    result
}
