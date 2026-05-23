/*
 * Nuva OS - SystemService - OpenGL - Fence Synchronization
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

//! Frame synchronization fence management for OpenGL contexts.

use alloc::collections::BTreeMap;
use core::sync::atomic::{AtomicU64, AtomicU32, Ordering};

use super::error::GlError;

/// Unique fence identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct FenceId(pub u64);

/// Fence state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FenceState {
    /// Fence is not yet signaled
    Pending = 0,
    /// Fence has been signaled (GPU work complete)
    Signaled = 1,
}

/// Global fence ID counter
static NEXT_FENCE_ID: AtomicU64 = AtomicU64::new(1);

/// A synchronization fence
pub struct Fence {
    /// Unique fence identifier
    pub id: FenceId,
    /// Current fence state
    state: AtomicU32,
}

impl Fence {
    /// Create a new fence in Pending state
    fn new() -> Self {
        let id = FenceId(NEXT_FENCE_ID.fetch_add(1, Ordering::Relaxed));
        Fence {
            id,
            state: AtomicU32::new(FenceState::Pending as u32),
        }
    }

    /// Get current fence state
    pub fn get_state(&self) -> FenceState {
        match self.state.load(Ordering::Acquire) {
            0 => FenceState::Pending,
            1 => FenceState::Signaled,
            _ => FenceState::Pending,
        }
    }

    /// Signal the fence (GPU work complete)
    pub fn signal(&self) {
        self.state.store(FenceState::Signaled as u32, Ordering::Release);
    }

    /// Check if the fence is signaled
    pub fn is_signaled(&self) -> bool {
        self.get_state() == FenceState::Signaled
    }
}

/// Per-context fence manager
pub struct ContextFenceManager {
    /// Active fences indexed by ID
    fences: BTreeMap<u64, Fence>,
    /// Outstanding (pending) fence count
    pending_count: AtomicU32,
}

impl ContextFenceManager {
    /// Create a new fence manager
    pub fn new() -> Self {
        ContextFenceManager {
            fences: BTreeMap::new(),
            pending_count: AtomicU32::new(0),
        }
    }

    /// Create a new synchronization fence
    pub fn create_fence(&mut self) -> FenceId {
        let fence = Fence::new();
        let id = fence.id;
        self.fences.insert(id.0, fence);
        self.pending_count.fetch_add(1, Ordering::Relaxed);
        log_debug!("Created GL fence {}", id.0);
        id
    }

    /// Signal a fence (mark GPU work as complete)
    pub fn signal_fence(&self, id: FenceId) -> Result<(), GlError> {
        if let Some(fence) = self.fences.get(&id.0) {
            if !fence.is_signaled() {
                fence.signal();
                self.pending_count.fetch_sub(1, Ordering::Relaxed);
                log_debug!("Signaled GL fence {}", id.0);
            }
            Ok(())
        } else {
            Err(GlError::InvalidResource)
        }
    }

    /// Wait for a fence to be signaled.
    /// Returns Ok(()) if signaled, Err(Timeout) if not signaled after check.
    /// In a full implementation, this would block with a timeout.
    pub fn wait_fence(&self, id: FenceId) -> Result<(), GlError> {
        if let Some(fence) = self.fences.get(&id.0) {
            if fence.is_signaled() {
                Ok(())
            } else {
                Err(GlError::GpuError)
            }
        } else {
            Err(GlError::InvalidResource)
        }
    }

    /// Get the state of a fence
    pub fn get_fence_state(&self, id: FenceId) -> Result<FenceState, GlError> {
        if let Some(fence) = self.fences.get(&id.0) {
            Ok(fence.get_state())
        } else {
            Err(GlError::InvalidResource)
        }
    }

    /// Destroy a signaled fence
    pub fn destroy_fence(&mut self, id: FenceId) -> Result<(), GlError> {
        if let Some(fence) = self.fences.get(&id.0) {
            if !fence.is_signaled() {
                return Err(GlError::GpuError);
            }
        }
        if self.fences.remove(&id.0).is_some() {
            log_debug!("Destroyed GL fence {}", id.0);
            Ok(())
        } else {
            Err(GlError::InvalidResource)
        }
    }

    /// Get the number of pending (unsigaled) fences
    pub fn pending_count(&self) -> u32 {
        self.pending_count.load(Ordering::Acquire)
    }

    /// Signal all pending fences (e.g. on context destruction)
    pub fn signal_all(&self) {
        for (_, fence) in self.fences.iter() {
            if !fence.is_signaled() {
                fence.signal();
            }
        }
        self.pending_count.store(0, Ordering::Release);
    }
}
