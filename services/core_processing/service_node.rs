/*
 * Nuva OS - SystemService - CoreProcessing - Service Node
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

//! Unified service node registration framework for Nuva IPC.
//! All core processing services implement CoreProcessingService trait
//! and register as Nuva IPC service nodes.

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

use super::error::ServiceError;

/// Service node identifier
pub type ServiceNodeId = u64;

/// Service version descriptor
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServiceVersion {
    /// Major version
    pub major: u32,
    /// Minor version
    pub minor: u32,
    /// Patch version
    pub patch: u32,
}

impl ServiceVersion {
    /// Create a new service version
    pub const fn new(major: u32, minor: u32, patch: u32) -> Self {
        ServiceVersion { major, minor, patch }
    }
}

/// Service configuration
#[derive(Debug, Clone, Copy)]
pub struct ServiceConfig {
    /// Service name (static string for no_std)
    pub name: &'static str,
    /// Service version
    pub version: ServiceVersion,
    /// Maximum concurrent requests
    pub max_concurrent_requests: u32,
    /// Request timeout in microseconds
    pub request_timeout_us: u64,
    /// Whether hardware acceleration is available
    pub hw_accel_available: bool,
}

/// Caller identity for permission verification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CallerIdentity {
    /// Process ID
    pub pid: u32,
    /// User ID
    pub uid: u32,
}

impl CallerIdentity {
    /// Create a new caller identity
    pub const fn new(pid: u32, uid: u32) -> Self {
        CallerIdentity { pid, uid }
    }
}

/// Service health status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceHealth {
    /// Service is healthy and running
    Healthy = 0,
    /// Service is degraded (e.g. software fallback)
    Degraded = 1,
    /// Service is unhealthy
    Unhealthy = 2,
    /// Service is not initialized
    NotInitialized = 3,
}

/// Service runtime statistics
#[derive(Debug)]
pub struct ServiceStats {
    /// Total requests served
    pub total_requests: AtomicU64,
    /// Total errors encountered
    pub total_errors: AtomicU64,
    /// Current active requests
    pub active_requests: AtomicU32,
    /// Last request latency in microseconds
    pub last_latency_us: AtomicU64,
    /// Peak memory usage in bytes
    pub peak_memory_bytes: AtomicU64,
}

impl ServiceStats {
    /// Create a new zero-initialized stats
    pub const fn new() -> Self {
        ServiceStats {
            total_requests: AtomicU64::new(0),
            total_errors: AtomicU64::new(0),
            active_requests: AtomicU32::new(0),
            last_latency_us: AtomicU64::new(0),
            peak_memory_bytes: AtomicU64::new(0),
        }
    }

    /// Record a successful request
    pub fn record_request(&self, latency_us: u64) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        self.active_requests.fetch_add(1, Ordering::Relaxed);
        self.last_latency_us.store(latency_us, Ordering::Relaxed);
    }

    /// Record an error
    pub fn record_error(&self) {
        self.total_errors.fetch_add(1, Ordering::Relaxed);
    }

    /// Complete a request
    pub fn complete_request(&self) {
        self.active_requests.fetch_sub(1, Ordering::Relaxed);
    }
}

/// Core processing service trait - all six services must implement this
pub trait CoreProcessingService: Send + Sync {
    /// Get service configuration
    fn config(&self) -> &ServiceConfig;

    /// Initialize the service and register to Nuva IPC
    fn init(&mut self) -> Result<ServiceNodeId, ServiceError>;

    /// Handle an incoming service request via Nuva IPC
    fn handle_request(
        &mut self,
        caller: CallerIdentity,
        request_id: u64,
        payload: &[u8],
    ) -> Result<(), ServiceError>;

    /// Shut down the service gracefully
    fn shutdown(&mut self) -> Result<(), ServiceError>;

    /// Get service health status
    fn health_check(&self) -> ServiceHealth;

    /// Get service statistics
    fn stats(&self) -> &ServiceStats;
}
