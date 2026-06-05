/*
 * Nuva OS - Hal - Ffi - Mod
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
 * HAL FFI Module - Foreign Function Interface
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * This module provides C/C++ compatible interfaces for HAL,
 * enabling driver development in C and C++.
 */

pub mod stability;

pub mod c_api {
    pub mod bindings;
}

pub mod cpp_api;

// Re-export main types
pub use stability::{
    ApiVersion, ApiFunction, ApiStruct, ApiField,
    ApiStabilityChecker, StabilityResult, CompatibilityReport,
    LayoutValidationResult,
};

/// Initialize HAL FFI subsystem
pub fn init_hal_ffi() -> Result<(), &'static str> {
    // Initialize API stability checker
    let checker = stability::define_hal_api();

    // Validate all API functions
    let report = checker.check_compatibility();
    if !report.is_compatible() {
        return Err("HAL API compatibility check failed");
    }
    
    // Validate struct layouts for ABI stability
    let layout_result = checker.validate_layouts();
    if !matches!(layout_result, stability::LayoutValidationResult::Valid) {
        return Err("HAL API layout validation failed");
    }

    Ok(())
}
