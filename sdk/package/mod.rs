/*
 * Nuva OS
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

// ! Package manager module

pub mod meta;
pub mod dependency;
pub mod resolver;
pub mod registry;
pub mod cache;
pub mod lock_file;
pub mod validator;

use crate::error::SdkError;
use alloc::format;
use alloc::vec::Vec;

/// Package manager
pub struct PackageManager {
    /// Local cache
    cache: cache::PackageCache,
    /// Registry
    registry: registry::CentralRegistry,
    /// Validator
    validator: validator::PackageValidator,
}

impl PackageManager {
    /// Create new package manager
    pub fn new(cache_dir: impl Into<std::path::PathBuf>) -> Self {
        Self {
            cache: cache::PackageCache::new(cache_dir),
            registry: registry::CentralRegistry::default(),
            validator: validator::PackageValidator::new(),
        }
    }

    /// Install package
    pub fn install(&mut self, name: &str, version: &str) -> Result<meta::Package, SdkError> {
        if let Some(pkg) = self.cache.get(name, version) {
            return Ok(pkg);
        }

        let pkg = self.registry.fetch(name, version)?;

        let validation = self.validator.validate(&pkg)?;
        if !validation.is_valid() {
            return Err(SdkError::ValidationError(format!(
                "Package validation failed: {}",
                validation.errors().join(", ")
            )));
        }

        self.cache.store(&pkg)?;

        Ok(pkg)
    }

    /// Resolve dependencies
    pub fn resolve(&self, pkg: &meta::Package) -> Result<resolver::ResolvedDeps, SdkError> {
        let resolver = resolver::DependencyResolver::new();
        resolver.resolve(pkg)
    }

    /// Search packages
    pub fn search(&self, query: &str) -> Result<Vec<meta::PackageSummary>, SdkError> {
        self.registry.search(query)
    }

    /// Validate a package
    pub fn validate(&self, pkg: &meta::Package) -> Result<validator::ValidationResult, SdkError> {
        self.validator.validate(pkg)
    }

    /// Verify package integrity
    pub fn verify_integrity(&self, pkg: &meta::Package, expected_checksum: &str) -> Result<bool, SdkError> {
        self.validator.verify_checksum(pkg, expected_checksum)
    }
}