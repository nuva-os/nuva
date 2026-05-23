/*
 * API Stability Test Framework
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * This module provides comprehensive API stability testing
 * for HAL C/C++ interfaces.
 */

use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use spin::RwLock;

/// API version information
#[derive(Debug, Clone)]
pub struct ApiVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub abi_version: u32,
}

impl ApiVersion {
    pub fn new(major: u32, minor: u32, patch: u32, abi_version: u32) -> Self {
        Self {
            major,
            minor,
            patch,
            abi_version,
        }
    }

    pub fn from_u32(version: u32) -> Self {
        Self {
            major: (version >> 16) & 0xFF,
            minor: (version >> 8) & 0xFF,
            patch: version & 0xFF,
            abi_version: version,
        }
    }

    pub fn to_u32(&self) -> u32 {
        (self.major << 16) | (self.minor << 8) | self.patch
    }

    pub fn is_compatible(&self, other: &ApiVersion) -> bool {
        // Major version must match
        // Minor version must be <=
        self.major == other.major && self.minor <= other.minor
    }
}

/// API function signature
#[derive(Debug, Clone)]
pub struct ApiFunction {
    pub name: String,
    pub return_type: String,
    pub params: Vec<String>,
    pub version_added: ApiVersion,
    pub version_deprecated: Option<ApiVersion>,
    pub is_stable: bool,
}

/// API structure definition
#[derive(Debug, Clone)]
pub struct ApiStruct {
    pub name: String,
    pub fields: Vec<ApiField>,
    pub size: usize,
    pub alignment: usize,
    pub version_added: ApiVersion,
    pub is_stable: bool,
}

/// API field definition
#[derive(Debug, Clone)]
pub struct ApiField {
    pub name: String,
    pub type_name: String,
    pub offset: usize,
    pub size: usize,
}

/// API stability checker
pub struct ApiStabilityChecker {
    /// Current API version
    current_version: ApiVersion,

    /// Registered functions
    functions: RwLock<BTreeMap<String, ApiFunction>>,

    /// Registered structures
    structures: RwLock<BTreeMap<String, ApiStruct>>,

    /// Compatibility matrix
    compatibility: RwLock<BTreeMap<(String, String), bool>>,
}

impl ApiStabilityChecker {
    /// Create new API stability checker
    /// @param version: Current API version
    pub fn new(version: ApiVersion) -> Self {
        Self {
            current_version: version,
            functions: RwLock::new(BTreeMap::new()),
            structures: RwLock::new(BTreeMap::new()),
            compatibility: RwLock::new(BTreeMap::new()),
        }
    }

    /// Register API function
    /// @param function: Function definition
    pub fn register_function(&self, function: ApiFunction) {
        let mut functions = self.functions.write();
        functions.insert(function.name.clone(), function);
    }

    /// Register API structure
    /// @param structure: Structure definition
    pub fn register_structure(&self, structure: ApiStruct) {
        let mut structures = self.structures.write();
        structures.insert(structure.name.clone(), structure);
    }

    /// Check function stability
    /// @param name: Function name
    /// @return: Stability result
    pub fn check_function_stability(&self, name: &str) -> StabilityResult {
        let functions = self.functions.read();

        if let Some(func) = functions.get(name) {
            // Check if function is stable
            if !func.is_stable {
                return StabilityResult::Unstable {
                    reason: String::from("Function marked as unstable"),
                };
            }

            // Check if function is deprecated
            if let Some(deprecated) = &func.version_deprecated {
                if self.current_version.is_compatible(deprecated) {
                    return StabilityResult::Deprecated {
                        since: deprecated.clone(),
                        alternative: None,
                    };
                }
            }

            StabilityResult::Stable
        } else {
            StabilityResult::NotFound
        }
    }

    /// Check structure stability
    /// @param name: Structure name
    /// @return: Stability result
    pub fn check_structure_stability(&self, name: &str) -> StabilityResult {
        let structures = self.structures.read();

        if let Some(structure) = structures.get(name) {
            // Check if structure is stable
            if !structure.is_stable {
                return StabilityResult::Unstable {
                    reason: String::from("Structure marked as unstable"),
                };
            }

            StabilityResult::Stable
        } else {
            StabilityResult::NotFound
        }
    }

