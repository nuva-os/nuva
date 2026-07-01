/*
 * Nuva OS - Kernel - RCU (Read-Copy-Update) Mechanism
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

//! RCU (Read-Copy-Update) mechanism for lock-free read-side access.
//! Provides zero-overhead read-side critical sections with deferred reclamation.

use core::sync::atomic::{AtomicU32, AtomicU64, AtomicPtr, Ordering};
// FnBox removed - was deprecated in alloc

use super::percpu::{PerCpu, MAX_CPUS};

/// RCU callback type: function pointer for deferred reclamation
pub type RcuCallback = unsafe fn(*mut u8);

/// Maximum pending callbacks per CPU before attempting to schedule grace period
const RCU_CALLBACK_BATCH_SIZE: usize = 64;

/// Per-CPU RCU state
#[repr(C, align(64))]
pub struct RcuCpuState {
    /// Nesting depth counter for read-side critical sections.
    /// If > 0, this CPU is in an RCU read-side critical section.
    nesting_depth: AtomicU32,
    /// Current grace period number this CPU has seen
    grace_period: AtomicU64,
    /// Number of pending callbacks
    pending_count: AtomicU32,
    /// Callback array (function pointers)
    callbacks: [AtomicPtr<RcuCallbackEntry>; RCU_CALLBACK_BATCH_SIZE],
    /// Callback data array
    callback_data: [AtomicPtr<u8>; RCU_CALLBACK_BATCH_SIZE],
}

/// RCU callback entry
#[repr(C)]
pub struct RcuCallbackEntry {
    /// Callback function
    func: RcuCallback,
    /// Data pointer
    data: *mut u8,
    /// Next entry in list
    next: *mut RcuCallbackEntry,
}

impl RcuCpuState {
    pub const fn new() -> Self {
        RcuCpuState {
            nesting_depth: AtomicU32::new(0),
            grace_period: AtomicU64::new(0),
            pending_count: AtomicU32::new(0),
            callbacks: [AtomicPtr::new(core::ptr::null_mut()); RCU_CALLBACK_BATCH_SIZE],
            callback_data: [AtomicPtr::new(core::ptr::null_mut()); RCU_CALLBACK_BATCH_SIZE],
        }
    }

    /// Enter RCU read-side critical section.
    /// Increments nesting depth. No memory barrier needed for the
    /// increment itself since we only need to ensure the read-side
    /// accesses happen within the critical section.
    #[inline(always)]
    pub fn enter(&self) {
        self.nesting_depth.fetch_add(1, Ordering::Relaxed);
    }

    /// Exit RCU read-side critical section.
    /// Decrements nesting depth. A release ordering ensures all
    /// read-side accesses are visible before exiting.
    #[inline(always)]
    pub fn exit(&self) {
        self.nesting_depth.fetch_sub(1, Ordering::Release);
    }

    /// Check if this CPU is in an RCU read-side critical section
    #[inline(always)]
    pub fn is_in_critical_section(&self) -> bool {
        self.nesting_depth.load(Ordering::Acquire) > 0
    }

    /// Get current nesting depth
    #[inline]
    pub fn nesting_depth(&self) -> u32 {
        self.nesting_depth.load(Ordering::Acquire)
    }
}

/// Global RCU state
pub struct RcuState {
    /// Per-CPU RCU state
    percpu: PerCpu<RcuCpuState>,
    /// Global grace period counter
    global_grace_period: AtomicU64,
    /// Completed grace period counter
    completed_grace_period: AtomicU64,
}

impl RcuState {
    pub const fn new() -> Self {
        RcuState {
            percpu: PerCpu::new(RcuCpuState::new()),
            global_grace_period: AtomicU64::new(0),
            completed_grace_period: AtomicU64::new(0),
        }
    }

    /// Enter RCU read-side critical section on current CPU
    #[inline(always)]
    pub fn read_lock(&self) {
        let cpu_id = PerCpu::<RcuCpuState>::current_cpu_id();
        // SAFETY: cpu_id is valid from hardware TLS register
        unsafe {
            self.percpu.for_cpu_unchecked(cpu_id).enter();
        }
    }

    /// Exit RCU read-side critical section on current CPU
    #[inline(always)]
    pub fn read_unlock(&self) {
        let cpu_id = PerCpu::<RcuCpuState>::current_cpu_id();
        // SAFETY: cpu_id is valid from hardware TLS register
        unsafe {
            self.percpu.for_cpu_unchecked(cpu_id).exit();
        }
    }

    /// Register a callback for deferred execution after a grace period.
    /// The callback will be invoked once all CPUs have exited any RCU
    /// read-side critical sections that were active at call_rcu() time.
    pub fn call_rcu(&self, func: RcuCallback, data: *mut u8) {
        let cpu_id = PerCpu::<RcuCpuState>::current_cpu_id();
        // SAFETY: cpu_id is valid from hardware TLS register
        unsafe {
            let state = self.percpu.for_cpu_unchecked(cpu_id);
            let idx = state.pending_count.load(Ordering::Acquire) as usize;
            if idx < RCU_CALLBACK_BATCH_SIZE {
                let entry = RcuCallbackEntry {
                    func,
                    data,
                    next: core::ptr::null_mut(),
                };
                // SAFETY: idx is within bounds (checked above)
                let entry_ptr = &entry as *const RcuCallbackEntry as *mut RcuCallbackEntry;
                state.callbacks[idx].store(entry_ptr, Ordering::Release);
                state.callback_data[idx].store(data, Ordering::Release);
                state.pending_count.fetch_add(1, Ordering::Release);
            }
        }
    }

    /// Check if a grace period has completed.
    /// A grace period completes when all CPUs have exited their
    /// read-side critical sections that were active when the
    /// grace period started.
    pub fn check_grace_period(&self) -> bool {
        let current_gp = self.global_grace_period.load(Ordering::Acquire);
        for cpu_id in 0..MAX_CPUS {
            // SAFETY: cpu_id < MAX_CPUS
            let state = unsafe { self.percpu.for_cpu_unchecked(cpu_id) };
            if state.is_in_critical_section() {
                return false;
            }
            if state.grace_period.load(Ordering::Acquire) < current_gp {
                return false;
            }
        }
        self.completed_grace_period.store(current_gp, Ordering::Release);
        true
    }

    /// Advance the global grace period counter
    pub fn advance_grace_period(&self) {
        self.global_grace_period.fetch_add(1, Ordering::AcqRel);
    }

    /// Execute pending callbacks whose grace periods have completed
    pub fn execute_callbacks(&self) {
        let cpu_id = PerCpu::<RcuCpuState>::current_cpu_id();
        // SAFETY: cpu_id is valid from hardware TLS register
        unsafe {
            let state = self.percpu.for_cpu_unchecked(cpu_id);
            let count = state.pending_count.load(Ordering::Acquire);
            let completed = self.completed_grace_period.load(Ordering::Acquire);

            for i in 0..count as usize {
                if i >= RCU_CALLBACK_BATCH_SIZE {
                    break;
                }
                let entry_ptr = state.callbacks[i].load(Ordering::Acquire);
                if !entry_ptr.is_null() {
                    let entry = &*entry_ptr;
                    if completed > 0 {
                        // SAFETY: callback invocation is the responsibility of the caller
                        // who registered it via call_rcu()
                        entry.func(entry.data);
                        state.callbacks[i].store(core::ptr::null_mut(), Ordering::Release);
                        state.callback_data[i].store(core::ptr::null_mut(), Ordering::Release);
                    }
                }
            }
            state.pending_count.store(0, Ordering::Release);
        }
    }
}

/// Global RCU state instance
static RCU_STATE: RcuState = RcuState::new();

/// Enter RCU read-side critical section
#[inline(always)]
pub fn rcu_read_lock() {
    RCU_STATE.read_lock();
}

/// Exit RCU read-side critical section
#[inline(always)]
pub fn rcu_read_unlock() {
    RCU_STATE.read_unlock();
}

/// Register a deferred callback after grace period
pub fn call_rcu(func: RcuCallback, data: *mut u8) {
    RCU_STATE.call_rcu(func, data);
}

/// Check if grace period has completed
pub fn rcu_check_grace_period() -> bool {
    RCU_STATE.check_grace_period()
}

/// Advance grace period
pub fn rcu_advance_grace_period() {
    RCU_STATE.advance_grace_period();
}

/// Execute completed callbacks
pub fn rcu_execute_callbacks() {
    RCU_STATE.execute_callbacks();
}

/// Macro for RCU read-side critical section.
/// Usage:
/// ```
/// rcu_read_lock!({
///     // read-side access here
/// });
/// ```
#[macro_export]
macro_rules! rcu_read_lock {
    ($body:block) => {
        $crate::kernel::sync::rcu::rcu_read_lock();
        $body
        $crate::kernel::sync::rcu::rcu_read_unlock();
    };
}
