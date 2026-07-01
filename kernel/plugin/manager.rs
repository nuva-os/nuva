/*
 * Nuva OS - Kernel - Plugin - Manager
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
 * Plugin Manager - Lifecycle Management
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * This module implements the plugin manager which handles
 * plugin lifecycle, dependency resolution, and state management.
 */

use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::fmt;
use spin::RwLock;

use super::core::{Plugin, PluginContext, PluginError, PluginId, PluginMeta, PluginState};
use super::loader::PluginLoader;
use super::registry::PluginRegistry;
use super::sandbox::SandboxExecutor;

/// Plugin manager - Central management for all plugins
/// Responsibilities:
/// - Plugin loading and unloading
/// - Lifecycle management (init, activate, deactivate)
/// - Dependency resolution
/// - State tracking
/// - Sandbox management
pub struct PluginManager {
    /// Plugin registry
    registry: Arc<RwLock<PluginRegistry>>,

    /// Plugin loader
    loader: Arc<RwLock<PluginLoader>>,

    /// Sandbox executor
    sandbox: Arc<RwLock<SandboxExecutor>>,

    /// Plugin instances
    plugins: RwLock<BTreeMap<PluginId, Arc<RwLock<Box<dyn Plugin>>>>>,

    /// Plugin states
    states: RwLock<BTreeMap<PluginId, PluginState>>,

    /// Plugin contexts
    contexts: RwLock<BTreeMap<PluginId, PluginContext>>,

    /// Next plugin ID
    next_id: RwLock<u64>,

    /// Count of plugins that failed to load
    failed_count: RwLock<usize>,

    /// Cumulative load time in milliseconds
    total_load_time_ms: RwLock<u64>,

    /// Manager configuration
    config: ManagerConfig,
}

impl PluginManager {
    /// Create new plugin manager
    pub fn new(config: ManagerConfig) -> Self {
        Self {
            registry: Arc::new(RwLock::new(PluginRegistry::new())),
            loader: Arc::new(RwLock::new(PluginLoader::new())),
            sandbox: Arc::new(RwLock::new(SandboxExecutor::new())),
            plugins: RwLock::new(BTreeMap::new()),
            states: RwLock::new(BTreeMap::new()),
            contexts: RwLock::new(BTreeMap::new()),
            next_id: RwLock::new(1),
            failed_count: RwLock::new(0),
            total_load_time_ms: RwLock::new(0),
            config,
        }
    }

    /// Load plugin from file
    /// @param path: Path to plugin file
    /// @return: Plugin ID
    pub fn load_plugin(&self, path: &str) -> Result<PluginId, PluginError> {
        let start_ts = crate::hal::cpu::read_cycle_counter() / 1000;

        let mut loader = self.loader.write();
        let mut plugin = match loader.load(path) {
            Ok(p) => p,
            Err(e) => {
                *self.failed_count.write() += 1;
                return Err(e);
            }
        };

        // Get plugin metadata
        let meta = plugin.meta().clone();
        let meta_name = meta.name;
        let meta_version = meta.version;

        // Check dependencies
        self.check_dependencies(&meta)?;

        // Generate plugin ID
        let id = self.generate_id();

        // Create plugin context
        let context = self.create_context(id);

        // Initialize plugin
        plugin.init(&context)?;

        // Register plugin
        let mut registry = self.registry.write();
        registry.register(id, meta.clone());

        // Store plugin instance
        let mut plugins = self.plugins.write();
        plugins.insert(id, Arc::new(RwLock::new(plugin)));

        // Set state
        let mut states = self.states.write();
        states.insert(id, PluginState::Initialized);

        // Store context
        let mut contexts = self.contexts.write();
        contexts.insert(id, context);

        // Accumulate load time
        let end_ts = crate::hal::cpu::read_cycle_counter() / 1000;
        if end_ts >= start_ts {
            *self.total_load_time_ms.write() += end_ts - start_ts;
        }

        Ok(id)
    }

