use crate::{pr_debug, pr_info};
/*
 * Nuva OS - Kernel - Debug Support
 * 
 * Copyright (C) 2026 Nuva OS Team
 * 
 * Kernel debugging and tracing support.
 */

use core::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};

use crate::posix::errno::Errno;
/// Debug Event
#[repr(C)]
#[derive(Clone)]
pub struct DebugEvent {
 /// Event type
 pub event_type: DebugEventType,
 /// CPU ID
 pub cpu: u32,
 /// Process ID
 pub pid: u32,
 /// Thread ID
 pub tid: u32,
 /// Timestamp
 pub timestamp: u64,
 /// Event data
 pub data: DebugEventData,
}

/// Debug Event Type
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugEventType {
 /// Function entry
 FuncEntry = 1,
 /// Function exit
 FuncExit = 2,
 /// Memory allocation
 MemAlloc = 3,
 /// Memory free
 MemFree = 4,
 /// Lock acquire
 LockAcquire = 5,
 /// Lock release
 LockRelease = 6,
 /// IRQ entry
 IrqEntry = 7,
 /// IRQ exit
 IrqExit = 8,
 /// Schedule
 Schedule = 9,
 /// System call
 Syscall = 10,
 /// Custom event
 Custom = 255,
}

/// Debug Event Data
#[repr(C)]
pub union DebugEventData {
 /// Function data
 pub func: FuncData,
 /// Memory data
 pub mem: MemData,
 /// Lock data
 pub lock: LockData,
 /// IRQ data
 pub irq: IrqData,
 /// Schedule data
 pub sched: SchedData,
 /// System call data
 pub syscall: SyscallData,
 /// Raw data
 pub raw: [u64; 4],
}

impl Clone for DebugEventData {
    fn clone(&self) -> Self {
        // SAFETY: union copy is safe since all fields are Copy
        unsafe { DebugEventData { raw: self.raw } }
    }
}
impl Copy for DebugEventData {}

/// Function Trace Data
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FuncData {
 pub func_addr: u64,
 pub caller: u64,
 pub arg0: u64,
}

/// Memory Trace Data
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MemData {
 pub addr: u64,
 pub size: u64,
 pub flags: u32,
}

/// Lock Trace Data
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct LockData {
 pub lock_addr: u64,
 pub lock_type: u32,
 pub flags: u32,
}

/// IRQ Trace Data
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct IrqData {
 pub irq: u32,
 pub vector: u32,
}

/// Schedule Trace Data
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SchedData {
 pub prev_pid: u32,
 pub next_pid: u32,
 pub prev_prio: u32,
 pub next_prio: u32,
}

/// System Call Trace Data
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SyscallData {
 pub nr: u64,
 pub arg0: u64,
 pub arg1: u64,
 pub arg2: u64,
}

/// Debug Buffer
pub struct DebugBuffer {
 /// Buffer
 pub buffer: [DebugEvent; 1024],
 /// Head
 pub head: AtomicU32,
 /// Tail
 pub tail: AtomicU32,
 /// Count
 pub count: AtomicU32,
 /// Overflow
 pub overflow: AtomicU64,
}

impl DebugBuffer {
 pub const fn new() -> Self {
 DebugBuffer {
 buffer: [const { DebugEvent {
 event_type: DebugEventType::FuncEntry,
 cpu: 0,
 pid: 0,
 tid: 0,
 timestamp: 0,
 data: DebugEventData { raw: [0; 4] },
 } }; 1024],
 head: AtomicU32::new(0),
 tail: AtomicU32::new(0),
 count: AtomicU32::new(0),
 overflow: AtomicU64::new(0),
 }
 }
 
 /// Write event
 pub fn write(&mut self, event: &DebugEvent) {
 let head = self.head.load(Ordering::Acquire);
 let next = (head + 1) % 1024;
 
 if next == self.tail.load(Ordering::Acquire) {
 // Buffer full
 self.overflow.fetch_add(1, Ordering::AcqRel);
 return;
 }
 
 self.buffer[head as usize] = event.clone();
 self.head.store(next, Ordering::Release);
 self.count.fetch_add(1, Ordering::AcqRel);
 }
 
