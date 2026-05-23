/*
 * Nuva OS - Kernel - Tombstone - Statistics
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

//! Tombstone statistics tracking.
/*!*/
//! Maintains atomic counters for tombstone generation, failures,
//! cache usage, and crash rate monitoring.

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/** Tombstone subsystem statistics (all fields atomically updated) */
#[repr(C)]
#[derive(Debug)]
pub struct TombstoneStats {
    /** Current total number of stored tombstone records */
    pub total_count: AtomicU32,
    /** Cumulative number of tombstones generated since boot */
    pub total_generated: AtomicU32,
    /** Cumulative number of generation failures */
    pub generation_failures: AtomicU32,
    /** Timestamp of the most recent crash (nanoseconds) */
    pub last_crash_ts: AtomicU64,
    /** PID of the most recently crashed process */
    pub last_crash_pid: AtomicU32,
    /** Crashes per minute (rolling average) */
    pub crash_rate_per_min: AtomicU32,
    /** Number of records currently in memory cache */
    pub mem_cache_count: AtomicU32,
    /** Number of times memory cache was flushed to FS */
    pub mem_cache_flush_count: AtomicU32,
}

impl TombstoneStats {
    /** Create a zero-initialized TombstoneStats */
    pub const fn new() -> Self {
        TombstoneStats {
            total_count: AtomicU32::new(0),
            total_generated: AtomicU32::new(0),
            generation_failures: AtomicU32::new(0),
            last_crash_ts: AtomicU64::new(0),
            last_crash_pid: AtomicU32::new(0),
            crash_rate_per_min: AtomicU32::new(0),
            mem_cache_count: AtomicU32::new(0),
            mem_cache_flush_count: AtomicU32::new(0),
        }
    }

    /** Atomically increment the generated counter */
    pub fn increment_generated(&self) {
        self.total_generated.fetch_add(1, Ordering::Relaxed);
        self.total_count.fetch_add(1, Ordering::Relaxed);
    }

    /** Atomically increment the failure counter */
    pub fn increment_failure(&self) {
        self.generation_failures.fetch_add(1, Ordering::Relaxed);
    }

    /** Atomically update the last crash info */
    pub fn update_last_crash(&self, pid: u32, ts: u64) {
        self.last_crash_pid.store(pid, Ordering::Relaxed);
        self.last_crash_ts.store(ts, Ordering::Relaxed);
    }

    /** Update crash rate (crashes per minute) */
    pub fn update_crash_rate(&self, rate: u32) {
        self.crash_rate_per_min.store(rate, Ordering::Relaxed);
    }

    /** Decrement total count (after pruning) */
    pub fn decrement_count(&self, n: u32) {
        self.total_count.fetch_sub(n, Ordering::Relaxed);
    }

    /** Increment memory cache count */
    pub fn increment_mem_cache(&self) {
        self.mem_cache_count.fetch_add(1, Ordering::Relaxed);
    }

    /** Decrement memory cache count */
    pub fn decrement_mem_cache(&self) {
        self.mem_cache_count.fetch_sub(1, Ordering::Relaxed);
    }

    /** Increment flush count and reset mem cache count */
    pub fn record_flush(&self, cached: u32) {
        self.mem_cache_flush_count.fetch_add(1, Ordering::Relaxed);
        self.mem_cache_count.fetch_sub(
            cached.min(self.mem_cache_count.load(Ordering::Relaxed)),
            Ordering::Relaxed,
        );
    }

    /** Take a consistent snapshot for reporting */
    pub fn snapshot(&self) -> TombstoneStatsSnapshot {
        TombstoneStatsSnapshot {
            total_count: self.total_count.load(Ordering::Relaxed),
            total_generated: self.total_generated.load(Ordering::Relaxed),
            generation_failures: self.generation_failures.load(Ordering::Relaxed),
            last_crash_ts: self.last_crash_ts.load(Ordering::Relaxed),
            last_crash_pid: self.last_crash_pid.load(Ordering::Relaxed),
            crash_rate_per_min: self.crash_rate_per_min.load(Ordering::Relaxed),
            mem_cache_count: self.mem_cache_count.load(Ordering::Relaxed),
            mem_cache_flush_count: self.mem_cache_flush_count.load(Ordering::Relaxed),
        }
    }
}

/** Immutable snapshot of statistics for user-space reporting */
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct TombstoneStatsSnapshot {
    pub total_count: u32,
    pub total_generated: u32,
    pub generation_failures: u32,
    pub last_crash_ts: u64,
    pub last_crash_pid: u32,
    pub crash_rate_per_min: u32,
    pub mem_cache_count: u32,
    pub mem_cache_flush_count: u32,
}