    /// Check ABI compatibility
    /// @param old_version: Old API version
    /// @param new_version: New API version
    /// @return: Compatibility report
    pub fn check_abi_compatibility(
        &self,
        old_version: &ApiVersion,
        new_version: &ApiVersion,
    ) -> CompatibilityReport {
        let mut report = CompatibilityReport::new();

        // Check version compatibility
        if !old_version.is_compatible(new_version) {
            report.add_error(format!(
                "Incompatible versions: {} -> {}",
                old_version.to_u32(),
                new_version.to_u32()
            ));
        }

        // Check all functions
        let functions = self.functions.read();
        for (name, func) in functions.iter() {
            if let Some(deprecated) = &func.version_deprecated {
                if new_version.is_compatible(deprecated) {
                    report.add_warning(format!(
                        "Function {} is deprecated since version {}",
                        name,
                        deprecated.to_u32()
                    ));
                }
            }
        }

        // Check all structures
        let structures = self.structures.read();
        for (name, structure) in structures.iter() {
            if !structure.is_stable {
                report.add_warning(format!(
                    "Structure {} is not stable",
                    name
                ));
            }
        }

        report
    }

    /// Validate structure layout
    /// @param name: Structure name
    /// @param expected_size: Expected size
    /// @param expected_alignment: Expected alignment
    /// @return: Validation result
    pub fn validate_structure_layout(
        &self,
        name: &str,
        expected_size: usize,
        expected_alignment: usize,
    ) -> LayoutValidationResult {
        let structures = self.structures.read();

        if let Some(structure) = structures.get(name) {
            let mut errors = Vec::new();

            if structure.size != expected_size {
                errors.push(format!(
                    "Size mismatch: expected {}, got {}",
                    expected_size, structure.size
                ));
            }

            if structure.alignment != expected_alignment {
                errors.push(format!(
                    "Alignment mismatch: expected {}, got {}",
                    expected_alignment, structure.alignment
                ));
            }

            if errors.is_empty() {
                LayoutValidationResult::Valid
            } else {
                LayoutValidationResult::Invalid { errors }
            }
        } else {
            LayoutValidationResult::NotFound
        }
    }

    /// Check overall compatibility
    pub fn check_compatibility(&self) -> CompatibilityReport {
        let mut report = CompatibilityReport::new();
        let functions = self.functions.read();
        for (name, func) in functions.iter() {
            if !func.is_stable {
                report.add_warning(format!("Function {} is not stable", name));
            }
        }
        report
    }

    /// Validate all structure layouts
    pub fn validate_layouts(&self) -> LayoutValidationResult {
        let structures = self.structures.read();
        let mut all_errors: Vec<String> = Vec::new();

        for (name, structure) in structures.iter() {
            if structure.fields.is_empty() {
                continue;
            }

            let mut errors = Vec::new();

            let mut expected_offset: usize = 0;
            let mut prev_size: usize = 0;

            for (i, field) in structure.fields.iter().enumerate() {
                if i > 0 && field.offset < expected_offset {
                    errors.push(format!(
                        "Field '{}' at offset {} overlaps with previous field (expected offset >= {})",
                        field.name, field.offset, expected_offset
                    ));
                }

                if field.size == 0 {
                    errors.push(format!(
                        "Field '{}' has zero size",
                        field.name
                    ));
                }

                if field.offset % field.size != 0 && field.size <= 8 {
                    errors.push(format!(
                        "Field '{}' at offset {} is not naturally aligned (alignment {} required)",
                        field.name, field.offset, field.size
                    ));
                }

                expected_offset = field.offset + field.size;
                prev_size = field.size;
            }

            if !structure.fields.is_empty() {
                let last_field = structure.fields.last().unwrap();
                let computed_size = last_field.offset + last_field.size;
                let aligned_size = (computed_size + structure.alignment - 1) & !(structure.alignment - 1);

                if structure.size < computed_size {
                    errors.push(format!(
                        "Structure size {} is smaller than computed minimum {} (last field end)",
                        structure.size, computed_size
                    ));
                }

                if structure.size != aligned_size && structure.size >= computed_size {
                    // Size should be a multiple of alignment
                    if structure.size % structure.alignment != 0 {
                        errors.push(format!(
                            "Structure size {} is not a multiple of alignment {}",
                            structure.size, structure.alignment
                        ));
                    }
                }
            }

            if !errors.is_empty() {
                all_errors.push(format!("Structure '{}': {}", name, errors.join("; ")));
            }
        }

        if all_errors.is_empty() {
            LayoutValidationResult::Valid
        } else {
            LayoutValidationResult::Invalid { errors: all_errors }
        }
    }
}

