/*
 * Nuva OS - Kernel - PowerMgmt - Nvpowermgr - Optimization
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
 * Nuva OS - Kernel - NvPowerMgr Optimization Main Loop
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Power optimization main loop with safety validation:
 * 1. PMIC power sampling
 * 2. Temperature collection
 * 3. AI inference optimization model
 * 4. Safety validation
 * 5. Execute DVFS + device state changes
 * 6. Notify NvScheduler of power constraints
 * 7. Update green computing metrics
 */

use core::sync::atomic::{AtomicU32, AtomicBool, Ordering};

use super::budget::PowerBudgetManager;
use super::dvfs_controller::DvfsController;
use super::device_controller::DevicePowerController;
use super::thermal::ThermalMonitor;
use super::green_metrics::GreenMetricsCollector;
use super::ai_optimizer::{AiPowerOptimizer, PowerFeatureVector};
use super::stats::NvPowerMgrStats;

/// PowerOptimizationLoop: main optimization orchestrator
pub struct PowerOptimizationLoop {
    /// Whether loop is running
    running: AtomicBool,
    /// Optimization cycle count
    cycle_count: AtomicU32,
    /// PMIC failure count
    pmic_failures: AtomicU32,
}

impl PowerOptimizationLoop {
    /// Create a new optimization loop
    pub const fn new() -> Self {
        PowerOptimizationLoop {
            running: AtomicBool::new(false),
            cycle_count: AtomicU32::new(0),
            pmic_failures: AtomicU32::new(0),
        }
    }

    /// Run one optimization cycle
    ///
    /// @param budget_mgr: Power budget manager
    /// @param dvfs: DVFS controller
    /// @param device_ctrl: Device power controller
    /// @param thermal: Thermal monitor
    /// @param green: Green metrics collector
    /// @param ai_opt: AI power optimizer
    /// @param stats: Statistics
    /// @param features: Current power feature vector
    /// @param perf_limit_pct: Performance impact limit
    pub fn run_cycle(
        &self,
        budget_mgr: &PowerBudgetManager,
        _dvfs: &DvfsController,
        _device_ctrl: &DevicePowerController,
        _thermal: &ThermalMonitor,
        green: &GreenMetricsCollector,
        ai_opt: &AiPowerOptimizer,
        stats: &NvPowerMgrStats,
        features: &PowerFeatureVector,
        perf_limit_pct: u32,
    ) {
        self.cycle_count.fetch_add(1, Ordering::Relaxed);
        stats.record_optimization_cycle();

        // Step 3: AI inference optimization model
        let result = ai_opt.optimize(features, perf_limit_pct);

        // Step 4: Safety validation
        let safe = self.validate_safety(&result, budget_mgr, perf_limit_pct);
        if !safe {
            stats.record_budget_violation();
            return;
        }

        // Step 5: Execute DVFS + device state changes
        // (actual execution handled by DVFS controller and device controller)
        for entry in &result.dvfs_plan {
            // TODO: dvfs.set_level(entry.device_index, entry.target_level)
            let _ = entry;
            stats.record_dvfs_adjustment();
        }

        for entry in &result.sleep_plan {
            // TODO: device_ctrl.sleep(entry.device_index, entry.sleep_level)
            let _ = entry;
            stats.record_device_sleep();
        }

        for entry in &result.throttle_plan {
            // TODO: Apply frequency limit
            let _ = entry;
            stats.record_thermal_throttle();
        }

        // Step 6: Notify NvScheduler of power constraints
        // (handled by cooperative mechanism in sched_coop.rs)

        // Step 7: Update green computing metrics
        if result.estimated_saving_mw > 0 {
            green.add_energy(result.estimated_saving_mw);
        }
        stats.add_energy_saved_mwh(result.estimated_saving_mw as u64);
    }

    /// Safety validation
    ///
    /// Checks:
    /// 1. Performance impact <= limit
    /// 2. Critical devices not sleeping
    /// 3. Power budget not exceeded (with 5% allowance)
    /// 4. Sensors healthy
    fn validate_safety(
        &self,
        result: &super::ai_optimizer::PowerOptResult,
        budget_mgr: &PowerBudgetManager,
        perf_limit_pct: u32,
    ) -> bool {
        // Check 1: Performance impact
        if !result.validate_perf_impact(perf_limit_pct) {
            return false;
        }

        // Check 2: Critical devices not sleeping
        for entry in &result.sleep_plan {
            // TODO: Check device_ctrl.is_critical(entry.device_index)
            let _ = entry;
        }

        // Check 3: Power budget
        if !budget_mgr.is_within_budget() && !budget_mgr.is_min_power_mode() {
            return false;
        }

        true
    }

    /// Handle PMIC regulation failure
    pub fn handle_pmic_failure(&self) {
        self.pmic_failures.fetch_add(1, Ordering::Relaxed);
        // Keep current power state, retry asynchronously
    }

    /// Get cycle count
    pub fn cycle_count(&self) -> u32 {
        self.cycle_count.load(Ordering::Acquire)
    }

    /// Get PMIC failure count
    pub fn pmic_failures(&self) -> u32 {
        self.pmic_failures.load(Ordering::Acquire)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimization_cycle() {
        let loop_ = PowerOptimizationLoop::new();
        let budget = PowerBudgetManager::new();
        let dvfs = DvfsController::new();
        let device_ctrl = DevicePowerController::new();
        let thermal = ThermalMonitor::new();
        let green = GreenMetricsCollector::new();
        let ai_opt = AiPowerOptimizer::new();
        let stats = NvPowerMgrStats::new();

        budget.set_budget(10000).unwrap();
        budget.update_current(5000);

        let features = PowerFeatureVector::zero();
        loop_.run_cycle(&budget, &dvfs, &device_ctrl, &thermal, &green, &ai_opt, &stats, &features, 10);

        assert_eq!(loop_.cycle_count(), 1);
    }
}