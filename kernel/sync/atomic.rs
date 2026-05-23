/*
 * Nuva OS - Kernel - Atomic Operations
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

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/** 32-bit atomic integer type alias */
pub type AtomicInt = AtomicU32;

/** 64-bit atomic integer type alias */
pub type AtomicLong = AtomicU64;

/**
 * Atomically read a value.
 *
 * Uses Acquire ordering to ensure subsequent reads see values
 * at least as recent as this read.
 */
#[inline(always)]
pub fn atomic_read(v: &AtomicU32) -> u32 {
    v.load(Ordering::Acquire)
}

/**
 * Atomically set a value.
 *
 * Uses Release ordering to ensure all prior writes are visible
 * to other threads that subsequently read this value with Acquire.
 */
#[inline(always)]
pub fn atomic_set(v: &AtomicU32, i: u32) {
    v.store(i, Ordering::Release);
}

/**
 * Atomically add and return the previous value.
 *
 * Uses AcqRel ordering: Acquire semantics on read, Release on write.
 * Required for synchronization (e.g., reference counts, semaphores).
 */
#[inline(always)]
pub fn atomic_add(v: &AtomicU32, i: u32) -> u32 {
    v.fetch_add(i, Ordering::AcqRel)
}

/**
 * Atomically subtract and return the previous value.
 *
 * Uses AcqRel ordering for synchronization correctness.
 */
#[inline(always)]
pub fn atomic_sub(v: &AtomicU32, i: u32) -> u32 {
    v.fetch_sub(i, Ordering::AcqRel)
}

/**
 * Atomically add and test if result is zero.
 *
 * Uses AcqRel: the add must be visible to other threads, and
 * we must see their updates.
 */
#[inline(always)]
pub fn atomic_add_and_test(v: &AtomicU32, i: u32) -> bool {
    v.fetch_add(i, Ordering::AcqRel) + i == 0
}

/**
 * Atomically subtract and test if result is zero.
 *
 * Uses AcqRel for synchronization: used in reference counting
 * where the decrement must synchronize with potential destruction.
 */
#[inline(always)]
pub fn atomic_sub_and_test(v: &AtomicU32, i: u32) -> bool {
    v.fetch_sub(i, Ordering::AcqRel) - i == 0
}

/**
 * Atomically decrement and test if result is zero.
 *
 * Used for reference count release: AcqRel ensures that
 * all accesses to the object are visible to the thread
 * that sees the count drop to zero and may free the object.
 */
#[inline(always)]
pub fn atomic_dec_and_test(v: &AtomicU32) -> bool {
    v.fetch_sub(1, Ordering::AcqRel) == 1
}

/**
 * Atomically increment and test if overflow.
 *
 * Uses AcqRel for synchronization with decrement operations.
 */
#[inline(always)]
pub fn atomic_inc_and_test(v: &AtomicU32) -> bool {
    v.fetch_add(1, Ordering::AcqRel) == u32::MAX
}

/**
 * Atomically compare and exchange.
 *
 * Uses AcqRel on success, Acquire on failure. The AcqRel ordering
 * on success ensures both prior writes are visible and subsequent
 * reads see the new value. Acquire on failure ensures we see
 * the current value.
 */
#[inline(always)]
pub fn atomic_cmpxchg(v: &AtomicU32, old: u32, new: u32) -> u32 {
    v.compare_exchange(old, new, Ordering::AcqRel, Ordering::Acquire)
        .unwrap_or_else(|x| x)
}

/**
 * Atomically exchange value.
 *
 * Uses AcqRel ordering for full synchronization on both
 * the read and write of the swapped value.
 */
#[inline(always)]
pub fn atomic_xchg(v: &AtomicU32, new: u32) -> u32 {
    v.swap(new, Ordering::AcqRel)
}

/**
 * Atomically bitwise AND.
 *
 * Uses AcqRel ordering: the read-modify-write must synchronize
 * with other atomic operations on the same variable.
 */
#[inline(always)]
pub fn atomic_and(v: &AtomicU32, i: u32) -> u32 {
    v.fetch_and(i, Ordering::AcqRel)
}