/// Stability result
#[derive(Debug, Clone)]
pub enum StabilityResult {
    /// API is stable
    Stable,

    /// API is unstable
    Unstable { reason: String },

    /// API is deprecated
    Deprecated {
        since: ApiVersion,
        alternative: Option<String>,
    },

    /// API not found
    NotFound,
}

/// Compatibility report
#[derive(Debug, Clone)]
pub struct CompatibilityReport {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl CompatibilityReport {
    fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    fn add_error(&mut self, error: String) {
        self.errors.push(error);
    }

    fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }

    pub fn is_compatible(&self) -> bool {
        self.errors.is_empty()
    }
}

/// Layout validation result
#[derive(Debug, Clone)]
pub enum LayoutValidationResult {
    /// Layout is valid
    Valid,

    /// Layout is invalid
    Invalid { errors: Vec<String> },

    /// Structure not found
    NotFound,
}

/// Define HAL API functions
pub fn define_hal_api() -> ApiStabilityChecker {
    let version = ApiVersion::new(1, 0, 0, 1);
    let checker = ApiStabilityChecker::new(version);

    // Register CPU functions
    checker.register_function(ApiFunction {
        name: String::from("nuva_cpu_get_info"),
        return_type: String::from("nuva_result_t"),
        params: vec![String::from("nuva_cpu_info_t*")],
        version_added: ApiVersion::new(1, 0, 0, 1),
        version_deprecated: None,
        is_stable: true,
    });

    checker.register_function(ApiFunction {
        name: String::from("nuva_cpu_get_core_id"),
        return_type: String::from("uint32_t"),
        params: vec![],
        version_added: ApiVersion::new(1, 0, 0, 1),
        version_deprecated: None,
        is_stable: true,
    });

    // Register GPU functions
    checker.register_function(ApiFunction {
        name: String::from("nuva_gpu_init"),
        return_type: String::from("nuva_result_t"),
        params: vec![],
        version_added: ApiVersion::new(1, 0, 0, 1),
        version_deprecated: None,
        is_stable: true,
    });

    // Register NPU functions
    checker.register_function(ApiFunction {
        name: String::from("nuva_npu_init"),
        return_type: String::from("nuva_result_t"),
        params: vec![],
        version_added: ApiVersion::new(1, 0, 0, 1),
        version_deprecated: None,
        is_stable: true,
    });

    // Register structures
    checker.register_structure(ApiStruct {
        name: String::from("nuva_cpu_info_t"),
        fields: vec![
            ApiField {
                name: String::from("core_count"),
                type_name: String::from("uint32_t"),
                offset: 0,
                size: 4,
            },
            ApiField {
                name: String::from("frequency_mhz"),
                type_name: String::from("uint32_t"),
                offset: 4,
                size: 4,
            },
        ],
        size: 120, // Approximate
        alignment: 8,
        version_added: ApiVersion::new(1, 0, 0, 1),
        is_stable: true,
    });

    checker
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_version() {
        let v1 = ApiVersion::new(1, 0, 0, 1);
        let v2 = ApiVersion::new(1, 1, 0, 1);
        let v3 = ApiVersion::new(2, 0, 0, 1);

        assert!(v1.is_compatible(&v2));
        assert!(!v1.is_compatible(&v3));
    }

    #[test]
    fn test_stability_checker() {
        let checker = define_hal_api();

        let result = checker.check_function_stability("nuva_cpu_get_info");
        assert!(matches!(result, StabilityResult::Stable));

        let result = checker.check_function_stability("nonexistent");
        assert!(matches!(result, StabilityResult::NotFound));
    }
}
