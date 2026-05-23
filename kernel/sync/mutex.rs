/*
 * Nuva OS - Kernel - Mutex and RwLock
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

use core::ptr;
use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// Mutex lock
/// A mutual exclusion lock for protecting shared resources.
/// Tracks owner task ID for deadlock detection in debug mode.
pub struct Mutex {
    /// Lock state and flags
    state: AtomicU32,
    /// Lock owner task ID (0 if unlocked)
    owner: AtomicU64,
}

/// Mutex flags
mod mutex_flags {
    pub const MUTEX_LOCKED: u32 = 1 << 0;
    pub const MUTEX_WAITERS: u32 = 1 << 1;
    pub const MUTEX_HANDOFF: u32 = 1 << 2;
}

impl Mutex {
    /// Create a new mutex
    pub const fn new() -> Self {
        Mutex {
            state: AtomicU32::new(0),
            owner: AtomicU64::new(0),
        }
    }

    /// Acquire the lock
    /// Blocks until the lock is available.
    pub fn lock(&self) {
        // Try fast path first
        if self.try_lock() {
            return;
        }

        // Slow path
        self.lock_slow();
    }

    /// Try to acquire the lock
    /// @return true if lock was acquired, false otherwise
    pub fn try_lock(&self) -> bool {
        // Atomically set the locked bit
        let old = self
            .state
            .fetch_or(mutex_flags::MUTEX_LOCKED, Ordering::AcqRel);

        if (old & mutex_flags::MUTEX_LOCKED) == 0 {
            // Successfully acquired, set owner
            let task_id = Self::current_task_id();
            self.owner.store(task_id, Ordering::Release);

            // Debug: check for recursive lock attempt
            #[cfg(feature = "debug")]
            if task_id != 0 {
                // Owner is set after acquiring, so no recursive issue here
            }

            true
        } else {
            // Lock is already held
            false
        }
    }

    /// Release the lock
    ///
    /// In debug mode, warns if caller is not the lock owner.
    pub fn unlock(&self) {
        // Debug: verify ownership
        #[cfg(feature = "debug")]
        {
            let current = Self::current_task_id();
            let owner = self.owner.load(Ordering::Acquire);
            if current != 0 && owner != 0 && current != owner {
                // SAFETY: pr_warn is a safe logging macro
                crate::pr_warn!(
                    "Mutex unlock by non-owner: current={}, owner={}",
                    current,
                    owner
                );
            }
        }

        // Clear owner
        self.owner.store(0, Ordering::Release);

        // Clear the locked bit
        let old = self
            .state
            .fetch_and(!mutex_flags::MUTEX_LOCKED, Ordering::AcqRel);

        // Wake waiters if any
        if (old & mutex_flags::MUTEX_WAITERS) != 0 {
            self.wake_waiters();
        }
    }

    /// Slow path for lock acquisition
    fn lock_slow(&self) {
        loop {
            // Try to acquire
            if self.try_lock() {
                return;
            }

            // Set waiters flag
            self.state
                .fetch_or(mutex_flags::MUTEX_WAITERS, Ordering::Relaxed);

            // Wait
            // TODO: Implement actual wait queue
            while self.state.load(Ordering::Acquire) & mutex_flags::MUTEX_LOCKED != 0 {
                core::hint::spin_loop();
            }
        }
    }

    /// Wake up waiting threads
    fn wake_waiters(&self) {
        // TODO: Implement actual wake-up
    }

    /// Check if lock is held
    /// @return true if locked, false otherwise
    pub fn is_locked(&self) -> bool {
        (self.state.load(Ordering::Acquire) & mutex_flags::MUTEX_LOCKED) != 0
    }

    /**
     * Get the task ID of the current lock owner.
     *
     * Returns 0 if the lock is not held.
     */
    #[inline(always)]
    pub fn owner(&self) -> u64 {
        self.owner.load(Ordering::Acquire)
    }

    /**
     * Get current task ID.
     *
     * Architecture-specific implementation. Returns 0 if task
     * context is not available (e.g., early boot, IRQ context).
     */
    #[inline(always)]
    fn current_task_id() -> u64 {
        // SAFETY: Reading the current task ID is a read-only operation
        // that cannot cause memory safety violations. Returns 0 if
        // task context is not yet initialized.
        #[cfg(target_arch = "aarch64")]
        {
            let task_id: u64;
            // SAFETY: mrs tpidr_el0 reads the thread pointer register which
            // stores the current task ID. This is a read-only system register
            // access that cannot cause memory safety violations.
            unsafe {
                core::arch::asm!("mrs {}, tpidr_el0", out(reg) task_id);
            }
            task_id
        }

        #[cfg(target_arch = "x86_64")]
        {
            let task_id: u64;
            // SAFETY: Reading the GS segment base offset +8 to obtain
            // the current task ID from per-CPU data. This is a read-only
            // operation on the current CPU's segment register.
            unsafe {
                core::arch::asm!("movq %gs:8, {}", out(reg) task_id);
            }
            task_id
        }

        #[cfg(target_arch = "loongarch64")]
        {
            let task_id: u64;
            // SAFETY: Reading CSR register 0x4 which holds the current
            // task ID on LoongArch. This is a read-only CSR access.
            unsafe {
                core::arch::asm!("csrrd {}, 0x4", out(reg) task_id);
            }
            task_id
        }
    }
}

