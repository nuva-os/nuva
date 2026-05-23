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

// ! Lock profiler for lock contention analysis

use std::collections::HashMap;
use crate::error::SdkError;

/// Lock profiler
pub struct LockProfiler {
    /// Whether profiling is active
    profiling: bool,
    /// Target process
    pid: Option<u32>,
    /// Lock events
    events: Vec<LockEvent>,
}

impl LockProfiler {
    pub fn new() -> Self {
        Self {
            profiling: false,
            pid: None,
            events: vec![],
        }
    }

    /// Start lock profiling
    pub fn start(&mut self, pid: Option<u32>) -> Result<(), SdkError> {
        self.profiling = true;
        self.pid = pid;
        self.events.clear();
        Ok(())
    }

    /// Stop lock profiling
    pub fn stop(&mut self) -> Result<LockProfile, SdkError> {
        self.profiling = false;

        let stats = self.compute_stats();
        let contention_map = self.build_contention_map();

        Ok(LockProfile {
            events: std::mem::take(&mut self.events),
            stats,
            contention_map,
        })
    }

    /// Record a lock event
    pub fn record_event(&mut self, event: LockEvent) {
        if self.profiling {
            self.events.push(event);
        }
    }

    /// Compute lock statistics
    fn compute_stats(&self) -> LockStats {
        let mut stats = LockStats::default();

        for event in &self.events {
            match event.kind {
                LockEventKind::Acquire => {
                    stats.total_acquires += 1;
                    if event.wait_time_us > 0 {
                        stats.contended_acquires += 1;
                        stats.total_wait_time_us += event.wait_time_us;
                        stats.max_wait_time_us = stats.max_wait_time_us.max(event.wait_time_us);
                    }
                }
                LockEventKind::Release => {
                    stats.total_releases += 1;
                    stats.total_hold_time_us += event.hold_time_us;
                    stats.max_hold_time_us = stats.max_hold_time_us.max(event.hold_time_us);
                }
                LockEventKind::TryAcquireFailed => {
                    stats.total_try_failures += 1;
                }
            }
        }

        if stats.contended_acquires > 0 {
            stats.avg_wait_time_us = stats.total_wait_time_us / stats.contended_acquires as u64;
        }
        if stats.total_releases > 0 {
            stats.avg_hold_time_us = stats.total_hold_time_us / stats.total_releases as u64;
        }

        stats
    }

    /// Build per-lock contention map
    fn build_contention_map(&self) -> HashMap<String, LockContention> {
        let mut map = HashMap::<String, LockContention>::new();

        for event in &self.events {
            let entry = map.entry(event.lock_name.clone()).or_default();
            match event.kind {
                LockEventKind::Acquire => {
                    entry.acquires += 1;
                    if event.wait_time_us > 0 {
                        entry.contended += 1;
                        entry.total_wait_us += event.wait_time_us;
                    }
                }
                LockEventKind::Release => {
                    entry.total_hold_us += event.hold_time_us;
                }
                LockEventKind::TryAcquireFailed => {
                    entry.try_failures += 1;
                }
            }
        }

        map
    }
}

impl Default for LockProfiler {
    fn default() -> Self {
        Self::new()
    }
}

/// Lock profile result
#[derive(Debug)]
pub struct LockProfile {
    /// Lock events
    pub events: Vec<LockEvent>,
    /// Lock statistics
    pub stats: LockStats,
    /// Per-lock contention map
    pub contention_map: HashMap<String, LockContention>,
}

/// Lock event
#[derive(Debug, Clone)]
pub struct LockEvent {
    /// Timestamp in microseconds
    pub timestamp_us: u64,
    /// Lock event kind
    pub kind: LockEventKind,
    /// Lock name or address
    pub lock_name: String,
    /// Wait time before acquisition (microseconds)
    pub wait_time_us: u64,
    /// Hold time for release events (microseconds)
    pub hold_time_us: u64,
    /// Thread ID
    pub thread_id: u64,
}

/// Lock event kind
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockEventKind {
    /// Lock acquired
    Acquire,
    /// Lock released
    Release,
    /// Try-acquire failed (non-blocking)
    TryAcquireFailed,
}

/// Lock statistics
#[derive(Debug, Default)]
pub struct LockStats {
    /// Total acquire operations
    pub total_acquires: usize,
    /// Total release operations
    pub total_releases: usize,
    /// Contended acquisitions (wait > 0)
    pub contended_acquires: usize,
    /// Try-acquire failures
    pub total_try_failures: usize,
    /// Total wait time (microseconds)
    pub total_wait_time_us: u64,
    /// Total hold time (microseconds)
    pub total_hold_time_us: u64,
    /// Average wait time (microseconds)
    pub avg_wait_time_us: u64,
    /// Average hold time (microseconds)
    pub avg_hold_time_us: u64,
    /// Maximum wait time (microseconds)
    pub max_wait_time_us: u64,
    /// Maximum hold time (microseconds)
    pub max_hold_time_us: u64,
}

/// Per-lock contention info
#[derive(Debug, Default)]
pub struct LockContention {
    /// Total acquires
    pub acquires: usize,
    /// Contended acquires
    pub contended: usize,
    /// Try failures
    pub try_failures: usize,
    /// Total wait time (microseconds)
    pub total_wait_us: u64,
    /// Total hold time (microseconds)
    pub total_hold_us: u64,
}

impl LockContention {
    /// Get contention ratio (0.0 - 1.0)
    pub fn contention_ratio(&self) -> f64 {
        if self.acquires == 0 {
            0.0
        } else {
            self.contended as f64 / self.acquires as f64
        }
    }
}
