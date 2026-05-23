/*
 * Nuva OS - SystemService - SQLite - SQL Parser
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

//! SQL syntax parser producing an AST (SqlNode) from input SQL text.
//! Supports DDL (CREATE TABLE, DROP TABLE), DML (SELECT, INSERT, UPDATE, DELETE),
//! transaction control (BEGIN, COMMIT, ROLLBACK), and index creation.

use alloc::string::String;
use alloc::vec::Vec;

use super::error::{SqliteError, Value};

/// Parsed SQL statement AST root
#[derive(Debug, Clone)]
pub enum SqlNode {
    /// CREATE TABLE statement
    CreateTable(CreateTableStmt),
    /// DROP TABLE statement
    DropTable(DropTableStmt),
    /// CREATE INDEX statement
    CreateIndex(CreateIndexStmt),
    /// SELECT statement
    Select(SelectStmt),
    /// INSERT statement
    Insert(InsertStmt),
    /// UPDATE statement
    Update(UpdateStmt),
    /// DELETE statement
    Delete(DeleteStmt),
    /// BEGIN TRANSACTION
    BeginTransaction(BeginStmt),
    /// COMMIT TRANSACTION
    CommitTransaction,
    /// ROLLBACK TRANSACTION
    RollbackTransaction,
}

/// Column definition in CREATE TABLE
#[derive(Debug, Clone)]
pub struct ColumnDef {
    /// Column name
    pub name: String,
    /// Column type name (e.g. "INTEGER", "TEXT")
    pub type_name: String,
    /// Whether the column has NOT NULL constraint
    pub not_null: bool,
    /// Whether the column is PRIMARY KEY
    pub primary_key: bool,
    /// Default value expression, if any
    pub default_value: Option<Value>,
}

/// CREATE TABLE statement
#[derive(Debug, Clone)]
pub struct CreateTableStmt {
    /// Table name
    pub table_name: String,
    /// If NOT EXISTS was specified
    pub if_not_exists: bool,
    /// Column definitions
    pub columns: Vec<ColumnDef>,
}

/// DROP TABLE statement
#[derive(Debug, Clone)]
pub struct DropTableStmt {
    /// Table name
    pub table_name: String,
    /// If EXISTS was specified
    pub if_exists: bool,
}

/// CREATE INDEX statement
#[derive(Debug, Clone)]
pub struct CreateIndexStmt {
    /// Index name
    pub index_name: String,
    /// Table name
    pub table_name: String,
    /// If NOT EXISTS was specified
    pub if_not_exists: bool,
    /// Whether this is a UNIQUE index
    pub unique: bool,
    /// Indexed column names
    pub columns: Vec<String>,
}

/// Expression in SQL
#[derive(Debug, Clone)]
pub enum Expr {
    /// Literal value
    Literal(Value),
    /// Column reference: [table.]column
    ColumnRef {
        /// Optional table qualifier
        table: Option<String>,
        /// Column name
        column: String,
    },
    /// Star wildcard (SELECT *)
    Star,
    /// Binary operation
    BinaryOp {
        /// Left operand
        left: Box<Expr>,
        /// Operator string
        op: String,
        /// Right operand
        right: Box<Expr>,
    },
    /// Unary operation
    UnaryOp {
        /// Operator string
        op: String,
        /// Operand
        operand: Box<Expr>,
    },
    /// Function call
    Function {
        /// Function name
        name: String,
        /// Arguments
        args: Vec<Expr>,
    },
    /// IS NULL / IS NOT NULL
    IsNull {
        /// Expression to test
        expr: Box<Expr>,
        /// True for IS NULL, false for IS NOT NULL
        is_null: bool,
    },
}

/// Join type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JoinType {
    /// INNER JOIN
    Inner,
    /// LEFT OUTER JOIN
    LeftOuter,
    /// CROSS JOIN
    Cross,
}

/// Table reference in FROM clause
#[derive(Debug, Clone)]
pub struct TableRef {
    /// Table name
    pub name: String,
    /// Optional alias
    pub alias: Option<String>,
}

/// Join clause
#[derive(Debug, Clone)]
pub struct JoinClause {
    /// Join type
    pub join_type: JoinType,
    /// Right table reference
    pub table: TableRef,
    /// Join condition (ON clause)
    pub condition: Option<Expr>,
}

/// Ordering direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ordering {
    /// Ascending
    Asc,
    /// Descending
    Desc,
}

/// ORDER BY item
#[derive(Debug, Clone)]
pub struct OrderByItem {
    /// Expression to order by
    pub expr: Expr,
    /// Ordering direction
    pub ordering: Ordering,
}

/// SELECT statement
#[derive(Debug, Clone)]
pub struct SelectStmt {
    /// Whether DISTINCT was specified
    pub distinct: bool,
    /// Selected expressions
    pub columns: Vec<Expr>,
    /// FROM clause tables
    pub from_tables: Vec<TableRef>,
    /// JOIN clauses
    pub joins: Vec<JoinClause>,
    /// WHERE clause
    pub where_clause: Option<Expr>,
    /// GROUP BY expressions
    pub group_by: Vec<Expr>,
    /// HAVING clause
    pub having: Option<Expr>,
    /// ORDER BY items
    pub order_by: Vec<OrderByItem>,
    /// LIMIT count
    pub limit: Option<Expr>,
    /// OFFSET
    pub offset: Option<Expr>,
}

/// INSERT statement
#[derive(Debug, Clone)]
pub struct InsertStmt {
    /// Table name
    pub table_name: String,
    /// Column names (empty means all columns in order)
    pub columns: Vec<String>,
    /// Values to insert (one row)
    pub values: Vec<Expr>,
}

/// UPDATE statement
#[derive(Debug, Clone)]
pub struct UpdateStmt {
    /// Table name
    pub table_name: String,
    /// Column assignments
    pub assignments: Vec<(String, Expr)>,
    /// WHERE clause
    pub where_clause: Option<Expr>,
}

/// DELETE statement
#[derive(Debug, Clone)]
pub struct DeleteStmt {
    /// Table name
    pub table_name: String,
    /// WHERE clause
    pub where_clause: Option<Expr>,
}

/// BEGIN TRANSACTION statement
#[derive(Debug, Clone)]
pub struct BeginStmt {
    /// Transaction type: DEFERRED, IMMEDIATE, or EXCLUSIVE
    pub tx_type: TxType,
}

/// Transaction type for BEGIN statement
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TxType {
    /// DEFERRED: locks acquired on first read/write
    Deferred,
    /// IMMEDIATE: write lock acquired immediately
    Immediate,
    /// EXCLUSIVE: exclusive lock acquired immediately
    Exclusive,
}

/// Simple recursive-descent SQL parser
pub struct SqlParser {
    /// Input SQL bytes
    input: Vec<u8>,
    /// Current position in input
    pos: usize,
}

impl SqlParser {
    /// Create a new parser for the given SQL string
    pub fn new(sql: &str) -> Self {
        SqlParser {
            input: sql.as_bytes().to_vec(),
            pos: 0,
        }
    }

    /// Parse the input SQL and return an AST
    pub fn parse(&mut self) -> Result<SqlNode, SqliteError> {
        self.skip_whitespace();
        let word = self.peek_keyword()?;

        match word.as_str() {
            "CREATE" => self.parse_create(),
            "DROP" => self.parse_drop(),
            "SELECT" => self.parse_select(),
            "INSERT" => self.parse_insert(),
            "UPDATE" => self.parse_update(),
            "DELETE" => self.parse_delete(),
            "BEGIN" => self.parse_begin(),
            "COMMIT" | "END" => {
                self.advance_word();
                Ok(SqlNode::CommitTransaction)
            }
            "ROLLBACK" => {
                self.advance_word();
                Ok(SqlNode::RollbackTransaction)
            }
            _ => Err(SqliteError::SyntaxError),
        }
    }

    /// Skip whitespace characters
    fn skip_whitespace(&mut self) {
        while self.pos < self.input.len() {
            let ch = self.input[self.pos];
            if ch == b' ' || ch == b'\t' || ch == b'\n' || ch == b'\r' {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    /// Peek at the next keyword without consuming it
    fn peek_keyword(&mut self) -> Result<String, SqliteError> {
        let saved = self.pos;
        self.skip_whitespace();
        let mut word = String::new();
        while self.pos < self.input.len() {
            let ch = self.input[self.pos];
            if ch.is_ascii_alphabetic() || ch == b'_' {
                word.push(ch as char);
                self.pos += 1;
            } else {
                break;
            }
        }
        self.pos = saved;
        if word.is_empty() {
            Err(SqliteError::SyntaxError)
        } else {
            Ok(word)
        }
    }

    /// Advance past the current keyword
    fn advance_word(&mut self) {
        self.skip_whitespace();
        while self.pos < self.input.len() {
            let ch = self.input[self.pos];
            if ch.is_ascii_alphabetic() || ch == b'_' {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    /// Read an identifier from the input
    fn read_identifier(&mut self) -> Result<String, SqliteError> {
        self.skip_whitespace();
        let mut ident = String::new();
        while self.pos < self.input.len() {
            let ch = self.input[self.pos];
            if ch.is_ascii_alphanumeric() || ch == b'_' {
                ident.push(ch as char);
                self.pos += 1;
            } else {
                break;
            }
        }
        if ident.is_empty() {
            Err(SqliteError::SyntaxError)
        } else {
            Ok(ident)
        }
    }

    /// Expect and consume a specific keyword
    fn expect_keyword(&mut self, keyword: &str) -> Result<(), SqliteError> {
        self.skip_whitespace();
        let word = self.read_identifier()?;
        if word.eq_ignore_ascii_case(keyword) {
            Ok(())
        } else {
            Err(SqliteError::SyntaxError)
        }
    }

    /// Parse a CREATE statement (TABLE or INDEX)
    fn parse_create(&mut self) -> Result<SqlNode, SqliteError> {
        self.advance_word(); // skip CREATE
        self.skip_whitespace();
        let next = self.peek_keyword()?;

        match next.as_str() {
            "TABLE" => self.parse_create_table(),
            "INDEX" => self.parse_create_index(),
            "UNIQUE" => {
                self.advance_word(); // skip UNIQUE
                self.expect_keyword("INDEX")?;
                self.parse_create_index_inner(true)
            }
            _ => Err(SqliteError::SyntaxError),
        }
    }

    /// Parse CREATE TABLE
    fn parse_create_table(&mut self) -> Result<SqlNode, SqliteError> {
        self.advance_word(); // skip TABLE

        let mut if_not_exists = false;
        self.skip_whitespace();
        let peek = self.peek_keyword()?;
        if peek.eq_ignore_ascii_case("IF") {
            self.advance_word();
            self.expect_keyword("NOT")?;
            self.expect_keyword("EXISTS")?;
            if_not_exists = true;
        }

        let table_name = self.read_identifier()?;
        self.skip_whitespace();

        // Expect '('
        if self.pos >= self.input.len() || self.input[self.pos] != b'(' {
            return Err(SqliteError::SyntaxError);
        }
        self.pos += 1;

        let mut columns = Vec::new();
        loop {
            self.skip_whitespace();
            if self.pos < self.input.len() && self.input[self.pos] == b')' {
                self.pos += 1;
                break;
            }

            if !columns.is_empty() {
                if self.pos < self.input.len() && self.input[self.pos] == b',' {
                    self.pos += 1;
                } else {
                    return Err(SqliteError::SyntaxError);
                }
            }

            let col_name = self.read_identifier()?;
            let type_name = self.read_identifier().unwrap_or_else(|_| String::new());

            let mut not_null = false;
            let mut primary_key = false;
            let mut default_value: Option<Value> = None;

            // Parse column constraints
            loop {
                self.skip_whitespace();
                if self.pos >= self.input.len() {
                    break;
                }
                let ch = self.input[self.pos];
                if ch == b',' || ch == b')' {
                    break;
                }
                let constraint = self.read_identifier()?;
                if constraint.eq_ignore_ascii_case("NOT") {
                    self.expect_keyword("NULL")?;
                    not_null = true;
                } else if constraint.eq_ignore_ascii_case("PRIMARY") {
                    self.expect_keyword("KEY")?;
                    primary_key = true;
                } else if constraint.eq_ignore_ascii_case("DEFAULT") {
                    self.skip_whitespace();
                    let val = self.read_literal_value()?;
                    default_value = Some(val);
                } else {
                    // Ignore unknown constraints
                }
            }

            columns.push(ColumnDef {
                name: col_name,
                type_name,
                not_null,
                primary_key,
                default_value,
            });
        }

        Ok(SqlNode::CreateTable(CreateTableStmt {
            table_name,
            if_not_exists,
            columns,
        }))
    }

    /// Parse CREATE INDEX
    fn parse_create_index(&mut self) -> Result<SqlNode, SqliteError> {
        self.advance_word(); // skip INDEX
        self.parse_create_index_inner(false)
    }

    /// Parse CREATE INDEX body (shared by UNIQUE INDEX path)
    fn parse_create_index_inner(&mut self, unique: bool) -> Result<SqlNode, SqliteError> {
        let mut if_not_exists = false;
        self.skip_whitespace();
        let peek = self.peek_keyword()?;
        if peek.eq_ignore_ascii_case("IF") {
            self.advance_word();
            self.expect_keyword("NOT")?;
            self.expect_keyword("EXISTS")?;
            if_not_exists = true;
        }

        let index_name = self.read_identifier()?;
        self.expect_keyword("ON")?;
        let table_name = self.read_identifier()?;

        self.skip_whitespace();
        if self.pos >= self.input.len() || self.input[self.pos] != b'(' {
            return Err(SqliteError::SyntaxError);
        }
        self.pos += 1;

        let mut columns = Vec::new();
        loop {
            self.skip_whitespace();
            if self.pos < self.input.len() && self.input[self.pos] == b')' {
                self.pos += 1;
                break;
            }
            if !columns.is_empty() {
                if self.pos < self.input.len() && self.input[self.pos] == b',' {
                    self.pos += 1;
                } else {
                    return Err(SqliteError::SyntaxError);
                }
            }
            columns.push(self.read_identifier()?);
        }

        Ok(SqlNode::CreateIndex(CreateIndexStmt {
            index_name,
            table_name,
            if_not_exists,
            unique,
            columns,
        }))
    }

    /// Parse DROP statement
    fn parse_drop(&mut self) -> Result<SqlNode, SqliteError> {
        self.advance_word(); // skip DROP
        self.expect_keyword("TABLE")?;

        let mut if_exists = false;
        self.skip_whitespace();
        let peek = self.peek_keyword()?;
        if peek.eq_ignore_ascii_case("IF") {
            self.advance_word();
            self.expect_keyword("EXISTS")?;
            if_exists = true;
        }

        let table_name = self.read_identifier()?;
        Ok(SqlNode::DropTable(DropTableStmt {
            table_name,
            if_exists,
        }))
    }

    /// Parse SELECT statement
    fn parse_select(&mut self) -> Result<SqlNode, SqliteError> {
        self.advance_word(); // skip SELECT

        let mut distinct = false;
        self.skip_whitespace();
        let peek = self.peek_keyword()?;
        if peek.eq_ignore_ascii_case("DISTINCT") {
            self.advance_word();
            distinct = true;
        } else if peek.eq_ignore_ascii_case("ALL") {
            self.advance_word();
        }

        let columns = self.parse_expr_list()?;

        let mut from_tables = Vec::new();
        let mut joins = Vec::new();
        let mut where_clause = None;
        let mut group_by = Vec::new();
        let mut having = None;
        let mut order_by = Vec::new();
        let mut limit = None;
        let mut offset = None;

        self.skip_whitespace();
        if self.pos < self.input.len() {
            let peek = self.peek_keyword()?;
            if peek.eq_ignore_ascii_case("FROM") {
                self.advance_word();
                from_tables = self.parse_table_refs()?;

                // Parse JOINs
                loop {
                    self.skip_whitespace();
                    if self.pos >= self.input.len() {
                        break;
                    }
                    let jpeek = self.peek_keyword()?;
                    if jpeek.eq_ignore_ascii_case("JOIN") || jpeek.eq_ignore_ascii_case("INNER") || jpeek.eq_ignore_ascii_case("LEFT") || jpeek.eq_ignore_ascii_case("CROSS") {
                        joins.push(self.parse_join()?);
                    } else {
                        break;
                    }
                }
            }
        }

        self.skip_whitespace();
        if self.pos < self.input.len() {
            let peek = self.peek_keyword()?;
            if peek.eq_ignore_ascii_case("WHERE") {
                self.advance_word();
                where_clause = Some(self.parse_expr()?);
            }
        }

        self.skip_whitespace();
        if self.pos < self.input.len() {
            let peek = self.peek_keyword()?;
            if peek.eq_ignore_ascii_case("GROUP") {
                self.advance_word();
                self.expect_keyword("BY")?;
                group_by = self.parse_expr_list()?;
            }
        }

        self.skip_whitespace();
        if self.pos < self.input.len() {
            let peek = self.peek_keyword()?;
            if peek.eq_ignore_ascii_case("HAVING") {
                self.advance_word();
                having = Some(self.parse_expr()?);
            }
        }

        self.skip_whitespace();
        if self.pos < self.input.len() {
            let peek = self.peek_keyword()?;
            if peek.eq_ignore_ascii_case("ORDER") {
                self.advance_word();
                self.expect_keyword("BY")?;
                order_by = self.parse_order_by_list()?;
            }
        }

        self.skip_whitespace();
        if self.pos < self.input.len() {
            let peek = self.peek_keyword()?;
            if peek.eq_ignore_ascii_case("LIMIT") {
                self.advance_word();
                limit = Some(self.parse_expr()?);
            }
        }

        self.skip_whitespace();
        if self.pos < self.input.len() {
            let peek = self.peek_keyword()?;
            if peek.eq_ignore_ascii_case("OFFSET") {
                self.advance_word();
                offset = Some(self.parse_expr()?);
            }
        }

        Ok(SqlNode::Select(SelectStmt {
            distinct,
            columns,
            from_tables,
            joins,
            where_clause,
            group_by,
            having,
            order_by,
            limit,
            offset,
        }))
    }

    /// Parse INSERT statement
    fn parse_insert(&mut self) -> Result<SqlNode, SqliteError> {
        self.advance_word(); // skip INSERT
        self.expect_keyword("INTO")?;

        let table_name = self.read_identifier()?;

        let mut columns = Vec::new();
        self.skip_whitespace();
        if self.pos < self.input.len() && self.input[self.pos] == b'(' {
            self.pos += 1;
            loop {
                self.skip_whitespace();
                if self.pos < self.input.len() && self.input[self.pos] == b')' {
                    self.pos += 1;
                    break;
                }
                if !columns.is_empty() {
                    if self.pos < self.input.len() && self.input[self.pos] == b',' {
                        self.pos += 1;
                    } else {
                        return Err(SqliteError::SyntaxError);
                    }
                }
                columns.push(self.read_identifier()?);
            }
        }

        self.expect_keyword("VALUES")?;
        self.skip_whitespace();
        if self.pos >= self.input.len() || self.input[self.pos] != b'(' {
            return Err(SqliteError::SyntaxError);
        }
        self.pos += 1;

        let values = self.parse_expr_list_inner()?;

        Ok(SqlNode::Insert(InsertStmt {
            table_name,
            columns,
            values,
        }))
    }

    /// Parse UPDATE statement
    fn parse_update(&mut self) -> Result<SqlNode, SqliteError> {
        self.advance_word(); // skip UPDATE
        let table_name = self.read_identifier()?;
        self.expect_keyword("SET")?;

        let mut assignments = Vec::new();
        loop {
            let col = self.read_identifier()?;
            self.skip_whitespace();
            if self.pos >= self.input.len() || self.input[self.pos] != b'=' {
                return Err(SqliteError::SyntaxError);
            }
            self.pos += 1;
            let val = self.parse_expr()?;
            assignments.push((col, val));

            self.skip_whitespace();
            if self.pos >= self.input.len() || self.input[self.pos] != b',' {
                break;
            }
            self.pos += 1;
        }

        let mut where_clause = None;
        self.skip_whitespace();
        if self.pos < self.input.len() {
            let peek = self.peek_keyword()?;
            if peek.eq_ignore_ascii_case("WHERE") {
                self.advance_word();
                where_clause = Some(self.parse_expr()?);
            }
        }

        Ok(SqlNode::Update(UpdateStmt {
            table_name,
            assignments,
            where_clause,
        }))
    }

    /// Parse DELETE statement
    fn parse_delete(&mut self) -> Result<SqlNode, SqliteError> {
        self.advance_word(); // skip DELETE
        self.expect_keyword("FROM")?;
        let table_name = self.read_identifier()?;

        let mut where_clause = None;
        self.skip_whitespace();
        if self.pos < self.input.len() {
            let peek = self.peek_keyword()?;
            if peek.eq_ignore_ascii_case("WHERE") {
                self.advance_word();
                where_clause = Some(self.parse_expr()?);
            }
        }

        Ok(SqlNode::Delete(DeleteStmt {
            table_name,
            where_clause,
        }))
    }

    /// Parse BEGIN TRANSACTION
    fn parse_begin(&mut self) -> Result<SqlNode, SqliteError> {
        self.advance_word(); // skip BEGIN

        let mut tx_type = TxType::Deferred;
        self.skip_whitespace();
        if self.pos < self.input.len() {
            let peek = self.peek_keyword()?;
            if peek.eq_ignore_ascii_case("DEFERRED") {
                self.advance_word();
                tx_type = TxType::Deferred;
            } else if peek.eq_ignore_ascii_case("IMMEDIATE") {
                self.advance_word();
                tx_type = TxType::Immediate;
            } else if peek.eq_ignore_ascii_case("EXCLUSIVE") {
                self.advance_word();
                tx_type = TxType::Exclusive;
            }
            // Optional TRANSACTION keyword
            self.skip_whitespace();
            if self.pos < self.input.len() {
                let peek2 = self.peek_keyword()?;
                if peek2.eq_ignore_ascii_case("TRANSACTION") {
                    self.advance_word();
                }
            }
        }

        Ok(SqlNode::BeginTransaction(BeginStmt { tx_type }))
    }

    /// Parse a comma-separated expression list (top-level)
    fn parse_expr_list(&mut self) -> Result<Vec<Expr>, SqliteError> {
        // Check for SELECT *
        self.skip_whitespace();
        if self.pos < self.input.len() && self.input[self.pos] == b'*' {
            self.pos += 1;
            return Ok(vec![Expr::Star]);
        }
        self.parse_expr_list_inner()
    }

    /// Parse a comma-separated expression list (within parens)
    fn parse_expr_list_inner(&mut self) -> Result<Vec<Expr>, SqliteError> {
        let mut exprs = Vec::new();
        loop {
            self.skip_whitespace();
            if self.pos < self.input.len() && self.input[self.pos] == b')' {
                self.pos += 1;
                break;
            }
            if !exprs.is_empty() {
                self.skip_whitespace();
                if self.pos < self.input.len() && self.input[self.pos] == b',' {
                    self.pos += 1;
                } else {
                    // End of list (no closing paren) or syntax error
                    break;
                }
            }
            exprs.push(self.parse_expr()?);
        }
        Ok(exprs)
    }

    /// Parse a single expression (simplified: handles literals, identifiers, and simple ops)
    fn parse_expr(&mut self) -> Result<Expr, SqliteError> {
        self.skip_whitespace();
        if self.pos >= self.input.len() {
            return Err(SqliteError::SyntaxError);
        }

        let ch = self.input[self.pos];

        // Try literal value
        if ch == b'\'' || ch == b'"' || ch.is_ascii_digit() || (ch == b'-' && self.pos + 1 < self.input.len() && self.input[self.pos + 1].is_ascii_digit()) {
            let val = self.read_literal_value()?;
            return Ok(Expr::Literal(val));
        }

        // NULL literal
        if ch == b'N' || ch == b'n' {
            let saved = self.pos;
            let ident = self.read_identifier()?;
            if ident.eq_ignore_ascii_case("NULL") {
                return Ok(Expr::Literal(Value::Null));
            }
            self.pos = saved;
        }

        // Identifier (column ref or function)
        let ident = self.read_identifier()?;
        self.skip_whitespace();

        // Check for function call
        if self.pos < self.input.len() && self.input[self.pos] == b'(' {
            self.pos += 1;
            let args = self.parse_expr_list_inner()?;
            return Ok(Expr::Function { name: ident, args });
        }

        // Check for dot-qualified table.column
        if self.pos < self.input.len() && self.input[self.pos] == b'.' {
            self.pos += 1;
            let col = self.read_identifier()?;
            return Ok(Expr::ColumnRef {
                table: Some(ident),
                column: col,
            });
        }

        // Simple column reference
        Ok(Expr::ColumnRef {
            table: None,
            column: ident,
        })
    }

    /// Parse table references in FROM clause
    fn parse_table_refs(&mut self) -> Result<Vec<TableRef>, SqliteError> {
        let mut tables = Vec::new();
        loop {
            let name = self.read_identifier()?;
            let mut alias = None;
            self.skip_whitespace();
            if self.pos < self.input.len() {
                let peek = self.peek_keyword()?;
                if peek.eq_ignore_ascii_case("AS") {
                    self.advance_word();
                    alias = Some(self.read_identifier()?);
                } else if !peek.eq_ignore_ascii_case("WHERE") && !peek.eq_ignore_ascii_case("JOIN") && !peek.eq_ignore_ascii_case("INNER") && !peek.eq_ignore_ascii_case("LEFT") && !peek.eq_ignore_ascii_case("CROSS") && !peek.eq_ignore_ascii_case("GROUP") && !peek.eq_ignore_ascii_case("ORDER") && !peek.eq_ignore_ascii_case("LIMIT") && !peek.eq_ignore_ascii_case("ON") {
                    // Implicit alias (identifier not a keyword)
                    if self.pos < self.input.len() && self.input[self.pos].is_ascii_alphabetic() {
                        alias = Some(self.read_identifier()?);
                    }
                }
            }

            tables.push(TableRef { name, alias });

            self.skip_whitespace();
            if self.pos >= self.input.len() || self.input[self.pos] != b',' {
                break;
            }
            self.pos += 1;
        }
        Ok(tables)
    }

    /// Parse a JOIN clause
    fn parse_join(&mut self) -> Result<JoinClause, SqliteError> {
        let mut join_type = JoinType::Inner;

        self.skip_whitespace();
        let peek = self.peek_keyword()?;
        if peek.eq_ignore_ascii_case("LEFT") {
            self.advance_word();
            self.skip_whitespace();
            let peek2 = self.peek_keyword()?;
            if peek2.eq_ignore_ascii_case("OUTER") {
                self.advance_word();
            }
            join_type = JoinType::LeftOuter;
        } else if peek.eq_ignore_ascii_case("INNER") {
            self.advance_word();
        } else if peek.eq_ignore_ascii_case("CROSS") {
            self.advance_word();
            join_type = JoinType::Cross;
        }

        self.expect_keyword("JOIN")?;

        let name = self.read_identifier()?;
        let mut alias = None;
        self.skip_whitespace();
        if self.pos < self.input.len() {
            let peek = self.peek_keyword()?;
            if peek.eq_ignore_ascii_case("AS") {
                self.advance_word();
                alias = Some(self.read_identifier()?);
            }
        }

        let table = TableRef { name, alias };

        let mut condition = None;
        self.skip_whitespace();
        if self.pos < self.input.len() {
            let peek = self.peek_keyword()?;
            if peek.eq_ignore_ascii_case("ON") {
                self.advance_word();
                condition = Some(self.parse_expr()?);
            }
        }

        Ok(JoinClause {
            join_type,
            table,
            condition,
        })
    }

    /// Parse ORDER BY list
    fn parse_order_by_list(&mut self) -> Result<Vec<OrderByItem>, SqliteError> {
        let mut items = Vec::new();
        loop {
            let expr = self.parse_expr()?;
            let mut ordering = Ordering::Asc;
            self.skip_whitespace();
            if self.pos < self.input.len() {
                let peek = self.peek_keyword()?;
                if peek.eq_ignore_ascii_case("ASC") {
                    self.advance_word();
                    ordering = Ordering::Asc;
                } else if peek.eq_ignore_ascii_case("DESC") {
                    self.advance_word();
                    ordering = Ordering::Desc;
                }
            }
            items.push(OrderByItem { expr, ordering });

            self.skip_whitespace();
            if self.pos >= self.input.len() || self.input[self.pos] != b',' {
                break;
            }
            self.pos += 1;
        }
        Ok(items)
    }

    /// Read a literal value from the input
    fn read_literal_value(&mut self) -> Result<Value, SqliteError> {
        self.skip_whitespace();
        if self.pos >= self.input.len() {
            return Err(SqliteError::SyntaxError);
        }

        let ch = self.input[self.pos];

        // String literal
        if ch == b'\'' {
            self.pos += 1;
            let mut s = String::new();
            while self.pos < self.input.len() {
                let c = self.input[self.pos];
                if c == b'\'' {
                    self.pos += 1;
                    // Handle escaped single quote ''
                    if self.pos < self.input.len() && self.input[self.pos] == b'\'' {
                        s.push('\'');
                        self.pos += 1;
                    } else {
                        break;
                    }
                } else {
                    s.push(c as char);
                    self.pos += 1;
                }
            }
            return Ok(Value::Text(s));
        }

        // Blob literal (X'...')
        if ch == b'X' || ch == b'x' {
            if self.pos + 1 < self.input.len() && self.input[self.pos + 1] == b'\'' {
                self.pos += 2;
                let mut blob = Vec::new();
                while self.pos < self.input.len() {
                    let c = self.input[self.pos];
                    if c == b'\'' {
                        self.pos += 1;
                        break;
                    }
                    if c.is_ascii_hexdigit() {
                        blob.push(c);
                        self.pos += 1;
                    } else {
                        return Err(SqliteError::SyntaxError);
                    }
                }
                let mut result = Vec::new();
                let hex_str: Vec<u8> = blob;
                let mut i = 0;
                while i + 1 < hex_str.len() {
                    let high = hex_val(hex_str[i]);
                    let low = hex_val(hex_str[i + 1]);
                    result.push((high << 4) | low);
                    i += 2;
                }
                return Ok(Value::Blob(result));
            }
        }

        // Numeric literal
        let mut num_str = String::new();
        let mut is_real = false;

        if ch == b'-' {
            num_str.push('-');
            self.pos += 1;
        }

        while self.pos < self.input.len() {
            let c = self.input[self.pos];
            if c.is_ascii_digit() {
                num_str.push(c as char);
                self.pos += 1;
            } else if c == b'.' {
                is_real = true;
                num_str.push('.');
                self.pos += 1;
            } else if c == b'e' || c == b'E' {
                is_real = true;
                num_str.push(c as char);
                self.pos += 1;
                if self.pos < self.input.len() && (self.input[self.pos] == b'+' || self.input[self.pos] == b'-') {
                    num_str.push(self.input[self.pos] as char);
                    self.pos += 1;
                }
            } else {
                break;
            }
        }

        if num_str.is_empty() {
            return Err(SqliteError::SyntaxError);
        }

        if is_real {
            let val: f64 = num_str.parse().map_err(|_| SqliteError::SyntaxError)?;
            Ok(Value::Real(val))
        } else {
            let val: i64 = num_str.parse().map_err(|_| SqliteError::SyntaxError)?;
            if val >= i32::MIN as i64 && val <= i32::MAX as i64 {
                Ok(Value::Integer(val as i32))
            } else {
                Ok(Value::I64(val))
            }
        }
    }
}

/// Convert a hex ASCII digit to its numeric value
fn hex_val(c: u8) -> u8 {
    match c {
        b'0'..=b'9' => c - b'0',
        b'a'..=b'f' => c - b'a' + 10,
        b'A'..=b'F' => c - b'A' + 10,
        _ => 0,
    }
}

/// Parse a SQL string into an AST node
pub fn parse_sql(sql: &str) -> Result<SqlNode, SqliteError> {
    let mut parser = SqlParser::new(sql);
    parser.parse()
}
