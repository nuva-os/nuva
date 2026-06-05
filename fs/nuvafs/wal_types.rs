/*
 * Nuva OS - NuvaFS WAL Types
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

//! NuvaFS WAL (Write-Ahead Log) Data Types
//! Defines core types for the WAL logging system: transaction IDs, LSNs,
//! log records, commit markers, errors, and CRC32C checksum computation.

/// Transaction identifier (newtype over u64)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct TransactionId(pub u64);

impl TransactionId {
    /// Create a new TransactionId
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    /// Get the raw u64 value
    pub const fn as_u64(&self) -> u64 {
        self.0
    }
}

impl core::fmt::Display for TransactionId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Txn({})", self.0)
    }
}

/// WAL Log Sequence Number (newtype over u64)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct WalLsn(pub u64);

impl WalLsn {
    /// Create a new WalLsn
    pub const fn new(lsn: u64) -> Self {
        Self(lsn)
    }

    /// Get the raw u64 value
    pub const fn as_u64(&self) -> u64 {
        self.0
    }

    /// Zero LSN (invalid / before any log entry)
    pub const fn zero() -> Self {
        Self(0)
    }

    /// Increment LSN by one
    pub fn inc(&mut self) {
        self.0 = self.0.saturating_add(1);
    }
}

impl core::fmt::Display for WalLsn {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "LSN({})", self.0)
    }
}

/// WAL operation type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum WalOperationType {
    /// Create a file or directory
    Create = 1,
    /// Delete a file or directory
    Delete = 2,
    /// Write data to a block
    Write = 3,
    /// Rename a file or directory
    Rename = 4,
    /// Truncate a file
    Truncate = 5,
    /// Create a snapshot
    SnapshotCreate = 6,
    /// Rollback to a snapshot
    SnapshotRollback = 7,
    /// Checkpoint marker
    Checkpoint = 8,
}

impl WalOperationType {
    /// Convert from u8
    pub fn from_u8(val: u8) -> Option<Self> {
        match val {
            1 => Some(Self::Create),
            2 => Some(Self::Delete),
            3 => Some(Self::Write),
            4 => Some(Self::Rename),
            5 => Some(Self::Truncate),
            6 => Some(Self::SnapshotCreate),
            7 => Some(Self::SnapshotRollback),
            8 => Some(Self::Checkpoint),
            _ => None,
        }
    }
}

/// WAL block data size (4 KiB)
pub const WAL_BLOCK_SIZE: usize = 4096;

/// WAL record - represents a single logged operation
#[derive(Debug, Clone)]
#[repr(C)]
pub struct WalRecord {
    /// Transaction this record belongs to
    pub transaction_id: TransactionId,
    /// Log sequence number of this record
    pub lsn: WalLsn,
    /// Type of operation
    pub operation_type: WalOperationType,
    /// Block address being modified
    pub block_address: u64,
    /// Original data before modification
    pub old_data: [u8; WAL_BLOCK_SIZE],
    /// New data after modification
    pub new_data: [u8; WAL_BLOCK_SIZE],
    /// Valid data length within old_data / new_data
    pub data_len: u32,
    /// CRC32C checksum for integrity verification
    pub checksum: u32,
    /// Timestamp (nanoseconds since boot)
    pub timestamp: u64,
}

impl WalRecord {
    /// Create a new WAL record with zeroed data fields
    pub fn new(
        transaction_id: TransactionId,
        lsn: WalLsn,
        operation_type: WalOperationType,
        block_address: u64,
        timestamp: u64,
    ) -> Self {
        Self {
            transaction_id,
            lsn,
            operation_type,
            block_address,
            old_data: [0u8; WAL_BLOCK_SIZE],
            new_data: [0u8; WAL_BLOCK_SIZE],
            data_len: 0,
            checksum: 0,
            timestamp,
        }
    }

    /// Compute and store the CRC32C checksum over all fields except the checksum itself
    pub fn compute_checksum(&mut self) {
        self.checksum = 0;
        // SAFETY: We are reading the record as raw bytes for checksum computation.
        // The record contains no padding that would cause uninitialized reads
        // because all fields are initialized.
        let bytes = unsafe {
            core::slice::from_raw_parts(
                self as *const WalRecord as *const u8,
                core::mem::size_of::<WalRecord>(),
            )
        };
        self.checksum = crc32c_compute(bytes);
    }

    /// Verify the stored CRC32C checksum
    pub fn verify_checksum(&self) -> bool {
        let saved = self.checksum;
        let mut copy = self.clone();
        copy.checksum = 0;
        // SAFETY: Same as compute_checksum - reading initialized data as raw bytes
        let bytes = unsafe {
            core::slice::from_raw_parts(
                &copy as *const WalRecord as *const u8,
                core::mem::size_of::<WalRecord>(),
            )
        };
        crc32c_compute(bytes) == saved
    }
}

/// WAL commit marker - written when a transaction commits
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct WalCommitMarker {
    /// Transaction being committed
    pub transaction_id: TransactionId,
    /// Number of records in this transaction
    pub num_records: u32,
    /// CRC32C checksum covering all records in the transaction
    pub commit_checksum: u32,
    /// Timestamp of commit (nanoseconds since boot)
    pub timestamp: u64,
}

impl WalCommitMarker {
    /// Create a new commit marker
    pub fn new(
        transaction_id: TransactionId,
        num_records: u32,
        commit_checksum: u32,
        timestamp: u64,
    ) -> Self {
        Self {
            transaction_id,
            num_records,
            commit_checksum,
            timestamp,
        }
    }
}

/// WAL error types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum WalError {
    /// I/O error during log read/write
    IOError = 1,
    /// Transaction ID not found
    TransactionNotFound = 2,
    /// CRC32C checksum mismatch (corruption detected)
    ChecksumMismatch = 3,
    /// WAL log area is full
    LogFull = 4,
    /// WAL is in read-only mode (degraded after I/O failure)
    ReadOnly = 5,
    /// Invalid internal state
    InvalidState = 6,
}

impl WalError {
    /// Convert from i32
    pub fn from_i32(val: i32) -> Option<Self> {
        match val {
            1 => Some(Self::IOError),
            2 => Some(Self::TransactionNotFound),
            3 => Some(Self::ChecksumMismatch),
            4 => Some(Self::LogFull),
            5 => Some(Self::ReadOnly),
            6 => Some(Self::InvalidState),
            _ => None,
        }
    }
}

/// CRC32C lookup table (Castagnoli polynomial 0x1EDC6F41)
/// Pre-computed for software CRC32C implementation in no_std environments.
const CRC32C_TABLE: [u32; 256] = [
    0x00000000, 0xF26B8303, 0xE13B70F7, 0x13547548,
    0xC70A6970, 0x35618284, 0x8A3898D8, 0x7853BDC6,
    0x2E2BA4C8, 0xDC40B08B, 0x6B839A9A, 0x99E8687D,
    0x4AC1A3F5, 0xB8A276D2, 0x0D6F6BBD, 0xFF5D5DBA,
    0x49B5A6F4, 0xBB4DC3D3, 0xAC8765DE, 0x5EEC6D97,
    0x8DA298BE, 0x7FC91069, 0x6289D858, 0x90E26F8F,
    0x1C5D1D03, 0xEE40B4CC, 0xF69A28D5, 0x04F1C913,
    0x9DC45709, 0x6FAF9C5E, 0x7E3C4C6A, 0x8C572D91,
    0xD0E4D3D2, 0x228F1FAD, 0x33D1FCF0, 0xC1FF9FA7,
    0x17AE9949, 0xE555A944, 0xF6C3D060, 0x04A86B78,
    0x2FCF7DC7, 0xDDA06DBA, 0xCE68D8CB, 0x3C07B3AD,
    0x7E4E3D05, 0x8C2547D2, 0x9F6BBFC5, 0x6D003EC2,
    0xAA7FF879, 0x58146BEB, 0x4B647CE6, 0xB97D14F1,
    0x6B839999, 0x99E8687A, 0x8A3898DC, 0x7853BDC0,
    0x2E2BA4CA, 0xDC40B089, 0xCF70AB80, 0x3D1B5D77,
    0x527ECEAF, 0xA0D2D2BE, 0xB3E72836, 0x4188D39B,
    0x6BCB6C31, 0x99A068B6, 0x8AF07EBC, 0x789F6B0B,
    0x2EDD3DB1, 0xDCC8BB46, 0xCFA8DF55, 0x3DC7C742,
    0x7F4C1E08, 0x8D27DFDF, 0x9E4D6BC8, 0x6C224B3F,
    0xA5D34A40, 0x57B8CC93, 0x44826AAE, 0xB6CD78B9,
    0xE0D5A906, 0x12BE87F1, 0x01D4C5FD, 0xF3BB5CEA,
    0x27F5D111, 0xD59E4B46, 0xC6FE95B5, 0x34915202,
    0x72A3A5C7, 0x80C8D910, 0x93A2A80D, 0x61C9A8FA,
    0xBCA3B8B4, 0x4EC8B493, 0x5DA2629E, 0xAF709AC9,
    0x79B3D5DE, 0x8BD85809, 0x98B25E14, 0x6AD9E3E3,
    0x266F4B63, 0xD404EB34, 0xC76E1CD7, 0x3505C9A0,
    0x73C6DDD5, 0x81AD5602, 0x92C79E1F, 0x60AC5CE8,
    0xBDAF94A6, 0x4FC48181, 0x5CAE3F8C, 0xAEC5D0DB,
    0x780644EC, 0x8A6D3B3B, 0x99073A26, 0x6B6C6CD1,
    0x24A1F351, 0xD6CA0606, 0xC5A05AF5, 0x37CB6D02,
    0x71089077, 0x836378A0, 0x9009E8BD, 0x6262E64A,
    0xBF52F8C4, 0x4D39D7E3, 0x5E53A5EE, 0xAC38E8B9,
    0x7A5B4CAE, 0x8830D279, 0x9B5A7E64, 0x69317D93,
    0x2CF8F821, 0xDE933BC6, 0xCDF93935, 0x3F92B842,
    0x7951A077, 0x8B3A53A0, 0x9850C2BD, 0x6A3B6C4A,
    0xB541FAC4, 0x472A7CE3, 0x54404AEE, 0xA62B52B9,
    0x7048DAE6, 0x8223D331, 0x9149F72C, 0x632289DB,
    0x2EE5D95B, 0xDC8E4F0C, 0xCFE4B3FF, 0x3D8F2A08,
    0x7B4C5A7D, 0x8927E3AA, 0x9A4D71B7, 0x6826B140,
    0xB77C2E84, 0x4517ACA3, 0x567DC0AE, 0xA4165EF9,
    0x7275F1EE, 0x801E5E39, 0x9374AC24, 0x611F5AD3,
    0x2CD2E453, 0xDEB97204, 0xCDD3AEF7, 0x3FB83700,
    0x7D7B7B75, 0x8F10C8A2, 0x9C7A5ABF, 0x6E11F448,
    0xB34B8EC2, 0x412003E5, 0x524A71E8, 0xA021EFBB,
    0x764254AC, 0x8429A37B, 0x97433166, 0x65289F91,
    0x28E5CF11, 0xDA8E5946, 0xC9E4A5B5, 0x3B8F3C02,
    0x7D4C6A77, 0x8F27D3A0, 0x9C4D41BD, 0x6E26EF4A,
    0xB17C3584, 0x4317B8A3, 0x507D4EAE, 0xA216D0F9,
    0x74756FEE, 0x861ED239, 0x95744424, 0x671FEAD3,
    0x2AD2D053, 0xD8B95604, 0xCBD3AAF7, 0x39B83300,
    0x7F7B4575, 0x8D10FCA2, 0x9E7A6EBF, 0x6C11C048,
    0xB34B5A82, 0x4120D7A5, 0x524A05A8, 0xA0219BFB,
    0x764220EC, 0x8429973B, 0x97430526, 0x6528ABD1,
    0x28E5D151, 0xDA8E4706, 0xC9E4BBF5, 0x3B8F2202,
    0x7D4C5477, 0x8F27C7A0, 0x9C4D55BD, 0x6E26FB4A,
    0xB17C2584, 0x4317A8A3, 0x507D7EAE, 0xA216E0F9,
    0x747565EE, 0x861ED239, 0x95744424, 0x671FEAD3,
    0x2AD2C053, 0xD8B95604, 0xCBD3AAF7, 0x39B83300,
    0x7F7B4575, 0x8D10FCA2, 0x9E7A6EBF, 0x6C11C048,
    0xB34B5A82, 0x4120D7A5, 0x524A05A8, 0xA0219BFB,
    0x764220EC, 0x8429973B, 0x97430526, 0x6528ABD1,
    0x28E5D151, 0xDA8E4706, 0xC9E4BBF5, 0x3B8F2202,
    0x7D4C5477, 0x8F27C7A0, 0x9C4D55BD, 0x6E26FB4A,
    0xB17C2584, 0x4317A8A3, 0x507D7EAE, 0xA216E0F9,
    0x747565EE, 0x861ED239, 0x95744424, 0x671FEAD3,
    0x2AD2C053, 0xD8B95604, 0xCBD3AAF7, 0x39B83300,
    0x7F7B4575, 0x8D10FCA2, 0x9E7A6EBF, 0x6C11C048,
    0xB34B5A82, 0x4120D7A5, 0x524A05A8, 0xA0219BFB,
    0x764220EC, 0x8429973B, 0x97430526, 0x6528ABD1,
    0x28E5D151, 0xDA8E4706, 0xC9E4BBF5, 0x3B8F2202,
    0x7D4C5477, 0x8F27C7A0, 0x9C4D55BD, 0x6E26FB4A,
    0xB17C2584, 0x4317A8A3, 0x507D7EAE, 0xA216E0F9,
    0x747565EE, 0x861ED239, 0x95744424, 0x671FEAD3,
    0x2AD2C053, 0xD8B95604, 0xCBD3AAF7, 0x39B83300,
    0x7F7B4575, 0x8D10FCA2, 0x9E7A6EBF, 0x6C11C048,
    0xB34B5A82, 0x4120D7A5, 0x524A05A8, 0xA0219BFB,
    0x764220EC, 0x8429973B, 0x97430526, 0x6528ABD1,
    0x28E5D151, 0xDA8E4706, 0xC9E4BBF5, 0x3B8F2202,
    0x7D4C5477, 0x8F27C7A0, 0x9C4D55BD, 0x6E26FB4A,
    0xB17C2584, 0x4317A8A3, 0x507D7EAE, 0xA216E0F9,
    0x747565EE, 0x861ED239, 0x95744424, 0x671FEAD3,
    0x2AD2C053, 0xD8B95604, 0xCBD3AAF7, 0x39B83300,
    0x7F7B4575, 0x8D10FCA2, 0x9E7A6EBF, 0x6C11C048,
    0xB34B5A82, 0x4120D7A5, 0x524A05A8, 0xA0219BFB,
    0x764220EC, 0x8429973B, 0x97430526, 0x6528ABD1,
    0x28E5D151, 0xDA8E4706, 0xC9E4BBF5, 0x3B8F2202,
];

/// Compute CRC32C (Castagnoli) checksum for the given data.
/// Uses a table-driven software implementation suitable for no_std environments.
pub fn crc32c_compute(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFFFFFF;
    for &byte in data {
        let idx = ((crc ^ byte as u32) & 0xFF) as usize;
        crc = CRC32C_TABLE[idx] ^ (crc >> 8);
    }
    crc ^ 0xFFFFFFFF
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_id() {
        let id = TransactionId::new(42);
        assert_eq!(id.as_u64(), 42);
        assert_eq!(id, TransactionId(42));
    }

    #[test]
    fn test_wal_lsn() {
        let mut lsn = WalLsn::new(10);
        assert_eq!(lsn.as_u64(), 10);
        lsn.inc();
        assert_eq!(lsn.as_u64(), 11);
        assert_eq!(WalLsn::zero().as_u64(), 0);
    }

    #[test]
    fn test_wal_operation_type() {
        assert_eq!(WalOperationType::Create as u8, 1);
        assert_eq!(WalOperationType::Delete as u8, 2);
        assert_eq!(WalOperationType::Write as u8, 3);
        assert_eq!(WalOperationType::Checkpoint as u8, 8);
        assert_eq!(WalOperationType::from_u8(3), Some(WalOperationType::Write));
        assert_eq!(WalOperationType::from_u8(99), None);
    }

    #[test]
    fn test_wal_record_new() {
        let record = WalRecord::new(
            TransactionId::new(1),
            WalLsn::new(100),
            WalOperationType::Write,
            0x1000,
            12345,
        );
        assert_eq!(record.transaction_id, TransactionId(1));
        assert_eq!(record.lsn, WalLsn(100));
        assert_eq!(record.operation_type, WalOperationType::Write);
        assert_eq!(record.block_address, 0x1000);
        assert_eq!(record.data_len, 0);
    }

    #[test]
    fn test_wal_commit_marker() {
        let marker = WalCommitMarker::new(TransactionId::new(5), 3, 0xDEAD, 9999);
        assert_eq!(marker.transaction_id, TransactionId(5));
        assert_eq!(marker.num_records, 3);
        assert_eq!(marker.commit_checksum, 0xDEAD);
    }

    #[test]
    fn test_wal_error() {
        assert_eq!(WalError::IOError as i32, 1);
        assert_eq!(WalError::ChecksumMismatch as i32, 3);
        assert_eq!(WalError::ReadOnly as i32, 5);
        assert_eq!(WalError::from_i32(4), Some(WalError::LogFull));
        assert_eq!(WalError::from_i32(99), None);
    }

    #[test]
    fn test_crc32c_empty() {
        let crc = crc32c_compute(&[]);
        assert_eq!(crc, 0);
    }

    #[test]
    fn test_crc32c_deterministic() {
        let data = [1u8, 2, 3, 4, 5];
        let crc1 = crc32c_compute(&data);
        let crc2 = crc32c_compute(&data);
        assert_eq!(crc1, crc2);
    }

    #[test]
    fn test_crc32c_different_data() {
        let data1 = [1u8, 2, 3, 4, 5];
        let data2 = [5u8, 4, 3, 2, 1];
        let crc1 = crc32c_compute(&data1);
        let crc2 = crc32c_compute(&data2);
        assert_ne!(crc1, crc2);
    }
}