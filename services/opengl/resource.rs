/*
 * Nuva OS - SystemService - OpenGL - Resource Registry
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

//! GPU resource registry with LRU eviction strategy.

use alloc::collections::BTreeMap;
use core::sync::atomic::{AtomicU64, Ordering};

use super::error::GlError;
use alloc::vec::Vec;

/// GPU resource type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceType {
    /// Vertex buffer object
    Buffer = 0,
    /// Texture object
    Texture = 1,
    /// Shader program
    Program = 2,
    /// Shader module (vertex/fragment)
    Shader = 3,
    /// Framebuffer object
    Framebuffer = 4,
    /// Renderbuffer object
    Renderbuffer = 5,
    /// Vertex array object
    VertexArray = 6,
    /// Sampler object
    Sampler = 7,
}

/// Unique resource identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ResourceId(pub u64);

/// Resource entry in the registry
#[derive(Debug, Clone)]
pub struct ResourceEntry {
    /// Resource identifier
    pub id: ResourceId,
    /// Resource type
    pub resource_type: ResourceType,
    /// Size in bytes
    pub size_bytes: u64,
    /// Owning context ID
    pub owner_context: u64,
    /// LRU timestamp (monotonically increasing)
    pub last_used_tick: u64,
}

/// Global resource ID counter
static NEXT_RESOURCE_ID: AtomicU64 = AtomicU64::new(1);

/// GPU resource registry with LRU eviction
pub struct ResourceRegistry {
    /// Registered resources indexed by ID
    entries: BTreeMap<u64, ResourceEntry>,
    /// Total GPU memory used in bytes
    total_memory_bytes: u64,
    /// Maximum GPU memory budget in bytes
    max_memory_bytes: u64,
}

impl ResourceRegistry {
    /// Create a new resource registry with the given memory budget
    pub fn new(max_memory_bytes: u64) -> Self {
        ResourceRegistry {
            entries: BTreeMap::new(),
            total_memory_bytes: 0,
            max_memory_bytes,
        }
    }

    /// Register a new GPU resource
    pub fn register(
        &mut self,
        resource_type: ResourceType,
        size_bytes: u64,
        owner_context: u64,
    ) -> Result<ResourceId, GlError> {
        // Check if we need to evict to make room
        if self.total_memory_bytes + size_bytes > self.max_memory_bytes {
            self.evict_lru(size_bytes)?;
        }

        let id = ResourceId(NEXT_RESOURCE_ID.fetch_add(1, Ordering::Relaxed));
        let entry = ResourceEntry {
            id,
            resource_type,
            size_bytes,
            owner_context,
            last_used_tick: 0,
        };
        self.entries.insert(id.0, entry);
        self.total_memory_bytes += size_bytes;

        log_debug!(
            "Registered GL resource {} type={:?} size={} ctx={}",
            id.0,
            resource_type,
            size_bytes,
            owner_context
        );
        Ok(id)
    }

    /// Unregister (destroy) a GPU resource
    pub fn unregister(&mut self, id: ResourceId) -> Result<(), GlError> {
        if let Some(entry) = self.entries.remove(&id.0) {
            self.total_memory_bytes = self.total_memory_bytes.saturating_sub(entry.size_bytes);
            log_debug!("Unregistered GL resource {}", id.0);
            Ok(())
        } else {
            Err(GlError::InvalidResource)
        }
    }

    /// Touch a resource to update its LRU timestamp
    pub fn touch(&mut self, id: ResourceId, current_tick: u64) -> Result<(), GlError> {
        if let Some(entry) = self.entries.get_mut(&id.0) {
            entry.last_used_tick = current_tick;
            Ok(())
        } else {
            Err(GlError::InvalidResource)
        }
    }

    /// Get a resource entry by ID
    pub fn get(&self, id: ResourceId) -> Option<&ResourceEntry> {
        self.entries.get(&id.0)
    }

    /// Evict least-recently-used resources until at least `needed_bytes` are free
    pub fn evict_lru(&mut self, needed_bytes: u64) -> Result<(), GlError> {
        let free_bytes = self.max_memory_bytes.saturating_sub(self.total_memory_bytes);
        if free_bytes >= needed_bytes {
            return Ok(());
        }

        let mut remaining_to_free = needed_bytes.saturating_sub(free_bytes);

        // Collect entries sorted by last_used_tick ascending (LRU first)
        let mut candidates: alloc::vec::Vec<(u64, u64)> = self
            .entries
            .iter()
            .map(|(id, e)| (*id, e.last_used_tick))
            .collect();

        candidates.sort_by_key(|&(_, tick)| tick);

        for (id, _tick) in candidates {
            if remaining_to_free == 0 {
                break;
            }
            if let Some(entry) = self.entries.remove(&id) {
                remaining_to_free = remaining_to_free.saturating_sub(entry.size_bytes);
                self.total_memory_bytes = self.total_memory_bytes.saturating_sub(entry.size_bytes);
                log_debug!("LRU evicted GL resource {} ({} bytes)", id, entry.size_bytes);
            }
        }

        if remaining_to_free > 0 {
            Err(GlError::OutOfMemory)
        } else {
            Ok(())
        }
    }

    /// Destroy all resources owned by a given context
    pub fn destroy_context_resources(&mut self, context_id: u64) {
        let to_remove: alloc::vec::Vec<u64> = self
            .entries
            .iter()
            .filter(|(_, e)| e.owner_context == context_id)
            .map(|(id, _)| *id)
            .collect();

        for id in to_remove {
            if let Some(entry) = self.entries.remove(&id) {
                self.total_memory_bytes = self.total_memory_bytes.saturating_sub(entry.size_bytes);
            }
        }
    }

    /// Get total GPU memory usage in bytes
    pub fn memory_usage(&self) -> u64 {
        self.total_memory_bytes
    }

    /// Get the number of registered resources
    pub fn resource_count(&self) -> usize {
        self.entries.len()
    }
}
