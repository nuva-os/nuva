/*
 * Nuva OS - Kernel - Sched - Nvsched - NpuInferenceEngine
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
 * Nuva OS - Kernel - NvScheduler NPU Inference Engine
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * NPU-based scheduling inference engine that converts
 * feature vectors into scheduling decisions via Da Vinci NPU.
 */

use core::sync::atomic::{AtomicU32, AtomicU64, AtomicBool, Ordering};

use super::feature_vector::SchedFeatureVector;
use super::inference_result::{SchedInferenceResult, TargetDeviceType};

/// Default scheduling model ID
pub const DEFAULT_SCHED_MODEL_ID: u32 = 1;

/// Default scheduling model version
pub const DEFAULT_SCHED_MODEL_VERSION: u32 = 1;

/// Maximum inference cache entries
pub const MAX_INFERENCE_CACHE: usize = 8;

/// NpuInferenceEngine: NPU-based scheduling inference
///
/// Manages the scheduling model on Da Vinci NPU,
/// builds feature vectors, submits inference requests,
/// and caches recent results for fallback.
pub struct NpuInferenceEngine {
    /// Scheduling model ID on NPU
    model_id: AtomicU32,
    /// Scheduling model version
    model_version: AtomicU32,
    /// NPU affinity mask (which NPU cores to use)
    npu_affinity_mask: AtomicU32,
    /// Whether NPU is available for inference
    npu_available: AtomicBool,
    /// Total inference requests submitted
    total_inferences: AtomicU64,
    /// Successful inferences
    successful_inferences: AtomicU64,
    /// Timed-out inferences
    timed_out_inferences: AtomicU64,
    /// Last valid inference result (for fallback)
    last_valid_result: core::cell::UnsafeCell<SchedInferenceResult>,
    /// Last valid result valid flag
    last_valid_available: AtomicBool,
}

impl NpuInferenceEngine {
    /// Create a new NPU inference engine
    pub const fn new() -> Self {
        NpuInferenceEngine {
            model_id: AtomicU32::new(DEFAULT_SCHED_MODEL_ID),
            model_version: AtomicU32::new(DEFAULT_SCHED_MODEL_VERSION),
            npu_affinity_mask: AtomicU32::new(0x1),
            npu_available: AtomicBool::new(false),
            total_inferences: AtomicU64::new(0),
            successful_inferences: AtomicU64::new(0),
            timed_out_inferences: AtomicU64::new(0),
            last_valid_result: core::cell::UnsafeCell::new(
                SchedInferenceResult::default_low_confidence()
            ),
            last_valid_available: AtomicBool::new(false),
        }
    }

    /// Initialize NPU inference engine
    pub fn init(&self, npu_available: bool) {
        self.npu_available.store(npu_available, Ordering::Release);
    }

    /// Check if NPU is available
    #[inline(always)]
    pub fn is_available(&self) -> bool {
        self.npu_available.load(Ordering::Acquire)
    }

    /// Set NPU availability
    pub fn set_npu_available(&self, available: bool) {
        self.npu_available.store(available, Ordering::Release);
    }

    /// Get model ID
    #[inline(always)]
    pub fn model_id(&self) -> u32 {
        self.model_id.load(Ordering::Acquire)
    }

    /// Set model ID (for model hot-swap)
    pub fn set_model_id(&self, id: u32) {
        self.model_id.store(id, Ordering::Release);
    }

    /// Get model version
    #[inline(always)]
    pub fn model_version(&self) -> u32 {
        self.model_version.load(Ordering::Acquire)
    }

    /// Set model version
    pub fn set_model_version(&self, version: u32) {
        self.model_version.store(version, Ordering::Release);
    }