/**
 * Atomically bitwise OR.
 *
 * Uses AcqRel ordering for synchronization with other
 * atomic bit operations (e.g., set_bit/clear_bit pairs).
 */
#[inline(always)]
pub fn atomic_or(v: &AtomicU32, i: u32) -> u32 {
    v.fetch_or(i, Ordering::AcqRel)
}

/**
 * Atomically bitwise XOR.
 *
 * Uses AcqRel ordering: change_bit uses XOR and must
 * synchronize with test_and_set/clear operations.
 */
#[inline(always)]
pub fn atomic_xor(v: &AtomicU32, i: u32) -> u32 {
    v.fetch_xor(i, Ordering::AcqRel)
}

/**
 * Atomically test if a bit is set.
 *
 * Read-only operation: Acquire ordering ensures we see
 * the most recent bit modifications.
 */
#[inline(always)]
pub fn atomic_test_bit(v: &AtomicU32, nr: u32) -> bool {
    (atomic_read(v) & (1 << nr)) != 0
}

/**
 * Atomically set a bit.
 *
 * Uses AcqRel ordering to synchronize with test_bit and
 * clear_bit operations.
 */
#[inline(always)]
pub fn atomic_set_bit(v: &AtomicU32, nr: u32) {
    atomic_or(v, 1 << nr);
}

/**
 * Atomically clear a bit.
 *
 * Uses AcqRel ordering to synchronize with test_bit and
 * set_bit operations.
 */
#[inline(always)]
pub fn atomic_clear_bit(v: &AtomicU32, nr: u32) {
    atomic_and(v, !(1 << nr));
}

/**
 * Atomically toggle a bit.
 *
 * Uses AcqRel ordering to synchronize with other bit operations.
 */
#[inline(always)]
pub fn atomic_change_bit(v: &AtomicU32, nr: u32) {
    atomic_xor(v, 1 << nr);
}

/**
 * Atomically test and set a bit.
 *
 * Uses AcqRel ordering: the test must see the current state
 * and the set must be visible to subsequent test operations.
 */
#[inline(always)]
pub fn atomic_test_and_set_bit(v: &AtomicU32, nr: u32) -> bool {
    let old = atomic_or(v, 1 << nr);
    (old & (1 << nr)) != 0
}

/**
 * Atomically test and clear a bit.
 *
 * Uses AcqRel ordering for synchronization.
 */
#[inline(always)]
pub fn atomic_test_and_clear_bit(v: &AtomicU32, nr: u32) -> bool {
    let old = atomic_and(v, !(1 << nr));
    (old & (1 << nr)) != 0
}

/**
 * Atomically test and toggle a bit.
 *
 * Uses AcqRel ordering for synchronization.
 */
#[inline(always)]
pub fn atomic_test_and_change_bit(v: &AtomicU32, nr: u32) -> bool {
    let old = atomic_xor(v, 1 << nr);
    (old & (1 << nr)) != 0
}

/**
 * Full memory barrier.
 *
 * SeqCst ordering ensures total ordering across all threads.
 * Use when ordering must be established between different
 * atomic variables.
 */
#[inline(always)]
pub fn barrier() {
    core::sync::atomic::fence(Ordering::SeqCst);
}

/**
 * Read memory barrier.
 *
 * Acquire ordering ensures subsequent reads see values
 * written before this barrier.
 */
#[inline(always)]
pub fn rmb() {
    core::sync::atomic::fence(Ordering::Acquire);
}

/**
 * Write memory barrier.
 *
 * Release ordering ensures prior writes are visible to
 * threads that execute an Acquire fence after this.
 */
#[inline(always)]
pub fn wmb() {
    core::sync::atomic::fence(Ordering::Release);
}

/**
 * Data Synchronization Barrier (ARM DSB).
 *
 * Ensures all explicit memory accesses before this instruction
 * complete before any after it. Required for device register
 * access ordering.
 */
