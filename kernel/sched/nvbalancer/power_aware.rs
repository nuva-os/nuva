/*
 * Nuva OS - Kernel - Sched - Nvbalancer - PowerAware
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
 * Nuva OS - Kernel - NvBalancer Power-Aware Cooperation
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Balance decisions consider power efficiency:
 * BalanceOptimizer queries NvPowerMgr device power
 * state and prefers power-efficient devices.
 */

use core::sync::atomic::{AtomicU64, Ordering};

/// Device power attributes for balance evaluation
#[derive(Clone, Debug)]
pub struct DevicePowerAttrs {
    /// Device index
    pub device_index: usize,
    /// Power efficiency score (0-100, higher = better)
    pub efficiency_score: u8,
    /// Current power consumption (milliwatts)
    pub power_mw: u32,
    /// Whether device is thermally throttled
    pub thermally_throttled: bool,
}

/// BalancerPowerCoop: balance-power cooperation
///
/// When BalanceOptimizer evaluates candidate devices,
/// it queries NvPowerMgr for per-device power state
/// and prefers power-efficient devices.
pub struct BalancerPowerCoop {
    /// Cooperation events
    coop_events: AtomicU64,
    /// Decisions changed due to power awareness
    decision_changes: AtomicU64,
}

impl BalancerPowerCoop {
    /// Create a new balancer-power cooperation
    pub const fn new() -> Self {
        BalancerPowerCoop {
            coop_events: AtomicU64::new(0),
            decision_changes: AtomicU64::new(0),
        }
    }

    /// Adjust balance score based on power efficiency
    ///
    /// @param base_score: Base matching score (0-100)
    /// @param power_attrs: Device power attributes
    /// @return: Power-adjusted score (0-100)
    pub fn adjust_score(&self, base_score: u32, power_attrs: &DevicePowerAttrs) -> u32 {
        self.coop_events.fetch_add(1, Ordering::Relaxed);

        let efficiency_factor = power_attrs.efficiency_score as u32;
        let thermal_penalty = if power_attrs.thermally_throttled { 30 } else { 0 };

        let adjusted = (base_score * efficiency_factor / 100).saturating_sub(thermal_penalty);

        if adjusted != base_score {
            self.decision_changes.fetch_add(1, Ordering::Relaxed);
        }

        adjusted.min(100)
    }

    /// Get statistics
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

    #[test]
    fn test_adjust_score_efficient() {
        let coop = BalancerPowerCoop::new();
        let attrs = DevicePowerAttrs {
            device_index: 0,
            efficiency_score: 80,
            power_mw: 3000,
            thermally_throttled: false,
        };
        let score = coop.adjust_score(70, &attrs);
        assert!(score > 0);
        assert!(score <= 100);
    }

    #[test]
    fn test_adjust_score_throttled() {
        let coop = BalancerPowerCoop::new();
        let attrs = DevicePowerAttrs {
            device_index: 0,
            efficiency_score: 50,
            power_mw: 5000,
            thermally_throttled: true,
        };
        let score = coop.adjust_score(70, &attrs);
        assert!(score < 70);
    }
}