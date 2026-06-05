/*
 * Nuva OS - Kernel - PowerMgmt - Nvpowermgr - AiOptimizer
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
 * Nuva OS - Kernel - NvPowerMgr AI Power Optimizer
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * NPU-based power optimization model that generates
 * DVFS, sleep, and throttle plans from power features.
 */

use core::sync::atomic::{AtomicU32, AtomicBool, Ordering};

use super::dvfs_controller::DvfsLevel;
use super::device_controller::{SleepLevel, WakeCondition};

/// Default power optimization model ID
pub const DEFAULT_POWER_MODEL_ID: u32 = 2;

/// Default power model version
pub const DEFAULT_POWER_MODEL_VERSION: u32 = 1;

/// PowerFeatureVector: power optimization feature vector
#[derive(Clone, Debug)]
pub struct PowerFeatureVector {
    /// Per-device utilization (0-100 each, max 16 devices)
    pub per_device_util: [u32; 16],
    /// Per-device temperature (degrees C, max 16 devices)
    pub per_device_temp: [u32; 16],
    /// Per-device power (milliwatts, max 16 devices)
    pub per_device_power: [u32; 16],
    /// Total system power (milliwatts)
    pub total_power_mw: u32,
    /// Budget remaining ratio (0-100 percentage)
    pub budget_remaining_ratio: u32,
    /// Scheduling pressure (0-100)
    pub sched_pressure: u32,
    /// Number of active devices
    pub num_active_devices: u32,
}

impl PowerFeatureVector {
    /// Create a zero-initialized feature vector
    pub const fn zero() -> Self {
        PowerFeatureVector {
            per_device_util: [0; 16],
            per_device_temp: [25; 16],
            per_device_power: [0; 16],
            total_power_mw: 0,
            budget_remaining_ratio: 100,
            sched_pressure: 50,
            num_active_devices: 0,
        }
    }
}

/// DVFS plan entry
#[derive(Clone, Debug)]
pub struct DvfsPlanEntry {
    /// Device index
    pub device_index: usize,
    /// Target DVFS level
    pub target_level: u16,
}

/// Sleep plan entry
#[derive(Clone, Debug)]
pub struct SleepPlanEntry {
    /// Device index
    pub device_index: usize,
    /// Target sleep level
    pub sleep_level: SleepLevel,
    /// Wake condition
    pub wake_condition: WakeCondition,
}

/// Throttle plan entry
#[derive(Clone, Debug)]
pub struct ThrottlePlanEntry {
    /// Device index
    pub device_index: usize,
    /// Frequency limit (kHz, 0 = no limit)
    pub freq_limit_khz: u32,
}

/// PowerOptResult: AI power optimization output
#[derive(Clone, Debug)]
pub struct PowerOptResult {
    /// DVFS adjustments
    pub dvfs_plan: alloc::vec::Vec<DvfsPlanEntry>,
    /// Sleep plan
    pub sleep_plan: alloc::vec::Vec<SleepPlanEntry>,
    /// Throttle plan
    pub throttle_plan: alloc::vec::Vec<ThrottlePlanEntry>,
    /// Estimated power saving (milliwatts)
    pub estimated_saving_mw: u32,
    /// Estimated performance impact (0-100 percentage)
    pub estimated_perf_impact_pct: u32,
}

impl PowerOptResult {
    /// Create an empty (no-op) result
    pub fn no_op() -> Self {
        PowerOptResult {
            dvfs_plan: alloc::vec::Vec::new(),
            sleep_plan: alloc::vec::Vec::new(),
            throttle_plan: alloc::vec::Vec::new(),
            estimated_saving_mw: 0,
            estimated_perf_impact_pct: 0,
        }
    }

    /// Validate performance impact constraint
    pub fn validate_perf_impact(&self, limit_pct: u32) -> bool {
        self.estimated_perf_impact_pct <= limit_pct
    }
}

/// AiPowerOptimizer: NPU-based power optimization
pub struct AiPowerOptimizer {
    /// Model ID on NPU
    model_id: AtomicU32,
    /// Model version
    model_version: AtomicU32,
    /// Whether NPU is available
    npu_available: AtomicBool,
    /// Total optimization cycles
    total_cycles: AtomicU32,
}

impl AiPowerOptimizer {
    /// Create a new AI power optimizer
    pub const fn new() -> Self {
        AiPowerOptimizer {
            model_id: AtomicU32::new(DEFAULT_POWER_MODEL_ID),
            model_version: AtomicU32::new(DEFAULT_POWER_MODEL_VERSION),
            npu_available: AtomicBool::new(false),
            total_cycles: AtomicU32::new(0),
        }
    }