#[inline(always)]
pub fn dsb() {
    // SAFETY: dsb sy is a barrier instruction that only affects
    // the ordering of memory accesses. It does not modify memory
    // in an unsafe way, and cannot cause undefined behavior.
    // It is required for correct device MMIO access ordering.
    #[cfg(target_arch = "aarch64")]
    // SAFETY: dsb sy is a data synchronization barrier on ARM64.
    unsafe {
        core::arch::asm!("dsb sy");
    }
    #[cfg(target_arch = "x86_64")]
    {
        // x86_64: mfence provides full barrier semantics
        core::sync::atomic::fence(Ordering::SeqCst);
    }
    #[cfg(target_arch = "loongarch64")]
    {
        // LoongArch: barrier.dbar provides data barrier semantics
        // SAFETY: dbar 0 is a data barrier instruction on LoongArch that
        // only affects memory access ordering, similar to ARM DSB.
        unsafe {
            core::arch::asm!("dbar 0");
        }
    }
}

/**
 * Data Memory Barrier (ARM DMB).
 *
 * Ensures memory accesses before this barrier are ordered
 * with respect to those after it, but does not require
 * completion (unlike DSB).
 */
#[inline(always)]
pub fn dmb() {
    // SAFETY: dmb sy is a memory barrier instruction that only
    // affects ordering of memory accesses. It is less strict
    // than DSB and is appropriate for shareability domain
    // synchronization.
    #[cfg(target_arch = "aarch64")]
    // SAFETY: dmb sy is a data memory barrier on ARM64.
    unsafe {
        core::arch::asm!("dmb sy");
    }
    #[cfg(target_arch = "x86_64")]
    {
        // x86_64: mfence provides full barrier semantics
        core::sync::atomic::fence(Ordering::SeqCst);
    }
    #[cfg(target_arch = "loongarch64")]
    {
        // SAFETY: dbar 0 is a data memory barrier on LoongArch that only
        // affects ordering of memory accesses, similar to ARM DMB.
        unsafe {
            core::arch::asm!("dbar 0");
        }
    }
}

/**
 * Instruction Synchronization Barrier (ARM ISB).
 *
 * Flushes the pipeline and ensures all subsequent instruction
 * fetches see the results of prior context-altering operations
 * (e.g., system register writes, TLB maintenance).
 */
#[inline(always)]
pub fn isb() {
    // SAFETY: isb is a pipeline flush instruction that ensures
    // instruction fetch ordering. It is required after writing
    // system registers (e.g., SCTLR, TTBR) to ensure the
    // processor sees the new configuration. It cannot cause
    // memory safety violations.
    #[cfg(target_arch = "aarch64")]
    // SAFETY: isb is an instruction synchronization barrier on ARM64.
    unsafe {
        core::arch::asm!("isb");
    }
    #[cfg(target_arch = "x86_64")]
    {
        // x86_64: no direct equivalent; serializing instruction
        // lfence provides sufficient ordering for most cases
        // SAFETY: lfence is a load fence instruction on x86_64 that
        // serializes instruction fetches, similar to ARM ISB.
        unsafe {
            core::arch::asm!("lfence");
        }
    }
    #[cfg(target_arch = "loongarch64")]
    {
        // LoongArch: ibar provides instruction barrier semantics
        // SAFETY: ibar 0 is an instruction barrier on LoongArch that
        // flushes the pipeline, similar to ARM ISB.
        unsafe {
            core::arch::asm!("ibar 0");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atomic() {
        let v = AtomicU32::new(0);

        assert_eq!(atomic_add(&v, 5), 0);
        assert_eq!(atomic_read(&v), 5);

        assert_eq!(atomic_sub(&v, 3), 5);
        assert_eq!(atomic_read(&v), 2);

        assert_eq!(atomic_cmpxchg(&v, 2, 10), 2);
        assert_eq!(atomic_read(&v), 10);
    }

    #[test]
    fn test_atomic_bits() {
        let v = AtomicU32::new(0);

        assert!(!atomic_test_bit(&v, 3));
        atomic_set_bit(&v, 3);
        assert!(atomic_test_bit(&v, 3));

        assert!(atomic_test_and_clear_bit(&v, 3));
        assert!(!atomic_test_bit(&v, 3));
    }
}
