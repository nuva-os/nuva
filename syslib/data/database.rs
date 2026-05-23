/*
 * Nuva OS - SystemLibrary - Data
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

//! Database Engine

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// Database Error
#[derive(Debug, Clone, Copy)]
pub enum DatabaseError {
    ConnectionFailed,
    QueryFailed,
    TransactionFailed,
    NotFound,
    AlreadyExists,
    InvalidQuery,
    OutOfMemory,
    DiskFull,
    Corruption,
}

/// Database Result
pub type DatabaseResult<T> = Result<T, DatabaseError>;

/// Database Connection Configuration
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub path: [u8; 256],
    pub path_len: u8,
    pub max_connections: u32,
    pub cache_size: u64,
    pub journal_mode: JournalMode,
    pub synchronous: SynchronousMode,
}

/// Journal Mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum JournalMode {
    Delete = 0,
    Truncate = 1,
    Persist = 2,
    Memory = 3,
    WAL = 4,
    Off = 5,
}

/// Synchronous Mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SynchronousMode {
    Off = 0,
    Normal = 1,
    Full = 2,
    Extra = 3,
}

/// SQL Value Type
#[derive(Debug, Clone, Copy)]
pub enum SqlValue {
    Null,
    Integer(i64),
    Float(f64),
    Text([u8; 256], u8),
    Blob([u8; 1024], u16),
}

impl SqlValue {
    pub fn is_null(&self) -> bool {
        matches!(self, SqlValue::Null)
    }

    pub fn as_integer(&self) -> Option<i64> {
        match self {
            SqlValue::Integer(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        match self {
            SqlValue::Float(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_text(&self) -> Option<&[u8]> {
        match self {
            SqlValue::Text(buf, len) => Some(&buf[..*len as usize]),
            _ => None,
        }
    }
}

/// Database Column
#[derive(Debug, Clone)]
pub struct Column {
    pub name: [u8; 64],
    pub name_len: u8,
    pub data_type: ColumnType,
    pub nullable: bool,
    pub primary_key: bool,
    pub auto_increment: bool,
}

impl Column {
    pub const fn new() -> Self {
        Column {
            name: [0; 64],
            name_len: 0,
            data_type: ColumnType::Integer,
            nullable: true,
            primary_key: false,
            auto_increment: false,
        }
    }
}


/// Column Type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ColumnType {
    Integer = 0,
    Real = 1,
    Text = 2,
    Blob = 3,
}

/// Database Table
#[derive(Debug)]
pub struct Table {
    pub name: [u8; 64],
    pub name_len: u8,
    pub columns: [Column; 32],
    pub num_columns: u8,
    pub row_count: AtomicU64,
}

impl Clone for Table {
    fn clone(&self) -> Self {
        Table {
            name: self.name.clone(),
            name_len: self.name_len,
            columns: self.columns.clone(),
            num_columns: self.num_columns,
            row_count: AtomicU64::new(self.row_count.load(Ordering::Relaxed)),
        }
    }
}


impl Table {
    pub fn new(name: &[u8]) -> Self {
        let mut name_buf = [0u8; 64];
        let len = name.len().min(63);
        name_buf[..len].copy_from_slice(&name[..len]);

        Self {
            name: name_buf,
            name_len: len as u8,
            columns: [const { Column::new() }; 32],
            num_columns: 0,
            row_count: AtomicU64::new(0),
        }
    }

    pub fn add_column(&mut self, column: Column) {
        if self.num_columns < 32 {
            self.columns[self.num_columns as usize] = column;
            self.num_columns += 1;
        }
    }

    pub fn name(&self) -> &[u8] {
        &self.name[..self.name_len as usize]
    }
}

/// Query Result Row
#[derive(Debug, Clone, Copy)]
pub struct Row {
    pub values: [SqlValue; 32],
    pub num_values: u8,
}

impl Row {
    pub const fn new() -> Self {
        Self {
            values: [SqlValue::Null; 32],
            num_values: 0,
        }
    }

    pub fn add_value(&mut self, value: SqlValue) {
        if self.num_values < 32 {
            self.values[self.num_values as usize] = value;
            self.num_values += 1;
        }
    }

    pub fn get(&self, index: usize) -> Option<&SqlValue> {
        if index < self.num_values as usize {
            Some(&self.values[index])
        } else {
            None
        }
    }
}

/// Query Result
#[derive(Debug)]
pub struct QueryResult {
    pub rows: [Row; 1024],
    pub num_rows: AtomicU32,
    pub columns_affected: u32,
    pub last_insert_id: u64,
}

impl QueryResult {
    pub fn new() -> Self {
        Self {
            rows: [const { Row::new() }; 1024],
            num_rows: AtomicU32::new(0),
            columns_affected: 0,
            last_insert_id: 0,
        }
    }

    pub fn add_row(&mut self, row: Row) {
        let idx = self.num_rows.load(Ordering::Relaxed) as usize;
        if idx < 1024 {
            self.rows[idx] = row;
            self.num_rows.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn row_count(&self) -> u32 {
        self.num_rows.load(Ordering::Relaxed)
    }
}

/// Transaction Isolation Level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum IsolationLevel {
    ReadUncommitted = 0,
    ReadCommitted = 1,
    RepeatableRead = 2,
    Serializable = 3,
}

/// Transaction
pub struct Transaction {
    pub id: u64,
    pub isolation_level: IsolationLevel,
    pub is_active: AtomicU32,
    pub start_time: AtomicU64,
}

impl Transaction {
    pub fn new(id: u64, level: IsolationLevel) -> Self {
        Self {
            id,
            isolation_level: level,
            is_active: AtomicU32::new(1),
            start_time: AtomicU64::new(0),
        }
    }

    pub fn is_active(&self) -> bool {
        self.is_active.load(Ordering::Relaxed) != 0
    }

    pub fn commit(&self) -> DatabaseResult<()> {
        self.is_active.store(0, Ordering::Release);
        Ok(())
    }

    pub fn rollback(&self) -> DatabaseResult<()> {
        self.is_active.store(0, Ordering::Release);
        Ok(())
    }
}

/// Database Connection
pub struct DatabaseConnection {
    pub id: u64,
    pub config: DatabaseConfig,
    pub is_open: AtomicU32,
    pub transaction: AtomicU64,
}

impl Clone for DatabaseConnection {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            config: self.config.clone(),
            is_open: AtomicU32::new(self.is_open.load(core::sync::atomic::Ordering::Relaxed)),
            transaction: AtomicU64::new(self.transaction.load(core::sync::atomic::Ordering::Relaxed)),
        }
    }
}

impl DatabaseConnection {
    pub fn new(id: u64, config: DatabaseConfig) -> Self {
        Self {
            id,
            config,
            is_open: AtomicU32::new(0),
            transaction: AtomicU64::new(0),
        }
    }

    pub fn open(&mut self) -> DatabaseResult<()> {
        self.is_open.store(1, Ordering::Release);
        Ok(())
    }

    pub fn close(&mut self) -> DatabaseResult<()> {
        self.is_open.store(0, Ordering::Release);
        Ok(())
    }

    pub fn is_open(&self) -> bool {
        self.is_open.load(Ordering::Relaxed) != 0
    }

    pub fn execute(&mut self, _sql: &[u8]) -> DatabaseResult<QueryResult> {
        if !self.is_open() {
            return Err(DatabaseError::ConnectionFailed);
        }

        Ok(QueryResult::new())
    }

    pub fn query(&mut self, _sql: &[u8]) -> DatabaseResult<QueryResult> {
        if !self.is_open() {
            return Err(DatabaseError::ConnectionFailed);
        }

        Ok(QueryResult::new())
    }

    pub fn begin_transaction(&mut self, level: IsolationLevel) -> DatabaseResult<Transaction> {
        let txn = Transaction::new(self.id * 1000, level);
        self.transaction.store(txn.id, Ordering::Release);
        Ok(txn)
    }
}

/// Database Manager
pub struct DatabaseManager {
    connections: [Option<DatabaseConnection>; 16],
    num_connections: AtomicU32,
    next_connection_id: AtomicU64,
    tables: [Option<Table>; 64],
    num_tables: AtomicU32,
}

impl DatabaseManager {
    pub fn new() -> Self {
        Self {
            connections: [const { None }; 16],
            num_connections: AtomicU32::new(0),
            next_connection_id: AtomicU64::new(1),
            tables: [const { None }; 64],
            num_tables: AtomicU32::new(0),
        }
    }

    pub fn init(&mut self) {
        crate::log_info!("Database manager initialized");
    }

    pub fn create_connection(&mut self, config: DatabaseConfig) -> DatabaseResult<u64> {
        let id = self.next_connection_id.fetch_add(1, Ordering::Relaxed);
        let conn = DatabaseConnection::new(id, config);

        let idx = self.num_connections.load(Ordering::Relaxed) as usize;
        if idx < 16 {
            self.connections[idx] = Some(conn);
            self.num_connections.fetch_add(1, Ordering::Relaxed);
            return Ok(id);
        }

        Err(DatabaseError::OutOfMemory)
    }

    pub fn get_connection(&mut self, id: u64) -> Option<&mut DatabaseConnection> {
        let num = self.num_connections.load(Ordering::Relaxed) as usize;
        for conn in self.connections[..num].iter_mut() {
            if let Some(ref mut c) = conn {
                if c.id == id {
                    return Some(c);
                }
            }
        }
        None
    }

    pub fn close_connection(&mut self, id: u64) -> DatabaseResult<()> {
        if let Some(conn) = self.get_connection(id) {
            conn.close()?;
        }
        Ok(())
    }

    pub fn create_table(&mut self, table: Table) -> DatabaseResult<()> {
        let idx = self.num_tables.load(Ordering::Relaxed) as usize;
        if idx < 64 {
            self.tables[idx] = Some(table);
            self.num_tables.fetch_add(1, Ordering::Relaxed);
            return Ok(());
        }
        Err(DatabaseError::OutOfMemory)
    }

    pub fn get_table(&self, name: &[u8]) -> Option<&Table> {
        for i in 0..self.num_tables.load(Ordering::Relaxed) as usize {
            if let Some(ref table) = self.tables[i] {
                if table.name() == name {
                    return Some(table);
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_config() -> DatabaseConfig {
        DatabaseConfig {
            path: [0u8; 256],
            path_len: 0,
            max_connections: 10,
            cache_size: 1024 * 1024,
            journal_mode: JournalMode::WAL,
            synchronous: SynchronousMode::Normal,
        }
    }

    #[test]
    fn test_journal_mode() {
        assert_eq!(JournalMode::Delete as u8, 0);
        assert_eq!(JournalMode::Truncate as u8, 1);
        assert_eq!(JournalMode::WAL as u8, 4);
        assert_eq!(JournalMode::Off as u8, 5);
    }

    #[test]
    fn test_synchronous_mode() {
        assert_eq!(SynchronousMode::Off as u8, 0);
        assert_eq!(SynchronousMode::Normal as u8, 1);
        assert_eq!(SynchronousMode::Full as u8, 2);
        assert_eq!(SynchronousMode::Extra as u8, 3);
    }

    #[test]
    fn test_sql_value_null() {
        let value = SqlValue::Null;
        assert!(value.is_null());
        assert!(value.as_integer().is_none());
        assert!(value.as_float().is_none());
        assert!(value.as_text().is_none());
    }

    #[test]
    fn test_sql_value_integer() {
        let value = SqlValue::Integer(42);
        assert!(!value.is_null());
        assert_eq!(value.as_integer(), Some(42));
        assert!(value.as_float().is_none());
    }

    #[test]
    fn test_sql_value_float() {
        let value = SqlValue::Float(3.14);
        assert!(!value.is_null());
        assert_eq!(value.as_float(), Some(3.14));
        assert!(value.as_integer().is_none());
    }

    #[test]
    fn test_sql_value_text() {
        let mut buf = [0u8; 256];
        buf[..5].copy_from_slice(b"hello");
        let value = SqlValue::Text(buf, 5);

        assert!(!value.is_null());
        assert_eq!(value.as_text(), Some(&b"hello"[..]));
    }

    #[test]
    fn test_column_type() {
        assert_eq!(ColumnType::Integer as u8, 0);
        assert_eq!(ColumnType::Real as u8, 1);
        assert_eq!(ColumnType::Text as u8, 2);
        assert_eq!(ColumnType::Blob as u8, 3);
    }

    #[test]
    fn test_table_new() {
        let table = Table::new(b"users");

        assert_eq!(table.name(), b"users");
        assert_eq!(table.num_columns, 0);
        assert_eq!(table.row_count.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_table_add_column() {
        let mut table = Table::new(b"users");

        let col = Column {
            name: [0; 64],
            name_len: 0,
            data_type: ColumnType::Integer,
            nullable: false,
            primary_key: true,
            auto_increment: true,
        };

        table.add_column(col);
        assert_eq!(table.num_columns, 1);
    }

    #[test]
    fn test_row_new() {
        let row = Row::new();

        assert_eq!(row.num_values, 0);
        assert!(row.get(0).is_none());
    }

    #[test]
    fn test_row_add_value() {
        let mut row = Row::new();

        row.add_value(SqlValue::Integer(1));
        row.add_value(SqlValue::Integer(2));

        assert_eq!(row.num_values, 2);
        assert_eq!(row.get(0).unwrap().as_integer(), Some(1));
        assert_eq!(row.get(1).unwrap().as_integer(), Some(2));
    }

    #[test]
    fn test_query_result_new() {
        let result = QueryResult::new();

        assert_eq!(result.row_count(), 0);
        assert_eq!(result.columns_affected, 0);
        assert_eq!(result.last_insert_id, 0);
    }

    #[test]
    fn test_query_result_add_row() {
        let mut result = QueryResult::new();

        let mut row = Row::new();
        row.add_value(SqlValue::Integer(1));

        result.add_row(row);

        assert_eq!(result.row_count(), 1);
    }

    #[test]
    fn test_isolation_level() {
        assert_eq!(IsolationLevel::ReadUncommitted as u8, 0);
        assert_eq!(IsolationLevel::ReadCommitted as u8, 1);
        assert_eq!(IsolationLevel::RepeatableRead as u8, 2);
        assert_eq!(IsolationLevel::Serializable as u8, 3);
    }

    #[test]
    fn test_transaction() {
        let txn = Transaction::new(1, IsolationLevel::ReadCommitted);

        assert_eq!(txn.id, 1);
        assert_eq!(txn.isolation_level, IsolationLevel::ReadCommitted);
        assert!(txn.is_active());
    }

    #[test]
    fn test_transaction_commit() {
        let txn = Transaction::new(1, IsolationLevel::ReadCommitted);

        assert!(txn.is_active());
        txn.commit().unwrap();
        assert!(!txn.is_active());
    }

    #[test]
    fn test_transaction_rollback() {
        let txn = Transaction::new(1, IsolationLevel::ReadCommitted);

        assert!(txn.is_active());
        txn.rollback().unwrap();
        assert!(!txn.is_active());
    }

    #[test]
    fn test_database_connection() {
        let config = default_config();
        let mut conn = DatabaseConnection::new(1, config);

        assert_eq!(conn.id, 1);
        assert!(!conn.is_open());

        conn.open().unwrap();
        assert!(conn.is_open());

        conn.close().unwrap();
        assert!(!conn.is_open());
    }

    #[test]
    fn test_database_connection_execute() {
        let config = default_config();
        let mut conn = DatabaseConnection::new(1, config);

        // Execute should fail when not open
        let result = conn.execute(b"SELECT 1");
        assert!(result.is_err());

        // Execute should succeed after opening
        conn.open().unwrap();
        let result = conn.execute(b"SELECT 1");
        assert!(result.is_ok());
    }

    #[test]
    fn test_database_connection_transaction() {
        let config = default_config();
        let mut conn = DatabaseConnection::new(1, config);
        conn.open().unwrap();

        let txn = conn.begin_transaction(IsolationLevel::ReadCommitted);
        assert!(txn.is_ok());

        let txn = txn.unwrap();
        assert!(txn.is_active());
    }

    #[test]
    fn test_database_manager() {
        let mut manager = DatabaseManager::new();

        let config = default_config();
        let id = manager.create_connection(config);

        assert!(id.is_ok());
        assert_eq!(manager.num_connections.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_database_manager_get_connection() {
        let mut manager = DatabaseManager::new();

        let config = default_config();
        let id = manager.create_connection(config).unwrap();

        let conn = manager.get_connection(id);
        assert!(conn.is_some());

        let conn = manager.get_connection(999);
        assert!(conn.is_none());
    }

    #[test]
    fn test_database_manager_create_table() {
        let mut manager = DatabaseManager::new();

        let table = Table::new(b"users");
        let result = manager.create_table(table);

        assert!(result.is_ok());
        assert_eq!(manager.num_tables.load(Ordering::Relaxed), 1);

        let table = manager.get_table(b"users");
        assert!(table.is_some());
    }
}
