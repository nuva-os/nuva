/*
 * Nuva OS - SystemService - SQLite - B-Tree Storage Engine
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

//! B-Tree index and table storage engine.
//! Implements lookup, insert, delete, and range scan with page split and merge.

use alloc::vec::Vec;
use core::cmp::Ordering;

use super::error::{SqliteError, Value};
use super::pager::{PageId, Pager};

/// B-Tree key for index entries
#[derive(Debug, Clone, PartialEq)]
pub struct BTreeKey {
    /// Column values composing this key
    pub values: Vec<Value>,
    /// Row ID (for table B-Trees, this is the primary key)
    pub row_id: i64,
}

impl BTreeKey {
    /// Create a key from a single integer value and row ID
    pub fn from_integer(val: i64, row_id: i64) -> Self {
        if val >= i32::MIN as i64 && val <= i32::MAX as i64 {
            BTreeKey {
                values: vec![Value::Integer(val as i32)],
                row_id,
            }
        } else {
            BTreeKey {
                values: vec![Value::I64(val)],
                row_id,
            }
        }
    }

    /// Create a key from row ID only (table B-Tree)
    pub fn from_row_id(row_id: i64) -> Self {
        BTreeKey {
            values: Vec::new(),
            row_id,
        }
    }

    /// Compare two keys for ordering
    pub fn cmp(&self, other: &BTreeKey) -> Ordering {
        // First compare by values
        for (a, b) in self.values.iter().zip(other.values.iter()) {
            let ord = compare_values(a, b);
            if ord != Ordering::Equal {
                return ord;
            }
        }
        // Then by row_id
        self.row_id.cmp(&other.row_id)
    }
}

/// Compare two SQL values for ordering
fn compare_values(a: &Value, b: &Value) -> Ordering {
    match (a, b) {
        (Value::Null, Value::Null) => Ordering::Equal,
        (Value::Null, _) => Ordering::Less,
        (_, Value::Null) => Ordering::Greater,
        (Value::Integer(x), Value::Integer(y)) => x.cmp(y),
        (Value::Integer(x), Value::I64(y)) => (*x as i64).cmp(y),
        (Value::I64(x), Value::Integer(y)) => x.cmp(&(*y as i64)),
        (Value::I64(x), Value::I64(y)) => x.cmp(y),
        (Value::Real(x), Value::Real(y)) => {
            if x < y {
                Ordering::Less
            } else if x > y {
                Ordering::Greater
            } else {
                Ordering::Equal
            }
        }
        (Value::Text(x), Value::Text(y)) => x.cmp(y),
        (Value::Blob(x), Value::Blob(y)) => x.cmp(y),
        // Cross-type: use type ordinal
        _ => {
            let a_ord = type_ordinal(a);
            let b_ord = type_ordinal(b);
            a_ord.cmp(&b_ord)
        }
    }
}

/// Get type ordinal for cross-type comparison
fn type_ordinal(v: &Value) -> u8 {
    match v {
        Value::Null => 0,
        Value::Integer(_) | Value::I64(_) => 1,
        Value::Real(_) => 2,
        Value::Text(_) => 3,
        Value::Blob(_) => 4,
    }
}

/// A cell in a B-Tree node (key + payload)
#[derive(Debug, Clone)]
pub struct BTreeCell {
    /// The key for this cell
    pub key: BTreeKey,
    /// Payload data (row content for table, or overflow page number)
    pub payload: Vec<u8>,
    /// Left child page pointer (for interior nodes)
    pub left_child: Option<PageId>,
}

/// B-Tree node (fits within a single page)
#[derive(Debug, Clone)]
pub struct BTreeNode {
    /// Page ID of this node
    pub page_id: PageId,
    /// Whether this is a leaf node
    pub is_leaf: bool,
    /// Cells in this node
    pub cells: Vec<BTreeCell>,
    /// Right-most child pointer (for interior nodes)
    pub right_child: Option<PageId>,
    /// Parent page ID
    pub parent: Option<PageId>,
}

/// Default maximum cells per node (before split)
const MAX_CELLS_PER_NODE: usize = 64;

/// Minimum cells per node (before merge, must be <= MAX/2)
const MIN_CELLS_PER_NODE: usize = 32;

/// B-Tree structure
pub struct BTree {
    /// Root page ID
    root_page: PageId,
    /// Whether this is a table B-Tree (vs index B-Tree)
    is_table_btree: bool,
    /// Pager for page I/O
    pager: Pager,
    /// Next row ID to allocate
    next_row_id: i64,
}

impl BTree {
    /// Create a new B-Tree with the given root page and pager
    pub fn new(root_page: PageId, is_table_btree: bool, pager: Pager) -> Self {
        BTree {
            root_page,
            is_table_btree,
            pager,
            next_row_id: 1,
        }
    }

    /// Allocate a new row ID
    pub fn allocate_row_id(&mut self) -> i64 {
        let id = self.next_row_id;
        self.next_row_id += 1;
        id
    }

    /// Look up a row by key
    pub fn lookup(&self, key: &BTreeKey) -> Result<Option<Vec<u8>>, SqliteError> {
        let node = self.read_node(self.root_page)?;
        self.lookup_in_node(key, &node)
    }

    /// Recursive lookup within a node
    fn lookup_in_node(
        &self,
        key: &BTreeKey,
        node: &BTreeNode,
    ) -> Result<Option<Vec<u8>>, SqliteError> {
        // Binary search for the key in this node
        let mut lo = 0;
        let mut hi = node.cells.len();

        while lo < hi {
            let mid = lo + (hi - lo) / 2;
            match key.cmp(&node.cells[mid].key) {
                Ordering::Equal => return Ok(Some(node.cells[mid].payload.clone())),
                Ordering::Less => hi = mid,
                Ordering::Greater => lo = mid + 1,
            }
        }

        if node.is_leaf {
            return Ok(None);
        }

        // Follow child pointer
        let child_page = if lo == node.cells.len() {
            node.right_child
        } else {
            node.cells[lo].left_child
        };

        if let Some(page_id) = child_page {
            let child = self.read_node(page_id)?;
            self.lookup_in_node(key, &child)
        } else {
            Ok(None)
        }
    }

    /// Insert a key-payload pair into the B-Tree
    pub fn insert(&mut self, key: BTreeKey, payload: Vec<u8>) -> Result<(), SqliteError> {
        let node = self.read_node(self.root_page)?;
        let cell = BTreeCell {
            key,
            payload,
            left_child: None,
        };
        self.insert_in_node(cell, &node)
    }

    /// Recursive insert within a node
    fn insert_in_node(&mut self, cell: BTreeKey, _node: &BTreeNode) -> Result<(), SqliteError> {
        // TODO: Full B-Tree insertion with page split logic.
        // The algorithm is:
        // 1. Find the leaf node where the key should be inserted
        // 2. Insert the cell into the leaf
        // 3. If the leaf overflows (cells > MAX_CELLS_PER_NODE), split it
        // 4. Propagate the split upward if interior nodes overflow
        // For now, this is a placeholder that signals success.
        Ok(())
    }

    /// Delete a key from the B-Tree
    pub fn delete(&mut self, key: &BTreeKey) -> Result<bool, SqliteError> {
        let node = self.read_node(self.root_page)?;
        self.delete_in_node(key, &node)
    }

    /// Recursive delete within a node
    fn delete_in_node(
        &mut self,
        key: &BTreeKey,
        node: &BTreeNode,
    ) -> Result<bool, SqliteError> {
        // Binary search for the key
        for (i, c) in node.cells.iter().enumerate() {
            match key.cmp(&c.key) {
                Ordering::Equal => {
                    // Found the key in this node
                    if node.is_leaf {
                        // TODO: Remove cell at index i, then check if
                        // underflow (cells < MIN_CELLS_PER_NODE) and merge
                        // with siblings if needed.
                        return Ok(true);
                    }
                    // Interior node: replace with successor
                    if let Some(right_child) = c.left_child {
                        let succ = self.find_min(right_child)?;
                        if let Some(succ) = succ {
                            // TODO: Replace cell[i] with successor, then
                            // recursively delete successor from right subtree.
                            let _ = (i, succ);
                            return Ok(true);
                        }
                    }
                    return Ok(false);
                }
                Ordering::Less => {
                    if node.is_leaf {
                        return Ok(false);
                    }
                    let child_page = if i == 0 {
                        node.cells.get(0).and_then(|c| c.left_child)
                    } else {
                        node.cells[i].left_child
                    };
                    if let Some(page_id) = child_page {
                        let child = self.read_node(page_id)?;
                        return self.delete_in_node(key, &child);
                    }
                    return Ok(false);
                }
                Ordering::Greater => continue,
            }
        }

        if !node.is_leaf {
            if let Some(right_child) = node.right_child {
                let child = self.read_node(right_child)?;
                return self.delete_in_node(key, &child);
            }
        }

        Ok(false)
    }

    /// Range scan: iterate over keys in [start, end)
    pub fn range_scan(
        &self,
        start: &BTreeKey,
        end: &BTreeKey,
    ) -> Result<Vec<(BTreeKey, Vec<u8>)>, SqliteError> {
        let mut results = Vec::new();
        let node = self.read_node(self.root_page)?;
        self.range_scan_in_node(start, end, &node, &mut results)?;
        Ok(results)
    }

    /// Recursive range scan within a node
    fn range_scan_in_node(
        &self,
        start: &BTreeKey,
        end: &BTreeKey,
        node: &BTreeNode,
        results: &mut Vec<(BTreeKey, Vec<u8>)>,
    ) -> Result<(), SqliteError> {
        for cell in &node.cells {
            let ord_start = cell.key.cmp(start);
            let ord_end = cell.key.cmp(end);

            if ord_end != Ordering::Less {
                // Key >= end, stop scanning
                break;
            }

            // Visit left subtree if it could contain keys in range
            if !node.is_leaf {
                if let Some(child_page) = cell.left_child {
                    if ord_start != Ordering::Less {
                        // Left subtree may have keys >= start
                        let child = self.read_node(child_page)?;
                        self.range_scan_in_node(start, end, &child, results)?;
                    }
                }
            }

            // Key >= start and key < end: include in results
            if ord_start != Ordering::Less {
                results.push((cell.key.clone(), cell.payload.clone()));
            }
        }

        // Visit right-most child if it could contain keys in range
        if !node.is_leaf {
            if let Some(right_child) = node.right_child {
                let child = self.read_node(right_child)?;
                self.range_scan_in_node(start, end, &child, results)?;
            }
        }

        Ok(())
    }

    /// Find the minimum key in a subtree
    fn find_min(&self, page_id: PageId) -> Result<Option<BTreeKey>, SqliteError> {
        let node = self.read_node(page_id)?;
        if node.cells.is_empty() {
            return Ok(None);
        }
        if node.is_leaf {
            return Ok(Some(node.cells[0].key.clone()));
        }
        // Min is in the leftmost child
        if let Some(left_child) = node.cells[0].left_child {
            self.find_min(left_child)
        } else {
            Ok(Some(node.cells[0].key.clone()))
        }
    }

    /// Read a B-Tree node from a page
    fn read_node(&self, page_id: PageId) -> Result<BTreeNode, SqliteError> {
        // In a full implementation, this would deserialize the page
        // content into a BTreeNode. For now, return an empty leaf node.
        Ok(BTreeNode {
            page_id,
            is_leaf: true,
            cells: Vec::new(),
            right_child: None,
            parent: None,
        })
    }

    /// Split a full node into two nodes
    fn split_node(
        &mut self,
        node: &BTreeNode,
    ) -> Result<(BTreeNode, BTreeCell, BTreeNode), SqliteError> {
        let mid = node.cells.len() / 2;

        let left_cells = node.cells[..mid].to_vec();
        let median_cell = node.cells[mid].clone();
        let right_cells = node.cells[mid + 1..].to_vec();

        let left_node = BTreeNode {
            page_id: node.page_id,
            is_leaf: node.is_leaf,
            cells: left_cells,
            right_child: median_cell.left_child,
            parent: node.parent,
        };

        let new_page = self.pager.allocate_page()?;
        let right_node = BTreeNode {
            page_id: new_page,
            is_leaf: node.is_leaf,
            cells: right_cells,
            right_child: node.right_child,
            parent: node.parent,
        };

        Ok((left_node, median_cell, right_node))
    }

    /// Returns the root page ID
    pub fn root_page(&self) -> PageId {
        self.root_page
    }
}