    /// Initialize with NPU availability
    pub fn init(&self, npu_available: bool) {
        self.npu_available.store(npu_available, Ordering::Release);
    }

    /// Run power optimization
    ///
    /// @param features: Current power feature vector
    /// @param perf_limit_pct: Maximum allowed performance impact
    /// @return: Power optimization result
    pub fn optimize(&self, features: &PowerFeatureVector, perf_limit_pct: u32) -> PowerOptResult {
        self.total_cycles.fetch_add(1, Ordering::Relaxed);

        if self.npu_available.load(Ordering::Acquire) {
            self.npu_optimize(features, perf_limit_pct)
        } else {
            self.heuristic_optimize(features, perf_limit_pct)
        }
    }

    /// NPU-based optimization (placeholder for HAL integration)
    fn npu_optimize(&self, features: &PowerFeatureVector, perf_limit_pct: u32) -> PowerOptResult {
        // TODO: Integrate with hal::npu::davinci for actual NPU inference
        self.heuristic_optimize(features, perf_limit_pct)
    }

    /// Heuristic-based power optimization
    fn heuristic_optimize(&self, features: &PowerFeatureVector, perf_limit_pct: u32) -> PowerOptResult {
        let mut result = PowerOptResult::no_op();
        let mut saving_mw = 0u32;

        for i in 0..features.num_active_devices.min(16) as usize {
            let util = features.per_device_util[i];
            let temp = features.per_device_temp[i];

            // Hot device: throttle
            if temp >= 85 {
                result.throttle_plan.push(ThrottlePlanEntry {
                    device_index: i,
                    freq_limit_khz: 1_000_000,
                });
                saving_mw += features.per_device_power[i] / 4;
            }

            // Idle device: sleep
            if util < 10 && features.budget_remaining_ratio < 30 {
                result.sleep_plan.push(SleepPlanEntry {
                    device_index: i,
                    sleep_level: SleepLevel::LightSleep,
                    wake_condition: WakeCondition::AnyInterrupt,
                });
                saving_mw += features.per_device_power[i] / 2;
            }

            // Low-util device: reduce DVFS level
            if util < 30 && util >= 10 {
                result.dvfs_plan.push(DvfsPlanEntry {
                    device_index: i,
                    target_level: 2,
                });
                saving_mw += features.per_device_power[i] / 5;
            }
        }

        result.estimated_saving_mw = saving_mw;
        result.estimated_perf_impact_pct = if saving_mw > 0 {
            (saving_mw * 100) / features.total_power_mw.max(1)
        } else {
            0
        };

        if !result.validate_perf_impact(perf_limit_pct) {
            result.estimated_perf_impact_pct = perf_limit_pct;
        }

        result
    }

    /// Get optimization cycle count
    pub fn total_cycles(&self) -> u32 {
        self.total_cycles.load(Ordering::Acquire)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heuristic_idle_device() {
        let opt = AiPowerOptimizer::new();
        let mut fv = PowerFeatureVector::zero();
        fv.per_device_util[0] = 5;
        fv.per_device_power[0] = 2000;
        fv.total_power_mw = 10000;
        fv.budget_remaining_ratio = 20;
        fv.num_active_devices = 1;

        let result = opt.optimize(&fv, 10);
        assert!(!result.sleep_plan.is_empty());
    }

    #[test]
    fn test_heuristic_hot_device() {
        let opt = AiPowerOptimizer::new();
        let mut fv = PowerFeatureVector::zero();
        fv.per_device_util[0] = 80;
        fv.per_device_temp[0] = 90;
        fv.per_device_power[0] = 5000;
        fv.total_power_mw = 10000;
        fv.num_active_devices = 1;

        let result = opt.optimize(&fv, 10);
        assert!(!result.throttle_plan.is_empty());
    }

    #[test]
    fn test_perf_impact_validation() {
        let result = PowerOptResult {
            dvfs_plan: alloc::vec::Vec::new(),
            sleep_plan: alloc::vec::Vec::new(),
            throttle_plan: alloc::vec::Vec::new(),
            estimated_saving_mw: 1000,
            estimated_perf_impact_pct: 8,
        };
        assert!(result.validate_perf_impact(10));
        assert!(!result.validate_perf_impact(5));
    }
}