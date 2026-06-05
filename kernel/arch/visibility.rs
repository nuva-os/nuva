/*
 * Nuva OS - Kernel - Arch - Visibility
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
 * Layer Visibility Control
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * This module implements visibility restrictions for layers,
 * controlling which modules and APIs are accessible from each layer.
 */

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;
use spin::RwLock;

use super::layer::Layer;

/// Visibility level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum VisibilityLevel {
    /// Public - visible to all layers
    Public,

    /// Protected - visible to same and higher layers
    Protected,

    /// Internal - visible only to same layer
    Internal,

    /// Private - visible only to containing module
    Private,
}

impl VisibilityLevel {
    /// Check if this visibility allows access from another layer
    pub fn allows_access_from(&self, source_layer: Layer, target_layer: Layer) -> bool {
        match self {
            // Public: always accessible
            Self::Public => true,

            // Protected: accessible from same or higher layers
            Self::Protected => source_layer >= target_layer,

            // Internal: accessible only from same layer
            Self::Internal => source_layer == target_layer,

            // Private: never accessible from other modules
            Self::Private => false,
        }
    }
}

/// Module visibility rule
#[derive(Debug, Clone)]
pub struct VisibilityRule {
    /// Module path
    pub module_path: String,

    /// Visibility level
    pub visibility: VisibilityLevel,

    /// Allowed layers (if restricted)
    pub allowed_layers: Vec<Layer>,

    /// Exported symbols
    pub exported_symbols: Vec<String>,
}

/// Visibility manager
pub struct VisibilityManager {
    /// Visibility rules per module
    rules: RwLock<BTreeMap<String, VisibilityRule>>,

    /// Layer visibility defaults
    layer_defaults: BTreeMap<Layer, VisibilityLevel>,
}

impl VisibilityManager {
    /// Create new visibility manager
    pub fn new() -> Self {
        let mut layer_defaults = BTreeMap::new();

        // Set default visibility for each layer
        layer_defaults.insert(Layer::Hal, VisibilityLevel::Public);
        layer_defaults.insert(Layer::Kernel, VisibilityLevel::Protected);
        layer_defaults.insert(Layer::Lib, VisibilityLevel::Public);
        layer_defaults.insert(Layer::Services, VisibilityLevel::Protected);
        layer_defaults.insert(Layer::Application, VisibilityLevel::Public);

        Self {
            rules: RwLock::new(BTreeMap::new()),
            layer_defaults,
        }
    }

    /// Register visibility rule
    pub fn register_rule(&self, rule: VisibilityRule) {
        let mut rules = self.rules.write();
        rules.insert(rule.module_path.clone(), rule);
    }

    /// Check if module is accessible
    pub fn is_accessible(
        &self,
        source_layer: Layer,
        source_module: &str,
        target_module: &str,
    ) -> bool {
        let rules = self.rules.read();

        // Check if there's a specific rule for the target module
        if let Some(rule) = rules.get(target_module) {
            // Check allowed layers first
            if !rule.allowed_layers.is_empty() && !rule.allowed_layers.contains(&source_layer) {
                return false;
            }

            // Check visibility level
            let target_layer = Self::get_layer_for_module(target_module);
            return rule
                .visibility
                .allows_access_from(source_layer, target_layer);
        }

        // Use layer default visibility
        let target_layer = Self::get_layer_for_module(target_module);
        if let Some(default_visibility) = self.layer_defaults.get(&target_layer) {
            return default_visibility.allows_access_from(source_layer, target_layer);
        }

        // Default: allow access
        true
    }

    /// Check if symbol is exported
    pub fn is_symbol_exported(&self, module: &str, symbol: &str) -> bool {
        let rules = self.rules.read();

        if let Some(rule) = rules.get(module) {
            // If exported_symbols is empty, all symbols are exported
            if rule.exported_symbols.is_empty() {
                return true;
            }

            return rule.exported_symbols.contains(&String::from(symbol));
        }

        // Default: all symbols exported
        true
    }

    /// Get layer for module path
    fn get_layer_for_module(module: &str) -> Layer {
        if module.starts_with("hal::") {
            Layer::Hal
        } else if module.starts_with("kernel::") {
            Layer::Kernel
        } else if module.starts_with("lib::") {
            Layer::Lib
        } else if module.starts_with("services::") {
            Layer::Services
        } else if module.starts_with("application::") {
            Layer::Application
        } else {
            Layer::Application // Default to highest layer
        }
    }

    /// Get visibility for module
    pub fn get_visibility(&self, module: &str) -> VisibilityLevel {
        let rules = self.rules.read();

        if let Some(rule) = rules.get(module) {
            return rule.visibility;
        }

        let layer = Self::get_layer_for_module(module);
        if let Some(default) = self.layer_defaults.get(&layer) {
            return *default;
        }

        VisibilityLevel::Public
    }
}

/// Global visibility manager
static VISIBILITY_MANAGER: core::sync::OnceLock<VisibilityManager> = core::sync::OnceLock::new();

/// Initialize visibility manager
pub fn init_visibility_manager() {
    // Safety: Single-threaded initialization
    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe {
        VISIBILITY_MANAGER = Some(VisibilityManager::new());
    }
}

/// Get visibility manager
pub fn visibility_manager() -> &'static VisibilityManager {
    // Safety: Initialized once, read-only access
    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe {
        VISIBILITY_MANAGER
            .as_ref()
            .ok_or(KernelError::NotInitialized)?
    }
}

/// Check module access (convenience function)
pub fn check_module_access(source_layer: Layer, source_module: &str, target_module: &str) -> bool {
    visibility_manager().is_accessible(source_layer, source_module, target_module)
}

/// Check symbol access (convenience function)
pub fn check_symbol_access(module: &str, symbol: &str) -> bool {
    visibility_manager().is_symbol_exported(module, symbol)
}

/// Visibility attribute macro
/// Usage:
/// ```rust
/// #[layer_visibility("internal")]
/// pub fn internal_function() { }
/// ```
#[macro_export]
macro_rules! layer_visibility {
    ("public") => {
        // Public visibility - no restrictions
    };

    ("protected") => {
        // Protected visibility - compile-time check
        // TODO: Implement compile-time visibility check
    };

    ("internal") => {
        // Internal visibility - same layer only
        // TODO: Implement compile-time visibility check
    };

    ("private") => {
        // Private visibility - module only
        // TODO: Implement compile-time visibility check
    };
}

/// Export control macro
/// Usage:
/// ```rust
/// layer_export!(kernel::api, [process_create, process_exit]);
/// ```
#[macro_export]
macro_rules! layer_export {
    ($module:path, [$($symbol:ident),*]) => {
        // Register exported symbols
        // TODO: Implement symbol registration
    };
}

/// Layer API macro - marks API as stable and exported
/// Usage:
/// ```rust
/// #[layer_api(stable = "1.0.0")]
/// pub fn public_api() { }
/// ```
#[macro_export]
macro_rules! layer_api {
    (stable = $version:expr) => {
        // Mark as stable API
        // TODO: Implement API versioning
    };

    (unstable) => {
        // Mark as unstable API
        // TODO: Implement API versioning
    };

    (deprecated = $version:expr) => {
        // Mark as deprecated API
        // TODO: Implement API versioning
    };
}
