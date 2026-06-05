/*
 * Nuva OS - Kernel - Vulkan - GpuCapability
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
 *
 * Nuva OS - Kernel - Vulkan GPU Capability Security Model
 *
 * Nuva native GPU capability system for secure Vulkan access.
 * Replaces Unix uid/gid-based GPU access control with fine-grained
 * capability tokens (NvGpuCapability).
 *
 * Design: nuva is not unix, nuva is not linux.
 * GPU access is governed by capability tokens, not file permissions.
 * This is superior to Android (Gralloc HAL permission checks) and
 * Apple (Metal process entitlements) because capabilities are
 * delegatable, revocable, and enforce memory quotas at the kernel level.
 */

use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use crate::types::{NuvaCapabilityId, NuvaProcessId, NuvaError};

bitflags::bitflags! {
    /// GPU capability permission flags.
    /// Fine-grained access control for Vulkan GPU operations.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct NvGpuPermission: u32 {
        const GPU_COMPUTE   = 0b0000_0001;
        const GPU_RENDER    = 0b0000_0010;
        const GPU_MEMORY    = 0b0000_0100;
        const GPU_PRESENT   = 0b0000_1000;
        const GPU_VIDEO     = 0b0001_0000;
        const GPU_ALL       = 0b0001_1111;
    }
}

/// GPU capability token for secure Vulkan access.
/// Bound to a specific process, with permission flags and memory quota.
#[derive(Debug)]
pub struct NvGpuCapability {
    pub cap_id: NuvaCapabilityId,
    pub owner: NuvaProcessId,
    pub permissions: NvGpuPermission,
    pub max_memory_bytes: u64,
    pub current_memory_bytes: AtomicU64,
    pub bound_instance_id: AtomicU64,
    pub valid: AtomicBool,
}

impl NvGpuCapability {
    pub fn new(
        cap_id: NuvaCapabilityId,
        owner: NuvaProcessId,
        permissions: NvGpuPermission,
        max_memory_bytes: u64,
    ) -> Self {
        NvGpuCapability {
            cap_id,
            owner,
            permissions,
            max_memory_bytes,
            current_memory_bytes: AtomicU64::new(0),
            bound_instance_id: AtomicU64::new(0),
            valid: AtomicBool::new(true),
        }
    }

    /// Check if this capability permits the requested GPU operation.
    pub fn check_permission(&self, required: NvGpuPermission) -> Result<(), NuvaError> {
        if !self.valid.load(Ordering::Acquire) {
            return Err(NuvaError::CapabilityExpired);
        }
        if self.permissions.contains(required) {
            Ok(())
        } else {
            Err(NuvaError::CapabilityDenied)
        }
    }

    /// Check if allocating `bytes` of GPU memory is within quota.
    pub fn check_memory_quota(&self, bytes: u64) -> Result<(), NuvaError> {
        if !self.valid.load(Ordering::Acquire) {
            return Err(NuvaError::CapabilityExpired);
        }
        let current = self.current_memory_bytes.load(Ordering::Acquire);
        if current + bytes <= self.max_memory_bytes {
            Ok(())
        } else {
            Err(NuvaError::NoMemory)
        }
    }

    /// Record GPU memory allocation.
    pub fn allocate_memory(&self, bytes: u64) -> Result<(), NuvaError> {
        self.check_memory_quota(bytes)?;
        self.current_memory_bytes.fetch_add(bytes, Ordering::AcqRel);
        Ok(())
    }

    /// Record GPU memory deallocation.
    pub fn free_memory(&self, bytes: u64) {
        let prev = self.current_memory_bytes.fetch_sub(bytes, Ordering::AcqRel);
        let _ = prev;
    }

    /// Bind this capability to a Vulkan Instance.
    /// When the capability is revoked, the bound Instance is invalidated.
    pub fn bind_instance(&self, instance_id: u64) {
        self.bound_instance_id.store(instance_id, Ordering::Release);
    }

    /// Get the bound Vulkan Instance ID.
    pub fn get_bound_instance(&self) -> u64 {
        self.bound_instance_id.load(Ordering::Acquire)
    }

    /// Revoke this capability. Cascades to bound Vulkan Instance.
    pub fn revoke(&self) -> u64 {
        self.valid.store(false, Ordering::Release);
        self.bound_instance_id.swap(0, Ordering::AcqRel)
    }

    /// Check if this capability is still valid.
    pub fn is_valid(&self) -> bool {
        self.valid.load(Ordering::Acquire)
    }
}

/// GPU capability manager for tracking all GPU capabilities.
pub struct NvGpuCapabilityManager {
    next_cap_id: AtomicU64,
}

impl NvGpuCapabilityManager {
    pub const fn new() -> Self {
        NvGpuCapabilityManager {
            next_cap_id: AtomicU64::new(1),
        }
    }

    /// Grant a new GPU capability to a process.
    pub fn grant(
        &self,
        owner: NuvaProcessId,
        permissions: NvGpuPermission,
        max_memory_bytes: u64,
    ) -> NvGpuCapability {
        let cap_id = NuvaCapabilityId::new(self.next_cap_id.fetch_add(1, Ordering::AcqRel));
        NvGpuCapability::new(cap_id, owner, permissions, max_memory_bytes)
    }

    /// Check a GPU capability for the required permission.
    pub fn check(
        cap: &NvGpuCapability,
        required: NvGpuPermission,
    ) -> Result<(), NuvaError> {
        cap.check_permission(required)
    }

    /// Revoke a GPU capability. Returns the bound Instance ID for cascade invalidation.
    pub fn revoke(cap: &NvGpuCapability) -> u64 {
        cap.revoke()
    }
}

/// Global GPU capability manager
static GPU_CAP_MGR: NvGpuCapabilityManager = NvGpuCapabilityManager::new();

/// Grant GPU capability (convenience function)
pub fn gpu_capability_grant(
    owner: NuvaProcessId,
    permissions: NvGpuPermission,
    max_memory_bytes: u64,
) -> NvGpuCapability {
    GPU_CAP_MGR.grant(owner, permissions, max_memory_bytes)
}

/// Check GPU capability (convenience function)
pub fn gpu_capability_check(
    cap: &NvGpuCapability,
    required: NvGpuPermission,
) -> Result<(), NuvaError> {
    NvGpuCapabilityManager::check(cap, required)
}
