/*
 * Nuva OS - System Library - Core
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

//! Core Library — foundation types, synchronization, and memory utilities.

// Re-export print macros from crate root
pub use crate::{pr_emerg, pr_alert, pr_crit, pr_err, pr_warn, pr_notice, pr_info, pr_debug};

pub mod sync;
pub mod alloc;

// Re-export main types
pub use sync::{MpscQueue, SpscQueue, LockFreeStack};
pub use alloc::{MemoryPool, PoolManager, PoolManagerConfig, PoolBox};

/// Initialize core library
pub fn init_core() {
    log_info!("Core library initialized");
}
