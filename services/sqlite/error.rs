/*
 * Nuva OS - SystemService - SQLite - Error Model
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

//! SQLite service error types, SQL value types, and column metadata.

use alloc::vec::Vec;
use core::fmt;

/// SQLite service error type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SqliteError {
    /// SQL syntax error
    SyntaxError = 0,
    /// Database file is corrupted
    DatabaseCorrupted = 1,
    /// Disk is full
    DiskFull = 2,
    /// Database is locked by another connection
    Busy = 3,
    /// I/O error on database file
    IoError = 4,
    /// Permission denied on database file
    PermissionDenied = 5,
    /// Maximum concurrent connections exceeded
    ConnectionLimitExceeded = 6,
    /// Encryption/decryption error
    EncryptionError = 7,
    /// Invalid connection handle
    InvalidConnection = 8,
    /// No active transaction for this operation
    NoActiveTransaction = 9,
}

impl fmt::Display for SqliteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SqliteError::SyntaxError => write!(f, "SQL syntax error"),
            SqliteError::DatabaseCorrupted => write!(f, "Database corrupted"),
            SqliteError::DiskFull => write!(f, "Disk full"),
            SqliteError::Busy => write!(f, "Database busy"),
            SqliteError::IoError => write!(f, "I/O error"),
            SqliteError::PermissionDenied => write!(f, "Permission denied"),
            SqliteError::ConnectionLimitExceeded => write!(f, "Connection limit exceeded"),
            SqliteError::EncryptionError => write!(f, "Encryption error"),
            SqliteError::InvalidConnection => write!(f, "Invalid connection"),
            SqliteError::NoActiveTransaction => write!(f, "No active transaction"),
        }
    }
}

/// SQL value type
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// SQL NULL
    Null,
    /// 32-bit integer
    Integer(i32),
    /// 64-bit integer
    I64(i64),
    /// IEEE 754 floating point
    Real(f64),
    /// UTF-8 text string
    Text(alloc::string::String),
    /// Binary blob
    Blob(Vec<u8>),
}

impl Value {
    /// Returns true if this value is NULL
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    /// Returns the column type of this value
    pub fn column_type(&self) -> ColumnType {
        match self {
            Value::Null => ColumnType::Null,
            Value::Integer(_) => ColumnType::Integer,
            Value::I64(_) => ColumnType::Integer,
            Value::Real(_) => ColumnType::Real,
            Value::Text(_) => ColumnType::Text,
            Value::Blob(_) => ColumnType::Blob,
        }
    }
}

/// Column type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnType {
    /// NULL type
    Null = 0,
    /// Integer type
    Integer = 1,
    /// Floating point type
    Real = 2,
    /// Text type
    Text = 3,
    /// Binary blob type
    Blob = 4,
}

impl fmt::Display for ColumnType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ColumnType::Null => write!(f, "NULL"),
            ColumnType::Integer => write!(f, "INTEGER"),
            ColumnType::Real => write!(f, "REAL"),
            ColumnType::Text => write!(f, "TEXT"),
            ColumnType::Blob => write!(f, "BLOB"),
        }
    }
}

/// Prepared statement identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct StatementId(pub u64);

impl StatementId {
    /// Null/invalid statement ID
    pub const NULL: StatementId = StatementId(0);
}

/// Column descriptor in a result set
#[derive(Debug, Clone)]
pub struct ColumnInfo {
    /// Column name
    pub name: alloc::string::String,
    /// Column type
    pub column_type: ColumnType,
    /// Origin table name
    pub table_name: alloc::string::String,
    /// Whether this column may contain NULL
    pub nullable: bool,
}
