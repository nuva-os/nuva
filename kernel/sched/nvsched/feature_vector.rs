/*
 * Nuva OS - Kernel - Sched - Nvsched - FeatureVector
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
 * Nuva OS - Kernel - NvScheduler Feature Vector
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * 12-dimensional scheduling feature vector for NPU inference.
 * All values are fixed-point (Q16.16) for NPU compatibility.
 */

/// Fixed-point scaling factor (Q16.16)
pub const FP_SCALE: u32 = 65536;

/// SchedFeatureVector: 12-dimensional scheduling feature vector
///
/// Encodes the current system state as input to the NPU
/// scheduling inference model. All fields are fixed-point
/// Q16.16 format for deterministic NPU processing.
#[derive(Clone, Debug)]
#[repr(C, align(64))]
pub struct SchedFeatureVector {
    /// Task compute ratio (0.0-1.0, Q16.16)
    pub task_compute_ratio: u32,
    /// Task memory usage in MB (Q16.16)
    pub task_memory_mb: u32,
    /// Task has NPU access (0 or FP_SCALE)
    pub task_has_npu_access: u32,
    /// Task scheduling class (0-4, scaled by FP_SCALE/4)
    pub task_sched_class: u32,
    /// Big core utilization (0.0-1.0, Q16.16)
    pub cpu_util_big: u32,
    /// Little core utilization (0.0-1.0, Q16.16)
    pub cpu_util_little: u32,
    /// GPU utilization (0.0-1.0, Q16.16)
    pub gpu_util: u32,
    /// NPU utilization (0.0-1.0, Q16.16)
    pub npu_util: u32,
    /// Device load variance (0.0-1.0, Q16.16)
    pub device_load_variance: u32,
    /// Power budget remaining ratio (0.0-1.0, Q16.16)
    pub power_budget_remaining: u32,
    /// Thermal pressure (0.0-1.0, Q16.16)
    pub thermal_pressure: u32,
    /// Number of runnable tasks (Q16.16)
    pub num_runnable: u32,
}

impl SchedFeatureVector {
    /// Create a zero-initialized feature vector
    pub const fn zero() -> Self {
        SchedFeatureVector {
            task_compute_ratio: 0,
            task_memory_mb: 0,
            task_has_npu_access: 0,
            task_sched_class: 0,
            cpu_util_big: 0,
            cpu_util_little: 0,
            gpu_util: 0,
            npu_util: 0,
            device_load_variance: 0,
            power_budget_remaining: FP_SCALE,
            thermal_pressure: 0,
            num_runnable: 0,
        }
    }

    /// Convert a float ratio (0.0-1.0) to Q16.16 fixed-point
    #[inline(always)]
    pub fn ratio_to_fp(ratio: f32) -> u32 {
        if ratio <= 0.0 {
            0
        } else if ratio >= 1.0 {
            FP_SCALE
        } else {
            (ratio * FP_SCALE as f32) as u32
        }
    }

    /// Convert Q16.16 fixed-point to float ratio
    #[inline(always)]
    pub fn fp_to_ratio(fp: u32) -> f32 {
        fp as f32 / FP_SCALE as f32
    }

    /// Convert an integer value to Q16.16 fixed-point
    #[inline(always)]
    pub fn int_to_fp(val: u32) -> u32 {
        val * FP_SCALE
    }

    /// Convert Q16.16 fixed-point to integer (truncated)
    #[inline(always)]
    pub fn fp_to_int(fp: u32) -> u32 {
        fp / FP_SCALE
    }

    /// Build feature vector from raw scheduling state
    pub fn build(
        compute_ratio: f32,
        memory_mb: u32,
        has_npu_access: bool,
        sched_class: u8,
        cpu_util_big: f32,
        cpu_util_little: f32,
        gpu_util: f32,
        npu_util: f32,
        device_load_variance: f32,
        power_budget_remaining: f32,
        thermal_pressure: f32,
        num_runnable: u32,
    ) -> Self {
        SchedFeatureVector {
            task_compute_ratio: Self::ratio_to_fp(compute_ratio),
            task_memory_mb: Self::int_to_fp(memory_mb),
            task_has_npu_access: if has_npu_access { FP_SCALE } else { 0 },
            task_sched_class: (sched_class as u32) * (FP_SCALE / 4),
            cpu_util_big: Self::ratio_to_fp(cpu_util_big),
            cpu_util_little: Self::ratio_to_fp(cpu_util_little),
            gpu_util: Self::ratio_to_fp(gpu_util),
            npu_util: Self::ratio_to_fp(npu_util),
            device_load_variance: Self::ratio_to_fp(device_load_variance),
            power_budget_remaining: Self::ratio_to_fp(power_budget_remaining),
            thermal_pressure: Self::ratio_to_fp(thermal_pressure),
            num_runnable: Self::int_to_fp(num_runnable),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_vector_zero() {
        let fv = SchedFeatureVector::zero();
        assert_eq!(fv.task_compute_ratio, 0);
        assert_eq!(fv.power_budget_remaining, FP_SCALE);
    }

    #[test]
    fn test_ratio_to_fp() {
        assert_eq!(SchedFeatureVector::ratio_to_fp(0.0), 0);
        assert_eq!(SchedFeatureVector::ratio_to_fp(1.0), FP_SCALE);
        let half = SchedFeatureVector::ratio_to_fp(0.5);
        assert!((half as f32 - FP_SCALE as f32 / 2.0).abs() < 2.0);
    }

    #[test]
    fn test_fp_to_ratio() {
        let r = SchedFeatureVector::fp_to_ratio(FP_SCALE / 2);
        assert!((r - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_build() {
        let fv = SchedFeatureVector::build(
            0.8, 256, true, 0,
            0.6, 0.3, 0.5, 0.7,
            0.2, 0.8, 0.1, 10,
        );
        assert!(fv.task_compute_ratio > 0);
        assert_eq!(fv.task_has_npu_access, FP_SCALE);
        assert!(fv.gpu_util > 0);
        assert!(fv.npu_util > 0);
    }
}