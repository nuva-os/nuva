/*
 * Nuva OS - Kernel - Sched - Nvsched - DecisionMaker
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
 * Nuva OS - Kernel - NvScheduler Decision Maker
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Core scheduling decision flow:
 * 1. Build SchedFeatureVector
 * 2. Submit NPU inference
 * 3. Evaluate confidence
 * 4. Evaluate power impact
 * 5. Evaluate balance need
 * 6. Output final decision
 */

use core::sync::atomic::{AtomicU64, Ordering};

use super::feature_vector::SchedFeatureVector;
use super::inference_result::SchedInferenceResult;
use super::npu_inference_engine::NpuInferenceEngine;
use super::sched_class::NvAiSchedClass;
use super::task_classifier::{AiTaskClassifier, TaskClassFeatures};

/// Scheduling decision action
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SchedAction {
    /// Place task on target device
    Place = 0,
    /// Migrate task to target device
    Migrate = 1,
    /// Boost task priority
    Boost = 2,
    /// Yield current scheduling decision (no change)
    Yield = 3,
}

/// Complete scheduling decision
#[derive(Clone, Debug)]
pub struct SchedDecision {
    /// Decision ID (monotonically increasing)
    pub decision_id: u64,
    /// Target process ID
    pub target_pid: u32,
    /// Scheduling action
    pub action: SchedAction,
    /// Assigned scheduling class
    pub sched_class: NvAiSchedClass,
    /// NPU inference result
    pub inference_result: SchedInferenceResult,
    /// Whether this is a fallback decision
    pub is_fallback: bool,
}

/// SchedDecisionMaker: core scheduling decision flow
///
/// Orchestrates the complete scheduling decision pipeline:
/// feature extraction -> NPU inference -> confidence check ->
/// power/balance evaluation -> final decision output.
pub struct SchedDecisionMaker {
    /// NPU inference engine
    npu_engine: NpuInferenceEngine,
    /// Decision counter
    next_decision_id: AtomicU64,
    /// AI-driven decisions count
    ai_decisions: AtomicU64,
    /// Fallback decisions count
    fallback_decisions: AtomicU64,
    /// Low-confidence events count
    low_confidence_events: AtomicU64,
}

impl SchedDecisionMaker {
    /// Create a new decision maker
    pub const fn new() -> Self {
        SchedDecisionMaker {
            npu_engine: NpuInferenceEngine::new(),
            next_decision_id: AtomicU64::new(1),
            ai_decisions: AtomicU64::new(0),
            fallback_decisions: AtomicU64::new(0),
            low_confidence_events: AtomicU64::new(0),
        }
    }

    /// Initialize decision maker
    pub fn init(&self, npu_available: bool) {
        self.npu_engine.init(npu_available);
    }

    /// Make scheduling decision
    ///
    /// @param feature_vec: Current scheduling feature vector
    /// @param task_features: Task classification features
    /// @param target_pid: Target process ID
    /// @param confidence_threshold: AI confidence threshold (0-100)
    /// @param budget_us: Inference time budget in microseconds
    /// @param power_aware: Whether to consider power impact
    /// @return: Scheduling decision
    pub fn make_decision(
        &self,
        feature_vec: &SchedFeatureVector,
        task_features: &TaskClassFeatures,
        target_pid: u32,
        confidence_threshold: u32,
        budget_us: u32,
        power_aware: bool,
    ) -> SchedDecision {
        let decision_id = self.next_decision_id.fetch_add(1, Ordering::Relaxed);

        // Step 1: Classify task
        let sched_class = AiTaskClassifier::classify(task_features);

        // Step 2: Submit NPU inference
        let inference_result = self.npu_engine.infer(feature_vec, budget_us);

        // Step 3: Evaluate confidence
        let is_fallback;
        let final_inference;

        if inference_result.meets_confidence(confidence_threshold) && !inference_result.is_timeout(budget_us) {
            // High-confidence AI decision
            self.ai_decisions.fetch_add(1, Ordering::Relaxed);
            is_fallback = false;
            final_inference = inference_result;
        } else {
            // Low confidence or timeout - use fallback
            self.fallback_decisions.fetch_add(1, Ordering::Relaxed);
            if !inference_result.meets_confidence(confidence_threshold) {
                self.low_confidence_events.fetch_add(1, Ordering::Relaxed);
            }
            is_fallback = true;

            // Try last valid result first
            if let Some(last_valid) = self.npu_engine.get_last_valid() {
                final_inference = last_valid;
            } else {
                final_inference = inference_result;
            }
        }

        // Step 4: Evaluate power impact (if power-aware)
        let action = if power_aware && final_inference.power_efficiency_score < 40 {
            SchedAction::Yield
        } else if final_inference.migration_hint {
            SchedAction::Migrate
        } else if final_inference.priority_boost > 0 {
            SchedAction::Boost
        } else {
            SchedAction::Place
        };

        SchedDecision {
            decision_id,
            target_pid,
            action,
            sched_class,
            inference_result: final_inference,
            is_fallback,
        }
    }

    /// Get NPU inference engine reference
    pub fn npu_engine(&self) -> &NpuInferenceEngine {
        &self.npu_engine
    }

    /// Get decision statistics
    pub fn stats(&self) -> (u64, u64, u64, u64) {
        (
            self.next_decision_id.load(Ordering::Acquire) - 1,
            self.ai_decisions.load(Ordering::Acquire),
            self.fallback_decisions.load(Ordering::Acquire),
            self.low_confidence_events.load(Ordering::Acquire),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::feature_vector::FP_SCALE;

    fn make_test_feature_vec() -> SchedFeatureVector {
        SchedFeatureVector::build(
            0.8, 50, true, 0,
            0.5, 0.3, 0.4, 0.6,
            0.1, 0.9, 0.2, 5,
        )
    }

    #[test]
    fn test_make_decision() {
        let maker = SchedDecisionMaker::new();
        maker.init(false);

        let fv = make_test_feature_vec();
        let tf = TaskClassFeatures::new(80, 50, true, 90, false);

        let decision = maker.make_decision(&fv, &tf, 1, 50, 100, true);
        assert_eq!(decision.target_pid, 1);
        assert!(decision.decision_id > 0);
    }

    #[test]
    fn test_decision_fallback_on_no_npu() {
        let maker = SchedDecisionMaker::new();
        maker.init(false);

        let fv = make_test_feature_vec();
        let tf = TaskClassFeatures::new(80, 50, true, 90, false);

        let decision = maker.make_decision(&fv, &tf, 1, 50, 100, false);
        assert!(decision.is_fallback);
    }

    #[test]
    fn test_decision_stats() {
        let maker = SchedDecisionMaker::new();
        maker.init(false);

        let fv = make_test_feature_vec();
        let tf = TaskClassFeatures::new(50, 50, true, 50, false);

        let _ = maker.make_decision(&fv, &tf, 1, 50, 100, false);
        let _ = maker.make_decision(&fv, &tf, 2, 50, 100, false);

        let (total, _, fallback, _) = maker.stats();
        assert_eq!(total, 2);
        assert!(fallback > 0);
    }
}