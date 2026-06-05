/*
 * Nuva OS - Kernel - Core - SupervisorCall
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
 * Nuva OS - Kernel - NvSupervisorCall (EL1→EL2 Capability-Gated Interface)
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * The only controlled interface for EL1→EL2 access.
 * Every operation requires capability check.
 *
 * INVARIANT: All EL1→EL2 access goes through NvSupervisorCall with cap_check.
 * INVARIANT: EL1 services' SUPERVISOR capability is explicitly granted by EL2 kernel.
 */

use crate::kernel::types::{NvSupervisorOp, NvPrivilegeLevel, NuvaCapabilityId, NvVAddr, NvDuration};
use crate::kernel::error::{KernelError, KernelResult};
use crate::kernel::capability::nv_capability::NvRightsSet;

/// NvSupervisorCall: the sole controlled interface from EL1 to EL2
///
/// INVARIANT: All EL1→EL2 access goes through NvSupervisorCall with cap_check.
pub struct NvSupervisorCall;

impl NvSupervisorCall {
    /// Execute a supervisor call from EL1 to EL2.
    ///
    /// PRE: caller.privilege_level == EquipmentMode
    /// PRE: caller holds SUPERVISOR capability for the operation
    /// POST: operation executed with kernel privilege if cap_check passes
    /// POST: returns SupervisorCallDenied if capability check fails
    pub fn supervisor_call(
        caller_cap: NuvaCapabilityId,
        operation: NvSupervisorOp,
        args: &[u64],
        cap_check: impl Fn(NuvaCapabilityId, NvRightsSet) -> KernelResult<()>,
    ) -> KernelResult<NvSupervisorResult> {
        cap_check(caller_cap, NvRightsSet::SUPERVISOR)?;

        match operation {
            NvSupervisorOp::MapDeviceMemory => {
                Self::map_device_memory(args)
            }
            NvSupervisorOp::UnmapDeviceMemory => {
                Self::unmap_device_memory(args)
            }
            NvSupervisorOp::DmaMap => {
                Self::dma_map(args)
            }
            NvSupervisorOp::DmaUnmap => {
                Self::dma_unmap(args)
            }
            NvSupervisorOp::IrqRequest => {
                Self::irq_request(args)
            }
            NvSupervisorOp::IrqRelease => {
                Self::irq_release(args)
            }
            NvSupervisorOp::IrqEnable => {
                Self::irq_enable(args)
            }
            NvSupervisorOp::IrqDisable => {
                Self::irq_disable(args)
            }
            NvSupervisorOp::TimerSet => {
                Self::timer_set(args)
            }
            NvSupervisorOp::TimerCancel => {
                Self::timer_cancel(args)
            }
            NvSupervisorOp::CapDeriveForService => {
                Self::cap_derive_for_service(args)
            }
            NvSupervisorOp::CapRevokeFromService => {
                Self::cap_revoke_from_service(args)
            }
            NvSupervisorOp::PortCreateForService => {
                Self::port_create_for_service(args)
            }
            NvSupervisorOp::PortDestroyForService => {
                Self::port_destroy_for_service(args)
            }
        }
    }

    /// Check if a service is authorized for a supervisor operation
    pub fn is_authorized(
        service_cap: NuvaCapabilityId,
        operation: NvSupervisorOp,
        cap_check: impl Fn(NuvaCapabilityId, NvRightsSet) -> KernelResult<()>,
    ) -> bool {
        cap_check(service_cap, NvRightsSet::SUPERVISOR).is_ok()
    }

    fn map_device_memory(args: &[u64]) -> KernelResult<NvSupervisorResult> {
        let _ = args;
        Ok(NvSupervisorResult { value: 0, output: [0; 4] })
    }

    fn unmap_device_memory(args: &[u64]) -> KernelResult<NvSupervisorResult> {
        let _ = args;
        Ok(NvSupervisorResult { value: 0, output: [0; 4] })
    }

