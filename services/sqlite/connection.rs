/*
 * Nuva OS - SystemService - SQLite - Connection Management
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

//! Connection pool and per-connection state management for SQLite service.

use alloc::collections::BTreeMap;
use alloc::string::String;
use core::sync::atomic::{AtomicU64, Ordering};

use super::error::SqliteError;
use super::transaction::TransactionState;

/// Connection identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ConnectionId(pub u64);

impl ConnectionId {
    /// Invalid connection ID sentinel
    pub const INVALID: ConnectionId = ConnectionId(0);
}

/// Monotonically increasing connection ID generator
static NEXT_CONNECTION_ID: AtomicU64 = AtomicU64::new(1);

/// Transaction isolation level for a connection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IsolationLevel {
    /// Read-uncommitted: allows dirty reads
    ReadUncommitted = 0,
    /// Read-committed: default SQLite behavior
    ReadCommitted = 1,
    /// Serializable: full isolation via BEGIN EXCLUSIVE
    Serializable = 2,
}

/// A single database connection
#[derive(Debug)]
pub struct DatabaseConnection {
    /// Unique connection ID
    pub id: ConnectionId,
    /// Database file path
    pub db_path: String,
    /// Owning process ID
    pub pid: u32,
    /// Owning user ID
    pub uid: u32,
    /// Current transaction state
    pub tx_state: TransactionState,
    /// Isolation level for this connection
    pub isolation: IsolationLevel,
    /// Whether the connection is read-only
    pub read_only: bool,
    /// Whether encryption is enabled for this connection
    pub encrypted: bool,
    /// Number of active prepared statements
    pub stmt_count: u32,
}

impl DatabaseConnection {
    /// Create a new database connection
    pub fn new(db_path: String, pid: u32, uid: u32, read_only: bool, encrypted: bool) -> Self {
        let id = ConnectionId(NEXT_CONNECTION_ID.fetch_add(1, Ordering::Relaxed));
        DatabaseConnection {
            id,
            db_path,
            pid,
            uid,
            tx_state: TransactionState::None,
            isolation: IsolationLevel::ReadCommitted,
            read_only,
            encrypted,
            stmt_count: 0,
        }
    }

    /// Returns true if this connection has an active transaction
    pub fn in_transaction(&self) -> bool {
        self.tx_state != TransactionState::None
    }

    /// Returns true if this connection can perform writes
    pub fn can_write(&self) -> bool {
        !self.read_only
    }
}

/// Connection pool managing all active database connections
#[derive(Debug)]
pub struct ConnectionPool {
    /// Active connections indexed by ConnectionId
    connections: BTreeMap<u64, DatabaseConnection>,
    /// Maximum number of concurrent connections
    max_connections: u32,
}

/// Default maximum concurrent connections
const DEFAULT_MAX_CONNECTIONS: u32 = 128;

impl ConnectionPool {
    /// Create a new connection pool with default limits
    pub fn new() -> Self {
        ConnectionPool::with_max_connections(DEFAULT_MAX_CONNECTIONS)
    }

    /// Create a connection pool with a custom maximum
    pub fn with_max_connections(max_connections: u32) -> Self {
        ConnectionPool {
            connections: BTreeMap::new(),
            max_connections,
        }
    }

    /// Open a new connection and add it to the pool
    pub fn open(
        &mut self,
        db_path: String,
        pid: u32,
        uid: u32,
        read_only: bool,
        encrypted: bool,
    ) -> Result<ConnectionId, SqliteError> {
        if self.connections.len() >= self.max_connections as usize {
            return Err(SqliteError::ConnectionLimitExceeded);
        }

        let conn = DatabaseConnection::new(db_path, pid, uid, read_only, encrypted);
        let id = conn.id;
        self.connections.insert(id.0, conn);
        Ok(id)
    }

    /// Close a connection and remove it from the pool
    pub fn close(&mut self, conn_id: ConnectionId) -> Result<(), SqliteError> {
        if self.connections.remove(&conn_id.0).is_some() {
            Ok(())
        } else {
            Err(SqliteError::InvalidConnection)
        }
    }

    /// Get a reference to a connection by ID
    pub fn get(&self, conn_id: ConnectionId) -> Option<&DatabaseConnection> {
        self.connections.get(&conn_id.0)
    }

    /// Get a mutable reference to a connection by ID
    pub fn get_mut(&mut self, conn_id: ConnectionId) -> Option<&mut DatabaseConnection> {
        self.connections.get_mut(&conn_id.0)
    }

    /// Returns the number of active connections
    pub fn count(&self) -> u32 {
        self.connections.len() as u32
    }

    /// Returns true if the pool has reached its maximum capacity
    pub fn is_full(&self) -> bool {
        self.connections.len() >= self.max_connections as usize
    }

    /// Close all connections (used during shutdown)
    pub fn close_all(&mut self) {
        self.connections.clear();
    }
}
