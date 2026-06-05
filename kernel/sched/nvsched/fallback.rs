/*
 * Nuva OS - Kernel - Sched - Nvsched - Fallback
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
 * Nuva OS - Kernel - NvScheduler Fallback Scheduler
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Three-tier fallback mechanism:
 * Level 1: NPU unavailable -> Declarative policy
 * Level 2: Inference timeout -> Last valid decision
 * Level 3: Low confidence -> CFS+RT traditional
 */

use core::sync::atomic::{AtomicU64, Ordering};

use super::sched_class::NvAiSchedClass;
use super::inference_result::{SchedInferenceResult, TargetDeviceType};
use super::feature_vector::SchedFeatureVector;

/// Fallback event types for logging
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FallbackEvent {
    /// NPU not available
    NpuUnavailable = 0,
    /// Inference timeout
    InferenceTimeout = 1,
    /// Low confidence score
    LowConfidence = 2,
    /// Model version mismatch
    ModelMismatch = 3,
}

/// FallbackScheduler: three-tier fallback mechanism
///
/// Provides graceful degradation when AI inference
/// is unavailable or unreliable:
/// 1. NPU down -> declarative policy decisions
/// 2. Inference timeout -> reuse last valid decision
/// 3. Low confidence -> traditional CFS+RT scheduling
pub struct FallbackScheduler {
    /// NPU unavailable fallback count
    npu_unavailable_count: AtomicU64,
    /// Inference timeout fallback count
    inference_timeout_count: AtomicU64,
    /// Low confidence fallback count
    low_confidence_count: AtomicU64,
    /// Model mismatch fallback count
    model_mismatch_count: AtomicU64,
    /// Total fallback events
    total_fallbacks: AtomicU64,
}

impl FallbackScheduler {
    /// Create a new fallback scheduler
    pub const fn new() -> Self {
        FallbackScheduler {
            npu_unavailable_count: AtomicU64::new(0),
            inference_timeout_count: AtomicU64::new(0),
            low_confidence_count: AtomicU64::new(0),
            model_mismatch_count: AtomicU64::new(0),
            total_fallbacks: AtomicU64::new(0),
        }
    }

    /// Handle NPU unavailable fallback
    ///
    /// Returns a declarative policy-based decision.
    /// Uses task scheduling class to determine placement.
    pub fn fallback_npu_unavailable(&self, sched_class: NvAiSchedClass) -> SchedInferenceResult {
        self.npu_unavailable_count.fetch_add(1, Ordering::Relaxed);
        self.total_fallbacks.fetch_add(1, Ordering::Relaxed);

        let (target, boost, confidence) = match sched_class {
            NvAiSchedClass::AiRealtime => (TargetDeviceType::CpuBig, 5i8, 60u8),
            NvAiSchedClass::AiNormal => (TargetDeviceType::CpuBig, 2i8, 55u8),
            NvAiSchedClass::AiBatch => (TargetDeviceType::CpuLittle, 0i8, 50u8),
            NvAiSchedClass::AiIdle => (TargetDeviceType::CpuLittle, 0i8, 45u8),
        };

        SchedInferenceResult {
            target_device_type: target,
            target_device_id: 0,
            priority_boost: boost,
            confidence,
            migration_hint: false,
            power_efficiency_score: 50,
            inference_latency_us: 0,
        }
    }

    /// Handle inference timeout fallback
    ///
    /// Returns last valid decision if available,
    /// otherwise falls back to heuristic.
    pub fn fallback_inference_timeout(
        &self,
        last_valid: Option<&SchedInferenceResult>,
        feature_vec: &SchedFeatureVector,
    ) -> SchedInferenceResult {
        self.inference_timeout_count.fetch_add(1, Ordering::Relaxed);
        self.total_fallbacks.fetch_add(1, Ordering::Relaxed);

        if let Some(last) = last_valid {
            last.clone()
        } else {
            self.heuristic_decision(feature_vec)
        }
    }

