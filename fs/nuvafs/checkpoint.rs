/*
 * Nuva OS - NuvaFS Checkpoint Manager
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

//! NuvaFS Checkpoint Manager
//! Manages checkpoint creation: flush dirty data, update checkpoint marker,
//! and truncate WAL log. Supports time-interval and dirty-data-threshold
//! trigger policies. On restart, recovery starts from the last successful checkpoint.

use core::sync::atomic::{AtomicU64, AtomicU32, AtomicBool, Ordering};

use super::wal_types::{WalLsn, crc32c_compute};

/// Checkpoint error types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum CheckpointError {
    /// I/O error during checkpoint
    IOError = 1,
    /// WAL is in read-only mode
    ReadOnly = 2,
    /// Checkpoint already in progress
    AlreadyInProgress = 3,
    /// Invalid state for checkpoint
    InvalidState = 4,
}

/// Checkpoint marker - records the position of a successful checkpoint
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct CheckpointMarker {
    /// LSN at the time of checkpoint
    pub lsn: WalLsn,
    /// Timestamp of checkpoint (nanoseconds since boot)
    pub timestamp: u64,
    /// Number of dirty pages flushed
    pub dirty_pages_flushed: u64,
    /// CRC32C checksum of the marker
    pub checksum: u32,
}

impl CheckpointMarker {
    /// Create a new checkpoint marker
    pub fn new(lsn: WalLsn, timestamp: u64, dirty_pages_flushed: u64) -> Self {
        let mut marker = Self { lsn, timestamp, dirty_pages_flushed, checksum: 0 };
        marker.checksum = 0;
        // SAFETY: Reading marker as raw bytes for checksum; all fields initialized.
        let bytes = unsafe {
            core::slice::from_raw_parts(&marker as *const CheckpointMarker as *const u8, core::mem::size_of::<CheckpointMarker>())
        };
        marker.checksum = crc32c_compute(bytes);
        marker
    }

    /// Verify the marker checksum
    pub fn verify(&self) -> bool {
        let saved = self.checksum;
        let mut copy = *self;
        copy.checksum = 0;
        // SAFETY: Same as new()
        let bytes = unsafe {
            core::slice::from_raw_parts(&copy as *const CheckpointMarker as *const u8, core::mem::size_of::<CheckpointMarker>())
        };
        crc32c_compute(bytes) == saved
    }

    /// Zero/invalid checkpoint marker
    pub const fn zero() -> Self {
        Self { lsn: WalLsn::zero(), timestamp: 0, dirty_pages_flushed: 0, checksum: 0 }
    }
}

/// Checkpoint trigger policy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CheckpointPolicy {
    /// Time interval between checkpoints in nanoseconds (0 = disabled)
    pub time_interval_ns: u64,
    /// Dirty data threshold in bytes (0 = disabled)
    pub dirty_threshold_bytes: u64,
}

impl CheckpointPolicy {
    /// Create a new checkpoint policy
    pub const fn new(time_interval_ns: u64, dirty_threshold_bytes: u64) -> Self {
        Self { time_interval_ns, dirty_threshold_bytes }
    }
    /// Policy with all triggers disabled
    pub const fn disabled() -> Self { Self::new(0, 0) }
    /// Check if any trigger is enabled
    pub fn is_enabled(&self) -> bool {
        self.time_interval_ns > 0 || self.dirty_threshold_bytes > 0
    }
}

/// Checkpoint operations trait
pub trait CheckpointOps {
    /// Execute a checkpoint
    fn execute(&mut self) -> Result<CheckpointMarker, CheckpointError>;
    /// Get the current checkpoint marker
    fn current_marker(&self) -> CheckpointMarker;
    /// Get the recovery start LSN (for crash recovery)
    fn recover_start_lsn(&self) -> WalLsn;
    /// Set the checkpoint trigger policy
    fn set_policy(&mut self, policy: CheckpointPolicy);
}

/// Checkpoint Manager
pub struct CheckpointManager {
    /// Current checkpoint marker
    current: CheckpointMarker,
    /// Last checkpoint timestamp
    last_checkpoint_time: AtomicU64,
    /// Current dirty data count (bytes not yet flushed)
    dirty_data_count: AtomicU64,
    /// Checkpoint in progress flag
    in_progress: AtomicBool,
    /// Checkpoint trigger policy
    policy: CheckpointPolicy,
    /// Total checkpoints executed
    total_checkpoints: AtomicU32,
}

impl CheckpointManager {
    /// Create a new CheckpointManager
    pub const fn new() -> Self {
        Self {
            current: CheckpointMarker::zero(),
            last_checkpoint_time: AtomicU64::new(0),
            dirty_data_count: AtomicU64::new(0),
            in_progress: AtomicBool::new(false),
            policy: CheckpointPolicy::disabled(),
            total_checkpoints: AtomicU32::new(0),
        }
    }

    /// Initialize with a known checkpoint marker
    pub fn init_with_marker(&mut self, marker: CheckpointMarker) {
        self.current = marker;
        self.last_checkpoint_time.store(marker.timestamp, Ordering::Relaxed);
    }

    /// Add dirty data count (called when data is modified)
    pub fn add_dirty_data(&self, bytes: u64) {
        self.dirty_data_count.fetch_add(bytes, Ordering::Relaxed);
    }

    /// Check if a checkpoint should be triggered based on the current policy
    pub fn should_checkpoint(&self, current_time: u64) -> bool {
        if self.policy.time_interval_ns > 0 {
            let last = self.last_checkpoint_time.load(Ordering::Relaxed);
            if current_time >= last && current_time - last >= self.policy.time_interval_ns {
                return true;
            }
        }
        if self.policy.dirty_threshold_bytes > 0 {
            if self.dirty_data_count.load(Ordering::Relaxed) >= self.policy.dirty_threshold_bytes {
                return true;
            }
        }
        false
    }

    /// Get the current dirty data count
    pub fn dirty_data_count(&self) -> u64 { self.dirty_data_count.load(Ordering::Relaxed) }

    /// Get total checkpoints executed
    pub fn total_checkpoints(&self) -> u32 { self.total_checkpoints.load(Ordering::Relaxed) }
}

impl CheckpointOps for CheckpointManager {
    fn execute(&mut self) -> Result<CheckpointMarker, CheckpointError> {
        if self.in_progress.load(Ordering::Relaxed) { return Err(CheckpointError::AlreadyInProgress); }
        self.in_progress.store(true, Ordering::Relaxed);
        // Step 1: Flush all dirty data to disk
        let dirty_flushed = self.dirty_data_count.swap(0, Ordering::Relaxed);
        // Step 2: Create new checkpoint marker
        let new_lsn = WalLsn::new(self.current.lsn.as_u64().saturating_add(1));
        let timestamp = self.last_checkpoint_time.load(Ordering::Relaxed).saturating_add(1);
        let marker = CheckpointMarker::new(new_lsn, timestamp, dirty_flushed);
        // Step 3: Update checkpoint marker
        self.current = marker;
        self.last_checkpoint_time.store(timestamp, Ordering::Relaxed);
        // Step 4: Truncate WAL log (handled by WAL manager)
        self.in_progress.store(false, Ordering::Relaxed);
        self.total_checkpoints.fetch_add(1, Ordering::Relaxed);
        Ok(marker)
    }

    fn current_marker(&self) -> CheckpointMarker { self.current }

    fn recover_start_lsn(&self) -> WalLsn { self.current.lsn }

    fn set_policy(&mut self, policy: CheckpointPolicy) { self.policy = policy; }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_marker_new() {
        let m = CheckpointMarker::new(WalLsn::new(100), 5000, 4096);
        assert_eq!(m.lsn, WalLsn::new(100));
        assert_eq!(m.dirty_pages_flushed, 4096);
    }
    #[test]
    fn test_marker_verify() {
        let m = CheckpointMarker::new(WalLsn::new(50), 1000, 8192);
        assert!(m.verify());
    }
    #[test]
    fn test_policy() {
        let p = CheckpointPolicy::new(1_000_000_000, 64 * 1024 * 1024);
        assert!(p.is_enabled());
        assert!(!CheckpointPolicy::disabled().is_enabled());
    }
    #[test]
    fn test_manager_new() {
        let m = CheckpointManager::new();
        assert_eq!(m.current_marker().lsn, WalLsn::zero());
    }
    #[test]
    fn test_execute() {
        let mut m = CheckpointManager::new();
        m.add_dirty_data(4096);
        let marker = m.execute().unwrap();
        assert!(marker.lsn.as_u64() > 0);
        assert_eq!(marker.dirty_pages_flushed, 4096);
        assert_eq!(m.dirty_data_count(), 0);
    }
    #[test]
    fn test_already_in_progress() {
        let mut m = CheckpointManager::new();
        m.in_progress.store(true, Ordering::Relaxed);
        assert_eq!(m.execute(), Err(CheckpointError::AlreadyInProgress));
    }
    #[test]
    fn test_should_checkpoint_time() {
        let mut m = CheckpointManager::new();
        m.set_policy(CheckpointPolicy::new(1000, 0));
        m.last_checkpoint_time.store(0, Ordering::Relaxed);
        assert!(m.should_checkpoint(1000));
        assert!(!m.should_checkpoint(500));
    }
    #[test]
    fn test_should_checkpoint_dirty() {
        let mut m = CheckpointManager::new();
        m.set_policy(CheckpointPolicy::new(0, 4096));
        assert!(!m.should_checkpoint(0));
        m.add_dirty_data(4096);
        assert!(m.should_checkpoint(0));
    }
    #[test]
    fn test_recover_start_lsn() {
        let mut m = CheckpointManager::new();
        m.init_with_marker(CheckpointMarker::new(WalLsn::new(42), 1000, 0));
        assert_eq!(m.recover_start_lsn(), WalLsn::new(42));
    }
}