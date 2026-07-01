/*
 * Nuva OS - Kernel - Profiling
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

//! ProfilingModule
/*!*/
//! Provides kernel profiling and hotspot detection capabilities.
/*!*/
//! # Features
/*!*/
//! - CPU PerformanceCounter
//! - Function execution time measurement
//! - Memory allocation statistics
//! - InterruptDelayMeasurement
/*!*/
//! # Usage Example
/*!*/
//! ```ignore
//! // Measure function execution time
//! let timer = PerfTimer::start();
//! some_function();
//! let elapsed = timer.elapsed();
/*!*/
//! // Record performance event
//! perf_event!(PERF_EVENT_CONTEXT_SWITCH);
//! ```

use crate::{pr_info};
use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// PerformanceEventType
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum PerfEventType {
    /// ContextSwitch
    ContextSwitch = 0,
    /// System call
    Syscall = 1,
    /// InterruptHandle
    Interrupt = 2,
    /// Page fault
    PageFault = 3,
    /// MemoryAllocate
    MemoryAlloc = 4,
    /// MemoryFree
    MemoryFree = 5,
    /// File read
    FileRead = 6,
    /// File write
    FileWrite = 7,
    /// NetworkSend
    NetSend = 8,
    /// NetworkReceive
    NetRecv = 9,
    /// Lock contention
    LockContention = 10,
    /// Scheduling latency
    ScheduleLatency = 11,
}

/// PerformanceEventCounter
pub struct PerfEventCounter {
    /// EventCount
    pub count: AtomicU64,
    /// Total time (nanoseconds)
    pub total_time: AtomicU64,
    /// Max time (nanoseconds)
    pub max_time: AtomicU64,
    /// Min time (nanoseconds)
    pub min_time: AtomicU64,
}

impl PerfEventCounter {
    pub const fn new() -> Self {
        PerfEventCounter {
            count: AtomicU64::new(0),
            total_time: AtomicU64::new(0),
            max_time: AtomicU64::new(0),
            min_time: AtomicU64::new(u64::MAX),
        }
    }

    /// Record event
    pub fn record(&self, duration_ns: u64) {
        self.count.fetch_add(1, Ordering::Relaxed);
        self.total_time.fetch_add(duration_ns, Ordering::Relaxed);

        // Update max value
        let mut current_max = self.max_time.load(Ordering::Relaxed);
        while duration_ns > current_max {
            match self.max_time.compare_exchange_weak(
                current_max,
                duration_ns,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(actual) => current_max = actual,
            }
        }

        // Update min value
        let mut current_min = self.min_time.load(Ordering::Relaxed);
        while duration_ns < current_min {
            match self.min_time.compare_exchange_weak(
                current_min,
                duration_ns,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(actual) => current_min = actual,
            }
        }
    }

    /// Get average time
    pub fn avg_time(&self) -> u64 {
        let count = self.count.load(Ordering::Relaxed);
        if count == 0 {
            return 0;
        }
        self.total_time.load(Ordering::Relaxed) / count
    }

    /// ResetCounter
    pub fn reset(&self) {
        self.count.store(0, Ordering::Relaxed);
        self.total_time.store(0, Ordering::Relaxed);
        self.max_time.store(0, Ordering::Relaxed);
        self.min_time.store(u64::MAX, Ordering::Relaxed);
    }
}

/// Performance statistics
pub struct PerfStats {
    /// EventCounterArray
    pub events: [PerfEventCounter; 12],
    /// CPU PeriodCount
    pub cpu_cycles: AtomicU64,
    /// InstructionCount
    pub instructions: AtomicU64,
    /// Cache misses
    pub cache_misses: AtomicU64,
    /// BranchPredictError
    pub branch_misses: AtomicU64,
}