    /// Submit scheduling inference request
    ///
    /// Converts feature vector to NPU input, submits inference,
    /// and returns scheduling decision. Falls back to heuristic
    /// if NPU is unavailable or inference times out.
    ///
    /// @param feature_vec: Current scheduling feature vector
    /// @param budget_us: Maximum inference time budget in microseconds
    /// @return: Scheduling inference result
    pub fn infer(&self, feature_vec: &SchedFeatureVector, budget_us: u32) -> SchedInferenceResult {
        self.total_inferences.fetch_add(1, Ordering::Relaxed);

        if !self.npu_available.load(Ordering::Acquire) {
            return self.heuristic_fallback(feature_vec);
        }

        let result = self.submit_npu_inference(feature_vec, budget_us);

        if result.confidence > 0 && !result.is_timeout(budget_us) {
            self.successful_inferences.fetch_add(1, Ordering::Relaxed);
            self.cache_last_valid(&result);
            result
        } else {
            if result.is_timeout(budget_us) {
                self.timed_out_inferences.fetch_add(1, Ordering::Relaxed);
            }
            self.get_last_valid_or_heuristic(feature_vec)
        }
    }

    /// Get last valid cached result
    pub fn get_last_valid(&self) -> Option<SchedInferenceResult> {
        if self.last_valid_available.load(Ordering::Acquire) {
            unsafe { Some((*self.last_valid_result.get()).clone()) }
        } else {
            None
        }
    }

    /// Get inference statistics
    pub fn stats(&self) -> (u64, u64, u64) {
        (
            self.total_inferences.load(Ordering::Acquire),
            self.successful_inferences.load(Ordering::Acquire),
            self.timed_out_inferences.load(Ordering::Acquire),
        )
    }

    /// Submit inference to NPU (placeholder for HAL integration)
    fn submit_npu_inference(&self, feature_vec: &SchedFeatureVector, _budget_us: u32) -> SchedInferenceResult {
        // TODO: Integrate with hal::npu::davinci for actual NPU inference
        // For now, use heuristic-based inference
        self.heuristic_fallback(feature_vec)
    }

    /// Heuristic-based fallback when NPU is unavailable
    fn heuristic_fallback(&self, fv: &SchedFeatureVector) -> SchedInferenceResult {
        use super::feature_vector::FP_SCALE;

        let compute_ratio = fv.task_compute_ratio;
        let npu_access = fv.task_has_npu_access;
        let npu_util = fv.npu_util;
        let gpu_util = fv.gpu_util;
        let thermal = fv.thermal_pressure;

        let (target_type, priority_boost, confidence) = if npu_access == FP_SCALE && npu_util < (FP_SCALE * 80 / 100) {
            (TargetDeviceType::Npu, 4i8, 85u8)
        } else if compute_ratio > (FP_SCALE * 70 / 100) && gpu_util < (FP_SCALE * 80 / 100) {
            (TargetDeviceType::Gpu, 3i8, 80u8)
        } else if compute_ratio > (FP_SCALE * 50 / 100) {
            if thermal > (FP_SCALE * 70 / 100) {
                (TargetDeviceType::CpuLittle, 1i8, 70u8)
            } else {
                (TargetDeviceType::CpuBig, 2i8, 75u8)
            }
        } else {
            (TargetDeviceType::CpuLittle, 0i8, 65u8)
        };

        SchedInferenceResult {
            target_device_type: target_type,
            target_device_id: 0,
            priority_boost,
            confidence,
            migration_hint: false,
            power_efficiency_score: if thermal > (FP_SCALE * 50 / 100) { 40 } else { 70 },
            inference_latency_us: 0,
        }
    }

    /// Cache last valid result
    fn cache_last_valid(&self, result: &SchedInferenceResult) {
        unsafe {
            *self.last_valid_result.get() = result.clone();
        }
        self.last_valid_available.store(true, Ordering::Release);
    }

    /// Get last valid result or fall back to heuristic
    fn get_last_valid_or_heuristic(&self, fv: &SchedFeatureVector) -> SchedInferenceResult {
        if let Some(last) = self.get_last_valid() {
            last
        } else {
            self.heuristic_fallback(fv)
        }
    }
}

unsafe impl Sync for NpuInferenceEngine {}