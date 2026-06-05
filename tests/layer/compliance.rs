/*
 * Nuva OS - Tests - Layer - Compliance
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
 * Layer Compliance Tests
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * This module provides comprehensive tests for verifying
 * layer boundary compliance in the Nuva OS architecture.
 */

#[cfg(test)]
mod tests {
    use crate::kernel::arch::layer::*;

    /// Test layer ordering
    #[test]
    fn test_layer_ordering() {
        // HAL is the lowest layer
        assert!(Layer::Hal.is_lower_than(&Layer::Kernel));
        assert!(Layer::Hal.is_lower_than(&Layer::Lib));
        assert!(Layer::Hal.is_lower_than(&Layer::Services));
        assert!(Layer::Hal.is_lower_than(&Layer::Application));

        // Kernel is above HAL
        assert!(Layer::Kernel.is_higher_than(&Layer::Hal));
        assert!(Layer::Kernel.is_lower_than(&Layer::Lib));

        // Lib is above Kernel
        assert!(Layer::Lib.is_higher_than(&Layer::Kernel));
        assert!(Layer::Lib.is_lower_than(&Layer::Services));

        // Services is above Lib
        assert!(Layer::Services.is_higher_than(&Layer::Lib));
        assert!(Layer::Services.is_lower_than(&Layer::Application));

        // Application is the highest layer
        assert!(Layer::Application.is_higher_than(&Layer::Services));
        assert!(Layer::Application.is_higher_than(&Layer::Lib));
        assert!(Layer::Application.is_higher_than(&Layer::Kernel));
        assert!(Layer::Application.is_higher_than(&Layer::Hal));
    }

    /// Test layer dependency rules
    #[test]
    fn test_layer_dependencies() {
        // HAL cannot depend on any layer
        assert!(!Layer::Hal.can_depend_on(&Layer::Kernel));
        assert!(!Layer::Hal.can_depend_on(&Layer::Lib));
        assert!(!Layer::Hal.can_depend_on(&Layer::Services));
        assert!(!Layer::Hal.can_depend_on(&Layer::Application));

        // Kernel can only depend on HAL
        assert!(Layer::Kernel.can_depend_on(&Layer::Hal));
        assert!(Layer::Kernel.can_depend_on(&Layer::Kernel));
        assert!(!Layer::Kernel.can_depend_on(&Layer::Lib));
        assert!(!Layer::Kernel.can_depend_on(&Layer::Services));
        assert!(!Layer::Kernel.can_depend_on(&Layer::Application));

        // Lib can depend on Kernel and HAL
        assert!(Layer::Lib.can_depend_on(&Layer::Hal));
        assert!(Layer::Lib.can_depend_on(&Layer::Kernel));
        assert!(Layer::Lib.can_depend_on(&Layer::Lib));
        assert!(!Layer::Lib.can_depend_on(&Layer::Services));
        assert!(!Layer::Lib.can_depend_on(&Layer::Application));

        // Services can depend on Lib and Kernel
        assert!(Layer::Services.can_depend_on(&Layer::Hal));
        assert!(Layer::Services.can_depend_on(&Layer::Kernel));
        assert!(Layer::Services.can_depend_on(&Layer::Lib));
        assert!(Layer::Services.can_depend_on(&Layer::Services));
        assert!(!Layer::Services.can_depend_on(&Layer::Application));

        // Application can depend on all layers
        assert!(Layer::Application.can_depend_on(&Layer::Hal));
        assert!(Layer::Application.can_depend_on(&Layer::Kernel));
        assert!(Layer::Application.can_depend_on(&Layer::Lib));
        assert!(Layer::Application.can_depend_on(&Layer::Services));
        assert!(Layer::Application.can_depend_on(&Layer::Application));
    }

    /// Test layer boundary checker
    #[test]
    fn test_layer_boundary_checker() {
        let checker = LayerBoundaryChecker::new();

        // Valid dependencies
        assert!(checker
            .check_dependency("kernel::mm", "hal::cpu", Layer::Kernel, Layer::Hal)
            .is_ok());
        assert!(checker
            .check_dependency("lib::ai", "kernel::api", Layer::Lib, Layer::Kernel)
            .is_ok());
        assert!(checker
            .check_dependency("services::net", "lib::net", Layer::Services, Layer::Lib)
            .is_ok());

        // Invalid dependencies (upward)
        assert!(checker
            .check_dependency("hal::cpu", "kernel::mm", Layer::Hal, Layer::Kernel)
            .is_err());
        assert!(checker
            .check_dependency("kernel::mm", "lib::ai", Layer::Kernel, Layer::Lib)
            .is_err());
        assert!(checker
            .check_dependency("lib::ai", "services::net", Layer::Lib, Layer::Services)
            .is_err());
    }

