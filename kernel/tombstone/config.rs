/*
 * Nuva OS - Kernel - Tombstone - Configuration
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

//! Tombstone configuration management.
/*!*/
//! Defines the TombstoneStoreConfig structure with storage path,
//! capacity limits, and auto-prune settings.

use super::record::{TombstoneError, TOMBSTONE_MAX_COUNT, TOMBSTONE_MAX_FILE_SIZE};

/** Maximum length of a store directory path */
pub const STORE_DIR_MAX_LEN: usize = 256;

/** Default memory cache capacity when FS is unavailable */
pub const DEFAULT_MEMORY_CACHE_SIZE: u32 = 4;

/** Tombstone storage configuration */
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct TombstoneStoreConfig {
    /** Store directory path (null-terminated) */
    pub store_dir: [u8; STORE_DIR_MAX_LEN],
    /** Maximum number of tombstone files to retain */
    pub max_count: u32,
    /** Maximum size of a single tombstone file in bytes */
    pub max_file_size: u32,
    /** Number of in-memory cache slots when FS is unavailable */
    pub memory_cache_size: u32,
    /** Whether auto-pruning is enabled when capacity is reached */
    pub auto_prune_enabled: bool,
}

impl TombstoneStoreConfig {
    /** Create a default configuration:
     *  - store_dir = "/data/tombstones/"
     *  - max_count = 100
     *  - max_file_size = 8192
     *  - memory_cache_size = 4
     *  - auto_prune_enabled = true */
    pub fn default() -> Self {
        let mut store_dir = [0u8; STORE_DIR_MAX_LEN];
        let default_path = b"/data/tombstones/";
        let len = default_path.len().min(STORE_DIR_MAX_LEN);
        store_dir[..len].copy_from_slice(&default_path[..len]);

        TombstoneStoreConfig {
            store_dir,
            max_count: TOMBSTONE_MAX_COUNT,
            max_file_size: TOMBSTONE_MAX_FILE_SIZE,
            memory_cache_size: DEFAULT_MEMORY_CACHE_SIZE,
            auto_prune_enabled: true,
        }
    }

    /** Validate configuration parameters.
     *  Returns Err(InvalidParam) if values are out of range. */
    pub fn validate(&self) -> Result<(), TombstoneError> {
        if self.max_count == 0 || self.max_count > 1000 {
            return Err(TombstoneError::InvalidParam);
        }
        if self.memory_cache_size < 2 {
            return Err(TombstoneError::InvalidParam);
        }
        if self.max_file_size == 0 {
            return Err(TombstoneError::InvalidParam);
        }
        if self.store_dir[0] == 0 {
            return Err(TombstoneError::InvalidParam);
        }
        Ok(())
    }

    /** Return the store directory as a byte slice (up to first null) */
    pub fn store_dir_str(&self) -> &[u8] {
        let end = self
            .store_dir
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(STORE_DIR_MAX_LEN);
        &self.store_dir[..end]
    }
}
