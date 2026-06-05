/*
 * Nuva OS - NuvaFS WAL Stats
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
//! NuvaFS WAL Statistics
//! Lock-free counters using AtomicU64 for WAL performance monitoring.
use core::sync::atomic::{AtomicU64, Ordering};
/// WAL statistics - all counters are lock-free using AtomicU64
#[derive(Debug)]
pub struct WalStats {
    pub transactions_started: AtomicU64,
    pub transactions_committed: AtomicU64,
    pub transactions_rolled_back: AtomicU64,
    pub records_appended: AtomicU64,
    pub checkpoints_executed: AtomicU64,
    pub bytes_written: AtomicU64,
    pub recovery_count: AtomicU64,
}
impl WalStats {
    pub const fn new() -> Self {
        Self {
            transactions_started: AtomicU64::new(0),
            transactions_committed: AtomicU64::new(0),
            transactions_rolled_back: AtomicU64::new(0),
            records_appended: AtomicU64::new(0),
            checkpoints_executed: AtomicU64::new(0),
            bytes_written: AtomicU64::new(0),
            recovery_count: AtomicU64::new(0),
        }
    }
    pub fn reset(&self) {
        self.transactions_started.store(0, Ordering::Relaxed);
        self.transactions_committed.store(0, Ordering::Relaxed);
        self.transactions_rolled_back.store(0, Ordering::Relaxed);
        self.records_appended.store(0, Ordering::Relaxed);
        self.checkpoints_executed.store(0, Ordering::Relaxed);
        self.bytes_written.store(0, Ordering::Relaxed);
        self.recovery_count.store(0, Ordering::Relaxed);
    }
    pub fn inc_transactions_started(&self) {
        self.transactions_started.fetch_add(1, Ordering::Relaxed);
    }
    pub fn inc_transactions_committed(&self) {
        self.transactions_committed.fetch_add(1, Ordering::Relaxed);
    }
    pub fn inc_transactions_rolled_back(&self) {
        self.transactions_rolled_back.fetch_add(1, Ordering::Relaxed);
    }
    pub fn inc_records_appended(&self) {
        self.records_appended.fetch_add(1, Ordering::Relaxed);
    }
    pub fn inc_checkpoints_executed(&self) {
        self.checkpoints_executed.fetch_add(1, Ordering::Relaxed);
    }
    pub fn add_bytes_written(&self, bytes: u64) {
        self.bytes_written.fetch_add(bytes, Ordering::Relaxed);
    }
    pub fn inc_recovery_count(&self) {
        self.recovery_count.fetch_add(1, Ordering::Relaxed);
    }
    pub fn snapshot(&self) -> WalStatsSnapshot {
        WalStatsSnapshot {
            transactions_started: self.transactions_started.load(Ordering::Relaxed),
            transactions_committed: self.transactions_committed.load(Ordering::Relaxed),
            transactions_rolled_back: self.transactions_rolled_back.load(Ordering::Relaxed),
            records_appended: self.records_appended.load(Ordering::Relaxed),
            checkpoints_executed: self.checkpoints_executed.load(Ordering::Relaxed),
            bytes_written: self.bytes_written.load(Ordering::Relaxed),
            recovery_count: self.recovery_count.load(Ordering::Relaxed),
        }
    }
}
/// Snapshot of WAL statistics (all values captured at a point in time)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WalStatsSnapshot {
    pub transactions_started: u64,
    pub transactions_committed: u64,
    pub transactions_rolled_back: u64,
    pub records_appended: u64,
    pub checkpoints_executed: u64,
    pub bytes_written: u64,
    pub recovery_count: u64,
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_wal_stats_new() {
        let stats = WalStats::new();
        let snap = stats.snapshot();
        assert_eq!(snap.transactions_started, 0);
        assert_eq!(snap.transactions_committed, 0);
        assert_eq!(snap.records_appended, 0);
        assert_eq!(snap.bytes_written, 0);
    }
    #[test]
    fn test_wal_stats_increment() {
        let stats = WalStats::new();
        stats.inc_transactions_started();
        stats.inc_transactions_started();
        stats.inc_transactions_committed();
        stats.inc_records_appended();
        stats.add_bytes_written(4096);
        let snap = stats.snapshot();
        assert_eq!(snap.transactions_started, 2);
        assert_eq!(snap.transactions_committed, 1);
        assert_eq!(snap.records_appended, 1);
        assert_eq!(snap.bytes_written, 4096);
    }
    #[test]
    fn test_wal_stats_reset() {
        let stats = WalStats::new();
        stats.inc_transactions_started();
        stats.inc_records_appended();
        stats.reset();
        let snap = stats.snapshot();
        assert_eq!(snap.transactions_started, 0);
        assert_eq!(snap.records_appended, 0);
    }
}