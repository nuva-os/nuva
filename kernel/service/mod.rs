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

//! Nuva OS Service Framework
//!
//! System service management framework, supporting SSL, WebKit, SQLite,
//! OpenGL/ES, Media, Location, Telephony, SMS, and other services.
//!
//! # Kernel Features
//!
//! - **System Interface**: Built-in service implementation system interface
//! - **Service Discovery**: Supports automatic service registration and discovery
//! - **Permission Management**: Service access permission control
//! - **IPC Messaging**: Inter-service messaging based on Nuva IPC
//! - **Lifecycle Management**: Service start, stop, and restart
//!
//! # Service Types
//!
//! - SSL/TLS encryption service
//! - WebKit browser engine service
//! - SQLite database service
//! - OpenGL/ES graphics service
//! - Media multimedia service
//! - Location service
//! - Telephony service
//! - SMS service

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use alloc::sync::Arc;
use alloc::string::String;
use alloc::vec::Vec;

// ============================================================================
// Service Type Definitions
// ============================================================================

/// Service type enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceType {
    /// SSL/TLS encryption service
    Ssl = 0,
    /// WebKit browser engine service
    WebKit = 1,
    /// SQLite database service
    Sqlite = 2,
    /// OpenGL/ES graphics service
    OpenGLES = 3,
    /// Media multimedia service
    Media = 4,
    /// Location service
    Location = 5,
    /// Telephony service
    Telephony = 6,
    /// SMS service
    Sms = 7,
    /// System service
    System = 8,
    /// Application service
    Application = 9,
}

/// Service lifecycle state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceState {
    /// Stopped
    Stopped = 0,
    /// Starting
    Starting = 1,
    /// Running
    Running = 2,
    /// Stopping
    Stopping = 3,
    /// Error state
    Error = 4,
    /// Restarting
    Restarting = 5,
}

/// Service access permissions.
#[derive(Debug, Clone, Copy)]
pub struct ServicePermission {
    /// Read permission
    pub read: bool,
    /// Write permission
    pub write: bool,
    /// Execute permission
    pub execute: bool,
    /// Management permission
    pub manage: bool,
}

// ============================================================================
// Service Info Structures
// ============================================================================

/// Unique service identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServiceId {
    /// Service type
    pub service_type: ServiceType,
    /// Instance ID
    pub instance_id: u32,
}

/// Service metadata information.
#[derive(Debug, Clone)]
pub struct ServiceInfo {
    /// Service name
    pub name: String,
    /// Service type
    pub service_type: ServiceType,
    /// Service ID
    pub id: ServiceId,
    /// Service state
    pub state: ServiceState,
    /// Service version
    pub version: String,
    /// Service description
    pub description: String,
    /// Service binary path
    pub path: String,
    /// Access permissions
    pub permission: ServicePermission,
    /// Priority level
    pub priority: u32,
    /// Flags
    pub flags: u32,
}

// ============================================================================
// Service Error Types
// ============================================================================

/// Service operation errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceError {
    /// Service not found
    ServiceNotFound,
    /// Service already exists
    ServiceExists,
    /// Service is not running
    ServiceNotRunning,
    /// Service is already running
    ServiceAlreadyRunning,
    /// Start failed
    StartFailed,
    /// Stop failed
    StopFailed,
    /// Permission denied
    PermissionDenied,
    /// Invalid argument
    InvalidArgument,
    /// Insufficient memory
    NoMemory,
    /// Operation timed out
    Timeout,
    /// IPC error
    IpcError,
    /// Operation not supported
    NotSupported,
}

// ============================================================================
// System Service Interface
// ============================================================================

/// System service operation interface.
///
/// Every service must implement this trait.
pub trait ServiceOps: Send + Sync {
    /// Get service metadata.
    fn get_info(&self) -> &ServiceInfo;

    /// Start the service.
    fn start(&mut self) -> Result<(), ServiceError>;

    /// Stop the service.
    fn stop(&mut self) -> Result<(), ServiceError>;

    /// Restart the service.
    fn restart(&mut self) -> Result<(), ServiceError>;

    /// Get the current service state.
    fn get_state(&self) -> ServiceState;

    /// Set the service state.
    fn set_state(&mut self, state: ServiceState);

    /// Handle an arbitrary service request.
    fn handle_request(&mut self, request: &[u8]) -> Result<Vec<u8>, ServiceError>;

    /// Handle an IPC message.
    fn handle_ipc(&mut self, message: &[u8]) -> Result<Vec<u8>, ServiceError>;

    /// Perform a health check on the service.
    fn health_check(&self) -> Result<bool, ServiceError>;
}

// ============================================================================
// Service Manager
// ============================================================================

use spin::Mutex as SpinLock;

