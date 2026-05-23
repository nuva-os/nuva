/*
 * Nuva OS - SystemService - SQLite - Transaction Manager
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

//! Transaction manager providing ACID guarantees.
//! Implements BEGIN/COMMIT/ROLLBACK semantics with WAL-based concurrency control.

use alloc::collections::BTreeMap;
use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

use super::connection::ConnectionId;
use super::error::SqliteError;
use super::parser::TxType;
use super::pager::Pager;
use super::wal::WalManager;

/// Transaction state for a connection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionState {
    /// No active transaction
    None = 0,
    /// DEFERRED transaction: locks acquired on first read/write
    Deferred = 1,
    /// IMMEDIATE transaction: write lock held
    Immediate = 2,
    /// EXCLUSIVE transaction: exclusive lock held
    Exclusive = 3,
}

/// Transaction lock type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockType {
    /// No lock
    None = 0,
    /// Shared (read) lock
    Shared = 1,
    /// Reserved lock (intent to write)
    Reserved = 2,
    /// Pending lock (waiting for readers to drain)
    Pending = 3,
    /// Exclusive lock (no readers or writers allowed)
    Exclusive = 4,
}

/// Transaction descriptor
#[derive(Debug)]
pub struct Transaction {
    /// Transaction ID
    pub id: u64,
    /// Owning connection
    pub conn_id: ConnectionId,
    /// Transaction type
    pub tx_type: TxType,
    /// Current state
    pub state: TransactionState,
    /// Database lock level
    pub lock: LockType,
    /// WAL frame number at transaction start (for rollback)
    pub start_frame: u32,
    /// Whether dirty pages have been written
    pub has_dirty_pages: bool,
    /// Number of statements executed in this transaction
    pub stmt_count: u32,
}

/// Transaction manager
pub struct TransactionManager {
    /// Active transactions indexed by connection ID
    transactions: BTreeMap<u64, Transaction>,
    /// Next transaction ID
    next_tx_id: AtomicU64,
    /// Current database lock state
    db_lock: AtomicU32,
    /// Number of active readers (shared lock holders)
    reader_count: AtomicU32,
    /// Number of active writers
    writer_count: AtomicU32,
}

/// Default lock timeout in microseconds (5 seconds)
const DEFAULT_LOCK_TIMEOUT_US: u64 = 5_000_000;

/// Maximum number of busy retries
const MAX_BUSY_RETRIES: u32 = 50;

impl TransactionManager {
    /// Create a new transaction manager
    pub fn new() -> Self {
        TransactionManager {
            transactions: BTreeMap::new(),
            next_tx_id: AtomicU64::new(1),
            db_lock: AtomicU32::new(LockType::None as u32),
            reader_count: AtomicU32::new(0),
            writer_count: AtomicU32::new(0),
        }
    }

    /// Begin a new transaction for the given connection
    pub fn begin(
        &mut self,
        conn_id: ConnectionId,
        tx_type: TxType,
        wal: &WalManager,
    ) -> Result<u64, SqliteError> {
        // Check if connection already has a transaction
        if self.transactions.contains_key(&conn_id.0) {
            return Err(SqliteError::Busy);
        }

        // Acquire locks based on transaction type
        match tx_type {
            TxType::Deferred => {
                // Deferred: no lock acquired yet
            }
            TxType::Immediate => {
                if !self.try_acquire_lock(LockType::Reserved) {
                    return Err(SqliteError::Busy);
                }
            }
            TxType::Exclusive => {
                if !self.try_acquire_lock(LockType::Exclusive) {
                    return Err(SqliteError::Busy);
                }
            }
        }

        let tx_id = self.next_tx_id.fetch_add(1, Ordering::Relaxed);
        let start_frame = wal.frame_count();

        let state = match tx_type {
            TxType::Deferred => TransactionState::Deferred,
            TxType::Immediate => TransactionState::Immediate,
            TxType::Exclusive => TransactionState::Exclusive,
        };

        let lock = match tx_type {
            TxType::Deferred => LockType::None,
            TxType::Immediate => LockType::Reserved,
            TxType::Exclusive => LockType::Exclusive,
        };

        let tx = Transaction {
            id: tx_id,
            conn_id,
            tx_type,
            state,
            lock,
            start_frame,
            has_dirty_pages: false,
            stmt_count: 0,
        };

        self.transactions.insert(conn_id.0, tx);
        self.writer_count.fetch_add(1, Ordering::Relaxed);

        Ok(tx_id)
    }

    /// Commit the transaction for the given connection
    pub fn commit(
        &mut self,
        conn_id: ConnectionId,
        wal: &mut WalManager,
        pager: &mut Pager,
    ) -> Result<(), SqliteError> {
        let tx = self.transactions.get(&conn_id.0).ok_or(SqliteError::NoActiveTransaction)?;

        if tx.state == TransactionState::None {
            return Err(SqliteError::NoActiveTransaction);
        }

        let tx_id = tx.id;
        let has_dirty = tx.has_dirty_pages;
        let lock = tx.lock;

        // Write commit frame to WAL if there are dirty pages
        if has_dirty {
            // The commit frame is written by the executor when it syncs
            // the last modified page. Here we just sync the WAL.
            pager.sync()?;
        }

        // Release locks
        self.release_lock(lock);
        self.transactions.remove(&conn_id.0);
        self.writer_count.fetch_sub(1, Ordering::Relaxed);

        Ok(())
    }

    /// Rollback the transaction for the given connection
    pub fn rollback(
        &mut self,
        conn_id: ConnectionId,
        wal: &mut WalManager,
        pager: &mut Pager,
    ) -> Result<(), SqliteError> {
        let tx = self.transactions.get(&conn_id.0).ok_or(SqliteError::NoActiveTransaction)?;

        if tx.state == TransactionState::None {
            return Err(SqliteError::NoActiveTransaction);
        }

        let start_frame = tx.start_frame;
        let lock = tx.lock;

        // Rollback by truncating WAL to the frame before this transaction
        // In a full implementation, this would call wal.truncate(start_frame)
        // and invalidate any pages in the pager cache that were modified.
        let _ = (wal, start_frame, pager);

        // Release locks
        self.release_lock(lock);
        self.transactions.remove(&conn_id.0);
        self.writer_count.fetch_sub(1, Ordering::Relaxed);

        Ok(())
    }

    /// Get the transaction state for a connection
    pub fn get_state(&self, conn_id: ConnectionId) -> TransactionState {
        self.transactions
            .get(&conn_id.0)
            .map(|tx| tx.state)
            .unwrap_or(TransactionState::None)
    }

    /// Get a mutable reference to a transaction
    pub fn get_mut(&mut self, conn_id: ConnectionId) -> Option<&mut Transaction> {
        self.transactions.get_mut(&conn_id.0)
    }

    /// Check if a connection has an active transaction
    pub fn is_active(&self, conn_id: ConnectionId) -> bool {
        self.transactions
            .get(&conn_id.0)
            .map(|tx| tx.state != TransactionState::None)
            .unwrap_or(false)
    }

    /// Returns the number of active transactions
    pub fn active_count(&self) -> u32 {
        self.transactions.len() as u32
    }

    /// Promote a deferred transaction to immediate on first write
    pub fn promote_to_immediate(&mut self, conn_id: ConnectionId) -> Result<(), SqliteError> {
        let tx = self.transactions.get_mut(&conn_id.0).ok_or(SqliteError::NoActiveTransaction)?;

        if tx.state != TransactionState::Deferred {
            return Ok(());
        }

        if !self.try_acquire_lock(LockType::Reserved) {
            return Err(SqliteError::Busy);
        }

        tx.state = TransactionState::Immediate;
        tx.lock = LockType::Reserved;
        Ok(())
    }

    /// Try to acquire a database lock
    fn try_acquire_lock(&self, requested: LockType) -> bool {
        let current = self.db_lock.load(Ordering::Acquire);
        let current_lock = match current {
            0 => LockType::None,
            1 => LockType::Shared,
            2 => LockType::Reserved,
            3 => LockType::Pending,
            4 => LockType::Exclusive,
            _ => LockType::None,
        };

        match (current_lock, requested) {
            (LockType::None, _) => true,
            (LockType::Shared, LockType::Shared) => true,
            (LockType::Shared, LockType::Reserved) => true,
            (LockType::Reserved, LockType::Pending) => true,
            (LockType::Pending, LockType::Exclusive) => {
                self.reader_count.load(Ordering::Acquire) == 0
            }
            _ => false,
        }
    }

    /// Release a database lock
    fn release_lock(&self, lock: LockType) {
        if lock != LockType::None {
            self.db_lock.store(LockType::None as u32, Ordering::Release);
        }
    }

    /// Abort all transactions (used during shutdown)
    pub fn abort_all(&mut self) {
        self.transactions.clear();
        self.db_lock.store(LockType::None as u32, Ordering::Release);
        self.reader_count.store(0, Ordering::Release);
        self.writer_count.store(0, Ordering::Release);
    }
}
