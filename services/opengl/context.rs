/*
 * Nuva OS - SystemService - OpenGL - Render Context
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

//! OpenGL rendering context management.
//! Each CallerIdentity owns an independent RenderContext.

use alloc::collections::BTreeSet;
use core::sync::atomic::{AtomicU64, Ordering};

use super::command::GlCommandBatch;
use super::error::GlError;
use super::fence::ContextFenceManager;
use super::resource::ResourceRegistry;

/// Unique context identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ContextId(pub u64);

/// Rendering context lifecycle state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextState {
    /// Context created but not yet ready
    Created = 0,
    /// Context is active and ready for commands
    Active = 1,
    /// Context is being destroyed
    Destroying = 2,
    /// Context has been destroyed
    Destroyed = 3,
}

/// Hardware acceleration path for a context
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccelPath {
    /// GPU hardware rendering
    Hardware = 0,
    /// CPU software rendering fallback
    Software = 1,
}

/// Global context ID counter
static NEXT_CONTEXT_ID: AtomicU64 = AtomicU64::new(1);

/// OpenGL rendering context
pub struct RenderContext {
    /// Unique context identifier
    pub id: ContextId,
    /// Owner process PID
    pub owner_pid: u32,
    /// Current lifecycle state
    pub state: ContextState,
    /// Command buffers associated with this context
    pub cmd_buffers: GlCommandBatch,
    /// Resource IDs owned by this context
    pub resources: BTreeSet<u64>,
    /// Fence manager for this context
    pub fence_mgr: ContextFenceManager,
    /// Active acceleration path
    pub accel_path: AccelPath,
}

impl RenderContext {
    /// Create a new rendering context for the given owner PID
    pub fn create(owner_pid: u32, hw_available: bool) -> Self {
        let id = ContextId(NEXT_CONTEXT_ID.fetch_add(1, Ordering::Relaxed));
        let accel_path = if hw_available {
            AccelPath::Hardware
        } else {
            AccelPath::Software
        };
        log_info!(
            "Created GL context {} for PID {} ({})",
            id.0,
            owner_pid,
            match accel_path {
                AccelPath::Hardware => "HW",
                AccelPath::Software => "SW",
            }
        );
        RenderContext {
            id,
            owner_pid,
            state: ContextState::Created,
            cmd_buffers: GlCommandBatch::new(),
            resources: BTreeSet::new(),
            fence_mgr: ContextFenceManager::new(),
            accel_path,
        }
    }

    /// Activate the context for rendering
    pub fn activate(&mut self) -> Result<(), GlError> {
        match self.state {
            ContextState::Created | ContextState::Active => {
                self.state = ContextState::Active;
                Ok(())
            }
            ContextState::Destroying => Err(GlError::InvalidContext),
            ContextState::Destroyed => Err(GlError::InvalidContext),
        }
    }

    /// Begin destroying the context (marks it as Destroying)
    pub fn begin_destroy(&mut self) -> Result<(), GlError> {
        match self.state {
            ContextState::Created | ContextState::Active => {
                self.state = ContextState::Destroying;
                log_info!("Destroying GL context {}", self.id.0);
                Ok(())
            }
            ContextState::Destroying | ContextState::Destroyed => {
                Err(GlError::InvalidContext)
            }
        }
    }

    /// Finalize context destruction
    pub fn finish_destroy(&mut self) {
        self.state = ContextState::Destroyed;
        self.resources.clear();
    }

    /// Add a resource ID to this context's ownership set
    pub fn add_resource(&mut self, resource_id: u64) {
        self.resources.insert(resource_id);
    }

    /// Remove a resource ID from this context's ownership set
    pub fn remove_resource(&mut self, resource_id: u64) {
        self.resources.remove(&resource_id);
    }

    /// Switch acceleration path (e.g. GPU failure triggers software fallback)
    pub fn switch_accel_path(&mut self, new_path: AccelPath) {
        if self.accel_path != new_path {
            log_warn!(
                "GL context {} switching accel path: {:?} -> {:?}",
                self.id.0,
                self.accel_path,
                new_path
            );
            self.accel_path = new_path;
        }
    }

    /// Check if this context is usable for rendering
    pub fn is_active(&self) -> bool {
        self.state == ContextState::Active
    }
}