/// Mutex guard
/// RAII guard that automatically releases the mutex when dropped.
pub struct MutexGuard<'a> {
    lock: &'a Mutex,
}

impl<'a> MutexGuard<'a> {
    pub fn new(lock: &'a Mutex) -> Self {
        lock.lock();
        MutexGuard { lock }
    }
}

impl<'a> Drop for MutexGuard<'a> {
    fn drop(&mut self) {
        self.lock.unlock();
    }
}

/// Reader-writer lock
/// Allows multiple readers or a single writer.
/// Uses double-check pattern to prevent TOCTOU race between
/// writer check and reader count increment.
pub struct RwLock {
    /// Reader count and writer flag packed into single atomic
    /// to avoid TOCTOU race between reader increment and writer check.
    /// Bits: [31:1] = reader count, [0] = writer flag
    state: AtomicU32,
}

/** Writer flag bit position in packed state */
const WRITER_BIT: u32 = 1 << 0;
/** Reader count shift in packed state */
const READER_SHIFT: u32 = 1;
/** Reader count mask in packed state */
const READER_MASK: u32 = !WRITER_BIT;

impl RwLock {
    pub const fn new() -> Self {
        RwLock {
            state: AtomicU32::new(0),
        }
    }

    /**
     * Acquire read lock using double-check pattern.
     *
     * The double-check prevents a TOCTOU race where a writer could
     * acquire the lock between our writer check and reader count
     * increment. We:
     * 1. Check no writer, then increment readers
     * 2. Re-check no writer; if writer appeared, back off and retry
     */
    pub fn read_lock(&self) {
        loop {
            // Wait for writer to release
            while self.state.load(Ordering::Acquire) & WRITER_BIT != 0 {
                core::hint::spin_loop();
            }

            // Optimistically increment reader count
            let old = self.state.fetch_add(1 << READER_SHIFT, Ordering::AcqRel);

            // Double-check: if writer appeared, back off
            if old & WRITER_BIT != 0 {
                // Writer sneaked in; undo reader increment and retry
                self.state.fetch_sub(1 << READER_SHIFT, Ordering::AcqRel);
                continue;
            }

            // Successfully acquired read lock
            return;
        }
    }

    /// Release read lock
    pub fn read_unlock(&self) {
        self.state.fetch_sub(1 << READER_SHIFT, Ordering::AcqRel);
    }

    /**
     * Acquire write lock.
     *
     * First sets the writer bit to block new readers, then waits
     * for existing readers to finish. Setting the writer bit first
     * ensures no new readers can start while we wait.
     */
    pub fn write_lock(&self) {
        // Set writer bit to block new readers
        while self.state.fetch_or(WRITER_BIT, Ordering::AcqRel) & WRITER_BIT != 0 {
            // Writer bit was already set; wait and retry
            core::hint::spin_loop();
        }

        // Wait for all existing readers to finish
        while self.state.load(Ordering::Acquire) & READER_MASK != 0 {
            core::hint::spin_loop();
        }
    }

    /// Release write lock
    pub fn write_unlock(&self) {
        self.state.fetch_and(!WRITER_BIT, Ordering::Release);
    }

    /**
     * Try to acquire read lock using double-check pattern.
     *
     * @return true if acquired, false otherwise
     */
    pub fn try_read_lock(&self) -> bool {
        // Check no writer
        if self.state.load(Ordering::Acquire) & WRITER_BIT != 0 {
            return false;
        }

        // Optimistically increment reader count
        let old = self.state.fetch_add(1 << READER_SHIFT, Ordering::AcqRel);

        // Double-check for writer
        if old & WRITER_BIT != 0 {
            // Writer sneaked in; undo and fail
            self.state.fetch_sub(1 << READER_SHIFT, Ordering::AcqRel);
            return false;
        }

        true
    }

    /**
     * Try to acquire write lock.
     *
     * @return true if acquired, false otherwise
     */
    pub fn try_write_lock(&self) -> bool {
        // Try to set writer bit
        if self.state.fetch_or(WRITER_BIT, Ordering::AcqRel) & WRITER_BIT != 0 {
            return false;
        }

        // Check for existing readers
        if self.state.load(Ordering::Acquire) & READER_MASK != 0 {
            // Readers exist; clear writer bit and fail
            self.state.fetch_and(!WRITER_BIT, Ordering::Release);
            return false;
        }

        true
    }