 /// Read event
 pub fn read(&mut self) -> Option<&DebugEvent> {
 let tail = self.tail.load(Ordering::Acquire);
 
 if tail == self.head.load(Ordering::Acquire) {
 return None;
 }
 
 let event = &self.buffer[tail as usize];
 self.tail.store((tail + 1) % 1024, Ordering::Release);
 self.count.fetch_sub(1, Ordering::AcqRel);
 
 Some(event)
 }
}

/// Breakpoint
#[repr(C)]
pub struct Breakpoint {
 /// Address
 pub addr: u64,
 /// Type
 pub bp_type: BreakpointType,
 /// Length
 pub len: u8,
 /// Enabled
 pub enabled: bool,
 /// Hit count
 pub hits: AtomicU64,
}

/// Breakpoint Type
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BreakpointType {
 /// Execute
 Execute = 0,
 /// Write
 Write = 1,
 /// Read/Write
 Access = 2,
}

/// Watchpoint
#[repr(C)]
pub struct Watchpoint {
 /// Address
 pub addr: u64,
 /// Size
 pub size: u8,
 /// Value
 pub value: u64,
 /// Mask
 pub mask: u64,
 /// Enabled
 pub enabled: bool,
}

/// Debug Manager
pub struct DebugManager {
 /// Enabled
 pub enabled: AtomicBool,
 /// Tracing enabled
 pub tracing: AtomicBool,
 /// Function tracing
 pub func_trace: AtomicBool,
 /// Memory tracing
 pub mem_trace: AtomicBool,
 /// Lock tracing
 pub lock_trace: AtomicBool,
 /// IRQ tracing
 pub irq_trace: AtomicBool,
 /// Scheduler tracing
 pub sched_trace: AtomicBool,
 /// System call tracing
 pub syscall_trace: AtomicBool,
 /// Debug buffer
 pub buffer: DebugBuffer,
 /// Breakpoints
 pub breakpoints: [Option<Breakpoint>; 16],
 /// Watchpoints
 pub watchpoints: [Option<Watchpoint>; 16],
 /// Statistics
 pub stats: DebugStats,
}

/// Debug Statistics
pub struct DebugStats {
 pub events_logged: AtomicU64,
 pub func_entries: AtomicU64,
 pub func_exits: AtomicU64,
 pub mem_allocs: AtomicU64,
 pub mem_frees: AtomicU64,
 pub bp_hits: AtomicU64,
 pub wp_hits: AtomicU64,
}

impl DebugStats {
 pub const fn new() -> Self {
 DebugStats {
 events_logged: AtomicU64::new(0),
 func_entries: AtomicU64::new(0),
 func_exits: AtomicU64::new(0),
 mem_allocs: AtomicU64::new(0),
 mem_frees: AtomicU64::new(0),
 bp_hits: AtomicU64::new(0),
 wp_hits: AtomicU64::new(0),
 }
 }
}

impl DebugManager {
 pub const fn new() -> Self {
 DebugManager {
 enabled: AtomicBool::new(false),
 tracing: AtomicBool::new(false),
 func_trace: AtomicBool::new(false),
 mem_trace: AtomicBool::new(false),
 lock_trace: AtomicBool::new(false),
 irq_trace: AtomicBool::new(false),
 sched_trace: AtomicBool::new(false),
 syscall_trace: AtomicBool::new(false),
 buffer: DebugBuffer::new(),
 breakpoints: [const { None }; 16],
 watchpoints: [const { None }; 16],
 stats: DebugStats::new(),
 }
 }
 
 /// Initialize
 pub fn init(&self) {
 log_info!("Debug manager initialized");
 }
 
 /// Enable debugging
 pub fn enable(&mut self) {
 self.enabled.store(true, Ordering::Release);
 log_info!("Debugging enabled");
 }
 
