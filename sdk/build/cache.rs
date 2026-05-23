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

// ! buildcache

use std::path::PathBuf;
use std::collections::HashMap;
use crate::error::SdkError;
use super::config::BuildConfig;
use super::target::Target;

/// buildcache
pub struct BuildCache {
    /// cachedirectory
    cache_dir: PathBuf,
    /// fileHashcache
    file_hashes: HashMap<PathBuf, String>,
}

impl BuildCache {
    pub fn new(config: &BuildConfig) -> Self {
        Self {
            cache_dir: config.out_dir.join(".cache"),
            file_hashes: HashMap::new(),
        }
    }

    /// checktargetiswhetherismostnew
    pub fn is_up_to_date(&self, target: &Target) -> bool {
        // Check if output file exists
        let output = self.cache_dir.parent().unwrap().join(&target.name);
        if !output.exists() {
            return false;
        }
        
        // Check if source files are modified
        for source in &target.sources {
            if !source.exists() {
                return false;
            }
            
            // Compute current hash
            let current_hash = match self.compute_hash(source) {
                Ok(hash) => hash,
                Err(_) => return false,
            };
            
            // Compare with cached hash
            if let Some(cached_hash) = self.file_hashes.get(source) {
                if current_hash != *cached_hash {
                    return false;
                }
            } else {
                return false;
            }
        }
        
        true
    }

    /// updatecache
    pub fn update(&mut self, target: &Target) -> Result<(), SdkError> {
        // calculateparallelstorefileHash
        let hash = self.compute_hash(&target.path)?;
        self.file_hashes.insert(target.path.clone(), hash);
        Ok(())
    }

    /// clearadministrationcache
    pub fn clear(&mut self) -> Result<(), SdkError> {
        self.file_hashes.clear();
        
        if self.cache_dir.exists() {
            std::fs::remove_dir_all(&self.cache_dir)
                .map_err(|e| SdkError::IoError(e.to_string()))?;
        }
        
        Ok(())
    }

    /// clearadministrationplacefinitebuildproductobject
    pub fn clear_all(&mut self) -> Result<(), SdkError> {
        self.clear()?;
        
        if let Some(parent) = self.cache_dir.parent() {
            if parent.exists() {
                std::fs::remove_dir_all(parent)
                    .map_err(|e| SdkError::IoError(e.to_string()))?;
            }
        }
        
        Ok(())
    }

    /// calculatefileHash
    fn compute_hash(&self, path: &PathBuf) -> Result<String, SdkError> {
        use std::io::Read;
        
        let mut file = std::fs::File::open(path)
            .map_err(|e| SdkError::IoError(e.to_string()))?;
        
        let mut hasher = blake3::Hasher::new();
        let mut buffer = [0u8; 8192];
        
        loop {
            let bytes_read = file.read(&mut buffer)
                .map_err(|e| SdkError::IoError(e.to_string()))?;
            
            if bytes_read == 0 {
                break;
            }
            
            hasher.update(&buffer[..bytes_read]);
        }
        
        Ok(hasher.finalize().to_hex().to_string())
    }

    /// getcachestatistics
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            entries: self.file_hashes.len(),
        }
    }
}

/// cachestatistics
pub struct CacheStats {
    pub entries: usize,
}