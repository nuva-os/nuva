/*
 * Nuva OS - Kernel - Plugin - Core - Mod
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
 * Plugin Core - Trait Definitions and Types
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * This module defines the core Plugin trait and associated types
 * for the Nuva OS plugin system.
 */

use core::fmt;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::sync::Arc;
use spin::Mutex;

/// Plugin trait - All plugins must implement this interface
/// This trait defines the lifecycle of a plugin:
/// 1. init: Initialize plugin with context
/// 2. activate: Activate plugin for use
/// 3. deactivate: Deactivate plugin (prepare for unload)
/// 4. unload: Cleanup and unload plugin
pub trait Plugin: Send + Sync {
    /// Get plugin metadata
    fn meta(&self) -> &PluginMeta;

    /// Initialize plugin with context
    /// Called once when plugin is loaded
    fn init(&mut self, ctx: &PluginContext) -> Result<(), PluginError>;

    /// Activate plugin
    /// Called to make plugin active and ready for use
    fn activate(&mut self) -> Result<(), PluginError>;

    /// Deactivate plugin
    /// Called to deactivate plugin, preparing for unload
    fn deactivate(&mut self) -> Result<(), PluginError>;

    /// Unload plugin
    /// Called before plugin is unloaded, cleanup resources
    fn unload(&mut self) -> Result<(), PluginError>;
}

/// Plugin metadata
#[derive(Debug, Clone)]
pub struct PluginMeta {
    /// Plugin name
    pub name: &'static str,

    /// Plugin version
    pub version: Version,

    /// Plugin type
    pub plugin_type: PluginType,

    /// Plugin dependencies
    pub dependencies: Vec<Dependency>,

    /// Plugin capabilities
    pub capabilities: Capabilities,

    /// Plugin author
    pub author: &'static str,

    /// Plugin description
    pub description: &'static str,

    /// Plugin priority (higher = more important)
    pub priority: u32,

    /// Plugin flags
    pub flags: PluginFlags,
    pub info: PluginInfo,
    pub next: *mut PluginMeta,
}

impl PluginMeta {
    pub fn new(_id: u32, _name: &str) -> Self {
        PluginMeta {
            name: "",
            version: Version::new(0, 0, 0),
            plugin_type: PluginType::Kernel,
            dependencies: Vec::new(),
            capabilities: Capabilities::empty(),
            author: "",
            description: "",
            priority: 0,
            flags: PluginFlags::empty(),
            info: PluginInfo::default(),
            next: core::ptr::null_mut(),
        }
    }
}


/// Plugin type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PluginType {
    /// Device driver plugin
    Driver,

    /// File system plugin
    FileSystem,

    /// Network protocol plugin
    Network,

    /// Security module plugin
    Security,

    /// Quantum computing plugin
    Quantum,

    /// AI/ML plugin
    Ai,

    /// Power management plugin
    Power,

    /// Debug/trace plugin
    Debug,

    /// Platform-specific plugin
    Platform,

    /// General extension plugin
    Extension,

    /// Kernel-level plugin
    Kernel,
}

/// Plugin state enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginState {
    /// Plugin is loaded but not initialized
    Loaded,

    /// Plugin is initialized
    Initialized,

    /// Plugin is active and running
    Active,

    /// Plugin is deactivated
    Deactivated,

    /// Plugin has error
    Error,

    /// Plugin is unloading
    Unloading,
}

/// Plugin context - provides access to system services
pub struct PluginContext {
    /// Plugin ID
    pub id: PluginId,

    /// System services interface
    pub services: Arc<PluginServices>,

    /// Plugin configuration
    pub config: PluginConfig,
}

/// Plugin ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PluginId(pub u64);

/// Version structure
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl Version {
    pub const fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self { major, minor, patch }
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Dependency specification
#[derive(Debug, Clone)]
pub struct Dependency {
    /// Dependency name
    pub name: &'static str,

    /// Minimum version required
    pub min_version: Version,

    /// Maximum version allowed (optional)
    pub max_version: Option<Version>,

    /// Is this dependency optional?
    pub optional: bool,
}

/// Plugin capabilities
#[derive(Debug, Clone)]
pub struct Capabilities {
    /// Can be hot-plugged
    pub hot_plug: bool,

    /// Can be unloaded
    pub unloadable: bool,

    /// Requires sandbox
    pub sandbox: bool,

    /// Supports suspend/resume
    pub power_management: bool,

    /// Custom capabilities
    pub custom: Vec<&'static str>,
}

impl Default for Capabilities {
    fn default() -> Self {
        Self {
            hot_plug: false,
            unloadable: true,
            sandbox: false,
            power_management: false,
            custom: Vec::new(),
        }
    }
}

impl Capabilities {
    /// Create empty capabilities
    pub fn empty() -> Self {
        Self {
            hot_plug: false,
            unloadable: false,
            sandbox: false,
            power_management: false,
            custom: Vec::new(),
        }
    }
}

/// Plugin flags
bitflags::bitflags! {
    pub struct PluginFlags: u32 {
        /// No flags
        const NONE = 0;

        /// Plugin is essential (cannot be unloaded)
        const ESSENTIAL = 1 << 0;

        /// Plugin is experimental
        const EXPERIMENTAL = 1 << 1;

        /// Plugin is deprecated
        const DEPRECATED = 1 << 2;

        /// Plugin requires root privileges
        const ROOT_REQUIRED = 1 << 3;

        /// Plugin is sandboxed
        const SANDBOXED = 1 << 4;

        /// Plugin is signed
        const SIGNED = 1 << 5;

        /// Plugin is verified
        const VERIFIED = 1 << 6;
    }

}

impl Clone for PluginFlags {
    fn clone(&self) -> Self { *self }
}
impl Copy for PluginFlags {}

impl core::fmt::Debug for PluginFlags {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "PluginFlags({})", self.bits())
    }
}

