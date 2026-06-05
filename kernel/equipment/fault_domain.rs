/*
 * Nuva OS - Kernel - Equipment - FaultDomain
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
 * Nuva OS - Kernel - NvEquipmentFaultDomain
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Independent fault domain for each EL1 equipment service.
 *
 * INVARIANT: Equipment services are isolated by independent fault domains.
 * INVARIANT: One service fault does not affect other services.
 */

use crate::kernel::types::{
    NvFaultDomainId, NvAddressSpaceId, NvServiceName, NvPortId,
    NuvaProcessId, NuvaCapabilityId, NvTimestamp, NvDuration,
};
use crate::kernel::capability::nv_capability::NvRightsSet;

/// Equipment service state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum NvEquipmentServiceState {
    Starting     = 0,
    Running      = 1,
    Unhealthy    = 2,
    Crashed      = 3,
    Restarting   = 4,
    Unrecoverable= 5,
    Stopped      = 6,
}

/// Restart policy for equipment services
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum NvRestartPolicy {
    AutoRestart   = 0,
    ManualRestart = 1,
    NoRestart     = 2,
}

/// Equipment service configuration
#[derive(Debug, Clone)]
pub struct NvEquipmentServiceConfig {
    /// Maximum restart attempts before marking Unrecoverable
    pub max_restart_count: u32,
    /// Heartbeat timeout
    pub heartbeat_timeout: NvDuration,
    /// Restart policy
    pub restart_policy: NvRestartPolicy,
    /// Service priority for scheduling
    pub priority: i32,
}

impl Default for NvEquipmentServiceConfig {
    fn default() -> Self {
        NvEquipmentServiceConfig {
            max_restart_count: 3,
            heartbeat_timeout: NvDuration::new(5_000_000_000),
            restart_policy: NvRestartPolicy::AutoRestart,
            priority: 0,
        }
    }
}

/// NvEquipmentFaultDomain: independent fault domain for an EL1 service
///
/// INVARIANT: Equipment services are isolated by independent fault domains.
#[derive(Debug, Clone)]
pub struct NvEquipmentFaultDomain {
    /// Unique fault domain identifier
    pub domain_id: NvFaultDomainId,
    /// Service name
    pub service_name: NvServiceName,
    /// Service process ID
    pub service_pid: NuvaProcessId,
    /// Service IPC port
    pub service_port: NvPortId,
    /// Heartbeat monitoring port
    pub heartbeat_port: NvPortId,
    /// Independent address space (memory isolation)
    pub address_space: NvAddressSpaceId,
    /// Independent capability boundary
    pub capability_boundary: alloc::vec::Vec<NuvaCapabilityId>,
    /// EL1→EL2 authorized supervisor call operations
    pub supervisor_caps: alloc::vec::Vec<NuvaCapabilityId>,
    /// Restart policy
    pub restart_policy: NvRestartPolicy,
    /// Current restart count
    pub restart_count: u32,
    /// Maximum restart attempts
    pub max_restart_count: u32,
    /// Heartbeat timeout duration
    pub heartbeat_timeout: NvDuration,
    /// Last heartbeat timestamp
    pub last_heartbeat: NvTimestamp,
    /// Current service state
    pub state: NvEquipmentServiceState,
    /// Service configuration
    pub config: NvEquipmentServiceConfig,
}

impl NvEquipmentFaultDomain {
    /// Create a new fault domain for an equipment service
    pub fn new(
        domain_id: NvFaultDomainId,
        service_name: NvServiceName,
        service_pid: NuvaProcessId,
        service_port: NvPortId,
        heartbeat_port: NvPortId,
        address_space: NvAddressSpaceId,
        config: NvEquipmentServiceConfig,
    ) -> Self {
        NvEquipmentFaultDomain {
            domain_id,
            service_name,
            service_pid,
            service_port,
            heartbeat_port,
            address_space,
            capability_boundary: alloc::vec::Vec::new(),
            supervisor_caps: alloc::vec::Vec::new(),
            restart_policy: config.restart_policy,
            restart_count: 0,
            max_restart_count: config.max_restart_count,
            heartbeat_timeout: config.heartbeat_timeout,
            last_heartbeat: NvTimestamp::new(0),
            state: NvEquipmentServiceState::Starting,
            config,
        }
    }

