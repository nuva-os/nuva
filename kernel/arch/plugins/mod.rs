/*
 * Nuva OS - Kernel - Architecture Plugin System
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


//! Architecture plugin system
/*!*/
//! Provides multi-architecture support, supports plugin-based management, dynamically loads architecture plugins based on different devices

pub mod device;
pub mod build_config;

use core::fmt;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use alloc::boxed::Box;

use super::{ArchOps, PageTableOps, IrqControllerOps, TimerOps, PowerOps, ContextOps};
use super::{PhysAddr, VirtAddr, ProtFlags, CpuContext};
use crate::{pr_info};

// ============================================================================
// Architecture Plugin Interface
// ============================================================================

/// Architecture plugin metadata
#[derive(Debug, Clone)]
pub struct ArchPluginMeta {
    /// Plugin name
    pub name: &'static str,
    /// Plugin version
    pub version: &'static str,
    /// Architecture type
    pub arch_type: ArchType,
    /// Supported device list
    pub supported_devices: &'static [&'static str],
    /// Plugin description
    pub description: &'static str,
    /// Priority (smaller value = higher priority)
    pub priority: u32,
}

/// Architecture type
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ArchType {
    /// ARM64 architecture
    Arm64,
    /// x86-64 architecture
    X64,
    /// LoongArch64 architecture
    LoongArch64,
    /// RISC-V 64 architecture
    RiscV64,
}

impl fmt::Display for ArchType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ArchType::Arm64 => write!(f, "arm64"),
            ArchType::X64 => write!(f, "x86_64"),
            ArchType::LoongArch64 => write!(f, "loongarch64"),
            ArchType::RiscV64 => write!(f, "riscv64"),
        }
    }
}

/// Device type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceType {
    /// Desktop device
    Desktop,
    /// Mobile device
    Mobile,
    /// Server device
    Server,
    /// Embedded device
    Embedded,
    /// IoT device
    IoT,
}

impl fmt::Display for DeviceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeviceType::Desktop => write!(f, "desktop"),
            DeviceType::Mobile => write!(f, "mobile"),
            DeviceType::Server => write!(f, "server"),
            DeviceType::Embedded => write!(f, "embedded"),
            DeviceType::IoT => write!(f, "iot"),
        }
    }
}

/// Device information
#[derive(Debug, Clone)]
pub struct DeviceInfo {
    /// Device name
    pub name: String,
    /// Device type
    pub device_type: DeviceType,
    /// CPU vendor
    pub cpu_vendor: String,
    /// CPU model
    pub cpu_model: String,
    /// CPU core count
    pub cpu_cores: u32,
    /// Supported features
    pub features: Vec<String>,
    /// Memory size (bytes)
    pub memory_size: u64,
}

impl DeviceInfo {
    /// Create new device information
    pub fn new(name: &str, device_type: DeviceType) -> Self {
        Self {
            name: name.to_string(),
            device_type,
            cpu_vendor: String::new(),
            cpu_model: String::new(),
            cpu_cores: 0,
            features: Vec::new(),
            memory_size: 0,
        }
    }

    /// Detect current device
    pub fn detect() -> Self {
        // TODO: Implement actual device detection
        // Read device tree or ACPI information
        Self::new("unknown", DeviceType::Desktop)
    }

    /// Match architecture plugin
    pub fn matches_plugin(&self, meta: &ArchPluginMeta) -> bool {
        // Check if device is in supported list
        for device in meta.supported_devices {
            if self.name.contains(device) || self.cpu_model.contains(device) {
                return true;
            }
        }
        false
    }
}

// ============================================================================
// Architecture Plugin Trait
// ============================================================================

/// Architecture plugin interface
pub trait ArchPlugin: Send + Sync {
    /// Get plugin metadata
    fn meta(&self) -> &ArchPluginMeta;

    /// Initialize plugin
    fn init(&self) -> Result<(), PluginError>;

    /// Shutdown plugin
    fn shutdown(&self) -> Result<(), PluginError>;

    /// Get architecture operations interface
    fn ops(&self) -> &dyn ArchOps;

    /// Check device compatibility
    fn is_compatible(&self, device: &DeviceInfo) -> bool;

    /// Get feature support list
    fn get_features(&self) -> Vec<&'static str>;
}

/// Plugin error
#[derive(Debug)]
pub enum PluginError {
    /// Initialization failed
    InitFailed(String),
    /// Shutdown failed
    ShutdownFailed(String),
    /// Plugin not found
    NotFound(String),
    /// Plugin already loaded
    AlreadyLoaded(String),
    /// Incompatible
    Incompatible(String),
    /// Memory error
    MemoryError,
}

impl fmt::Display for PluginError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PluginError::InitFailed(msg) => write!(f, "Plugin init failed: {}", msg),
            PluginError::ShutdownFailed(msg) => write!(f, "Plugin shutdown failed: {}", msg),
            PluginError::NotFound(name) => write!(f, "Plugin not found: {}", name),
            PluginError::AlreadyLoaded(name) => write!(f, "Plugin already loaded: {}", name),
            PluginError::Incompatible(msg) => write!(f, "Plugin incompatible: {}", msg),
            PluginError::MemoryError => write!(f, "Memory error"),
        }
    }
}

