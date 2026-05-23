/*
 * Nuva OS - SystemService - SQLite
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

//! SQLite embedded database service for Nuva OS.
//! Provides a full-featured embedded SQL database with:
//! - SQL parsing and query planning
//! - B-Tree storage engine with page split/merge
//! - Write-Ahead Logging (WAL) for crash recovery
//! - ACID transaction management
//! - Page-level encryption (AES-256-XTS)
//! - Connection pooling with concurrency limits
//! - Zero-copy result set transfer via shared memory

pub use crate::{pr_emerg, pr_alert, pr_crit, pr_err, pr_warn, pr_notice, pr_info, pr_debug};

pub mod service_node;
pub mod connection;
pub mod parser;
pub mod planner;
pub mod executor;
pub mod btree;
pub mod wal;
pub mod pager;
pub mod transaction;
pub mod crypto;
pub mod result_set;
pub mod error;

pub use service_node::SqliteService;
pub use connection::{ConnectionId, ConnectionPool, DatabaseConnection, IsolationLevel};
pub use error::{SqliteError, Value, ColumnType, StatementId};
pub use executor::SqlExecutor;
pub use result_set::ResultSet;

/// Initialize the SQLite embedded database service
pub fn init_sqlite_service() {
    log_info!("SQLite service module loaded");
}