    /// Test layer registry
    #[test]
    fn test_layer_registry() {
        let registry = LayerRegistry::new();

        // Check HAL config
        let hal_config = registry.get_config(Layer::Hal).unwrap();
        assert_eq!(hal_config.layer, Layer::Hal);
        assert!(hal_config.allowed_deps.is_empty());
        assert_eq!(hal_config.visibility, Visibility::Public);

        // Check Kernel config
        let kernel_config = registry.get_config(Layer::Kernel).unwrap();
        assert_eq!(kernel_config.layer, Layer::Kernel);
        assert!(kernel_config.allowed_deps.contains(&Layer::Hal));
        assert_eq!(kernel_config.visibility, Visibility::Restricted);

        // Check Lib config
        let lib_config = registry.get_config(Layer::Lib).unwrap();
        assert_eq!(lib_config.layer, Layer::Lib);
        assert!(lib_config.allowed_deps.contains(&Layer::Kernel));
        assert!(lib_config.allowed_deps.contains(&Layer::Hal));
        assert_eq!(lib_config.visibility, Visibility::Public);

        // Check Services config
        let services_config = registry.get_config(Layer::Services).unwrap();
        assert_eq!(services_config.layer, Layer::Services);
        assert!(services_config.allowed_deps.contains(&Layer::Lib));
        assert!(services_config.allowed_deps.contains(&Layer::Kernel));
        assert_eq!(services_config.visibility, Visibility::Restricted);

        // Check Application config
        let app_config = registry.get_config(Layer::Application).unwrap();
        assert_eq!(app_config.layer, Layer::Application);
        assert!(app_config.allowed_deps.contains(&Layer::Services));
        assert!(app_config.allowed_deps.contains(&Layer::Lib));
        assert_eq!(app_config.visibility, Visibility::Public);
    }

    /// Test dependency validation
    #[test]
    fn test_dependency_validation() {
        let registry = LayerRegistry::new();

        // Valid dependencies
        assert!(registry.is_dependency_allowed(Layer::Kernel, Layer::Hal));
        assert!(registry.is_dependency_allowed(Layer::Lib, Layer::Kernel));
        assert!(registry.is_dependency_allowed(Layer::Lib, Layer::Hal));
        assert!(registry.is_dependency_allowed(Layer::Services, Layer::Lib));
        assert!(registry.is_dependency_allowed(Layer::Services, Layer::Kernel));
        assert!(registry.is_dependency_allowed(Layer::Application, Layer::Services));
        assert!(registry.is_dependency_allowed(Layer::Application, Layer::Lib));

        // Invalid dependencies
        assert!(!registry.is_dependency_allowed(Layer::Hal, Layer::Kernel));
        assert!(!registry.is_dependency_allowed(Layer::Hal, Layer::Lib));
        assert!(!registry.is_dependency_allowed(Layer::Kernel, Layer::Lib));
        assert!(!registry.is_dependency_allowed(Layer::Kernel, Layer::Services));
        assert!(!registry.is_dependency_allowed(Layer::Lib, Layer::Services));
        assert!(!registry.is_dependency_allowed(Layer::Services, Layer::Application));
    }

    /// Test layer guard
    #[test]
    fn test_layer_guard() {
        let guard = LayerGuard::new(Layer::Kernel, "kernel::mm");

        assert_eq!(guard.layer(), Layer::Kernel);
        assert_eq!(guard.module(), "kernel::mm");
    }

    /// Test violation type display
    #[test]
    fn test_violation_display() {
        let violation = ViolationType::UpwardDependency;
        assert_eq!(format!("{}", violation), "Upward dependency violation");

        let violation = ViolationType::DirectCrossLayer;
        assert_eq!(format!("{}", violation), "Direct cross-layer dependency");

        let violation = ViolationType::CircularDependency;
        assert_eq!(format!("{}", violation), "Circular dependency");

        let violation = ViolationType::InvalidAccess;
        assert_eq!(format!("{}", violation), "Invalid module access");
    }

    /// Test layer display
    #[test]
    fn test_layer_display() {
        assert_eq!(format!("{}", Layer::Hal), "HAL");
        assert_eq!(format!("{}", Layer::Kernel), "Kernel");
        assert_eq!(format!("{}", Layer::Lib), "Lib");
        assert_eq!(format!("{}", Layer::Services), "Services");
        assert_eq!(format!("{}", Layer::Application), "Application");
    }

    /// Test layer level
    #[test]
    fn test_layer_level() {
        assert_eq!(Layer::Hal.level(), 0);
        assert_eq!(Layer::Kernel.level(), 1);
        assert_eq!(Layer::Lib.level(), 2);
        assert_eq!(Layer::Services.level(), 3);
        assert_eq!(Layer::Application.level(), 4);
    }
}
