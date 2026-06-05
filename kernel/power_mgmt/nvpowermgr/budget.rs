/*
 * Nuva OS - Kernel - PowerMgmt - Nvpowermgr - Budget
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
 * Nuva OS - Kernel - NvPowerMgr Power Budget Manager
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Manages system power budget with 5% overshoot allowance
 * and minimum-power fallback when budget is infeasible.
 */

use core::sync::atomic::{AtomicU32, AtomicBool, Ordering};

use crate::kernel::error::{KernelError, KernelResult};

/// Default overshoot allowance (5%)
pub const DEFAULT_OVERSHOOT_ALLOWANCE_PCT: u32 = 5;

/// PowerBudgetManager: system power budget management
pub struct PowerBudgetManager {
    /// Power budget in milliwatts
    budget_mw: AtomicU32,
    /// Current total power consumption in milliwatts
    current_total_mw: AtomicU32,
    /// Overshoot allowance percentage
    overshoot_allowance_pct: AtomicU32,
    /// Whether budget is feasible
    budget_feasible: AtomicBool,
    /// Whether running in minimum power mode
    min_power_mode: AtomicBool,
}

impl PowerBudgetManager {
    /// Create a new power budget manager
    pub const fn new() -> Self {
        PowerBudgetManager {
            budget_mw: AtomicU32::new(0),
            current_total_mw: AtomicU32::new(0),
            overshoot_allowance_pct: AtomicU32::new(DEFAULT_OVERSHOOT_ALLOWANCE_PCT),
            budget_feasible: AtomicBool::new(true),
            min_power_mode: AtomicBool::new(false),
        }
    }

    /// Set power budget
    pub fn set_budget(&self, budget_mw: u32) -> KernelResult<()> {
        if budget_mw == 0 {
            return Err(KernelError::InvalidArgument);
        }
        self.budget_mw.store(budget_mw, Ordering::Release);
        self.check_feasibility();
        Ok(())
    }

    /// Get current power budget
    #[inline(always)]
    pub fn budget_mw(&self) -> u32 {
        self.budget_mw.load(Ordering::Acquire)
    }

    /// Update current total power consumption
    pub fn update_current(&self, total_mw: u32) {
        self.current_total_mw.store(total_mw, Ordering::Release);
        self.check_feasibility();
    }

    /// Get current total power consumption
    #[inline(always)]
    pub fn current_total_mw(&self) -> u32 {
        self.current_total_mw.load(Ordering::Acquire)
    }

    /// Check if current consumption is within budget
    pub fn is_within_budget(&self) -> bool {
        let budget = self.budget_mw.load(Ordering::Acquire);
        let current = self.current_total_mw.load(Ordering::Acquire);
        let allowance = self.overshoot_allowance_pct.load(Ordering::Acquire);
        let limit = budget + (budget * allowance / 100);
        current <= limit
    }

    /// Check if budget is feasible
    #[inline(always)]
    pub fn is_feasible(&self) -> bool {
        self.budget_feasible.load(Ordering::Acquire)
    }

    /// Check if running in minimum power mode
    #[inline(always)]
    pub fn is_min_power_mode(&self) -> bool {
        self.min_power_mode.load(Ordering::Acquire)
    }

    /// Enter minimum power mode (when budget is infeasible)
    pub fn enter_min_power_mode(&self) {
        self.min_power_mode.store(true, Ordering::Release);
        self.budget_feasible.store(false, Ordering::Release);
    }

    /// Exit minimum power mode
    pub fn exit_min_power_mode(&self) {
        self.min_power_mode.store(false, Ordering::Release);
        self.budget_feasible.store(true, Ordering::Release);
    }

    /// Get budget utilization percentage
    pub fn utilization_pct(&self) -> u32 {
        let budget = self.budget_mw.load(Ordering::Acquire);
        if budget == 0 {
            return 0;
        }
        let current = self.current_total_mw.load(Ordering::Acquire);
        (current * 100) / budget
    }

    /// Get remaining budget in milliwatts
    pub fn remaining_mw(&self) -> u32 {
        let budget = self.budget_mw.load(Ordering::Acquire);
        let current = self.current_total_mw.load(Ordering::Acquire);
        budget.saturating_sub(current)
    }

    /// Internal feasibility check
    fn check_feasibility(&self) {
        let budget = self.budget_mw.load(Ordering::Acquire);
        if budget == 0 {
            return;
        }
        let current = self.current_total_mw.load(Ordering::Acquire);
        let allowance = self.overshoot_allowance_pct.load(Ordering::Acquire);
        let limit = budget + (budget * allowance / 100);
        let feasible = current <= limit;
        self.budget_feasible.store(feasible, Ordering::Release);
        if !feasible && !self.min_power_mode.load(Ordering::Acquire) {
            self.enter_min_power_mode();
        } else if feasible && self.min_power_mode.load(Ordering::Acquire) {
            self.exit_min_power_mode();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_within_budget() {
        let mgr = PowerBudgetManager::new();
        mgr.set_budget(10000).unwrap();
        mgr.update_current(8000);
        assert!(mgr.is_within_budget());
    }

    #[test]
    fn test_over_budget_with_allowance() {
        let mgr = PowerBudgetManager::new();
        mgr.set_budget(10000).unwrap();
        mgr.update_current(10499);
        assert!(mgr.is_within_budget());
    }

    #[test]
    fn test_over_budget_exceeds_allowance() {
        let mgr = PowerBudgetManager::new();
        mgr.set_budget(10000).unwrap();
        mgr.update_current(11000);
        assert!(!mgr.is_within_budget());
    }

    #[test]
    fn test_utilization() {
        let mgr = PowerBudgetManager::new();
        mgr.set_budget(10000).unwrap();
        mgr.update_current(7500);
        assert_eq!(mgr.utilization_pct(), 75);
    }

    #[test]
    fn test_remaining() {
        let mgr = PowerBudgetManager::new();
        mgr.set_budget(10000).unwrap();
        mgr.update_current(3000);
        assert_eq!(mgr.remaining_mw(), 7000);
    }
}