/// Plugin info stub
#[derive(Debug, Clone)]
pub struct PluginInfo {
    pub name: &'static str,
    pub version: u32,
    pub api_version: u32,
}

impl Default for PluginInfo {
    fn default() -> Self {
        PluginInfo { name: "", version: 0, api_version: 0 }
    }
}

/// Plugin error type
#[derive(Debug, Clone)]
pub enum PluginError {
    /// Plugin not found
    NotFound(PluginId),

    /// Plugin already loaded
    AlreadyLoaded(PluginId),

    /// Plugin not loaded
    NotLoaded(PluginId),

    /// Invalid plugin
    InvalidPlugin(String),

    /// Initialization failed
    InitFailed(String),

    /// Activation failed
    ActivateFailed(String),

    /// Deactivation failed
    DeactivateFailed(String),

    /// Unload failed
    UnloadFailed(String),

    /// Dependency error
    DependencyError(String),

    /// Version mismatch
    VersionMismatch { expected: Version, found: Version },

    /// Permission denied
    PermissionDenied,

    /// Out of memory
    OutOfMemory,

    /// I/O error
    IoError(String),

    /// Not supported
    NotSupported,

    /// Invalid state
    InvalidState { current: PluginState, expected: PluginState },
}

impl fmt::Display for PluginError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound(id) => write!(f, "Plugin not found: {:?}", id),
            Self::AlreadyLoaded(id) => write!(f, "Plugin already loaded: {:?}", id),
            Self::NotLoaded(id) => write!(f, "Plugin not loaded: {:?}", id),
            Self::InvalidPlugin(msg) => write!(f, "Invalid plugin: {}", msg),
            Self::InitFailed(msg) => write!(f, "Initialization failed: {}", msg),
            Self::ActivateFailed(msg) => write!(f, "Activation failed: {}", msg),
            Self::DeactivateFailed(msg) => write!(f, "Deactivation failed: {}", msg),
            Self::UnloadFailed(msg) => write!(f, "Unload failed: {}", msg),
            Self::DependencyError(msg) => write!(f, "Dependency error: {}", msg),
            Self::VersionMismatch { expected, found } => {
                write!(f, "Version mismatch: expected {}, found {}", expected, found)
            }
            Self::PermissionDenied => write!(f, "Permission denied"),
            Self::OutOfMemory => write!(f, "Out of memory"),
            Self::IoError(msg) => write!(f, "I/O error: {}", msg),
            Self::NotSupported => write!(f, "Operation not supported"),
            Self::InvalidState { current, expected } => {
                write!(f, "Invalid state: current {:?}, expected {:?}", current, expected)
            }
        }
    }
}

/// Plugin services interface
/// Provides access to kernel services for plugins
pub struct PluginServices {
    /// Maximum memory this plugin may allocate (in bytes)
    pub memory_limit: usize,
    /// Number of IPC channels available
    pub ipc_channel_limit: u32,
    /// Whether device access is granted
    pub device_access: bool,
    /// Whether logging is enabled
    pub logging_enabled: bool,
}

impl PluginServices {
    /// Create plugin services with default kernel limits
    pub fn new() -> Self {
        Self {
            memory_limit: 4 * 1024 * 1024,
            ipc_channel_limit: 16,
            device_access: false,
            logging_enabled: true,
        }
    }

    /// Create plugin services with elevated privileges (kernel-level plugins)
    pub fn kernel_privileges() -> Self {
        Self {
            memory_limit: usize::MAX,
            ipc_channel_limit: 256,
            device_access: true,
            logging_enabled: true,
        }
    }

    /// Check if a memory allocation of `size` bytes is within limits
    pub fn check_memory_limit(&self, size: usize) -> bool {
        size <= self.memory_limit
    }

    /// Check if an IPC channel can be opened
    pub fn check_ipc_limit(&self, current_channels: u32) -> bool {
        current_channels < self.ipc_channel_limit
    }
}

impl Default for PluginServices {
    fn default() -> Self {
        Self::new()
    }
}

/// Plugin configuration
pub struct PluginConfig {
    /// Configuration data (key-value pairs)
    pub data: Vec<(&'static str, &'static str)>,
}

impl PluginConfig {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<&'static str> {
        self.data.iter().find(|(k, _)| *k == key).map(|(_, v)| *v)
    }
}
