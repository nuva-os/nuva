/*
 * Nuva OS - Kernel - Sched - Nvbalancer - Hotplug
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
 * Nuva OS - Kernel - NvBalancer Hot-Plug Handler
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Handles device hot-add and hot-remove events,
 * updating topology and triggering task migration
 * for affected running tasks.
 */

use core::sync::atomic::{AtomicU64, Ordering};

use super::topology::{HeteroDeviceTopology, HeteroDeviceNode};
use super::device_types::HeteroDeviceState;

/// Hot-plug event type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum HotplugEvent {
    /// Device added
    Add = 0,
    /// Device removed
    Remove = 1,
}

/// HotplugHandler: handles device hot-plug events
///
/// On device add:
/// - Register device in topology
/// - Increment generation counter
/// - New tasks use updated topology
///
/// On device remove:
/// - Save checkpoints for running tasks
/// - Migrate tasks to backup devices
/// - Unregister device from topology
/// - Running tasks are not disrupted
pub struct HotplugHandler {
    /// Total hot-add events
    hot_add_count: AtomicU64,
    /// Total hot-remove events
    hot_remove_count: AtomicU64,
    /// Total tasks migrated due to hot-plug
    tasks_migrated: AtomicU64,
}

impl HotplugHandler {
    /// Create a new hot-plug handler
    pub const fn new() -> Self {
        HotplugHandler {
            hot_add_count: AtomicU64::new(0),
            hot_remove_count: AtomicU64::new(0),
            tasks_migrated: AtomicU64::new(0),
        }
    }

    /// Handle device hot-add
    ///
    /// @param topology: Device topology to update
    /// @param device: New device to add
    /// @return: Ok(device_index) or Err if topology full
    pub fn handle_add(&self, topology: &mut HeteroDeviceTopology, mut device: HeteroDeviceNode) -> Result<usize, ()> {
        device.state = HeteroDeviceState::Active;
        let result = topology.register_device(device);
        if result.is_ok() {
            self.hot_add_count.fetch_add(1, Ordering::Relaxed);
        }
        result
    }

    /// Handle device hot-remove
    ///
    /// @param topology: Device topology to update
    /// @param device_id: Device ID to remove
    /// @return: Ok(num_tasks_migrated) or Err if device not found
    pub fn handle_remove(&self, topology: &mut HeteroDeviceTopology, device_id: u32) -> Result<u32, ()> {
        // TODO: In full implementation:
        // 1. Find all tasks on this device
        // 2. Save checkpoints for each task
        // 3. Migrate tasks to backup devices
        // 4. Unregister device

        let result = topology.unregister_device(device_id);
        if result.is_ok() {
            self.hot_remove_count.fetch_add(1, Ordering::Relaxed);
            Ok(0)
        } else {
            Err(())
        }
    }

    /// Get hot-plug statistics
    pub fn stats(&self) -> (u64, u64, u64) {
        (
            self.hot_add_count.load(Ordering::Acquire),
            self.hot_remove_count.load(Ordering::Acquire),
            self.tasks_migrated.load(Ordering::Acquire),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::device_types::NvHeteroDeviceType;

    #[test]
    fn test_hot_add() {
        let handler = HotplugHandler::new();
        let mut topo = HeteroDeviceTopology::new();
        let dev = HeteroDeviceNode::new(10, NvHeteroDeviceType::GpuRtxSpark, 0);
        let result = handler.handle_add(&mut topo, dev);
        assert!(result.is_ok());
        assert_eq!(topo.num_devices(), 1);
    }

    #[test]
    fn test_hot_remove() {
        let handler = HotplugHandler::new();
        let mut topo = HeteroDeviceTopology::new();
        let dev = HeteroDeviceNode::new(20, NvHeteroDeviceType::NpuDavinci, 0);
        let _ = handler.handle_add(&mut topo, dev);
        let result = handler.handle_remove(&mut topo, 20);
        assert!(result.is_ok());
        assert_eq!(topo.num_devices(), 0);
    }

    #[test]
    fn test_hot_remove_not_found() {
        let handler = HotplugHandler::new();
        let mut topo = HeteroDeviceTopology::new();
        let result = handler.handle_remove(&mut topo, 999);
        assert!(result.is_err());
    }
}