    /// Check if the service is in a healthy state
    pub fn is_healthy(&self) -> bool {
        self.state == NvEquipmentServiceState::Running
    }

    /// Check if the service has exceeded restart threshold
    pub fn is_unrecoverable(&self) -> bool {
        self.state == NvEquipmentServiceState::Unrecoverable
            || self.restart_count >= self.max_restart_count
    }

    /// Register a heartbeat from the service
    pub fn register_heartbeat(&mut self, now: NvTimestamp) {
        self.last_heartbeat = now;
        if self.state == NvEquipmentServiceState::Unhealthy {
            self.state = NvEquipmentServiceState::Running;
        }
    }

    /// Check if heartbeat has timed out
    pub fn check_heartbeat_timeout(&self, now: NvTimestamp) -> bool {
        if self.state != NvEquipmentServiceState::Running {
            return false;
        }
        let elapsed = now.as_u64().saturating_sub(self.last_heartbeat.as_u64());
        elapsed > self.heartbeat_timeout.as_u64()
    }

    /// Mark service as crashed
    pub fn mark_crashed(&mut self) {
        self.state = NvEquipmentServiceState::Crashed;
    }

    /// Mark service as restarting
    pub fn mark_restarting(&mut self) {
        self.state = NvEquipmentServiceState::Restarting;
        self.restart_count += 1;
    }

    /// Mark service as running (after successful restart)
    pub fn mark_running(&mut self) {
        self.state = NvEquipmentServiceState::Running;
        self.restart_count = 0;
    }

    /// Mark service as unrecoverable
    pub fn mark_unrecoverable(&mut self) {
        self.state = NvEquipmentServiceState::Unrecoverable;
    }

    /// Mark service as unhealthy (heartbeat timeout)
    pub fn mark_unhealthy(&mut self) {
        self.state = NvEquipmentServiceState::Unhealthy;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_domain() -> NvEquipmentFaultDomain {
        NvEquipmentFaultDomain::new(
            NvFaultDomainId::new(1),
            NvServiceName::new(1),
            NuvaProcessId::new(100),
            NvPortId::new(200),
            NvPortId::new(201),
            NvAddressSpaceId::new(1),
            NvEquipmentServiceConfig::default(),
        )
    }

    #[test]
    fn test_fault_domain_initial_state() {
        let d = make_domain();
        assert_eq!(d.state, NvEquipmentServiceState::Starting);
        assert!(!d.is_healthy());
    }

    #[test]
    fn test_fault_domain_mark_running() {
        let mut d = make_domain();
        d.state = NvEquipmentServiceState::Running;
        assert!(d.is_healthy());
    }

    #[test]
    fn test_fault_domain_heartbeat() {
        let mut d = make_domain();
        d.state = NvEquipmentServiceState::Running;
        d.register_heartbeat(NvTimestamp::new(1000));
        assert!(!d.check_heartbeat_timeout(NvTimestamp::new(5000)));
        assert!(d.check_heartbeat_timeout(NvTimestamp::new(10_000_000_000)));
    }

    #[test]
    fn test_fault_domain_restart_threshold() {
        let mut d = make_domain();
        d.mark_crashed();
        d.mark_restarting();
        d.mark_crashed();
        d.mark_restarting();
        d.mark_crashed();
        d.mark_restarting();
        assert!(d.is_unrecoverable());
    }

    #[test]
    fn test_fault_domain_unrecoverable() {
        let mut d = make_domain();
        d.mark_unrecoverable();
        assert!(d.is_unrecoverable());
    }
}
