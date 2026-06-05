/*
 * Nuva OS - NuvaFS Snapshot Chain
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

//! NuvaFS Snapshot Chain
//! Manages a linked chain of snapshots via parent_snapshot pointers,
//! enabling efficient traversal from any snapshot back to the origin.

use core::sync::atomic::{AtomicU32, AtomicU64, AtomicBool, Ordering};
use alloc::collections::BTreeMap;
use alloc::vec::Vec;

/// Maximum chain depth to prevent infinite loops from corrupted chains
pub const MAX_CHAIN_DEPTH: u32 = 256;

/// Snapshot chain node metadata
#[derive(Debug, Clone, Copy)]
pub struct ChainNode {
    /// Snapshot ID
    pub id: u64,
    /// Parent snapshot ID (0 = root / no parent)
    pub parent_id: u64,
    /// Creation timestamp
    pub create_time: u64,
    /// Depth in the chain (0 = root)
    pub depth: u32,
    /// Number of direct children
    pub child_count: AtomicU32,
    /// Whether this snapshot is still active
    pub active: AtomicBool,
}

impl ChainNode {
    /// Create a new chain node
    pub fn new(id: u64, parent_id: u64, create_time: u64, depth: u32) -> Self {
        Self {
            id,
            parent_id,
            create_time,
            depth,
            child_count: AtomicU32::new(0),
            active: AtomicBool::new(true),
        }
    }

    /// Check if this node is the root of a chain
    pub fn is_root(&self) -> bool {
        self.parent_id == 0
    }

    /// Check if this node is active
    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::Relaxed)
    }

    /// Mark this node as inactive (deleted)
    pub fn deactivate(&self) {
        self.active.store(false, Ordering::Relaxed);
    }

    /// Get the number of direct children
    pub fn child_count(&self) -> u32 {
        self.child_count.load(Ordering::Relaxed)
    }

    /// Increment child count
    pub fn add_child(&self) {
        self.child_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Decrement child count
    pub fn remove_child(&self) {
        self.child_count.fetch_sub(1, Ordering::Relaxed);
    }
}

/// Snapshot chain manager.
///
/// Maintains a BTreeMap of chain nodes and supports traversal,
/// branch detection, and chain-consistent deletion.
pub struct SnapshotChain {
    /// All chain nodes indexed by snapshot ID
    nodes: BTreeMap<u64, ChainNode>,
    /// Total active snapshot count
    active_count: AtomicU32,
    /// Maximum chain depth observed
    max_depth: AtomicU32,
    /// Next snapshot ID allocator
    next_id: AtomicU64,
}

/// Errors for snapshot chain operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChainError {
    /// Snapshot not found in the chain
    NotFound,
    /// Parent snapshot not found
    ParentNotFound,
    /// Chain depth would exceed maximum
    ChainTooDeep,
    /// Snapshot has children and cannot be deleted
    HasChildren,
    /// Snapshot is already inactive
    AlreadyInactive,
    /// Cycle detected in chain
    CycleDetected,
}

impl SnapshotChain {
    /// Create a new empty snapshot chain
    pub fn new() -> Self {
        Self {
            nodes: BTreeMap::new(),
            active_count: AtomicU32::new(0),
            max_depth: AtomicU32::new(0),
            next_id: AtomicU64::new(1),
        }
    }

    /// Add a root snapshot (no parent) to the chain.
    /// Returns the allocated snapshot ID.
    pub fn add_root(&mut self, create_time: u64) -> Result<u64, ChainError> {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let node = ChainNode::new(id, 0, create_time, 0);
        self.nodes.insert(id, node);
        self.active_count.fetch_add(1, Ordering::Relaxed);
        Ok(id)
    }

    /// Add a child snapshot under the given parent.
    /// Returns the allocated snapshot ID.
    pub fn add_child(&mut self, parent_id: u64, create_time: u64) -> Result<u64, ChainError> {
        // Look up parent and compute depth
        let parent_depth = {
            let parent = self.nodes.get(&parent_id).ok_or(ChainError::ParentNotFound)?;
            if !parent.is_active() {
                return Err(ChainError::ParentNotFound);
            }
            let depth = parent.depth + 1;
            if depth > MAX_CHAIN_DEPTH {
                return Err(ChainError::ChainTooDeep);
            }
            depth
        };

        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let node = ChainNode::new(id, parent_id, create_time, parent_depth);
        self.nodes.insert(id, node);

        // Update parent child count
        if let Some(parent) = self.nodes.get(&parent_id) {
            parent.add_child();
        }

        // Update max depth
        {
            let current_max = self.max_depth.load(Ordering::Relaxed);
            if parent_depth > current_max {
                self.max_depth.store(parent_depth, Ordering::Relaxed);
            }
        }

        self.active_count.fetch_add(1, Ordering::Relaxed);
        Ok(id)
    }

    /// Remove (deactivate) a snapshot from the chain.
    /// Fails if the snapshot has active children.
    pub fn remove(&mut self, id: u64) -> Result<(), ChainError> {
        let node = self.nodes.get(&id).ok_or(ChainError::NotFound)?;
        if !node.is_active() {
            return Err(ChainError::AlreadyInactive);
        }
        if node.child_count() > 0 {
            return Err(ChainError::HasChildren);
        }

        // Deactivate the node
        node.deactivate();

        // Decrement parent child count
        let parent_id = node.parent_id;
        if parent_id != 0 {
            if let Some(parent) = self.nodes.get(&parent_id) {
                parent.remove_child();
            }
        }

        self.active_count.fetch_sub(1, Ordering::Relaxed);
        Ok(())
    }

    /// Traverse the chain from a snapshot back to the root.
    /// Returns the path as a Vec of snapshot IDs from the given snapshot
    /// to the root (inclusive). Returns None if the chain is broken.
    pub fn traverse_to_root(&self, id: u64) -> Option<Vec<u64>> {
        let mut path = Vec::new();
        let mut current_id = id;
        let mut depth = 0u32;

        loop {
            let node = self.nodes.get(&current_id)?;
            path.push(current_id);

            if node.is_root() {
                return Some(path);
            }

            current_id = node.parent_id;
            depth += 1;
            if depth > MAX_CHAIN_DEPTH {
                return None; // chain too deep, likely corrupted
            }
        }
    }

    /// Get all direct children of a snapshot
    pub fn children(&self, parent_id: u64) -> Vec<u64> {
        let mut result = Vec::new();
        for (&id, node) in self.nodes.iter() {
            if node.parent_id == parent_id && node.is_active() {
                result.push(id);
            }
        }
        result
    }

    /// Get a chain node by ID
    pub fn get(&self, id: u64) -> Option<&ChainNode> {
        self.nodes.get(&id)
    }

    /// Get the number of active snapshots
    pub fn active_count(&self) -> u32 {
        self.active_count.load(Ordering::Relaxed)
    }

    /// Get the maximum chain depth
    pub fn max_depth(&self) -> u32 {
        self.max_depth.load(Ordering::Relaxed)
    }

    /// List all active snapshot IDs in ascending order
    pub fn list_active(&self) -> Vec<u64> {
        let mut result = Vec::new();
        for (&id, node) in self.nodes.iter() {
            if node.is_active() {
                result.push(id);
            }
        }
        result
    }

    /// Check if a snapshot exists and is active
    pub fn is_active(&self, id: u64) -> bool {
        self.nodes.get(&id).map_or(false, |n| n.is_active())
    }
}

impl Default for SnapshotChain {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_add_root() {
        let mut chain = SnapshotChain::new();
        let id = chain.add_root(1000).unwrap();
        assert!(chain.is_active(id));
        assert_eq!(chain.active_count(), 1);
    }

    #[test]
    fn test_chain_add_child() {
        let mut chain = SnapshotChain::new();
        let root = chain.add_root(1000).unwrap();
        let child = chain.add_child(root, 2000).unwrap();
        assert!(chain.is_active(child));
        assert_eq!(chain.active_count(), 2);

        let node = chain.get(child).unwrap();
        assert_eq!(node.parent_id, root);
        assert_eq!(node.depth, 1);
    }

    #[test]
    fn test_chain_traverse_to_root() {
        let mut chain = SnapshotChain::new();
        let root = chain.add_root(1000).unwrap();
        let child = chain.add_child(root, 2000).unwrap();
        let grandchild = chain.add_child(child, 3000).unwrap();

        let path = chain.traverse_to_root(grandchild).unwrap();
        assert_eq!(path.len(), 3);
        assert_eq!(path[0], grandchild);
        assert_eq!(path[1], child);
        assert_eq!(path[2], root);
    }

    #[test]
    fn test_chain_remove_leaf() {
        let mut chain = SnapshotChain::new();
        let root = chain.add_root(1000).unwrap();
        let child = chain.add_child(root, 2000).unwrap();

        assert!(chain.remove(child).is_ok());
        assert!(!chain.is_active(child));
        assert_eq!(chain.active_count(), 1);
    }

    #[test]
    fn test_chain_remove_with_children_fails() {
        let mut chain = SnapshotChain::new();
        let root = chain.add_root(1000).unwrap();
        let _child = chain.add_child(root, 2000).unwrap();

        assert_eq!(chain.remove(root), Err(ChainError::HasChildren));
    }

    #[test]
    fn test_chain_children() {
        let mut chain = SnapshotChain::new();
        let root = chain.add_root(1000).unwrap();
        let c1 = chain.add_child(root, 2000).unwrap();
        let c2 = chain.add_child(root, 3000).unwrap();

        let kids = chain.children(root);
        assert_eq!(kids.len(), 2);
        assert!(kids.contains(&c1));
        assert!(kids.contains(&c2));
    }

    #[test]
    fn test_chain_parent_not_found() {
        let mut chain = SnapshotChain::new();
        assert_eq!(chain.add_child(999, 1000), Err(ChainError::ParentNotFound));
    }
}
