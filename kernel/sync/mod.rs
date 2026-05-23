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

//! Synchronization primitives module

// Re-export print macros from crate root
pub use crate::{pr_emerg, pr_alert, pr_crit, pr_err, pr_warn, pr_notice, pr_info, pr_debug};
pub mod atomic;
pub mod mutex;
pub mod percpu;
pub mod preempt;
pub mod primitives;
pub mod rcu;
pub mod spinlock;

// Re-export synchronization types
pub use primitives::{
    SpinLock, SpinLockGuard, Mutex, Semaphore, RwLock, MutexState,
};
pub use preempt::{
    preempt_disable, preempt_enable, preempt_count, preemptible,
    allocation_allowed, in_irq, in_softirq, irq_enter, irq_exit,
    PreemptGuard,
};

/// Initialize sync module
pub fn init_sync() {
    // Initialize synchronization primitives
}
