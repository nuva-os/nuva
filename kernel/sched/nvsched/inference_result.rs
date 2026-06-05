/*
 * Nuva OS - Kernel - Sched - Nvsched - InferenceResult
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
 * Nuva OS - Kernel - NvScheduler Inference Result
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * NPU scheduling inference output structure.
 */

/// Target device type for scheduling decision
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TargetDeviceType {
    /// Big (performance) CPU cluster
    CpuBig = 0,
    /// Little (efficiency) CPU cluster
    CpuLittle = 1,
    /// GPU (RTX Spark)
    Gpu = 2,
    /// NPU (Da Vinci)
    Npu = 3,
    /// Quantum device
    Quantum = 4,
}

impl TargetDeviceType {
    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => TargetDeviceType::CpuBig,
            1 => TargetDeviceType::CpuLittle,
            2 => TargetDeviceType::Gpu,
            3 => TargetDeviceType::Npu,
            4 => TargetDeviceType::Quantum,
            _ => TargetDeviceType::CpuBig,
        }
    }
}

/// SchedInferenceResult: NPU scheduling inference output
///
/// Contains the AI model's scheduling recommendation
/// including target device, priority adjustment, confidence,
/// and migration/power efficiency hints.
#[derive(Clone, Debug)]
#[repr(C)]
pub struct SchedInferenceResult {
    /// Target device type for task placement
    pub target_device_type: TargetDeviceType,
    /// Target device ID (index within device type)
    pub target_device_id: u32,
    /// Priority boost value (-5 to +5, 0 = no change)
    pub priority_boost: i8,
    /// Confidence score (0-100, percentage)
    pub confidence: u8,
    /// Migration hint: whether task should migrate
    pub migration_hint: bool,
    /// Power efficiency score (0-100, higher = more efficient)
    pub power_efficiency_score: u8,
    /// Inference latency in microseconds
    pub inference_latency_us: u32,
}

impl SchedInferenceResult {
    /// Create a default (low-confidence) inference result
    pub const fn default_low_confidence() -> Self {
        SchedInferenceResult {
            target_device_type: TargetDeviceType::CpuBig,
            target_device_id: 0,
            priority_boost: 0,
            confidence: 0,
            migration_hint: false,
            power_efficiency_score: 50,
            inference_latency_us: 0,
        }
    }

    /// Check if confidence meets threshold (percentage)
    #[inline(always)]
    pub fn meets_confidence(&self, threshold_pct: u32) -> bool {
        (self.confidence as u32) >= threshold_pct
    }

    /// Check if inference timed out
    #[inline(always)]
    pub fn is_timeout(&self, budget_us: u32) -> bool {
        self.inference_latency_us > budget_us
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_low_confidence() {
        let r = SchedInferenceResult::default_low_confidence();
        assert_eq!(r.confidence, 0);
        assert!(!r.meets_confidence(50));
    }

    #[test]
    fn test_meets_confidence() {
        let r = SchedInferenceResult {
            target_device_type: TargetDeviceType::Npu,
            target_device_id: 0,
            priority_boost: 3,
            confidence: 75,
            migration_hint: false,
            power_efficiency_score: 80,
            inference_latency_us: 50,
        };
        assert!(r.meets_confidence(50));
        assert!(!r.meets_confidence(80));
    }

    #[test]
    fn test_is_timeout() {
        let r = SchedInferenceResult {
            target_device_type: TargetDeviceType::CpuBig,
            target_device_id: 0,
            priority_boost: 0,
            confidence: 90,
            migration_hint: false,
            power_efficiency_score: 70,
            inference_latency_us: 150,
        };
        assert!(r.is_timeout(100));
        assert!(!r.is_timeout(200));
    }
}