impl PerfStats {
    pub const fn new() -> Self {
        PerfStats {
            events: [
                PerfEventCounter::new(),  // ContextSwitch
                PerfEventCounter::new(),  // Syscall
                PerfEventCounter::new(),  // Interrupt
                PerfEventCounter::new(),  // PageFault
                PerfEventCounter::new(),  // MemoryAlloc
                PerfEventCounter::new(),  // MemoryFree
                PerfEventCounter::new(),  // FileRead
                PerfEventCounter::new(),  // FileWrite
                PerfEventCounter::new(),  // NetSend
                PerfEventCounter::new(),  // NetRecv
                PerfEventCounter::new(),  // LockContention
                PerfEventCounter::new(),  // ScheduleLatency
            ],
            cpu_cycles: AtomicU64::new(0),
            instructions: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
            branch_misses: AtomicU64::new(0),
        }
    }

    /// Record event
    pub fn record_event(&self, event_type: PerfEventType, duration_ns: u64) {
        let idx = event_type as usize;
        if idx < self.events.len() {
            self.events[idx].record(duration_ns);
        }
    }

    /// Print statistics info
    pub fn print_stats(&self) {
        log_info!("=== Performance Statistics ===");

        let event_names = [
            "Context Switch",
            "Syscall",
            "Interrupt",
            "Page Fault",
            "Memory Alloc",
            "Memory Free",
            "File Read",
            "File Write",
            "Net Send",
            "Net Recv",
            "Lock Contention",
            "Schedule Latency",
        ];

        for (i, name) in event_names.iter().enumerate() {
            let counter = &self.events[i];
            let count = counter.count.load(Ordering::Relaxed);
            if count > 0 {
                log_info!(
                    "  {}: count={}, avg={}ns, max={}ns, min={}ns",
                    name,
                    count,
                    counter.avg_time(),
                    counter.max_time.load(Ordering::Relaxed),
                    counter.min_time.load(Ordering::Relaxed)
                );
            }
        }

        log_info!("  CPU Cycles: {}", self.cpu_cycles.load(Ordering::Relaxed));
        log_info!("  Instructions: {}", self.instructions.load(Ordering::Relaxed));
        log_info!("  Cache Misses: {}", self.cache_misses.load(Ordering::Relaxed));
        log_info!("  Branch Misses: {}", self.branch_misses.load(Ordering::Relaxed));
    }

    /// Reset all counters
    pub fn reset(&self) {
        for event in &self.events {
            event.reset();
        }
        self.cpu_cycles.store(0, Ordering::Relaxed);
        self.instructions.store(0, Ordering::Relaxed);
        self.cache_misses.store(0, Ordering::Relaxed);
        self.branch_misses.store(0, Ordering::Relaxed);
    }
}

/// Global performance statistics
static PERF_STATS: crate::sync_oncelock::OnceLock<PerfStats> = crate::sync_oncelock::OnceLock::new();

/// Get performance statistics
pub fn get_perf_stats() -> &'static PerfStats {
    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe { &PERF_STATS }
}

/// Performance timer
pub struct PerfTimer {
    start_time: u64,
}

impl PerfTimer {
    /// Create and start timer
    pub fn start() -> Self {
        PerfTimer {
            start_time: Self::read_cycle_counter(),
        }
    }

    /// Read cycle counter
    fn read_cycle_counter() -> u64 {
        #[cfg(target_arch = "aarch64")]
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let mut cnt: u64;
            core::arch::asm!("mrs {}, cntvct_el0", out(reg) cnt);
            cnt
        }

        #[cfg(target_arch = "x86_64")]
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let mut low: u32;
            let mut high: u32;
            core::arch::asm!("rdtsc", out("eax") low, out("edx") high);
            ((high as u64) << 32) | (low as u64)
        }

        #[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
        0
    }

    /// Get elapsed cycles
    pub fn elapsed_cycles(&self) -> u64 {
        Self::read_cycle_counter().saturating_sub(self.start_time)
    }

    /// Get elapsed time (nanoseconds)
    pub fn elapsed_ns(&self) -> u64 {
        // Assume 1 GHz clock
        self.elapsed_cycles()
    }

    /// Get elapsed time (microseconds)
    pub fn elapsed_us(&self) -> u64 {
        self.elapsed_ns() / 1000
    }

    /// Get elapsed time (milliseconds)
    pub fn elapsed_ms(&self) -> u64 {
        self.elapsed_us() / 1000
    }

    /// Stop timer and record event
    pub fn stop_and_record(&self, event_type: PerfEventType) {
        let duration = self.elapsed_ns();
        get_perf_stats().record_event(event_type, duration);
    }
}

