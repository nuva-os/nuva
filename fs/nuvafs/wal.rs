/*
 * Nuva OS - NuvaFS WAL Log Manager
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

//! NuvaFS WAL Log Manager
//! Manages WAL transactions: begin, append, commit, rollback, and checkpoint.
//! On write failure, degrades to read-only mode instead of panicking.
//! When the log area is full, forces a checkpoint and pauses new writes.

use core::sync::atomic::{AtomicU64, AtomicU32, AtomicBool, Ordering};

use super::wal_types::{
    TransactionId, WalLsn, WalOperationType, WalRecord, WalCommitMarker,
    WalError, crc32c_compute,
};
use super::wal_appender::WalAppender;
use super::wal_stats::WalStats;

/// WAL log state
pub const WAL_STATE_CLEAN: u32 = 0;
pub const WAL_STATE_DIRTY: u32 = 1;
pub const WAL_STATE_READONLY: u32 = 2;
pub const WAL_STATE_FULL: u32 = 3;

/// Maximum concurrent transactions
pub const WAL_MAX_TRANSACTIONS: usize = 64;

/// Maximum WAL log size in bytes (1 GiB)
pub const WAL_MAX_LOG_SIZE: u64 = 1 * 1024 * 1024 * 1024;

/// Transaction state for tracking in-flight transactions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TransactionState {
    /// Transaction is active and accepting records
    Active = 0,
    /// Transaction is being committed
    Committing = 1,
    /// Transaction has been committed
    Committed = 2,
    /// Transaction has been rolled back
    Aborted = 3,
}

/// In-flight transaction tracking entry
#[derive(Debug, Clone)]
pub struct TransactionEntry {
    /// Transaction ID
    pub id: TransactionId,
    /// Operation type
    pub op_type: WalOperationType,
    /// State
    pub state: TransactionState,
    /// Number of records appended
    pub num_records: u32,
    /// Running checksum over all records
    pub running_checksum: u32,
    /// Start LSN of this transaction
    pub start_lsn: WalLsn,
}

impl TransactionEntry {
    /// Create a new transaction entry
    pub fn new(id: TransactionId, op_type: WalOperationType, start_lsn: WalLsn) -> Self {
        Self { id, op_type, state: TransactionState::Active, num_records: 0, running_checksum: 0, start_lsn }
    }
}

/// WAL log operations trait
pub trait WalLogOps {
    /// Begin a new transaction
    fn begin_transaction(&mut self, op_type: WalOperationType) -> Result<TransactionId, WalError>;
    /// Append a record to an existing transaction
    fn append_record(&mut self, txn_id: TransactionId, record: &mut WalRecord) -> Result<(), WalError>;
    /// Commit a transaction
    fn commit_transaction(&mut self, txn_id: TransactionId) -> Result<(), WalError>;
    /// Rollback a transaction
    fn rollback_transaction(&mut self, txn_id: TransactionId) -> Result<(), WalError>;
    /// Force a checkpoint
    fn force_checkpoint(&mut self) -> Result<WalLsn, WalError>;
    /// Check if the log is clean
    fn is_clean(&self) -> bool;
    /// Get the current LSN
    fn current_lsn(&self) -> WalLsn;
}

/// WAL Log Manager
pub struct WalLogManager {
    /// Current LSN (monotonically increasing)
    current_lsn: AtomicU64,
    /// Next transaction ID (monotonically increasing)
    next_txn_id: AtomicU64,
    /// WAL state
    state: AtomicU32,
    /// Whether the WAL is in read-only mode (degraded)
    read_only: AtomicBool,
    /// Total bytes written to the log
    bytes_written: AtomicU64,
    /// In-flight transaction table
    transactions: [Option<TransactionEntry>; WAL_MAX_TRANSACTIONS],
    /// Number of active transactions
    num_active: u32,
    /// WAL appender for sequential writes
    appender: WalAppender,
    /// Statistics
    stats: WalStats,
    /// Checkpoint LSN
    checkpoint_lsn: AtomicU64,
}

impl WalLogManager {
    /// Create a new WalLogManager
    pub const fn new() -> Self {
        Self {
            current_lsn: AtomicU64::new(1),
            next_txn_id: AtomicU64::new(1),
            state: AtomicU32::new(WAL_STATE_CLEAN),
            read_only: AtomicBool::new(false),
            bytes_written: AtomicU64::new(0),
            transactions: [None; WAL_MAX_TRANSACTIONS],
            num_active: 0,
            appender: WalAppender::new(),
            stats: WalStats::new(),
            checkpoint_lsn: AtomicU64::new(0),
        }
    }

    /// Initialize the WAL log manager
    pub fn init(&mut self) {
        self.state.store(WAL_STATE_CLEAN, Ordering::Relaxed);
        self.read_only.store(false, Ordering::Relaxed);
        crate::log_info!("WAL log manager initialized");
    }

    /// Find a transaction slot by ID
    fn find_transaction(&self, txn_id: TransactionId) -> Option<usize> {
        for i in 0..WAL_MAX_TRANSACTIONS {
            if let Some(ref entry) = self.transactions[i] {
                if entry.id == txn_id { return Some(i); }
            }
        }
        None
    }

    /// Find a free transaction slot
    fn find_free_slot(&self) -> Option<usize> {
        for i in 0..WAL_MAX_TRANSACTIONS {
            if self.transactions[i].is_none() { return Some(i); }
        }
        None
    }

    /// Check if the log area is full
    fn is_log_full(&self) -> bool {
        self.bytes_written.load(Ordering::Relaxed) >= WAL_MAX_LOG_SIZE
    }

    /// Get the WAL stats reference
    pub fn stats(&self) -> &WalStats { &self.stats }

    /// Check if WAL is in read-only (degraded) mode
    pub fn is_read_only(&self) -> bool { self.read_only.load(Ordering::Relaxed) }

    /// Get the checkpoint LSN
    pub fn checkpoint_lsn(&self) -> WalLsn { WalLsn::new(self.checkpoint_lsn.load(Ordering::Relaxed)) }
}

impl WalLogOps for WalLogManager {
    fn begin_transaction(&mut self, op_type: WalOperationType) -> Result<TransactionId, WalError> {
        if self.read_only.load(Ordering::Relaxed) { return Err(WalError::ReadOnly); }
        if self.is_log_full() {
            if self.force_checkpoint().is_err() {
                self.state.store(WAL_STATE_FULL, Ordering::Relaxed);
                return Err(WalError::LogFull);
            }
            if self.is_log_full() { return Err(WalError::LogFull); }
        }
        let slot = self.find_free_slot().ok_or(WalError::InvalidState)?;
        let txn_id = TransactionId::new(self.next_txn_id.fetch_add(1, Ordering::Relaxed));
        let lsn = WalLsn::new(self.current_lsn.fetch_add(1, Ordering::Relaxed));
        let entry = TransactionEntry::new(txn_id, op_type, lsn);
        self.transactions[slot] = Some(entry);
        self.num_active += 1;
        self.state.store(WAL_STATE_DIRTY, Ordering::Relaxed);
        self.stats.inc_transactions_started();
        Ok(txn_id)
    }

    fn append_record(&mut self, txn_id: TransactionId, record: &mut WalRecord) -> Result<(), WalError> {
        if self.read_only.load(Ordering::Relaxed) { return Err(WalError::ReadOnly); }
        let slot = self.find_transaction(txn_id).ok_or(WalError::TransactionNotFound)?;
        {
            let entry = self.transactions[slot].as_ref().unwrap();
            if entry.state != TransactionState::Active { return Err(WalError::InvalidState); }
        }
        let lsn = WalLsn::new(self.current_lsn.fetch_add(1, Ordering::Relaxed));
        record.transaction_id = txn_id;
        record.lsn = lsn;
        record.compute_checksum();
        if self.appender.batch_append_record(record).is_err() {
            self.read_only.store(true, Ordering::Relaxed);
            self.state.store(WAL_STATE_READONLY, Ordering::Relaxed);
            return Err(WalError::IOError);
        }
        if let Some(ref mut entry) = self.transactions[slot] {
            entry.num_records += 1;
            let rec_bytes = unsafe {
                core::slice::from_raw_parts(record as *const WalRecord as *const u8, core::mem::size_of::<WalRecord>())
            };
            entry.running_checksum ^= crc32c_compute(rec_bytes);
        }
        let sz = core::mem::size_of::<WalRecord>() as u64;
        self.bytes_written.fetch_add(sz, Ordering::Relaxed);
        self.stats.add_bytes_written(sz);
        self.stats.inc_records_appended();
        Ok(())
    }

    fn commit_transaction(&mut self, txn_id: TransactionId) -> Result<(), WalError> {
        let slot = self.find_transaction(txn_id).ok_or(WalError::TransactionNotFound)?;
        let (num_records, running_checksum, timestamp) = {
            let entry = self.transactions[slot].as_mut().unwrap();
            if entry.state != TransactionState::Active { return Err(WalError::InvalidState); }
            entry.state = TransactionState::Committing;
            (entry.num_records, entry.running_checksum, self.current_lsn.load(Ordering::Relaxed))
        };
        let marker = WalCommitMarker::new(txn_id, num_records, running_checksum, timestamp);
        if self.appender.batch_append_commit(&marker).is_err() {
            self.read_only.store(true, Ordering::Relaxed);
            self.state.store(WAL_STATE_READONLY, Ordering::Relaxed);
            if let Some(ref mut e) = self.transactions[slot] { e.state = TransactionState::Active; }
            return Err(WalError::IOError);
        }
        let flushed = self.appender.batch_flush();
        self.bytes_written.fetch_add(flushed as u64, Ordering::Relaxed);
        self.transactions[slot] = None;
        self.num_active = self.num_active.saturating_sub(1);
        if self.num_active == 0 { self.state.store(WAL_STATE_CLEAN, Ordering::Relaxed); }
        self.stats.inc_transactions_committed();
        Ok(())
    }

    fn rollback_transaction(&mut self, txn_id: TransactionId) -> Result<(), WalError> {
        let slot = self.find_transaction(txn_id).ok_or(WalError::TransactionNotFound)?;
        self.transactions[slot] = None;
        self.num_active = self.num_active.saturating_sub(1);
        if self.num_active == 0 { self.state.store(WAL_STATE_CLEAN, Ordering::Relaxed); }
        self.stats.inc_transactions_rolled_back();
        Ok(())
    }

    fn force_checkpoint(&mut self) -> Result<WalLsn, WalError> {
        for i in 0..WAL_MAX_TRANSACTIONS { self.transactions[i] = None; }
        self.num_active = 0;
        let flushed = self.appender.batch_flush();
        self.bytes_written.fetch_add(flushed as u64, Ordering::Relaxed);
        let current = self.current_lsn.load(Ordering::Relaxed);
        self.checkpoint_lsn.store(current, Ordering::Relaxed);
        self.bytes_written.store(0, Ordering::Relaxed);
        self.state.store(WAL_STATE_CLEAN, Ordering::Relaxed);
        self.stats.inc_checkpoints_executed();
        Ok(WalLsn::new(current))
    }

    fn is_clean(&self) -> bool { self.state.load(Ordering::Relaxed) == WAL_STATE_CLEAN }
    fn current_lsn(&self) -> WalLsn { WalLsn::new(self.current_lsn.load(Ordering::Relaxed)) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_wal_log_manager_new() {
        let mgr = WalLogManager::new();
        assert!(mgr.is_clean());
        assert!(!mgr.is_read_only());
    }
    #[test]
    fn test_begin_commit() {
        let mut mgr = WalLogManager::new();
        mgr.init();
        let txn_id = mgr.begin_transaction(WalOperationType::Write).unwrap();
        assert!(!mgr.is_clean());
        let mut record = WalRecord::new(txn_id, WalLsn::new(0), WalOperationType::Write, 0x1000, 0);
        assert!(mgr.append_record(txn_id, &mut record).is_ok());
        assert!(mgr.commit_transaction(txn_id).is_ok());
        assert!(mgr.is_clean());
    }
    #[test]
    fn test_rollback() {
        let mut mgr = WalLogManager::new();
        mgr.init();
        let txn_id = mgr.begin_transaction(WalOperationType::Create).unwrap();
        assert!(mgr.rollback_transaction(txn_id).is_ok());
        assert!(mgr.is_clean());
    }
    #[test]
    fn test_txn_not_found() {
        let mut mgr = WalLogManager::new();
        mgr.init();
        assert_eq!(mgr.commit_transaction(TransactionId::new(999)), Err(WalError::TransactionNotFound));
    }
    #[test]
    fn test_force_checkpoint() {
        let mut mgr = WalLogManager::new();
        mgr.init();
        let txn_id = mgr.begin_transaction(WalOperationType::Write).unwrap();
        let mut r = WalRecord::new(txn_id, WalLsn::new(0), WalOperationType::Write, 0, 0);
        mgr.append_record(txn_id, &mut r).unwrap();
        let lsn = mgr.force_checkpoint().unwrap();
        assert!(mgr.is_clean());
        assert!(lsn.as_u64() > 0);
    }
    #[test]
    fn test_multiple_txns() {
        let mut mgr = WalLogManager::new();
        mgr.init();
        let t1 = mgr.begin_transaction(WalOperationType::Write).unwrap();
        let t2 = mgr.begin_transaction(WalOperationType::Create).unwrap();
        assert_ne!(t1, t2);
        let mut r = WalRecord::new(t1, WalLsn::new(0), WalOperationType::Write, 0, 0);
        mgr.append_record(t1, &mut r).unwrap();
        assert!(mgr.commit_transaction(t1).is_ok());
        assert!(!mgr.is_clean());
        assert!(mgr.commit_transaction(t2).is_ok());
        assert!(mgr.is_clean());
    }
}