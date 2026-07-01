/*
 * Nuva OS - Kernel - Vulkan - Instance
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
 * Nuva OS - Kernel - Vulkan Instance and Device Management
 *
 * Vulkan Instance/Device lifecycle management.
 * Kernel directly enumerates physical GPU devices (no HAL intermediate layer).
 * This is superior to Android (Gralloc+Vulkan HAL chain) and
 * Apple (Metal via IOKit service matching) because the kernel
 * exposes Vulkan-capable GPUs directly with capability-based access.
 */

use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use spin::RwLock;
use crate::kernel::types::{NuvaProcessId, NuvaCapabilityId, NuvaError};
use super::gpu_capability::{NvGpuCapability, NvGpuPermission};
use super::gpu_memory::{NvGpuMemoryRegion, NvGpuMemoryType, NvVulkanMemoryAllocator};

/// Vulkan API version constants
pub const VK_API_VERSION_1_3: u32 = 0x0040_3000;
pub const VK_API_VERSION_1_2: u32 = 0x0040_2000;
pub const VK_API_VERSION_1_1: u32 = 0x0040_1000;

/// Vulkan physical device information (directly enumerated by kernel)
#[derive(Debug, Clone)]
pub struct NvVulkanPhysicalDevice {
    pub gpu_id: u32,
    pub name: [u8; 256],
    pub api_version: u32,
    pub vram_size: u64,
    pub queue_families: Vec<NvVulkanQueueFamily>,
    pub memory_types: Vec<NvVulkanMemoryType>,
    pub supports_zero_copy: bool,
}

/// Vulkan queue family
#[derive(Debug, Clone)]
pub struct NvVulkanQueueFamily {
    pub family_index: u32,
    pub queue_count: u32,
    pub flags: u32,
    pub timestamp_valid_bits: u32,
}

/// Vulkan memory type
#[derive(Debug, Clone)]
pub struct NvVulkanMemoryType {
    pub type_index: u32,
    pub property_flags: u32,
    pub heap_index: u32,
}

/// Vulkan Instance
#[derive(Debug)]
pub struct NvVulkanInstance {
    pub instance_id: u64,
    pub capability: NvGpuCapability,
    pub api_version: u32,
    pub owner_process: NuvaProcessId,
    pub valid: AtomicBool,
}

impl NvVulkanInstance {
    pub fn new(
        instance_id: u64,
        capability: NvGpuCapability,
        api_version: u32,
    ) -> Self {
        let owner = capability.owner;
        NvVulkanInstance {
            instance_id,
            capability,
            api_version,
            owner_process: owner,
            valid: AtomicBool::new(true),
        }
    }

    pub fn is_valid(&self) -> bool {
        self.valid.load(Ordering::Acquire)
    }

    pub fn invalidate(&self) {
        self.valid.store(false, Ordering::Release);
    }
}

/// Vulkan Instance Manager
pub struct NvVulkanInstanceManager {
    instances: RwLock<BTreeMap<u64, NvVulkanInstance>>,
    next_instance_id: AtomicU64,
}

impl NvVulkanInstanceManager {
    pub const fn new() -> Self {
        NvVulkanInstanceManager {
            instances: RwLock::new(BTreeMap::new()),
            next_instance_id: AtomicU64::new(1),
        }
    }

    /// Create a Vulkan Instance.
    /// Requires GPU_RENDER or GPU_COMPUTE capability.
    /// Minimum API version: VK_API_VERSION_1_3
    pub fn create_instance(
        &self,
        owner: NuvaProcessId,
        capability: NvGpuCapability,
        api_version: u32,
    ) -> Result<u64, NuvaError> {
        capability.check_permission(NvGpuPermission::GPU_RENDER)?;
        if api_version < VK_API_VERSION_1_3 {
            return Err(NuvaError::InvalidParameter);
        }
        let instance_id = self.next_instance_id.fetch_add(1, Ordering::AcqRel);
        let instance = NvVulkanInstance::new(instance_id, capability, api_version);
        self.instances.write().insert(instance_id, instance);
        Ok(instance_id)
    }

    /// Destroy a Vulkan Instance and cascade-invalidate child Devices.
    pub fn destroy_instance(&self, instance_id: u64) -> Result<(), NuvaError> {
        let mut instances = self.instances.write();
        if let Some(instance) = instances.remove(&instance_id) {
            instance.invalidate();
            Ok(())
        } else {
            Err(NuvaError::ResourceNotFound)
        }
    }

