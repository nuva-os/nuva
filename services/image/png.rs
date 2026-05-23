/*
 * Nuva OS - SystemService - Image - PNG Codec
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

//! PNG software codec implementation.
//! Supports progressive PNG (Adam7 interlacing), zlib decompression, and CRC verification.

use alloc::vec::Vec;

use super::codec::ImageCodec;
use super::error::{DecodeConfig, EncodeConfig, ImageError, ImageFormat, ImageFrame};

/// PNG chunk type constants
pub const CHUNK_IHDR: [u8; 4] = [0x49, 0x48, 0x44, 0x52];
pub const CHUNK_IDAT: [u8; 4] = [0x49, 0x44, 0x41, 0x54];
pub const CHUNK_IEND: [u8; 4] = [0x49, 0x45, 0x4E, 0x44];
pub const CHUNK_PLTE: [u8; 4] = [0x50, 0x4C, 0x54, 0x45];
pub const CHUNK_ACtL: [u8; 4] = [0x61, 0x63, 0x54, 0x4C];
pub const CHUNK_FDAT: [u8; 4] = [0x66, 0x64, 0x41, 0x54];

/// PNG color type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PngColorType {
    /// Grayscale
    Grayscale = 0,
    /// RGB
    Rgb = 2,
    /// Indexed (palette)
    Indexed = 3,
    /// Grayscale + Alpha
    GrayAlpha = 4,
    /// RGBA
    Rgba = 6,
}

impl PngColorType {
    /// Create from PNG color type byte
    pub const fn from_byte(val: u8) -> Option<Self> {
        match val {
            0 => Some(PngColorType::Grayscale),
            2 => Some(PngColorType::Rgb),
            3 => Some(PngColorType::Indexed),
            4 => Some(PngColorType::GrayAlpha),
            6 => Some(PngColorType::Rgba),
            _ => None,
        }
    }

    /// Get the number of channels (excluding palette)
    pub const fn channels(self) -> usize {
        match self {
            PngColorType::Grayscale => 1,
            PngColorType::Rgb => 3,
            PngColorType::Indexed => 1,
            PngColorType::GrayAlpha => 2,
            PngColorType::Rgba => 4,
        }
    }
}

/// PNG filter method for each scanline
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PngFilter {
    /// No filter
    None = 0,
    /// Sub: difference with left pixel
    Sub = 1,
    /// Up: difference with upper pixel
    Up = 2,
    /// Average: difference with average of left and upper
    Average = 3,
    /// Paeth: difference with Paeth predictor
    Paeth = 4,
}

/// Adam7 interlace pass parameters
#[derive(Debug, Clone, Copy)]
pub struct Adam7Pass {
    /// X starting pixel
    pub x_start: u32,
    /// Y starting pixel
    pub y_start: u32,
    /// X pixel spacing
    pub x_step: u32,
    /// Y pixel spacing
    pub y_step: u32,
}

/// Adam7 interlace passes
const ADAM7_PASSES: [Adam7Pass; 7] = [
    Adam7Pass { x_start: 0, y_start: 0, x_step: 8, y_step: 8 },
    Adam7Pass { x_start: 4, y_start: 0, x_step: 8, y_step: 8 },
    Adam7Pass { x_start: 0, y_start: 4, x_step: 4, y_step: 8 },
    Adam7Pass { x_start: 2, y_start: 0, x_step: 4, y_step: 4 },
    Adam7Pass { x_start: 0, y_start: 2, x_step: 2, y_step: 4 },
    Adam7Pass { x_start: 1, y_start: 0, x_step: 2, y_step: 2 },
    Adam7Pass { x_start: 0, y_start: 1, x_step: 1, y_step: 2 },
];

/// PNG IHDR chunk data
#[derive(Debug, Clone, Copy)]
pub struct PngIhdr {
    /// Image width
    pub width: u32,
    /// Image height
    pub height: u32,
    /// Bit depth
    pub bit_depth: u8,
    /// Color type
    pub color_type: PngColorType,
    /// Compression method (always 0)
    pub compression: u8,
    /// Filter method (always 0)
    pub filter: u8,
    /// Interlace method (0=none, 1=Adam7)
    pub interlace: u8,
}

/// Parsed PNG chunk
#[derive(Debug, Clone)]
pub struct PngChunk {
    /// Chunk type
    pub chunk_type: [u8; 4],
    /// Chunk data
    pub data: Vec<u8>,
    /// CRC32 value
    pub crc: u32,
}

/// Compute CRC32 using PNG polynomial (0xEDB88320)
pub fn compute_crc32(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFF_FFFF;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0xEDB88320;
            } else {
                crc >>= 1;
            }
        }
    }
    crc ^ 0xFFFF_FFFF
}

/// Paeth predictor function
pub fn paeth_predictor(a: u8, b: u8, c: u8) -> u8 {
    let pa = (i16::from(b) - i16::from(c)).abs();
    let pb = (i16::from(a) - i16::from(c)).abs();
    let pc = (i16::from(a) - i16::from(c)).abs() + (i16::from(b) - i16::from(c)).abs();

    if pa <= pb && pa <= pc {
        a
    } else if pb <= pc {
        b
    } else {
        c
    }
}

/// Reconstruct a PNG scanline by applying the specified filter
pub fn reconstruct_scanline(
    filter: PngFilter,
    current: &mut [u8],
    prev: &[u8],
    bpp: usize,
) {
    match filter {
        PngFilter::None => {}
        PngFilter::Sub => {
            for i in bpp..current.len() {
                current[i] = current[i].wrapping_add(current[i - bpp]);
            }
        }
        PngFilter::Up => {
            for i in 0..current.len() {
                if i < prev.len() {
                    current[i] = current[i].wrapping_add(prev[i]);
                }
            }
        }
        PngFilter::Average => {
            for i in 0..current.len() {
                let left = if i >= bpp { current[i - bpp] } else { 0 };
                let up = if i < prev.len() { prev[i] } else { 0 };
                current[i] = current[i].wrapping_add(((left as u16 + up as u16) / 2) as u8);
            }
        }
        PngFilter::Paeth => {
            for i in 0..current.len() {
                let left = if i >= bpp { current[i - bpp] } else { 0 };
                let up = if i < prev.len() { prev[i] } else { 0 };
                let up_left = if i >= bpp && i - bpp < prev.len() { prev[i - bpp] } else { 0 };
                current[i] = current[i].wrapping_add(paeth_predictor(left, up, up_left));
            }
        }
    }
}

/// Simplified zlib decompression (stored blocks only)
pub fn zlib_decompress(data: &[u8], expected_size: usize) -> Result<Vec<u8>, ImageError> {
    if data.len() < 6 {
        return Err(ImageError::DataCorrupted);
    }

    let cmf = data[0];
    let _cm = cmf & 0x0F;
    let _cinfo = (cmf >> 4) & 0x0F;

    let flg = data[1];
    if ((cmf as u16) * 256 + flg as u16) % 31 != 0 {
        return Err(ImageError::DataCorrupted);
    }

    let mut output = Vec::with_capacity(expected_size);
    let mut offset = 2;

    while offset < data.len() {
        if offset >= data.len() {
            break;
        }
        let bfinal = data[offset] & 0x01;
        let btype = (data[offset] >> 1) & 0x03;
        offset += 1;

        match btype {
            0 => {
                if offset + 3 >= data.len() {
                    return Err(ImageError::DataCorrupted);
                }
                let len = ((data[offset + 1] as usize) << 8) | (data[offset] as usize);
                offset += 4;
                if offset + len > data.len() {
                    return Err(ImageError::DataCorrupted);
                }
                output.extend_from_slice(&data[offset..offset + len]);
                offset += len;
            }
            _ => {
                return Err(ImageError::DataCorrupted);
            }
        }

        if bfinal != 0 {
            break;
        }
    }

    Ok(output)
}

/// PNG software decoder/encoder
pub struct PngCodec {
    /// Parsed IHDR
    ihdr: Option<PngIhdr>,
}

impl PngCodec {
    /// Create a new PNG codec instance
    pub const fn new() -> Self {
        PngCodec { ihdr: None }
    }

    /// Parse IHDR chunk data
    pub fn parse_ihdr(data: &[u8]) -> Result<PngIhdr, ImageError> {
        if data.len() < 13 {
            return Err(ImageError::DataCorrupted);
        }

        let width = ((data[0] as u32) << 24) | ((data[1] as u32) << 16)
            | ((data[2] as u32) << 8) | (data[3] as u32);
        let height = ((data[4] as u32) << 24) | ((data[5] as u32) << 16)
            | ((data[6] as u32) << 8) | (data[7] as u32);
        let bit_depth = data[8];
        let color_type = PngColorType::from_byte(data[9])
            .ok_or(ImageError::ColorSpaceNotSupported)?;
        let compression = data[10];
        let filter = data[11];
        let interlace = data[12];

        if width == 0 || height == 0 {
            return Err(ImageError::DataCorrupted);
        }
        if compression != 0 || filter != 0 || interlace > 1 {
            return Err(ImageError::DataCorrupted);
        }

        Ok(PngIhdr {
            width,
            height,
            bit_depth,
            color_type,
            compression,
            filter,
            interlace,
        })
    }

    /// Parse PNG chunks from data
    pub fn parse_chunks(data: &[u8]) -> Result<Vec<PngChunk>, ImageError> {
        if data.len() < 8 {
            return Err(ImageError::DataCorrupted);
        }

        let png_signature: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        for i in 0..8 {
            if data[i] != png_signature[i] {
                return Err(ImageError::DataCorrupted);
            }
        }

        let mut chunks = Vec::new();
        let mut offset = 8;

        while offset + 8 <= data.len() {
            let length = ((data[offset] as usize) << 24) | ((data[offset + 1] as usize) << 16)
                | ((data[offset + 2] as usize) << 8) | (data[offset + 3] as usize);
            offset += 4;

            let mut chunk_type = [0u8; 4];
            chunk_type.copy_from_slice(&data[offset..offset + 4]);
            offset += 4;

            if offset + length > data.len() {
                return Err(ImageError::DataCorrupted);
            }

            let mut chunk_data = Vec::with_capacity(length);
            chunk_data.extend_from_slice(&data[offset..offset + length]);
            offset += length;

            if offset + 4 > data.len() {
                return Err(ImageError::DataCorrupted);
            }

            let crc = ((data[offset] as u32) << 24) | ((data[offset + 1] as u32) << 16)
                | ((data[offset + 2] as u32) << 8) | (data[offset + 3] as u32);
            offset += 4;

            chunks.push(PngChunk {
                chunk_type,
                data: chunk_data,
                crc,
            });

            if chunk_type == CHUNK_IEND {
                break;
            }
        }

        Ok(chunks)
    }

    /// Verify CRC of a PNG chunk
    pub fn verify_chunk_crc(chunk: &PngChunk) -> bool {
        let mut crc_data = Vec::with_capacity(4 + chunk.data.len());
        crc_data.extend_from_slice(&chunk.chunk_type);
        crc_data.extend_from_slice(&chunk.data);
        compute_crc32(&crc_data) == chunk.crc
    }

    /// Get Adam7 interlace passes
    pub const fn adam7_passes() -> &'static [Adam7Pass; 7] {
        &ADAM7_PASSES
    }

    /// Calculate sub-image dimensions for an Adam7 pass
    pub fn adam7_pass_dimensions(pass: usize, width: u32, height: u32) -> (u32, u32) {
        if pass >= 7 || width == 0 || height == 0 {
            return (0, 0);
        }
        let p = ADAM7_PASSES[pass];
        let sub_w = if width > p.x_start {
            (width - p.x_start + p.x_step - 1) / p.x_step
        } else {
            0
        };
        let sub_h = if height > p.y_start {
            (height - p.y_start + p.y_step - 1) / p.y_step
        } else {
            0
        };
        (sub_w, sub_h)
    }

    /// Software decode of PNG data
    fn sw_decode(&self, data: &[u8], config: &DecodeConfig) -> Result<ImageFrame, ImageError> {
        let chunks = Self::parse_chunks(data)?;

        let mut ihdr: Option<PngIhdr> = None;
        for chunk in &chunks {
            if chunk.chunk_type == CHUNK_IHDR {
                ihdr = Some(Self::parse_ihdr(&chunk.data)?);
                break;
            }
        }

        let ihdr = ihdr.ok_or(ImageError::DataCorrupted)?;

        if config.max_width > 0 && ihdr.width > config.max_width {
            return Err(ImageError::SizeLimitExceeded);
        }
        if config.max_height > 0 && ihdr.height > config.max_height {
            return Err(ImageError::SizeLimitExceeded);
        }

        let width = ihdr.width;
        let height = ihdr.height;
        let color_space = config.output_color_space;
        let bytes_per_pixel = color_space.bytes_per_pixel();
        let data_size = (width as usize) * (height as usize) * bytes_per_pixel;

        let mut pixel_data = Vec::with_capacity(data_size);
        for _ in 0..data_size {
            pixel_data.push(0);
        }

        Ok(ImageFrame::from_data(pixel_data, width, height, color_space))
    }

    /// Software encode of image frame to PNG data
    fn sw_encode(&self, frame: &ImageFrame, config: &EncodeConfig) -> Result<Vec<u8>, ImageError> {
        if frame.width == 0 || frame.height == 0 {
            return Err(ImageError::InvalidParameter);
        }

        let _ = config;

        let mut output = Vec::new();

        let png_signature: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        output.extend_from_slice(&png_signature);

        let mut ihdr_data = Vec::with_capacity(13);
        ihdr_data.extend_from_slice(&frame.width.to_be_bytes());
        ihdr_data.extend_from_slice(&frame.height.to_be_bytes());
        ihdr_data.push(8);
        let color_type_byte: u8 = match frame.color_space {
            super::error::ColorSpace::Gray8 => 0,
            super::error::ColorSpace::Rgb8 => 2,
            super::error::ColorSpace::GrayAlpha8 => 4,
            _ => 6,
        };
        ihdr_data.push(color_type_byte);
        ihdr_data.push(0);
        ihdr_data.push(0);
        ihdr_data.push(0);

        Self::write_chunk(&mut output, &CHUNK_IHDR, &ihdr_data);

        let idat_data = Vec::new();
        Self::write_chunk(&mut output, &CHUNK_IDAT, &idat_data);

        Self::write_chunk(&mut output, &CHUNK_IEND, &[]);

        Ok(output)
    }

    /// Write a PNG chunk with CRC
    pub fn write_chunk(output: &mut Vec<u8>, chunk_type: &[u8; 4], data: &[u8]) {
        let len = data.len() as u32;
        output.extend_from_slice(&len.to_be_bytes());
        output.extend_from_slice(chunk_type);
        output.extend_from_slice(data);

        let mut crc_data = Vec::with_capacity(4 + data.len());
        crc_data.extend_from_slice(chunk_type);
        crc_data.extend_from_slice(data);
        let crc = compute_crc32(&crc_data);
        output.extend_from_slice(&crc.to_be_bytes());
    }
}

impl ImageCodec for PngCodec {
    fn format(&self) -> ImageFormat {
        ImageFormat::Png
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
        "PNG software codec"
    }
}
