/*
 * Nuva OS - Build Configuration System
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


//! Build configuration system
/*!*/
//! Supports on-demand compilation, reduces redundant compilation

use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;
use core::fmt;

use super::{ArchType, DeviceType};
use crate::{pr_info};

// ============================================================================
// Build Target Configuration
// ============================================================================

/// Build target
#[derive(Debug, Clone)]
pub struct BuildTarget {
    /// Target name
    pub name: String,
    /// Architecture type
    pub arch_type: ArchType,
    /// Device type
    pub device_type: DeviceType,
    /// Enabled features
    pub features: Vec<String>,
    /// Disabled features
    pub disabled_features: Vec<String>,
    /// Compilation optimization level
    pub opt_level: OptLevel,
    /// Whether LTO is enabled
    pub lto: bool,
    /// Whether debug info is enabled
    pub debug: bool,
}

/// Optimization level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptLevel {
    /// No optimization
    O0,
    /// Basic optimization
    O1,
    /// Standard optimization
    O2,
    /// Aggressive optimization
    O3,
    /// Optimize for size
    Os,
    /// Optimize for size (more aggressive)
    Oz,
}

impl fmt::Display for OptLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OptLevel::O0 => write!(f, "0"),
            OptLevel::O1 => write!(f, "1"),
            OptLevel::O2 => write!(f, "2"),
            OptLevel::O3 => write!(f, "3"),
            OptLevel::Os => write!(f, "s"),
            OptLevel::Oz => write!(f, "z"),
        }
    }
}

impl Default for BuildTarget {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            arch_type: ArchType::Arm64,
            device_type: DeviceType::Desktop,
            features: Vec::new(),
            disabled_features: Vec::new(),
            opt_level: OptLevel::O2,
            lto: false,
            debug: false,
        }
    }
}

impl BuildTarget {
    /// Create ARM64 mobile device target
    pub fn arm64_mobile() -> Self {
        Self {
            name: "arm64-mobile".to_string(),
            arch_type: ArchType::Arm64,
            device_type: DeviceType::Mobile,
            features: vec![
                "neon".to_string(),
                "crypto".to_string(),
            ],
            disabled_features: vec!["sve".to_string()],
            opt_level: OptLevel::Os,
            lto: true,
            debug: false,
        }
    }

    /// Create ARM64 server target
    pub fn arm64_server() -> Self {
        Self {
            name: "arm64-server".to_string(),
            arch_type: ArchType::Arm64,
            device_type: DeviceType::Server,
            features: vec![
                "neon".to_string(),
                "sve".to_string(),
                "crypto".to_string(),
            ],
            disabled_features: Vec::new(),
            opt_level: OptLevel::O3,
            lto: true,
            debug: false,
        }
    }

    /// Create x64 desktop target
    pub fn x64_desktop() -> Self {
        Self {
            name: "x64-desktop".to_string(),
            arch_type: ArchType::X64,
            device_type: DeviceType::Desktop,
            features: vec![
                "sse".to_string(),
                "sse2".to_string(),
                "avx".to_string(),
                "avx2".to_string(),
            ],
            disabled_features: vec!["avx512".to_string()],
            opt_level: OptLevel::O2,
            lto: false,
            debug: true,
        }
    }

    /// Create x64 server target
    pub fn x64_server() -> Self {
        Self {
            name: "x64-server".to_string(),
            arch_type: ArchType::X64,
            device_type: DeviceType::Server,
            features: vec![
                "sse".to_string(),
                "sse2".to_string(),
                "avx".to_string(),
                "avx2".to_string(),
                "avx512".to_string(),
            ],
            disabled_features: Vec::new(),
            opt_level: OptLevel::O3,
            lto: true,
            debug: false,
        }
    }

    /// Create LoongArch desktop target
    pub fn loongarch_desktop() -> Self {
        Self {
            name: "loongarch-desktop".to_string(),
            arch_type: ArchType::LoongArch64,
            device_type: DeviceType::Desktop,
            features: vec![
                "lsx".to_string(),
                "lasx".to_string(),
            ],
            disabled_features: Vec::new(),
            opt_level: OptLevel::O2,
            lto: false,
            debug: true,
        }
    }

