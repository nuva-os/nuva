/*
 * Nuva OS - SystemService - Image - Error Model
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

//! Image service specific error types and image data types.

use core::fmt;

/// Image service error type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageError {
    /// Image format not supported
    FormatNotSupported = 0,
    /// Image data corrupted
    DataCorrupted = 1,
    /// Color space not supported
    ColorSpaceNotSupported = 2,
    /// Size limit exceeded
    SizeLimitExceeded = 3,
    /// Out of memory
    OutOfMemory = 4,
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
    /// Progressive decode incomplete
    ProgressiveIncomplete = 10,
    /// Hardware decode/encode error
    HardwareError = 11,
}

impl fmt::Display for ImageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ImageError::FormatNotSupported => write!(f, "Image format not supported"),
            ImageError::DataCorrupted => write!(f, "Image data corrupted"),
            ImageError::ColorSpaceNotSupported => write!(f, "Color space not supported"),
            ImageError::SizeLimitExceeded => write!(f, "Image size limit exceeded"),
            ImageError::OutOfMemory => write!(f, "Out of memory"),
            ImageError::InvalidParameter => write!(f, "Invalid image parameter"),
            ImageError::NotInitialized => write!(f, "Image service not initialized"),
            ImageError::CodecNotFound => write!(f, "Codec not found"),
            ImageError::DecoderNotFound => write!(f, "Decoder not found"),
            ImageError::EncoderNotFound => write!(f, "Encoder not found"),
            ImageError::ProgressiveIncomplete => write!(f, "Progressive decode incomplete"),
            ImageError::HardwareError => write!(f, "Hardware image error"),
        }
    }
}

/// Image format identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    /// JPEG
    Jpeg = 0,
    /// PNG
    Png = 1,
    /// WebP
    Webp = 2,
    /// BMP
    Bmp = 3,
    /// GIF
    Gif = 4,
    /// Unknown format
    Unknown = 255,
}

impl ImageFormat {
    /// Convert from format ID used in core_processing::format_detect
    pub const fn from_format_id(id: u32) -> Self {
        match id {
            1 => ImageFormat::Jpeg,
            2 => ImageFormat::Png,
            3 => ImageFormat::Webp,
            4 => ImageFormat::Bmp,
            5 => ImageFormat::Gif,
            _ => ImageFormat::Unknown,
        }
    }

    /// Convert to format ID used in core_processing::format_detect
    pub const fn to_format_id(self) -> u32 {
        match self {
            ImageFormat::Jpeg => 1,
            ImageFormat::Png => 2,
            ImageFormat::Webp => 3,
            ImageFormat::Bmp => 4,
            ImageFormat::Gif => 5,
            ImageFormat::Unknown => 255,
        }
    }

    /// Get file extension
    pub const fn extension(self) -> &'static str {
        match self {
            ImageFormat::Jpeg => "jpg",
            ImageFormat::Png => "png",
            ImageFormat::Webp => "webp",
            ImageFormat::Bmp => "bmp",
            ImageFormat::Gif => "gif",
            ImageFormat::Unknown => "",
        }
    }
}

/// Color space identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorSpace {
    /// RGB (3 channels, 8-bit each)
    Rgb8 = 0,
    /// RGBA (4 channels, 8-bit each)
    Rgba8 = 1,
    /// BGR (3 channels, 8-bit each)
    Bgr8 = 2,
    /// BGRA (4 channels, 8-bit each)
    Bgra8 = 3,
    /// Grayscale (1 channel, 8-bit)
    Gray8 = 4,
    /// Grayscale with alpha (2 channels, 8-bit each)
    GrayAlpha8 = 5,
    /// YCbCr (JPEG native color space)
    Ycbcr = 6,
    /// CMYK (4 channels)
    Cmyk = 7,
    /// L*a*b*
    Lab = 8,
}

impl ColorSpace {
    /// Get the number of channels for this color space
    pub const fn channels(self) -> usize {
        match self {
            ColorSpace::Rgb8 => 3,
            ColorSpace::Rgba8 => 4,
            ColorSpace::Bgr8 => 3,
            ColorSpace::Bgra8 => 4,
            ColorSpace::Gray8 => 1,
            ColorSpace::GrayAlpha8 => 2,
            ColorSpace::Ycbcr => 3,
            ColorSpace::Cmyk => 4,
            ColorSpace::Lab => 3,
        }
    }

    /// Get the bytes per pixel for this color space
    pub const fn bytes_per_pixel(self) -> usize {
        self.channels()
    }

    /// Check if this color space has alpha channel
    pub const fn has_alpha(self) -> bool {
        matches!(self, ColorSpace::Rgba8 | ColorSpace::Bgra8 | ColorSpace::GrayAlpha8)
    }
}

/// Decoded image frame
#[derive(Debug, Clone)]
pub struct ImageFrame {
    /// Pixel data
    pub data: alloc::vec::Vec<u8>,
    /// Frame width in pixels
    pub width: u32,
    /// Frame height in pixels
    pub height: u32,
    /// Color space of the pixel data
    pub color_space: ColorSpace,
    /// Row stride in bytes (0 = tightly packed)
    pub stride: u32,
    /// Presentation timestamp in microseconds (0 for still images)
    pub pts_us: i64,
    /// Duration in microseconds for animated frames
    pub duration_us: u64,
    /// Whether this is a key frame
    pub keyframe: bool,
}

impl ImageFrame {
    /// Create an empty image frame
    pub fn new(width: u32, height: u32, color_space: ColorSpace) -> Self {
        ImageFrame {
            data: alloc::vec::Vec::new(),
            width,
            height,
            color_space,
            stride: 0,
            pts_us: 0,
            duration_us: 0,
            keyframe: true,
        }
    }

    /// Create an image frame from pixel data
    pub fn from_data(
        data: alloc::vec::Vec<u8>,
        width: u32,
        height: u32,
        color_space: ColorSpace,
    ) -> Self {
        let stride = width * color_space.bytes_per_pixel() as u32;
        ImageFrame {
            data,
            width,
            height,
            color_space,
            stride,
            pts_us: 0,
            duration_us: 0,
            keyframe: true,
        }
    }

    /// Get the effective stride (tightly packed if stride is 0)
    pub fn effective_stride(&self) -> u32 {
        if self.stride > 0 {
            self.stride
        } else {
            self.width * self.color_space.bytes_per_pixel() as u32
        }
    }

    /// Calculate expected data size
    pub fn expected_size(&self) -> usize {
        (self.effective_stride() as usize) * (self.height as usize)
    }
}

/// Image decode configuration
#[derive(Debug, Clone, Copy)]
pub struct DecodeConfig {
    /// Input image format
    pub format: ImageFormat,
    /// Desired output color space
    pub output_color_space: ColorSpace,
    /// Maximum output width (0 = no limit)
    pub max_width: u32,
    /// Maximum output height (0 = no limit)
    pub max_height: u32,
    /// Whether to use hardware acceleration if available
    pub hw_accel: bool,
    /// Whether to decode progressively
    pub progressive: bool,
}

impl DecodeConfig {
    /// Create a default decode config for the given format
    pub const fn new(format: ImageFormat) -> Self {
        DecodeConfig {
            format,
            output_color_space: ColorSpace::Rgba8,
            max_width: 0,
            max_height: 0,
            hw_accel: true,
            progressive: false,
        }
    }
}

/// Image encode configuration
#[derive(Debug, Clone, Copy)]
pub struct EncodeConfig {
    /// Output image format
    pub format: ImageFormat,
    /// Input color space
    pub input_color_space: ColorSpace,
    /// Input width
    pub width: u32,
    /// Input height: u32,
    pub height: u32,
    /// Quality factor (0-100, format-specific)
    pub quality: u8,
    /// Whether to use hardware acceleration if available
    pub hw_accel: bool,
    /// Whether to encode progressively (JPEG/PNG)
    pub progressive: bool,
}

impl EncodeConfig {
    /// Create a default encode config for the given format
    pub const fn new(format: ImageFormat, width: u32, height: u32) -> Self {
        EncodeConfig {
            format,
            input_color_space: ColorSpace::Rgba8,
            width,
            height,
            quality: 80,
            hw_accel: true,
            progressive: false,
        }
    }
}

/// Progressive decode session state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgressiveState {
    /// Session not started
    Idle = 0,
    /// Receiving scan data
    Scanning = 1,
    /// More data needed
    NeedMoreData = 2,
    /// Decode complete
    Complete = 3,
    /// Decode failed
    Failed = 4,
}

/// Progressive decode session handle
#[derive(Debug, Clone)]
pub struct ProgressiveSession {
    /// Session identifier
    pub id: u64,
    /// Image format being decoded
    pub format: ImageFormat,
    /// Current state
    pub state: ProgressiveState,
    /// Current scan pass (0-based)
    pub current_pass: u32,
    /// Total passes expected
    pub total_passes: u32,
    /// Current approximation quality (0-100)
    pub quality: u8,
    /// Bytes consumed so far
    pub bytes_consumed: usize,
}

impl ProgressiveSession {
    /// Create a new progressive session
    pub fn new(id: u64, format: ImageFormat, total_passes: u32) -> Self {
        ProgressiveSession {
            id,
            format,
            state: ProgressiveState::Idle,
            current_pass: 0,
            total_passes,
            quality: 0,
            bytes_consumed: 0,
        }
    }

    /// Check if the session is complete
    pub fn is_complete(&self) -> bool {
        self.state == ProgressiveState::Complete
    }

    /// Check if more data is needed
    pub fn needs_more_data(&self) -> bool {
        self.state == ProgressiveState::NeedMoreData || self.state == ProgressiveState::Scanning
    }
}

/// Decoder instance identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DecoderId(pub u64);

/// Encoder instance identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EncoderId(pub u64);
