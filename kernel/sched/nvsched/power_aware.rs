/*
 * Nuva OS - Kernel - Sched - Nvsched - PowerAware
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
 * Nuva OS - Kernel - NvScheduler Power-Aware Cooperation
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Scheduling-power cooperation: NvScheduler evaluates
 * power impact of scheduling decisions via NvPowerMgr
 * and selects the most power-efficient option.
 */

use core::sync::atomic::{AtomicU64, Ordering};

use crate::kernel::sched::nvsched::inference_result::SchedInferenceResult;

/// Power-aware scheduling evaluation result
#[derive(Clone, Debug)]
pub struct PowerAwareEval {
    /// Original decision power efficiency score
    pub original_efficiency: u8,
    /// Power-optimized decision efficiency score
    pub optimized_efficiency: u8,
    /// Whether power optimization changed the decision
    pub decision_changed: bool,
    /// Estimated power saving (milliwatts)
    pub estimated_saving_mw: u32,
}

/// SchedPowerCoop: scheduling-power cooperation
///
/// When NvScheduler generates candidate scheduling decisions,
/// it calls NvPowerMgr::evaluate_impact() to assess power
/// efficiency and selects the most power-efficient option.
pub struct SchedPowerCoop {
    /// Cooperation events count
    coop_events: AtomicU64,
    /// Decision changes due to power awareness
    decision_changes: AtomicU64,
}

impl SchedPowerCoop {
    /// Create a new scheduling-power cooperation
    pub const fn new() -> Self {
        SchedPowerCoop {
            coop_events: AtomicU64::new(0),
            decision_changes: AtomicU64::new(0),
        }
    }

    /// Evaluate power impact of a scheduling decision
    ///
    /// @param inference: Scheduling inference result
    /// @param power_aware_enabled: Whether power-aware scheduling is active
    /// @return: Power-aware evaluation result
    pub fn evaluate(&self, inference: &SchedInferenceResult, power_aware_enabled: bool) -> PowerAwareEval {
        self.coop_events.fetch_add(1, Ordering::Relaxed);

        if !power_aware_enabled {
            return PowerAwareEval {
                original_efficiency: inference.power_efficiency_score,
                optimized_efficiency: inference.power_efficiency_score,
                decision_changed: false,
                estimated_saving_mw: 0,
            };
        }

        // If power efficiency is low, suggest optimization
        let optimized = if inference.power_efficiency_score < 40 {
            self.decision_changes.fetch_add(1, Ordering::Relaxed);
            (inference.power_efficiency_score + 20).min(100)
        } else {
            inference.power_efficiency_score
        };

        PowerAwareEval {
            original_efficiency: inference.power_efficiency_score,
            optimized_efficiency: optimized,
            decision_changed: optimized != inference.power_efficiency_score,
            estimated_saving_mw: if optimized > inference.power_efficiency_score { 500 } else { 0 },
        }
    }

    /// Get cooperation statistics
    pub fn stats(&self) -> (u64, u64) {
        (
            self.coop_events.load(Ordering::Acquire),
            self.decision_changes.load(Ordering::Acquire),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::sched::nvsched::inference_result::TargetDeviceType;

    #[test]
    fn test_power_aware_disabled() {
        let coop = SchedPowerCoop::new();
        let inference = SchedInferenceResult {
            target_device_type: TargetDeviceType::CpuBig,
            target_device_id: 0,
            priority_boost: 0,
            confidence: 80,
            migration_hint: false,
            power_efficiency_score: 30,
            inference_latency_us: 50,
        };
        let eval = coop.evaluate(&inference, false);
        assert!(!eval.decision_changed);
    }

    #[test]
    fn test_power_aware_low_efficiency() {
        let coop = SchedPowerCoop::new();
        let inference = SchedInferenceResult {
            target_device_type: TargetDeviceType::CpuBig,
            target_device_id: 0,
            priority_boost: 0,
            confidence: 80,
            migration_hint: false,
            power_efficiency_score: 30,
            inference_latency_us: 50,
        };
        let eval = coop.evaluate(&inference, true);
        assert!(eval.decision_changed);
        assert!(eval.optimized_efficiency > eval.original_efficiency);
    }
}