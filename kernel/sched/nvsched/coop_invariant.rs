/*
 * Nuva OS - Kernel - Sched - Nvsched - CoopInvariant
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
 * Nuva OS - Kernel - Cooperation Invariant Verification
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Runtime verification of three-party cooperation invariants:
 * 1. Scheduling decisions consider power impact
 * 2. Power optimization considers scheduling needs
 * 3. Balance decisions are scheduler-driven
 * 4. AI fallback: performance degradation <= 10% and energy reduction >= 15%
 */

use core::sync::atomic::{AtomicU64, AtomicBool, Ordering};

/// Cooperation invariant ID
#[derive(Debug, Clone, Copy)]
pub enum CoopInvariant {
    /// Scheduling considers power impact
    SchedConsidersPower = 0,
    /// Power optimization considers scheduling
    PowerConsidersSched = 1,
    /// Balance is scheduler-driven
    BalanceSchedDriven = 2,
    /// AI fallback performance constraint
    AiFallbackPerfConstraint = 3,
}

/// Invariant check result
#[derive(Debug, Clone, Copy)]
pub struct InvariantResult {
    /// Which invariant was checked
    pub invariant: CoopInvariant,
    /// Whether invariant holds
    pub holds: bool,
    /// Measured value (context-dependent)
    pub measured_value: u32,
    /// Expected threshold
    pub threshold: u32,
}

/// CoopInvariantChecker: runtime cooperation invariant verification
pub struct CoopInvariantChecker {
    /// Total checks performed
    total_checks: AtomicU64,
    /// Violations detected
    violations: AtomicU64,
    /// Whether all invariants hold
    all_hold: AtomicBool,
}

impl CoopInvariantChecker {
    /// Create a new invariant checker
    pub const fn new() -> Self {
        CoopInvariantChecker {
            total_checks: AtomicU64::new(0),
            violations: AtomicU64::new(0),
            all_hold: AtomicBool::new(true),
        }
    }

    /// Check invariant: scheduling considers power impact
    pub fn check_sched_considers_power(&self, power_efficiency_considered: bool) -> InvariantResult {
        let result = InvariantResult {
            invariant: CoopInvariant::SchedConsidersPower,
            holds: power_efficiency_considered,
            measured_value: if power_efficiency_considered { 1 } else { 0 },
            threshold: 1,
        };
        self.record(result.holds);
        result
    }

    /// Check invariant: power optimization considers scheduling needs
    pub fn check_power_considers_sched(&self, active_tasks_not_slept: bool) -> InvariantResult {
        let result = InvariantResult {
            invariant: CoopInvariant::PowerConsidersSched,
            holds: active_tasks_not_slept,
            measured_value: if active_tasks_not_slept { 1 } else { 0 },
            threshold: 1,
        };
        self.record(result.holds);
        result
    }

    /// Check invariant: balance is scheduler-driven
    pub fn check_balance_sched_driven(&self, balance_triggered_by_sched: bool) -> InvariantResult {
        let result = InvariantResult {
            invariant: CoopInvariant::BalanceSchedDriven,
            holds: balance_triggered_by_sched,
            measured_value: if balance_triggered_by_sched { 1 } else { 0 },
            threshold: 1,
        };
        self.record(result.holds);
        result
    }

    /// Check invariant: AI fallback performance constraint
    ///
    /// @param perf_degradation_pct: Actual performance degradation (0-100)
    /// @param energy_reduction_pct: Actual energy reduction (0-100)
    pub fn check_ai_fallback_constraint(&self, perf_degradation_pct: u32, energy_reduction_pct: u32) -> InvariantResult {
        let holds = perf_degradation_pct <= 10 && energy_reduction_pct >= 15;
        let result = InvariantResult {
            invariant: CoopInvariant::AiFallbackPerfConstraint,
            holds,
            measured_value: perf_degradation_pct,
            threshold: 10,
        };
        self.record(result.holds);
        result
    }

    /// Record a check result
    fn record(&self, holds: bool) {
        self.total_checks.fetch_add(1, Ordering::Relaxed);
        if !holds {
            self.violations.fetch_add(1, Ordering::Relaxed);
            self.all_hold.store(false, Ordering::Release);
        }
    }

    /// Check if all invariants hold
    pub fn all_hold(&self) -> bool {
        self.all_hold.load(Ordering::Acquire)
    }

    /// Get statistics
    pub fn stats(&self) -> (u64, u64) {
        (
            self.total_checks.load(Ordering::Acquire),
            self.violations.load(Ordering::Acquire),
        )
    }

    /// Reset all_hold flag (for periodic re-check)
    pub fn reset(&self) {
        self.all_hold.store(true, Ordering::Release);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sched_considers_power_holds() {
        let checker = CoopInvariantChecker::new();
        let result = checker.check_sched_considers_power(true);
        assert!(result.holds);
        assert!(checker.all_hold());
    }

    #[test]
    fn test_sched_considers_power_violates() {
        let checker = CoopInvariantChecker::new();
        let result = checker.check_sched_considers_power(false);
        assert!(!result.holds);
        assert!(!checker.all_hold());
    }

    #[test]
    fn test_ai_fallback_constraint_holds() {
        let checker = CoopInvariantChecker::new();
        let result = checker.check_ai_fallback_constraint(8, 20);
        assert!(result.holds);
    }

    #[test]
    fn test_ai_fallback_constraint_violates_perf() {
        let checker = CoopInvariantChecker::new();
        let result = checker.check_ai_fallback_constraint(15, 20);
        assert!(!result.holds);
    }

    #[test]
    fn test_ai_fallback_constraint_violates_energy() {
        let checker = CoopInvariantChecker::new();
        let result = checker.check_ai_fallback_constraint(8, 10);
        assert!(!result.holds);
    }
}