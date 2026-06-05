/*
 * Nuva OS - Kernel - Fs - Wal
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
/*
 * Nuva OS - Kernel - Write-Ahead Log (WAL)
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Write-ahead logging for NuvaFS metadata consistency.
 * Ensures crash recovery by logging modifications
 * before applying them to the filesystem.
 */

use core::sync::atomic::{AtomicU64, AtomicU32, Ordering};

use crate::kernel::error::{KernelError, KernelResult};

/// Maximum WAL record size (4KB)
pub const MAX_WAL_RECORD_SIZE: usize = 4096;

/// Maximum WAL entries before checkpoint
pub const MAX_WAL_ENTRIES: u32 = 1024;

/// WAL record type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum WalRecordType {
    /// Inode modification
    InodeModify = 1,
    /// Block allocation
    BlockAlloc = 2,
    /// Block deallocation
    BlockFree = 3,
    /// Directory entry add
    DirEntryAdd = 4,
    /// Directory entry remove
    DirEntryRemove = 5,
    /// Superblock update
    SuperblockUpdate = 6,
    /// Checkpoint marker
    Checkpoint = 7,
    /// Commit marker
    Commit = 8,
}

/// WAL record header
#[derive(Clone, Debug)]
#[repr(C)]
pub struct WalRecordHeader {
    /// Transaction ID
    pub txn_id: u64,
    /// Record sequence within transaction
    pub seq: u32,
    /// Record type
    pub record_type: WalRecordType,
    /// Data length
    pub data_len: u32,
    /// CRC32 checksum of data
    pub checksum: u32,
}

impl WalRecordHeader {
    /// Size of header in bytes
    pub const SIZE: usize = 8 + 4 + 1 + 4 + 4;
}

/// WAL record
#[derive(Clone, Debug)]
pub struct WalRecord {
    /// Record header
    pub header: WalRecordHeader,
    /// Record data (e.g., inode bytes, block number)
    pub data: [u8; MAX_WAL_RECORD_SIZE],
}

impl WalRecord {
    /// Create a new WAL record
    pub fn new(txn_id: u64, seq: u32, record_type: WalRecordType, data: &[u8]) -> KernelResult<Self> {
        if data.len() > MAX_WAL_RECORD_SIZE {
            return Err(KernelError::InvalidArgument);
        }

        let mut record = WalRecord {
            header: WalRecordHeader {
                txn_id,
                seq,
                record_type,
                data_len: data.len() as u32,
                checksum: Self::compute_checksum(data),
            },
            data: [0u8; MAX_WAL_RECORD_SIZE],
        };
        record.data[..data.len()].copy_from_slice(data);
        Ok(record)
    }

    /// Verify record integrity
    pub fn verify(&self) -> bool {
        if self.header.data_len as usize > MAX_WAL_RECORD_SIZE {
            return false;
        }
        let computed = Self::compute_checksum(&self.data[..self.header.data_len as usize]);
        computed == self.header.checksum
    }

    /// Compute CRC32 checksum (simplified FNV-1a for no_std)
    fn compute_checksum(data: &[u8]) -> u32 {
        let mut hash: u32 = 0x811c9dc5;
        for &byte in data {
            hash ^= byte as u32;
            hash = hash.wrapping_mul(0x01000193);
        }
        hash
    }
}

/// WAL transaction state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum WalTxnState {
    /// Transaction is active
    Active = 0,
    /// Transaction is committed
    Committed = 1,
    /// Transaction is aborted
    Aborted = 2,
}

/// WAL: Write-Ahead Log manager
///
/// Provides transaction logging for metadata operations.
/// On crash, uncommitted transactions are rolled back
/// during recovery.
pub struct WalManager {
    /// Next transaction ID
    next_txn_id: AtomicU64,
    /// Current active transaction count
    active_txns: AtomicU32,
    /// Total records written
    total_records: AtomicU64,
    /// Total transactions committed
    total_committed: AtomicU64,
    /// Total transactions aborted
    total_aborted: AtomicU64,
}

