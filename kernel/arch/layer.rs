/*
 * Nuva OS - Kernel - Arch - Layer
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
 * Layer Boundary API - Architecture Enforcement
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * This module provides runtime layer boundary checking and
 * enforcement for the Nuva OS layered architecture.
 */

use core::fmt;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

/// Architecture layer enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Layer {
    /// Hardware Abstraction Layer (Layer 0)
    Hal = 0,

    /// Kernel Layer (Layer 1)
    Kernel = 1,

    /// Library Layer (Layer 2)
    Lib = 2,

    /// Services Layer (Layer 3)
    Services = 3,

    /// Application Layer (Layer 4)
    Application = 4,
}

impl Layer {
    /// Get layer name
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Hal => "HAL",
            Self::Kernel => "Kernel",
            Self::Lib => "Lib",
            Self::Services => "Services",
            Self::Application => "Application",
        }
    }

    /// Get layer level (0 = lowest)
    pub const fn level(&self) -> u32 {
        *self as u32
    }

    /// Check if this layer can depend on another layer
    pub const fn can_depend_on(&self, other: &Self) -> bool {
        // A layer can only depend on lower or same layers
        // with special rules for HAL access
        self.level() >= other.level()
    }

    /// Check if this layer is lower than another
    pub const fn is_lower_than(&self, other: &Self) -> bool {
        self.level() < other.level()
    }

    /// Check if this layer is higher than another
    pub const fn is_higher_than(&self, other: &Self) -> bool {
        self.level() > other.level()
    }
}

impl fmt::Display for Layer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Layer boundary violation
#[derive(Debug, Clone)]
pub struct LayerViolation {
    /// Source module
    pub from_module: String,

    /// Target module
    pub to_module: String,

    /// Source layer
    pub from_layer: Layer,

    /// Target layer
    pub to_layer: Layer,

    /// Violation type
    pub violation_type: ViolationType,
}

/// Violation type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViolationType {
    /// Lower layer depending on higher layer
    UpwardDependency,

    /// Direct cross-layer dependency without abstraction
    DirectCrossLayer,

    /// Circular dependency
    CircularDependency,

    /// Invalid module access
    InvalidAccess,
}

impl fmt::Display for ViolationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UpwardDependency => write!(f, "Upward dependency violation"),
            Self::DirectCrossLayer => write!(f, "Direct cross-layer dependency"),
            Self::CircularDependency => write!(f, "Circular dependency"),
            Self::InvalidAccess => write!(f, "Invalid module access"),
        }
    }
}

/// Layer boundary checker
pub struct LayerBoundaryChecker {
    /// Enable runtime checks
    enabled: bool,

    /// Violation callback
    on_violation: Option<fn(&LayerViolation)>,
}

impl LayerBoundaryChecker {
    /// Create new layer boundary checker
    pub const fn new() -> Self {
        Self {
            enabled: true,
            on_violation: None,
        }
    }

    /// Enable or disable checks
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Set violation callback
    pub fn set_violation_callback(&mut self, callback: fn(&LayerViolation)) {
        self.on_violation = Some(callback);
    }

    /// Check if dependency is allowed
    pub fn check_dependency(
        &self,
        from_module: &str,
        to_module: &str,
        from_layer: Layer,
        to_layer: Layer,
    ) -> Result<(), LayerViolation> {
        if !self.enabled {
            return Ok(());
        }

        // Check for upward dependency
        if from_layer.is_lower_than(&to_layer) {
            let violation = LayerViolation {
                from_module: String::from(from_module),
                to_module: String::from(to_module),
                from_layer,
                to_layer,
                violation_type: ViolationType::UpwardDependency,
            };

            if let Some(callback) = self.on_violation {
                callback(&violation);
            }

            return Err(violation);
        }

        Ok(())
    }

    /// Check module access
    pub fn check_access(
        &self,
        accessor_layer: Layer,
        target_module: &str,
        target_layer: Layer,
    ) -> Result<(), LayerViolation> {
        if !self.enabled {
            return Ok(());
        }

        // HAL modules can only be accessed through traits
        if target_layer == Layer::Hal && accessor_layer != Layer::Kernel {
            // Lib and Services can access HAL through traits only
            if accessor_layer == Layer::Lib || accessor_layer == Layer::Services {
                // This is allowed if using trait abstraction
                // The actual check happens at compile time
                return Ok(());
            }

            let violation = LayerViolation {
                from_module: String::new(),
                to_module: String::from(target_module),
                from_layer: accessor_layer,
                to_layer: target_layer,
                violation_type: ViolationType::InvalidAccess,
            };

            if let Some(callback) = self.on_violation {
                callback(&violation);
            }

            return Err(violation);
        }

        Ok(())
    }
}

