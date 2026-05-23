/*
 * Nuva OS - SystemLibrary - Brain
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
pub mod inference;
pub mod model;
pub mod npu;
pub mod operators;
pub mod service;

/// Initialize AI Engine
pub fn init_brain() {
    // Initialize inference engine
    inference::init_inference();
    
    // Initialize model management
    model::init_model_manager();
    
    // Initialize NPU scheduling
    npu::init_npu_scheduler();
    
    // Initialize AI Service
    service::init_ai_service();
    
    log_info!("AI Engine initialized");
}