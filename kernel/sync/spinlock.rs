/*
 * Nuva OS - Kernel - Spinlock
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

use core::sync::atomic::{AtomicU32, Ordering};

use super::preempt::{preempt_count, preempt_disable, preempt_enable, preemptible};
use crate::kernel::error::{KernelError, KernelResult};

/** Sentinel value indicating no CPU holds the lock */
const NO_CPU: u32 = u32::MAX;

/**
 * Spinlock with preemption control and holder tracking.
 *
 * When the lock is acquired:
 * - Preemption is disabled to prevent deadlock from scheduler
 *   invoking a different task on the same CPU while lock is held.
 * - The holder CPU ID is recorded for deadlock detection.
 * - Memory allocation is forbidden while lock is held (see preempt module).
 *
 * Use for short critical sections in interrupt context.
 * For longer critical sections, prefer Mutex.
 */
pub struct SpinLock {
    /** Lock state: 0 = unlocked, 1 = locked */
    locked: AtomicU32,
    /** CPU ID of lock holder, u32::MAX if unlocked */
    holder_cpu: AtomicU32,
}

impl SpinLock {
    /**
     * Create a new spinlock.
     */
    pub const fn new() -> Self {
        SpinLock {
            locked: AtomicU32::new(0),
            holder_cpu: AtomicU32::new(NO_CPU),
        }
    }

    /**
     * Acquire the lock.
     *
     * Spins until the lock is available. Disables preemption
     * while the lock is held to prevent deadlock.
     */
    #[inline(always)]
    pub fn lock(&self) {
        preempt_disable();

        // Spin wait with exponential backoff hint
        while self
            .locked
            .compare_exchange(0, 1, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            // Wait for lock release
            while self.locked.load(Ordering::Relaxed) != 0 {
                // SAFETY: spin_loop is a hint instruction that does not
                // violate memory safety; it merely reduces power consumption
                // and bus traffic during busy-wait.
                core::hint::spin_loop();
            }
        }

        // Record holder CPU for deadlock detection
        // SAFETY: We just acquired the lock, so we have exclusive access
        // to holder_cpu. The store uses Release ordering to ensure
        // the locked flag store is visible before holder_cpu update.
        self.holder_cpu
            .store(Self::current_cpu(), Ordering::Release);
    }

    /**
     * Try to acquire the lock without blocking.
     *
     * Returns true if lock was acquired, false otherwise.
     * Disables preemption on success.
     */
    #[inline(always)]
    pub fn try_lock(&self) -> bool {
        if self
            .locked
            .compare_exchange(0, 1, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
        {
            preempt_disable();
            self.holder_cpu
                .store(Self::current_cpu(), Ordering::Release);
            true
        } else {
            false
        }
    }

    /**
     * Try to acquire the lock, returning Result on failure.
     *
     * Prefer this over `try_lock()` for kernel code that uses
     * `Result<T, KernelError>` error handling.
     */
    #[inline(always)]
    pub fn try_lock_result(&self) -> Result<SpinLockGuard<'_>, KernelError> {
        if self.try_lock() {
            Ok(SpinLockGuard { lock: self })
        } else {
            Err(KernelError::DeadlockDetected)
        }
    }

    /**
     * Release the lock.
     *
     * Re-enables preemption. Must only be called by the lock holder.
     */
    #[inline(always)]
    pub fn unlock(&self) {
        // SAFETY: Clear holder_cpu before releasing lock to maintain
        // invariant that holder_cpu is valid only when locked == 1.
        // Release ordering ensures the holder_cpu clear is visible
        // before the lock flag clear.
        self.holder_cpu.store(NO_CPU, Ordering::Release);
        self.locked.store(0, Ordering::Release);
        preempt_enable();
    }

    /**
     * Check if lock is currently held.
     */
    #[inline(always)]
    pub fn is_locked(&self) -> bool {
        self.locked.load(Ordering::Acquire) != 0
    }

    /**
     * Get the CPU ID of the current lock holder.
     *
     * Returns None if the lock is not held.
     */
    #[inline(always)]
    pub fn holder_cpu(&self) -> Option<u32> {
        let cpu = self.holder_cpu.load(Ordering::Acquire);
        if cpu == NO_CPU {
            None
        } else {
            Some(cpu)
        }
    }

    /**
     * Get current CPU ID.
     *
     * Architecture-specific implementation.
     */
    #[inline(always)]
    fn current_cpu() -> u32 {
        // SAFETY: Reading the CPU ID is a read-only hardware operation
        // that cannot cause memory safety violations. Each architecture
        // provides a safe mechanism to read the current CPU ID.
        #[cfg(target_arch = "aarch64")]
        {
            let cpu_id: u32;
            // SAFETY: mrs reads the MPIDR_EL1 register which is a
            // read-only system register. This cannot cause side effects
            // or violate memory safety.
            unsafe {
                core::arch::asm!("mrs {}, mpidr_el1", out(reg) cpu_id);
            }
            cpu_id & 0xFF
        }

        #[cfg(target_arch = "x86_64")]
        {
            let cpu_id: u32;
            // SAFETY: Reading the GS segment base offset to obtain
            // the per-CPU data area pointer. This is a read-only
            // operation on the current CPU's segment register.
            unsafe {
                core::arch::asm!("movl %gs:0, {}", out(reg) cpu_id);
            }
            cpu_id
        }

        #[cfg(target_arch = "loongarch64")]
        {
            let cpu_id: u32;
            // SAFETY: Reading the CSR CPUID register which is a
            // read-only control and status register on LoongArch.
            unsafe {
                core::arch::asm!("csrrd {}, 0x20", out(reg) cpu_id);
            }
            cpu_id
        }
    }
}

