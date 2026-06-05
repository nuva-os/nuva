/*
 * Nuva OS - Kernel - Sched - Nvbalancer - Stats
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
 * Nuva OS - Kernel - NvBalancer Statistics
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Comprehensive balancing statistics.
 */

use core::sync::atomic::{AtomicU64, AtomicU32, Ordering};

/// NvBalancerStats: comprehensive balancing statistics
pub struct NvBalancerStats {
    /// Balance cycles executed
    pub balance_cycles: AtomicU64,
    /// Migrations executed
    pub migrations_executed: AtomicU64,
    /// Oscillation events detected
    pub oscillation_detected: AtomicU64,
    /// Hot-plug events
    pub hotplug_events: AtomicU64,
    /// Average balance quality (0-100)
    pub avg_balance_quality: AtomicU32,
    /// Total migration overhead (microseconds)
    pub total_migration_overhead_us: AtomicU64,
}

impl NvBalancerStats {
    /// Create zero-initialized stats
    pub const fn new() -> Self {
        NvBalancerStats {
            balance_cycles: AtomicU64::new(0),
            migrations_executed: AtomicU64::new(0),
            oscillation_detected: AtomicU64::new(0),
            hotplug_events: AtomicU64::new(0),
            avg_balance_quality: AtomicU32::new(100),
            total_migration_overhead_us: AtomicU64::new(0),
        }
    }

    /// Record a balance cycle
    pub fn record_cycle(&self, quality: u32) {
        self.balance_cycles.fetch_add(1, Ordering::Relaxed);
        self.avg_balance_quality.store(quality.min(100), Ordering::Release);
    }

    /// Record a migration
    pub fn record_migration(&self, overhead_us: u32) {
        self.migrations_executed.fetch_add(1, Ordering::Relaxed);
        self.total_migration_overhead_us.fetch_add(overhead_us as u64, Ordering::Relaxed);
    }

    /// Record oscillation event
    pub fn record_oscillation(&self) {
        self.oscillation_detected.fetch_add(1, Ordering::Relaxed);
    }

    /// Record hot-plug event
    pub fn record_hotplug(&self) {
        self.hotplug_events.fetch_add(1, Ordering::Relaxed);
    }
}

/// Global NvBalancerStats instance
static NV_BALANCER_STATS: NvBalancerStats = NvBalancerStats::new();

/// Get global balancer statistics
pub fn get_nv_balancer_stats() -> &'static NvBalancerStats {
    &NV_BALANCER_STATS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_record() {
        let stats = NvBalancerStats::new();
        stats.record_cycle(85);
        stats.record_migration(200);
        stats.record_oscillation();
        stats.record_hotplug();

        assert_eq!(stats.balance_cycles.load(Ordering::Relaxed), 1);
        assert_eq!(stats.migrations_executed.load(Ordering::Relaxed), 1);
        assert_eq!(stats.oscillation_detected.load(Ordering::Relaxed), 1);
        assert_eq!(stats.hotplug_events.load(Ordering::Relaxed), 1);
        assert_eq!(stats.avg_balance_quality.load(Ordering::Relaxed), 85);
    }
}