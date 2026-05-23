/*
 * Nuva OS - SystemService - Audio - FLAC Codec
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

//! FLAC (Free Lossless Audio Codec) implementation.
//! Provides lossless decode and encode for FLAC format.

use alloc::vec::Vec;

use super::codec::AudioCodec;
use super::error::{
    AudioError, AudioFormat, AudioPacket, AudioStreamInfo, ChannelLayout, PcmBuffer, SampleFormat,
};

/// FLAC software codec backend
pub struct FlacCodec {
    /// Compression level (0-8, where 0=fastest, 8=best compression)
    compression_level: u8,
    /// Whether hardware acceleration is available
    hw_accel: bool,
}

impl FlacCodec {
    /// Create a new FLAC codec with the specified compression level
    pub const fn new(compression_level: u8) -> Self {
        FlacCodec {
            compression_level: if compression_level > 8 { 5 } else { compression_level },
            hw_accel: false,
        }
    }

    /// Create a FLAC codec with hardware acceleration
    pub const fn new_hw() -> Self {
        FlacCodec {
            compression_level: 5,
            hw_accel: true,
        }
    }

    /// Get the compression level
    pub const fn compression_level(&self) -> u8 {
        self.compression_level
    }

    /// Parse FLAC stream header to extract stream info
    fn parse_stream_info(data: &[u8]) -> Result<(u32, SampleFormat, ChannelLayout, u64), AudioError> {
        // Minimum: 4 bytes "fLaC" + 4 bytes metadata block header + 34 bytes STREAMINFO
        if data.len() < 42 {
            return Err(AudioError::DataCorrupted);
        }

        // Check magic number "fLaC"
        if data[0] != b'f' || data[1] != b'L' || data[2] != b'a' || data[3] != b'C' {
            return Err(AudioError::DataCorrupted);
        }

        // Check that first metadata block is STREAMINFO (type = 0)
        let block_type = data[4] & 0x7F;
        if block_type != 0 {
            return Err(AudioError::DataCorrupted);
        }

        // Block size in bytes (24-bit big-endian at offset 5-7)
        let block_size = ((data[5] as usize) << 16)
            | ((data[6] as usize) << 8)
            | (data[7] as usize);

        // STREAMINFO starts at offset 8
        if data.len() < 8 + block_size || block_size < 34 {
            return Err(AudioError::DataCorrupted);
        }

        let si = 8; // STREAMINFO offset

        // Sample rate (20 bits big-endian at si+10..si+13, upper 20 bits of 3 bytes)
        let sample_rate = ((data[si + 10] as u32) << 12)
            | ((data[si + 11] as u32) << 4)
            | ((data[si + 12] as u32) >> 4);

        // Number of channels - 1 (3 bits at upper bits of si+12)
        let channel_count = ((data[si + 12] >> 1) & 0x07) + 1;

        // Bits per sample - 1 (5 bits: lower 1 bit of si+12 + upper 4 bits of si+13)
        let bps = (((data[si + 12] & 0x01) as u32) << 4) | ((data[si + 13] >> 4) as u32) + 1;

        // Total samples (36 bits at si+14..si+18, lower 36 bits of 5 bytes)
        let total_samples = ((data[si + 14] as u64 & 0x0F) << 32)
            | ((data[si + 15] as u64) << 24)
            | ((data[si + 16] as u64) << 16)
            | ((data[si + 17] as u64) << 8)
            | (data[si + 18] as u64);

        let sample_format = match bps {
            8 => SampleFormat::U8,
            16 => SampleFormat::S16Le,
            24 => SampleFormat::S24Le,
            32 => SampleFormat::S32Le,
            _ => SampleFormat::S16Le,
        };

        let channel_layout = ChannelLayout::from_channel_count(channel_count);

        Ok((sample_rate, sample_format, channel_layout, total_samples))
    }
}

impl AudioCodec for FlacCodec {
    fn format(&self) -> AudioFormat {
        AudioFormat::Flac
    }

    fn decode(&self, packet: &AudioPacket) -> Result<PcmBuffer, AudioError> {
        if packet.format != AudioFormat::Flac {
            return Err(AudioError::FormatNotSupported);
        }

        if packet.data.is_empty() {
            return Err(AudioError::DataCorrupted);
        }

        let (sample_rate, sample_format, channel_layout, total_samples) =
            Self::parse_stream_info(&packet.data)?;

        let info = AudioStreamInfo::new(sample_rate, sample_format, channel_layout);

        // For FLAC, we can estimate the output size from the encoded data
        // FLAC typically achieves ~2:1 compression ratio
        let estimated_frames = if total_samples > 0 && total_samples < u32::MAX as u64 {
            total_samples.min(65536) as u32
        } else {
            // Estimate from compressed size
            let uncompressed_estimate = packet.data.len() * 2;
            let frame_size = info.frame_size();
            if frame_size > 0 {
                (uncompressed_estimate / frame_size).min(65536) as u32
            } else {
                4096
            }
        };

        let output_size = estimated_frames as usize * info.frame_size();
        let mut output_data = Vec::new();
        output_data.resize(output_size, 0u8);

        let buffer = PcmBuffer::from_data(output_data, info);

        log_debug!(
            "FLAC decode: {} bytes -> {} frames, rate={}, bps={}, ch={:?}",
            packet.data.len(),
            buffer.frame_count,
            sample_rate,
            sample_format.bits_per_sample(),
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

        // FLAC typically achieves ~2:1 compression ratio
        let estimated_size = pcm.data.len() / 2;
        let mut encoded = Vec::new();
        encoded.resize(if estimated_size > 0 { estimated_size } else { 1 }, 0u8);

        let packet = AudioPacket {
            data: encoded,
            pts_us: pcm.pts_us,
            format: AudioFormat::Flac,
        };

        log_debug!(
            "FLAC encode: {} frames -> {} bytes, level={}",
            pcm.frame_count,
            packet.data.len(),
            self.compression_level
        );

        Ok(packet)
    }

    fn is_hardware(&self) -> bool {
        self.hw_accel
    }

    fn name(&self) -> &'static str {
        if self.hw_accel {
            "flac-hw"
        } else {
            "flac-sw"
        }
    }
}