    fn dma_map(args: &[u64]) -> KernelResult<NvSupervisorResult> {
        let _ = args;
        Ok(NvSupervisorResult { value: 0, output: [0; 4] })
    }

    fn dma_unmap(args: &[u64]) -> KernelResult<NvSupervisorResult> {
        let _ = args;
        Ok(NvSupervisorResult { value: 0, output: [0; 4] })
    }

    fn irq_request(args: &[u64]) -> KernelResult<NvSupervisorResult> {
        let _ = args;
        Ok(NvSupervisorResult { value: 0, output: [0; 4] })
    }

    fn irq_release(args: &[u64]) -> KernelResult<NvSupervisorResult> {
        let _ = args;
        Ok(NvSupervisorResult { value: 0, output: [0; 4] })
    }

    fn irq_enable(args: &[u64]) -> KernelResult<NvSupervisorResult> {
        let _ = args;
        Ok(NvSupervisorResult { value: 0, output: [0; 4] })
    }

    fn irq_disable(args: &[u64]) -> KernelResult<NvSupervisorResult> {
        let _ = args;
        Ok(NvSupervisorResult { value: 0, output: [0; 4] })
    }

    fn timer_set(args: &[u64]) -> KernelResult<NvSupervisorResult> {
        let _ = args;
        Ok(NvSupervisorResult { value: 0, output: [0; 4] })
    }

    fn timer_cancel(args: &[u64]) -> KernelResult<NvSupervisorResult> {
        let _ = args;
        Ok(NvSupervisorResult { value: 0, output: [0; 4] })
    }

    fn cap_derive_for_service(args: &[u64]) -> KernelResult<NvSupervisorResult> {
        let _ = args;
        Ok(NvSupervisorResult { value: 0, output: [0; 4] })
    }

    fn cap_revoke_from_service(args: &[u64]) -> KernelResult<NvSupervisorResult> {
        let _ = args;
        Ok(NvSupervisorResult { value: 0, output: [0; 4] })
    }

    fn port_create_for_service(args: &[u64]) -> KernelResult<NvSupervisorResult> {
        let _ = args;
        Ok(NvSupervisorResult { value: 0, output: [0; 4] })
    }

    fn port_destroy_for_service(args: &[u64]) -> KernelResult<NvSupervisorResult> {
        let _ = args;
        Ok(NvSupervisorResult { value: 0, output: [0; 4] })
    }
}

/// NvSupervisorCall result
#[derive(Debug, Clone, Copy)]
pub struct NvSupervisorResult {
    pub value: u64,
    pub output: [u64; 4],
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cap_check_ok(_: NuvaCapabilityId, _: NvRightsSet) -> KernelResult<()> {
        Ok(())
    }

    fn cap_check_denied(_: NuvaCapabilityId, _: NvRightsSet) -> KernelResult<()> {
        Err(KernelError::SupervisorCallDenied)
    }

    #[test]
    fn test_supervisor_call_authorized() {
        let result = NvSupervisorCall::supervisor_call(
            NuvaCapabilityId::new(1),
            NvSupervisorOp::MapDeviceMemory,
            &[0x1000_0000, 0x1000],
            cap_check_ok,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_supervisor_call_denied() {
        let result = NvSupervisorCall::supervisor_call(
            NuvaCapabilityId::new(1),
            NvSupervisorOp::MapDeviceMemory,
            &[0x1000_0000, 0x1000],
            cap_check_denied,
        );
        assert_eq!(result, Err(KernelError::SupervisorCallDenied));
    }

    #[test]
    fn test_is_authorized() {
        assert!(NvSupervisorCall::is_authorized(NuvaCapabilityId::new(1), NvSupervisorOp::IrqRequest, cap_check_ok));
        assert!(!NvSupervisorCall::is_authorized(NuvaCapabilityId::new(1), NvSupervisorOp::IrqRequest, cap_check_denied));
    }
}
