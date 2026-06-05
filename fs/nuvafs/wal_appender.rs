/*
 * Nuva OS - NuvaFS WAL Appender
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

//! NuvaFS WAL Appender
//! Provides append-only sequential write, CRC32C checksum computation,
//! log record serialization/deserialization, and batch write optimization.

use super::wal_types::{
    WalRecord, WalCommitMarker, TransactionId, WalLsn, WalOperationType,
    WAL_BLOCK_SIZE, crc32c_compute,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum WalEntryType {
    Record = 1,
    Commit = 2,
    Checkpoint = 3,
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct WalEntryHeader {
    pub entry_type: u8,
    pub _reserved: u8,
    pub payload_len: u16,
    pub payload_crc: u32,
}

pub const WAL_ENTRY_HEADER_SIZE: usize = core::mem::size_of::<WalEntryHeader>();
pub const WAL_RECORD_SERIALIZED_SIZE: usize = 8 + 8 + 1 + 7 + 8 + WAL_BLOCK_SIZE + WAL_BLOCK_SIZE + 4 + 4 + 8;
pub const WAL_COMMIT_MARKER_SERIALIZED_SIZE: usize = 8 + 4 + 4 + 8;
pub const WAL_MAX_ENTRY_SIZE: usize = WAL_ENTRY_HEADER_SIZE + WAL_RECORD_SERIALIZED_SIZE;
pub const WAL_BATCH_MAX_ENTRIES: usize = 64;
pub const WAL_BATCH_BUFFER_SIZE: usize = WAL_MAX_ENTRY_SIZE * WAL_BATCH_MAX_ENTRIES;

pub struct WalAppender {
    buffer: [u8; WAL_BATCH_BUFFER_SIZE],
    buffer_pos: usize,
    batch_count: usize,
    total_bytes_written: u64,
}

impl WalAppender {
    pub const fn new() -> Self {
        Self { buffer: [0u8; WAL_BATCH_BUFFER_SIZE], buffer_pos: 0, batch_count: 0, total_bytes_written: 0 }
    }

    pub fn serialize_record(record: &WalRecord, out: &mut [u8]) -> Option<usize> {
        let needed = WAL_ENTRY_HEADER_SIZE + WAL_RECORD_SERIALIZED_SIZE;
        if out.len() < needed { return None; }
        let ps = WAL_ENTRY_HEADER_SIZE;
        let p = &mut out[ps..];
        p[0..8].copy_from_slice(&record.transaction_id.0.to_le_bytes());
        p[8..16].copy_from_slice(&record.lsn.0.to_le_bytes());
        p[16] = record.operation_type as u8;
        p[17..24].copy_from_slice(&[0u8; 7]);
        p[24..32].copy_from_slice(&record.block_address.to_le_bytes());
        p[32..32 + WAL_BLOCK_SIZE].copy_from_slice(&record.old_data);
        let nd = 32 + WAL_BLOCK_SIZE;
        p[nd..nd + WAL_BLOCK_SIZE].copy_from_slice(&record.new_data);
        let dl = nd + WAL_BLOCK_SIZE;
        p[dl..dl + 4].copy_from_slice(&record.data_len.to_le_bytes());
        let cs = dl + 4;
        p[cs..cs + 4].copy_from_slice(&record.checksum.to_le_bytes());
        let ts = cs + 4;
        p[ts..ts + 8].copy_from_slice(&record.timestamp.to_le_bytes());
        let crc = crc32c_compute(&out[ps..ps + WAL_RECORD_SERIALIZED_SIZE]);
        out[0] = WalEntryType::Record as u8; out[1] = 0;
        out[2..4].copy_from_slice(&(WAL_RECORD_SERIALIZED_SIZE as u16).to_le_bytes());
        out[4..8].copy_from_slice(&crc.to_le_bytes());
        Some(needed)
    }

    pub fn deserialize_record(data: &[u8]) -> Option<(WalRecord, usize)> {
        let needed = WAL_ENTRY_HEADER_SIZE + WAL_RECORD_SERIALIZED_SIZE;
        if data.len() < needed { return None; }
        if data[0] != WalEntryType::Record as u8 { return None; }
        let plen = u16::from_le_bytes([data[2], data[3]]) as usize;
        if plen != WAL_RECORD_SERIALIZED_SIZE { return None; }
        let stored = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
        let ps = WAL_ENTRY_HEADER_SIZE;
        if crc32c_compute(&data[ps..ps + plen]) != stored { return None; }
        let p = &data[ps..];
        let tid = TransactionId(u64::from_le_bytes([p[0],p[1],p[2],p[3],p[4],p[5],p[6],p[7]]));
        let lsn = WalLsn(u64::from_le_bytes([p[8],p[9],p[10],p[11],p[12],p[13],p[14],p[15]]));
        let ot = WalOperationType::from_u8(p[16])?;
        let ba = u64::from_le_bytes([p[24],p[25],p[26],p[27],p[28],p[29],p[30],p[31]]);
        let mut od = [0u8; WAL_BLOCK_SIZE]; od.copy_from_slice(&p[32..32 + WAL_BLOCK_SIZE]);
        let nd = 32 + WAL_BLOCK_SIZE;
        let mut nd2 = [0u8; WAL_BLOCK_SIZE]; nd2.copy_from_slice(&p[nd..nd + WAL_BLOCK_SIZE]);
        let dl = nd + WAL_BLOCK_SIZE;
        let dlen = u32::from_le_bytes([p[dl],p[dl+1],p[dl+2],p[dl+3]]);
        let cs = dl + 4;
        let chk = u32::from_le_bytes([p[cs],p[cs+1],p[cs+2],p[cs+3]]);
        let ts = cs + 4;
        let tst = u64::from_le_bytes([p[ts],p[ts+1],p[ts+2],p[ts+3],p[ts+4],p[ts+5],p[ts+6],p[ts+7]]);
        Some((WalRecord{transaction_id:tid,lsn,operation_type:ot,block_address:ba,old_data:od,new_data:nd2,data_len:dlen,checksum:chk,timestamp:tst}, needed))
    }

    pub fn serialize_commit(marker: &WalCommitMarker, out: &mut [u8]) -> Option<usize> {
        let needed = WAL_ENTRY_HEADER_SIZE + WAL_COMMIT_MARKER_SERIALIZED_SIZE;
        if out.len() < needed { return None; }
        let ps = WAL_ENTRY_HEADER_SIZE;
        let p = &mut out[ps..];
        p[0..8].copy_from_slice(&marker.transaction_id.0.to_le_bytes());
        p[8..12].copy_from_slice(&marker.num_records.to_le_bytes());
        p[12..16].copy_from_slice(&marker.commit_checksum.to_le_bytes());
        p[16..24].copy_from_slice(&marker.timestamp.to_le_bytes());
        let crc = crc32c_compute(&out[ps..ps + WAL_COMMIT_MARKER_SERIALIZED_SIZE]);
        out[0] = WalEntryType::Commit as u8; out[1] = 0;
        out[2..4].copy_from_slice(&(WAL_COMMIT_MARKER_SERIALIZED_SIZE as u16).to_le_bytes());
        out[4..8].copy_from_slice(&crc.to_le_bytes());
        Some(needed)
    }

    pub fn deserialize_commit(data: &[u8]) -> Option<(WalCommitMarker, usize)> {
        let needed = WAL_ENTRY_HEADER_SIZE + WAL_COMMIT_MARKER_SERIALIZED_SIZE;
        if data.len() < needed { return None; }
        if data[0] != WalEntryType::Commit as u8 { return None; }
        let plen = u16::from_le_bytes([data[2], data[3]]) as usize;
        if plen != WAL_COMMIT_MARKER_SERIALIZED_SIZE { return None; }
        let stored = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
        let ps = WAL_ENTRY_HEADER_SIZE;
        if crc32c_compute(&data[ps..ps + plen]) != stored { return None; }
        let p = &data[ps..];
        let tid = TransactionId(u64::from_le_bytes([p[0],p[1],p[2],p[3],p[4],p[5],p[6],p[7]]));
        let nr = u32::from_le_bytes([p[8],p[9],p[10],p[11]]);
        let cc = u32::from_le_bytes([p[12],p[13],p[14],p[15]]);
        let ts = u64::from_le_bytes([p[16],p[17],p[18],p[19],p[20],p[21],p[22],p[23]]);
        Some((WalCommitMarker{transaction_id:tid,num_records:nr,commit_checksum:cc,timestamp:ts}, needed))
    }

    pub fn batch_append_record(&mut self, record: &WalRecord) -> Result<(), ()> {
        let remaining = self.buffer.len() - self.buffer_pos;
        let needed = WAL_ENTRY_HEADER_SIZE + WAL_RECORD_SERIALIZED_SIZE;
        if remaining < needed || self.batch_count >= WAL_BATCH_MAX_ENTRIES { return Err(()); }
        match Self::serialize_record(record, &mut self.buffer[self.buffer_pos..]) {
            Some(w) => { self.buffer_pos += w; self.batch_count += 1; Ok(()) }
            None => Err(()),
        }
    }

    pub fn batch_append_commit(&mut self, marker: &WalCommitMarker) -> Result<(), ()> {
        let remaining = self.buffer.len() - self.buffer_pos;
        let needed = WAL_ENTRY_HEADER_SIZE + WAL_COMMIT_MARKER_SERIALIZED_SIZE;
        if remaining < needed || self.batch_count >= WAL_BATCH_MAX_ENTRIES { return Err(()); }
        match Self::serialize_commit(marker, &mut self.buffer[self.buffer_pos..]) {
            Some(w) => { self.buffer_pos += w; self.batch_count += 1; Ok(()) }
            None => Err(()),
        }
    }

    pub fn batch_flush(&mut self) -> usize {
        let bytes = self.buffer_pos;
        self.total_bytes_written += bytes as u64;
        self.buffer_pos = 0; self.batch_count = 0;
        bytes
    }

    pub fn batch_count(&self) -> usize { self.batch_count }
    pub fn total_bytes_written(&self) -> u64 { self.total_bytes_written }
    pub fn batch_data(&self) -> &[u8] { &self.buffer[..self.buffer_pos] }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_serialize_deserialize_record() {
        let record = WalRecord::new(TransactionId::new(42), WalLsn::new(100), WalOperationType::Write, 0xDEAD_BEEF, 99999);
        let mut buf = [0u8; WAL_MAX_ENTRY_SIZE];
        let written = WalAppender::serialize_record(&record, &mut buf).unwrap();
        let (decoded, consumed) = WalAppender::deserialize_record(&buf[..written]).unwrap();
        assert_eq!(consumed, written);
        assert_eq!(decoded.transaction_id, TransactionId(42));
        assert_eq!(decoded.lsn, WalLsn(100));
        assert_eq!(decoded.block_address, 0xDEAD_BEEF);
    }
    #[test]
    fn test_serialize_deserialize_commit() {
        let marker = WalCommitMarker::new(TransactionId::new(7), 5, 0x1234_5678, 77777);
        let mut buf = [0u8; WAL_MAX_ENTRY_SIZE];
        let written = WalAppender::serialize_commit(&marker, &mut buf).unwrap();
        let (decoded, consumed) = WalAppender::deserialize_commit(&buf[..written]).unwrap();
        assert_eq!(consumed, written);
        assert_eq!(decoded.transaction_id, TransactionId(7));
        assert_eq!(decoded.num_records, 5);
    }
    #[test]
    fn test_crc_mismatch() {
        let record = WalRecord::new(TransactionId::new(1), WalLsn::new(1), WalOperationType::Create, 0, 0);
        let mut buf = [0u8; WAL_MAX_ENTRY_SIZE];
        let written = WalAppender::serialize_record(&record, &mut buf).unwrap();
        buf[WAL_ENTRY_HEADER_SIZE + 10] ^= 0xFF;
        assert!(WalAppender::deserialize_record(&buf[..written]).is_none());
    }
    #[test]
    fn test_batch() {
        let mut a = WalAppender::new();
        let r = WalRecord::new(TransactionId::new(1), WalLsn::new(1), WalOperationType::Write, 0, 0);
        assert!(a.batch_append_record(&r).is_ok());
        let m = WalCommitMarker::new(TransactionId::new(1), 1, 0, 0);
        assert!(a.batch_append_commit(&m).is_ok());
        assert!(a.batch_flush() > 0);
    }
}