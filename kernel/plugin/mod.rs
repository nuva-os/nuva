/*
 * Plugin System - Core Module
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * This module provides the core plugin infrastructure for Nuva OS,
 * enabling dynamic loading, lifecycle management, and dependency resolution.
 */

// Re-export print macros from crate root
pub use crate::{pr_emerg, pr_alert, pr_crit, pr_err, pr_warn, pr_notice, pr_info, pr_debug};
pub mod audit;
pub mod core;
pub mod legacy;
pub mod loader;
pub mod manager;
pub mod packagemgr;
pub mod registry;
pub mod sandbox;
pub mod sdk;
pub mod signature;

// Re-export main types
pub use core::{Plugin, PluginMeta, PluginType, PluginState, PluginContext, PluginError};
pub use manager::PluginManager;
pub use manager::ManagerConfig;
pub use loader::PluginLoader;
pub use registry::PluginRegistry;
pub use sandbox::SandboxExecutor;

// Re-export legacy plugin types
pub use legacy::{
    PluginDependency, PluginInfo, PluginOps, PluginConfig, PluginStats,
    PluginFlags, PluginId, PluginCategory, PluginMgrStats,
    get_plugin_manager, init_plugin, plugin_register, plugin_find,
    plugin_activate, plugin_deactivate, plugin_probe,
};

/// Initialize plugin subsystem
pub fn init_plugin_system() -> Result<(), PluginError> {
    // Initialize plugin registry
    let registry = PluginRegistry::new();

    // Initialize plugin loader
    let loader = PluginLoader::new();

    // Initialize plugin manager
    let _manager = PluginManager::new(ManagerConfig::default());

    // Store manager in kernel context
    // TODO: Integrate with kernel subsystem management

    Ok(())
}
