/*
 * Nuva OS - SystemService - Video - Error Model
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

//! Video service specific error types and video data types.

use core::fmt;

/// Video service error type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoError {
    /// Video format not supported
    FormatNotSupported = 0,
    /// Video data corrupted
    DataCorrupted = 1,
    /// Out of memory
    OutOfMemory = 2,
    /// Hardware decode/encode error
    HardwareError = 3,
    /// Operation timed out
    Timeout = 4,
    /// Invalid parameter
    InvalidParameter = 5,
    /// Service not initialized
    NotInitialized = 6,
    /// Codec not found
    CodecNotFound = 7,
    /// Decoder not found
    DecoderNotFound = 8,
    /// Encoder not found
    EncoderNotFound = 9,
    /// Frame buffer exhausted
    FrameBufferExhausted = 10,
}

impl fmt::Display for VideoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VideoError::FormatNotSupported => write!(f, "Video format not supported"),
            VideoError::DataCorrupted => write!(f, "Video data corrupted"),
            VideoError::OutOfMemory => write!(f, "Out of memory"),
            VideoError::HardwareError => write!(f, "Hardware video error"),
            VideoError::Timeout => write!(f, "Operation timed out"),
            VideoError::InvalidParameter => write!(f, "Invalid video parameter"),
            VideoError::NotInitialized => write!(f, "Video service not initialized"),
            VideoError::CodecNotFound => write!(f, "Codec not found"),
            VideoError::DecoderNotFound => write!(f, "Decoder not found"),
            VideoError::EncoderNotFound => write!(f, "Encoder not found"),
            VideoError::FrameBufferExhausted => write!(f, "Frame buffer exhausted"),
        }
    }
}

/// Video format identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoFormat {
    /// H.264/AVC
    H264 = 0,
    /// H.265/HEVC
    Hevc = 1,
    /// VP9
    Vp9 = 2,
    /// AV1
    Av1 = 3,
    /// Unknown format
    Unknown = 255,
}

impl VideoFormat {
    /// Convert from format ID used in core_processing::format_detect
    pub const fn from_format_id(id: u32) -> Self {
        match id {
            10 => VideoFormat::H264,
            11 => VideoFormat::Hevc,
            12 => VideoFormat::Vp9,
            13 => VideoFormat::Av1,
            _ => VideoFormat::Unknown,
        }
    }

    /// Convert to format ID used in core_processing::format_detect
    pub const fn to_format_id(self) -> u32 {
        match self {
            VideoFormat::H264 => 10,
            VideoFormat::Hevc => 11,
            VideoFormat::Vp9 => 12,
            VideoFormat::Av1 => 13,
            VideoFormat::Unknown => 255,
        }
    }
}

/// Pixel format for decoded frames
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelFormat {
    /// YUV 4:2:0 planar
    Yuv420P = 0,
    /// YUV 4:2:2 planar
    Yuv422P = 1,
    /// YUV 4:4:4 planar
    Yuv444P = 2,
    /// NV12 (YUV 4:2:0 semi-planar)
    Nv12 = 3,
    /// NV21 (YUV 4:2:0 semi-planar, VU order)
    Nv21 = 4,
    /// RGBA 8:8:8:8
    Rgba8888 = 5,
    /// BGRA 8:8:8:8
    Bgra8888 = 6,
}

/// Video packet for encoded bitstream data
#[derive(Debug, Clone)]
pub struct VideoPacket {
    /// Packet data
    pub data: alloc::vec::Vec<u8>,
    /// Presentation timestamp in microseconds
    pub pts_us: i64,
    /// Decode timestamp in microseconds
    pub dts_us: i64,
    /// Whether this is a keyframe
    pub keyframe: bool,
    /// Video format of this packet
    pub format: VideoFormat,
}

impl VideoPacket {
    /// Create an empty video packet
    pub fn new(format: VideoFormat) -> Self {
        VideoPacket {
            data: alloc::vec::Vec::new(),
            pts_us: 0,
            dts_us: 0,
            keyframe: false,
            format,
        }
    }

    /// Create a video packet from data
    pub fn from_data(data: alloc::vec::Vec<u8>, format: VideoFormat) -> Self {
        VideoPacket {
            data,
            pts_us: 0,
            dts_us: 0,
            keyframe: false,
            format,
        }
    }
}

/// Video decode configuration
#[derive(Debug, Clone, Copy)]
pub struct VideoDecodeConfig {
    /// Input video format
    pub format: VideoFormat,
    /// Output pixel format
    pub output_pixel_format: PixelFormat,
    /// Maximum output width
    pub max_width: u32,
    /// Maximum output height
    pub max_height: u32,
    /// Whether to use hardware acceleration if available
    pub hw_accel: bool,
}

impl VideoDecodeConfig {
    /// Create a default decode config for the given format
    pub const fn new(format: VideoFormat) -> Self {
        VideoDecodeConfig {
            format,
            output_pixel_format: PixelFormat::Nv12,
            max_width: 3840,
            max_height: 2160,
            hw_accel: true,
        }
    }
}

/// Video encode configuration
#[derive(Debug, Clone, Copy)]
pub struct VideoEncodeConfig {
    /// Output video format
    pub format: VideoFormat,
    /// Input pixel format
    pub input_pixel_format: PixelFormat,
    /// Input width
    pub width: u32,
    /// Input height
    pub height: u32,
    /// Target bitrate in bits per second
    pub bitrate_bps: u32,
    /// Target frame rate numerator
    pub frame_rate_num: u32,
    /// Target frame rate denominator
    pub frame_rate_den: u32,
    /// Whether to use hardware acceleration if available
    pub hw_accel: bool,
}

impl VideoEncodeConfig {
    /// Create a default encode config for the given format
    pub const fn new(format: VideoFormat, width: u32, height: u32) -> Self {
        VideoEncodeConfig {
            format,
            input_pixel_format: PixelFormat::Nv12,
            width,
            height,
            bitrate_bps: 5_000_000,
            frame_rate_num: 30,
            frame_rate_den: 1,
            hw_accel: true,
        }
    }
}

/// Decoder instance identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DecoderId(pub u64);

/// Encoder instance identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EncoderId(pub u64);
