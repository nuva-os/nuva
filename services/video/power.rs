/*
 * Nuva OS - SystemService - Video - Power Coordination
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

//! Video codec power coordination.
//! Reports Active when decode/encode work is in progress,
//! reports Idle when all codec operations are complete.

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// Video power state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoPowerState {
    /// No active decode/encode work
    Idle = 0,
    /// Decode/encode work active
    Active = 1,
    /// High-performance burst mode (e.g. 4K60 encode)
    Burst = 2,
    /// Hardware codec suspended
    Suspended = 3,
}

/// Video power coordination manager
pub struct VideoPowerManager {
    /// Current power state
    state: AtomicU32,
    /// Number of active decoder instances
    active_decoders: AtomicU32,
    /// Number of active encoder instances
    active_encoders: AtomicU32,
    /// Number of pending decode operations
    pending_decodes: AtomicU32,
    /// Number of pending encode operations
    pending_encodes: AtomicU32,
    /// Idle timeout in microseconds
    idle_timeout_us: u64,
    /// Timestamp of last activity
    last_activity_us: AtomicU64,
}

impl VideoPowerManager {
    /// Create a new video power manager
    pub const fn new(idle_timeout_us: u64) -> Self {
        VideoPowerManager {
            state: AtomicU32::new(VideoPowerState::Idle as u32),
            active_decoders: AtomicU32::new(0),
            active_encoders: AtomicU32::new(0),
            pending_decodes: AtomicU32::new(0),
            pending_encodes: AtomicU32::new(0),
            idle_timeout_us,
            last_activity_us: AtomicU64::new(0),
        }
    }

    /// Get current power state
    pub fn get_state(&self) -> VideoPowerState {
        match self.state.load(Ordering::Acquire) {
            0 => VideoPowerState::Idle,
            1 => VideoPowerState::Active,
            2 => VideoPowerState::Burst,
            3 => VideoPowerState::Suspended,
            _ => VideoPowerState::Idle,
        }
    }

    /// Called when a decoder instance is created
    pub fn decoder_created(&self) {
        self.active_decoders.fetch_add(1, Ordering::Relaxed);
        self.report_active();
    }

    /// Called when a decoder instance is destroyed
    pub fn decoder_destroyed(&self) {
        self.active_decoders.fetch_sub(1, Ordering::Relaxed);
        self.check_idle_transition();
    }

    /// Called when an encoder instance is created
    pub fn encoder_created(&self) {
        self.active_encoders.fetch_add(1, Ordering::Relaxed);
        self.report_active();
    }

    /// Called when an encoder instance is destroyed
    pub fn encoder_destroyed(&self) {
        self.active_encoders.fetch_sub(1, Ordering::Relaxed);
        self.check_idle_transition();
    }

    /// Called when a decode operation starts
    pub fn decode_started(&self, timestamp_us: u64) {
        self.pending_decodes.fetch_add(1, Ordering::Relaxed);
        self.last_activity_us.store(timestamp_us, Ordering::Release);
        self.report_active();
        log_debug!("Video power: decode started, state=Active");
    }

    /// Called when a decode operation completes
    pub fn decode_completed(&self) {
        self.pending_decodes.fetch_sub(1, Ordering::Relaxed);
        self.check_idle_transition();
    }

    /// Called when an encode operation starts
    pub fn encode_started(&self, timestamp_us: u64) {
        self.pending_encodes.fetch_add(1, Ordering::Relaxed);
        self.last_activity_us.store(timestamp_us, Ordering::Release);
        self.report_active();
        log_debug!("Video power: encode started, state=Active");
    }

    /// Called when an encode operation completes
    pub fn encode_completed(&self) {
        self.pending_encodes.fetch_sub(1, Ordering::Relaxed);
        self.check_idle_transition();
    }

    /// Request high-performance burst mode for intensive encode/decode
    pub fn request_burst(&self) {
        self.state.store(VideoPowerState::Burst as u32, Ordering::Release);
        log_debug!("Video power: burst mode requested");
    }

    /// Release burst mode back to active
    pub fn release_burst(&self) {
        if self.get_state() == VideoPowerState::Burst {
            self.state.store(VideoPowerState::Active as u32, Ordering::Release);
            log_debug!("Video power: burst mode released");
        }
    }

    /// Check if video codec can transition to idle
    fn check_idle_transition(&self) {
        let pending_decode = self.pending_decodes.load(Ordering::Acquire);
        let pending_encode = self.pending_encodes.load(Ordering::Acquire);
        let decoders = self.active_decoders.load(Ordering::Acquire);
        let encoders = self.active_encoders.load(Ordering::Acquire);

        if pending_decode == 0 && pending_encode == 0 && decoders == 0 && encoders == 0 {
            self.report_idle();
        }
    }

    /// Report video codec active state to the power service
    fn report_active(&self) {
        let current = self.get_state();
        if current != VideoPowerState::Active && current != VideoPowerState::Burst {
            self.state.store(VideoPowerState::Active as u32, Ordering::Release);
            log_debug!("Video power: reporting Active to power service");
        }
    }

    /// Report video codec idle state to the power service
    fn report_idle(&self) {
        let current = self.get_state();
        if current == VideoPowerState::Active {
            self.state.store(VideoPowerState::Idle as u32, Ordering::Release);
            log_debug!("Video power: reporting Idle to power service");
        }
    }

    /// Check if idle timeout has elapsed and transition to Suspended
    pub fn check_suspend(&self, current_timestamp_us: u64) {
        if self.get_state() != VideoPowerState::Idle {
            return;
        }
        let last = self.last_activity_us.load(Ordering::Acquire);
        if current_timestamp_us >= last && current_timestamp_us - last >= self.idle_timeout_us {
            self.state.store(VideoPowerState::Suspended as u32, Ordering::Release);
            log_debug!("Video power: suspended after idle timeout");
        }
    }

    /// Wake video codec from suspended state
    pub fn wake(&self) {
        if self.get_state() == VideoPowerState::Suspended {
            self.state.store(VideoPowerState::Idle as u32, Ordering::Release);
            log_debug!("Video power: woken from suspend");
        }
    }

    /// Get number of active decoder instances
    pub fn active_decoder_count(&self) -> u32 {
        self.active_decoders.load(Ordering::Acquire)
    }

    /// Get number of active encoder instances
    pub fn active_encoder_count(&self) -> u32 {
        self.active_encoders.load(Ordering::Acquire)
    }

    /// Get number of pending decode operations
    pub fn pending_decode_count(&self) -> u32 {
        self.pending_decodes.load(Ordering::Acquire)
    }

    /// Get number of pending encode operations
    pub fn pending_encode_count(&self) -> u32 {
        self.pending_encodes.load(Ordering::Acquire)
    }
}