    /// Handle low confidence fallback
    ///
    /// Falls back to CFS+RT traditional scheduling.
    pub fn fallback_low_confidence(&self, feature_vec: &SchedFeatureVector) -> SchedInferenceResult {
        self.low_confidence_count.fetch_add(1, Ordering::Relaxed);
        self.total_fallbacks.fetch_add(1, Ordering::Relaxed);

        self.heuristic_decision(feature_vec)
    }

    /// Handle model version mismatch fallback
    pub fn fallback_model_mismatch(&self, feature_vec: &SchedFeatureVector) -> SchedInferenceResult {
        self.model_mismatch_count.fetch_add(1, Ordering::Relaxed);
        self.total_fallbacks.fetch_add(1, Ordering::Relaxed);

        self.heuristic_decision(feature_vec)
    }

    /// Record a fallback event
    pub fn record_event(&self, event: FallbackEvent) {
        self.total_fallbacks.fetch_add(1, Ordering::Relaxed);
        match event {
            FallbackEvent::NpuUnavailable => self.npu_unavailable_count.fetch_add(1, Ordering::Relaxed),
            FallbackEvent::InferenceTimeout => self.inference_timeout_count.fetch_add(1, Ordering::Relaxed),
            FallbackEvent::LowConfidence => self.low_confidence_count.fetch_add(1, Ordering::Relaxed),
            FallbackEvent::ModelMismatch => self.model_mismatch_count.fetch_add(1, Ordering::Relaxed),
        };
    }

    /// Get fallback statistics
    pub fn stats(&self) -> (u64, u64, u64, u64, u64) {
        (
            self.npu_unavailable_count.load(Ordering::Acquire),
            self.inference_timeout_count.load(Ordering::Acquire),
            self.low_confidence_count.load(Ordering::Acquire),
            self.model_mismatch_count.load(Ordering::Acquire),
            self.total_fallbacks.load(Ordering::Acquire),
        )
    }

    /// Heuristic-based decision for traditional fallback
    fn heuristic_decision(&self, fv: &SchedFeatureVector) -> SchedInferenceResult {
        use super::feature_vector::FP_SCALE;

        let compute = fv.task_compute_ratio;
        let thermal = fv.thermal_pressure;

        let target = if compute > (FP_SCALE * 60 / 100) && thermal < (FP_SCALE * 70 / 100) {
            TargetDeviceType::CpuBig
        } else {
            TargetDeviceType::CpuLittle
        };

        SchedInferenceResult {
            target_device_type: target,
            target_device_id: 0,
            priority_boost: 0,
            confidence: 40,
            migration_hint: false,
            power_efficiency_score: 60,
            inference_latency_us: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fallback_npu_unavailable() {
        let fb = FallbackScheduler::new();
        let result = fb.fallback_npu_unavailable(NvAiSchedClass::AiRealtime);
        assert_eq!(result.target_device_type, TargetDeviceType::CpuBig);
        assert_eq!(result.priority_boost, 5);

        let (_, _, _, _, total) = fb.stats();
        assert_eq!(total, 1);
    }

    #[test]
    fn test_fallback_inference_timeout_with_last_valid() {
        let fb = FallbackScheduler::new();
        let last = SchedInferenceResult {
            target_device_type: TargetDeviceType::Npu,
            target_device_id: 0,
            priority_boost: 3,
            confidence: 80,
            migration_hint: false,
            power_efficiency_score: 70,
            inference_latency_us: 50,
        };
        let fv = SchedFeatureVector::zero();
        let result = fb.fallback_inference_timeout(Some(&last), &fv);
        assert_eq!(result.target_device_type, TargetDeviceType::Npu);
    }

    #[test]
    fn test_fallback_low_confidence() {
        let fb = FallbackScheduler::new();
        let fv = SchedFeatureVector::zero();
        let result = fb.fallback_low_confidence(&fv);
        assert!(result.confidence < 50);
    }

    #[test]
    fn test_record_event() {
        let fb = FallbackScheduler::new();
        fb.record_event(FallbackEvent::ModelMismatch);
        let (_, _, _, model, total) = fb.stats();
        assert_eq!(model, 1);
        assert_eq!(total, 1);
    }
}