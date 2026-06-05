/*
 * Nuva OS - Kernel - Vulkan - Mod
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
 * Nuva OS - Kernel - Vulkan Native Integration Module
 *
 * Vulkan native integration for Nuva OS.
 * Provides zero-copy GPU direct passthrough, capability-based
 * GPU access, and independent Vulkan system call interface.
 *
 * Architecture: Superior to Android (no HAL intermediate layer)
 * and Apple (open standard Vulkan vs proprietary Metal).
 *
 * This module is only compiled when the "vulkan" feature is enabled.
 */

pub mod gpu_capability;
pub mod gpu_memory;
pub mod instance;

pub use gpu_capability::{
    NvGpuCapability, NvGpuPermission, NvGpuCapabilityManager,
    gpu_capability_grant, gpu_capability_check,
};
pub use gpu_memory::{
    NvGpuMemoryRegion, NvGpuMemoryType, NvGpuPageTable, NvVulkanMemoryAllocator,
    NvVulkanCommandSubmit, GpuMapFlags,
    vk_queue_submit_zero_copy, vk_batch_submit,
};
pub use instance::{
    NvVulkanInstance, NvVulkanDevice, NvVulkanPhysicalDevice,
    NvVulkanQueueFamily, NvVulkanMemoryType as NvVulkanMemType,
    NvVulkanInstanceManager, NvVulkanDeviceManager,
    NvGpuMemoryTracker,
    VK_API_VERSION_1_3,
    get_instance_manager, get_device_manager,
};

/// Initialize Vulkan subsystem
pub fn init_vulkan() {
    // Vulkan subsystem initialized
    // Physical device enumeration happens on first Instance creation
}