/// Performance event macro
#[macro_export]
macro_rules! perf_event {
    ($event_type:expr) => {
        let _timer = $crate::kernel::debug::perf::PerfTimer::start();
    };
}

/// Performance event macro with recording
#[macro_export]
macro_rules! perf_event_record {
    ($event_type:expr, $code:block) => {
        {
            let timer = $crate::kernel::debug::perf::PerfTimer::start();
            let result = $code;
            timer.stop_and_record($event_type);
            result
        }
    };
}

/// heatDotFunctionTracking
pub struct HotspotTracker {
    /// Functionname
    pub name: &'static str,
    /// tuneusetimenumber
    pub call_count: AtomicU64,
    /// Total time
    pub total_time: AtomicU64,
}

impl HotspotTracker {
    pub const fn new(name: &'static str) -> Self {
        HotspotTracker {
            name,
            call_count: AtomicU64::new(0),
            total_time: AtomicU64::new(0),
        }
    }

    /// Record call
    pub fn record(&self, duration_ns: u64) {
        self.call_count.fetch_add(1, Ordering::Relaxed);
        self.total_time.fetch_add(duration_ns, Ordering::Relaxed);
    }

    /// Get average time
    pub fn avg_time(&self) -> u64 {
        let count = self.call_count.load(Ordering::Relaxed);
        if count == 0 {
            return 0;
        }
        self.total_time.load(Ordering::Relaxed) / count
    }
}

/// InitializeProfiling
pub fn init_perf() {
    log_info!("Performance analysis initialized");
}

/// Print performance report
pub fn print_perf_report() {
    get_perf_stats().print_stats();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perf_event_counter() {
        let counter = PerfEventCounter::new();

        counter.record(100);
        counter.record(200);
        counter.record(300);

        assert_eq!(counter.count.load(Ordering::Relaxed), 3);
        assert_eq!(counter.total_time.load(Ordering::Relaxed), 600);
        assert_eq!(counter.avg_time(), 200);
        assert_eq!(counter.max_time.load(Ordering::Relaxed), 300);
        assert_eq!(counter.min_time.load(Ordering::Relaxed), 100);
    }

    #[test]
    fn test_perf_event_counter_reset() {
        let counter = PerfEventCounter::new();

        counter.record(100);
        counter.reset();

        assert_eq!(counter.count.load(Ordering::Relaxed), 0);
        assert_eq!(counter.total_time.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_perf_stats() {
        let stats = PerfStats::new();

        stats.record_event(PerfEventType::Syscall, 1000);
        stats.record_event(PerfEventType::Syscall, 2000);

        let counter = &stats.events[PerfEventType::Syscall as usize];
        assert_eq!(counter.count.load(Ordering::Relaxed), 2);
        assert_eq!(counter.avg_time(), 1500);
    }

    #[test]
    fn test_perf_timer() {
        let timer = PerfTimer::start();

        // Simple delay
        for _ in 0..1000 {
            core::hint::spin_loop();
        }

        let elapsed = timer.elapsed_cycles();
        assert!(elapsed > 0);
    }

    #[test]
    fn test_hotspot_tracker() {
        let tracker = HotspotTracker::new("test_function");

        tracker.record(100);
        tracker.record(200);

        assert_eq!(tracker.call_count.load(Ordering::Relaxed), 2);
        assert_eq!(tracker.avg_time(), 150);
    }

    #[test]
    fn test_perf_event_type() {
        assert_eq!(PerfEventType::ContextSwitch as u32, 0);
        assert_eq!(PerfEventType::Syscall as u32, 1);
        assert_eq!(PerfEventType::Interrupt as u32, 2);
    }
}