/*
 * Nuva OS - Kernel - PowerMgmt - Nvpowermgr - Fallback
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
 * Nuva OS - Kernel - NvPowerMgr Fallback Policy
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Fallback strategies when AI optimization is unavailable:
 * - NPU unavailable -> heuristic DVFS lookup + temperature thresholds
 * - Budget infeasible -> minimum power mode
 * - Sensor failure -> conservative policy (limit max frequency)
 * - PMIC failure -> maintain current state
 */

use super::ai_optimizer::{PowerOptResult, PowerFeatureVector, DvfsPlanEntry, ThrottlePlanEntry};
use super::device_controller::{SleepLevel, WakeCondition};
use super::ai_optimizer::SleepPlanEntry;

/// PowerFallbackPolicy: fallback power management
pub struct PowerFallbackPolicy;

impl PowerFallbackPolicy {
    /// Heuristic-based fallback when NPU is unavailable
    ///
    /// Uses DVFS lookup tables and temperature thresholds
    /// instead of AI inference.
    pub fn heuristic_fallback(features: &PowerFeatureVector, perf_limit_pct: u32) -> PowerOptResult {
        let mut result = PowerOptResult::no_op();
        let mut saving_mw = 0u32;

        for i in 0..features.num_active_devices.min(16) as usize {
            let util = features.per_device_util[i];
            let temp = features.per_device_temp[i];

            // Temperature-based throttling (conservative)
            if temp >= 80 {
                result.throttle_plan.push(ThrottlePlanEntry {
                    device_index: i,
                    freq_limit_khz: 800_000,
                });
                saving_mw += features.per_device_power[i] / 3;
            }

            // DVFS lookup table approach
            let target_level = if util > 80 { 0 }
            else if util > 60 { 1 }
            else if util > 40 { 2 }
            else if util > 20 { 3 }
            else { 4 };

            if target_level > 0 {
                result.dvfs_plan.push(DvfsPlanEntry {
                    device_index: i,
                    target_level,
                });
                saving_mw += (features.per_device_power[i] / 10) * (target_level as u32).min(3);
            }
        }

        result.estimated_saving_mw = saving_mw;
        result.estimated_perf_impact_pct = if saving_mw > 0 && features.total_power_mw > 0 {
            ((saving_mw * 100) / features.total_power_mw).min(perf_limit_pct)
        } else {
            0
        };

        result
    }

    /// Minimum power mode (budget infeasible)
    ///
    /// All devices at lowest DVFS level, non-critical
    /// devices in deep sleep.
    pub fn min_power_mode(features: &PowerFeatureVector) -> PowerOptResult {
        let mut result = PowerOptResult::no_op();
        let mut saving_mw = 0u32;

        for i in 0..features.num_active_devices.min(16) as usize {
            // All devices to lowest DVFS level
            result.dvfs_plan.push(DvfsPlanEntry {
                device_index: i,
                target_level: 4,
            });
            saving_mw += features.per_device_power[i] / 2;

            // Non-critical devices to deep sleep
            // (critical check would be done at execution time)
            if features.per_device_util[i] < 50 {
                result.sleep_plan.push(SleepPlanEntry {
                    device_index: i,
                    sleep_level: SleepLevel::DeepSleep,
                    wake_condition: WakeCondition::SoftwareRequest,
                });
                saving_mw += features.per_device_power[i] / 2;
            }
        }

        result.estimated_saving_mw = saving_mw;
        result.estimated_perf_impact_pct = 10; // Maximum allowed
        result
    }

    /// Conservative policy on sensor failure
    ///
    /// Limits maximum frequency to prevent overheating
    /// when temperature readings are unreliable.
    pub fn conservative_policy(features: &PowerFeatureVector) -> PowerOptResult {
        let mut result = PowerOptResult::no_op();

        for i in 0..features.num_active_devices.min(16) as usize {
            // Limit to 70% max frequency
            result.throttle_plan.push(ThrottlePlanEntry {
                device_index: i,
                freq_limit_khz: 1_400_000,
            });
        }

        result.estimated_perf_impact_pct = 30;
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_features() -> PowerFeatureVector {
        let mut fv = PowerFeatureVector::zero();
        fv.per_device_util[0] = 30;
        fv.per_device_temp[0] = 45;
        fv.per_device_power[0] = 3000;
        fv.total_power_mw = 10000;
        fv.num_active_devices = 1;
        fv
    }

    #[test]
    fn test_heuristic_fallback() {
        let fv = make_features();
        let result = PowerFallbackPolicy::heuristic_fallback(&fv, 10);
        assert!(result.estimated_perf_impact_pct <= 10);
    }

    #[test]
    fn test_min_power_mode() {
        let fv = make_features();
        let result = PowerFallbackPolicy::min_power_mode(&fv);
        assert!(result.estimated_saving_mw > 0);
    }

    #[test]
    fn test_conservative_policy() {
        let fv = make_features();
        let result = PowerFallbackPolicy::conservative_policy(&fv);
        assert!(!result.throttle_plan.is_empty());
    }
}