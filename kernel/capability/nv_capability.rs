/*
 * Nuva OS - Kernel - Capability - NvCapability
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
 * Nuva OS - Kernel - NvCapability Data Structure
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Nuva native capability token: kernel-issued, unforgeable,
 * capability-based access control replacing uid/gid.
 *
 * INVARIANT: token_id is kernel-issued and unforgeable.
 * INVARIANT: child.rights ⊆ parent.rights (permission monotonicity).
 */

use core::fmt;
use crate::kernel::types::NuvaCapabilityId;
use crate::kernel::types::NuvaProcessId;
use crate::kernel::types::NvTimestamp;

bitflags::bitflags! {
    /// Nuva capability rights set (replaces POSIX permission bits)
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct NvRightsSet: u64 {
        const SEND      = 0b0000_0001;
        const RECEIVE   = 0b0000_0010;
        const READ      = 0b0000_0100;
        const WRITE     = 0b0000_1000;
        const EXECUTE   = 0b0001_0000;
        const TRANSFER  = 0b0010_0000;
        const DERIVE    = 0b0100_0000;
        const REVOKE    = 0b1000_0000;
        const GRANT     = 0b0001_0000_0000;
        const ADMIN     = 0b0010_0000_0000;
        /// SUPERVISOR: NvSupervisorCall gate access (only EL1 can hold)
        const SUPERVISOR= 0b0100_0000_0000;
        const ALL       = 0b0111_1111_1111;
    }
}

/// Nuva resource type for capability binding
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum NvResourceType {
    Port            = 0,
    MemoryRegion    = 1,
    File            = 2,
    Device          = 3,
    Process         = 4,
    Service         = 5,
    Notification    = 6,
    Network         = 7,
    /// SupervisorGate: EL1→EL2 controlled interface gate
    SupervisorGate  = 8,
}

impl fmt::Display for NvResourceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NvResourceType::Port => write!(f, "Port"),
            NvResourceType::MemoryRegion => write!(f, "MemoryRegion"),
            NvResourceType::File => write!(f, "File"),
            NvResourceType::Device => write!(f, "Device"),
            NvResourceType::Process => write!(f, "Process"),
            NvResourceType::Service => write!(f, "Service"),
            NvResourceType::Notification => write!(f, "Notification"),
            NvResourceType::Network => write!(f, "Network"),
            NvResourceType::SupervisorGate => write!(f, "SupervisorGate"),
        }
    }
}

/// Nuva capability token structure
///
/// INVARIANT: token_id is kernel-issued and unforgeable.
/// INVARIANT: child.rights ⊆ parent.rights (permission monotonicity).
#[derive(Debug, Clone)]
pub struct NvCapability {
    /// Kernel-issued unique token identifier
    pub token_id: NuvaCapabilityId,
    /// Resource type this capability governs
    pub resource_type: NvResourceType,
    /// Resource instance identifier
    pub resource_id: u64,
    /// Permission rights set
    pub rights: NvRightsSet,
    /// Parent capability (None for root capabilities)
    pub parent_cap: Option<NuvaCapabilityId>,
    /// Owning process
    pub owner: NuvaProcessId,
    /// Whether this capability has been revoked
    pub revoked: bool,
    /// Creation timestamp
    pub created_at: NvTimestamp,
}

impl NvCapability {
    /// Create a new capability token.
    ///
    /// PRE: token_id must be cryptographically random and kernel-issued.
    /// POST: revoked == false, rights == initial rights.
    pub fn new(
        token_id: NuvaCapabilityId,
        resource_type: NvResourceType,
        resource_id: u64,
        rights: NvRightsSet,
        parent_cap: Option<NuvaCapabilityId>,
        owner: NuvaProcessId,
        created_at: NvTimestamp,
    ) -> Self {
        NvCapability {
            token_id,
            resource_type,
            resource_id,
            rights,
            parent_cap,
            owner,
            revoked: false,
            created_at,
        }
    }

    /// Check if this capability has been revoked.
    pub fn is_revoked(&self) -> bool {
        self.revoked
    }

    /// Check if this capability grants all specified rights.
    ///
    /// POST: returns true iff !revoked && (rights ⊇ required_rights).
    pub fn has_rights(&self, required_rights: NvRightsSet) -> bool {
        !self.revoked && self.rights.contains(required_rights)
    }

    /// Check if child_rights is a valid subset of this capability's rights.
    ///
    /// INVARIANT: child.rights ⊆ parent.rights (permission monotonicity).
    pub fn can_derive(&self, child_rights: NvRightsSet) -> bool {
        !self.revoked && self.rights.contains(child_rights)
    }

    /// Mark this capability as revoked.
    /// Once revoked, cap_check will always return CapabilityDenied.
    pub fn revoke(&mut self) {
        self.revoked = true;
    }
}

impl fmt::Display for NvCapability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "NvCapability(id={}, type={:?}, res={}, rights={:?}, owner={}, revoked={})",
            self.token_id.as_u64(),
            self.resource_type,
            self.resource_id,
            self.rights,
            self.owner.as_u64(),
            self.revoked
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rights_set_basic() {
        let r = NvRightsSet::READ | NvRightsSet::WRITE;
        assert!(r.contains(NvRightsSet::READ));
        assert!(r.contains(NvRightsSet::WRITE));
        assert!(!r.contains(NvRightsSet::EXECUTE));
    }

    #[test]
    fn test_rights_subset() {
        let parent = NvRightsSet::READ | NvRightsSet::WRITE | NvRightsSet::EXECUTE;
        let child = NvRightsSet::READ | NvRightsSet::WRITE;
        assert!(parent.contains(child));
        assert!(!child.contains(parent));
    }

    #[test]
    fn test_capability_not_revoked() {
        let cap = NvCapability::new(
            NuvaCapabilityId::new(1),
            NvResourceType::Port,
            100,
            NvRightsSet::SEND | NvRightsSet::RECEIVE,
            None,
            NuvaProcessId::new(1),
            NvTimestamp::new(0),
        );
        assert!(!cap.is_revoked());
        assert!(cap.has_rights(NvRightsSet::SEND));
        assert!(!cap.has_rights(NvRightsSet::EXECUTE));
    }

    #[test]
    fn test_capability_revoked() {
        let mut cap = NvCapability::new(
            NuvaCapabilityId::new(2),
            NvResourceType::MemoryRegion,
            200,
            NvRightsSet::READ | NvRightsSet::WRITE,
            None,
            NuvaProcessId::new(1),
            NvTimestamp::new(0),
        );
        cap.revoke();
        assert!(cap.is_revoked());
        assert!(!cap.has_rights(NvRightsSet::READ));
    }

    #[test]
    fn test_capability_derive() {
        let cap = NvCapability::new(
            NuvaCapabilityId::new(3),
            NvResourceType::File,
            300,
            NvRightsSet::READ | NvRightsSet::WRITE,
            None,
            NuvaProcessId::new(1),
            NvTimestamp::new(0),
        );
        assert!(cap.can_derive(NvRightsSet::READ));
        assert!(cap.can_derive(NvRightsSet::READ | NvRightsSet::WRITE));
        assert!(!cap.can_derive(NvRightsSet::EXECUTE));
    }
}
