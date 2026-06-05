/*
 * Nuva OS - Kernel - PowerMgmt - Nvpowermgr - Stats
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
 * Nuva OS - Kernel - NvPowerMgr Statistics
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Comprehensive power management statistics.
 */

use core::sync::atomic::{AtomicU64, AtomicU32, Ordering};

/// NvPowerMgrStats: comprehensive power management statistics
pub struct NvPowerMgrStats {
    /// Optimization cycles
    pub optimization_cycles: AtomicU64,
    /// DVFS adjustments
    pub dvfs_adjustments: AtomicU64,
    /// Device sleep events
    pub device_sleep_events: AtomicU64,
    /// Device wake events
    pub device_wakeup_events: AtomicU64,
    /// Thermal throttle events
    pub thermal_throttle_events: AtomicU64,
    /// Budget violations
    pub budget_violations: AtomicU64,
    /// PMIC failures
    pub pmic_failures: AtomicU64,
    /// Total energy saved (milliwatt-hours)
    pub total_energy_saved_mwh: AtomicU64,
}

impl NvPowerMgrStats {
    /// Create zero-initialized stats
    pub const fn new() -> Self {
        NvPowerMgrStats {
            optimization_cycles: AtomicU64::new(0),
            dvfs_adjustments: AtomicU64::new(0),
            device_sleep_events: AtomicU64::new(0),
            device_wakeup_events: AtomicU64::new(0),
            thermal_throttle_events: AtomicU64::new(0),
            budget_violations: AtomicU64::new(0),
            pmic_failures: AtomicU64::new(0),
            total_energy_saved_mwh: AtomicU64::new(0),
        }
    }

    /// Record optimization cycle
    pub fn record_optimization_cycle(&self) {
        self.optimization_cycles.fetch_add(1, Ordering::Relaxed);
    }

    /// Record DVFS adjustment
    pub fn record_dvfs_adjustment(&self) {
        self.dvfs_adjustments.fetch_add(1, Ordering::Relaxed);
    }

    /// Record device sleep
    pub fn record_device_sleep(&self) {
        self.device_sleep_events.fetch_add(1, Ordering::Relaxed);
    }

    /// Record device wake
    pub fn record_device_wakeup(&self) {
        self.device_wakeup_events.fetch_add(1, Ordering::Relaxed);
    }

    /// Record thermal throttle
    pub fn record_thermal_throttle(&self) {
        self.thermal_throttle_events.fetch_add(1, Ordering::Relaxed);
    }

    /// Record budget violation
    pub fn record_budget_violation(&self) {
        self.budget_violations.fetch_add(1, Ordering::Relaxed);
    }

    /// Record PMIC failure
    pub fn record_pmic_failure(&self) {
        self.pmic_failures.fetch_add(1, Ordering::Relaxed);
    }

    /// Add energy saved
    pub fn add_energy_saved_mwh(&self, mwh: u64) {
        self.total_energy_saved_mwh.fetch_add(mwh, Ordering::Relaxed);
    }
}

/// Global NvPowerMgrStats instance
static NV_POWERMGR_STATS: NvPowerMgrStats = NvPowerMgrStats::new();

/// Get global power management statistics
pub fn get_nv_powermgr_stats() -> &'static NvPowerMgrStats {
    &NV_POWERMGR_STATS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats() {
        let stats = NvPowerMgrStats::new();
        stats.record_optimization_cycle();
        stats.record_dvfs_adjustment();
        stats.record_device_sleep();
        stats.add_energy_saved_mwh(500);

        assert_eq!(stats.optimization_cycles.load(Ordering::Relaxed), 1);
        assert_eq!(stats.dvfs_adjustments.load(Ordering::Relaxed), 1);
        assert_eq!(stats.device_sleep_events.load(Ordering::Relaxed), 1);
        assert_eq!(stats.total_energy_saved_mwh.load(Ordering::Relaxed), 500);
    }
}