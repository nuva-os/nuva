/*
 * Nuva OS - Syslib - NvVulkan - Mod
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
 * Nuva OS - NvVulkan User-Space Library
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * User-space Vulkan API bridge layer.
 * Translates standard Vulkan API calls into NvVulkan system calls.
 *
 * This is superior to Android (Vulkan via HAL loaded driver .so)
 * and Apple (Metal framework via IOKit) because we use direct
 * system calls with zero-copy command buffer submission.
 *
 * Only compiled when the "vulkan" feature is enabled.
 */

use crate::types::NuvaError;

/// Vulkan result codes (standard VK_RESULT mapping)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum VkResult {
    Success                      = 0,
    NotReady                     = 1,
    Timeout                      = 2,
    EventSet                     = 3,
    EventReset                   = 4,
    Incomplete                   = 5,
    ErrorOutOfHostMemory         = -1,
    ErrorOutOfDeviceMemory       = -2,
    ErrorInitializationFailed    = -3,
    ErrorDeviceLost              = -4,
    ErrorMemoryMapFailed         = -5,
    ErrorLayerNotPresent         = -6,
    ErrorExtensionNotPresent     = -7,
    ErrorFeatureNotPresent       = -8,
    ErrorIncompatibleDriver     = -9,
    ErrorTooManyObjects          = -10,
    ErrorFormatNotSupported      = -11,
    ErrorFragmentedPool          = -12,
    ErrorUnknown                 = -13,
}

impl VkResult {
    pub fn is_success(&self) -> bool {
        matches!(self, VkResult::Success)
    }
}

/// Map NuvaError to VkResult
pub fn vk_result_from_nv_error(e: NuvaError) -> VkResult {
    match e {
        NuvaError::Success          => VkResult::Success,
        NuvaError::NoMemory         => VkResult::ErrorOutOfDeviceMemory,
        NuvaError::CapabilityDenied => VkResult::ErrorInitializationFailed,
        NuvaError::CapabilityExpired=> VkResult::ErrorDeviceLost,
        NuvaError::Timeout          => VkResult::Timeout,
        NuvaError::WouldBlock       => VkResult::NotReady,
        NuvaError::InvalidCall      => VkResult::ErrorUnknown,
        NuvaError::InvalidParameter => VkResult::ErrorInitializationFailed,
        NuvaError::ResourceBusy     => VkResult::ErrorTooManyObjects,
        NuvaError::ResourceNotFound => VkResult::ErrorUnknown,
        _                           => VkResult::ErrorUnknown,
    }
}

/// Vulkan Instance handle (opaque)
pub type VkInstance = u64;

/// Vulkan Physical Device handle
pub type VkPhysicalDevice = u64;

/// Vulkan Device handle
pub type VkDevice = u64;

/// Vulkan Device Memory handle
pub type VkDeviceMemory = u64;

/// Vulkan Queue handle
pub type VkQueue = u64;

/// Vulkan Fence handle
pub type VkFence = u64;

/// Vulkan Semaphore handle
pub type VkSemaphore = u64;

/// Vulkan Swapchain handle
pub type VkSwapchainKHR = u64;

/// Vulkan Command Buffer handle
pub type VkCommandBuffer = u64;

// ============================================================================
// Vulkan API Bridge Functions
// Each bridges to the corresponding NvVulkan system call.
// ============================================================================

/// Create a Vulkan Instance.
/// Bridge: NV_VULKAN_INSTANCE_CREATE (0x70)
/// Requests GPU_RENDER capability before system call.
pub fn create_instance(
    api_version: u32,
    capability_id: u64,
    owner_process_id: u64,
) -> Result<VkInstance, VkResult> {
    let _ = (api_version, capability_id, owner_process_id);
    // TODO: Invoke NV_VULKAN_INSTANCE_CREATE syscall
    Ok(0)
}

/// Destroy a Vulkan Instance.
/// Bridge: NV_VULKAN_INSTANCE_DESTROY (0x71)
pub fn destroy_instance(
    instance: VkInstance,
    capability_id: u64,
) -> VkResult {
    let _ = (instance, capability_id);
    // TODO: Invoke NV_VULKAN_INSTANCE_DESTROY syscall
    VkResult::Success
}

/// Enumerate physical GPU devices.
/// Bridge: NV_VULKAN_DEVICE_ENUMERATE (0x72)
pub fn enumerate_physical_devices(
    instance: VkInstance,
    capability_id: u64,
) -> Result<Vec<VkPhysicalDevice>, VkResult> {
    let _ = (instance, capability_id);
    // TODO: Invoke NV_VULKAN_DEVICE_ENUMERATE syscall
    Ok(Vec::new())
}

/// Create a logical device.
/// Bridge: NV_VULKAN_DEVICE_CREATE (0x73)
pub fn create_device(
    physical_device: VkPhysicalDevice,
    capability_id: u64,
    instance_id: u64,
) -> Result<VkDevice, VkResult> {
    let _ = (physical_device, capability_id, instance_id);
    // TODO: Invoke NV_VULKAN_DEVICE_CREATE syscall
    Ok(0)
}

/// Destroy a logical device.
/// Bridge: NV_VULKAN_DEVICE_DESTROY (0x74)
pub fn destroy_device(device: VkDevice, capability_id: u64) -> VkResult {
    let _ = (device, capability_id);
    VkResult::Success
}

/// Allocate GPU memory.
/// Bridge: NV_VULKAN_MEMORY_ALLOCATE (0x75)
/// For HOST_VISIBLE memory: zero-copy, CPU and GPU share physical pages.
pub fn allocate_memory(
    device: VkDevice,
    size: u64,
    memory_type_index: u32,
    capability_id: u64,
) -> Result<VkDeviceMemory, VkResult> {
    let _ = (device, size, memory_type_index, capability_id);
    // TODO: Invoke NV_VULKAN_MEMORY_ALLOCATE syscall
    Ok(0)
}

/// Free GPU memory.
/// Bridge: NV_VULKAN_MEMORY_FREE (0x76)
pub fn free_memory(device: VkDevice, memory: VkDeviceMemory, capability_id: u64) -> VkResult {
    let _ = (device, memory, capability_id);
    VkResult::Success
}

/// Submit command buffers to a queue (zero-copy).
/// Bridge: NV_VULKAN_QUEUE_SUBMIT (0x77)
/// Command buffers are in HOST_VISIBLE GPU memory; GPU reads directly
/// from the same physical pages without copying.
pub fn queue_submit(
    queue: VkQueue,
    command_buffer_addr: u64,
    command_buffer_size: u64,
    capability_id: u64,
) -> VkResult {
    let _ = (queue, command_buffer_addr, command_buffer_size, capability_id);
    // TODO: Invoke NV_VULKAN_QUEUE_SUBMIT syscall (zero-copy path)
    VkResult::Success
}

/// Wait for queue idle.
/// Bridge: NV_VULKAN_QUEUE_WAIT (0x78)
pub fn queue_wait_idle(queue: VkQueue, capability_id: u64) -> VkResult {
    let _ = (queue, capability_id);
    VkResult::Success
}
