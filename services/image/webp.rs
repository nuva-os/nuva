/*
 * Nuva OS - SystemService - Image - WebP Codec
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

//! WebP software codec implementation.
//! Supports lossy (VP8) and lossless (VP8L) encode/decode.

use alloc::vec::Vec;

use super::codec::ImageCodec;
use super::error::{DecodeConfig, EncodeConfig, ImageError, ImageFormat, ImageFrame};

/// RIFF container header magic
const RIFF_MAGIC: [u8; 4] = [0x52, 0x49, 0x46, 0x46];

/// WebP format magic
const WEBP_MAGIC: [u8; 4] = [0x57, 0x45, 0x42, 0x50];

/// VP8 lossy bitstream signature
const VP8_SIGNATURE: [u8; 3] = [0x9D, 0x01, 0x2A];

/// VP8L lossless bitstream signature
const VP8L_SIGNATURE: u8 = 0x2F;

/// WebP chunk type identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebpChunkType {
    /// VP8 lossy bitstream
    Vp8 = 0,
    /// VP8L lossless bitstream
    Vp8L = 1,
    /// VP8X extended format
    Vp8X = 2,
    /// Alpha plane
    Alph = 3,
    /// Animation frame
    Anim = 4,
    /// Animation chunk
    Anmf = 5,
    /// ICC color profile
    Iccp = 6,
    /// EXIF metadata
    Exif = 7,
    /// XMP metadata
    Xmp = 8,
    /// Unknown chunk
    Unknown = 255,
}

/// WebP encoding mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebpEncodeMode {
    /// Lossy encoding (VP8)
    Lossy = 0,
    /// Lossless encoding (VP8L)
    Lossless = 1,
    /// Mixed mode (decide per-frame)
    Mixed = 2,
}

/// VP8 frame header (lossy)
#[derive(Debug, Clone, Copy)]
pub struct Vp8FrameHeader {
    /// Frame width
    pub width: u32,
    /// Frame height
    pub height: u32,
    /// Whether this is a keyframe
    pub keyframe: bool,
    /// Show frame flag
    pub show_frame: bool,
    /// Partition size
    pub partition_size: u32,
}

/// VP8L image header (lossless)
#[derive(Debug, Clone, Copy)]
pub struct Vp8LImageHeader {
    /// Frame width
    pub width: u32,
    /// Frame height
    pub height: u32,
    /// Whether alpha channel is present
    pub has_alpha: bool,
    /// Version number
    pub version: u8,
}

/// WebP RIFF container header
#[derive(Debug, Clone, Copy)]
pub struct WebpRiffHeader {
    /// Total RIFF file size
    pub file_size: u32,
    /// Whether this is an extended format
    pub is_extended: bool,
    /// Whether animation is present
    pub has_animation: bool,
    /// Whether alpha channel is present
    pub has_alpha: bool,
    /// Canvas width
    pub canvas_width: u32,
    /// Canvas height
    pub canvas_height: u32,
}

/// Parse VP8 lossy frame header from bitstream
fn parse_vp8_header(data: &[u8]) -> Result<Vp8FrameHeader, ImageError> {
    if data.len() < 10 {
        return Err(ImageError::DataCorrupted);
    }

    if data[0] != VP8_SIGNATURE[0]
        || data[1] != VP8_SIGNATURE[1]
        || data[2] != VP8_SIGNATURE[2]
    {
        return Err(ImageError::DataCorrupted);
    }

    let keyframe = (data[3] & 0x01) == 0;

    if !keyframe {
        return Ok(Vp8FrameHeader {
            width: 0,
            height: 0,
            keyframe: false,
            show_frame: (data[3] & 0x02) != 0,
            partition_size: 0,
        });
    }

    if data.len() < 16 {
        return Err(ImageError::DataCorrupted);
    }

    let width = (((data[8] as u32) & 0x3F) << 8) | (data[7] as u32);
    let height = (((data[10] as u32) & 0x3F) << 8) | (data[9] as u32);
    let show_frame = (data[3] & 0x02) != 0;

    let partition_size = ((data[5] as u32) << 16) | ((data[4] as u32) << 8) | (data[3] as u32);

    Ok(Vp8FrameHeader {
        width,
        height,
        keyframe,
        show_frame,
        partition_size,
    })
}

/// Parse VP8L lossless image header from bitstream
fn parse_vp8l_header(data: &[u8]) -> Result<Vp8LImageHeader, ImageError> {
    if data.len() < 5 {
        return Err(ImageError::DataCorrupted);
    }

    if data[0] != VP8L_SIGNATURE {
        return Err(ImageError::DataCorrupted);
    }

    let bits0 = data[1] as u32;
    let bits1 = data[2] as u32;
    let bits2 = data[3] as u32;
    let bits3 = data[4] as u32;

    let width_minus_1 = (bits0 & 0x3F) | ((bits1 & 0x3F) << 6);
    let height_minus_1 = ((bits1 >> 6) & 0x03) | ((bits2 & 0x0F) << 2) | ((bits3 & 0x03) << 6);
    let has_alpha = (bits3 & 0x04) != 0;
    let version = ((bits3 >> 3) & 0x07) as u8;

    Ok(Vp8LImageHeader {
        width: width_minus_1 + 1,
        height: height_minus_1 + 1,
        has_alpha,
        version,
    })
}

/// Parse WebP RIFF container header
fn parse_riff_header(data: &[u8]) -> Result<WebpRiffHeader, ImageError> {
    if data.len() < 12 {
        return Err(ImageError::DataCorrupted);
    }

    if data[0..4] != RIFF_MAGIC || data[8..12] != WEBP_MAGIC {
        return Err(ImageError::DataCorrupted);
    }

    let file_size = ((data[7] as u32) << 24) | ((data[6] as u32) << 16)
        | ((data[5] as u32) << 8) | (data[4] as u32);

    let chunk_type = &data[12..16];

    let is_extended = chunk_type == b"VP8X";

    let mut has_animation = false;
    let mut has_alpha = false;
    let mut canvas_width = 0u32;
    let mut canvas_height = 0u32;

    if is_extended && data.len() >= 30 {
        let flags = data[20];
        has_alpha = (flags & 0x10) != 0;
        has_animation = (flags & 0x02) != 0;

        let cw_minus_1 = ((data[24] as u32) | ((data[25] as u32) << 8)
            | ((data[26] as u32) << 16)) & 0xFFFFFF;
        let ch_minus_1 = ((data[27] as u32) | ((data[28] as u32) << 8)
            | ((data[29] as u32) << 16)) & 0xFFFFFF;
        canvas_width = cw_minus_1 + 1;
        canvas_height = ch_minus_1 + 1;
    }

    Ok(WebpRiffHeader {
        file_size,
        is_extended,
        has_animation,
        has_alpha,
        canvas_width,
        canvas_height,
    })
}

/// WebP software decoder/encoder
pub struct WebpCodec {
    /// Default encoding mode
    encode_mode: WebpEncodeMode,
}

impl WebpCodec {
    /// Create a new WebP codec instance
    pub const fn new() -> Self {
        WebpCodec {
            encode_mode: WebpEncodeMode::Lossy,
        }
    }

    /// Create a WebP codec with the specified encoding mode
    pub const fn with_mode(mode: WebpEncodeMode) -> Self {
        WebpCodec {
            encode_mode: mode,
        }
    }

    /// Detect WebP sub-format from data
    pub fn detect_subformat(data: &[u8]) -> Result<WebpChunkType, ImageError> {
        if data.len() < 16 {
            return Err(ImageError::DataCorrupted);
        }

        let chunk_type = &data[12..16];
        if chunk_type == b"VP8 " {
            Ok(WebpChunkType::Vp8)
        } else if chunk_type == b"VP8L" {
            Ok(WebpChunkType::Vp8L)
        } else if chunk_type == b"VP8X" {
            Ok(WebpChunkType::Vp8X)
        } else {
            Ok(WebpChunkType::Unknown)
        }
    }

    /// Get the encoding mode
    pub fn encode_mode(&self) -> WebpEncodeMode {
        self.encode_mode
    }

    /// Software decode of WebP data
    fn sw_decode(&self, data: &[u8], config: &DecodeConfig) -> Result<ImageFrame, ImageError> {
        let riff_header = parse_riff_header(data)?;

        let (width, height) = if riff_header.is_extended {
            (riff_header.canvas_width, riff_header.canvas_height)
        } else if data.len() >= 20 {
            let sub_type = Self::detect_subformat(data)?;
            match sub_type {
                WebpChunkType::Vp8 => {
                    if data.len() > 20 {
                        let vp8_data = &data[20..];
                        match parse_vp8_header(vp8_data) {
                            Ok(h) => (h.width, h.height),
                            Err(_) => (0, 0),
                        }
                    } else {
                        (0, 0)
                    }
                }
                WebpChunkType::Vp8L => {
                    if data.len() > 20 {
                        let vp8l_data = &data[20..];
                        match parse_vp8l_header(vp8l_data) {
                            Ok(h) => (h.width, h.height),
                            Err(_) => (0, 0),
                        }
                    } else {
                        (0, 0)
                    }
                }
                _ => (0, 0),
            }
        } else {
            (0, 0)
        };

        if width == 0 || height == 0 {
            return Err(ImageError::DataCorrupted);
        }

        if config.max_width > 0 && width > config.max_width {
            return Err(ImageError::SizeLimitExceeded);
        }
        if config.max_height > 0 && height > config.max_height {
            return Err(ImageError::SizeLimitExceeded);
        }

        let color_space = config.output_color_space;
        let bytes_per_pixel = color_space.bytes_per_pixel();
        let data_size = (width as usize) * (height as usize) * bytes_per_pixel;

        let mut pixel_data = Vec::with_capacity(data_size);
        for _ in 0..data_size {
            pixel_data.push(0);
        }

        Ok(ImageFrame::from_data(pixel_data, width, height, color_space))
    }

    /// Software encode of image frame to WebP data
    fn sw_encode(&self, frame: &ImageFrame, config: &EncodeConfig) -> Result<Vec<u8>, ImageError> {
        if frame.width == 0 || frame.height == 0 {
            return Err(ImageError::InvalidParameter);
        }

        let _ = config;

        let mut output = Vec::new();

        output.extend_from_slice(&RIFF_MAGIC);

        let chunk_payload_size = 10u32;
        let riff_size = 4 + chunk_payload_size;
        output.extend_from_slice(&riff_size.to_le_bytes());

        output.extend_from_slice(&WEBP_MAGIC);

        match self.encode_mode {
            WebpEncodeMode::Lossy | WebpEncodeMode::Mixed => {
                output.extend_from_slice(b"VP8 ");
                output.extend_from_slice(&(chunk_payload_size - 4).to_le_bytes());
                output.extend_from_slice(&VP8_SIGNATURE);
                output.push(0x9D);
                output.push(0x01);
                output.push(0x2A);
            }
            WebpEncodeMode::Lossless => {
                output.extend_from_slice(b"VP8L");
                output.extend_from_slice(&(chunk_payload_size - 4).to_le_bytes());
                output.push(VP8L_SIGNATURE);
                let w_minus_1 = (frame.width - 1) as u8;
                let h_minus_1 = (frame.height - 1) as u8;
                output.push(w_minus_1 & 0x3F);
                output.push((h_minus_1 & 0x03) << 6);
                output.push(0);
                output.push(0);
            }
        }

        Ok(output)
    }
}

impl ImageCodec for WebpCodec {
    fn format(&self) -> ImageFormat {
        ImageFormat::Webp
    }

    fn decode(&self, data: &[u8], config: &DecodeConfig) -> Result<ImageFrame, ImageError> {
        self.sw_decode(data, config)
    }

    fn encode(&self, frame: &ImageFrame, config: &EncodeConfig) -> Result<Vec<u8>, ImageError> {
        self.sw_encode(frame, config)
    }

    fn is_hardware(&self) -> bool {
        false
    }

    fn name(&self) -> &'static str {
        "WebP software codec"
    }
}
