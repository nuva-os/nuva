/*
 * Nuva OS - SystemService - OpenGL
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

//! OpenGL rendering service for Nuva OS.
//! Provides GPU-accelerated graphics rendering with software fallback,
//! per-caller rendering contexts, GPU resource management with LRU eviction,
//! fence-based frame synchronization, pipeline state caching,
//! and power coordination with the system power service.

pub use crate::{pr_emerg, pr_alert, pr_crit, pr_err, pr_warn, pr_notice, pr_info, pr_debug};

pub mod error;
pub mod service_node;
pub mod context;
pub mod command;
pub mod pipeline;
pub mod resource;
pub mod fence;
pub mod software;
pub mod power;

pub use service_node::OpenGLService;
pub use error::GlError;
pub use context::{ContextId, RenderContext, AccelPath, ContextState};
pub use command::{GlCommand, GlCommandBatch};
pub use resource::{ResourceId, ResourceType, ResourceRegistry};
pub use fence::{FenceId, FenceState, ContextFenceManager};
pub use pipeline::PipelineState;
pub use power::GpuPowerManager;

/// Initialize the OpenGL rendering service
pub fn init_opengl_service() {
    log_info!("OpenGL service module loaded");
    // The OpenGLService is instantiated and initialized by
    // the system services manager via CoreProcessingService::init()
}
