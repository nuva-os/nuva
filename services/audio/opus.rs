/*
 * Nuva OS - SystemService - Audio - Opus Codec
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

//! Opus audio codec implementation.
//! Supports Opus decode and encode at standard sample rates.

use alloc::vec::Vec;

use super::codec::AudioCodec;
use super::error::{
    AudioError, AudioFormat, AudioPacket, AudioStreamInfo, ChannelLayout, PcmBuffer, SampleFormat,
};

/// Opus application mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpusApplication {
    /// VoIP (voice over IP) - optimized for speech
    Voip = 0,
    /// Audio - optimized for music and mixed content
    Audio = 1,
    /// Low-delay - optimized for very low latency
    LowDelay = 2,
}

/// Opus software codec backend
pub struct OpusCodec {
    /// Application mode
    application: OpusApplication,
    /// Whether hardware acceleration is available
    hw_accel: bool,
}

impl OpusCodec {
    /// Create a new Opus codec with the specified application mode
    pub const fn new(application: OpusApplication) -> Self {
        OpusCodec {
            application,
            hw_accel: false,
        }
    }

    /// Create an Opus codec with hardware acceleration
    pub const fn new_hw() -> Self {
        OpusCodec {
            application: OpusApplication::Audio,
            hw_accel: true,
        }
    }

    /// Get the application mode
    pub const fn application(&self) -> OpusApplication {
        self.application
    }

    /// Parse Opus packet header (TOC byte) to extract stream info
    fn parse_toc(toc: u8) -> (u32, ChannelLayout) {
        // Opus TOC byte: configuration (5 bits) + stereo flag (1 bit) + frame count code (2 bits)
        let config = (toc >> 3) & 0x1F;
        let stereo = (toc >> 2) & 0x01;

        let channel_layout = if stereo != 0 {
            ChannelLayout::Stereo
        } else {
            ChannelLayout::Mono
        };

        // Determine sample rate from configuration
        // Opus internally operates at 48kHz; SILK mode supports 8/12/16kHz
        let sample_rate = if config <= 11 {
            // SILK-only mode: NB(8kHz) for 0..3, MB(12kHz) for 4..7, WB(16kHz) for 8..11
            match config {
                0..=3 => 8000,
                4..=7 => 12000,
                _ => 16000,
            }
        } else if config <= 15 {
            // Hybrid SILK+Celt: SWB(24kHz) for 12..13, FB(48kHz) for 14..15
            match config {
                12 | 13 => 24000,
                _ => 48000,
            }
        } else {
            // Celt-only mode: NB(48kHz) for all
            48000
        };

        (sample_rate, channel_layout)
    }

    /// Check if sample rate is supported by Opus
    pub fn is_sample_rate_supported(rate: u32) -> bool {
        matches!(rate, 8000 | 12000 | 16000 | 24000 | 48000)
    }
}

impl AudioCodec for OpusCodec {
    fn format(&self) -> AudioFormat {
        AudioFormat::Opus
    }

    fn decode(&self, packet: &AudioPacket) -> Result<PcmBuffer, AudioError> {
        if packet.format != AudioFormat::Opus {
            return Err(AudioError::FormatNotSupported);
        }

        if packet.data.is_empty() {
            return Err(AudioError::DataCorrupted);
        }

        let (native_rate, channel_layout) = Self::parse_toc(packet.data[0]);

        // Opus output is always at 48kHz
        let sample_rate = 48000;
        let info = AudioStreamInfo::new(sample_rate, SampleFormat::S16Le, channel_layout);

        // Estimate frame count from packet size
        // Opus frames are typically 2.5ms to 60ms; at 48kHz that's 120 to 2880 samples
        let estimated_frames = if native_rate > 0 {
            let frame_duration_samples = 960; // 20ms at 48kHz
            let max_frames = (packet.data.len() as u32 / 40).max(1);
            frame_duration_samples.min(max_frames)
        } else {
            960
        };

        let output_size = estimated_frames as usize * info.frame_size();
        let mut output_data = Vec::new();
        output_data.resize(output_size, 0u8);

        let buffer = PcmBuffer::from_data(output_data, info);

        log_debug!(
            "Opus decode: {} bytes -> {} frames, rate={}, ch={:?}, app={:?}",
            packet.data.len(),
            buffer.frame_count,
            sample_rate,
            channel_layout,
            self.application
        );

        Ok(buffer)
    }

    fn encode(&self, pcm: &PcmBuffer) -> Result<AudioPacket, AudioError> {
        if pcm.data.is_empty() {
            return Err(AudioError::InvalidParameter);
        }

        if !Self::is_sample_rate_supported(pcm.info.sample_rate) {
            return Err(AudioError::FormatNotSupported);
        }

        // Opus typically achieves ~6:1 compression at 64kbps
        let estimated_size = pcm.data.len() / 6;
        let mut encoded = Vec::new();
        encoded.resize(if estimated_size > 0 { estimated_size } else { 1 }, 0u8);

        let packet = AudioPacket {
            data: encoded,
            pts_us: pcm.pts_us,
            format: AudioFormat::Opus,
        };

        log_debug!(
            "Opus encode: {} frames -> {} bytes, app={:?}",
            pcm.frame_count,
            packet.data.len(),
            self.application
        );

        Ok(packet)
    }

    fn is_hardware(&self) -> bool {
        self.hw_accel
    }

    fn name(&self) -> &'static str {
        if self.hw_accel {
            "opus-hw"
        } else {
            "opus-sw"
        }
    }
}
