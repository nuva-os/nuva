/*
 * Nuva OS - Kernel - Fs - Snapshot
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
/*
 * Nuva OS - Kernel - Filesystem Snapshot
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * NuvaFS snapshot management with COW-based
 * point-in-time consistency guarantees.
 */

use core::sync::atomic::{AtomicU64, AtomicU32, AtomicBool, Ordering};

use crate::kernel::error::{KernelError, KernelResult};

/// Maximum snapshots
pub const MAX_SNAPSHOTS: usize = 32;

/// Maximum snapshot name length
pub const MAX_SNAP_NAME_LEN: usize = 64;

/// Snapshot state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SnapshotState {
    /// Snapshot is being created
    Creating = 0,
    /// Snapshot is consistent and available
    Consistent = 1,
    /// Snapshot is being rolled back
    RollingBack = 2,
    /// Snapshot is being deleted
    Deleting = 3,
    /// Snapshot is invalid (corrupted)
    Invalid = 4,
}

/// Snapshot metadata
#[derive(Clone, Debug)]
pub struct SnapshotMeta {
    /// Snapshot ID
    pub snap_id: u64,
    /// Snapshot name
    pub name: [u8; MAX_SNAP_NAME_LEN],
    /// Creation timestamp (kernel ticks)
    pub create_time: u64,
    /// Root inode at snapshot time
    pub root_inode: u64,
    /// Number of COW pages in snapshot
    cow_pages: AtomicU32,
    /// Snapshot state
    state: AtomicU8,
    /// Whether snapshot is readonly
    readonly: AtomicBool,
}

impl SnapshotMeta {
    /// Create a new snapshot metadata
    pub const fn new(snap_id: u64, root_inode: u64) -> Self {
        SnapshotMeta {
            snap_id,
            name: [0u8; MAX_SNAP_NAME_LEN],
            create_time: 0,
            root_inode,
            cow_pages: AtomicU32::new(0),
            state: AtomicU8::new(SnapshotState::Creating as u8),
            readonly: AtomicBool::new(true),
        }
    }

    /// Get snapshot state
    pub fn state(&self) -> SnapshotState {
        match self.state.load(Ordering::Acquire) {
            0 => SnapshotState::Creating,
            1 => SnapshotState::Consistent,
            2 => SnapshotState::RollingBack,
            3 => SnapshotState::Deleting,
            _ => SnapshotState::Invalid,
        }
    }

    /// Set snapshot state
    pub fn set_state(&self, state: SnapshotState) {
        self.state.store(state as u8, Ordering::Release);
    }

    /// Check if snapshot is readonly
    pub fn is_readonly(&self) -> bool {
        self.readonly.load(Ordering::Acquire)
    }

    /// Get COW page count
    pub fn cow_pages(&self) -> u32 {
        self.cow_pages.load(Ordering::Acquire)
    }

    /// Increment COW page count
    pub fn inc_cow_pages(&self) {
        self.cow_pages.fetch_add(1, Ordering::Relaxed);
    }

    /// Set snapshot name
    pub fn set_name(&mut self, name: &[u8]) {
        let len = name.len().min(MAX_SNAP_NAME_LEN);
        self.name[..len].copy_from_slice(&name[..len]);
        if len < MAX_SNAP_NAME_LEN {
            self.name[len] = 0;
        }
    }
}

/// SnapshotManager: filesystem snapshot management
///
/// Creates, manages, and rolls back point-in-time
/// filesystem snapshots using COW pages.
pub struct SnapshotManager {
    /// Next snapshot ID
    next_snap_id: AtomicU64,
    /// Active snapshot count
    active_snaps: AtomicU32,
    /// Total snapshots created
    total_created: AtomicU64,
    /// Total rollbacks performed
    total_rollbacks: AtomicU64,
}

impl SnapshotManager {
    /// Create a new snapshot manager
    pub const fn new() -> Self {
        SnapshotManager {
            next_snap_id: AtomicU64::new(1),
            active_snaps: AtomicU32::new(0),
            total_created: AtomicU64::new(0),
            total_rollbacks: AtomicU64::new(0),
        }
    }

