/*
 * Nuva OS - Kernel - Sched - Nvsched - BalancerCoop
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
 * Nuva OS - Kernel - NvScheduler-Balancer Cooperation
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * NvScheduler drives NvBalancer: scheduling decisions
 * trigger load balancing when imbalance is detected.
 */

use core::sync::atomic::{AtomicU64, Ordering};

/// Balancer cooperation event
#[derive(Debug, Clone, Copy)]
pub struct BalancerCoopEvent {
    /// Scheduling decision ID that triggered balance
    pub decision_id: u64,
    /// Whether balance was triggered
    pub balance_triggered: bool,
    /// Load deviation at trigger time (percentage)
    pub load_deviation_pct: u32,
}

/// SchedBalancerCoop: scheduler-driven balancing
///
/// NvScheduler drives NvBalancer: when scheduling decisions
/// detect load imbalance, NvBalancer::request_balance()
/// is called. Balance is triggered by AI inference or
/// declarative policy, not fixed thresholds.
pub struct SchedBalancerCoop {
    /// Balance requests from scheduler
    balance_requests: AtomicU64,
    /// Balance executions completed
    balance_executions: AtomicU64,
}

impl SchedBalancerCoop {
    /// Create a new scheduler-balancer cooperation
    pub const fn new() -> Self {
        SchedBalancerCoop {
            balance_requests: AtomicU64::new(0),
            balance_executions: AtomicU64::new(0),
        }
    }

    /// Request balance from scheduler
    ///
    /// @param decision_id: Current scheduling decision ID
    /// @param max_load: Maximum device load (0-100)
    /// @param min_load: Minimum device load (0-100)
    /// @param trigger_pct: Imbalance trigger threshold
    /// @return: Cooperation event
    pub fn request_balance(
        &self,
        decision_id: u64,
        max_load: u32,
        min_load: u32,
        trigger_pct: u32,
    ) -> BalancerCoopEvent {
        let deviation = if max_load > 0 {
            ((max_load - min_load) * 100) / max_load
        } else {
            0
        };

        let triggered = deviation >= trigger_pct;

        if triggered {
            self.balance_requests.fetch_add(1, Ordering::Relaxed);
        }

        BalancerCoopEvent {
            decision_id,
            balance_triggered: triggered,
            load_deviation_pct: deviation,
        }
    }

    /// Record balance execution completion
    pub fn record_balance_execution(&self) {
        self.balance_executions.fetch_add(1, Ordering::Relaxed);
    }

    /// Get statistics
    pub fn stats(&self) -> (u64, u64) {
        (
            self.balance_requests.load(Ordering::Acquire),
            self.balance_executions.load(Ordering::Acquire),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_imbalance() {
        let coop = SchedBalancerCoop::new();
        let event = coop.request_balance(1, 50, 45, 30);
        assert!(!event.balance_triggered);
    }

    #[test]
    fn test_imbalance_triggered() {
        let coop = SchedBalancerCoop::new();
        let event = coop.request_balance(1, 80, 20, 30);
        assert!(event.balance_triggered);
    }

    #[test]
    fn test_stats() {
        let coop = SchedBalancerCoop::new();
        coop.request_balance(1, 80, 20, 30);
        coop.record_balance_execution();
        let (reqs, execs) = coop.stats();
        assert_eq!(reqs, 1);
        assert_eq!(execs, 1);
    }
}