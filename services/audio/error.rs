/*
 * Nuva OS - SystemService - Audio - Error Model
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

//! Audio service specific error types and audio data types.

use core::fmt;
use alloc::vec::Vec;

/// Audio service error type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioError {
    /// Audio format not supported
    FormatNotSupported = 0,
    /// Audio data corrupted
    DataCorrupted = 1,
    /// Latency exceeded
    LatencyExceeded = 2,
    /// Out of memory
    OutOfMemory = 3,
    /// Invalid parameter
    InvalidParameter = 4,
    /// Service not initialized
    NotInitialized = 5,
    /// Decoder not found
    DecoderNotFound = 6,
    /// Encoder not found
    EncoderNotFound = 7,
    /// Codec not found
    CodecNotFound = 8,
    /// Buffer overflow
    BufferOverflow = 9,
    /// Sample rate conversion failed
    ResampleError = 10,
    /// Mixer channel overflow
    MixerOverflow = 11,
}

impl fmt::Display for AudioError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AudioError::FormatNotSupported => write!(f, "Audio format not supported"),
            AudioError::DataCorrupted => write!(f, "Audio data corrupted"),
            AudioError::LatencyExceeded => write!(f, "Audio latency exceeded"),
            AudioError::OutOfMemory => write!(f, "Out of memory"),
            AudioError::InvalidParameter => write!(f, "Invalid audio parameter"),
            AudioError::NotInitialized => write!(f, "Audio service not initialized"),
            AudioError::DecoderNotFound => write!(f, "Decoder not found"),
            AudioError::EncoderNotFound => write!(f, "Encoder not found"),
            AudioError::CodecNotFound => write!(f, "Codec not found"),
            AudioError::BufferOverflow => write!(f, "Audio buffer overflow"),
            AudioError::ResampleError => write!(f, "Sample rate conversion failed"),
            AudioError::MixerOverflow => write!(f, "Mixer channel overflow"),
        }
    }
}

/// Audio format identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioFormat {
    /// AAC-LC (Low Complexity)
    AacLc = 0,
    /// Opus
    Opus = 1,
    /// FLAC (Free Lossless Audio Codec)
    Flac = 2,
    /// PCM (Pulse Code Modulation)
    Pcm = 3,
    /// Unknown format
    Unknown = 255,
}

impl AudioFormat {
    /// Convert from format ID
    pub const fn from_id(id: u32) -> Self {
        match id {
            0 => AudioFormat::AacLc,
            1 => AudioFormat::Opus,
            2 => AudioFormat::Flac,
            3 => AudioFormat::Pcm,
            _ => AudioFormat::Unknown,
        }
    }

    /// Convert to format ID
    pub const fn to_id(self) -> u32 {
        match self {
            AudioFormat::AacLc => 0,
            AudioFormat::Opus => 1,
            AudioFormat::Flac => 2,
            AudioFormat::Pcm => 3,
            AudioFormat::Unknown => 255,
        }
    }
}

/// Sample format (bit depth and signedness)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SampleFormat {
    /// 8-bit unsigned
    U8 = 0,
    /// 16-bit signed (little-endian)
    S16Le = 1,
    /// 16-bit signed (big-endian)
    S16Be = 2,
    /// 24-bit signed (little-endian, packed in 3 bytes)
    S24Le = 3,
    /// 32-bit signed (little-endian)
    S32Le = 4,
    /// 32-bit IEEE float (little-endian)
    F32Le = 5,
    /// 64-bit IEEE float (little-endian)
    F64Le = 6,
}

impl SampleFormat {
    /// Get the number of bytes per sample
    pub const fn bytes_per_sample(self) -> usize {
        match self {
            SampleFormat::U8 => 1,
            SampleFormat::S16Le | SampleFormat::S16Be => 2,
            SampleFormat::S24Le => 3,
            SampleFormat::S32Le | SampleFormat::F32Le => 4,
            SampleFormat::F64Le => 8,
        }
    }

    /// Get the number of bits per sample
    pub const fn bits_per_sample(self) -> u32 {
        match self {
            SampleFormat::U8 => 8,
            SampleFormat::S16Le | SampleFormat::S16Be => 16,
            SampleFormat::S24Le => 24,
            SampleFormat::S32Le | SampleFormat::F32Le => 32,
            SampleFormat::F64Le => 64,
        }
    }
}

/// Channel layout
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelLayout {
    /// Mono (1 channel)
    Mono = 1,
    /// Stereo (2 channels: L, R)
    Stereo = 2,
    /// 2.1 (3 channels: L, R, LFE)
    Sur21 = 3,
    /// 4.0 (4 channels: L, R, Ls, Rs)
    Sur40 = 4,
    /// 5.1 (6 channels: L, R, C, LFE, Ls, Rs)
    Sur51 = 6,
    /// 7.1 (8 channels: L, R, C, LFE, Ls, Rs, Rls, Rrs)
    Sur71 = 8,
}

impl ChannelLayout {
    /// Get the number of channels
    pub const fn channel_count(self) -> u32 {
        match self {
            ChannelLayout::Mono => 1,
            ChannelLayout::Stereo => 2,
            ChannelLayout::Sur21 => 3,
            ChannelLayout::Sur40 => 4,
            ChannelLayout::Sur51 => 6,
            ChannelLayout::Sur71 => 8,
        }
    }

    /// Create from channel count
    pub const fn from_channel_count(count: u32) -> Self {
        match count {
            1 => ChannelLayout::Mono,
            2 => ChannelLayout::Stereo,
            3 => ChannelLayout::Sur21,
            4 => ChannelLayout::Sur40,
            6 => ChannelLayout::Sur51,
            8 => ChannelLayout::Sur71,
            _ => ChannelLayout::Stereo,
        }
    }
}

/// Audio stream information
#[derive(Debug, Clone, Copy)]
pub struct AudioStreamInfo {
    /// Sample rate in Hz
    pub sample_rate: u32,
    /// Sample format
    pub sample_format: SampleFormat,
    /// Channel layout
    pub channel_layout: ChannelLayout,
    /// Bit rate in bits per second (0 if unknown)
    pub bit_rate: u32,
}

impl AudioStreamInfo {
    /// Create a new audio stream info
    pub const fn new(
        sample_rate: u32,
        sample_format: SampleFormat,
        channel_layout: ChannelLayout,
    ) -> Self {
        AudioStreamInfo {
            sample_rate,
            sample_format,
            channel_layout,
            bit_rate: 0,
        }
    }

    /// Calculate frame size in bytes (one sample across all channels)
    pub const fn frame_size(&self) -> usize {
        self.sample_format.bytes_per_sample() * self.channel_layout.channel_count() as usize
    }

    /// Calculate byte rate
    pub const fn byte_rate(&self) -> u64 {
        self.sample_rate as u64 * self.frame_size() as u64
    }
}

/// PCM audio buffer
#[derive(Debug, Clone)]
pub struct PcmBuffer {
    /// PCM sample data (interleaved)
    pub data: alloc::vec::Vec<u8>,
    /// Stream information describing the buffer format
    pub info: AudioStreamInfo,
    /// Presentation timestamp in microseconds
    pub pts_us: i64,
    /// Number of frames (samples per channel) in this buffer
    pub frame_count: u32,
}

impl PcmBuffer {
    /// Create an empty PCM buffer with the given stream info
    pub fn new(info: AudioStreamInfo) -> Self {
        PcmBuffer {
            data: alloc::vec::Vec::new(),
            info,
            pts_us: 0,
            frame_count: 0,
        }
    }

    /// Create a PCM buffer from raw data
    pub fn from_data(data: alloc::vec::Vec<u8>, info: AudioStreamInfo) -> Self {
        let frame_size = info.frame_size();
        let frame_count = if frame_size > 0 {
            (data.len() / frame_size) as u32
        } else {
            0
        };
        PcmBuffer {
            data,
            info,
            pts_us: 0,
            frame_count,
        }
    }

    /// Calculate duration in microseconds
    pub fn duration_us(&self) -> u64 {
        if self.info.sample_rate == 0 {
            return 0;
        }
        (self.frame_count as u64 * 1_000_000) / self.info.sample_rate as u64
    }
}

/// Decoder instance identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DecoderId(pub u64);

/// Encoder instance identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EncoderId(pub u64);

/// Audio decode configuration
#[derive(Debug, Clone, Copy)]
pub struct AudioDecodeConfig {
    /// Input audio format
    pub format: AudioFormat,
    /// Output sample format
    pub output_sample_format: SampleFormat,
    /// Output sample rate (0 = native)
    pub output_sample_rate: u32,
    /// Output channel layout
    pub output_channel_layout: ChannelLayout,
    /// Whether to use hardware acceleration if available
    pub hw_accel: bool,
}

impl AudioDecodeConfig {
    /// Create a default decode config for the given format
    pub const fn new(format: AudioFormat) -> Self {
        AudioDecodeConfig {
            format,
            output_sample_format: SampleFormat::S16Le,
            output_sample_rate: 0,
            output_channel_layout: ChannelLayout::Stereo,
            hw_accel: true,
        }
    }
}

/// Audio encode configuration
#[derive(Debug, Clone, Copy)]
pub struct AudioEncodeConfig {
    /// Output audio format
    pub format: AudioFormat,
    /// Input sample format
    pub input_sample_format: SampleFormat,
    /// Input sample rate
    pub sample_rate: u32,
    /// Input channel layout
    pub channel_layout: ChannelLayout,
    /// Target bit rate in bits per second (0 = default)
    pub bit_rate: u32,
    /// Whether to use hardware acceleration if available
    pub hw_accel: bool,
}

impl AudioEncodeConfig {
    /// Create a default encode config for the given format
    pub const fn new(format: AudioFormat, sample_rate: u32) -> Self {
        AudioEncodeConfig {
            format,
            input_sample_format: SampleFormat::S16Le,
            sample_rate,
            channel_layout: ChannelLayout::Stereo,
            bit_rate: 0,
            hw_accel: true,
        }
    }
}

/// Encoded audio packet
#[derive(Debug, Clone)]
pub struct AudioPacket {
    /// Encoded bitstream data
    pub data: alloc::vec::Vec<u8>,
    /// Presentation timestamp in microseconds
    pub pts_us: i64,
    /// Audio format of this packet
    pub format: AudioFormat,
}

impl AudioPacket {
    /// Create an empty audio packet
    pub fn new(format: AudioFormat) -> Self {
        AudioPacket {
            data: alloc::vec::Vec::new(),
            pts_us: 0,
            format,
        }
    }

    /// Create an audio packet from data
    pub fn from_data(data: alloc::vec::Vec<u8>, format: AudioFormat) -> Self {
        AudioPacket {
            data,
            pts_us: 0,
            format,
        }
    }
}
