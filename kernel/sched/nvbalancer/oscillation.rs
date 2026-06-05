/*
 * Nuva OS - Kernel - Sched - Nvbalancer - Oscillation
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
 * Nuva OS - Kernel - NvBalancer Oscillation Detector
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Detects and suppresses task migration oscillation
 * using a ring buffer of recent migration history.
 * Prevents tasks from bouncing between devices.
 */

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// Ring buffer size for migration history
pub const OSCILLATION_BUFFER_SIZE: usize = 32;

/// Oscillation trigger threshold (same task migrates between
/// two devices >= 3 times)
pub const OSCILLATION_THRESHOLD: u32 = 3;

/// Migration history entry
#[derive(Clone, Copy, Debug)]
pub struct MigrationHistoryEntry {
    /// Task ID
    pub task_id: u32,
    /// Source device
    pub source: usize,
    /// Target device
    pub target: usize,
}

impl MigrationHistoryEntry {
    /// Create a new history entry
    pub const fn new(task_id: u32, source: usize, target: usize) -> Self {
        MigrationHistoryEntry { task_id, source, target }
    }
}

/// OscillationDetector: detects and suppresses oscillation
///
/// Maintains a ring buffer of recent migration events.
/// When the same task bounces between two devices
/// >= OSCILLATION_THRESHOLD times, it triggers suppression:
/// - Locks the task to its current device
/// - Increases suppression factor
pub struct OscillationDetector {
    /// Ring buffer of migration history
    buffer: [MigrationHistoryEntry; OSCILLATION_BUFFER_SIZE],
    /// Write position in ring buffer
    write_pos: AtomicU32,
    /// Number of entries in buffer
    count: AtomicU32,
    /// Oscillation events detected
    oscillation_events: AtomicU64,
    /// Currently suppressed task IDs (simplified: single task)
    suppressed_task: AtomicU32,
    /// Suppression factor (0 = none, higher = stronger)
    suppression_factor: AtomicU32,
}

impl OscillationDetector {
    /// Create a new oscillation detector
    pub const fn new() -> Self {
        OscillationDetector {
            buffer: [MigrationHistoryEntry::new(0, 0, 0); OSCILLATION_BUFFER_SIZE],
            write_pos: AtomicU32::new(0),
            count: AtomicU32::new(0),
            oscillation_events: AtomicU64::new(0),
            suppressed_task: AtomicU32::new(0),
            suppression_factor: AtomicU32::new(0),
        }
    }

    /// Record a migration event and check for oscillation
    ///
    /// @param task_id: Task being migrated
    /// @param source: Source device index
    /// @param target: Target device index
    /// @return: true if oscillation detected
    pub fn record_and_check(&mut self, task_id: u32, source: usize, target: usize) -> bool {
        let pos = self.write_pos.load(Ordering::Acquire) as usize % OSCILLATION_BUFFER_SIZE;
        self.buffer[pos] = MigrationHistoryEntry::new(task_id, source, target);
        self.write_pos.fetch_add(1, Ordering::Release);
        let count = self.count.load(Ordering::Acquire);
        self.count.store((count + 1).min(OSCILLATION_BUFFER_SIZE as u32), Ordering::Release);

        let is_oscillation = self.detect_oscillation(task_id, source, target);

        if is_oscillation {
            self.oscillation_events.fetch_add(1, Ordering::Relaxed);
            self.suppressed_task.store(task_id, Ordering::Release);
            self.suppression_factor.fetch_add(1, Ordering::Release);
        }

        is_oscillation
    }

    /// Check if a task is currently suppressed
    pub fn is_suppressed(&self, task_id: u32) -> bool {
        self.suppressed_task.load(Ordering::Acquire) == task_id && self.suppression_factor.load(Ordering::Acquire) > 0
    }

    /// Get suppression factor for a task
    pub fn suppression_factor(&self, task_id: u32) -> u32 {
        if self.is_suppressed(task_id) {
            self.suppression_factor.load(Ordering::Acquire)
        } else {
            0
        }
    }

    /// Clear suppression for a task (e.g., after cooldown)
    pub fn clear_suppression(&self, task_id: u32) {
        if self.suppressed_task.load(Ordering::Acquire) == task_id {
            self.suppressed_task.store(0, Ordering::Release);
            self.suppression_factor.store(0, Ordering::Release);
        }
    }

    /// Get oscillation event count
    pub fn oscillation_count(&self) -> u64 {
        self.oscillation_events.load(Ordering::Acquire)
    }

    /// Detect oscillation pattern in history
    fn detect_oscillation(&self, task_id: u32, source: usize, target: usize) -> bool {
        let mut bounce_count = 0u32;
        let count = self.count.load(Ordering::Acquire) as usize;
        let start = if count >= OSCILLATION_BUFFER_SIZE {
            self.write_pos.load(Ordering::Acquire) as usize % OSCILLATION_BUFFER_SIZE
        } else {
            0
        };

        for i in 0..count.min(OSCILLATION_BUFFER_SIZE) {
            let idx = (start + i) % OSCILLATION_BUFFER_SIZE;
            let entry = &self.buffer[idx];
            if entry.task_id == task_id {
                // Check for A->B followed by B->A pattern
                if (entry.source == source && entry.target == target) ||
                   (entry.source == target && entry.target == source) {
                    bounce_count += 1;
                }
            }
        }

        bounce_count >= OSCILLATION_THRESHOLD
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_oscillation_single_migration() {
        let mut det = OscillationDetector::new();
        let result = det.record_and_check(1, 0, 1);
        assert!(!result);
    }

    #[test]
    fn test_oscillation_detected() {
        let mut det = OscillationDetector::new();
        det.record_and_check(1, 0, 1);
        det.record_and_check(1, 1, 0);
        det.record_and_check(1, 0, 1);
        let result = det.record_and_check(1, 1, 0);
        assert!(result);
        assert!(det.is_suppressed(1));
    }

    #[test]
    fn test_suppression_factor() {
        let mut det = OscillationDetector::new();
        det.record_and_check(1, 0, 1);
        det.record_and_check(1, 1, 0);
        det.record_and_check(1, 0, 1);
        det.record_and_check(1, 1, 0);
        assert!(det.suppression_factor(1) > 0);
    }

    #[test]
    fn test_clear_suppression() {
        let mut det = OscillationDetector::new();
        det.record_and_check(1, 0, 1);
        det.record_and_check(1, 1, 0);
        det.record_and_check(1, 0, 1);
        det.record_and_check(1, 1, 0);
        det.clear_suppression(1);
        assert!(!det.is_suppressed(1));
    }
}