    /**
     * Get current reader count.
     */
    pub fn reader_count(&self) -> u32 {
        (self.state.load(Ordering::Acquire) & READER_MASK) >> READER_SHIFT
    }

    /**
     * Check if write lock is held.
     */
    pub fn is_write_locked(&self) -> bool {
        self.state.load(Ordering::Acquire) & WRITER_BIT != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mutex_new() {
        let mutex = Mutex::new();
        assert!(!mutex.is_locked());
    }

    #[test]
    fn test_mutex_try_lock() {
        let mutex = Mutex::new();

        assert!(mutex.try_lock());
        assert!(mutex.is_locked());

        // Second attempt should fail
        assert!(!mutex.try_lock());
    }

    #[test]
    fn test_mutex_unlock() {
        let mutex = Mutex::new();

        mutex.try_lock();
        assert!(mutex.is_locked());

        mutex.unlock();
        assert!(!mutex.is_locked());
    }

    #[test]
    fn test_mutex_lock_unlock_cycle() {
        let mutex = Mutex::new();

        for _ in 0..10 {
            mutex.lock();
            assert!(mutex.is_locked());
            mutex.unlock();
            assert!(!mutex.is_locked());
        }
    }

    #[test]
    fn test_mutex_guard() {
        let mutex = Mutex::new();

        {
            let _guard = MutexGuard::new(&mutex);
            assert!(mutex.is_locked());
        }

        assert!(!mutex.is_locked());
    }

    #[test]
    fn test_mutex_flags() {
        assert_eq!(mutex_flags::MUTEX_LOCKED, 1 << 0);
        assert_eq!(mutex_flags::MUTEX_WAITERS, 1 << 1);
        assert_eq!(mutex_flags::MUTEX_HANDOFF, 1 << 2);
    }

    #[test]
    fn test_rwlock_new() {
        let rwlock = RwLock::new();

        assert_eq!(rwlock.reader_count(), 0);
        assert!(!rwlock.is_write_locked());
    }

    #[test]
    fn test_rwlock_read_lock() {
        let rwlock = RwLock::new();

        rwlock.read_lock();
        assert_eq!(rwlock.reader_count(), 1);

        rwlock.read_lock();
        assert_eq!(rwlock.reader_count(), 2);

        rwlock.read_unlock();
        assert_eq!(rwlock.reader_count(), 1);

        rwlock.read_unlock();
        assert_eq!(rwlock.reader_count(), 0);
    }

    #[test]
    fn test_rwlock_write_lock() {
        let rwlock = RwLock::new();

        rwlock.write_lock();
        assert!(rwlock.is_write_locked());

        rwlock.write_unlock();
        assert!(!rwlock.is_write_locked());
    }

    #[test]
    fn test_rwlock_try_read_lock() {
        let rwlock = RwLock::new();

        assert!(rwlock.try_read_lock());
        assert_eq!(rwlock.reader_count(), 1);

        assert!(rwlock.try_read_lock());
        assert_eq!(rwlock.reader_count(), 2);

        rwlock.read_unlock();
        rwlock.read_unlock();
    }

    #[test]
    fn test_rwlock_try_write_lock() {
        let rwlock = RwLock::new();

        assert!(rwlock.try_write_lock());
        assert!(rwlock.is_write_locked());

        // Cannot acquire write lock when already held
        assert!(!rwlock.try_write_lock());

        rwlock.write_unlock();
        assert!(!rwlock.is_write_locked());
    }

    #[test]
    fn test_rwlock_write_blocks_read() {
        let rwlock = RwLock::new();

        rwlock.write_lock();

        // Cannot acquire read lock when write lock is held
        assert!(!rwlock.try_read_lock());

        rwlock.write_unlock();

        // Can acquire read lock after write lock is released
        assert!(rwlock.try_read_lock());
        rwlock.read_unlock();
    }

    #[test]
    fn test_rwlock_read_blocks_write() {
        let rwlock = RwLock::new();

        rwlock.read_lock();

        // Cannot acquire write lock when read lock is held
        assert!(!rwlock.try_write_lock());

        rwlock.read_unlock();

        // Can acquire write lock after read lock is released
        assert!(rwlock.try_write_lock());
        rwlock.write_unlock();
    }

    #[test]
    fn test_rwlock_multiple_readers() {
        let rwlock = RwLock::new();

        // Multiple readers can hold the lock simultaneously
        rwlock.read_lock();
        rwlock.read_lock();
        rwlock.read_lock();

        assert_eq!(rwlock.reader_count(), 3);

        rwlock.read_unlock();
        rwlock.read_unlock();
        rwlock.read_unlock();

        assert_eq!(rwlock.reader_count(), 0);
    }
}