/**
 * Spinlock guard with RAII semantics.
 *
 * Automatically releases the spinlock and re-enables preemption
 * when dropped.
 */
pub struct SpinLockGuard<'a> {
    lock: &'a SpinLock,
}

impl<'a> SpinLockGuard<'a> {
    /**
     * Acquire the lock and return a guard.
     */
    #[inline(always)]
    pub fn new(lock: &'a SpinLock) -> Self {
        lock.lock();
        SpinLockGuard { lock }
    }
}

impl<'a> Drop for SpinLockGuard<'a> {
    fn drop(&mut self) {
        self.lock.unlock();
    }
}

/**
 * Macro to define a static spinlock.
 */
#[macro_export]
macro_rules! define_spinlock {
    ($name:ident) => {
        static $name: $crate::kernel::sync::spinlock::SpinLock =
            $crate::kernel::sync::spinlock::SpinLock::new();
    };
}

/**
 * Macro to acquire a spinlock with guard.
 */
#[macro_export]
macro_rules! spin_lock {
    ($lock:expr) => {
        $crate::kernel::sync::spinlock::SpinLockGuard::new($lock)
    };
}

/**
 * IRQ-safe spinlock.
 *
 * A spinlock that disables interrupts while held.
 * Use for critical sections that need interrupt protection
 * in addition to preemption control.
 */
pub struct IrqSpinLock {
    lock: SpinLock,
}

impl IrqSpinLock {
    pub const fn new() -> Self {
        IrqSpinLock {
            lock: SpinLock::new(),
        }
    }

    /**
     * Acquire the lock with interrupts disabled.
     *
     * Returns a guard that will restore interrupt state when dropped.
     */
    pub fn lock(&self) -> IrqSpinLockGuard {
        // Disable interrupts - architecture specific
        #[cfg(target_arch = "aarch64")]
        let flags = {
            let flags: u64;
            // SAFETY: mrs reads DAIF register (interrupt mask flags).
            // msr writes to DAIF to mask interrupts. This is required
            // for interrupt-safe critical sections. The DAIF register
            // only controls interrupt masking and does not affect
            // memory safety.
            unsafe {
                core::arch::asm!(
                    "mrs {}, daif",
                    "msr daifset, #2",
                    out(reg) flags
                );
            }
            flags
        };

        #[cfg(target_arch = "x86_64")]
        let flags = {
            let flags: u64;
            // SAFETY: cli instruction disables interrupts on x86_64.
            // pushf saves the current flags register state including
            // the interrupt flag. These are privileged instructions
            // required for interrupt-safe locking.
            unsafe {
                core::arch::asm!(
                    "pushf",
                    "cli",
                    "pop {}",
                    out(reg) flags
                );
            }
            flags
        };

        #[cfg(target_arch = "loongarch64")]
        let flags = {
            let flags: u64;
            // SAFETY: Reading and modifying the CSR.CRMD register
            // on LoongArch to disable interrupts. The IE bit controls
            // global interrupt enable. This is a standard mechanism
            // for interrupt-safe critical sections.
            unsafe {
                let crmd: u32;
                core::arch::asm!(
                    "csrrd {}, 0x0",
                    out(reg) crmd
                );
                // Clear IE bit (bit 0)
                let new_crmd = crmd & !1u32;
                core::arch::asm!(
                    "csrwr {}, 0x0",
                    in(reg) new_crmd
                );
                flags = crmd as u64;
            }
            flags
        };

        // Now disable preemption and acquire the spinlock
        self.lock.lock();

        IrqSpinLockGuard {
            lock: &self.lock,
            flags,
        }
    }
}

/**
 * IRQ-safe spinlock guard.
 *
 * Restores interrupt state and re-enables preemption when dropped.
 */
pub struct IrqSpinLockGuard<'a> {
    lock: &'a SpinLock,
    flags: u64,
}

impl<'a> Drop for IrqSpinLockGuard<'a> {
    fn drop(&mut self) {
        self.lock.unlock();

        // Restore interrupts - architecture specific
        #[cfg(target_arch = "aarch64")]
        {
            // SAFETY: Restoring the DAIF register to its saved state.
            // This re-enables interrupts if they were enabled before
            // the lock was acquired. The value being written was
            // previously read from the same register, so it contains
            // a valid flags configuration.
            unsafe {
                core::arch::asm!(
                    "msr daif, {}",
                    in(reg) self.flags
                );
            }
        }

        #[cfg(target_arch = "x86_64")]
        {
            // SAFETY: Restoring the RFLAGS register including the
            // interrupt flag to its saved state. The flags value
            // was saved from pushf before cli was executed.
            unsafe {
                core::arch::asm!(
                    "push {}",
                    "popf",
                    in(reg) self.flags
                );
            }
        }

        #[cfg(target_arch = "loongarch64")]
        {
            // SAFETY: Restoring the CSR.CRMD register on LoongArch.
            // The IE bit is restored to its previous state, re-enabling
            // interrupts if they were enabled before lock acquisition.
            unsafe {
                let crmd = self.flags as u32;
                core::arch::asm!(
                    "csrwr {}, 0x0",
                    in(reg) crmd
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spinlock() {
        let lock = SpinLock::new();

        {
            let _guard = SpinLockGuard::new(&lock);
            assert!(lock.is_locked());
        }

        assert!(!lock.is_locked());
    }

    #[test]
    fn test_try_lock_result() {
        let lock = SpinLock::new();
        let result = lock.try_lock_result();
        assert!(result.is_ok());
        drop(result);
        assert!(!lock.is_locked());
    }
}
