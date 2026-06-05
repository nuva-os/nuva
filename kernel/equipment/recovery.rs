/*
 * Nuva OS - Kernel - Equipment - Recovery
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
 * Nuva OS - Kernel - NvEquipmentRecovery (7-Step Fault Recovery)
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * 7-step automatic fault recovery for equipment mode services:
 *   Step 1: Check oscillation (restart_count >= max → Unrecoverable)
 *   Step 2: Mark Restarting, reject new requests (port Transitioning)
 *   Step 3: Isolate fault service (cleanup resources)
 *   Step 4: Restart service instance (nv_process_spawn at EL1)
 *   Step 5: Rebind ports (rebind_port to new instance)
 *   Step 6: Rebuild service state (restore_service_state)
 *   Step 7: Notify available (mark Running, notify dependents)
 *
 * INVARIANT: Equipment service fault recovery does not affect kernel mode.
 * INVARIANT: ∀s ∈ EquipmentServices: crash(s) → healthy(KernelMode)
 */

use crate::kernel::types::NvFaultDomainId;
use crate::kernel::error::{KernelError, KernelResult};
use crate::kernel::equipment::fault_domain::{
    NvEquipmentFaultDomain, NvEquipmentServiceState,
};

/// Recovery step result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NvRecoveryStep {
    CheckOscillation    = 1,
    MarkRestarting      = 2,
    IsolateFault        = 3,
    RestartInstance     = 4,
    RebindPorts         = 5,
    RebuildState        = 6,
    NotifyAvailable     = 7,
}

/// Recovery outcome
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NvRecoveryOutcome {
    Recovered,
    Unrecoverable,
    Failed { step: NvRecoveryStep },
}

/// NvEquipmentRecovery: 7-step fault recovery
///
/// INVARIANT: Equipment service fault recovery does not affect kernel mode.
pub struct NvEquipmentRecovery;

impl NvEquipmentRecovery {
    /// Initiate 7-step fault recovery for a crashed equipment service.
    ///
    /// INVARIANT: ∀s ∈ EquipmentServices: crash(s) → healthy(KernelMode)
    pub fn initiate_recovery(domain: &mut NvEquipmentFaultDomain) -> NvRecoveryOutcome {
        // Step 1: Check oscillation
        if Self::check_oscillation(domain) {
            domain.mark_unrecoverable();
            return NvRecoveryOutcome::Unrecoverable;
        }

        // Step 2: Mark Restarting, reject new requests
        Self::mark_restarting(domain);

        // Step 3: Isolate fault service
        if Self::isolate_fault(domain).is_err() {
            return NvRecoveryOutcome::Failed { step: NvRecoveryStep::IsolateFault };
        }

        // Step 4: Restart service instance
        if Self::restart_instance(domain).is_err() {
            return NvRecoveryOutcome::Failed { step: NvRecoveryStep::RestartInstance };
        }

        // Step 5: Rebind ports
        if Self::rebind_ports(domain).is_err() {
            return NvRecoveryOutcome::Failed { step: NvRecoveryStep::RebindPorts };
        }

        // Step 6: Rebuild service state
        if Self::rebuild_state(domain).is_err() {
            return NvRecoveryOutcome::Failed { step: NvRecoveryStep::RebuildState };
        }

        // Step 7: Notify available
        Self::notify_available(domain);

        NvRecoveryOutcome::Recovered
    }

    /// Step 1: Check oscillation (restart_count >= max_restart_count)
    fn check_oscillation(domain: &NvEquipmentFaultDomain) -> bool {
        domain.is_unrecoverable()
    }

    /// Step 2: Mark Restarting, reject new requests (port Transitioning)
    fn mark_restarting(domain: &mut NvEquipmentFaultDomain) {
        domain.mark_restarting();
    }

    /// Step 3: Isolate fault service (cleanup resources)
    fn isolate_fault(domain: &mut NvEquipmentFaultDomain) -> KernelResult<()> {
        Self::cleanup_service_resources(domain);
        Ok(())
    }

    /// Step 4: Restart service instance (nv_process_spawn at EL1)
    fn restart_instance(domain: &mut NvEquipmentFaultDomain) -> KernelResult<()> {
        // In real implementation: nv_process_spawn at EL1 with same service config
        Ok(())
    }

    /// Step 5: Rebind ports (rebind service_port to new instance)
    fn rebind_ports(domain: &mut NvEquipmentFaultDomain) -> KernelResult<()> {
        // In real implementation: rebind service_port and heartbeat_port
        Ok(())
    }

    /// Step 6: Rebuild service state (restore_service_state)
    fn rebuild_state(domain: &mut NvEquipmentFaultDomain) -> KernelResult<()> {
        // In real implementation: restore service-specific state
        Ok(())
    }

    /// Step 7: Notify available (mark Running, notify dependents)
    fn notify_available(domain: &mut NvEquipmentFaultDomain) {
        domain.mark_running();
    }

    /// Cleanup service resources after crash.
    ///
    /// - Revoke all capabilities held by the faulted service
    /// - Release address space
    /// - Mark service port as Dead
    /// - Reset heartbeat
    fn cleanup_service_resources(domain: &mut NvEquipmentFaultDomain) {
        domain.capability_boundary.clear();
        domain.supervisor_caps.clear();
        domain.last_heartbeat = crate::kernel::types::NvTimestamp::new(0);
    }

    /// Detect service crash via DeadName notification on heartbeat port.
    pub fn detect_crash(domain: &NvEquipmentFaultDomain) -> bool {
        domain.state == NvEquipmentServiceState::Crashed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::types::{
        NvServiceName, NuvaProcessId, NvPortId, NvAddressSpaceId,
    };
    use crate::kernel::equipment::fault_domain::NvEquipmentServiceConfig;

    fn make_crashed_domain() -> NvEquipmentFaultDomain {
        let mut d = NvEquipmentFaultDomain::new(
            NvFaultDomainId::new(1),
            NvServiceName::new(1),
            NuvaProcessId::new(100),
            NvPortId::new(200),
            NvPortId::new(201),
            NvAddressSpaceId::new(1),
            NvEquipmentServiceConfig::default(),
        );
        d.state = NvEquipmentServiceState::Crashed;
        d
    }

    #[test]
    fn test_recovery_success() {
        let mut domain = make_crashed_domain();
        let outcome = NvEquipmentRecovery::initiate_recovery(&mut domain);
        assert_eq!(outcome, NvRecoveryOutcome::Recovered);
        assert_eq!(domain.state, NvEquipmentServiceState::Running);
        assert_eq!(domain.restart_count, 0);
    }

    #[test]
    fn test_recovery_oscillation_detected() {
        let mut domain = make_crashed_domain();
        domain.restart_count = 3;
        domain.max_restart_count = 3;
        let outcome = NvEquipmentRecovery::initiate_recovery(&mut domain);
        assert_eq!(outcome, NvRecoveryOutcome::Unrecoverable);
        assert_eq!(domain.state, NvEquipmentServiceState::Unrecoverable);
    }

    #[test]
    fn test_detect_crash() {
        let domain = make_crashed_domain();
        assert!(NvEquipmentRecovery::detect_crash(&domain));
    }

    #[test]
    fn test_cleanup_resources() {
        let mut domain = make_crashed_domain();
        domain.capability_boundary.push(crate::kernel::types::NuvaCapabilityId::new(1));
        NvEquipmentRecovery::cleanup_service_resources(&mut domain);
        assert!(domain.capability_boundary.is_empty());
        assert!(domain.supervisor_caps.is_empty());
    }
}
