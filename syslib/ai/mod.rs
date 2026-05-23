/*
 * Nuva OS - System Library - AI
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

//! AI Library — model management, optimization, and intelligent scheduling.

// Re-export print macros from crate root
pub use crate::{pr_emerg, pr_alert, pr_crit, pr_err, pr_warn, pr_notice, pr_info, pr_debug};

pub mod model_manager;
pub mod optimizer;
pub mod scheduler;

// Re-export main types
pub use model_manager::ModelManager;
pub use optimizer::{PerformanceOptimizer, BottleneckType};
pub use scheduler::{IntelligentScheduler, SchedulingDecision};

/// Initialize AI library
pub fn init_ai() {
    log_info!("AI library initialized");
}
