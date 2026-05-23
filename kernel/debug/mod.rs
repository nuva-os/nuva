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

//! Kernel debug module

// Re-export print macros from crate root
pub use crate::{pr_emerg, pr_alert, pr_crit, pr_err, pr_warn, pr_notice, pr_info, pr_debug};
pub mod perf;
pub mod printk;
pub mod tracing;

// Re-export tracing types
pub use tracing::{
    DebugEvent, DebugEventType, DebugEventData, FuncData, MemData, LockData,
    IrqData, SchedData, SyscallData, DebugBuffer, Breakpoint, BreakpointType,
    Watchpoint, DebugManager, DebugStats, get_debug_manager,
};

/// Initialize debug module
pub fn init_debug() {
    printk::init_printk();
    perf::init_perf();
    tracing::init_debug();
}
