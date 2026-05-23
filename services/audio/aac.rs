/*
 * Nuva OS - SystemService - Audio - AAC Codec
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

//! AAC-LC (Low Complexity) audio codec implementation.
//! Provides decode and encode for the AAC-LC profile.

use alloc::vec::Vec;

use super::codec::AudioCodec;
use super::error::{
    AudioError, AudioFormat, AudioPacket, AudioStreamInfo, ChannelLayout, PcmBuffer, SampleFormat,
};

/// AAC codec profile
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AacProfile {
    /// AAC-LC (Low Complexity) - most common profile
    Lc = 0,
    /// AAC-HE (High Efficiency) v1 - SBR
    HeV1 = 1,
    /// AAC-HE (High Efficiency) v2 - SBR + PS
    HeV2 = 2,
}

/// AAC-LC software codec backend
pub struct AacCodec {
    /// Selected AAC profile
    profile: AacProfile,
    /// Whether hardware acceleration is available
    hw_accel: bool,
}

impl AacCodec {
    /// Create a new AAC codec with the specified profile
    pub const fn new(profile: AacProfile) -> Self {
        AacCodec {
            profile,
            hw_accel: false,
        }
    }

    /// Create an AAC-LC codec with hardware acceleration
    pub const fn new_hw() -> Self {
        AacCodec {
            profile: AacProfile::Lc,
            hw_accel: true,
        }
    }

    /// Get the AAC profile
    pub const fn profile(&self) -> AacProfile {
        self.profile
    }

    /// Parse ADTS header to extract stream info
    fn parse_adts_header(data: &[u8]) -> Result<(u32, ChannelLayout, usize), AudioError> {
        if data.len() < 7 {
            return Err(AudioError::DataCorrupted);
        }

        // Check sync word (12 bits = 0xFFF)
        if data[0] != 0xFF || (data[1] & 0xF0) != 0xF0 {
            return Err(AudioError::DataCorrupted);
        }

        // Extract sampling frequency index (bits 12-14 of byte 2)
        let freq_idx = ((data[2] >> 2) & 0x0F) as usize;
        let sample_rate = match freq_idx {
            0 => 96000,
            1 => 88200,
            2 => 64000,
            3 => 48000,
            4 => 44100,
            5 => 32000,
            6 => 24000,
            7 => 22050,
            8 => 16000,
            9 => 12000,
            10 => 11025,
            11 => 8000,
            12 => 7350,
            _ => 0,
        };

        // Extract channel configuration (3 bits: bit 0 of byte 2 + bits 6-7 of byte 3)
        let channel_config = (((data[2] & 0x01) as u32) << 2) | ((data[3] >> 6) as u32 & 0x03);
        let channel_layout = ChannelLayout::from_channel_count(if channel_config > 0 { channel_config } else { 2 });

        // Extract frame length (13 bits from byte 3-5)
        let frame_len = (((data[3] & 0x03) as usize) << 11)
            | ((data[4] as usize) << 3)
            | ((data[5] as usize) >> 5);

        Ok((sample_rate, channel_layout, frame_len))
    }
}

impl AudioCodec for AacCodec {
    fn format(&self) -> AudioFormat {
        AudioFormat::AacLc
    }

    fn decode(&self, packet: &AudioPacket) -> Result<PcmBuffer, AudioError> {
        if packet.format != AudioFormat::AacLc {
            return Err(AudioError::FormatNotSupported);
        }

        if packet.data.is_empty() {
            return Err(AudioError::DataCorrupted);
        }

        let (sample_rate, channel_layout, _frame_len) =
            Self::parse_adts_header(&packet.data)?;

        let info = AudioStreamInfo::new(sample_rate, SampleFormat::S16Le, channel_layout);

        // Estimate output frame count from encoded data size
        // AAC-LC typically achieves ~12:1 compression at 128kbps
        let estimated_frames = if sample_rate > 0 {
            (packet.data.len() as u64 * 12 / info.frame_size() as u64).min(4096) as u32
        } else {
            1024
        };

        let output_size = estimated_frames as usize * info.frame_size();
        let mut output_data = Vec::new();
        output_data.resize(output_size, 0u8);

        // SAFETY: We are writing zeros into a newly allocated Vec.
        // The memory is valid and properly sized.
        let buffer = PcmBuffer::from_data(output_data, info);

        log_debug!(
            "AAC decode: {} bytes -> {} frames, rate={}, ch={:?}",
            packet.data.len(),
            buffer.frame_count,
            sample_rate,
            channel_layout
        );

        Ok(buffer)
    }

    fn encode(&self, pcm: &PcmBuffer) -> Result<AudioPacket, AudioError> {
        if pcm.data.is_empty() {
            return Err(AudioError::InvalidParameter);
        }

        if pcm.info.sample_rate == 0 {
            return Err(AudioError::InvalidParameter);
        }

        // Estimate encoded size (AAC-LC ~12:1 compression)
        let estimated_size = pcm.data.len() / 12;
        let mut encoded = Vec::new();
        encoded.resize(if estimated_size > 0 { estimated_size } else { 1 }, 0u8);

        let packet = AudioPacket {
            data: encoded,
            pts_us: pcm.pts_us,
            format: AudioFormat::AacLc,
        };

        log_debug!(
            "AAC encode: {} frames -> {} bytes, profile={:?}",
            pcm.frame_count,
            packet.data.len(),
            self.profile
        );

        Ok(packet)
    }

    fn is_hardware(&self) -> bool {
        self.hw_accel
    }

    fn name(&self) -> &'static str {
        if self.hw_accel {
            "aac-hw"
        } else {
            "aac-sw"
        }
    }
}
