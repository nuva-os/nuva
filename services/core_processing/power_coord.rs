/*
 * Nuva OS - SystemService - CoreProcessing - Power Coordination
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

//! Power coordination framework for core processing services.
//! Services report their power state via Nuva IPC to nuva.service.power.

use core::sync::atomic::{AtomicU32, Ordering};

/// Power state of a service
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerState {
    /// No active processing
    Idle = 0,
    /// Normal processing active
    Active = 1,
    /// High-performance burst mode
    Burst = 2,
}

/// Power coordination manager - reports service power states
pub struct PowerCoordManager {
    /// Current power state
    state: AtomicU32,
    /// Service name for reporting
    service_name: &'static str,
}

impl PowerCoordManager {
    /// Create a new power coordination manager
    pub const fn new(service_name: &'static str) -> Self {
        PowerCoordManager {
            state: AtomicU32::new(PowerState::Idle as u32),
            service_name,
        }
    }

    /// Get current power state
    pub fn get_state(&self) -> PowerState {
        match self.state.load(Ordering::Acquire) {
            0 => PowerState::Idle,
            1 => PowerState::Active,
            2 => PowerState::Burst,
            _ => PowerState::Idle,
        }
    }

    /// Report current power state to nuva.service.power via Nuva IPC
    pub fn report_state(&self, new_state: PowerState) {
        self.state.store(new_state as u32, Ordering::Release);
        // Report to power service via Nuva IPC
        // In a full implementation, this sends a Nuva IPC message
        // to nuva.service.power with the updated state
        log_debug!(
            "Power state reported for {}: {:?}",
            self.service_name,
            new_state
        );
    }

    /// Request CPU frequency boost for burst processing
    pub fn request_boost(&self) {
        self.report_state(PowerState::Burst);
    }

    /// Release CPU frequency boost back to normal
    pub fn release_boost(&self) {
        self.report_state(PowerState::Active);
    }
}
