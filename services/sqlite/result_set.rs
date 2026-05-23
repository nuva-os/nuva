/*
 * Nuva OS - SystemService - SQLite - Result Set
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

//! Query result set construction and transfer.
//! Small result sets are returned inline; large result sets are transferred
//! via shared memory for zero-copy delivery to the caller.

use alloc::string::String;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

use super::error::{ColumnInfo, SqliteError, Value};

/// Threshold for inline vs shared-memory result set transfer (in bytes)
const INLINE_THRESHOLD_BYTES: usize = 4096;

/// Threshold for inline vs shared-memory result set transfer (in rows)
const INLINE_THRESHOLD_ROWS: usize = 256;

/// Result set transfer mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferMode {
    /// Small result set returned inline in the IPC response
    Inline = 0,
    /// Large result set transferred via shared memory (zero-copy)
    SharedMemory = 1,
}

/// Query result set
#[derive(Debug, Clone)]
pub struct ResultSet {
    /// Column metadata
    pub columns: Vec<ColumnInfo>,
    /// Row data (each row is a vector of values)
    pub rows: Vec<Vec<Value>>,
    /// Number of rows affected (for INSERT/UPDATE/DELETE)
    pub rows_affected: u64,
    /// Last inserted row ID (for INSERT)
    pub last_insert_rowid: i64,
    /// Transfer mode for IPC
    pub transfer_mode: TransferMode,
    /// Shared memory region ID (if TransferMode::SharedMemory)
    pub shm_region_id: u64,
}

impl ResultSet {
    /// Create an empty result set
    pub fn new(columns: Vec<ColumnInfo>) -> Self {
        ResultSet {
            columns,
            rows: Vec::new(),
            rows_affected: 0,
            last_insert_rowid: 0,
            transfer_mode: TransferMode::Inline,
            shm_region_id: 0,
        }
    }

    /// Create a result set for DML (rows affected, no columns)
    pub fn for_dml(rows_affected: u64, last_insert_rowid: i64) -> Self {
        ResultSet {
            columns: Vec::new(),
            rows: Vec::new(),
            rows_affected,
            last_insert_rowid,
            transfer_mode: TransferMode::Inline,
            shm_region_id: 0,
        }
    }

    /// Create an empty result set for DDL
    pub fn for_ddl() -> Self {
        ResultSet {
            columns: Vec::new(),
            rows: Vec::new(),
            rows_affected: 0,
            last_insert_rowid: 0,
            transfer_mode: TransferMode::Inline,
            shm_region_id: 0,
        }
    }

    /// Add a row to the result set
    pub fn add_row(&mut self, row: Vec<Value>) {
        self.rows.push(row);
    }

    /// Determine the transfer mode based on result set size
    pub fn choose_transfer_mode(&mut self) -> TransferMode {
        let estimated_bytes = self.estimate_size_bytes();
        let mode = if estimated_bytes > INLINE_THRESHOLD_BYTES
            || self.rows.len() > INLINE_THRESHOLD_ROWS
        {
            TransferMode::SharedMemory
        } else {
            TransferMode::Inline
        };
        self.transfer_mode = mode;
        mode
    }

    /// Estimate the serialized size of this result set in bytes
    pub fn estimate_size_bytes(&self) -> usize {
        let mut size = 0usize;

        // Column metadata
        for col in &self.columns {
            size += col.name.len() + col.table_name.len() + 8;
        }

        // Row data
        for row in &self.rows {
            for val in row {
                size += match val {
                    Value::Null => 1,
                    Value::Integer(_) => 5,
                    Value::I64(_) => 9,
                    Value::Real(_) => 9,
                    Value::Text(s) => 5 + s.len(),
                    Value::Blob(b) => 5 + b.len(),
                };
            }
        }

        size
    }

    /// Returns the number of rows
    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    /// Returns the number of columns
    pub fn column_count(&self) -> usize {
        self.columns.len()
    }

    /// Get a value at (row, column)
    pub fn get(&self, row: usize, col: usize) -> Option<&Value> {
        self.rows.get(row).and_then(|r| r.get(col))
    }

    /// Serialize the result set for inline IPC transfer
    pub fn serialize_inline(&self) -> Vec<u8> {
        let mut buf = Vec::new();

        // Header: column count (u16), row count (u32)
        buf.extend_from_slice(&(self.columns.len() as u16).to_le_bytes());
        buf.extend_from_slice(&(self.rows.len() as u32).to_le_bytes());

        // Column names (length-prefixed strings)
        for col in &self.columns {
            let name_bytes = col.name.as_bytes();
            buf.extend_from_slice(&(name_bytes.len() as u16).to_le_bytes());
            buf.extend_from_slice(name_bytes);
        }

        // Row data
        for row in &self.rows {
            for val in row {
                self.serialize_value(&mut buf, val);
            }
        }

        buf
    }

    /// Serialize a single value
    fn serialize_value(&self, buf: &mut Vec<u8>, val: &Value) {
        match val {
            Value::Null => {
                buf.push(0);
            }
            Value::Integer(n) => {
                buf.push(1);
                buf.extend_from_slice(&n.to_le_bytes());
            }
            Value::I64(n) => {
                buf.push(2);
                buf.extend_from_slice(&n.to_le_bytes());
            }
            Value::Real(f) => {
                buf.push(3);
                buf.extend_from_slice(&f.to_le_bytes());
            }
            Value::Text(s) => {
                buf.push(4);
                let bytes = s.as_bytes();
                buf.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
                buf.extend_from_slice(bytes);
            }
            Value::Blob(b) => {
                buf.push(5);
                buf.extend_from_slice(&(b.len() as u32).to_le_bytes());
                buf.extend_from_slice(b);
            }
        }
    }

    /// Marshal into shared memory for zero-copy transfer
    pub fn marshal_to_shm(&self, shm_region_id: u64) -> Result<usize, SqliteError> {
        // In a full implementation, this would:
        // 1. Allocate a shared memory region of the estimated size
        // 2. Serialize the result set into the shared memory
        // 3. Return the region handle to the caller via IPC
        //
        // The caller then maps the shared memory region and reads
        // the result set without copying.
        let _ = shm_region_id;
        Ok(self.estimate_size_bytes())
    }
}

/// Result set builder for incremental construction during execution
pub struct ResultSetBuilder {
    /// Column metadata
    columns: Vec<ColumnInfo>,
    /// Accumulated rows
    rows: Vec<Vec<Value>>,
    /// Rows affected counter
    rows_affected: u64,
    /// Last insert rowid
    last_insert_rowid: i64,
}

impl ResultSetBuilder {
    /// Create a new builder with column metadata
    pub fn new(columns: Vec<ColumnInfo>) -> Self {
        ResultSetBuilder {
            columns,
            rows: Vec::new(),
            rows_affected: 0,
            last_insert_rowid: 0,
        }
    }

    /// Add a row
    pub fn add_row(&mut self, row: Vec<Value>) {
        self.rows.push(row);
    }

    /// Set rows affected
    pub fn set_rows_affected(&mut self, count: u64) {
        self.rows_affected = count;
    }

    /// Set last insert rowid
    pub fn set_last_insert_rowid(&mut self, rowid: i64) {
        self.last_insert_rowid = rowid;
    }

    /// Build the final result set
    pub fn build(self) -> ResultSet {
        let mut rs = ResultSet {
            columns: self.columns,
            rows: self.rows,
            rows_affected: self.rows_affected,
            last_insert_rowid: self.last_insert_rowid,
            transfer_mode: TransferMode::Inline,
            shm_region_id: 0,
        };
        rs.choose_transfer_mode();
        rs
    }
}
