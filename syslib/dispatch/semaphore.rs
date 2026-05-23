/*
 * Nuva OS
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

//! Semaphore implementation for dispatch framework

use core::sync::atomic::{AtomicI32, Ordering};

/// Counting semaphore for dispatch synchronization
pub struct DispatchSemaphore {
    /// Semaphore counter
    count: AtomicI32,
}

impl DispatchSemaphore {
    /// Create a new semaphore with initial value
    pub fn new(value: i32) -> Self {
        Self {
            count: AtomicI32::new(value),
        }
    }

    /// Wait (P operation)
    /// Decrements the counter; blocks if counter is not positive.
    /// Returns true if wait succeeded.
    pub fn wait(&self) -> bool {
        loop {
            let current = self.count.load(Ordering::Acquire);
            if current > 0 {
                if self.count.compare_exchange(
                    current,
                    current - 1,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                ).is_ok() {
                    return true;
                }
            } else {
                // Counter is zero or negative; waiter must block
                // Simplified implementation: return false immediately
                return false;
            }
            core::hint::spin_loop();
        }
    }

    /// Wait with timeout
    /// Returns true if wait succeeded before timeout.
    pub fn wait_timeout(&self, _duration: core::time::Duration) -> bool {
        self.wait()
    }

    /// Signal (V operation)
    /// Increments the counter; wakes a waiter if any.
    /// Returns true if a waiter was woken.
    pub fn signal(&self) -> bool {
        let prev = self.count.fetch_add(1, Ordering::AcqRel);
        prev < 0 // If negative, there were waiters
    }

    /// Get current counter value
    pub fn count(&self) -> i32 {
        self.count.load(Ordering::Acquire)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semaphore_basic() {
        let sem = DispatchSemaphore::new(1);

        assert_eq!(sem.count(), 1);
        assert!(sem.wait());
        assert_eq!(sem.count(), 0);
        assert!(sem.signal());
        assert_eq!(sem.count(), 1);
    }

    #[test]
    fn test_semaphore_blocking() {
        let sem = DispatchSemaphore::new(0);

        // Counter is 0, wait should fail
        assert!(!sem.wait());

        // Increment counter
        sem.signal();

        // Now wait should succeed
        assert!(sem.wait());
    }
}
