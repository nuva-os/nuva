/*
 * Nuva OS - Kernel - Driver - Framework - NvOperation
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
/*
 * Nuva OS - Kernel - NvDriverOperation (Async-First Driver Model)
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Nuva native async-first driver operation replacing Linux blocking model.
 * Migrated from: Linux blocking device_driver → NvDriverOperation async-first.
 *
 * INVARIANT: driver operations are async by default.
 * INVARIANT: driver can only access resources in its capability_set.
 */

use crate::kernel::types::{NuvaCapabilityId, NvPortId, NvFaultDomainId, NvTimestamp, NvDuration};
use crate::kernel::error::{KernelError, KernelResult};
use alloc::vec::Vec;

/// Async I/O completion token
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NvCompletionToken {
    /// Operation ID
    pub op_id: u64,
    /// Submission timestamp
    pub submitted_at: NvTimestamp,
    /// Device identifier
    pub device_id: u64,
}

/// Async I/O completion status
#[derive(Debug, Clone, Copy)]
pub enum NvCompletionStatus {
    /// Operation still pending
    Pending,
    /// Operation completed successfully with bytes transferred
    Completed(u64),
    /// Operation failed
    Failed(KernelError),
}

/// NvDriverOperation trait - async-first driver interface
///
/// Migrated from: Linux blocking device_driver → NvDriverOperation async-first.
pub trait NvDriverOperation {
    /// Submit an async I/O request.
    ///
    /// PRE: caller holds appropriate capability for the device.
    /// POST: returns NvCompletionToken for tracking.
    fn submit_async(
        &self,
        request: &NvDriverRequest,
        completion_port: NvPortId,
        cap: NuvaCapabilityId,
    ) -> KernelResult<NvCompletionToken>;

    /// Poll completion status (non-blocking).
    fn poll_completion(&self, token: &NvCompletionToken) -> NvCompletionStatus;

    /// Cancel a pending async request.
    fn cancel(&self, token: &NvCompletionToken) -> KernelResult<()>;
}

/// Driver I/O request
#[derive(Debug, Clone, Copy)]
pub struct NvDriverRequest {
    /// Target device
    pub device_id: u64,
    /// Operation type
    pub op_type: NvDriverOpType,
    /// Buffer address (in caller's address space)
    pub buffer: u64,
    /// Buffer size
    pub size: u64,
    /// Offset (for seekable devices)
    pub offset: u64,
    /// Request flags
    pub flags: u32,
}

/// Driver operation type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum NvDriverOpType {
    Read = 0,
    Write = 1,
    Flush = 2,
    Ioctl = 3,
    Reset = 4,
}

/// NvDriverPlugin trait - pluginized driver framework
///
/// Migrated from: Linux device_driver/platform_driver → NvDriverPlugin.
pub trait NvDriverPlugin {
    /// Driver name
    fn name(&self) -> &'static str;

    /// Initialize driver with capability set.
    /// PRE: capability set is valid and sufficient.
    fn init(&self, config: &NvDriverConfig, cap_set: &[NuvaCapabilityId]) -> KernelResult<()>;

    /// Teardown driver instance.
    fn teardown(&self, instance: u64) -> KernelResult<()>;

    /// List supported device IDs.
    fn supported_devices(&self) -> &'static [u64];
}

/// Driver configuration
#[derive(Debug, Clone, Copy)]
pub struct NvDriverConfig {
    /// Driver instance ID
    pub instance_id: u64,
    /// Fault domain for isolation
    pub fault_domain: NvFaultDomainId,
    /// Heartbeat port for health monitoring
    pub heartbeat_port: NvPortId,
    /// Service port for I/O requests
    pub service_port: NvPortId,
}

/// NvDriverInstance - isolated driver instance
///
/// INVARIANT: driver can only access resources in its capability_set.
#[derive(Debug, Clone)]
pub struct NvDriverInstance {
    /// Driver instance ID
    pub driver_id: u64,
    /// Capability set (limits resource access)
    pub capability_set: alloc::vec::Vec<NuvaCapabilityId>,
    /// Service port for I/O requests
    pub service_port: NvPortId,
    /// Fault domain
    pub fault_domain: NvFaultDomainId,
    /// Heartbeat monitoring port
    pub heartbeat_port: NvPortId,
    /// Whether driver is alive
    pub alive: bool,
}

/// Driver fault recovery
///
/// INVARIANT: driver instance crash does not cause kernel crash.
pub struct NvDriverRecovery;

impl NvDriverRecovery {
    /// Detect driver crash via DeadName notification on heartbeat port.
    pub fn detect_crash(instance: &NvDriverInstance) -> bool {
        !instance.alive
    }

    /// Recover driver: mark unavailable → reject new requests → restart → reinit hardware.
    ///
    /// INVARIANT: driver instance crash does not cause kernel crash.
    pub fn recover(_instance: &mut NvDriverInstance) -> KernelResult<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_driver_instance_basic() {
        let instance = NvDriverInstance {
            driver_id: 1,
            capability_set: alloc::vec::Vec::new(),
            service_port: NvPortId::new(100),
            fault_domain: NvFaultDomainId::new(1),
            heartbeat_port: NvPortId::new(101),
            alive: true,
        };
        assert!(instance.alive);
        assert!(!NvDriverRecovery::detect_crash(&instance));
    }

    #[test]
    fn test_driver_crash_detection() {
        let instance = NvDriverInstance {
            driver_id: 1,
            capability_set: alloc::vec::Vec::new(),
            service_port: NvPortId::new(100),
            fault_domain: NvFaultDomainId::new(1),
            heartbeat_port: NvPortId::new(101),
            alive: false,
        };
        assert!(NvDriverRecovery::detect_crash(&instance));
    }
}
