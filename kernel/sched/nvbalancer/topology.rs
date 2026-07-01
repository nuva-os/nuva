/*
 * Nuva OS - Kernel - Sched - Nvbalancer - Topology
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
 * Nuva OS - Kernel - NvBalancer Heterogeneous Device Topology
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Manages the heterogeneous device topology including
 * NUMA mapping, PCIe bandwidth matrix, and interconnect
 * latency matrix with generation-based hot-plug support.
 */

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

use super::device_types::{NvHeteroDeviceType, HeteroDeviceState, DeviceCapabilityFlags};
use super::MAX_HETERO_DEVICES;
use alloc::vec::Vec;

/// Maximum NUMA nodes
pub const MAX_NUMA_NODES: usize = 8;

/// Maximum PCIe bandwidth matrix dimension
pub const MAX_TOPO_DIM: usize = MAX_HETERO_DEVICES;

/// HeteroDeviceNode: a node in the device topology
#[derive(Clone, Debug)]
pub struct HeteroDeviceNode {
    /// Device ID
    pub device_id: u32,
    /// Device type
    pub device_type: NvHeteroDeviceType,
    /// NUMA node affinity
    pub numa_node: u32,
    /// Compute capability score (0-1000)
    pub compute_score: u32,
    /// Memory bandwidth (MB/s)
    pub memory_bandwidth_mbps: u32,
    /// Current load (0-100 percentage)
    pub load: u32,
    /// Current power consumption (mW)
    pub power_mw: u32,
    /// Capability flags
    pub capability_flags: DeviceCapabilityFlags,
    /// Device state
    pub state: HeteroDeviceState,
}

impl HeteroDeviceNode {
    /// Create a new device node
    pub const fn new(
        device_id: u32,
        device_type: NvHeteroDeviceType,
        numa_node: u32,
    ) -> Self {
        HeteroDeviceNode {
            device_id,
            device_type,
            numa_node,
            compute_score: device_type.default_compute_score(),
            memory_bandwidth_mbps: device_type.default_memory_bandwidth(),
            load: 0,
            power_mw: 0,
            capability_flags: DeviceCapabilityFlags::empty(),
            state: HeteroDeviceState::Offline,
        }
    }

    /// Check if device is usable for task assignment
    #[inline(always)]
    pub fn is_usable(&self) -> bool {
        self.state.is_usable()
    }

    /// Get effective compute score considering load
    pub fn effective_compute_score(&self) -> u32 {
        if self.load >= 100 {
            return 0;
        }
        self.compute_score * (100 - self.load) / 100
    }
}

/// HeteroDeviceTopology: complete device topology
///
/// Maintains device registry, NUMA mapping, PCIe bandwidth
/// matrix, and interconnect latency matrix. Uses generation
/// counter to track topology changes from hot-plug events.
pub struct HeteroDeviceTopology {
    /// Device registry (fixed-size, slot-based)
    devices: [Option<HeteroDeviceNode>; MAX_HETERO_DEVICES],
    /// Number of registered devices
    num_devices: AtomicU32,
    /// NUMA node mapping (device_index -> numa_node)
    numa_map: [u32; MAX_HETERO_DEVICES],
    /// PCIe bandwidth matrix (MB/s, symmetric)
    pcie_bw_matrix: [[u32; MAX_TOPO_DIM]; MAX_TOPO_DIM],
    /// Interconnect latency matrix (nanoseconds)
    latency_matrix: [[u32; MAX_TOPO_DIM]; MAX_TOPO_DIM],
    /// Topology generation counter (incremented on changes)
    generation: AtomicU64,
}

impl HeteroDeviceTopology {
    /// Create a new empty topology
    pub const fn new() -> Self {
        HeteroDeviceTopology {
            devices: [None; MAX_HETERO_DEVICES],
            num_devices: AtomicU32::new(0),
            numa_map: [0; MAX_HETERO_DEVICES],
            pcie_bw_matrix: [[0; MAX_TOPO_DIM]; MAX_TOPO_DIM],
            latency_matrix: [[0; MAX_TOPO_DIM]; MAX_TOPO_DIM],
            generation: AtomicU64::new(0),
        }
    }

    /// Register a device in the topology
    ///
    /// @param device: Device node to register
    /// @return: Ok(device_index) or Err if full
    pub fn register_device(&mut self, device: HeteroDeviceNode) -> Result<usize, ()> {
        for i in 0..MAX_HETERO_DEVICES {
            if self.devices[i].is_none() {
                self.numa_map[i] = device.numa_node;
                self.devices[i] = Some(device);
                self.num_devices.fetch_add(1, Ordering::Relaxed);
                self.generation.fetch_add(1, Ordering::Release);
                return Ok(i);
            }
        }
        Err(())
    }

