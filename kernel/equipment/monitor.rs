/*
 * Nuva OS - Kernel - Equipment - Monitor
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
 * Nuva OS - Kernel - NvEquipmentMonitor
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Dual-mechanism fault detection for EL1 equipment services:
 * 1. DeadName notification: instant crash detection when port receiver dies
 * 2. Heartbeat timeout: periodic liveness detection for dead-loop/resource-exhaustion
 *
 * INVARIANT: DeadName detects service crash, heartbeat detects dead-loop/resource-exhaustion.
 */

use crate::kernel::types::{NvFaultDomainId, NvPortId, NvTimestamp, NvDuration};
use crate::kernel::equipment::fault_domain::{
    NvEquipmentFaultDomain, NvEquipmentServiceState,
};

/// NvEquipmentMonitor: dual-mechanism fault detection
///
/// INVARIANT: DeadName detects service crash, heartbeat detects dead-loop/resource-exhaustion.
pub struct NvEquipmentMonitor {
    /// Port for receiving DeadName notifications
    pub deadname_port: NvPortId,
    /// Heartbeat check interval
    pub heartbeat_check_interval: NvDuration,
}

impl NvEquipmentMonitor {
    /// Create a new equipment monitor
    pub fn new(deadname_port: NvPortId, check_interval: NvDuration) -> Self {
        NvEquipmentMonitor {
            deadname_port,
            heartbeat_check_interval: check_interval,
        }
    }

    /// Handle DeadName notification for instant crash detection.
    ///
    /// When a port's Receive right holder crashes, the kernel marks the port Dead
    /// and sends a DeadName notification to Send right holders.
    /// The monitor receives this notification and identifies the crashed service.
    ///
    /// Returns the fault domain ID of the crashed service.
    pub fn handle_deadname_notification(
        &self,
        dead_port: NvPortId,
        domains: &mut [NvEquipmentFaultDomain],
    ) -> Option<NvFaultDomainId> {
        for domain in domains.iter_mut() {
            if domain.service_port == dead_port && domain.state == NvEquipmentServiceState::Running {
                domain.mark_crashed();
                return Some(domain.domain_id);
            }
        }
        None
    }

    /// Check heartbeats for all running services (periodic liveness detection).
    ///
    /// Iterates all Running state fault domains, checks if elapsed > heartbeat_timeout,
    /// marks as Unhealthy, and returns their domain IDs.
    pub fn check_heartbeats(
        &self,
        now: NvTimestamp,
        domains: &mut [NvEquipmentFaultDomain],
    ) -> alloc::vec::Vec<NvFaultDomainId> {
        let mut unhealthy = alloc::vec::Vec::new();
        for domain in domains.iter_mut() {
            if domain.check_heartbeat_timeout(now) {
                domain.mark_unhealthy();
                unhealthy.push(domain.domain_id);
            }
        }
        unhealthy
    }

    /// Register a heartbeat from an equipment service
    pub fn register_heartbeat(
        &self,
        domain_id: NvFaultDomainId,
        now: NvTimestamp,
        domains: &mut [NvEquipmentFaultDomain],
    ) {
        for domain in domains.iter_mut() {
            if domain.domain_id == domain_id {
                domain.register_heartbeat(now);
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::types::{
        NvServiceName, NuvaProcessId, NvAddressSpaceId, NuvaCapabilityId,
    };
    use crate::kernel::equipment::fault_domain::NvEquipmentServiceConfig;
use alloc::vec;
use alloc::vec::Vec;

    fn make_domain(id: u64, port: u64) -> NvEquipmentFaultDomain {
        let mut d = NvEquipmentFaultDomain::new(
            NvFaultDomainId::new(id),
            NvServiceName::new(id),
            NuvaProcessId::new(id * 100),
            NvPortId::new(port),
            NvPortId::new(port + 1),
            NvAddressSpaceId::new(id),
            NvEquipmentServiceConfig::default(),
        );
        d.state = NvEquipmentServiceState::Running;
        d
    }

    #[test]
    fn test_deadname_crash_detection() {
        let monitor = NvEquipmentMonitor::new(
            NvPortId::new(9999),
            NvDuration::new(1_000_000_000),
        );
        let mut domains = [make_domain(1, 200), make_domain(2, 300)];

        let crashed = monitor.handle_deadname_notification(NvPortId::new(200), &mut domains);
        assert_eq!(crashed, Some(NvFaultDomainId::new(1)));
        assert_eq!(domains[0].state, NvEquipmentServiceState::Crashed);
        assert_eq!(domains[1].state, NvEquipmentServiceState::Running);
    }

    #[test]
    fn test_heartbeat_timeout_detection() {
        let monitor = NvEquipmentMonitor::new(
            NvPortId::new(9999),
            NvDuration::new(1_000_000_000),
        );
        let mut domains = [make_domain(1, 200)];
        domains[0].last_heartbeat = NvTimestamp::new(0);

        let unhealthy = monitor.check_heartbeats(NvTimestamp::new(10_000_000_000), &mut domains);
        assert_eq!(unhealthy, alloc::vec![NvFaultDomainId::new(1)]);
        assert_eq!(domains[0].state, NvEquipmentServiceState::Unhealthy);
    }

    #[test]
    fn test_heartbeat_registration() {
        let monitor = NvEquipmentMonitor::new(
            NvPortId::new(9999),
            NvDuration::new(1_000_000_000),
        );
        let mut domains = [make_domain(1, 200)];
        domains[0].state = NvEquipmentServiceState::Unhealthy;

        monitor.register_heartbeat(NvFaultDomainId::new(1), NvTimestamp::new(5000), &mut domains);
        assert_eq!(domains[0].state, NvEquipmentServiceState::Running);
        assert_eq!(domains[0].last_heartbeat, NvTimestamp::new(5000));
    }
}
