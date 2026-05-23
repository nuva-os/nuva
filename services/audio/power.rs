/*
 * Nuva OS - SystemService - Audio - Power Coordination
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

//! Audio processing power coordination.
//! Reports Active when decode/encode/resample work is in progress,
//! reports Idle when all audio operations are complete.

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// Audio power state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioPowerState {
    /// No active audio processing
    Idle = 0,
    /// Audio processing active
    Active = 1,
    /// Low-latency mode (reduced buffering for real-time)
    LowLatency = 2,
    /// Audio hardware suspended
    Suspended = 3,
}

/// Audio power coordination manager
pub struct AudioPowerManager {
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
    /// Number of active resampler instances
    active_resamplers: AtomicU32,
    /// Number of active mixer instances
    active_mixers: AtomicU32,
    /// Idle timeout in microseconds
    idle_timeout_us: u64,
    /// Timestamp of last activity
    last_activity_us: AtomicU64,
}

impl AudioPowerManager {
    /// Create a new audio power manager
    pub const fn new(idle_timeout_us: u64) -> Self {
        AudioPowerManager {
            state: AtomicU32::new(AudioPowerState::Idle as u32),
            active_decoders: AtomicU32::new(0),
            active_encoders: AtomicU32::new(0),
            pending_decodes: AtomicU32::new(0),
            pending_encodes: AtomicU32::new(0),
            active_resamplers: AtomicU32::new(0),
            active_mixers: AtomicU32::new(0),
            idle_timeout_us,
            last_activity_us: AtomicU64::new(0),
        }
    }

    /// Get current power state
    pub fn get_state(&self) -> AudioPowerState {
        match self.state.load(Ordering::Acquire) {
            0 => AudioPowerState::Idle,
            1 => AudioPowerState::Active,
            2 => AudioPowerState::LowLatency,
            3 => AudioPowerState::Suspended,
            _ => AudioPowerState::Idle,
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
        log_debug!("Audio power: decode started, state=Active");
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
        log_debug!("Audio power: encode started, state=Active");
    }

    /// Called when an encode operation completes
    pub fn encode_completed(&self) {
        self.pending_encodes.fetch_sub(1, Ordering::Relaxed);
        self.check_idle_transition();
    }

    /// Called when a resampler instance is created
    pub fn resampler_created(&self) {
        self.active_resamplers.fetch_add(1, Ordering::Relaxed);
        self.report_active();
    }

    /// Called when a resampler instance is destroyed
    pub fn resampler_destroyed(&self) {
        self.active_resamplers.fetch_sub(1, Ordering::Relaxed);
        self.check_idle_transition();
    }

    /// Called when a mixer instance is created
    pub fn mixer_created(&self) {
        self.active_mixers.fetch_add(1, Ordering::Relaxed);
        self.report_active();
    }

    /// Called when a mixer instance is destroyed
    pub fn mixer_destroyed(&self) {
        self.active_mixers.fetch_sub(1, Ordering::Relaxed);
        self.check_idle_transition();
    }

    /// Request low-latency mode for real-time audio
    pub fn request_low_latency(&self) {
        self.state.store(AudioPowerState::LowLatency as u32, Ordering::Release);
        log_debug!("Audio power: low-latency mode requested");
    }

    /// Release low-latency mode back to active
    pub fn release_low_latency(&self) {
        if self.get_state() == AudioPowerState::LowLatency {
            self.state.store(AudioPowerState::Active as u32, Ordering::Release);
            log_debug!("Audio power: low-latency mode released");
        }
    }

    /// Check if audio can transition to idle
    fn check_idle_transition(&self) {
        let pending_decode = self.pending_decodes.load(Ordering::Acquire);
        let pending_encode = self.pending_encodes.load(Ordering::Acquire);
        let decoders = self.active_decoders.load(Ordering::Acquire);
        let encoders = self.active_encoders.load(Ordering::Acquire);
        let resamplers = self.active_resamplers.load(Ordering::Acquire);
        let mixers = self.active_mixers.load(Ordering::Acquire);

        if pending_decode == 0
            && pending_encode == 0
            && decoders == 0
            && encoders == 0
            && resamplers == 0
            && mixers == 0
        {
            self.report_idle();
        }
    }

    /// Report audio active state to the power service
    fn report_active(&self) {
        let current = self.get_state();
        if current != AudioPowerState::Active && current != AudioPowerState::LowLatency {
            self.state.store(AudioPowerState::Active as u32, Ordering::Release);
            log_debug!("Audio power: reporting Active to power service");
        }
    }

    /// Report audio idle state to the power service
    fn report_idle(&self) {
        let current = self.get_state();
        if current == AudioPowerState::Active {
            self.state.store(AudioPowerState::Idle as u32, Ordering::Release);
            log_debug!("Audio power: reporting Idle to power service");
        }
    }

    /// Check if idle timeout has elapsed and transition to Suspended
    pub fn check_suspend(&self, current_timestamp_us: u64) {
        if self.get_state() != AudioPowerState::Idle {
            return;
        }
        let last = self.last_activity_us.load(Ordering::Acquire);
        if current_timestamp_us >= last && current_timestamp_us - last >= self.idle_timeout_us {
            self.state.store(AudioPowerState::Suspended as u32, Ordering::Release);
            log_debug!("Audio power: suspended after idle timeout");
        }
    }

    /// Wake audio from suspended state
    pub fn wake(&self) {
        if self.get_state() == AudioPowerState::Suspended {
            self.state.store(AudioPowerState::Idle as u32, Ordering::Release);
            log_debug!("Audio power: woken from suspend");
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