    /// Create LoongArch server target
    pub fn loongarch_server() -> Self {
        Self {
            name: "loongarch-server".to_string(),
            arch_type: ArchType::LoongArch64,
            device_type: DeviceType::Server,
            features: vec![
                "lsx".to_string(),
                "lasx".to_string(),
                "lbt".to_string(), // Binary translation
            ],
            disabled_features: Vec::new(),
            opt_level: OptLevel::O3,
            lto: true,
            debug: false,
        }
    }
}

// ============================================================================
// Module Configuration
// ============================================================================

/// Module configuration
#[derive(Debug, Clone)]
pub struct ModuleConfig {
    /// Module name
    pub name: String,
    /// Whether enabled
    pub enabled: bool,
    /// Dependent modules
    pub dependencies: Vec<String>,
    /// Conditional compilation
    pub conditions: Vec<CompileCondition>,
}

/// Compilation condition
#[derive(Debug, Clone)]
pub enum CompileCondition {
    /// Architecture condition
    Arch(ArchType),
    /// Device type condition
    DeviceType(DeviceType),
    /// Feature condition
    Feature(String),
    /// Not
    Not(Box<CompileCondition>),
    /// And
    And(Box<CompileCondition>, Box<CompileCondition>),
    /// Or
    Or(Box<CompileCondition>, Box<CompileCondition>),
}

impl CompileCondition {
    /// Evaluate condition
    pub fn evaluate(&self, target: &BuildTarget) -> bool {
        match self {
            CompileCondition::Arch(arch) => target.arch_type == *arch,
            CompileCondition::DeviceType(dt) => target.device_type == *dt,
            CompileCondition::Feature(f) => target.features.contains(f),
            CompileCondition::Not(c) => !c.evaluate(target),
            CompileCondition::And(a, b) => a.evaluate(target) && b.evaluate(target),
            CompileCondition::Or(a, b) => a.evaluate(target) || b.evaluate(target),
        }
    }
}

// ============================================================================
// Build Configuration Manager
// ============================================================================

/// Build configuration manager
pub struct BuildConfigManager {
    /// Target configurations
    targets: BTreeMap<String, BuildTarget>,
    /// Module configurations
    modules: BTreeMap<String, ModuleConfig>,
    /// Current target
    current_target: Option<String>,
}

impl BuildConfigManager {
    /// Create new configuration manager (const, without defaults)
    pub const fn new() -> Self {
        Self {
            targets: BTreeMap::new(),
            modules: BTreeMap::new(),
            current_target: None,
        }
    }

    /// Initialize with default targets and modules
    pub fn init(&mut self) {
        // Register default targets
        self.register_default_targets();
        self.register_default_modules();
    }

    /// Register default targets
    fn register_default_targets(&mut self) {
        self.targets.insert("arm64-mobile".to_string(), BuildTarget::arm64_mobile());
        self.targets.insert("arm64-server".to_string(), BuildTarget::arm64_server());
        self.targets.insert("x64-desktop".to_string(), BuildTarget::x64_desktop());
        self.targets.insert("x64-server".to_string(), BuildTarget::x64_server());
        self.targets.insert("loongarch-desktop".to_string(), BuildTarget::loongarch_desktop());
        self.targets.insert("loongarch-server".to_string(), BuildTarget::loongarch_server());
    }

    /// Register default modules
    fn register_default_modules(&mut self) {
        // Core modules (always enabled)
        self.modules.insert("kernel".to_string(), ModuleConfig {
            name: "kernel".to_string(),
            enabled: true,
            dependencies: Vec::new(),
            conditions: Vec::new(),
        });

        // HAL modules
        self.modules.insert("hal-arm64".to_string(), ModuleConfig {
            name: "hal-arm64".to_string(),
            enabled: true,
            dependencies: vec!["kernel".to_string()],
            conditions: vec![CompileCondition::Arch(ArchType::Arm64)],
        });

        self.modules.insert("hal-x64".to_string(), ModuleConfig {
            name: "hal-x64".to_string(),
            enabled: true,
            dependencies: vec!["kernel".to_string()],
            conditions: vec![CompileCondition::Arch(ArchType::X64)],
        });

        self.modules.insert("hal-loongarch".to_string(), ModuleConfig {
            name: "hal-loongarch".to_string(),
            enabled: true,
            dependencies: vec!["kernel".to_string()],
            conditions: vec![CompileCondition::Arch(ArchType::LoongArch64)],
        });

        // NPU modules (mobile devices only)
        self.modules.insert("npu".to_string(), ModuleConfig {
            name: "npu".to_string(),
            enabled: true,
            dependencies: vec!["kernel".to_string()],
            conditions: vec![CompileCondition::DeviceType(DeviceType::Mobile)],
        });

        // Virtualization modules (servers only)
        self.modules.insert("virtualization".to_string(), ModuleConfig {
            name: "virtualization".to_string(),
            enabled: true,
            dependencies: vec!["kernel".to_string()],
            conditions: vec![CompileCondition::DeviceType(DeviceType::Server)],
        });
    }

