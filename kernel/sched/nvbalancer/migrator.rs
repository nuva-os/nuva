/*
 * Nuva OS - Kernel - Sched - Nvbalancer - Migrator
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
 * Nuva OS - Kernel - NvBalancer Migration Executor
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Executes task migrations between heterogeneous devices
 * with checkpoint-save, pause, migrate, and resume sequence.
 */

use core::sync::atomic::{AtomicU64, AtomicU32, Ordering};

use super::migration_entry::MigrationEntry;

/// Maximum allowed migration overhead ratio (15%)
pub const MAX_MIGRATION_OVERHEAD_RATIO: u32 = 15;

/// MigrationExecutor: executes task migrations
///
/// Migration sequence:
/// 1. Checkpoint save (task state)
/// 2. Pause task execution
/// 3. Migrate to target device
/// 4. Resume execution on target
///
/// Validates that migration overhead does not exceed
/// 15% of original task execution time.
pub struct MigrationExecutor {
    /// Total migrations executed
    total_migrations: AtomicU64,
    /// Total migration overhead (microseconds)
    total_overhead_us: AtomicU64,
    /// Failed migrations
    failed_migrations: AtomicU64,
    /// Peak migration overhead (microseconds)
    peak_overhead_us: AtomicU32,
}

impl MigrationExecutor {
    /// Create a new migration executor
    pub const fn new() -> Self {
        MigrationExecutor {
            total_migrations: AtomicU64::new(0),
            total_overhead_us: AtomicU64::new(0),
            failed_migrations: AtomicU64::new(0),
            peak_overhead_us: AtomicU32::new(0),
        }
    }

    /// Execute a single migration
    ///
    /// @param entry: Migration entry to execute
    /// @param task_execution_time_us: Original task execution time
    /// @return: Ok(overhead_us) or Err if overhead exceeds limit
    pub fn execute(&self, entry: &MigrationEntry, task_execution_time_us: u32) -> Result<u32, ()> {
        // Validate overhead constraint
        if task_execution_time_us > 0 {
            let overhead_ratio = (entry.estimated_overhead_us * 100) / task_execution_time_us;
            if overhead_ratio > MAX_MIGRATION_OVERHEAD_RATIO {
                self.failed_migrations.fetch_add(1, Ordering::Relaxed);
                return Err(());
            }
        }

        // TODO: Actual migration sequence:
        // 1. checkpoint_save(task_id)
        // 2. task_pause(task_id)
        // 3. migrate(task_id, source -> target)
        // 4. task_resume(task_id)

        let overhead = entry.estimated_overhead_us;
        self.total_migrations.fetch_add(1, Ordering::Relaxed);
        self.total_overhead_us.fetch_add(overhead as u64, Ordering::Relaxed);

        let current_peak = self.peak_overhead_us.load(Ordering::Acquire);
        if overhead > current_peak {
            self.peak_overhead_us.store(overhead, Ordering::Release);
        }

        Ok(overhead)
    }

    /// Execute multiple migrations in order
    ///
    /// @param entries: Migration entries to execute
    /// @param task_execution_time_us: Base execution time for overhead check
    /// @return: Number of successful migrations
    pub fn execute_batch(&self, entries: &[MigrationEntry], task_execution_time_us: u32) -> usize {
        let mut success = 0;
        for entry in entries {
            if self.execute(entry, task_execution_time_us).is_ok() {
                success += 1;
            }
        }
        success
    }

    /// Get migration statistics
    pub fn stats(&self) -> (u64, u64, u64, u32) {
        (
            self.total_migrations.load(Ordering::Acquire),
            self.total_overhead_us.load(Ordering::Acquire),
            self.failed_migrations.load(Ordering::Acquire),
            self.peak_overhead_us.load(Ordering::Acquire),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_success() {
        let exec = MigrationExecutor::new();
        let entry = MigrationEntry::new(1, 0, 1, 100);
        let result = exec.execute(&entry, 1000);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 100);

        let (total, _, _, _) = exec.stats();
        assert_eq!(total, 1);
    }

    #[test]
    fn test_execute_overhead_exceeded() {
        let exec = MigrationExecutor::new();
        let entry = MigrationEntry::new(1, 0, 1, 200);
        let result = exec.execute(&entry, 100);
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_batch() {
        let exec = MigrationExecutor::new();
        let entries = [
            MigrationEntry::new(1, 0, 1, 50),
            MigrationEntry::new(2, 0, 2, 100),
            MigrationEntry::new(3, 1, 2, 30),
        ];
        let success = exec.execute_batch(&entries, 1000);
        assert_eq!(success, 3);
    }
}