/*
 * Nuva OS - Kernel - ftrace (Function Tracer)
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

use core::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use alloc::vec::Vec;

use crate::posix::errno::Errno;
/// ftrace trace record
/// Captures function entry/exit events with metadata.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct FtraceRecord {
    /// Timestamp (cycle counter)
    pub timestamp: u64,
    /// CPU ID
    pub cpu_id: u32,
    /// Function address
    pub func_addr: u64,
    /// Caller (return) address
    pub caller_addr: u64,
    /// Record type: 0 = entry, 1 = exit
    pub record_type: u8,
    /// Reserved
    pub _reserved: [u8; 3],
}

impl FtraceRecord {
    pub const fn new() -> Self {
        FtraceRecord {
            timestamp: 0,
            cpu_id: 0,
            func_addr: 0,
            caller_addr: 0,
            record_type: 0,
            _reserved: [0; 3],
        }
    }

    pub fn is_entry(&self) -> bool {
        self.record_type == 0
    }

    pub fn is_exit(&self) -> bool {
        self.record_type == 1
    }
}

/// Maximum trace buffer records
const FTRACE_BUFFER_SIZE: usize = 4096;

/// Maximum filter functions
const FTRACE_MAX_FILTERS: usize = 256;

/// ftrace context
/// Manages function tracing state, filters, and trace buffer.
pub struct FtraceCtx {
    /// Enabled flag
    pub enabled: AtomicBool,
    /// Filter enabled flag
    pub filter_enabled: AtomicBool,
    /// Filter function addresses
    pub filter_addrs: [u64; FTRACE_MAX_FILTERS],
    /// Number of filter addresses
    pub filter_count: AtomicU32,
    /// Trace buffer (ring)
    pub buffer: [FtraceRecord; FTRACE_BUFFER_SIZE],
    /// Buffer head (write position)
    pub head: AtomicU32,
    /// Buffer tail (read position)
    pub tail: AtomicU32,
    /// Total records written
    pub total_records: AtomicU64,
    /// Records lost due to overflow
    pub lost_records: AtomicU64,
    /// Trace depth limit to prevent recursion
    pub max_depth: AtomicU32,
}

impl FtraceCtx {
    pub const fn new() -> Self {
        FtraceCtx {
            enabled: AtomicBool::new(false),
            filter_enabled: AtomicBool::new(false),
            filter_addrs: [0; FTRACE_MAX_FILTERS],
            filter_count: AtomicU32::new(0),
            buffer: [const { FtraceRecord::new() }; FTRACE_BUFFER_SIZE],
            head: AtomicU32::new(0),
            tail: AtomicU32::new(0),
            total_records: AtomicU64::new(0),
            lost_records: AtomicU64::new(0),
            max_depth: AtomicU32::new(64),
        }
    }

    /// Enable ftrace
    pub fn enable(&self) {
        self.enabled.store(true, Ordering::Release);
    }

    /// Disable ftrace
    pub fn disable(&self) {
        self.enabled.store(false, Ordering::Release);
    }

    /// Check if a function address passes the filter
    pub fn passes_filter(&self, func_addr: u64) -> bool {
        if !self.filter_enabled.load(Ordering::Acquire) {
            return true;
        }
        let count = self.filter_count.load(Ordering::Acquire) as usize;
        for i in 0..count {
            if self.filter_addrs[i] == func_addr {
                return true;
            }
        }
        false
    }

    /// Add a function address to the filter set
    /// @return: 0 on success, negative errno on failure
    pub fn add_filter(&mut self, func_addr: u64) -> i32 {
        let count = self.filter_count.load(Ordering::Acquire) as usize;
        if count >= FTRACE_MAX_FILTERS {
            return Errno::Enospc.to_ret_i32(); // ENOSPC
        }
        self.filter_addrs[count] = func_addr;
        self.filter_count.fetch_add(1, Ordering::AcqRel);
        0
    }

    /// Remove a function address from the filter set
    /// @return: 0 on success, negative errno on failure
    pub fn remove_filter(&mut self, func_addr: u64) -> i32 {
        let count = self.filter_count.load(Ordering::Acquire) as usize;
        for i in 0..count {
            if self.filter_addrs[i] == func_addr {
                self.filter_addrs[i] = self.filter_addrs[count - 1];
                self.filter_count.fetch_sub(1, Ordering::AcqRel);
                return 0;
            }
        }
        -2 // ENOENT
    }

    /// Set filter from a list of function addresses
    pub fn set_filter(&mut self, addrs: &[u64]) -> i32 {
        let limit = addrs.len().min(FTRACE_MAX_FILTERS);
        for i in 0..limit {
            self.filter_addrs[i] = addrs[i];
        }
        self.filter_count.store(limit as u32, Ordering::Release);
        self.filter_enabled.store(true, Ordering::Release);
        0
    }

    /// Clear all filters
    pub fn clear_filter(&mut self) {
        self.filter_count.store(0, Ordering::Release);
        self.filter_enabled.store(false, Ordering::Release);
    }

    /// Record a function entry tracepoint
    pub fn trace_entry(&mut self, func_addr: u64, caller_addr: u64, cpu_id: u32) {
        if !self.enabled.load(Ordering::Acquire) {
            return;
        }
        if !self.passes_filter(func_addr) {
            return;
        }
        self.write_record(func_addr, caller_addr, cpu_id, 0);
    }

    /// Record a function exit tracepoint
    pub fn trace_exit(&mut self, func_addr: u64, caller_addr: u64, cpu_id: u32) {
        if !self.enabled.load(Ordering::Acquire) {
            return;
        }
        if !self.passes_filter(func_addr) {
            return;
        }
        self.write_record(func_addr, caller_addr, cpu_id, 1);
    }

    /// Write a record into the ring buffer
    fn write_record(&mut self, func_addr: u64, caller_addr: u64, cpu_id: u32, record_type: u8) {
        let head = self.head.load(Ordering::Acquire);
        let next_head = (head + 1) % FTRACE_BUFFER_SIZE as u32;

        if next_head == self.tail.load(Ordering::Acquire) {
            self.lost_records.fetch_add(1, Ordering::AcqRel);
            return;
        }

        let record = FtraceRecord {
            timestamp: read_timestamp(),
            cpu_id,
            func_addr,
            caller_addr,
            record_type,
            _reserved: [0; 3],
        };

        self.buffer[head as usize] = record;
        self.head.store(next_head, Ordering::Release);
        self.total_records.fetch_add(1, Ordering::AcqRel);
    }

    /// Read a record from the ring buffer
    pub fn read_record(&mut self) -> Option<FtraceRecord> {
        let tail = self.tail.load(Ordering::Acquire);
        if tail == self.head.load(Ordering::Acquire) {
            return None;
        }
        let record = self.buffer[tail as usize];
        self.tail.store((tail + 1) % FTRACE_BUFFER_SIZE as u32, Ordering::Release);
        Some(record)
    }

    /// Drain all records into a Vec
    pub fn drain_records(&mut self) -> Vec<FtraceRecord> {
        let mut records = Vec::new();
        while let Some(rec) = self.read_record() {
            records.push(rec);
        }
        records
    }

    /// Reset the trace buffer
    pub fn reset(&mut self) {
        self.head.store(0, Ordering::Release);
        self.tail.store(0, Ordering::Release);
        self.total_records.store(0, Ordering::Release);
        self.lost_records.store(0, Ordering::Release);
    }
}

/// Read timestamp from cycle counter
fn read_timestamp() -> u64 {
    #[cfg(target_arch = "x86_64")]
    // SAFETY: reading the timestamp counter via RDTSC
    unsafe {
        let mut high: u32;
        let mut low: u32;
        core::arch::asm!(
            "rdtsc",
            out("eax") low,
            out("edx") high,
            options(nostack, preserves_flags)
        );
        ((high as u64) << 32) | (low as u64)
    }

    #[cfg(target_arch = "aarch64")]
    // SAFETY: reading the generic timer counter
    unsafe {
        let cycles: u64;
        core::arch::asm!(
            "mrs {}, cntvct_el0",
            out(reg) cycles,
            options(nostack, preserves_flags)
        );
        cycles
    }

    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    0
}

/// Global ftrace context
static FTRACE_CTX: core::sync::OnceLock<FtraceCtx> = core::sync::OnceLock::new();

/// Get global ftrace context
pub fn ftrace_ctx() -> &'static FtraceCtx {
    FTRACE_CTX.get_or_init(FtraceCtx::new)
}

/// Enable ftrace globally
pub fn ftrace_enable() {
    get_ftrace_ctx().enable();
}

/// Disable ftrace globally
pub fn ftrace_disable() {
    get_ftrace_ctx().disable();
}

/// Set ftrace filter from address list
pub fn ftrace_set_filter(addrs: &[u64]) -> i32 {
    get_ftrace_ctx().set_filter(addrs)
}

/// Initialize ftrace subsystem
pub fn init_ftrace() {
    let ctx = get_ftrace_ctx();
    ctx.reset();
}
