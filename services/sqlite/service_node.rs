/*
 * Nuva OS - SystemService - SQLite - Service Node
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

//! SQLite embedded database service node implementing CoreProcessingService trait.
//! Registered as "nuva.service.sqlite" in the Nuva IPC framework.

use core::sync::atomic::{AtomicU64, Ordering};

use crate::services::core_processing::service_node::{
    CallerIdentity, CoreProcessingService, ServiceConfig, ServiceHealth, ServiceNodeId,
    ServiceStats, ServiceVersion,
};
use crate::services::core_processing::error::ServiceError;

use super::connection::ConnectionId;
use super::error::SqliteError;
use super::executor::SqlExecutor;
use super::result_set::ResultSet;

/// Convert SqliteError to ServiceError
impl From<SqliteError> for ServiceError {
    fn from(e: SqliteError) -> ServiceError {
        use crate::services::core_processing::error::ServiceSpecificError as Spe;
        match e {
            SqliteError::SyntaxError => ServiceError::Specific(Spe::SqliteSyntaxError),
            SqliteError::DatabaseCorrupted => ServiceError::Specific(Spe::SqliteDatabaseCorrupted),
            SqliteError::DiskFull => ServiceError::Specific(Spe::SqliteDiskFull),
            SqliteError::Busy => ServiceError::Specific(Spe::SqliteBusy),
            SqliteError::IoError => ServiceError::Specific(Spe::SqliteIoError),
            SqliteError::PermissionDenied => ServiceError::PermissionDenied,
            SqliteError::ConnectionLimitExceeded => {
                ServiceError::Specific(Spe::SqliteConnectionLimitExceeded)
            }
            SqliteError::EncryptionError => ServiceError::Specific(Spe::SqliteEncryptionError),
            SqliteError::InvalidConnection => ServiceError::Specific(Spe::SqliteInvalidConnection),
            SqliteError::NoActiveTransaction => {
                ServiceError::Specific(Spe::SqliteNoActiveTransaction)
            }
        }
    }
}

/// SQLite service statistics
#[derive(Debug)]
pub struct SqliteStats {
    /// Total queries executed
    pub total_queries: AtomicU64,
    /// Total transactions started
    pub total_transactions: AtomicU64,
    /// Total connections opened
    pub total_connections: AtomicU64,
    /// Total pages read
    pub total_page_reads: AtomicU64,
    /// Total pages written
    pub total_page_writes: AtomicU64,
    /// WAL checkpoint count
    pub checkpoint_count: AtomicU64,
}

impl SqliteStats {
    /// Create zero-initialized stats
    pub const fn new() -> Self {
        SqliteStats {
            total_queries: AtomicU64::new(0),
            total_transactions: AtomicU64::new(0),
            total_connections: AtomicU64::new(0),
            total_page_reads: AtomicU64::new(0),
            total_page_writes: AtomicU64::new(0),
            checkpoint_count: AtomicU64::new(0),
        }
    }
}

/// SQLite embedded database service
pub struct SqliteService {
    /// Service configuration
    config: ServiceConfig,
    /// Core service statistics
    stats: ServiceStats,
    /// SQLite-specific statistics
    sqlite_stats: SqliteStats,
    /// SQL executor
    executor: SqlExecutor,
    /// Whether the service is initialized
    initialized: bool,
}

/// Default maximum concurrent requests for SQLite
const DEFAULT_MAX_CONCURRENT: u32 = 32;

/// Default request timeout in microseconds (30 seconds)
const DEFAULT_REQUEST_TIMEOUT_US: u64 = 30_000_000;

impl SqliteService {
    /// Create a new SQLite service instance
    pub fn new() -> Self {
        let config = ServiceConfig {
            name: "nuva.service.sqlite",
            version: ServiceVersion::new(1, 0, 0),
            max_concurrent_requests: DEFAULT_MAX_CONCURRENT,
            request_timeout_us: DEFAULT_REQUEST_TIMEOUT_US,
            hw_accel_available: true,
        };

        SqliteService {
            config,
            stats: ServiceStats::new(),
            sqlite_stats: SqliteStats::new(),
            executor: SqlExecutor::new(),
            initialized: false,
        }
    }

    /// Open a database connection
    pub fn open(
        &mut self,
        db_path: &str,
        pid: u32,
        uid: u32,
        read_only: bool,
        encrypted: bool,
    ) -> Result<ConnectionId, SqliteError> {
        if !self.initialized {
            return Err(SqliteError::InvalidConnection);
        }
        let conn_id = self.executor.open(db_path, pid, uid, read_only, encrypted)?;
        self.sqlite_stats
            .total_connections
            .fetch_add(1, Ordering::Relaxed);
        Ok(conn_id)
    }

    /// Close a database connection
    pub fn close(&mut self, conn_id: ConnectionId) -> Result<(), SqliteError> {
        self.executor.close(conn_id)
    }

    /// Execute a SQL statement
    pub fn execute(
        &mut self,
        conn_id: ConnectionId,
        sql: &str,
    ) -> Result<ResultSet, SqliteError> {
        if !self.initialized {
            return Err(SqliteError::InvalidConnection);
        }
        let result = self.executor.execute(conn_id, sql)?;
        self.sqlite_stats
            .total_queries
            .fetch_add(1, Ordering::Relaxed);
        Ok(result)
    }

    /// Begin a transaction
    pub fn begin_transaction(
        &mut self,
        conn_id: ConnectionId,
    ) -> Result<(), SqliteError> {
        if !self.initialized {
            return Err(SqliteError::InvalidConnection);
        }
        self.executor.begin_transaction(conn_id)?;
        self.sqlite_stats
            .total_transactions
            .fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    /// Commit the current transaction
    pub fn commit_transaction(
        &mut self,
        conn_id: ConnectionId,
    ) -> Result<(), SqliteError> {
        self.executor.commit_transaction(conn_id)
    }

    /// Rollback the current transaction
    pub fn rollback_transaction(
        &mut self,
        conn_id: ConnectionId,
    ) -> Result<(), SqliteError> {
        self.executor.rollback_transaction(conn_id)
    }

    /// Prepare a SQL statement
    pub fn prepare(
        &mut self,
        conn_id: ConnectionId,
        sql: &str,
    ) -> Result<super::error::StatementId, SqliteError> {
        if !self.initialized {
            return Err(SqliteError::InvalidConnection);
        }
        self.executor.prepare(conn_id, sql)
    }

    /// Bind a parameter and execute a prepared statement
    pub fn bind_execute(
        &mut self,
        stmt_id: super::error::StatementId,
    ) -> Result<ResultSet, SqliteError> {
        self.executor.bind_execute(stmt_id)
    }

    /// Get SQLite-specific statistics
    pub fn get_stats(&self) -> &SqliteStats {
        &self.sqlite_stats
    }
}

impl CoreProcessingService for SqliteService {
    fn config(&self) -> &ServiceConfig {
        &self.config
    }

    fn init(&mut self) -> Result<ServiceNodeId, ServiceError> {
        if self.initialized {
            return Err(ServiceError::Busy);
        }

        log_info!("Initializing SQLite service (nuva.service.sqlite)");

        // In a full implementation, this would:
        // 1. Initialize the WAL subsystem
        // 2. Perform crash recovery by replaying WAL
        // 3. Register service with Nuva IPC

        self.initialized = true;

        // SAFETY: converting reference to u64 for use as a unique ID within
        // this process. The reference remains valid as long as the service exists.
        let node_id = self as *const Self as u64;

        log_info!("SQLite service initialized, node_id={}", node_id);
        Ok(node_id)
    }

    fn handle_request(
        &mut self,
        caller: CallerIdentity,
        request_id: u64,
        payload: &[u8],
    ) -> Result<(), ServiceError> {
        if !self.initialized {
            return Err(ServiceError::NotInitialized);
        }

        self.stats.record_request(0);
        log_debug!(
            "SQLite service request: caller=({},{}) req_id={} len={}",
            caller.pid,
            caller.uid,
            request_id,
            payload.len()
        );

        // In a full implementation, payload is deserialized into
        // a SQLite IPC request (open/close/execute/prepare/bind_execute)
        // and dispatched to the appropriate method.
        self.stats.complete_request();
        Ok(())
    }

    fn shutdown(&mut self) -> Result<(), ServiceError> {
        if !self.initialized {
            return Err(ServiceError::NotInitialized);
        }

        log_info!("Shutting down SQLite service");

        // Close all connections
        self.executor.connections.close_all();

        // Abort all transactions
        self.executor.transactions.abort_all();

        self.initialized = false;
        Ok(())
    }

    fn health_check(&self) -> ServiceHealth {
        if !self.initialized {
            return ServiceHealth::NotInitialized;
        }
        ServiceHealth::Healthy
    }

    fn stats(&self) -> &ServiceStats {
        &self.stats
    }
}