// ============================================================================
// Architecture Plugin Manager
// ============================================================================

/// Architecture plugin manager
pub struct ArchPluginManager {
    /// Registered plugins
    plugins: BTreeMap<String, Box<dyn ArchPlugin>>,
    /// Currently active plugin
    active_plugin: Option<String>,
    /// Current device information
    current_device: Option<DeviceInfo>,
    /// Plugin load order (by priority)
    load_order: Vec<String>,
}

impl ArchPluginManager {
    /// Create new plugin manager
    pub const fn new() -> Self {
        Self {
            plugins: BTreeMap::new(),
            active_plugin: None,
            current_device: None,
            load_order: Vec::new(),
        }
    }

    /// Register plugin
    pub fn register(&mut self, plugin: Box<dyn ArchPlugin>) -> Result<(), PluginError> {
        let name = plugin.meta().name.to_string();

        if self.plugins.contains_key(&name) {
            return Err(PluginError::AlreadyLoaded(name));
        }

        // Insert into load order by priority
        let priority = plugin.meta().priority;
        let mut inserted = false;
        for (i, existing_name) in self.load_order.iter().enumerate() {
            if let Some(existing) = self.plugins.get(existing_name) {
                if existing.meta().priority > priority {
                    self.load_order.insert(i, name.clone());
                    inserted = true;
                    break;
                }
            }
        }
        if !inserted {
            self.load_order.push(name.clone());
        }

        self.plugins.insert(name, plugin);
        Ok(())
    }

    /// Unregister plugin
    pub fn unregister(&mut self, name: &str) -> Result<(), PluginError> {
        if let Some(plugin) = self.plugins.remove(name) {
            plugin.shutdown()?;
            self.load_order.retain(|n| n != name);
            if self.active_plugin.as_deref() == Some(name) {
                self.active_plugin = None;
            }
            Ok(())
        } else {
            Err(PluginError::NotFound(name.to_string()))
        }
    }

    /// Detect device and select best plugin
    pub fn detect_and_select(&mut self) -> Result<&dyn ArchPlugin, PluginError> {
        // Detect current device
        let device = DeviceInfo::detect();
        self.current_device = Some(device.clone());

        // Select plugin by priority and compatibility
        for name in &self.load_order.clone() {
            if let Some(plugin) = self.plugins.get(name) {
                if plugin.is_compatible(&device) {
                    plugin.init()?;
                    self.active_plugin = Some(name.clone());
                    log_info!("Selected architecture plugin: {} for device: {}",
                        name, device.name);
                    return Ok(plugin.as_ref());
                }
            }
        }

        Err(PluginError::NotFound("No compatible plugin found".to_string()))
    }

    /// Select plugin for device
    pub fn select_for_device(&mut self, device: DeviceInfo) -> Result<&dyn ArchPlugin, PluginError> {
        self.current_device = Some(device.clone());

        for name in &self.load_order.clone() {
            if let Some(plugin) = self.plugins.get(name) {
                if plugin.is_compatible(&device) {
                    plugin.init()?;
                    self.active_plugin = Some(name.clone());
                    return Ok(plugin.as_ref());
                }
            }
        }

        Err(PluginError::NotFound("No compatible plugin found".to_string()))
    }

    /// Get current active plugin
    pub fn get_active(&self) -> Option<&dyn ArchPlugin> {
        self.active_plugin.as_ref().and_then(|name| {
            self.plugins.get(name).map(|p| p.as_ref())
        })
    }

    /// Get all registered plugins
    pub fn get_all_plugins(&self) -> Vec<&dyn ArchPlugin> {
        self.plugins.values().map(|p| p.as_ref()).collect()
    }

    /// Get plugins of specified architecture type
    pub fn get_by_arch_type(&self, arch_type: ArchType) -> Vec<&dyn ArchPlugin> {
        self.plugins.values()
            .filter(|p| p.meta().arch_type == arch_type)
            .map(|p| p.as_ref())
            .collect()
    }

    /// Get current device information
    pub fn get_current_device(&self) -> Option<&DeviceInfo> {
        self.current_device.as_ref()
    }

    /// Check if plugin is loaded
    pub fn is_loaded(&self, name: &str) -> bool {
        self.plugins.contains_key(name)
    }

    /// Get plugin count
    pub fn plugin_count(&self) -> usize {
        self.plugins.len()
    }
}

impl Default for ArchPluginManager {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Global Plugin Manager
// ============================================================================

use spin::Mutex;

/// Global architecture plugin manager
static PLUGIN_MANAGER: Mutex<ArchPluginManager> = Mutex::new(ArchPluginManager::new());

/// Get plugin manager
pub fn plugin_manager() -> &'static Mutex<ArchPluginManager> {
    &PLUGIN_MANAGER
}

/// Initialize architecture plugin system
pub fn init_plugin_system() {
    let mut manager = PLUGIN_MANAGER.lock();

    // Register built-in plugins
    register_builtin_plugins(&mut manager);

    log_info!("Architecture plugin system initialized");
    log_info!("  Registered plugins: {}", manager.plugin_count());
}