/// Global layer boundary checker
static LAYER_CHECKER: crate::sync_oncelock::OnceLock<LayerBoundaryChecker> = crate::sync_oncelock::OnceLock::new();

/// Initialize layer boundary checker
pub fn init_layer_checker() {
    // Safety: Single-threaded initialization
    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe {
        LAYER_CHECKER.set_violation_callback(default_violation_handler);
    }
}

/// Default violation handler
fn default_violation_handler(violation: &LayerViolation) {
    // Log violation
    crate::log_error!(
        "Layer violation: {} -> {} ({})",
        violation.from_layer,
        violation.to_layer,
        violation.violation_type
    );
}

/// Check dependency (convenience function)
pub fn check_dependency(
    from_module: &str,
    to_module: &str,
    from_layer: Layer,
    to_layer: Layer,
) -> Result<(), LayerViolation> {
    // Safety: Read-only access
    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe { LAYER_CHECKER.check_dependency(from_module, to_module, from_layer, to_layer) }
}

/// Layer guard - RAII for layer context
pub struct LayerGuard {
    layer: Layer,
    module: String,
}

impl LayerGuard {
    /// Create layer guard
    pub fn new(layer: Layer, module: &str) -> Self {
        Self {
            layer,
            module: String::from(module),
        }
    }

    /// Get current layer
    pub fn layer(&self) -> Layer {
        self.layer
    }

    /// Get module name
    pub fn module(&self) -> &str {
        &self.module
    }
}

/// Layer-specific configuration
#[derive(Debug, Clone)]
pub struct LayerConfig {
    /// Layer
    pub layer: Layer,

    /// Allowed dependencies
    pub allowed_deps: alloc::vec::Vec<Layer>,

    /// Visibility restrictions
    pub visibility: Visibility,

    /// Build options
    pub build_options: BuildOptions,
}

/// Visibility settings
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    /// Public to all layers
    Public,

    /// Restricted to specific layers
    Restricted,

    /// Private to the layer
    Private,
}

/// Build options
#[derive(Debug, Clone)]
pub struct BuildOptions {
    /// Optimization level
    pub opt_level: u32,

    /// Enable LTO
    pub lto: bool,

    /// Debug assertions
    pub debug_assertions: bool,
}

impl Default for BuildOptions {
    fn default() -> Self {
        Self {
            opt_level: 2,
            lto: true,
            debug_assertions: false,
        }
    }
}

/// Layer registry
pub struct LayerRegistry {
    /// Layer configurations
    configs: alloc::collections::BTreeMap<Layer, LayerConfig>,
}

impl LayerRegistry {
    /// Create new layer registry
    pub fn new() -> Self {
        let mut configs = alloc::collections::BTreeMap::new();

        // HAL layer config
        configs.insert(
            Layer::Hal,
            LayerConfig {
                layer: Layer::Hal,
                allowed_deps: alloc::vec![],
                visibility: Visibility::Public,
                build_options: BuildOptions::default(),
            },
        );

        // Kernel layer config
        configs.insert(
            Layer::Kernel,
            LayerConfig {
                layer: Layer::Kernel,
                allowed_deps: alloc::vec![Layer::Hal],
                visibility: Visibility::Restricted,
                build_options: BuildOptions::default(),
            },
        );

        // Lib layer config
        configs.insert(
            Layer::Lib,
            LayerConfig {
                layer: Layer::Lib,
                allowed_deps: alloc::vec![Layer::Kernel, Layer::Hal],
                visibility: Visibility::Public,
                build_options: BuildOptions::default(),
            },
        );

        // Services layer config
        configs.insert(
            Layer::Services,
            LayerConfig {
                layer: Layer::Services,
                allowed_deps: alloc::vec![Layer::Lib, Layer::Kernel],
                visibility: Visibility::Restricted,
                build_options: BuildOptions::default(),
            },
        );

        // Application layer config
        configs.insert(
            Layer::Application,
            LayerConfig {
                layer: Layer::Application,
                allowed_deps: alloc::vec![Layer::Services, Layer::Lib],
                visibility: Visibility::Public,
                build_options: BuildOptions::default(),
            },
        );

        Self { configs }
    }

    /// Get layer configuration
    pub fn get_config(&self, layer: Layer) -> Option<&LayerConfig> {
        self.configs.get(&layer)
    }

    /// Check if dependency is allowed
    pub fn is_dependency_allowed(&self, from: Layer, to: Layer) -> bool {
        if let Some(config) = self.get_config(from) {
            config.allowed_deps.contains(&to)
        } else {
            false
        }
    }
}
