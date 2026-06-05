/*
 * Nuva OS - Kernel - Sched - Nvbalancer - MigrationEntry
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
 * Nuva OS - Kernel - NvBalancer Migration Entry
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Migration plan entries and balance decision output.
 */

/// MigrationEntry: a single task migration plan entry
#[derive(Clone, Debug)]
pub struct MigrationEntry {
    /// Task ID to migrate
    pub task_id: u32,
    /// Source device index
    pub source_device: usize,
    /// Target device index
    pub target_device: usize,
    /// Estimated migration overhead in microseconds
    pub estimated_overhead_us: u32,
}

impl MigrationEntry {
    /// Create a new migration entry
    pub const fn new(task_id: u32, source: usize, target: usize, overhead_us: u32) -> Self {
        MigrationEntry {
            task_id,
            source_device: source,
            target_device: target,
            estimated_overhead_us: overhead_us,
        }
    }
}

/// Device assignment entry
#[derive(Clone, Debug)]
pub struct DeviceAssignment {
    /// Task ID
    pub task_id: u32,
    /// Assigned device index
    pub device_index: usize,
}

/// BalanceDecision: complete balancing decision output
#[derive(Clone, Debug)]
pub struct BalanceDecision {
    /// Updated device assignments
    pub device_assignments: alloc::vec::Vec<DeviceAssignment>,
    /// Migration plan (ordered by priority)
    pub migration_plan: alloc::vec::Vec<MigrationEntry>,
    /// Current convergence step
    pub convergence_step: u32,
    /// Balance quality score (0-100, higher = better)
    pub balance_quality: u32,
    /// Confidence score (0-100)
    pub confidence: u8,
}

impl BalanceDecision {
    /// Create a balanced (no-migration) decision
    pub fn balanced(quality: u32) -> Self {
        BalanceDecision {
            device_assignments: alloc::vec::Vec::new(),
            migration_plan: alloc::vec::Vec::new(),
            convergence_step: 0,
            balance_quality: quality.min(100),
            confidence: 100,
        }
    }

    /// Check if any migrations are planned
    pub fn has_migrations(&self) -> bool {
        !self.migration_plan.is_empty()
    }

    /// Get total estimated migration overhead
    pub fn total_overhead_us(&self) -> u32 {
        self.migration_plan.iter().map(|m| m.estimated_overhead_us).sum()
    }

    /// Get number of migrations
    pub fn num_migrations(&self) -> usize {
        self.migration_plan.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_balanced_decision() {
        let d = BalanceDecision::balanced(85);
        assert!(!d.has_migrations());
        assert_eq!(d.balance_quality, 85);
        assert_eq!(d.confidence, 100);
    }

    #[test]
    fn test_migration_overhead() {
        let mut d = BalanceDecision::balanced(50);
        d.migration_plan.push(MigrationEntry::new(1, 0, 1, 200));
        d.migration_plan.push(MigrationEntry::new(2, 0, 2, 300));
        assert_eq!(d.total_overhead_us(), 500);
        assert_eq!(d.num_migrations(), 2);
    }
}