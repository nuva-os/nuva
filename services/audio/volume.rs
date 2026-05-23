/*
 * Nuva OS - SystemService - Audio - Volume Control
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

//! Per-stream volume control and mute.
//! Supports independent volume gain per audio stream, applied
//! independently of codec decode/encode processing.

use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

use super::error::{AudioError, PcmBuffer};

/// Stream identifier for volume control
pub type VolumeStreamId = u64;

/// Volume control for a single stream
#[derive(Debug)]
struct StreamVolume {
    /// Linear gain factor (0.0 = silent, 1.0 = unity)
    /// Stored as fixed-point Q16.16 for atomic access
    gain_fp: AtomicU32,
    /// Mute state (0 = unmuted, 1 = muted)
    muted: AtomicU32,
}

/// Fixed-point fractional bits for volume gain
const VOLUME_FRAC_BITS: u32 = 16;
/// Fixed-point scale (1.0 in fixed-point)
const VOLUME_FP_SCALE: u32 = 1u32 << VOLUME_FRAC_BITS;

impl StreamVolume {
    /// Create a new stream volume with unity gain
    fn new() -> Self {
        StreamVolume {
            gain_fp: AtomicU32::new(VOLUME_FP_SCALE),
            muted: AtomicU32::new(0),
        }
    }

    /// Set the linear gain factor
    fn set_gain(&self, gain: f32) {
        let clamped = gain.clamp(0.0, 128.0);
        let fp = (clamped * VOLUME_FP_SCALE as f32) as u32;
        self.gain_fp.store(fp, Ordering::Release);
    }

    /// Get the linear gain factor
    fn get_gain(&self) -> f32 {
        let fp = self.gain_fp.load(Ordering::Acquire);
        fp as f32 / VOLUME_FP_SCALE as f32
    }

    /// Set mute state
    fn set_mute(&self, mute: bool) {
        self.muted.store(if mute { 1 } else { 0 }, Ordering::Release);
    }

    /// Get mute state
    fn is_muted(&self) -> bool {
        self.muted.load(Ordering::Acquire) != 0
    }

    /// Get the effective gain (0 if muted)
    fn effective_gain(&self) -> f32 {
        if self.is_muted() {
            0.0
        } else {
            self.get_gain()
        }
    }
}

/// Per-stream volume manager
pub struct VolumeManager {
    /// Per-stream volume controls
    streams: BTreeMap<VolumeStreamId, StreamVolume>,
    /// Master volume (applied to all streams)
    master_volume: StreamVolume,
    /// Next stream ID for auto-assignment
    next_stream_id: AtomicU64,
}

impl VolumeManager {
    /// Create a new volume manager
    pub fn new() -> Self {
        VolumeManager {
            streams: BTreeMap::new(),
            master_volume: StreamVolume::new(),
            next_stream_id: AtomicU64::new(1),
        }
    }

    /// Register a new stream for volume control
    pub fn register_stream(&mut self) -> VolumeStreamId {
        let id = self.next_stream_id.fetch_add(1, Ordering::Relaxed);
        self.streams.insert(id, StreamVolume::new());
        id
    }

    /// Register a stream with a specific ID
    pub fn register_stream_with_id(&mut self, id: VolumeStreamId) -> Result<(), AudioError> {
        if self.streams.contains_key(&id) {
            return Err(AudioError::InvalidParameter);
        }
        self.streams.insert(id, StreamVolume::new());
        Ok(())
    }

    /// Unregister a stream
    pub fn unregister_stream(&mut self, id: VolumeStreamId) {
        self.streams.remove(&id);
    }

    /// Set the volume gain for a specific stream
    pub fn set_volume(&self, stream_id: VolumeStreamId, gain: f32) -> Result<(), AudioError> {
        let stream = self.streams.get(&stream_id)
            .ok_or(AudioError::InvalidParameter)?;
        stream.set_gain(gain);
        log_debug!("Volume: stream {} gain set to {:.2}", stream_id, gain);
        Ok(())
    }

    /// Get the volume gain for a specific stream
    pub fn get_volume(&self, stream_id: VolumeStreamId) -> Result<f32, AudioError> {
        let stream = self.streams.get(&stream_id)
            .ok_or(AudioError::InvalidParameter)?;
        Ok(stream.get_gain())
    }

    /// Mute a specific stream
    pub fn set_mute(&self, stream_id: VolumeStreamId, mute: bool) -> Result<(), AudioError> {
        let stream = self.streams.get(&stream_id)
            .ok_or(AudioError::InvalidParameter)?;
        stream.set_mute(mute);
        log_debug!("Volume: stream {} muted={}", stream_id, mute);
        Ok(())
    }

    /// Check if a stream is muted
    pub fn is_muted(&self, stream_id: VolumeStreamId) -> Result<bool, AudioError> {
        let stream = self.streams.get(&stream_id)
            .ok_or(AudioError::InvalidParameter)?;
        Ok(stream.is_muted())
    }

    /// Set master volume (applied to all streams)
    pub fn set_master_volume(&self, gain: f32) {
        self.master_volume.set_gain(gain);
        log_debug!("Volume: master gain set to {:.2}", gain);
    }

    /// Get master volume
    pub fn get_master_volume(&self) -> f32 {
        self.master_volume.get_gain()
    }

    /// Mute all audio (master mute)
    pub fn set_master_mute(&self, mute: bool) {
        self.master_volume.set_mute(mute);
        log_debug!("Volume: master muted={}", mute);
    }

    /// Check if master is muted
    pub fn is_master_muted(&self) -> bool {
        self.master_volume.is_muted()
    }

    /// Get the effective gain for a stream (master * stream * mute)
    pub fn effective_gain(&self, stream_id: VolumeStreamId) -> f32 {
        let master = self.master_volume.effective_gain();
        let stream_gain = self.streams.get(&stream_id)
            .map(|s| s.effective_gain())
            .unwrap_or(0.0);
        master * stream_gain
    }

    /// Apply volume gain to a PCM buffer in-place
    pub fn apply_volume(&self, stream_id: VolumeStreamId, buffer: &mut PcmBuffer) -> Result<(), AudioError> {
        if !self.streams.contains_key(&stream_id) {
            return Err(AudioError::InvalidParameter);
        }

        let gain = self.effective_gain(stream_id);

        if gain == 0.0 {
            // Muted - zero the buffer
            for byte in buffer.data.iter_mut() {
                *byte = 0;
            }
            return Ok(());
        }

        if (gain - 1.0).abs() < 1e-6 {
            // Unity gain - no processing needed
            return Ok(());
        }

        // Apply gain based on sample format
        let bytes_per_sample = buffer.info.sample_format.bytes_per_sample();

        match bytes_per_sample {
            2 => Self::apply_gain_s16(&mut buffer.data, gain),
            4 => Self::apply_gain_f32(&mut buffer.data, gain),
            _ => Self::apply_gain_generic(&mut buffer.data, gain),
        }

        Ok(())
    }

    /// Apply gain to 16-bit signed samples
    fn apply_gain_s16(data: &mut [u8], gain: f32) {
        let chunks = data.len() / 2;
        for i in 0..chunks {
            let offset = i * 2;
            let lo = data[offset] as u16;
            let hi = data[offset + 1] as u16;
            let sample = (hi << 8 | lo) as i16;

            let amplified = (sample as f32 * gain).clamp(i16::MIN as f32, i16::MAX as f32) as i16;
            let v = amplified as u16;
            data[offset] = (v & 0xFF) as u8;
            data[offset + 1] = ((v >> 8) & 0xFF) as u8;
        }
    }

    /// Apply gain to 32-bit float samples
    fn apply_gain_f32(data: &mut [u8], gain: f32) {
        let chunks = data.len() / 4;
        for i in 0..chunks {
            let offset = i * 4;
            let bytes = [
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ];
            let sample = f32::from_le_bytes(bytes);
            let amplified = (sample * gain).clamp(-1.0, 1.0);
            let out = amplified.to_le_bytes();
            data[offset] = out[0];
            data[offset + 1] = out[1];
            data[offset + 2] = out[2];
            data[offset + 3] = out[3];
        }
    }

    /// Apply gain generically (byte-level)
    fn apply_gain_generic(data: &mut [u8], gain: f32) {
        for byte in data.iter_mut() {
            *byte = ((*byte as f32 * gain).clamp(0.0, 255.0)) as u8;
        }
    }

    /// Get number of registered streams
    pub fn stream_count(&self) -> usize {
        self.streams.len()
    }
}