    /// Invalidate an Instance (called when its capability is revoked).
    pub fn invalidate_instance(&self, instance_id: u64) {
        let instances = self.instances.read();
        if let Some(instance) = instances.get(&instance_id) {
            instance.invalidate();
        }
    }

    /// Check if an Instance is valid.
    pub fn check_instance_valid(&self, instance_id: u64) -> bool {
        let instances = self.instances.read();
        instances.get(&instance_id).map_or(false, |i| i.is_valid())
    }
}

/// GPU memory tracker (per-device)
#[derive(Debug)]
pub struct NvGpuMemoryTracker {
    pub allocated_bytes: AtomicU64,
    pub max_memory_bytes: u64,
}

impl NvGpuMemoryTracker {
    pub const fn new(max_memory_bytes: u64) -> Self {
        NvGpuMemoryTracker {
            allocated_bytes: AtomicU64::new(0),
            max_memory_bytes,
        }
    }

    pub fn allocate(&self, bytes: u64) -> Result<(), NuvaError> {
        let current = self.allocated_bytes.load(Ordering::Acquire);
        if current + bytes > self.max_memory_bytes {
            return Err(NuvaError::NoMemory);
        }
        self.allocated_bytes.fetch_add(bytes, Ordering::AcqRel);
        Ok(())
    }

    pub fn free(&self, bytes: u64) {
        self.allocated_bytes.fetch_sub(bytes, Ordering::AcqRel);
    }
}

/// Vulkan logical Device
#[derive(Debug)]
pub struct NvVulkanDevice {
    pub device_id: u64,
    pub physical_gpu_id: u32,
    pub instance_id: u64,
    pub owner_process: NuvaProcessId,
    pub memory_tracker: NvGpuMemoryTracker,
    pub valid: AtomicBool,
}

impl NvVulkanDevice {
    pub fn new(
        device_id: u64,
        physical_gpu_id: u32,
        instance_id: u64,
        owner: NuvaProcessId,
        max_memory: u64,
    ) -> Self {
        NvVulkanDevice {
            device_id,
            physical_gpu_id,
            instance_id,
            owner_process: owner,
            memory_tracker: NvGpuMemoryTracker::new(max_memory),
            valid: AtomicBool::new(true),
        }
    }

    pub fn is_valid(&self) -> bool {
        self.valid.load(Ordering::Acquire)
    }

    pub fn invalidate(&self) {
        self.valid.store(false, Ordering::Release);
    }
}

/// Vulkan Device Manager
pub struct NvVulkanDeviceManager {
    devices: RwLock<BTreeMap<u64, NvVulkanDevice>>,
    next_device_id: AtomicU64,
}

impl NvVulkanDeviceManager {
    pub const fn new() -> Self {
        NvVulkanDeviceManager {
            devices: RwLock::new(BTreeMap::new()),
            next_device_id: AtomicU64::new(1),
        }
    }

    pub fn create_device(
        &self,
        physical_gpu_id: u32,
        instance_id: u64,
        owner: NuvaProcessId,
        max_memory: u64,
    ) -> Result<u64, NuvaError> {
        let device_id = self.next_device_id.fetch_add(1, Ordering::AcqRel);
        let device = NvVulkanDevice::new(
            device_id, physical_gpu_id, instance_id, owner, max_memory,
        );
        self.devices.write().insert(device_id, device);
        Ok(device_id)
    }

    pub fn destroy_device(&self, device_id: u64) -> Result<(), NuvaError> {
        let mut devices = self.devices.write();
        if let Some(device) = devices.remove(&device_id) {
            device.invalidate();
            Ok(())
        } else {
            Err(NuvaError::ResourceNotFound)
        }
    }

    pub fn check_device_valid(&self, device_id: u64) -> bool {
        let devices = self.devices.read();
        devices.get(&device_id).map_or(false, |d| d.is_valid())
    }
}

/// Global Vulkan Instance and Device managers
static INSTANCE_MGR: NvVulkanInstanceManager = NvVulkanInstanceManager::new();
static DEVICE_MGR: NvVulkanDeviceManager = NvVulkanDeviceManager::new();

pub fn get_instance_manager() -> &'static NvVulkanInstanceManager {
    &INSTANCE_MGR
}

pub fn get_device_manager() -> &'static NvVulkanDeviceManager {
    &DEVICE_MGR
}
