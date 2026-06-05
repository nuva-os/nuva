/*
 * Nuva OS - Kernel - Sched - Nvbalancer - DeviceTypes
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
 * Nuva OS - Kernel - NvBalancer Device Types
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Heterogeneous device type and state definitions.
 */

/// Heterogeneous device type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum NvHeteroDeviceType {
    /// NVIDIA RTX Spark GPU
    GpuRtxSpark = 0,
    /// Huawei Da Vinci NPU
    NpuDavinci = 1,
    /// CPU cluster (big or little)
    CpuCluster = 2,
    /// Quantum processing device
    QuantumDevice = 3,
}

impl NvHeteroDeviceType {
    /// Convert from u8
    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => NvHeteroDeviceType::GpuRtxSpark,
            1 => NvHeteroDeviceType::NpuDavinci,
            2 => NvHeteroDeviceType::CpuCluster,
            3 => NvHeteroDeviceType::QuantumDevice,
            _ => NvHeteroDeviceType::CpuCluster,
        }
    }

    /// Get default compute score for device type
    pub fn default_compute_score(&self) -> u32 {
        match self {
            NvHeteroDeviceType::GpuRtxSpark => 1000,
            NvHeteroDeviceType::NpuDavinci => 800,
            NvHeteroDeviceType::CpuCluster => 100,
            NvHeteroDeviceType::QuantumDevice => 500,
        }
    }

    /// Get default memory bandwidth (MB/s)
    pub fn default_memory_bandwidth(&self) -> u32 {
        match self {
            NvHeteroDeviceType::GpuRtxSpark => 900_000,
            NvHeteroDeviceType::NpuDavinci => 400_000,
            NvHeteroDeviceType::CpuCluster => 50_000,
            NvHeteroDeviceType::QuantumDevice => 10_000,
        }
    }

    /// Check if device type supports AI inference
    pub fn supports_ai_inference(&self) -> bool {
        matches!(self, NvHeteroDeviceType::GpuRtxSpark | NvHeteroDeviceType::NpuDavinci)
    }
}

/// Heterogeneous device state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum HeteroDeviceState {
    /// Device is active and operational
    Active = 0,
    /// Device is degraded (partial functionality)
    Degraded = 1,
    /// Device is offline
    Offline = 2,
    /// Device is pending hot-plug
    HotplugPending = 3,
}

impl HeteroDeviceState {
    /// Convert from u8
    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => HeteroDeviceState::Active,
            1 => HeteroDeviceState::Degraded,
            2 => HeteroDeviceState::Offline,
            3 => HeteroDeviceState::HotplugPending,
            _ => HeteroDeviceState::Offline,
        }
    }

    /// Check if device is usable for task assignment
    pub fn is_usable(&self) -> bool {
        matches!(self, HeteroDeviceState::Active | HeteroDeviceState::Degraded)
    }
}

bitflags::bitflags! {
    /// Device capability flags
    #[derive(Debug, Clone, Copy)]
    pub struct DeviceCapabilityFlags: u32 {
        /// Supports compute workloads
        const COMPUTE = 0x01;
        /// Supports AI inference
        const AI_INFERENCE = 0x02;
        /// Supports AI training
        const AI_TRAINING = 0x04;
        /// Supports memory-intensive workloads
        const MEMORY_INTENSIVE = 0x08;
        /// Supports low-latency operations
        const LOW_LATENCY = 0x10;
        /// Supports power management
        const POWER_MANAGED = 0x20;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_type_compute_score() {
        assert!(NvHeteroDeviceType::GpuRtxSpark.default_compute_score() > NvHeteroDeviceType::CpuCluster.default_compute_score());
    }

    #[test]
    fn test_device_state_usable() {
        assert!(HeteroDeviceState::Active.is_usable());
        assert!(HeteroDeviceState::Degraded.is_usable());
        assert!(!HeteroDeviceState::Offline.is_usable());
    }

    #[test]
    fn test_ai_inference_support() {
        assert!(NvHeteroDeviceType::GpuRtxSpark.supports_ai_inference());
        assert!(NvHeteroDeviceType::NpuDavinci.supports_ai_inference());
        assert!(!NvHeteroDeviceType::CpuCluster.supports_ai_inference());
    }
}