/*
 * Nuva OS - SystemService - Audio - PCM Codec
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

//! PCM (Pulse Code Modulation) passthrough codec.
//! Supports all sample rates, bit depths, and channel layouts.
//! Encode/decode are identity operations (passthrough).

use alloc::vec::Vec;

use super::codec::AudioCodec;
use super::error::{
    AudioError, AudioFormat, AudioPacket, AudioStreamInfo, ChannelLayout, PcmBuffer, SampleFormat,
};

/// Default sample rate for PCM
pub const DEFAULT_SAMPLE_RATE: u32 = 48000;

/// Default sample format
pub const DEFAULT_SAMPLE_FORMAT: SampleFormat = SampleFormat::S16Le;

/// Default channel layout
pub const DEFAULT_CHANNEL_LAYOUT: ChannelLayout = ChannelLayout::Stereo;

/// PCM passthrough codec backend
pub struct PcmCodec {
    /// Fixed sample rate (0 = accept any)
    sample_rate: u32,
    /// Fixed sample format (None = accept any)
    sample_format: Option<SampleFormat>,
    /// Fixed channel layout (None = accept any)
    channel_layout: Option<ChannelLayout>,
}

impl PcmCodec {
    /// Create a new PCM codec that accepts any format
    pub const fn new() -> Self {
        PcmCodec {
            sample_rate: 0,
            sample_format: None,
            channel_layout: None,
        }
    }

    /// Create a PCM codec restricted to a specific format
    pub const fn new_fixed(
        sample_rate: u32,
        sample_format: SampleFormat,
        channel_layout: ChannelLayout,
    ) -> Self {
        PcmCodec {
            sample_rate,
            sample_format: Some(sample_format),
            channel_layout: Some(channel_layout),
        }
    }

    /// Check if the given sample rate is a standard PCM rate
    pub fn is_standard_rate(rate: u32) -> bool {
        matches!(
            rate,
            8000 | 11025 | 16000 | 22050 | 32000 | 44100 | 48000 | 88200 | 96000 | 176400 | 192000
        )
    }

    /// Get all standard PCM sample rates
    pub const fn standard_rates() -> &'static [u32] {
        &[8000, 11025, 16000, 22050, 32000, 44100, 48000, 88200, 96000, 176400, 192000]
    }

    /// Validate stream info against codec constraints
    pub fn validate_stream_info(&self, info: &AudioStreamInfo) -> Result<(), AudioError> {
        if self.sample_rate > 0 && info.sample_rate != self.sample_rate {
            return Err(AudioError::FormatNotSupported);
        }
        if let Some(sf) = self.sample_format {
            if info.sample_format != sf {
                return Err(AudioError::FormatNotSupported);
            }
        }
        if let Some(cl) = self.channel_layout {
            if info.channel_layout != cl {
                return Err(AudioError::FormatNotSupported);
            }
        }
        Ok(())
    }
}

impl AudioCodec for PcmCodec {
    fn format(&self) -> AudioFormat {
        AudioFormat::Pcm
    }

    fn decode(&self, packet: &AudioPacket) -> Result<PcmBuffer, AudioError> {
        if packet.format != AudioFormat::Pcm {
            return Err(AudioError::FormatNotSupported);
        }

        if packet.data.is_empty() {
            return Err(AudioError::DataCorrupted);
        }

        // For PCM, encoded data IS the raw PCM data
        // Determine stream info from codec constraints or use defaults
        let sample_rate = if self.sample_rate > 0 {
            self.sample_rate
        } else {
            DEFAULT_SAMPLE_RATE
        };
        let sample_format = self.sample_format.unwrap_or(DEFAULT_SAMPLE_FORMAT);
        let channel_layout = self.channel_layout.unwrap_or(DEFAULT_CHANNEL_LAYOUT);

        let info = AudioStreamInfo::new(sample_rate, sample_format, channel_layout);
        let mut data = Vec::with_capacity(packet.data.len());
        data.extend_from_slice(&packet.data);

        let buffer = PcmBuffer::from_data(data, info);

        log_debug!(
            "PCM decode: {} bytes -> {} frames, rate={}, bps={}, ch={:?}",
            packet.data.len(),
            buffer.frame_count,
            sample_rate,
            sample_format.bits_per_sample(),
            channel_layout
        );

        Ok(buffer)
    }

    fn encode(&self, pcm: &PcmBuffer) -> Result<AudioPacket, AudioError> {
        self.validate_stream_info(&pcm.info)?;

        if pcm.data.is_empty() {
            return Err(AudioError::InvalidParameter);
        }

        // For PCM, encoding is a passthrough
        let mut data = Vec::with_capacity(pcm.data.len());
        data.extend_from_slice(&pcm.data);

        let packet = AudioPacket {
            data,
            pts_us: pcm.pts_us,
            format: AudioFormat::Pcm,
        };

        log_debug!(
            "PCM encode: {} frames -> {} bytes (passthrough)",
            pcm.frame_count,
            packet.data.len()
        );

        Ok(packet)
    }

    fn is_hardware(&self) -> bool {
        false
    }

    fn name(&self) -> &'static str {
        "pcm-passthrough"
    }
}