/// Central service manager that tracks all registered services.
pub struct ServiceManager {
    /// Registered services
    services: SpinLock<Vec<Arc<dyn ServiceOps>>>,
    /// Total service count
    service_count: AtomicU32,
    /// Running service count
    running_count: AtomicU32,
}

impl ServiceManager {
    /// Create a new service manager.
    pub fn new() -> Self {
        Self {
            services: SpinLock::new(Vec::new()),
            service_count: AtomicU32::new(0),
            running_count: AtomicU32::new(0),
        }
    }

    /// Register a new service.
    pub fn register_service(&self, service: Arc<dyn ServiceOps>) -> Result<u32, ServiceError> {
        let mut services = self.services.lock();

        // Check if service already exists
        let info = service.get_info();
        for existing in services.iter() {
            if existing.get_info().name == info.name {
                return Err(ServiceError::ServiceExists);
            }
        }

        let service_id = self.service_count.fetch_add(1, Ordering::AcqRel);
        services.push(service);

        Ok(service_id)
    }

    /// Unregister a service.
    pub fn unregister_service(&self, service_id: u32) -> Result<(), ServiceError> {
        let mut services = self.services.lock();

        if (service_id as usize) < services.len() {
            services.remove(service_id as usize);
            self.service_count.fetch_sub(1, Ordering::AcqRel);
            Ok(())
        } else {
            Err(ServiceError::ServiceNotFound)
        }
    }

    /// Start a registered service.
    pub fn start_service(&self, service_id: u32) -> Result<(), ServiceError> {
        let services = self.services.lock();

        if (service_id as usize) < services.len() {
            let service = &services[service_id as usize];
            // TODO: Need mutable access to call service.start()
            // service.start()?;
            self.running_count.fetch_add(1, Ordering::AcqRel);
            Ok(())
        } else {
            Err(ServiceError::ServiceNotFound)
        }
    }

    /// Stop a running service.
    pub fn stop_service(&self, service_id: u32) -> Result<(), ServiceError> {
        let services = self.services.lock();

        if (service_id as usize) < services.len() {
            // TODO: Need mutable access to call service.stop()
            // service.stop()?;
            self.running_count.fetch_sub(1, Ordering::AcqRel);
            Ok(())
        } else {
            Err(ServiceError::ServiceNotFound)
        }
    }

    /// Get a service by its ID.
    pub fn get_service(&self, service_id: u32) -> Option<Arc<dyn ServiceOps>> {
        let services = self.services.lock();

        if (service_id as usize) < services.len() {
            Some(services[service_id as usize].clone())
        } else {
            None
        }
    }

    /// Get all services of a given type.
    pub fn get_services_by_type(&self, service_type: ServiceType) -> Vec<Arc<dyn ServiceOps>> {
        let services = self.services.lock();

        services.iter()
            .filter(|s| s.get_info().service_type == service_type)
            .cloned()
            .collect()
    }

    /// Get a service by name.
    pub fn get_service_by_name(&self, name: &str) -> Option<Arc<dyn ServiceOps>> {
        let services = self.services.lock();

        services.iter()
            .find(|s| s.get_info().name == name)
            .cloned()
    }

    /// Get the total number of registered services.
    pub fn get_service_count(&self) -> u32 {
        self.service_count.load(Ordering::Acquire)
    }

    /// Get the number of running services.
    pub fn get_running_count(&self) -> u32 {
        self.running_count.load(Ordering::Acquire)
    }
}

// ============================================================================
// Global Service Manager
// ============================================================================

/// Global service manager instance.
pub static SERVICE_MANAGER: ServiceManager = ServiceManager {
    services: SpinLock::new(Vec::new()),
    service_count: AtomicU32::new(0),
    running_count: AtomicU32::new(0),
};

// ============================================================================
// Utility Functions
// ============================================================================

/// Initialize the service framework.
pub fn init_service_framework() {
    // Initialize the service manager
    // Register built-in services

    // TODO: Register built-in system services
}

/// Register a service with the global manager.
pub fn register_service(service: Arc<dyn ServiceOps>) -> Result<u32, ServiceError> {
    SERVICE_MANAGER.register_service(service)
}

/// Look up a service from the global manager.
pub fn get_service(service_id: u32) -> Option<Arc<dyn ServiceOps>> {
    SERVICE_MANAGER.get_service(service_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_type() {
        assert_eq!(ServiceType::Ssl as u8, 0);
        assert_eq!(ServiceType::WebKit as u8, 1);
        assert_eq!(ServiceType::Sqlite as u8, 2);
    }

    #[test]
    fn test_service_manager() {
        let manager = ServiceManager::new();
        assert_eq!(manager.get_service_count(), 0);
        assert_eq!(manager.get_running_count(), 0);
    }
}
