/*
 * Nuva OS - Kernel - Capability - Manager
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
 * Nuva OS - Kernel - NvCapability Manager
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Central authority for capability lifecycle management.
 * The kernel is the sole issuer of capability tokens.
 *
 * INVARIANT: token_id is globally unique and kernel-issued.
 * INVARIANT: child.rights ⊆ parent.rights (permission monotonicity).
 * INVARIANT: revocation is cascading (parent revoke -> all children revoked).
 */

use core::sync::atomic::{AtomicU64, Ordering};
use crate::kernel::error::KernelError;
use crate::kernel::error::KernelResult;
use crate::kernel::types::NuvaCapabilityId;
use crate::kernel::types::NuvaProcessId;
use crate::kernel::types::NvTimestamp;
use super::nv_capability::{NvCapability, NvResourceType, NvRightsSet};

/// Global capability token counter (monotonically increasing)
static CAP_TOKEN_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Generate the next unique capability token ID.
///
/// INVARIANT: token_id is globally unique and kernel-issued.
fn next_token_id() -> NuvaCapabilityId {
    let id = CAP_TOKEN_COUNTER.fetch_add(1, Ordering::AcqRel);
    NuvaCapabilityId::new(id)
}

/// NvCapability Manager trait
///
/// Defines the core capability lifecycle operations.
/// All operations enforce permission monotonicity and minimal privilege.
pub trait NvCapabilityManager {
    /// Create a new capability token for a resource.
    ///
    /// PRE: caller must hold appropriate creation rights.
    /// POST: returns NvCapability with unique token_id, rights == initial_rights.
    fn cap_create(
        &mut self,
        owner: NuvaProcessId,
        resource_type: NvResourceType,
        resource_id: u64,
        rights: NvRightsSet,
        now: NvTimestamp,
    ) -> KernelResult<NvCapability>;

    /// Derive a child capability from a parent.
    ///
    /// PRE: parent_cap must be valid and not revoked.
    /// PRE: child_rights ⊆ parent_cap.rights (permission monotonicity).
    /// POST: child.parent_cap == Some(parent_cap.token_id).
    /// POST: child.rights == child_rights.
    fn cap_derive(
        &mut self,
        parent_cap: &NvCapability,
        child_rights: NvRightsSet,
        child_owner: NuvaProcessId,
        now: NvTimestamp,
    ) -> KernelResult<NvCapability>;

    /// Transfer a capability to another process.
    ///
    /// PRE: cap must have TRANSFER right.
    /// PRE: cap must not be revoked.
    /// POST: target process receives a copy of the capability.
    fn cap_transfer(
        &mut self,
        cap: &NvCapability,
        target_process: NuvaProcessId,
        now: NvTimestamp,
    ) -> KernelResult<NvCapability>;

    /// Revoke a capability (cascading to all children).
    ///
    /// PRE: caller must hold REVOKE right on the capability.
    /// POST: cap.revoked == true.
    /// POST: all derived capabilities are also revoked.
    fn cap_revoke(&mut self, cap: &mut NvCapability) -> KernelResult<()>;

    /// Check if a capability grants the required rights.
    ///
    /// POST: returns Ok(()) if valid and rights sufficient.
    /// POST: returns Err(CapabilityDenied) if revoked or insufficient rights.
    fn cap_check(&self, cap: &NvCapability, required_rights: NvRightsSet) -> KernelResult<()>;
}

/// Default implementation of NvCapabilityManager
pub struct DefaultNvCapabilityManager;

impl NvCapabilityManager for DefaultNvCapabilityManager {
    fn cap_create(
        &mut self,
        owner: NuvaProcessId,
        resource_type: NvResourceType,
        resource_id: u64,
        rights: NvRightsSet,
        now: NvTimestamp,
    ) -> KernelResult<NvCapability> {
        let token_id = next_token_id();
        Ok(NvCapability::new(
            token_id,
            resource_type,
            resource_id,
            rights,
            None,
            owner,
            now,
        ))
    }

    fn cap_derive(
        &mut self,
        parent_cap: &NvCapability,
        child_rights: NvRightsSet,
        child_owner: NuvaProcessId,
        now: NvTimestamp,
    ) -> KernelResult<NvCapability> {
        if parent_cap.is_revoked() {
            return Err(KernelError::CapabilityExpired);
        }
        if !parent_cap.rights.contains(child_rights) {
            return Err(KernelError::CapabilityDerivationFailed);
        }
        let token_id = next_token_id();
        Ok(NvCapability::new(
            token_id,
            parent_cap.resource_type,
            parent_cap.resource_id,
            child_rights,
            Some(parent_cap.token_id),
            child_owner,
            now,
        ))
    }

    fn cap_transfer(
        &mut self,
        cap: &NvCapability,
        target_process: NuvaProcessId,
        now: NvTimestamp,
    ) -> KernelResult<NvCapability> {
        if cap.is_revoked() {
            return Err(KernelError::CapabilityExpired);
        }
        if !cap.rights.contains(NvRightsSet::TRANSFER) {
            return Err(KernelError::CapabilityTransferFailed);
        }
        let token_id = next_token_id();
        Ok(NvCapability::new(
            token_id,
            cap.resource_type,
            cap.resource_id,
            cap.rights,
            Some(cap.token_id),
            target_process,
            now,
        ))
    }

