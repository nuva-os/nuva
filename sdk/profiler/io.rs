/*
 * Nuva OS
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

// ! I/O profiler for disk I/O latency analysis

use std::collections::HashMap;
use crate::error::SdkError;
use alloc::vec;
use alloc::vec::Vec;

/// I/O profiler
pub struct IoProfiler {
    /// Whether profiling is active
    profiling: bool,
    /// Target process
    pid: Option<u32>,
    /// I/O event log
    events: Vec<IoEvent>,
}

impl IoProfiler {
    pub fn new() -> Self {
        Self {
            profiling: false,
            pid: None,
            events: vec![],
        }
    }

    /// Start I/O profiling
    pub fn start(&mut self, pid: Option<u32>) -> Result<(), SdkError> {
        self.profiling = true;
        self.pid = pid;
        self.events.clear();
        Ok(())
    }

    /// Stop I/O profiling
    pub fn stop(&mut self) -> Result<IoProfile, SdkError> {
        self.profiling = false;

        let stats = self.compute_stats();
        let latency_histogram = self.build_latency_histogram();

        Ok(IoProfile {
            events: std::mem::take(&mut self.events),
            stats,
            latency_histogram,
        })
    }

    /// Record an I/O event
    pub fn record_event(&mut self, event: IoEvent) {
        if self.profiling {
            self.events.push(event);
        }
    }

    /// Compute I/O statistics
    fn compute_stats(&self) -> IoStats {
        let mut stats = IoStats::default();

        for event in &self.events {
            match event.kind {
                IoKind::Read => {
                    stats.total_reads += 1;
                    stats.total_read_bytes += event.size;
                    stats.total_read_latency_us += event.latency_us;
                    stats.max_read_latency_us = stats.max_read_latency_us.max(event.latency_us);
                }
                IoKind::Write => {
                    stats.total_writes += 1;
                    stats.total_write_bytes += event.size;
                    stats.total_write_latency_us += event.latency_us;
                    stats.max_write_latency_us = stats.max_write_latency_us.max(event.latency_us);
                }
                IoKind::Sync => {
                    stats.total_syncs += 1;
                    stats.total_sync_latency_us += event.latency_us;
                }
                IoKind::Open => {
                    stats.total_opens += 1;
                }
                IoKind::Close => {
                    stats.total_closes += 1;
                }
            }
        }

        if stats.total_reads > 0 {
            stats.avg_read_latency_us = stats.total_read_latency_us / stats.total_reads as u64;
        }
        if stats.total_writes > 0 {
            stats.avg_write_latency_us = stats.total_write_latency_us / stats.total_writes as u64;
        }

        stats
    }

    /// Build latency histogram
    fn build_latency_histogram(&self) -> HashMap<String, usize> {
        let mut histogram = HashMap::new();
        let buckets = [
            ("<1us", 1u64),
            ("<10us", 10u64),
            ("<100us", 100u64),
            ("<1ms", 1_000u64),
            ("<10ms", 10_000u64),
            ("<100ms", 100_000u64),
            ("<1s", 1_000_000u64),
            (">=1s", u64::MAX),
        ];

        for event in &self.events {
            for (name, threshold) in &buckets {
                if event.latency_us < *threshold {
                    *histogram.entry(name.to_string()).or_insert(0) += 1;
                    break;
                }
            }
        }

        histogram
    }
}

impl Default for IoProfiler {
    fn default() -> Self {
        Self::new()
    }
}

/// I/O profile result
#[derive(Debug)]
pub struct IoProfile {
    /// I/O events
    pub events: Vec<IoEvent>,
    /// I/O statistics
    pub stats: IoStats,
    /// Latency histogram
    pub latency_histogram: HashMap<String, usize>,
}

/// I/O event
#[derive(Debug, Clone)]
pub struct IoEvent {
    /// Timestamp in microseconds
    pub timestamp_us: u64,
    /// I/O kind
    pub kind: IoKind,
    /// File path or descriptor
    pub target: String,
    /// Size in bytes
    pub size: usize,
    /// Latency in microseconds
    pub latency_us: u64,
    /// Offset in file
    pub offset: Option<u64>,
    /// Thread ID
    pub thread_id: u64,
}

/// I/O kind
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IoKind {
    Read,
    Write,
    Sync,
    Open,
    Close,
}

/// I/O statistics
#[derive(Debug, Default)]
pub struct IoStats {
    /// Total read operations
    pub total_reads: usize,
    /// Total write operations
    pub total_writes: usize,
    /// Total sync operations
    pub total_syncs: usize,
    /// Total open operations
    pub total_opens: usize,
    /// Total close operations
    pub total_closes: usize,
    /// Total bytes read
    pub total_read_bytes: usize,
    /// Total bytes written
    pub total_write_bytes: usize,
    /// Total read latency (microseconds)
    pub total_read_latency_us: u64,
    /// Total write latency (microseconds)
    pub total_write_latency_us: u64,
    /// Total sync latency (microseconds)
    pub total_sync_latency_us: u64,
    /// Average read latency (microseconds)
    pub avg_read_latency_us: u64,
    /// Average write latency (microseconds)
    pub avg_write_latency_us: u64,
    /// Maximum read latency (microseconds)
    pub max_read_latency_us: u64,
    /// Maximum write latency (microseconds)
    pub max_write_latency_us: u64,
}