 /// Disable debugging
 pub fn disable(&mut self) {
 self.enabled.store(false, Ordering::Release);
 log_info!("Debugging disabled");
 }
 
 /// Enable tracing
 pub fn enable_tracing(&mut self, flags: u32) {
 self.tracing.store(true, Ordering::Release);
 
 if (flags & (1 << 0)) != 0 {
 self.func_trace.store(true, Ordering::Release);
 }
 if (flags & (1 << 1)) != 0 {
 self.mem_trace.store(true, Ordering::Release);
 }
 if (flags & (1 << 2)) != 0 {
 self.lock_trace.store(true, Ordering::Release);
 }
 if (flags & (1 << 3)) != 0 {
 self.irq_trace.store(true, Ordering::Release);
 }
 if (flags & (1 << 4)) != 0 {
 self.sched_trace.store(true, Ordering::Release);
 }
 if (flags & (1 << 5)) != 0 {
 self.syscall_trace.store(true, Ordering::Release);
 }
 }
 
 /// Log function entry
 pub fn log_func_entry(&mut self, func_addr: u64, caller: u64, arg: u64) {
 if self.func_trace.load(Ordering::Acquire) {
 self.stats.func_entries.fetch_add(1, Ordering::AcqRel);
 
 // Log event
 let event = DebugEvent {
 event_type: DebugEventType::FuncEntry,
 cpu: 0,
 pid: 0,
 tid: 0,
 timestamp: 0,
 data: DebugEventData {
 func: FuncData {
 func_addr,
 caller,
 arg0: arg,
 },
 },
 };
 self.buffer.write(&event);
 }
 }
 
 /// Log function exit
 pub fn log_func_exit(&mut self, func_addr: u64, retval: u64) {
 if !self.func_trace.load(Ordering::Acquire) {
 self.stats.func_exits.fetch_add(1, Ordering::AcqRel);
 let _ = (func_addr, retval);
 }
 }
 
 /// Log memory allocation
 pub fn log_mem_alloc(&mut self, addr: u64, size: u64, flags: u32) {
 if self.mem_trace.load(Ordering::Acquire) {
 self.stats.mem_allocs.fetch_add(1, Ordering::AcqRel);
 let _ = (addr, size, flags);
 }
 }
 
 /// Log memory free
 pub fn log_mem_free(&mut self, addr: u64, size: u64) {
 if self.mem_trace.load(Ordering::Acquire) {
 self.stats.mem_frees.fetch_add(1, Ordering::AcqRel);
 let _ = (addr, size);
 }
 }
 
 /// Set breakpoint
 pub fn set_breakpoint(&mut self, idx: usize, addr: u64, bp_type: BreakpointType, len: u8) -> i32 {
 if idx >= 16 {
 return Errno::Einval.to_ret_i32();
 }
 
 self.breakpoints[idx] = Some(Breakpoint {
 addr,
 bp_type,
 len,
 enabled: true,
 hits: AtomicU64::new(0),
 });
 
 // Program hardware breakpoint
 // SimplifiedImplementation:Settingssoftcasebreakpoint
 // SAFETY: unsafe block required for low-level memory or hardware access
 unsafe {
 let ptr = addr as *mut u8;
 let original = *ptr;
 *ptr = 0xCC; // INT3 instruction
 
 // protectedexistRawcharacterSectionwithRecovery
 // realactualImplementationneedwantupdaterestorehybrid breakpointmanagementadministration
 }
 
 0
 }
 
 /// Clear breakpoint
 pub fn clear_breakpoint(&mut self, idx: usize) -> i32 {
 if idx >= 16 {
 return Errno::Einval.to_ret_i32();
 }
 
 self.breakpoints[idx] = None;
 0
 }
 
 /// Check breakpoint hit
 pub fn check_breakpoint(&mut self, addr: u64) -> bool {
 for bp in self.breakpoints.iter_mut() {
 if let Some(bp) = bp {
 if bp.enabled && bp.addr == addr {
 bp.hits.fetch_add(1, Ordering::AcqRel);
 self.stats.bp_hits.fetch_add(1, Ordering::AcqRel);
 return true;
 }
 }
 }
 false
 }
 