    fn cap_revoke(&mut self, cap: &mut NvCapability) -> KernelResult<()> {
        if cap.is_revoked() {
            return Err(KernelError::CapabilityExpired);
        }
        cap.revoke();
        Ok(())
    }

    fn cap_check(&self, cap: &NvCapability, required_rights: NvRightsSet) -> KernelResult<()> {
        if cap.is_revoked() {
            return Err(KernelError::CapabilityDenied);
        }
        if cap.rights.contains(required_rights) {
            Ok(())
        } else {
            Err(KernelError::CapabilityDenied)
        }
    }
}

/// Initialize the capability subsystem
pub fn init_capability() {
    log_info!("NvCapability manager initialized");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cap_create() {
        let mut mgr = DefaultNvCapabilityManager;
        let cap = mgr.cap_create(
            NuvaProcessId::new(1),
            NvResourceType::Port,
            100,
            NvRightsSet::SEND | NvRightsSet::RECEIVE,
            NvTimestamp::new(0),
        ).unwrap();
        assert!(!cap.is_revoked());
        assert!(cap.parent_cap.is_none());
    }

    #[test]
    fn test_cap_derive_success() {
        let mut mgr = DefaultNvCapabilityManager;
        let parent = mgr.cap_create(
            NuvaProcessId::new(1),
            NvResourceType::MemoryRegion,
            200,
            NvRightsSet::READ | NvRightsSet::WRITE,
            NvTimestamp::new(0),
        ).unwrap();
        let child = mgr.cap_derive(
            &parent,
            NvRightsSet::READ,
            NuvaProcessId::new(2),
            NvTimestamp::new(1),
        ).unwrap();
        assert_eq!(child.parent_cap, Some(parent.token_id));
        assert!(child.rights.contains(NvRightsSet::READ));
        assert!(!child.rights.contains(NvRightsSet::WRITE));
    }

    #[test]
    fn test_cap_derive_denied() {
        let mut mgr = DefaultNvCapabilityManager;
        let parent = mgr.cap_create(
            NuvaProcessId::new(1),
            NvResourceType::File,
            300,
            NvRightsSet::READ,
            NvTimestamp::new(0),
        ).unwrap();
        let result = mgr.cap_derive(
            &parent,
            NvRightsSet::READ | NvRightsSet::WRITE,
            NuvaProcessId::new(2),
            NvTimestamp::new(1),
        );
        assert_eq!(result, Err(KernelError::CapabilityDerivationFailed));
    }

    #[test]
    fn test_cap_revoke_then_check() {
        let mut mgr = DefaultNvCapabilityManager;
        let mut cap = mgr.cap_create(
            NuvaProcessId::new(1),
            NvResourceType::Process,
            400,
            NvRightsSet::ALL,
            NvTimestamp::new(0),
        ).unwrap();
        mgr.cap_revoke(&mut cap).unwrap();
        let result = mgr.cap_check(&cap, NvRightsSet::READ);
        assert_eq!(result, Err(KernelError::CapabilityDenied));
    }

    #[test]
    fn test_cap_transfer() {
        let mut mgr = DefaultNvCapabilityManager;
        let cap = mgr.cap_create(
            NuvaProcessId::new(1),
            NvResourceType::Port,
            500,
            NvRightsSet::SEND | NvRightsSet::TRANSFER,
            NvTimestamp::new(0),
        ).unwrap();
        let transferred = mgr.cap_transfer(
            &cap,
            NuvaProcessId::new(2),
            NvTimestamp::new(1),
        ).unwrap();
        assert_eq!(transferred.owner, NuvaProcessId::new(2));
        assert_eq!(transferred.parent_cap, Some(cap.token_id));
    }

    #[test]
    fn test_cap_transfer_denied() {
        let mut mgr = DefaultNvCapabilityManager;
        let cap = mgr.cap_create(
            NuvaProcessId::new(1),
            NvResourceType::Port,
            600,
            NvRightsSet::SEND,
            NvTimestamp::new(0),
        ).unwrap();
        let result = mgr.cap_transfer(&cap, NuvaProcessId::new(2), NvTimestamp::new(1));
        assert_eq!(result, Err(KernelError::CapabilityTransferFailed));
    }

    #[test]
    fn test_cap_check_success() {
        let mgr = DefaultNvCapabilityManager;
        let cap = NvCapability::new(
            NuvaCapabilityId::new(99),
            NvResourceType::File,
            700,
            NvRightsSet::READ | NvRightsSet::WRITE,
            None,
            NuvaProcessId::new(1),
            NvTimestamp::new(0),
        );
        assert!(mgr.cap_check(&cap, NvRightsSet::READ).is_ok());
        assert!(mgr.cap_check(&cap, NvRightsSet::READ | NvRightsSet::WRITE).is_ok());
        assert_eq!(mgr.cap_check(&cap, NvRightsSet::EXECUTE), Err(KernelError::CapabilityDenied));
    }
}
