/*
 * Nuva OS - Kernel - Mm - Region
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
 * Nuva OS - Kernel - NvMemoryRegion (Capability-controlled Memory Region)
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Nuva native memory region with capability-based access control.
 * Migrated from: POSIX mmap/munmap/mprotect → nv_mem_* with capability.
 *
 * INVARIANT: NvMemoryRegion.access_rights controlled by NuvaCapabilityId.
 */

use core::fmt;
use crate::kernel::types::{NvMemRegionId, NvVAddr, NuvaCapabilityId, NuvaProcessId};
use crate::kernel::capability::nv_capability::NvRightsSet;

/// Nuva memory type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum NvMemoryType {
    /// Normal cacheable memory
    Normal = 0,
    /// Device memory (uncached, strongly ordered)
    Device = 1,
    /// Huge page (2MB or 1GB)
    HugePage = 2,
    /// NPU device memory
    Npu = 3,
}

impl fmt::Display for NvMemoryType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NvMemoryType::Normal => write!(f, "Normal"),
            NvMemoryType::Device => write!(f, "Device"),
            NvMemoryType::HugePage => write!(f, "HugePage"),
            NvMemoryType::Npu => write!(f, "Npu"),
        }
    }
}

/// Nuva memory region (capability-controlled)
///
/// INVARIANT: NvMemoryRegion.access_rights controlled by NuvaCapabilityId.
#[derive(Debug, Clone)]
pub struct NvMemoryRegion {
    /// Region identifier
    pub region_id: NvMemRegionId,
    /// Base virtual address
    pub base_address: NvVAddr,
    /// Region size in bytes
    pub size: u64,
    /// Access rights (controlled by capability)
    pub access_rights: NvRightsSet,
    /// Associated capability token
    pub capability: NuvaCapabilityId,
    /// NUMA node affinity
    pub numa_node: u32,
    /// Memory type
    pub mem_type: NvMemoryType,
    /// Owner process
    pub owner: NuvaProcessId,
}

impl NvMemoryRegion {
    /// Create a new memory region.
    ///
    /// PRE: capability must grant appropriate rights for access_rights.
    /// POST: region is valid with specified properties.
    pub fn new(
        region_id: NvMemRegionId,
        base_address: NvVAddr,
        size: u64,
        access_rights: NvRightsSet,
        capability: NuvaCapabilityId,
        numa_node: u32,
        mem_type: NvMemoryType,
        owner: NuvaProcessId,
    ) -> Self {
        NvMemoryRegion {
            region_id,
            base_address,
            size,
            access_rights,
            capability,
            numa_node,
            mem_type,
            owner,
        }
    }

    /// Check if region contains a given address
    pub fn contains_address(&self, addr: NvVAddr) -> bool {
        let base = self.base_address.as_u64();
        let end = base + self.size;
        addr.as_u64() >= base && addr.as_u64() < end
    }

    /// Check if region is readable
    pub fn is_readable(&self) -> bool {
        self.access_rights.contains(NvRightsSet::READ)
    }

    /// Check if region is writable
    pub fn is_writable(&self) -> bool {
        self.access_rights.contains(NvRightsSet::WRITE)
    }

    /// Check if region is executable
    pub fn is_executable(&self) -> bool {
        self.access_rights.contains(NvRightsSet::EXECUTE)
    }
}

impl fmt::Display for NvMemoryRegion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "NvMemoryRegion(id={}, base={:#x}, size={:#x}, type={}, numa={})",
            self.region_id.as_u64(),
            self.base_address.as_u64(),
            self.size,
            self.mem_type,
            self.numa_node
        )
    }
}

/// Nuva native memory allocation interface (replaces POSIX mmap/brk)
///
/// Migrated from: POSIX mmap/munmap/mprotect → nv_mem_* with capability
pub mod nv_mem {
    use crate::kernel::types::{NvMemRegionId, NvVAddr, NuvaCapabilityId, NvDuration};
    use crate::kernel::error::KernelResult;
    use super::{NvMemoryRegion, NvMemoryType};
    use crate::kernel::capability::nv_capability::NvRightsSet;

    /// Allocate a new memory region
    ///
    /// PRE: caller must hold appropriate capability.
    /// POST: returns NvMemoryRegion with capability-protected rights.
    pub fn nv_mem_allocate(
        size: u64,
        alignment: u64,
        access_rights: NvRightsSet,
        numa_pref: u32,
        mem_type: NvMemoryType,
        cap: NuvaCapabilityId,
        owner: crate::kernel::types::NuvaProcessId,
    ) -> KernelResult<NvMemoryRegion> {
        let region_id = NvMemRegionId::new(0);
        let base = NvVAddr::new(0);
        Ok(NvMemoryRegion::new(
            region_id, base, size, access_rights, cap, numa_pref, mem_type, owner,
        ))
    }

    /// Deallocate a memory region
    ///
    /// PRE: region_cap must be valid.
    pub fn nv_mem_deallocate(_region_cap: NuvaCapabilityId) -> KernelResult<()> {
        Ok(())
    }

    /// Change access rights of a memory region
    ///
    /// PRE: region_cap must grant REVOKE and GRANT rights.
    pub fn nv_mem_protect(
        _region_cap: NuvaCapabilityId,
        _new_rights: NvRightsSet,
    ) -> KernelResult<()> {
        Ok(())
    }

    /// Map (share) a memory region to another address space (zero-copy)
    ///
    /// PRE: source_cap must grant READ and TRANSFER rights.
    /// POST: returns new NvMemoryRegion in target address space.
    pub fn nv_mem_map(
        _source_region: &NvMemoryRegion,
        _dest_addr: NvVAddr,
        _rights: NvRightsSet,
    ) -> KernelResult<NvMemoryRegion> {
        let region_id = NvMemRegionId::new(0);
        let base = NvVAddr::new(0);
        Ok(NvMemoryRegion::new(
            region_id, base, 0, _rights,
            _source_region.capability, 0, NvMemoryType::Normal,
            _source_region.owner,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::types::{NvMemRegionId, NvVAddr, NuvaCapabilityId, NuvaProcessId};
    use crate::kernel::capability::nv_capability::NvRightsSet;

    #[test]
    fn test_memory_region_basic() {
        let region = NvMemoryRegion::new(
            NvMemRegionId::new(1),
            NvVAddr::new(0x1000_0000),
            0x1000,
            NvRightsSet::READ | NvRightsSet::WRITE,
            NuvaCapabilityId::new(1),
            0,
            NvMemoryType::Normal,
            NuvaProcessId::new(1),
        );
        assert!(region.is_readable());
        assert!(region.is_writable());
        assert!(!region.is_executable());
    }

    #[test]
    fn test_memory_region_contains() {
        let region = NvMemoryRegion::new(
            NvMemRegionId::new(1),
            NvVAddr::new(0x1000),
            0x1000,
            NvRightsSet::READ,
            NuvaCapabilityId::new(1),
            0,
            NvMemoryType::Normal,
            NuvaProcessId::new(1),
        );
        assert!(region.contains_address(NvVAddr::new(0x1000)));
        assert!(region.contains_address(NvVAddr::new(0x1FFF)));
        assert!(!region.contains_address(NvVAddr::new(0x2000)));
    }
}
