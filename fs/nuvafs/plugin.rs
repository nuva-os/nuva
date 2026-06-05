/*
 * Nuva OS - NuvaFS FS Plugin Integration
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

//! NuvaFS Plugin Integration
//! Microkernel plugin adapters for integrating NuvaFS snapshot and WAL
//! functionality into the Nuva OS plugin system.

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use alloc::string::String;
use alloc::vec::Vec;
use alloc::sync::Arc;
use alloc::collections::BTreeMap;
use spin::Mutex;

/// Plugin state for NuvaFS plugins
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FsPluginState {
    /// Plugin loaded but not initialized
    Loaded = 0,
    /// Plugin initialized
    Initialized = 1,
    /// Plugin active and running
    Active = 2,
    /// Plugin deactivated
    Deactivated = 3,
    /// Plugin in error state
    Error = 4,
}

/// Plugin type for NuvaFS plugins
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FsPluginType {
    /// Snapshot management plugin
    Snapshot = 0,
    /// WAL (Write-Ahead Log) plugin
    WalLog = 1,
    /// Checkpoint plugin
    Checkpoint = 2,
    /// Audit logging plugin
    Audit = 3,
    /// Custom extension plugin
    Extension = 4,
}

/// Plugin metadata for NuvaFS plugins
#[derive(Debug, Clone)]
pub struct FsPluginMeta {
    /// Plugin name
    pub name: &'static str,
    /// Plugin version (major, minor, patch)
    pub version: (u32, u32, u32),
    /// Plugin type
    pub plugin_type: FsPluginType,
    /// Plugin priority (higher = more important)
    pub priority: u32,
}

/// Plugin error type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FsPluginError {
    /// Plugin not found
    NotFound,
    /// Already initialized
    AlreadyInitialized,
    /// Invalid state transition
    InvalidState,
    /// Initialization failed
    InitFailed,
    /// Activation failed
    ActivateFailed,
    /// Deactivation failed
    DeactivateFailed,
    /// Out of memory
    OutOfMemory,
    /// I/O error
    IoError,
}

/// Result type for plugin operations
pub type FsPluginResult<T> = Result<T, FsPluginError>;

/// Core trait for NuvaFS plugins.
///
/// This trait mirrors the kernel Plugin trait but is specialized
/// for filesystem operations, avoiding dependency on the full
/// kernel plugin infrastructure.
pub trait FsPlugin: Send + Sync {
    /// Get plugin metadata
    fn meta(&self) -> &FsPluginMeta;

    /// Initialize the plugin
    fn init(&mut self) -> FsPluginResult<()>;

    /// Activate the plugin
    fn activate(&mut self) -> FsPluginResult<()>;

    /// Deactivate the plugin
    fn deactivate(&mut self) -> FsPluginResult<()>;

    /// Get the current state
    fn state(&self) -> FsPluginState;
}

/// Snapshot plugin: integrates snapshot operations into the plugin system.
pub struct FsSnapshotPlugin {
    /// Plugin metadata
    meta: FsPluginMeta,
    /// Current state
    state: AtomicU32,
    /// Number of snapshots created through this plugin
    snapshot_count: AtomicU64,
    /// Number of rollbacks performed
    rollback_count: AtomicU64,
}

impl FsSnapshotPlugin {
    /// Create a new snapshot plugin
    pub fn new() -> Self {
        Self {
            meta: FsPluginMeta {
                name: "nuvafs-snapshot",
                version: (1, 0, 0),
                plugin_type: FsPluginType::Snapshot,
                priority: 100,
            },
            state: AtomicU32::new(FsPluginState::Loaded as u32),
            snapshot_count: AtomicU64::new(0),
            rollback_count: AtomicU64::new(0),
        }
    }

    /// Record a snapshot creation
    pub fn record_snapshot(&self) {
        self.snapshot_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a rollback
    pub fn record_rollback(&self) {
        self.rollback_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Get snapshot count
    pub fn snapshot_count(&self) -> u64 {
        self.snapshot_count.load(Ordering::Relaxed)
    }

    /// Get rollback count
    pub fn rollback_count(&self) -> u64 {
        self.rollback_count.load(Ordering::Relaxed)
    }
}

impl Default for FsSnapshotPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl FsPlugin for FsSnapshotPlugin {
    fn meta(&self) -> &FsPluginMeta {
        &self.meta
    }

    fn init(&mut self) -> FsPluginResult<()> {
        let current = self.state.load(Ordering::Relaxed);
        if current != FsPluginState::Loaded as u32 {
            return Err(FsPluginError::AlreadyInitialized);
        }
        self.state.store(FsPluginState::Initialized as u32, Ordering::Relaxed);
        Ok(())
    }

    fn activate(&mut self) -> FsPluginResult<()> {
        let current = self.state.load(Ordering::Relaxed);
        if current != FsPluginState::Initialized as u32 {
            return Err(FsPluginError::InvalidState);
        }
        self.state.store(FsPluginState::Active as u32, Ordering::Relaxed);
        Ok(())
    }

    fn deactivate(&mut self) -> FsPluginResult<()> {
        let current = self.state.load(Ordering::Relaxed);
        if current != FsPluginState::Active as u32 {
            return Err(FsPluginError::InvalidState);
        }
        self.state.store(FsPluginState::Deactivated as u32, Ordering::Relaxed);
        Ok(())
    }

    fn state(&self) -> FsPluginState {
        match self.state.load(Ordering::Relaxed) {
            0 => FsPluginState::Loaded,
            1 => FsPluginState::Initialized,
            2 => FsPluginState::Active,
            3 => FsPluginState::Deactivated,
            _ => FsPluginState::Error,
        }
    }
}

/// WAL log plugin: integrates WAL operations into the plugin system.
pub struct WalLogPlugin {
    /// Plugin metadata
    meta: FsPluginMeta,
    /// Current state
    state: AtomicU32,
    /// Number of WAL records written
    record_count: AtomicU64,
    /// Number of checkpoints performed
    checkpoint_count: AtomicU64,
}

impl WalLogPlugin {
    /// Create a new WAL log plugin
    pub fn new() -> Self {
        Self {
            meta: FsPluginMeta {
                name: "nuvafs-wallog",
                version: (1, 0, 0),
                plugin_type: FsPluginType::WalLog,
                priority: 90,
            },
            state: AtomicU32::new(FsPluginState::Loaded as u32),
            record_count: AtomicU64::new(0),
            checkpoint_count: AtomicU64::new(0),
        }
    }

    /// Record a WAL write
    pub fn record_write(&self) {
        self.record_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a checkpoint
    pub fn record_checkpoint(&self) {
        self.checkpoint_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Get record count
    pub fn record_count(&self) -> u64 {
        self.record_count.load(Ordering::Relaxed)
    }

    /// Get checkpoint count
    pub fn checkpoint_count(&self) -> u64 {
        self.checkpoint_count.load(Ordering::Relaxed)
    }
}

impl Default for WalLogPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl FsPlugin for WalLogPlugin {
    fn meta(&self) -> &FsPluginMeta {
        &self.meta
    }

    fn init(&mut self) -> FsPluginResult<()> {
        let current = self.state.load(Ordering::Relaxed);
        if current != FsPluginState::Loaded as u32 {
            return Err(FsPluginError::AlreadyInitialized);
        }
        self.state.store(FsPluginState::Initialized as u32, Ordering::Relaxed);
        Ok(())
    }

    fn activate(&mut self) -> FsPluginResult<()> {
        let current = self.state.load(Ordering::Relaxed);
        if current != FsPluginState::Initialized as u32 {
            return Err(FsPluginError::InvalidState);
        }
        self.state.store(FsPluginState::Active as u32, Ordering::Relaxed);
        Ok(())
    }

    fn deactivate(&mut self) -> FsPluginResult<()> {
        let current = self.state.load(Ordering::Relaxed);
        if current != FsPluginState::Active as u32 {
            return Err(FsPluginError::InvalidState);
        }
        self.state.store(FsPluginState::Deactivated as u32, Ordering::Relaxed);
        Ok(())
    }

    fn state(&self) -> FsPluginState {
        match self.state.load(Ordering::Relaxed) {
            0 => FsPluginState::Loaded,
            1 => FsPluginState::Initialized,
            2 => FsPluginState::Active,
            3 => FsPluginState::Deactivated,
            _ => FsPluginState::Error,
        }
    }
}

/// Plugin registry for managing NuvaFS plugins.
pub struct FsPluginRegistry {
    /// Registered plugins (by name)
    plugins: BTreeMap<&'static str, Arc<Mutex<dyn FsPlugin>>>,
    /// Number of active plugins
    active_count: AtomicU32,
}

impl FsPluginRegistry {
    /// Create a new empty plugin registry
    pub fn new() -> Self {
        Self {
            plugins: BTreeMap::new(),
            active_count: AtomicU32::new(0),
        }
    }

    /// Register a plugin
    pub fn register(&mut self, plugin: Arc<Mutex<dyn FsPlugin>>) -> FsPluginResult<()> {
        let name = {
            let p = plugin.lock();
            p.meta().name
        };
        if self.plugins.contains_key(name) {
            return Err(FsPluginError::AlreadyInitialized);
        }
        self.plugins.insert(name, plugin);
        Ok(())
    }

    /// Activate a plugin by name
    pub fn activate(&mut self, name: &str) -> FsPluginResult<()> {
        let plugin = self.plugins.get_mut(name).ok_or(FsPluginError::NotFound)?;
        {
            let mut p = plugin.lock();
            p.activate()?;
        }
        self.active_count.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    /// Deactivate a plugin by name
    pub fn deactivate(&mut self, name: &str) -> FsPluginResult<()> {
        let plugin = self.plugins.get_mut(name).ok_or(FsPluginError::NotFound)?;
        {
            let mut p = plugin.lock();
            p.deactivate()?;
        }
        self.active_count.fetch_sub(1, Ordering::Relaxed);
        Ok(())
    }

    /// Get the number of active plugins
    pub fn active_count(&self) -> u32 {
        self.active_count.load(Ordering::Relaxed)
    }
}

impl Default for FsPluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_plugin_lifecycle() {
        let mut plugin = FsSnapshotPlugin::new();
        assert_eq!(plugin.state(), FsPluginState::Loaded);

        plugin.init().unwrap();
        assert_eq!(plugin.state(), FsPluginState::Initialized);

        plugin.activate().unwrap();
        assert_eq!(plugin.state(), FsPluginState::Active);

        plugin.record_snapshot();
        plugin.record_snapshot();
        assert_eq!(plugin.snapshot_count(), 2);

        plugin.deactivate().unwrap();
        assert_eq!(plugin.state(), FsPluginState::Deactivated);
    }

    #[test]
    fn test_wallog_plugin_lifecycle() {
        let mut plugin = WalLogPlugin::new();
        assert_eq!(plugin.state(), FsPluginState::Loaded);

        plugin.init().unwrap();
        plugin.activate().unwrap();
        assert_eq!(plugin.state(), FsPluginState::Active);

        plugin.record_write();
        plugin.record_checkpoint();
        assert_eq!(plugin.record_count(), 1);
        assert_eq!(plugin.checkpoint_count(), 1);

        plugin.deactivate().unwrap();
    }

    #[test]
    fn test_plugin_invalid_state_transition() {
        let mut plugin = FsSnapshotPlugin::new();
        // Cannot activate without init
        assert_eq!(plugin.activate(), Err(FsPluginError::InvalidState));
    }
}
