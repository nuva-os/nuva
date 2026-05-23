/*
 * Nuva OS - SystemService - OpenGL - Service Node
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

//! OpenGL rendering service node implementing CoreProcessingService trait.
//! Registered as "nuva.service.opengl" in the Nuva IPC framework.

use alloc::collections::BTreeMap;
use core::sync::atomic::{AtomicU64, Ordering};

use crate::services::core_processing::service_node::{
    CallerIdentity, CoreProcessingService, ServiceConfig, ServiceHealth, ServiceNodeId,
    ServiceStats, ServiceVersion,
};
use crate::services::core_processing::error::ServiceError;

use super::command::GlCommandBatch;
use super::context::{AccelPath, ContextId, ContextState, RenderContext};
use super::error::GlError;
use super::fence::{ContextFenceManager, FenceId};
use super::pipeline::PipelineStateCache;
use super::power::GpuPowerManager;
use super::resource::{ResourceId, ResourceType, ResourceRegistry};
use super::software::SoftwareRenderer;

/// Convert GlError to ServiceError
impl From<GlError> for ServiceError {
    fn from(e: GlError) -> ServiceError {
        match e {
            GlError::NotInitialized => ServiceError::NotInitialized,
            GlError::OutOfMemory => ServiceError::OutOfMemory,
            GlError::InvalidContext => ServiceError::Specific(
                crate::services::core_processing::error::ServiceSpecificError::GlInvalidContext,
            ),
            GlError::InvalidResource => ServiceError::Specific(
                crate::services::core_processing::error::ServiceSpecificError::GlInvalidResource,
            ),
            GlError::InvalidCommand => ServiceError::Specific(
                crate::services::core_processing::error::ServiceSpecificError::GlInvalidCommand,
            ),
            GlError::GpuError => ServiceError::Specific(
                crate::services::core_processing::error::ServiceSpecificError::GlGpuError,
            ),
            GlError::FallbackActive => ServiceError::Specific(
                crate::services::core_processing::error::ServiceSpecificError::GlFallbackActive,
            ),
        }
    }
}

/// OpenGL service statistics
#[derive(Debug)]
pub struct OpenGLStats {
    /// Total contexts created
    pub total_contexts: AtomicU64,
    /// Total commands submitted
    pub total_commands: AtomicU64,
    /// Total fences created
    pub total_fences: AtomicU64,
    /// Total resources created
    pub total_resources: AtomicU64,
    /// Software fallback invocation count
    pub fallback_count: AtomicU64,
}

impl OpenGLStats {
    /// Create zero-initialized stats
    pub const fn new() -> Self {
        OpenGLStats {
            total_contexts: AtomicU64::new(0),
            total_commands: AtomicU64::new(0),
            total_fences: AtomicU64::new(0),
            total_resources: AtomicU64::new(0),
            fallback_count: AtomicU64::new(0),
        }
    }
}

/// OpenGL rendering service
pub struct OpenGLService {
    /// Service configuration
    config: ServiceConfig,
    /// Core service statistics
    stats: ServiceStats,
    /// OpenGL-specific statistics
    gl_stats: OpenGLStats,
    /// Active rendering contexts indexed by context ID
    contexts: BTreeMap<u64, RenderContext>,
    /// GPU resource registry
    resource_registry: ResourceRegistry,
    /// Pipeline state cache
    pipeline_cache: PipelineStateCache,
    /// Software renderer fallback
    software_renderer: SoftwareRenderer,
    /// GPU power manager
    power_mgr: GpuPowerManager,
    /// Whether the service is initialized
    initialized: bool,
}

/// Default GPU memory budget: 256 MB
const DEFAULT_GPU_MEMORY_BUDGET: u64 = 256 * 1024 * 1024;

/// Default software renderer framebuffer size
const DEFAULT_SW_FB_WIDTH: u32 = 1920;
const DEFAULT_SW_FB_HEIGHT: u32 = 1080;

/// Idle timeout for GPU suspend: 5 seconds in microseconds
const DEFAULT_IDLE_TIMEOUT_US: u64 = 5_000_000;

impl OpenGLService {
    /// Create a new OpenGL service instance
    pub fn new() -> Self {
        let config = ServiceConfig {
            name: "nuva.service.opengl",
            version: ServiceVersion::new(1, 0, 0),
            max_concurrent_requests: 64,
            request_timeout_us: 16_666,
            hw_accel_available: true,
        };

        OpenGLService {
            config,
            stats: ServiceStats::new(),
            gl_stats: OpenGLStats::new(),
            contexts: BTreeMap::new(),
            resource_registry: ResourceRegistry::new(DEFAULT_GPU_MEMORY_BUDGET),
            pipeline_cache: PipelineStateCache::new(),
            software_renderer: SoftwareRenderer::new(DEFAULT_SW_FB_WIDTH, DEFAULT_SW_FB_HEIGHT),
            power_mgr: GpuPowerManager::new(DEFAULT_IDLE_TIMEOUT_US),
            initialized: false,
        }
    }

    /// Create a rendering context for the given caller
    pub fn create_context(&mut self, caller: CallerIdentity) -> Result<ContextId, GlError> {
        if !self.initialized {
            return Err(GlError::NotInitialized);
        }

        let ctx = RenderContext::create(caller.pid, self.config.hw_accel_available);
        let ctx_id = ctx.id;
        self.contexts.insert(ctx_id.0, ctx);
        self.gl_stats.total_contexts.fetch_add(1, Ordering::Relaxed);
        self.power_mgr.context_created();
        Ok(ctx_id)
    }

    /// Destroy a rendering context
    pub fn destroy_context(&mut self, caller: CallerIdentity, ctx_id: ContextId) -> Result<(), GlError> {
        if !self.initialized {
            return Err(GlError::NotInitialized);
        }

        if let Some(ctx) = self.contexts.get_mut(&ctx_id.0) {
            if ctx.owner_pid != caller.pid {
                return Err(GlError::InvalidContext);
            }
            ctx.begin_destroy()?;
            ctx.fence_mgr.signal_all();
        } else {
            return Err(GlError::InvalidContext);
        }

        if let Some(ctx) = self.contexts.get_mut(&ctx_id.0) {
            let owner = ctx.owner_pid;
            self.resource_registry.destroy_context_resources(ctx_id.0);
            self.pipeline_cache.remove(ctx_id.0);
            ctx.finish_destroy();
            self.contexts.remove(&ctx_id.0);
            // SAFETY: owner is read before removal
            let _ = owner;
        }
        self.power_mgr.context_destroyed();
        Ok(())
    }

    /// Submit a batch of rendering commands to a context
    pub fn submit_commands(
        &mut self,
        caller: CallerIdentity,
        ctx_id: ContextId,
        batch: &GlCommandBatch,
    ) -> Result<FenceId, GlError> {
        if !self.initialized {
            return Err(GlError::NotInitialized);
        }

        let ctx = self.contexts.get(&ctx_id.0).ok_or(GlError::InvalidContext)?;
        if ctx.owner_pid != caller.pid || !ctx.is_active() {
            return Err(GlError::InvalidContext);
        }

        let cmd_count = batch.len();
        let accel_path = ctx.accel_path;

        // Execute via software fallback if needed
        if accel_path == AccelPath::Software {
            for cmd in batch.iter() {
                self.software_renderer.execute(cmd)?;
            }
            self.gl_stats.fallback_count.fetch_add(1, Ordering::Relaxed);
        }

        // Create fence for this submission
        let fence_id = if let Some(ctx) = self.contexts.get_mut(&ctx_id.0) {
            let fid = ctx.fence_mgr.create_fence();
            fid
        } else {
            return Err(GlError::InvalidContext);
        };

        self.gl_stats.total_commands.fetch_add(cmd_count as u64, Ordering::Relaxed);
        self.gl_stats.total_fences.fetch_add(1, Ordering::Relaxed);
        self.power_mgr.submit_commands(0);

        log_debug!(
            "Submitted {} GL commands to context {} ({})",
            cmd_count,
            ctx_id.0,
            match accel_path {
                AccelPath::Hardware => "HW",
                AccelPath::Software => "SW",
            }
        );

        Ok(fence_id)
    }

    /// Wait for a fence to be signaled
    pub fn wait_fence(&self, ctx_id: ContextId, fence_id: FenceId) -> Result<(), GlError> {
        let ctx = self.contexts.get(&ctx_id.0).ok_or(GlError::InvalidContext)?;
        ctx.fence_mgr.wait_fence(fence_id)
    }

    /// Create a GPU resource
    pub fn create_resource(
        &mut self,
        caller: CallerIdentity,
        ctx_id: ContextId,
        resource_type: ResourceType,
        size_bytes: u64,
    ) -> Result<ResourceId, GlError> {
        if !self.initialized {
            return Err(GlError::NotInitialized);
        }

        let ctx = self.contexts.get(&ctx_id.0).ok_or(GlError::InvalidContext)?;
        if ctx.owner_pid != caller.pid || !ctx.is_active() {
            return Err(GlError::InvalidContext);
        }

        let res_id = self.resource_registry.register(resource_type, size_bytes, ctx_id.0)?;

        if let Some(ctx) = self.contexts.get_mut(&ctx_id.0) {
            ctx.add_resource(res_id.0);
        }

        self.gl_stats.total_resources.fetch_add(1, Ordering::Relaxed);
        Ok(res_id)
    }

    /// Destroy a GPU resource
    pub fn destroy_resource(
        &mut self,
        caller: CallerIdentity,
        ctx_id: ContextId,
        res_id: ResourceId,
    ) -> Result<(), GlError> {
        if !self.initialized {
            return Err(GlError::NotInitialized);
        }

        let ctx = self.contexts.get(&ctx_id.0).ok_or(GlError::InvalidContext)?;
        if ctx.owner_pid != caller.pid {
            return Err(GlError::InvalidContext);
        }

        self.resource_registry.unregister(res_id)?;

        if let Some(ctx) = self.contexts.get_mut(&ctx_id.0) {
            ctx.remove_resource(res_id.0);
        }

        Ok(())
    }

    /// Get the acceleration path for a context
    pub fn get_accel_path(&self, ctx_id: ContextId) -> Result<AccelPath, GlError> {
        let ctx = self.contexts.get(&ctx_id.0).ok_or(GlError::InvalidContext)?;
        Ok(ctx.accel_path)
    }

    /// Get OpenGL-specific statistics
    pub fn get_stats(&self) -> &OpenGLStats {
        &self.gl_stats
    }
}

impl CoreProcessingService for OpenGLService {
    fn config(&self) -> &ServiceConfig {
        &self.config
    }

    fn init(&mut self) -> Result<ServiceNodeId, ServiceError> {
        if self.initialized {
            return Err(ServiceError::Busy);
        }

        log_info!("Initializing OpenGL service (nuva.service.opengl)");

        // In a full implementation, this would:
        // 1. Query HAL GPU interface for capabilities
        // 2. Initialize GPU driver via HAL
        // 3. Register service with Nuva IPC

        self.initialized = true;

        // Use address as service node ID (stable within process lifetime)
        // SAFETY: converting reference to u64 for use as a unique ID within
        // this process. The reference remains valid as long as the service exists.
        let node_id = self as *const Self as u64;

        log_info!("OpenGL service initialized, node_id={}", node_id);
        Ok(node_id)
    }

    fn handle_request(
        &mut self,
        caller: CallerIdentity,
        request_id: u64,
        payload: &[u8],
    ) -> Result<(), ServiceError> {
        if !self.initialized {
            return Err(ServiceError::NotInitialized);
        }

        self.stats.record_request(0);
        log_debug!(
            "OpenGL service request: caller=({},{}) req_id={} len={}",
            caller.pid,
            caller.uid,
            request_id,
            payload.len()
        );

        // In a full implementation, payload is deserialized into
        // an OpenGL IPC request and dispatched to the appropriate method.
        self.stats.complete_request();
        Ok(())
    }

    fn shutdown(&mut self) -> Result<(), ServiceError> {
        if !self.initialized {
            return Err(ServiceError::NotInitialized);
        }

        log_info!("Shutting down OpenGL service");

        // Signal all fences in all contexts
        for (_, ctx) in self.contexts.iter() {
            ctx.fence_mgr.signal_all();
        }

        // Destroy all contexts
        let ctx_ids: alloc::vec::Vec<u64> = self.contexts.keys().copied().collect();
        for id in ctx_ids {
            self.resource_registry.destroy_context_resources(id);
            self.pipeline_cache.remove(id);
        }
        self.contexts.clear();

        self.initialized = false;
        Ok(())
    }

    fn health_check(&self) -> ServiceHealth {
        if !self.initialized {
            return ServiceHealth::NotInitialized;
        }
        // Check if any context is in software fallback
        let has_fallback = self.contexts.values().any(|c| c.accel_path == AccelPath::Software);
        if has_fallback {
            ServiceHealth::Degraded
        } else {
            ServiceHealth::Healthy
        }
    }

    fn stats(&self) -> &ServiceStats {
        &self.stats
    }
}
