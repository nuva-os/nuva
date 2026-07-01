/*
 * Nuva OS - SystemService - SQLite - SQL Executor
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

//! SQL execution pipeline.
//! Chains parser -> planner -> btree -> wal -> transaction to execute SQL statements.

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};

use super::connection::{ConnectionId, ConnectionPool};
use super::crypto::DbCryptoLayer;
use super::error::{ColumnInfo, SqliteError, Value};
use super::parser::{self, Expr, SqlNode};
use super::planner::{self, PlanNode, QueryPlanner};
use super::result_set::{ResultSet, ResultSetBuilder};
use super::transaction::{TransactionManager, TransactionState};
use alloc::vec;

/// Prepared statement
#[derive(Debug, Clone)]
pub struct PreparedStatement {
    /// Statement ID
    pub id: super::error::StatementId,
    /// Original SQL text
    pub sql: String,
    /// Parsed AST
    pub ast: SqlNode,
    /// Cached query plan
    pub plan: Option<PlanNode>,
    /// Bound parameter values
    pub bound_params: Vec<Value>,
    /// Owning connection ID
    pub conn_id: ConnectionId,
}

/// SQL executor tying together all SQLite components
pub struct SqlExecutor {
    /// Connection pool
    pub connections: ConnectionPool,
    /// Transaction manager
    pub transactions: TransactionManager,
    /// Query planner
    pub planner: QueryPlanner,
    /// Database encryption layer
    pub crypto: DbCryptoLayer,
    /// Prepared statements indexed by StatementId
    prepared_statements: BTreeMap<u64, PreparedStatement>,
    /// Next statement ID
    next_stmt_id: AtomicU64,
    /// Total queries executed
    total_executed: AtomicU64,
}

/// Maximum number of prepared statements
const MAX_PREPARED_STATEMENTS: usize = 1024;

impl SqlExecutor {
    /// Create a new SQL executor
    pub fn new() -> Self {
        SqlExecutor {
            connections: ConnectionPool::new(),
            transactions: TransactionManager::new(),
            planner: QueryPlanner::new(),
            crypto: DbCryptoLayer::new(),
            prepared_statements: BTreeMap::new(),
            next_stmt_id: AtomicU64::new(1),
            total_executed: AtomicU64::new(0),
        }
    }

    /// Open a new database connection
    pub fn open(
        &mut self,
        db_path: &str,
        pid: u32,
        uid: u32,
        read_only: bool,
        encrypted: bool,
    ) -> Result<ConnectionId, SqliteError> {
        self.connections.open(db_path.to_string(), pid, uid, read_only, encrypted)
    }

    /// Close a database connection
    pub fn close(&mut self, conn_id: ConnectionId) -> Result<(), SqliteError> {
        if self.transactions.is_active(conn_id) {
            // Note: proper rollback requires WalManager and Pager refs
        }
        let stmt_ids: Vec<u64> = self
            .prepared_statements
            .iter()
            .filter(|(_, stmt)| stmt.conn_id == conn_id)
            .map(|(id, _)| *id)
            .collect();
        for id in stmt_ids {
            self.prepared_statements.remove(&id);
        }
        self.connections.close(conn_id)
    }

    /// Execute a SQL statement
    pub fn execute(
        &mut self,
        conn_id: ConnectionId,
        sql: &str,
    ) -> Result<ResultSet, SqliteError> {
        if self.connections.get(conn_id).is_none() {
            return Err(SqliteError::InvalidConnection);
        }

        let ast = parser::parse_sql(sql)?;

        match &ast {
            SqlNode::BeginTransaction(stmt) => {
                return self.execute_begin(conn_id, stmt.tx_type);
            }
            SqlNode::CommitTransaction => {
                return self.execute_commit(conn_id);
            }
            SqlNode::RollbackTransaction => {
                return self.execute_rollback(conn_id);
            }
            _ => {}
        }

        let plan = self.planner.plan(&ast)?;
        let result = self.execute_plan(conn_id, &plan)?;
        self.total_executed.fetch_add(1, Ordering::Relaxed);
        Ok(result)
    }

    /// Prepare a SQL statement (parse + plan, but do not execute)
    pub fn prepare(
        &mut self,
        conn_id: ConnectionId,
        sql: &str,
    ) -> Result<super::error::StatementId, SqliteError> {
        if self.connections.get(conn_id).is_none() {
            return Err(SqliteError::InvalidConnection);
        }
        if self.prepared_statements.len() >= MAX_PREPARED_STATEMENTS {
            return Err(SqliteError::Busy);
        }

        let ast = parser::parse_sql(sql)?;
        let plan = self.planner.plan(&ast).ok();
        let stmt_id = super::error::StatementId(self.next_stmt_id.fetch_add(1, Ordering::Relaxed));

        let stmt = PreparedStatement {
            id: stmt_id,
            sql: sql.to_string(),
            ast,
            plan,
            bound_params: Vec::new(),
            conn_id,
        };

        self.prepared_statements.insert(stmt_id.0, stmt);
        Ok(stmt_id)
    }

    /// Bind a parameter to a prepared statement
    pub fn bind(
        &mut self,
        stmt_id: super::error::StatementId,
        param_index: usize,
        value: Value,
    ) -> Result<(), SqliteError> {
        let stmt = self
            .prepared_statements
            .get_mut(&stmt_id.0)
            .ok_or(SqliteError::InvalidConnection)?;

        while stmt.bound_params.len() <= param_index {
            stmt.bound_params.push(Value::Null);
        }
        stmt.bound_params[param_index] = value;
        Ok(())
    }

    /// Execute a prepared statement with bound parameters
    pub fn bind_execute(
        &mut self,
        stmt_id: super::error::StatementId,
    ) -> Result<ResultSet, SqliteError> {
        let stmt = self
            .prepared_statements
            .get(&stmt_id.0)
            .ok_or(SqliteError::InvalidConnection)?;

        let conn_id = stmt.conn_id;

        match &stmt.ast {
            SqlNode::BeginTransaction(tx_stmt) => {
                return self.execute_begin(conn_id, tx_stmt.tx_type);
            }
            SqlNode::CommitTransaction => {
                return self.execute_commit(conn_id);
            }
            SqlNode::RollbackTransaction => {
                return self.execute_rollback(conn_id);
            }
            _ => {}
        }

        let plan = if let Some(ref plan) = stmt.plan {
            plan.clone()
        } else {
            self.planner.plan(&stmt.ast)?
        };

        self.execute_plan(conn_id, &plan)
    }

    /// Execute BEGIN TRANSACTION
    fn execute_begin(
        &mut self,
        conn_id: ConnectionId,
        tx_type: super::parser::TxType,
    ) -> Result<ResultSet, SqliteError> {
        if let Some(conn) = self.connections.get_mut(conn_id) {
            conn.tx_state = match tx_type {
                super::parser::TxType::Deferred => TransactionState::Deferred,
                super::parser::TxType::Immediate => TransactionState::Immediate,
                super::parser::TxType::Exclusive => TransactionState::Exclusive,
            };
        }
        Ok(ResultSet::for_ddl())
    }

    /// Execute COMMIT
    fn execute_commit(&mut self, conn_id: ConnectionId) -> Result<ResultSet, SqliteError> {
        if let Some(conn) = self.connections.get_mut(conn_id) {
            if conn.tx_state == TransactionState::None {
                return Err(SqliteError::NoActiveTransaction);
            }
            conn.tx_state = TransactionState::None;
        }
        Ok(ResultSet::for_ddl())
    }

    /// Execute ROLLBACK
    fn execute_rollback(&mut self, conn_id: ConnectionId) -> Result<ResultSet, SqliteError> {
        if let Some(conn) = self.connections.get_mut(conn_id) {
            if conn.tx_state == TransactionState::None {
                return Err(SqliteError::NoActiveTransaction);
            }
            conn.tx_state = TransactionState::None;
        }
        Ok(ResultSet::for_ddl())
    }

    /// Execute a query plan
    fn execute_plan(
        &mut self,
        conn_id: ConnectionId,
        plan: &PlanNode,
    ) -> Result<ResultSet, SqliteError> {
        match plan {
            PlanNode::TableScan { table, .. } => self.execute_table_scan(table),
            PlanNode::IndexScan { table, .. } => self.execute_table_scan(table),
            PlanNode::Insert { table, values } => self.execute_insert(conn_id, table, values),
            PlanNode::Update { table, assignments, predicate } => {
                self.execute_update(conn_id, table, assignments, predicate)
            }
            PlanNode::Delete { table, predicate } => {
                self.execute_delete(conn_id, table, predicate)
            }
            PlanNode::CreateBTree { table } => self.execute_create_table(table),
            PlanNode::DropBTree { table } => self.execute_drop_table(table),
            PlanNode::CreateIndex { index_name, table, unique, columns } => {
                self.execute_create_index(index_name, table, *unique, columns)
            }
            PlanNode::Filter { child, .. } => self.execute_plan(conn_id, child),
            PlanNode::Project { child, columns, .. } => {
                let result = self.execute_plan(conn_id, child)?;
                self.project_result(result, columns)
            }
            PlanNode::Sort { child, .. } => self.execute_plan(conn_id, child),
            PlanNode::Aggregate { child, .. } => self.execute_plan(conn_id, child),
            PlanNode::Limit { child, .. } => self.execute_plan(conn_id, child),
            PlanNode::NestedLoopJoin { left, right, .. } => {
                // Execute both sides of the join
                let _left_result = self.execute_plan(conn_id, left)?;
                let _right_result = self.execute_plan(conn_id, right)?;
                // TODO: Implement join logic
                Ok(ResultSet::new(Vec::new()))
            }
        }
    }

    /// Execute a table scan (placeholder)
    fn execute_table_scan(&self, table: &str) -> Result<ResultSet, SqliteError> {
        // In a full implementation, this would:
        // 1. Open the B-Tree for the table
        // 2. Iterate over all cells in the B-Tree
        // 3. Build a ResultSet from the cell payloads
        let columns = vec![ColumnInfo {
            name: "rowid".to_string(),
            column_type: super::error::ColumnType::Integer,
            table_name: table.to_string(),
            nullable: false,
        }];
        Ok(ResultSet::new(columns))
    }

    /// Execute an INSERT (placeholder)
    fn execute_insert(
        &mut self,
        _conn_id: ConnectionId,
        table: &str,
        _values: &[Expr],
    ) -> Result<ResultSet, SqliteError> {
        // In a full implementation, this would:
        // 1. Allocate a new row ID from the B-Tree
        // 2. Serialize the values into a cell payload
        // 3. Insert the cell into the B-Tree
        // 4. Write the modified page to the WAL
        Ok(ResultSet::for_dml(1, 1))
    }

    /// Execute an UPDATE (placeholder)
    fn execute_update(
        &mut self,
        _conn_id: ConnectionId,
        _table: &str,
        _assignments: &[(String, Expr)],
        _predicate: &Option<Expr>,
    ) -> Result<ResultSet, SqliteError> {
        Ok(ResultSet::for_dml(0, 0))
    }

    /// Execute a DELETE (placeholder)
    fn execute_delete(
        &mut self,
        _conn_id: ConnectionId,
        _table: &str,
        _predicate: &Option<Expr>,
    ) -> Result<ResultSet, SqliteError> {
        Ok(ResultSet::for_dml(0, 0))
    }

    /// Execute CREATE TABLE (placeholder)
    fn execute_create_table(&self, table: &str) -> Result<ResultSet, SqliteError> {
        // In a full implementation, this would:
        // 1. Allocate a new root page for the B-Tree
        // 2. Write the schema entry into the sqlite_master table
        // 3. Sync the WAL
        let _ = table;
        Ok(ResultSet::for_ddl())
    }

    /// Execute DROP TABLE (placeholder)
    fn execute_drop_table(&self, table: &str) -> Result<ResultSet, SqliteError> {
        // In a full implementation, this would:
        // 1. Remove the B-Tree pages (add to freelist)
        // 2. Remove the schema entry from sqlite_master
        // 3. Sync the WAL
        let _ = table;
        Ok(ResultSet::for_ddl())
    }

    /// Execute CREATE INDEX (placeholder)
    fn execute_create_index(
        &self,
        index_name: &str,
        table: &str,
        unique: bool,
        columns: &[String],
    ) -> Result<ResultSet, SqliteError> {
        // In a full implementation, this would:
        // 1. Allocate a new root page for the index B-Tree
        // 2. Populate the index by scanning the table B-Tree
        // 3. Write the schema entry into sqlite_master
        let _ = (index_name, table, unique, columns);
        Ok(ResultSet::for_ddl())
    }

    /// Apply column projection to a result set
    fn project_result(
        &self,
        result: ResultSet,
        columns: &[Expr],
    ) -> Result<ResultSet, SqliteError> {
        // In a full implementation, this would filter the columns
        // of each row according to the projection expressions.
        let _ = columns;
        Ok(result)
    }

    /// Begin a transaction
    pub fn begin_transaction(
        &mut self,
        conn_id: ConnectionId,
    ) -> Result<(), SqliteError> {
        if let Some(conn) = self.connections.get_mut(conn_id) {
            conn.tx_state = TransactionState::Deferred;
        }
        Ok(())
    }

    /// Commit the current transaction
    pub fn commit_transaction(
        &mut self,
        conn_id: ConnectionId,
    ) -> Result<(), SqliteError> {
        if let Some(conn) = self.connections.get_mut(conn_id) {
            if conn.tx_state == TransactionState::None {
                return Err(SqliteError::NoActiveTransaction);
            }
            conn.tx_state = TransactionState::None;
        }
        Ok(())
    }

    /// Rollback the current transaction
    pub fn rollback_transaction(
        &mut self,
        conn_id: ConnectionId,
    ) -> Result<(), SqliteError> {
        if let Some(conn) = self.connections.get_mut(conn_id) {
            if conn.tx_state == TransactionState::None {
                return Err(SqliteError::NoActiveTransaction);
            }
            conn.tx_state = TransactionState::None;
        }
        Ok(())
    }
}