    /// Set current target
    pub fn set_target(&mut self, name: &str) -> Result<(), BuildConfigError> {
        if self.targets.contains_key(name) {
            self.current_target = Some(name.to_string());
            Ok(())
        } else {
            Err(BuildConfigError::TargetNotFound(name.to_string()))
        }
    }

    /// Get current target
    pub fn get_current_target(&self) -> Option<&BuildTarget> {
        self.current_target.as_ref().and_then(|name| self.targets.get(name))
    }

    /// Get enabled module list
    pub fn get_enabled_modules(&self) -> Vec<&ModuleConfig> {
        let target = match self.get_current_target() {
            Some(t) => t,
            None => return Vec::new(),
        };

        self.modules.values()
            .filter(|m| {
                if !m.enabled {
                    return false;
                }
                if m.conditions.is_empty() {
                    return true;
                }
                m.conditions.iter().all(|c| c.evaluate(target))
            })
            .collect()
    }

    /// Check if module is enabled
    pub fn is_module_enabled(&self, name: &str) -> bool {
        let target = match self.get_current_target() {
            Some(t) => t,
            None => return false,
        };

        self.modules.get(name).map(|m| {
            if !m.enabled {
                return false;
            }
            if m.conditions.is_empty() {
                return true;
            }
            m.conditions.iter().all(|c| c.evaluate(target))
        }).unwrap_or(false)
    }

    /// Generate Cargo features
    pub fn generate_cargo_features(&self) -> Vec<String> {
        let mut features = Vec::new();

        if let Some(target) = self.get_current_target() {
            // Architecture feature
            features.push(target.arch_type.to_string());

            // Device type feature
            features.push(target.device_type.to_string());

            // Enabled features
            features.extend(target.features.clone());
        }

        // Enabled modules
        for module in self.get_enabled_modules() {
            features.push(module.name.clone());
        }

        features
    }

    /// Generate build command
    pub fn generate_build_command(&self) -> Option<String> {
        let target = self.get_current_target()?;
        let features = self.generate_cargo_features();

        let mut cmd = alloc::format!(
            "cargo build --target {}-unknown-nuva",
            target.arch_type
        );

        if !features.is_empty() {
            cmd.push_str(" --features ");
            cmd.push_str(&features.join(","));
        }

        if target.debug {
            cmd.push_str(" --debug");
        } else {
            cmd.push_str(&alloc::format!(" -O{}", target.opt_level));
        }

        if target.lto {
            cmd.push_str(" --lto");
        }

        Some(cmd)
    }
}

impl Default for BuildConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Build configuration error
#[derive(Debug)]
pub enum BuildConfigError {
    /// Target not found
    TargetNotFound(String),
    /// Module not found
    ModuleNotFound(String),
    /// Circular dependency
    CircularDependency(String),
}

impl fmt::Display for BuildConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BuildConfigError::TargetNotFound(name) => write!(f, "Target not found: {}", name),
            BuildConfigError::ModuleNotFound(name) => write!(f, "Module not found: {}", name),
            BuildConfigError::CircularDependency(msg) => write!(f, "Circular dependency: {}", msg),
        }
    }
}

// ============================================================================
// Global Configuration Manager
// ============================================================================

use spin::Mutex;

/// Global build configuration manager
static BUILD_CONFIG: Mutex<BuildConfigManager> = Mutex::new(BuildConfigManager::new());

/// Get build configuration manager
pub fn build_config() -> &'static Mutex<BuildConfigManager> {
    &BUILD_CONFIG
}

/// Initialize build configuration
pub fn init_build_config() {
    let config = BUILD_CONFIG.lock();
    log_info!("Build configuration initialized");
    log_info!("  Available targets: {}", config.targets.len());
}