    /// Activate plugin
    /// @param id: Plugin ID
    pub fn activate_plugin(&self, id: PluginId) -> Result<(), PluginError> {
        // Check current state
        let current_state = self.get_state(id)?;
        if current_state != PluginState::Initialized {
            return Err(PluginError::InvalidState {
                current: current_state,
                expected: PluginState::Initialized,
            });
        }

        // Get plugin
        let plugins = self.plugins.read();
        let plugin = plugins.get(&id).ok_or(PluginError::NotFound(id))?;

        // Activate dependencies first
        self.activate_dependencies(id)?;

        // Activate plugin
        let mut plugin = plugin.write();
        plugin.activate()?;

        // Update state
        let mut states = self.states.write();
        states.insert(id, PluginState::Active);

        Ok(())
    }

    /// Deactivate plugin
    /// @param id: Plugin ID
    pub fn deactivate_plugin(&self, id: PluginId) -> Result<(), PluginError> {
        // Check current state
        let current_state = self.get_state(id)?;
        if current_state != PluginState::Active {
            return Err(PluginError::InvalidState {
                current: current_state,
                expected: PluginState::Active,
            });
        }

        // Check if other plugins depend on this one
        self.check_dependents(id)?;

        // Get plugin
        let plugins = self.plugins.read();
        let plugin = plugins.get(&id).ok_or(PluginError::NotFound(id))?;

        // Deactivate plugin
        let mut plugin = plugin.write();
        plugin.deactivate()?;

        // Update state
        let mut states = self.states.write();
        states.insert(id, PluginState::Deactivated);

        Ok(())
    }

    /// Unload plugin
    /// @param id: Plugin ID
    pub fn unload_plugin(&self, id: PluginId) -> Result<(), PluginError> {
        // Check current state
        let current_state = self.get_state(id)?;
        if current_state == PluginState::Active {
            // Deactivate first
            self.deactivate_plugin(id)?;
        }

        // Get plugin
        let plugins = self.plugins.read();
        let plugin = plugins.get(&id).ok_or(PluginError::NotFound(id))?;

        // Unload plugin
        let mut plugin = plugin.write();
        plugin.unload()?;

        // Remove from all maps
        let mut plugins = self.plugins.write();
        plugins.remove(&id);

        let mut states = self.states.write();
        states.remove(&id);

        let mut contexts = self.contexts.write();
        contexts.remove(&id);

        let mut registry = self.registry.write();
        registry.unregister(id);

        Ok(())
    }

    /// Get plugin state
    pub fn get_state(&self, id: PluginId) -> Result<PluginState, PluginError> {
        let states = self.states.read();
        states.get(&id).copied().ok_or(PluginError::NotFound(id))
    }

    /// Get plugin metadata
    pub fn get_meta(&self, id: PluginId) -> Result<PluginMeta, PluginError> {
        let registry = self.registry.read();
        registry
            .get_meta(id)
            .cloned()
            .ok_or(PluginError::NotFound(id))
    }

    /// List all plugins
    pub fn list_plugins(&self) -> Vec<PluginId> {
        let plugins = self.plugins.read();
        plugins.keys().copied().collect()
    }

    /// List plugins by type
    pub fn list_plugins_by_type(&self, plugin_type: super::core::PluginType) -> Vec<PluginId> {
        let registry = self.registry.read();
        registry.list_by_type(plugin_type)
    }

    /// Check if plugin is loaded
    pub fn is_loaded(&self, id: PluginId) -> bool {
        let plugins = self.plugins.read();
        plugins.contains_key(&id)
    }

    /// Check if plugin is active
    pub fn is_active(&self, id: PluginId) -> bool {
        let states = self.states.read();
        states.get(&id) == Some(&PluginState::Active)
    }

    // Private helper methods

    /// Generate unique plugin ID
    fn generate_id(&self) -> PluginId {
        let mut next_id = self.next_id.write();
        let id = *next_id;
        *next_id += 1;
        PluginId(id)
    }

