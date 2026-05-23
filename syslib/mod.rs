/*
 * Nuva OS - System Library - Module Root
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

// Re-export print macros from crate root
pub use crate::{pr_emerg, pr_alert, pr_crit, pr_err, pr_warn, pr_notice, pr_info, pr_debug};

// AI Engine
pub mod brain;

// AI Library
pub mod ai;

// Core Library (allocator, runtime, sync)
pub mod core;

// Language Runtime
pub mod lang;

// Network Library
pub mod net;

// Data Library
pub mod data;

// Machine Learning Library
pub mod ml;

// UI Library
pub mod ui;

// Standard Library
pub mod std;

// Runtime Library
pub mod runtime;

// Graphics Library
pub mod gfx;

// Concurrency Framework (GCD-style)
pub mod dispatch;

// POSIX Compatibility Layer
pub mod posix;

/// Initialize system libraries
///
/// Initialization order follows dependency chain:
/// Phase 1 — no dependencies; Phase 2 — language & concurrency;
/// Phase 3 — domain libraries; Phase 4 — compatibility layers.
pub fn init_libs() {
    // Phase 1: Core infrastructure
    core::init_core();
    std::init_std();
    runtime::init_runtime();

    // Phase 2: Language runtime and concurrency
    lang::init_lang();
    dispatch::init_dispatch();

    // Phase 3: Domain-specific libraries
    brain::init_brain();
    ai::init_ai();
    ml::init_ml();
    data::init_data();
    net::init_net();
    gfx::init_gfx();
    ui::init_ui();

    // Phase 4: Compatibility layers
    posix::init_posix();

    log_info!("System libraries initialized");
}
