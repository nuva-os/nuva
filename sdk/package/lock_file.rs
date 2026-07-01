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

// ! lockfile

use std::path::Path;
use std::collections::HashMap;
use crate::error::SdkError;
use alloc::vec;
use alloc::format;
use alloc::vec::Vec;

/// lockfile
#[derive(Debug, Clone)]
pub struct LockFile {
    /// lockfileversion
    pub version: u32,
    /// packetlog
    pub packages: HashMap<String, LockedPackage>,
}

impl LockFile {
    /// createnew lockfile
    pub fn new() -> Self {
        Self {
            version: 1,
            packages: HashMap::new(),
        }
    }

    /// secondaryfileload
    pub fn load(path: &Path) -> Result<Self, SdkError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| SdkError::IoError(e.to_string()))?;
        
        let lock: LockFileToml = toml::from_str(&content)
            .map_err(|e| SdkError::ParseError(e.to_string()))?;
        
        let mut packages = HashMap::new();
        for pkg in lock.package {
            packages.insert(pkg.name.clone(), LockedPackage {
                name: pkg.name,
                version: pkg.version,
                source: pkg.source,
                checksum: pkg.checksum,
                dependencies: pkg.dependencies.unwrap_or_default(),
            });
        }
        
        Ok(Self {
            version: lock.version,
            packages,
        })
    }

    /// savetofile
    pub fn save(&self, path: &Path) -> Result<(), SdkError> {
        let packages: Vec<_> = self.packages.values().map(|p| LockedPackageToml {
            name: p.name.clone(),
            version: p.version.clone(),
            source: p.source.clone(),
            checksum: p.checksum.clone(),
            dependencies: Some(p.dependencies.clone()),
        }).collect();
        
        let lock = LockFileToml {
            version: self.version,
            package: packages,
        };
        
        let content = toml::to_string_pretty(&lock)
            .map_err(|e| SdkError::ParseError(e.to_string()))?;
        
        std::fs::write(path, content)
            .map_err(|e| SdkError::IoError(e.to_string()))?;
        
        Ok(())
    }

    /// addpacket
    pub fn add_package(&mut self, pkg: LockedPackage) {
        self.packages.insert(pkg.name.clone(), pkg);
    }

    /// getpacket
    pub fn get_package(&self, name: &str) -> Option<&LockedPackage> {
        self.packages.get(name)
    }

    /// verify checksums
    pub fn verify(&self) -> Result<(), SdkError> {
        for (name, pkg) in &self.packages {
            if pkg.checksum.is_empty() {
                return Err(SdkError::DependencyError(format!(
                    "Missing checksum for package {}",
                    name
                )));
            }
            if pkg.name.is_empty() {
                return Err(SdkError::DependencyError(format!(
                    "Empty package name in lock file entry",
                )));
            }
            if pkg.version.is_empty() {
                return Err(SdkError::DependencyError(format!(
                    "Empty version for package {}",
                    name
                )));
            }
        }
        Ok(())
    }

    /// checkiswhetherneedwantupdate
    pub fn needs_update(&self, current_deps: &HashMap<String, String>) -> bool {
        for (name, version) in current_deps {
            match self.packages.get(name) {
                Some(pkg) => {
                    if &pkg.version != version {
                        return true;
                    }
                }
                None => return true,
            }
        }
        false
    }
}

impl Default for LockFile {
    fn default() -> Self {
        Self::new()
    }
}

/// lockfixed packet
#[derive(Debug, Clone)]
pub struct LockedPackage {
    /// packetname
    pub name: String,
    /// precisecertainversion
    pub version: String,
    /// comesource
    pub source: String,
    /// checksum
    pub checksum: String,
    /// dependency
    pub dependencies: Vec<String>,
}

impl LockedPackage {
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            source: "registry".to_string(),
            checksum: String::new(),
            dependencies: vec![],
        }
    }

    pub fn with_checksum(mut self, checksum: impl Into<String>) -> Self {
        self.checksum = checksum.into();
        self
    }

    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = source.into();
        self
    }

    pub fn with_dependencies(mut self, deps: Vec<String>) -> Self {
        self.dependencies = deps;
        self
    }
}

// TOML struct
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct LockFileToml {
    version: u32,
    package: Vec<LockedPackageToml>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct LockedPackageToml {
    name: String,
    version: String,
    source: String,
    checksum: String,
    dependencies: Option<Vec<String>>,
}