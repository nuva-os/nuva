/*
 * Nuva OS - SystemService - Mod.Rs
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
pub mod app;
pub mod ipc;
pub mod net;
pub mod power;
pub mod security;
pub mod form_factor;
pub mod core_processing;
pub mod opengl;
pub mod sqlite;
pub mod web;
pub mod video;
pub mod audio;
pub mod image;

/// Initialize core processing services (six media/graphic services)
///
/// Must be called after IPC service is initialized.
/// Phase 1: Parallel-init services with no L3 cross-dependencies
/// Phase 2: Services with L3 cross-dependencies (web depends on net)
fn init_core_processing_services() {
    // Initialize core processing shared framework
    core_processing::init_core_processing_framework();

    // Phase 1: Services with no L3 cross-dependencies (can parallelize)
    opengl::init_opengl_service();
    sqlite::init_sqlite_service();
    audio::init_audio_service();
    image::init_image_service();
    video::init_video_service();

    // Phase 2: Services with L3 cross-dependencies
    // web depends on net service (already initialized)
    web::init_web_service();

    log_info!("Core processing services initialized");
}

/// Initialize system services
pub fn init_services() {
    // Initialize form factor manager (must be first - other services depend on it)
    form_factor::init_form_factor_manager();

    // Initialize power service
    power::init_power_service();
    
    // Initialize security service
    security::init_security_service();
    
    // Initialize network service
    net::init_network_service();
    
    // Initialize IPC service (must be before core processing services)
    ipc::init_ipc_service();

    // Initialize core processing services (after IPC)
    init_core_processing_services();
    
    // Initialize application service
    app::init_app_service();
    
    log_info!("System services initialized");
}
