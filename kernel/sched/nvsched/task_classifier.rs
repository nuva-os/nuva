/*
 * Nuva OS - Kernel - Sched - Nvsched - TaskClassifier
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
 * Nuva OS - Kernel - NvScheduler AI Task Classifier
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Classifies tasks into four-level scheduling classes
 * based on compute characteristics, NPU access, and
 * memory usage patterns.
 */

use super::sched_class::NvAiSchedClass;

/// Task classification features
#[derive(Clone, Debug)]
pub struct TaskClassFeatures {
    /// Compute/IO ratio (0-100, higher = more compute)
    pub compute_ratio: u32,
    /// Memory usage in MB
    pub memory_mb: u32,
    /// Whether task has NPU access
    pub has_npu_access: bool,
    /// Expected latency sensitivity (0-100, higher = more sensitive)
    pub latency_sensitivity: u32,
    /// Whether task is periodic/batch
    pub is_batch: bool,
}

impl TaskClassFeatures {
    /// Create new task classification features
    pub const fn new(
        compute_ratio: u32,
        memory_mb: u32,
        has_npu_access: bool,
        latency_sensitivity: u32,
        is_batch: bool,
    ) -> Self {
        TaskClassFeatures {
            compute_ratio,
            memory_mb,
            has_npu_access,
            latency_sensitivity,
            is_batch,
        }
    }
}

/// AiTaskClassifier: AI-driven task classification
///
/// Uses task characteristics to assign scheduling class.
/// Classification rules:
/// - NPU access + high latency sensitivity -> AI_REALTIME
/// - NPU access + moderate compute -> AI_NORMAL
/// - Batch flag or high memory -> AI_BATCH
/// - Low compute + no NPU -> AI_IDLE
pub struct AiTaskClassifier;

impl AiTaskClassifier {
    /// Classify a task based on its features
    ///
    /// @param features: Task classification features
    /// @return: Assigned scheduling class
    pub fn classify(features: &TaskClassFeatures) -> NvAiSchedClass {
        if features.is_batch {
            return NvAiSchedClass::AiBatch;
        }

        if features.has_npu_access {
            if features.latency_sensitivity >= 70 && features.compute_ratio >= 60 {
                return NvAiSchedClass::AiRealtime;
            }
            if features.compute_ratio >= 40 || features.latency_sensitivity >= 40 {
                return NvAiSchedClass::AiNormal;
            }
            return NvAiSchedClass::AiBatch;
        }

        if features.compute_ratio < 20 && features.latency_sensitivity < 30 {
            return NvAiSchedClass::AiIdle;
        }

        if features.compute_ratio >= 50 {
            return NvAiSchedClass::AiNormal;
        }

        NvAiSchedClass::AiBatch
    }

    /// Re-classify a task when its features change
    ///
    /// @param current_class: Current scheduling class
    /// @param new_features: Updated task features
    /// @return: New scheduling class (may be same as current)
    pub fn reclassify(current_class: NvAiSchedClass, new_features: &TaskClassFeatures) -> NvAiSchedClass {
        let new_class = Self::classify(new_features);

        // Prevent rapid oscillation: only change if difference > 1 level
        let current = current_class as u8;
        let new_val = new_class as u8;
        if current.abs_diff(new_val) <= 1 && current != new_val {
            return current_class;
        }

        new_class
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_inference_realtime() {
        let features = TaskClassFeatures::new(80, 50, true, 90, false);
        assert_eq!(AiTaskClassifier::classify(&features), NvAiSchedClass::AiRealtime);
    }

    #[test]
    fn test_classify_npu_normal() {
        let features = TaskClassFeatures::new(50, 100, true, 50, false);
        assert_eq!(AiTaskClassifier::classify(&features), NvAiSchedClass::AiNormal);
    }

    #[test]
    fn test_classify_batch() {
        let features = TaskClassFeatures::new(90, 500, true, 20, true);
        assert_eq!(AiTaskClassifier::classify(&features), NvAiSchedClass::AiBatch);
    }

    #[test]
    fn test_classify_idle() {
        let features = TaskClassFeatures::new(10, 5, false, 10, false);
        assert_eq!(AiTaskClassifier::classify(&features), NvAiSchedClass::AiIdle);
    }

    #[test]
    fn test_classify_npu_low_compute() {
        let features = TaskClassFeatures::new(20, 30, true, 20, false);
        assert_eq!(AiTaskClassifier::classify(&features), NvAiSchedClass::AiBatch);
    }

    #[test]
    fn test_reclassify_oscillation_prevention() {
        let features = TaskClassFeatures::new(55, 50, true, 45, false);
        let result = AiTaskClassifier::reclassify(NvAiSchedClass::AiRealtime, &features);
        assert_eq!(result, NvAiSchedClass::AiRealtime);
    }
}