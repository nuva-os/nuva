/*
 * Nuva OS - Kernel - PowerMgmt - Nvpowermgr - GreenMetrics
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
 * Nuva OS - Kernel - NvPowerMgr Green Metrics Collector
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Real-time green computing metrics: PUE, carbon emission,
 * and power efficiency score.
 */

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// Default carbon factor (500 gCO2e/kWh)
pub const DEFAULT_CARBON_FACTOR: u32 = 500;

/// GreenMetricsCollector: real-time green computing metrics
///
/// Calculates:
/// - PUE (Power Usage Effectiveness) = facility_power / IT_power
/// - Carbon emission equivalent = energy * carbon_factor
/// - Power efficiency score = performance / power
pub struct GreenMetricsCollector {
    /// IT equipment power (milliwatts)
    it_power_mw: AtomicU32,
    /// Total facility power (milliwatts)
    facility_power_mw: AtomicU32,
    /// Carbon emission factor (gCO2e/kWh)
    carbon_factor: AtomicU32,
    /// Total energy consumed (milliwatt-hours)
    total_energy_mwh: AtomicU64,
    /// Total carbon emitted (grams CO2e, scaled by 1000)
    total_carbon_g: AtomicU64,
    /// Performance score (arbitrary units)
    performance_score: AtomicU32,
    /// Collection count
    samples: AtomicU64,
}

impl GreenMetricsCollector {
    /// Create a new green metrics collector
    pub const fn new() -> Self {
        GreenMetricsCollector {
            it_power_mw: AtomicU32::new(0),
            facility_power_mw: AtomicU32::new(0),
            carbon_factor: AtomicU32::new(DEFAULT_CARBON_FACTOR),
            total_energy_mwh: AtomicU64::new(0),
            total_carbon_g: AtomicU64::new(0),
            performance_score: AtomicU32::new(0),
            samples: AtomicU64::new(0),
        }
    }

    /// Update power readings
    pub fn update_power(&self, it_power_mw: u32, facility_power_mw: u32) {
        self.it_power_mw.store(it_power_mw, Ordering::Release);
        self.facility_power_mw.store(facility_power_mw, Ordering::Release);
        self.samples.fetch_add(1, Ordering::Relaxed);
    }

    /// Calculate PUE (Power Usage Effectiveness)
    ///
    /// PUE = facility_power / IT_power
    /// Ideal PUE = 1.0, typical data center PUE = 1.2-2.0
    /// Returns PUE * 100 (fixed-point, e.g., 150 = PUE 1.50)
    pub fn pue(&self) -> u32 {
        let it = self.it_power_mw.load(Ordering::Acquire);
        let facility = self.facility_power_mw.load(Ordering::Acquire);
        if it == 0 {
            return 100;
        }
        (facility as u64 * 100 / it as u64) as u32
    }

    /// Update energy consumption
    pub fn add_energy(&self, energy_mwh: u32) {
        self.total_energy_mwh.fetch_add(energy_mwh as u64, Ordering::Relaxed);
        let factor = self.carbon_factor.load(Ordering::Acquire);
        // carbon_g = energy_mwh * carbon_factor / 1000000 (mWh to kWh)
        let carbon = (energy_mwh as u64 * factor as u64) / 1_000_000;
        self.total_carbon_g.fetch_add(carbon, Ordering::Relaxed);
    }

    /// Get total energy consumed (milliwatt-hours)
    pub fn total_energy_mwh(&self) -> u64 {
        self.total_energy_mw.load(Ordering::Acquire)
    }

    /// Get total carbon emitted (grams CO2e)
    pub fn total_carbon_g(&self) -> u64 {
        self.total_carbon_g.load(Ordering::Acquire)
    }

    /// Set carbon emission factor (gCO2e/kWh)
    pub fn set_carbon_factor(&self, factor: u32) {
        self.carbon_factor.store(factor, Ordering::Release);
    }

    /// Get carbon factor
    pub fn carbon_factor(&self) -> u32 {
        self.carbon_factor.load(Ordering::Acquire)
    }

    /// Update performance score
    pub fn set_performance_score(&self, score: u32) {
        self.performance_score.store(score, Ordering::Release);
    }

    /// Calculate power efficiency score
    ///
    /// efficiency = performance_score / power (higher = better)
    /// Returns efficiency * 1000 (fixed-point)
    pub fn power_efficiency_score(&self) -> u32 {
        let power = self.it_power_mw.load(Ordering::Acquire);
        let perf = self.performance_score.load(Ordering::Acquire);
        if power == 0 {
            return 0;
        }
        ((perf as u64 * 1000) / power as u64) as u32
    }

    /// Get sample count
    pub fn samples(&self) -> u64 {
        self.samples.load(Ordering::Acquire)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pue_calculation() {
        let gm = GreenMetricsCollector::new();
        gm.update_power(5000, 6000);
        assert_eq!(gm.pue(), 120);
    }

    #[test]
    fn test_pue_ideal() {
        let gm = GreenMetricsCollector::new();
        gm.update_power(5000, 5000);
        assert_eq!(gm.pue(), 100);
    }

    #[test]
    fn test_carbon_emission() {
        let gm = GreenMetricsCollector::new();
        gm.add_energy(1_000_000);
        assert!(gm.total_carbon_g() > 0);
    }

    #[test]
    fn test_power_efficiency() {
        let gm = GreenMetricsCollector::new();
        gm.update_power(5000, 6000);
        gm.set_performance_score(10000);
        let eff = gm.power_efficiency_score();
        assert!(eff > 0);
    }
}