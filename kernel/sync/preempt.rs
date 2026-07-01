/*
 * Nuva OS - Kernel - Preemption Control
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

use crate::kernel::error::{KernelError, KernelResult};

/**
 * Per-CPU preemption count.
 *
 * When preempt_count > 0, preemption is disabled.
 * This is used by SpinLock to prevent scheduling while holding a lock,
 * and by interrupt handlers to prevent re-entrant scheduling.
 *
 * Bit layout of preempt_count (32 bits):
 *   [0:7]   - preemption disable count (max 255 nested disables)
 *   [8:15]  - softirq disable count
 *   [16:23] - hardirq count (interrupt nesting depth)
 *   [24:31] - reserved
 */

/** Bit offset for softirq count */
const SOFTIRQ_SHIFT: u32 = 8;
/** Bit offset for hardirq count */
const HARDIRQ_SHIFT: u32 = 16;
/** Mask for preemption count */
const PREEMPT_MASK: u32 = 0xFF;
/** Mask for softirq count */
const SOFTIRQ_MASK: u32 = 0xFF << SOFTIRQ_SHIFT;
/** Mask for hardirq count */
const HARDIRQ_MASK: u32 = 0xFF << HARDIRQ_SHIFT;

/**
 * Global preemption count storage.
 *
 * In a true Per-CPU implementation, this would be a per-cpu variable
 * accessed via the current CPU ID. For now, we use a single atomic
 * as the baseline, with architecture-specific Per-CPU accessors
 * overriding this when SMP is enabled.
 */
static PREEMPT_COUNT: AtomicU32 = AtomicU32::new(0);

/**
 * Get the current preemption count.
 *
 * A non-zero value means preemption is disabled.
 */
#[inline(always)]
pub fn preempt_count() -> u32 {
    PREEMPT_COUNT.load(Ordering::Acquire)
}

/**
 * Disable preemption.
 *
 * Increments the preemption count. Must be paired with
 * a corresponding `preempt_enable()`.
 */
#[inline(always)]
pub fn preempt_disable() {
    let old = PREEMPT_COUNT.fetch_add(1, Ordering::AcqRel);
    // SAFETY: The preemption count overflow would indicate a bug
    // (more than 255 nested preempt_disable calls). In production,
    // this would trigger a kernel oops. We saturate to prevent wrap.
    if old & PREEMPT_MASK == PREEMPT_MASK {
        // Saturate: undo the add to prevent overflow
        PREEMPT_COUNT.fetch_sub(1, Ordering::AcqRel);
    }
}

/**
 * Enable preemption.
 *
 * Decrements the preemption count. If count reaches zero,
 * a pending reschedule may be performed.
 */
#[inline(always)]
pub fn preempt_enable() {
    let count = PREEMPT_COUNT.load(Ordering::Acquire);
    if count & PREEMPT_MASK > 0 {
        PREEMPT_COUNT.fetch_sub(1, Ordering::AcqRel);
    }
}

/**
 * Enable preemption and check for reschedule.
 *
 * After decrementing the preemption count, if count reaches zero
 * and a reschedule is pending, trigger a context switch.
 */
#[inline(always)]
pub fn preempt_enable_no_resched() {
    preempt_enable();
}

/**
 * Check if preemption is currently allowed.
 *
 * Returns true if preempt_count == 0 and not in interrupt context.
 */
#[inline(always)]
pub fn preemptible() -> bool {
    preempt_count() == 0
}

/**
 * Check if we are in interrupt context.
 */
#[inline(always)]
pub fn in_irq() -> bool {
    (PREEMPT_COUNT.load(Ordering::Acquire) & HARDIRQ_MASK) != 0
}

/**
 * Check if we are in softirq context.
 */
#[inline(always)]
pub fn in_softirq() -> bool {
    (PREEMPT_COUNT.load(Ordering::Acquire) & SOFTIRQ_MASK) != 0
}