impl WalManager {
    /// Create a new WAL manager
    pub const fn new() -> Self {
        WalManager {
            next_txn_id: AtomicU64::new(1),
            active_txns: AtomicU32::new(0),
            total_records: AtomicU64::new(0),
            total_committed: AtomicU64::new(0),
            total_aborted: AtomicU64::new(0),
        }
    }

    /// Begin a new transaction
    pub fn begin_txn(&self) -> u64 {
        let txn_id = self.next_txn_id.fetch_add(1, Ordering::AcqRel);
        self.active_txns.fetch_add(1, Ordering::Relaxed);
        txn_id
    }

    /// Write a record to the WAL
    pub fn write_record(&self, txn_id: u64, seq: u32, record_type: WalRecordType, data: &[u8]) -> KernelResult<()> {
        let record = WalRecord::new(txn_id, seq, record_type, data)?;
        // TODO: Write record to persistent storage
        let _ = record;
        self.total_records.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    /// Commit a transaction
    pub fn commit_txn(&self, txn_id: u64) -> KernelResult<()> {
        // TODO: Write commit record, flush to disk
        let _ = txn_id;
        self.active_txns.fetch_sub(1, Ordering::Relaxed);
        self.total_committed.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    /// Abort a transaction
    pub fn abort_txn(&self, txn_id: u64) -> KernelResult<()> {
        // TODO: Write abort record
        let _ = txn_id;
        self.active_txns.fetch_sub(1, Ordering::Relaxed);
        self.total_aborted.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    /// Recover from crash by replaying committed transactions
    pub fn recover(&self) -> KernelResult<u64> {
        // TODO: Read WAL from disk, replay committed, discard uncommitted
        Ok(0)
    }

    /// Force checkpoint (flush all committed records)
    pub fn checkpoint(&self) -> KernelResult<()> {
        // TODO: Flush all committed transactions to main filesystem
        Ok(())
    }

    /// Get statistics
    pub fn stats(&self) -> (u64, u32, u64, u64, u64) {
        (
            self.next_txn_id.load(Ordering::Acquire) - 1,
            self.active_txns.load(Ordering::Acquire),
            self.total_records.load(Ordering::Acquire),
            self.total_committed.load(Ordering::Acquire),
            self.total_aborted.load(Ordering::Acquire),
        )
    }
}

/// Global WAL manager
static WAL_MANAGER: core::sync::OnceLock<WalManager> = core::sync::OnceLock::new();

/// Get global WAL manager
pub fn get_wal_manager() -> &'static WalManager {
    WAL_MANAGER.get_or_init(WalManager::new)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wal_record_create() {
        let data = b"test_data";
        let record = WalRecord::new(1, 0, WalRecordType::InodeModify, data);
        assert!(record.is_ok());
        assert!(record.unwrap().verify());
    }

    #[test]
    fn test_wal_record_too_large() {
        let data = [0u8; MAX_WAL_RECORD_SIZE + 1];
        let record = WalRecord::new(1, 0, WalRecordType::InodeModify, &data);
        assert!(record.is_err());
    }

    #[test]
    fn test_wal_txn() {
        let mgr = WalManager::new();
        let txn_id = mgr.begin_txn();
        assert!(mgr.write_record(txn_id, 0, WalRecordType::InodeModify, b"data").is_ok());
        assert!(mgr.commit_txn(txn_id).is_ok());
        let (_, active, _, committed, _) = mgr.stats();
        assert_eq!(active, 0);
        assert_eq!(committed, 1);
    }

    #[test]
    fn test_checksum_integrity() {
        let data1 = b"hello";
        let data2 = b"world";
        let c1 = WalRecord::compute_checksum(data1);
        let c2 = WalRecord::compute_checksum(data2);
        assert_ne!(c1, c2);
    }
}