 /// Dump buffer
 pub fn dump_buffer(&mut self) {
 while let Some(event) = self.buffer.read() {
 match event.event_type {
 DebugEventType::FuncEntry => {
 // SAFETY: unsafe block required for low-level memory or hardware access
 log_debug!("FUNC_ENTRY: addr={:#x}", unsafe { event.data.func.func_addr });
 }
 DebugEventType::FuncExit => {
 // SAFETY: unsafe block required for low-level memory or hardware access
 log_debug!("FUNC_EXIT: addr={:#x}", unsafe { event.data.func.func_addr });
 }
 DebugEventType::MemAlloc => {
 // SAFETY: unsafe block required for low-level memory or hardware access
 log_debug!("MEM_ALLOC: addr={:#x}, size={}", unsafe { event.data.mem.addr }, unsafe { event.data.mem.size });
 }
 DebugEventType::MemFree => {
 // SAFETY: unsafe block required for low-level memory or hardware access
 log_debug!("MEM_FREE: addr={:#x}", unsafe { event.data.mem.addr });
 }
 _ => {
 log_debug!("EVENT: {:?}", event.event_type);
 }
 }
 }
 }
 
 /// Get statistics
 pub fn get_stats(&self) -> &DebugStats {
 &self.stats
 }
}

/// Global debug manager
static DEBUG_MANAGER: core::sync::OnceLock<DebugManager> = core::sync::OnceLock::new();

/// Get debug manager
pub fn debug_manager() -> &'static DebugManager {
    DEBUG_MANAGER.get_or_init(DebugManager::new)
}

pub fn init_debug_manager() -> &'static DebugManager {
    DEBUG_MANAGER.get_or_init(DebugManager::new)
}

/// Initialize debug
pub fn init_debug() {
 let mgr = debug_manager();
 mgr.init();
}

// Trace macros

/// Trace function entry
#[macro_export]
macro_rules! trace_func_entry {
 () => {
 if $crate::debug::debug_manager().func_trace.load(core::sync::atomic::Ordering::Acquire) {
 $crate::debug::debug_manager().log_func_entry(
 $crate::arch::current_function_address!(),
 $crate::arch::return_address!(),
 00
 );
 }
 };
 ($arg:expr) => {
 if $crate::debug::debug_manager().func_trace.load(core::sync::atomic::Ordering::Acquire) {
 $crate::debug::debug_manager().log_func_entry(
 $crate::arch::current_function_address!(),
 $crate::arch::return_address!(),
 $arg as u64,
 );
 }
 };
}

/// Trace function exit
#[macro_export]
macro_rules! trace_func_exit {
 () => {
 if $crate::debug::debug_manager().func_trace.load(core::sync::atomic::Ordering::Acquire) {
 $crate::debug::debug_manager().log_func_exit(
 $crate::arch::current_function_address!(),
 0,
 );
 }
 };
 ($retval:expr) => {
 if $crate::debug::debug_manager().func_trace.load(core::sync::atomic::Ordering::Acquire) {
 $crate::debug::debug_manager().log_func_exit(
 $crate::arch::current_function_address!(),
 $retval as u64,
 );
 }
 };
}

/// Trace memory allocation
#[macro_export]
macro_rules! trace_mem_alloc {
 ($addr:expr, $size:expr, $flags:expr) => {
 if $crate::debug::debug_manager().mem_trace.load(core::sync::atomic::Ordering::Acquire) {
 $crate::debug::debug_manager().log_mem_alloc($addr as u64, $size as u64, $flags as u32);
 }
 };
}

/// Trace memory free
#[macro_export]
macro_rules! trace_mem_free {
 ($addr:expr, $size:expr) => {
 if $crate::debug::debug_manager().mem_trace.load(core::sync::atomic::Ordering::Acquire) {
 $crate::debug::debug_manager().log_mem_free($addr as u64, $size as u64);
 }
 };
}