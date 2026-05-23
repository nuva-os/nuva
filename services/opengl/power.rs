/*
 * Nuva OS - SystemService - OpenGL - Power Coordination
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

//! GPU power state coordination for the OpenGL service.
//! Reports Active when rendering commands are submitted,
//! reports Idle when no rendering work is pending.

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// GPU power state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuPowerState {
    /// GPU is idle, no rendering work pending
    Idle = 0,
    /// GPU is actively processing rendering commands
    Active = 1,
    /// GPU is in burst mode (high-performance rendering)
    Burst = 2,
    /// GPU is suspended (deep idle)
    Suspended = 3,
}

/// GPU power coordination manager for the OpenGL service
pub struct GpuPowerManager {
    /// Current GPU power state
    state: AtomicU32,
    /// Number of active rendering contexts
    active_contexts: AtomicU32,
    /// Number of pending command submissions
    pending_submissions: AtomicU32,
    /// Idle timeout in microseconds (transition to Suspended after this)
    idle_timeout_us: u64,
    /// Timestamp of last activity in microseconds
    last_activity_us: AtomicU64,
}

impl GpuPowerManager {
    /// Create a new GPU power manager
    pub const fn new(idle_timeout_us: u64) -> Self {
        GpuPowerManager {
            state: AtomicU32::new(GpuPowerState::Idle as u32),
            active_contexts: AtomicU32::new(0),
            pending_submissions: AtomicU32::new(0),
            idle_timeout_us,
            last_activity_us: AtomicU64::new(0),
        }
    }

    /// Get current GPU power state
    pub fn get_state(&self) -> GpuPowerState {
        match self.state.load(Ordering::Acquire) {
            0 => GpuPowerState::Idle,
            1 => GpuPowerState::Active,
            2 => GpuPowerState::Burst,
            3 => GpuPowerState::Suspended,
            _ => GpuPowerState::Idle,
        }
    }

    /// Called when a rendering context is created
    pub fn context_created(&self) {
        self.active_contexts.fetch_add(1, Ordering::Relaxed);
        self.report_active();
    }

    /// Called when a rendering context is destroyed
    pub fn context_destroyed(&self) {
        self.active_contexts.fetch_sub(1, Ordering::Relaxed);
        self.check_idle_transition();
    }

    /// Called when rendering commands are submitted.
    /// Transitions GPU to Active state.
    pub fn submit_commands(&self, timestamp_us: u64) {
        self.pending_submissions.fetch_add(1, Ordering::Relaxed);
        self.last_activity_us.store(timestamp_us, Ordering::Release);
        self.report_active();
        log_debug!("GPU power: commands submitted, state=Active");
    }

    /// Called when rendering commands complete.
    /// Decrements pending count and checks for idle transition.
    pub fn commands_completed(&self) {
        self.pending_submissions.fetch_sub(1, Ordering::Relaxed);
        self.check_idle_transition();
    }

    /// Request high-performance burst mode for intensive rendering
    pub fn request_burst(&self) {
        self.state.store(GpuPowerState::Burst as u32, Ordering::Release);
        log_debug!("GPU power: burst mode requested");
    }

    /// Release burst mode back to active
    pub fn release_burst(&self) {
        if self.get_state() == GpuPowerState::Burst {
            self.state.store(GpuPowerState::Active as u32, Ordering::Release);
            log_debug!("GPU power: burst mode released");
        }
    }

    /// Check if GPU can transition to idle
    fn check_idle_transition(&self) {
        let pending = self.pending_submissions.load(Ordering::Acquire);
        let contexts = self.active_contexts.load(Ordering::Acquire);
        if pending == 0 && contexts == 0 {
            self.report_idle();
        }
    }

    /// Report GPU active state to the power service
    fn report_active(&self) {
        let current = self.get_state();
        if current != GpuPowerState::Active && current != GpuPowerState::Burst {
            self.state.store(GpuPowerState::Active as u32, Ordering::Release);
            // In a full implementation, sends Nuva IPC message
            // to nuva.service.power reporting GPU Active
            log_debug!("GPU power: reporting Active to power service");
        }
    }

    /// Report GPU idle state to the power service
    fn report_idle(&self) {
        let current = self.get_state();
        if current == GpuPowerState::Active {
            self.state.store(GpuPowerState::Idle as u32, Ordering::Release);
            // In a full implementation, sends Nuva IPC message
            // to nuva.service.power reporting GPU Idle
            log_debug!("GPU power: reporting Idle to power service");
        }
    }

    /// Check if idle timeout has elapsed and transition to Suspended
    pub fn check_suspend(&self, current_timestamp_us: u64) {
        if self.get_state() != GpuPowerState::Idle {
            return;
        }
        let last = self.last_activity_us.load(Ordering::Acquire);
        if current_timestamp_us >= last && current_timestamp_us - last >= self.idle_timeout_us {
            self.state.store(GpuPowerState::Suspended as u32, Ordering::Release);
            log_debug!("GPU power: suspended after idle timeout");
        }
    }

    /// Wake GPU from suspended state
    pub fn wake(&self) {
        if self.get_state() == GpuPowerState::Suspended {
            self.state.store(GpuPowerState::Idle as u32, Ordering::Release);
            log_debug!("GPU power: woken from suspend");
        }
    }

    /// Get the number of active rendering contexts
    pub fn active_context_count(&self) -> u32 {
        self.active_contexts.load(Ordering::Acquire)
    }

    /// Get the number of pending command submissions
    pub fn pending_submission_count(&self) -> u32 {
        self.pending_submissions.load(Ordering::Acquire)
    }
}
