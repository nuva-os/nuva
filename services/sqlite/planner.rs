/*
 * Nuva OS - SystemService - SQLite - Query Planner
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

//! Query plan generation and optimization.
//! Converts parsed AST into executable query plans with index scan selection.

use alloc::string::String;
use alloc::vec::Vec;

use super::error::SqliteError;
use super::parser::{
    DeleteStmt, Expr, InsertStmt, JoinClause, JoinType, Ordering, OrderByItem, SelectStmt,
    SqlNode, UpdateStmt,
};

/// Query plan node
#[derive(Debug, Clone)]
pub enum PlanNode {
    /// Full table scan
    TableScan {
        /// Table name
        table: String,
        /// Estimated row count
        estimated_rows: u64,
    },
    /// Index scan using a specific index
    IndexScan {
        /// Table name
        table: String,
        /// Index name
        index_name: String,
        /// Whether this is a unique index scan (at most 1 row)
        unique: bool,
        /// Estimated row count
        estimated_rows: u64,
    },
    /// Nested loop join
    NestedLoopJoin {
        /// Left child plan
        left: Box<PlanNode>,
        /// Right child plan
        right: Box<PlanNode>,
        /// Join type
        join_type: JoinType,
        /// Join key columns from left table
        left_keys: Vec<String>,
        /// Join key columns from right table
        right_keys: Vec<String>,
    },
    /// Sort operator
    Sort {
        /// Child plan
        child: Box<PlanNode>,
        /// Sort keys
        keys: Vec<OrderByItem>,
    },
    /// Filter operator (apply WHERE clause after scan)
    Filter {
        /// Child plan
        child: Box<PlanNode>,
        /// Filter expression
        predicate: Expr,
    },
    /// Project operator (select specific columns)
    Project {
        /// Child plan
        child: Box<PlanNode>,
        /// Projected column expressions
        columns: Vec<Expr>,
        /// Whether DISTINCT is applied
        distinct: bool,
    },
    /// Aggregate operator
    Aggregate {
        /// Child plan
        child: Box<PlanNode>,
        /// GROUP BY keys
        group_keys: Vec<Expr>,
        /// HAVING filter
        having: Option<Expr>,
    },
    /// Limit/Offset operator
    Limit {
        /// Child plan
        child: Box<PlanNode>,
        /// Maximum rows to return
        limit: u64,
        /// Number of rows to skip
        offset: u64,
    },
    /// Insert rows into a table
    Insert {
        /// Table name
        table: String,
        /// Values to insert
        values: Vec<Expr>,
    },
    /// Update rows in a table
    Update {
        /// Table name
        table: String,
        /// Column assignments
        assignments: Vec<(String, Expr)>,
        /// Filter for rows to update
        predicate: Option<Expr>,
    },
    /// Delete rows from a table
    Delete {
        /// Table name
        table: String,
        /// Filter for rows to delete
        predicate: Option<Expr>,
    },
    /// Create a new B-Tree for a table
    CreateBTree {
        /// Table name
        table: String,
    },
    /// Drop a B-Tree for a table
    DropBTree {
        /// Table name
        table: String,
    },
    /// Create a new index B-Tree
    CreateIndex {
        /// Index name
        index_name: String,
        /// Table name
        table: String,
        /// Whether the index is unique
        unique: bool,
        /// Indexed columns
        columns: Vec<String>,
    },
}

/// Statistics about a table (used for cost estimation)
#[derive(Debug, Clone, Copy)]
pub struct TableStats {
    /// Estimated number of rows
    pub row_count: u64,
    /// Number of pages in the B-Tree
    pub page_count: u32,
    /// Whether the table has a primary key index
    pub has_pk_index: bool,
}

/// Index metadata for planner
#[derive(Debug, Clone)]
pub struct IndexInfo {
    /// Index name
    pub name: String,
    /// Table name
    pub table_name: String,
    /// Whether the index is unique
    pub unique: bool,
    /// Indexed column names
    pub columns: Vec<String>,
    /// Estimated number of rows in the index
    pub row_count: u64,
}

/// Query planner and optimizer
pub struct QueryPlanner {
    /// Table statistics cache
    table_stats: alloc::collections::BTreeMap<String, TableStats>,
    /// Available indexes
    indexes: alloc::collections::BTreeMap<String, Vec<IndexInfo>>,
}

/// Default estimated row count for tables without statistics
const DEFAULT_ESTIMATED_ROWS: u64 = 1000;

impl QueryPlanner {
    /// Create a new query planner
    pub fn new() -> Self {
        QueryPlanner {
            table_stats: alloc::collections::BTreeMap::new(),
            indexes: alloc::collections::BTreeMap::new(),
        }
    }

    /// Generate an execution plan from a parsed SQL AST
    pub fn plan(&self, ast: &SqlNode) -> Result<PlanNode, SqliteError> {
        match ast {
            SqlNode::CreateTable(stmt) => Ok(PlanNode::CreateBTree {
                table: stmt.table_name.clone(),
            }),
            SqlNode::DropTable(stmt) => Ok(PlanNode::DropBTree {
                table: stmt.table_name.clone(),
            }),
            SqlNode::CreateIndex(stmt) => Ok(PlanNode::CreateIndex {
                index_name: stmt.index_name.clone(),
                table: stmt.table_name.clone(),
                unique: stmt.unique,
                columns: stmt.columns.clone(),
            }),
            SqlNode::Select(stmt) => self.plan_select(stmt),
            SqlNode::Insert(stmt) => self.plan_insert(stmt),
            SqlNode::Update(stmt) => self.plan_update(stmt),
            SqlNode::Delete(stmt) => self.plan_delete(stmt),
            SqlNode::BeginTransaction(_) | SqlNode::CommitTransaction | SqlNode::RollbackTransaction => {
                Err(SqliteError::SyntaxError)
            }
        }
    }

    /// Plan a SELECT statement
    fn plan_select(&self, stmt: &SelectStmt) -> Result<PlanNode, SqliteError> {
        // Build the scan plan for the first FROM table
        let mut plan = if stmt.from_tables.is_empty() {
            PlanNode::TableScan {
                table: String::new(),
                estimated_rows: 1,
            }
        } else {
            let table = &stmt.from_tables[0];
            self.choose_scan_plan(&table.name, &stmt.where_clause)
        };

        // Apply JOINs
        for join in &stmt.joins {
            let right_plan = self.choose_scan_plan(&join.table.name, &join.condition);
            let (left_keys, right_keys) = self.extract_join_keys(&join.condition);
            plan = PlanNode::NestedLoopJoin {
                left: Box::new(plan),
                right: Box::new(right_plan),
                join_type: join.join_type,
                left_keys,
                right_keys,
            };
        }

        // Apply WHERE filter
        if let Some(predicate) = &stmt.where_clause {
            // Only add explicit filter if index scan did not cover the predicate
            if !self.predicate_covered_by_index(predicate) {
                plan = PlanNode::Filter {
                    child: Box::new(plan),
                    predicate: predicate.clone(),
                };
            }
        }

        // Apply GROUP BY / HAVING
        if !stmt.group_by.is_empty() || stmt.having.is_some() {
            plan = PlanNode::Aggregate {
                child: Box::new(plan),
                group_keys: stmt.group_by.clone(),
                having: stmt.having.clone(),
            };
        }

        // Apply projection
        plan = PlanNode::Project {
            child: Box::new(plan),
            columns: stmt.columns.clone(),
            distinct: stmt.distinct,
        };

        // Apply ORDER BY
        if !stmt.order_by.is_empty() {
            plan = PlanNode::Sort {
                child: Box::new(plan),
                keys: stmt.order_by.clone(),
            };
        }

        // Apply LIMIT / OFFSET
        if stmt.limit.is_some() || stmt.offset.is_some() {
            let limit = self.expr_to_u64(stmt.limit.as_ref(), u64::MAX);
            let offset = self.expr_to_u64(stmt.offset.as_ref(), 0);
            plan = PlanNode::Limit {
                child: Box::new(plan),
                limit,
                offset,
            };
        }

        Ok(plan)
    }

    /// Plan an INSERT statement
    fn plan_insert(&self, stmt: &InsertStmt) -> Result<PlanNode, SqliteError> {
        Ok(PlanNode::Insert {
            table: stmt.table_name.clone(),
            values: stmt.values.clone(),
        })
    }

    /// Plan an UPDATE statement
    fn plan_update(&self, stmt: &UpdateStmt) -> Result<PlanNode, SqliteError> {
        Ok(PlanNode::Update {
            table: stmt.table_name.clone(),
            assignments: stmt.assignments.clone(),
            predicate: stmt.where_clause.clone(),
        })
    }

    /// Plan a DELETE statement
    fn plan_delete(&self, stmt: &DeleteStmt) -> Result<PlanNode, SqliteError> {
        Ok(PlanNode::Delete {
            table: stmt.table_name.clone(),
            predicate: stmt.where_clause.clone(),
        })
    }

    /// Choose between table scan and index scan
    fn choose_scan_plan(&self, table: &str, _predicate: &Option<Expr>) -> PlanNode {
        let stats = self
            .table_stats
            .get(table)
            .copied()
            .unwrap_or(TableStats {
                row_count: DEFAULT_ESTIMATED_ROWS,
                page_count: 10,
                has_pk_index: false,
            });

        // TODO: Analyze predicate to find matching index and estimate selectivity.
        // For now, always use table scan.

        PlanNode::TableScan {
            table: table.to_string(),
            estimated_rows: stats.row_count,
        }
    }

    /// Extract join key column names from a join condition
    fn extract_join_keys(&self, condition: &Option<Expr>) -> (Vec<String>, Vec<String>) {
        if let Some(Expr::BinaryOp { left, op, right }) = condition {
            if op == "=" {
                if let (Expr::ColumnRef { column: lc, .. }, Expr::ColumnRef { column: rc, .. }) =
                    (left.as_ref(), right.as_ref())
                {
                    return (vec![lc.clone()], vec![rc.clone()]);
                }
            }
        }
        (Vec::new(), Vec::new())
    }

    /// Check if a predicate is fully covered by an available index
    fn predicate_covered_by_index(&self, _predicate: &Expr) -> bool {
        // TODO: Implement index coverage analysis
        false
    }

    /// Convert an optional expression to a u64 value
    fn expr_to_u64(&self, expr: Option<&Expr>, default: u64) -> u64 {
        match expr {
            Some(Expr::Literal(super::error::Value::Integer(n))) => *n as u64,
            Some(Expr::Literal(super::error::Value::I64(n))) => *n as u64,
            Some(Expr::Literal(super::error::Value::Real(f))) => *f as u64,
            _ => default,
        }
    }

    /// Register table statistics
    pub fn update_table_stats(&mut self, table: &str, stats: TableStats) {
        self.table_stats.insert(table.to_string(), stats);
    }

    /// Register an index
    pub fn add_index(&mut self, table: &str, index: IndexInfo) {
        self.indexes
            .entry(table.to_string())
            .or_insert_with(Vec::new)
            .push(index);
    }

    /// Remove all indexes for a table
    pub fn remove_indexes(&mut self, table: &str) {
        self.indexes.remove(table);
    }
}