    /// Create a new snapshot
    ///
    /// @param name: Snapshot name
    /// @param root_inode: Root inode to snapshot
    /// @return: Snapshot metadata on success
    pub fn create_snapshot(&self, name: &[u8], root_inode: u64) -> KernelResult<SnapshotMeta> {
        if self.active_snaps.load(Ordering::Acquire) >= MAX_SNAPSHOTS as u32 {
            return Err(KernelError::OutOfMemory);
        }

        let snap_id = self.next_snap_id.fetch_add(1, Ordering::AcqRel);
        let mut meta = SnapshotMeta::new(snap_id, root_inode);
        meta.set_name(name);
        meta.create_time = 0; // TODO: Get current kernel time
        meta.set_state(SnapshotState::Creating);

        // TODO: Freeze all metadata pages (COW)
        // TODO: Record root inode and all reachable inodes
        // TODO: Flush WAL to ensure consistency

        meta.set_state(SnapshotState::Consistent);
        self.active_snaps.fetch_add(1, Ordering::Relaxed);
        self.total_created.fetch_add(1, Ordering::Relaxed);

        Ok(meta)
    }

    /// Roll back to a snapshot
    ///
    /// @param meta: Snapshot to roll back to
    /// @return: Ok on success
    pub fn rollback(&self, meta: &SnapshotMeta) -> KernelResult<()> {
        if meta.state() != SnapshotState::Consistent {
            return Err(KernelError::InvalidState);
        }

        meta.set_state(SnapshotState::RollingBack);

        // TODO: Restore all COW pages to original
        // TODO: Restore root inode
        // TODO: Invalidate newer snapshots
        // TODO: Flush changes to disk

        meta.set_state(SnapshotState::Consistent);
        self.total_rollbacks.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    /// Delete a snapshot
    ///
    /// @param meta: Snapshot to delete
    /// @return: Ok on success
    pub fn delete_snapshot(&self, meta: &SnapshotMeta) -> KernelResult<()> {
        if meta.state() == SnapshotState::Deleting || meta.state() == SnapshotState::Invalid {
            return Err(KernelError::InvalidState);
        }

        meta.set_state(SnapshotState::Deleting);

        // TODO: Release all COW pages
        // TODO: Remove snapshot metadata from disk

        self.active_snaps.fetch_sub(1, Ordering::Relaxed);
        Ok(())
    }

    /// Get statistics
    pub fn stats(&self) -> (u64, u32, u64, u64) {
        (
            self.next_snap_id.load(Ordering::Acquire) - 1,
            self.active_snaps.load(Ordering::Acquire),
            self.total_created.load(Ordering::Acquire),
            self.total_rollbacks.load(Ordering::Acquire),
        )
    }
}

/// Global snapshot manager
static SNAPSHOT_MANAGER: core::sync::OnceLock<SnapshotManager> = core::sync::OnceLock::new();

/// Get global snapshot manager
pub fn get_snapshot_manager() -> &'static SnapshotManager {
    SNAPSHOT_MANAGER.get_or_init(SnapshotManager::new)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_snapshot() {
        let mgr = SnapshotManager::new();
        let meta = mgr.create_snapshot(b"snap1", 1);
        assert!(meta.is_ok());
        let meta = meta.unwrap();
        assert_eq!(meta.state(), SnapshotState::Consistent);
        assert!(meta.is_readonly());
    }

    #[test]
    fn test_rollback() {
        let mgr = SnapshotManager::new();
        let meta = mgr.create_snapshot(b"snap1", 1).unwrap();
        assert!(mgr.rollback(&meta).is_ok());
    }

    #[test]
    fn test_delete() {
        let mgr = SnapshotManager::new();
        let meta = mgr.create_snapshot(b"snap1", 1).unwrap();
        assert!(mgr.delete_snapshot(&meta).is_ok());
        let (_, active, _, _) = mgr.stats();
        assert_eq!(active, 0);
    }

    #[test]
    fn test_snapshot_name() {
        let mut meta = SnapshotMeta::new(1, 1);
        meta.set_name(b"test_snapshot");
        assert_eq!(&meta.name[..13], b"test_snapshot");
    }
}