/// Register built-in plugins
fn register_builtin_plugins(_manager: &mut ArchPluginManager) {
    // ARM64 plugin - TODO: implement arm64::plugin::Arm64Plugin
    // #[cfg(feature = "arm64")]
    // {
    //     use super::arm64::plugin::Arm64Plugin;
    //     let _ = _manager.register(Box::new(Arm64Plugin::new()));
    // }

    // x64 plugin - TODO: implement x64::plugin::X64Plugin
    // #[cfg(feature = "x64")]
    // {
    //     use super::x64::plugin::X64Plugin;
    //     let _ = _manager.register(Box::new(X64Plugin::new()));
    // }

    // LoongArch64 plugin - TODO: implement loongarch64::plugin::LoongArch64Plugin
    // #[cfg(feature = "loongarch64")]
    // {
    //     use super::loongarch64::plugin::LoongArch64Plugin;
    //     let _ = _manager.register(Box::new(LoongArch64Plugin::new()));
    // }

    // RISC-V 64 plugin - TODO: implement riscv64::plugin::RiscV64Plugin
    // #[cfg(feature = "riscv64")]
    // {
    //     use super::riscv64::plugin::RiscV64Plugin;
    //     let _ = _manager.register(Box::new(RiscV64Plugin::new()));
    // }
}

/// Get current architecture operations
// TODO: ArchOps is not dyn compatible; restructure trait or use concrete type
// pub fn current_arch_ops() -> Option<&'static dyn ArchOps> {
//     let manager = PLUGIN_MANAGER.lock();
//     manager.get_active().map(|p| p.ops())
// }

// ============================================================================
// Compile-time Feature Support
// ============================================================================

/// Compile-time architecture features
pub struct ArchFeatures {
    /// Architecture type
    pub arch_type: ArchType,
    /// Supported feature bitmask
    pub features: u64,
}

// Feature bit definitions
pub const FEATURE_SIMD_128: u64 = 1 << 0;   // 128-bit SIMD
pub const FEATURE_SIMD_256: u64 = 1 << 1;   // 256-bit SIMD
pub const FEATURE_SIMD_512: u64 = 1 << 2;   // 512-bit SIMD
pub const FEATURE_VECTORIZATION: u64 = 1 << 3; // Vectorization
pub const FEATURE_VIRTUALIZATION: u64 = 1 << 4; // Virtualization
pub const FEATURE_ATOMIC: u64 = 1 << 5;     // Atomic operation
pub const FEATURE_FPU: u64 = 1 << 6;        // Floating point unit
pub const FEATURE_DEBUG: u64 = 1 << 7;      // Hardware debug
pub const FEATURE_CRYPTO: u64 = 1 << 8;     // Hardware acceleration encryption

impl ArchFeatures {
    /// Check if feature is supported
    pub fn has_feature(&self, feature: u64) -> bool {
        (self.features & feature) != 0
    }

    /// Get ARM64 features
    #[cfg(target_arch = "aarch64")]
    pub fn arm64() -> Self {
        Self {
            arch_type: ArchType::Arm64,
            features: FEATURE_SIMD_128 | FEATURE_FPU | FEATURE_ATOMIC,
        }
    }

    /// Get x64 features
    #[cfg(target_arch = "x86_64")]
    pub fn x64() -> Self {
        Self {
            arch_type: ArchType::X64,
            features: FEATURE_SIMD_128 | FEATURE_SIMD_256 | FEATURE_FPU | FEATURE_ATOMIC,
        }
    }

    /// Get LoongArch64 features
    #[cfg(target_arch = "loongarch64")]
    pub fn loongarch64() -> Self {
        Self {
            arch_type: ArchType::LoongArch64,
            features: FEATURE_SIMD_128 | FEATURE_SIMD_256 | FEATURE_FPU | FEATURE_ATOMIC,
        }
    }

    /// Get RISC-V 64 features
    #[cfg(target_arch = "riscv64")]
    pub fn riscv64() -> Self {
        Self {
            arch_type: ArchType::RiscV64,
            features: FEATURE_SIMD_128 | FEATURE_FPU | FEATURE_ATOMIC | FEATURE_VECTORIZATION,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arch_type_display() {
        assert_eq!(ArchType::Arm64.to_string(), "arm64");
        assert_eq!(ArchType::X64.to_string(), "x86_64");
        assert_eq!(ArchType::LoongArch64.to_string(), "loongarch64");
        assert_eq!(ArchType::RiscV64.to_string(), "riscv64");
    }

    #[test]
    fn test_device_type_display() {
        assert_eq!(DeviceType::Desktop.to_string(), "desktop");
        assert_eq!(DeviceType::Mobile.to_string(), "mobile");
        assert_eq!(DeviceType::Server.to_string(), "server");
    }

    #[test]
    fn test_arch_features() {
        let features = ArchFeatures {
            arch_type: ArchType::Arm64,
            features: FEATURE_SIMD_128 | FEATURE_FPU,
        };
        assert!(features.features & FEATURE_SIMD_128 != 0);
        assert!(features.features & FEATURE_FPU != 0);
    }
}