    /// Unregister a device from the topology
    ///
    /// @param device_id: Device ID to unregister
    /// @return: Ok(()) or Err if not found
    pub fn unregister_device(&mut self, device_id: u32) -> Result<(), ()> {
        for i in 0..MAX_HETERO_DEVICES {
            if let Some(ref dev) = self.devices[i] {
                if dev.device_id == device_id {
                    self.devices[i] = None;
                    self.num_devices.fetch_sub(1, Ordering::Relaxed);
                    self.generation.fetch_add(1, Ordering::Release);
                    return Ok(());
                }
            }
        }
        Err(())
    }

    /// Get device by index
    pub fn get_device(&self, index: usize) -> Option<&HeteroDeviceNode> {
        if index < MAX_HETERO_DEVICES {
            self.devices[index].as_ref()
        } else {
            None
        }
    }

    /// Get device by ID
    pub fn find_device_by_id(&self, device_id: u32) -> Option<usize> {
        for i in 0..MAX_HETERO_DEVICES {
            if let Some(ref dev) = self.devices[i] {
                if dev.device_id == device_id {
                    return Some(i);
                }
            }
        }
        None
    }

    /// Get number of registered devices
    pub fn num_devices(&self) -> u32 {
        self.num_devices.load(Ordering::Acquire)
    }

    /// Get current topology generation
    pub fn generation(&self) -> u64 {
        self.generation.load(Ordering::Acquire)
    }

    /// Set PCIe bandwidth between two devices
    pub fn set_pcie_bandwidth(&mut self, i: usize, j: usize, bw_mbps: u32) {
        if i < MAX_TOPO_DIM && j < MAX_TOPO_DIM {
            self.pcie_bw_matrix[i][j] = bw_mbps;
            self.pcie_bw_matrix[j][i] = bw_mbps;
        }
    }

    /// Get PCIe bandwidth between two devices
    pub fn pcie_bandwidth(&self, i: usize, j: usize) -> u32 {
        if i < MAX_TOPO_DIM && j < MAX_TOPO_DIM {
            self.pcie_bw_matrix[i][j]
        } else {
            0
        }
    }

    /// Set interconnect latency between two devices
    pub fn set_latency(&mut self, i: usize, j: usize, latency_ns: u32) {
        if i < MAX_TOPO_DIM && j < MAX_TOPO_DIM {
            self.latency_matrix[i][j] = latency_ns;
            self.latency_matrix[j][i] = latency_ns;
        }
    }

    /// Get interconnect latency between two devices
    pub fn latency(&self, i: usize, j: usize) -> u32 {
        if i < MAX_TOPO_DIM && j < MAX_TOPO_DIM {
            self.latency_matrix[i][j]
        } else {
            u32::MAX
        }
    }

    /// Find devices of a specific type
    pub fn find_devices_by_type(&self, device_type: NvHeteroDeviceType) -> alloc::vec::Vec<usize> {
        let mut result = alloc::vec::Vec::new();
        for i in 0..MAX_HETERO_DEVICES {
            if let Some(ref dev) = self.devices[i] {
                if dev.device_type == device_type && dev.is_usable() {
                    result.push(i);
                }
            }
        }
        result
    }

    /// Compute max load across all usable devices
    pub fn max_load(&self) -> u32 {
        let mut max = 0u32;
        for i in 0..MAX_HETERO_DEVICES {
            if let Some(ref dev) = self.devices[i] {
                if dev.is_usable() && dev.load > max {
                    max = dev.load;
                }
            }
        }
        max
    }

    /// Compute min load across all usable devices
    pub fn min_load(&self) -> u32 {
        let mut min = 100u32;
        let mut found = false;
        for i in 0..MAX_HETERO_DEVICES {
            if let Some(ref dev) = self.devices[i] {
                if dev.is_usable() {
                    found = true;
                    if dev.load < min {
                        min = dev.load;
                    }
                }
            }
        }
        if found { min } else { 0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_device() {
        let mut topo = HeteroDeviceTopology::new();
        let dev = HeteroDeviceNode::new(0, NvHeteroDeviceType::GpuRtxSpark, 0);
        let idx = topo.register_device(dev);
        assert!(idx.is_ok());
        assert_eq!(topo.num_devices(), 1);
        assert_eq!(topo.generation(), 1);
    }

    #[test]
    fn test_unregister_device() {
        let mut topo = HeteroDeviceTopology::new();
        let dev = HeteroDeviceNode::new(42, NvHeteroDeviceType::NpuDavinci, 0);
        let _ = topo.register_device(dev);
        let result = topo.unregister_device(42);
        assert!(result.is_ok());
        assert_eq!(topo.num_devices(), 0);
    }

    #[test]
    fn test_effective_compute_score() {
        let mut dev = HeteroDeviceNode::new(0, NvHeteroDeviceType::GpuRtxSpark, 0);
        dev.load = 50;
        assert_eq!(dev.effective_compute_score(), 500);
    }

    #[test]
    fn test_pcie_bandwidth_symmetric() {
        let mut topo = HeteroDeviceTopology::new();
        topo.set_pcie_bandwidth(0, 1, 256000);
        assert_eq!(topo.pcie_bandwidth(0, 1), 256000);
        assert_eq!(topo.pcie_bandwidth(1, 0), 256000);
    }
}