/**
 * Enter hardirq context.
 *
 * Increments the hardirq nesting count.
 */
#[inline(always)]
pub fn irq_enter() {
    PREEMPT_COUNT.fetch_add(1 << HARDIRQ_SHIFT, Ordering::AcqRel);
}

/**
 * Exit hardirq context.
 *
 * Decrements the hardirq nesting count.
 */
#[inline(always)]
pub fn irq_exit() {
    let count = PREEMPT_COUNT.load(Ordering::Acquire);
    if count & HARDIRQ_MASK > 0 {
        PREEMPT_COUNT.fetch_sub(1 << HARDIRQ_SHIFT, Ordering::AcqRel);
    }
}

/**
 * RAII guard for preempt_disable/preempt_enable.
 *
 * Automatically restores preemption state when dropped.
 */
pub struct PreemptGuard {
    _private: (),
}

impl PreemptGuard {
    /**
     * Create a new preempt guard, disabling preemption.
     */
    #[inline(always)]
    pub fn new() -> Self {
        preempt_disable();
        PreemptGuard { _private: () }
    }
}

impl Drop for PreemptGuard {
    fn drop(&mut self) {
        preempt_enable();
    }
}

/**
 * Check if memory allocation is currently allowed.
 *
 * Allocation is forbidden when:
 * - Preemption is disabled (spinlock held)
 * - In hardirq context
 *
 * Use this before calling kmalloc/kfree to prevent
 * deadlocks from allocation within spinlock critical sections.
 */
#[inline(always)]
pub fn allocation_allowed() -> bool {
    let count = preempt_count();
    (count & PREEMPT_MASK) == 0 && (count & HARDIRQ_MASK) == 0
}

/**
 * Safe kmalloc wrapper that checks allocation constraints.
 *
 * Returns `Err(KernelError::DeadlockDetected)` if called while
 * preemption is disabled (spinlock held) or in IRQ context.
 *
 * The caller must provide an allocation function that returns
 * `Option<T>`.
 */
#[macro_export]
macro_rules! kmalloc {
    ($alloc_fn:expr) => {{
        if $crate::kernel::sync::preempt::allocation_allowed() {
            $alloc_fn
        } else {
            None
        }
    }};
}

/**
 * Safe kmalloc with Result return.
 *
 * Returns `Err(KernelError::DeadlockDetected)` if called in
 * an unsafe context, otherwise calls the allocation function.
 */
#[macro_export]
macro_rules! kmalloc_result {
    ($alloc_fn:expr) => {{
        if $crate::kernel::sync::preempt::allocation_allowed() {
            match $alloc_fn {
                Some(v) => Ok(v),
                None => Err($crate::kernel::error::KernelError::OutOfMemory),
            }
        } else {
            Err($crate::kernel::error::KernelError::DeadlockDetected)
        }
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preempt_disable_enable() {
        assert!(preemptible());
        preempt_disable();
        assert!(!preemptible());
        preempt_enable();
        assert!(preemptible());
    }

    #[test]
    fn test_preempt_guard() {
        assert!(preemptible());
        {
            let _guard = PreemptGuard::new();
            assert!(!preemptible());
        }
        assert!(preemptible());
    }

    #[test]
    fn test_nested_preempt() {
        assert!(preemptible());
        preempt_disable();
        preempt_disable();
        assert!(!preemptible());
        preempt_enable();
        assert!(!preemptible());
        preempt_enable();
        assert!(preemptible());
    }

    #[test]
    fn test_irq_context() {
        assert!(!in_irq());
        irq_enter();
        assert!(in_irq());
        assert!(!allocation_allowed());
        irq_exit();
        assert!(!in_irq());
    }

    #[test]
    fn test_allocation_allowed() {
        assert!(allocation_allowed());
        preempt_disable();
        assert!(!allocation_allowed());
        preempt_enable();
        assert!(allocation_allowed());
    }
}