    /// Create plugin context
    fn create_context(&self, id: PluginId) -> PluginContext {
        PluginContext {
            id,
            services: Arc::new(super::core::PluginServices::new()),
            config: super::core::PluginConfig::new(),
        }
    }

    /// Check plugin dependencies
    fn check_dependencies(&self, meta: &PluginMeta) -> Result<(), PluginError> {
        let registry = self.registry.read();

        for dep in &meta.dependencies {
            // Check if dependency is loaded
            if let Some(dep_id) = registry.find_by_name(dep.name) {
                // Check version
                let dep_meta = match registry.get_meta(dep_id) {
                    Some(m) => m,
                    None => return Err(PluginError::NotFound),
                };
                if dep_meta.version < dep.min_version {
                    return Err(PluginError::VersionMismatch {
                        expected: dep.min_version,
                        found: dep_meta.version,
                    });
                }

                if let Some(max_version) = dep.max_version {
                    if dep_meta.version > max_version {
                        return Err(PluginError::VersionMismatch {
                            expected: max_version,
                            found: dep_meta.version,
                        });
                    }
                }
            } else if !dep.optional {
                // Required dependency not found
                return Err(PluginError::DependencyError(String::from(dep.name)));
            }
        }

        Ok(())
    }

    /// Activate dependencies
    fn activate_dependencies(&self, id: PluginId) -> Result<(), PluginError> {
        let registry = self.registry.read();
        let meta = registry.get_meta(id).ok_or(PluginError::NotFound(id))?;

        for dep in &meta.dependencies {
            if let Some(dep_id) = registry.find_by_name(dep.name) {
                if !self.is_active(dep_id) {
                    drop(registry);
                    self.activate_plugin(dep_id)?;
                    return Ok(());
                }
            }
        }

        Ok(())
    }

    /// Check dependents (plugins that depend on this one)
    fn check_dependents(&self, id: PluginId) -> Result<(), PluginError> {
        let registry = self.registry.read();
        let meta = registry.get_meta(id).ok_or(PluginError::NotFound(id))?;

        // Check all active plugins
        for (plugin_id, plugin_meta) in registry.iter() {
            if *plugin_id == id {
                continue;
            }

            if self.is_active(*plugin_id) {
                // Check if this plugin depends on the one we're deactivating
                for dep in &plugin_meta.dependencies {
                    if dep.name == meta.name {
                        return Err(PluginError::DependencyError(String::from(
                            "Plugin has active dependents",
                        )));
                    }
                }
            }
        }

        Ok(())
    }
}

/// Manager configuration
#[derive(Debug, Clone)]
pub struct ManagerConfig {
    /// Enable sandbox by default
    pub enable_sandbox: bool,

    /// Maximum number of plugins
    pub max_plugins: usize,

    /// Plugin search paths
    pub search_paths: Vec<String>,

    /// Enable hot-plug
    pub enable_hot_plug: bool,

    /// Auto-activate on load
    pub auto_activate: bool,
}

impl Default for ManagerConfig {
    fn default() -> Self {
        Self {
            enable_sandbox: true,
            max_plugins: 1024,
            search_paths: Vec::new(),
            enable_hot_plug: true,
            auto_activate: false,
        }
    }
}

/// Plugin manager statistics
#[derive(Debug, Clone)]
pub struct ManagerStats {
    /// Total plugins loaded
    pub total_loaded: usize,

    /// Active plugins
    pub active_plugins: usize,

    /// Failed plugins
    pub failed_plugins: usize,

    /// Total load time (ms)
    pub total_load_time_ms: u64,
}

impl PluginManager {
    /// Get manager statistics
    pub fn stats(&self) -> ManagerStats {
        let plugins = self.plugins.read();
        let states = self.states.read();

        let total_loaded = plugins.len();
        let active_plugins = states
            .values()
            .filter(|s| **s == PluginState::Active)
            .count();

        ManagerStats {
            total_loaded,
            active_plugins,
            failed_plugins: *self.failed_count.read(),
            total_load_time_ms: *self.total_load_time_ms.read(),
        }
    }
}
