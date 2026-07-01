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

// ! packetcache

use std::path::PathBuf;
use super::meta::Package;
use crate::error::SdkError;
use alloc::vec;
use alloc::format;
use alloc::vec::Vec;

/// packetcache
pub struct PackageCache {
    /// cachedirectory
    cache_dir: PathBuf,
    /// memorycache
    memory_cache: std::collections::HashMap<String, Package>,
}

impl PackageCache {
    pub fn new(cache_dir: impl Into<PathBuf>) -> Self {
        Self {
            cache_dir: cache_dir.into(),
            memory_cache: std::collections::HashMap::new(),
        }
    }

    /// getcache packet
    pub fn get(&self, name: &str, version: &str) -> Option<Package> {
        let key = format!("{}@{}", name, version);
        
        // firstcheckmemorycache
        if let Some(pkg) = self.memory_cache.get(&key) {
            return Some(pkg.clone());
        }
        
        // checkmagneticdiskcache
        let cache_path = self.cache_path(name, version);
        if cache_path.exists() {
            Package::from_file(&cache_path.join("Nuva.toml")).ok()
        } else {
            None
        }
    }

    /// storepackettocache
    pub fn store(&mut self, pkg: &Package) -> Result<(), SdkError> {
        let key = format!("{}@{}", pkg.name, pkg.version);
        
        // storetomemory
        self.memory_cache.insert(key.clone(), pkg.clone());
        
        // Store to disk
        let cache_path = self.cache_path(&pkg.name, &pkg.version.to_string());
        std::fs::create_dir_all(&cache_path)
            .map_err(|e| SdkError::IoError(e.to_string()))?;
        
        // Store package file
        let package_file = cache_path.join("package.tar.gz");
        if let Some(data) = &pkg.data {
            std::fs::write(&package_file, data)
                .map_err(|e| SdkError::IoError(e.to_string()))?;
            
            log_debug!("Stored package file: {}", package_file.display());
        }
        
        // Store package metadata
        let metadata_file = cache_path.join("metadata.json");
        let metadata = serde_json::to_string_pretty(&pkg)
            .map_err(|e| SdkError::SerializationError(e.to_string()))?;
        std::fs::write(&metadata_file, metadata)
            .map_err(|e| SdkError::IoError(e.to_string()))?;
        
        log_debug!("Stored package metadata: {}", metadata_file.display());
        
        Ok(())
    }

    /// checkpacketiswhether alreadycache
    pub fn has(&self, name: &str, version: &str) -> bool {
        let key = format!("{}@{}", name, version);
        self.memory_cache.contains_key(&key) || self.cache_path(name, version).exists()
    }

    /// clearadministrationcache
    pub fn clear(&mut self) -> Result<(), SdkError> {
        self.memory_cache.clear();
        
        if self.cache_dir.exists() {
            std::fs::remove_dir_all(&self.cache_dir)
                .map_err(|e| SdkError::IoError(e.to_string()))?;
        }
        
        Ok(())
    }

    /// getcachesize
    pub fn size(&self) -> Result<u64, SdkError> {
        let mut size = 0;
        
        if self.cache_dir.exists() {
            for entry in walkdir::WalkDir::new(&self.cache_dir) {
                let entry = entry.map_err(|e| SdkError::IoError(e.to_string()))?;
                if entry.file_type().is_file() {
                    size += entry.metadata()
                        .map_err(|e| SdkError::IoError(e.to_string()))?
                        .len();
                }
            }
        }
        
        Ok(size)
    }

    /// columnexitcache packet
    pub fn list(&self) -> Result<Vec<(String, String)>, SdkError> {
        let mut packages = vec![];
        
        if self.cache_dir.exists() {
            for entry in std::fs::read_dir(&self.cache_dir)
                .map_err(|e| SdkError::IoError(e.to_string()))?
            {
                let entry = entry.map_err(|e| SdkError::IoError(e.to_string()))?;
                let name = entry.file_name().to_string_lossy().to_string();
                
                let pkg_dir = entry.path();
                for version_entry in std::fs::read_dir(&pkg_dir)
                    .map_err(|e| SdkError::IoError(e.to_string()))?
                {
                    let version_entry = version_entry
                        .map_err(|e| SdkError::IoError(e.to_string()))?;
                    let version = version_entry.file_name().to_string_lossy().to_string();
                    
                    packages.push((name.clone(), version));
                }
            }
        }
        
        Ok(packages)
    }

    fn cache_path(&self, name: &str, version: &str) -> PathBuf {
        self.cache_dir.join(name).join(version)
    }
}