/*
 * Nuva OS - SystemService - CoreProcessing
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

//! Core processing shared framework for all six media/graphic services.
//! Provides unified service node registration, zero-copy transfer,
//! hardware acceleration with software fallback, power coordination,
//! permission verification, format detection, and error model.

pub use crate::{pr_emerg, pr_alert, pr_crit, pr_err, pr_warn, pr_notice, pr_info, pr_debug};

pub mod service_node;
pub mod shm_transfer;
pub mod hw_accel;
pub mod power_coord;
pub mod permission;
pub mod format_detect;
pub mod error;

pub use service_node::{CoreProcessingService, ServiceConfig, ServiceVersion, ServiceHealth, ServiceStats, ServiceNodeId, CallerIdentity};
pub use error::{ServiceError, ServiceSpecificError};

/// Initialize core processing shared framework
pub fn init_core_processing_framework() {
    log_info!("Core processing framework initialized");
}
