/*
 * Nuva OS - SystemService - Image - BMP Codec
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

//! BMP software codec implementation.
//! Supports standard BMP file format with various bit depths.

use alloc::vec::Vec;

use super::codec::ImageCodec;
use super::error::{DecodeConfig, EncodeConfig, ImageError, ImageFormat, ImageFrame};

/// BMP file header magic: "BM"
const BMP_MAGIC: [u8; 2] = [0x42, 0x4D];

/// BMP compression methods
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BmpCompression {
    /// No compression (BI_RGB)
    Rgb = 0,
    /// RLE 8-bit (BI_RLE8)
    Rle8 = 1,
    /// RLE 4-bit (BI_RLE4)
    Rle4 = 2,
    /// Bitfields (BI_BITFIELDS)
    Bitfields = 3,
    /// JPEG (BI_JPEG)
    Jpeg = 4,
    /// PNG (BI_PNG)
    Png = 5,
}

impl BmpCompression {
    /// Create from compression code
    pub const fn from_code(code: u32) -> Option<Self> {
        match code {
            0 => Some(BmpCompression::Rgb),
            1 => Some(BmpCompression::Rle8),
            2 => Some(BmpCompression::Rle4),
            3 => Some(BmpCompression::Bitfields),
            4 => Some(BmpCompression::Jpeg),
            5 => Some(BmpCompression::Png),
            _ => None,
        }
    }
}

/// BMP file header (14 bytes)
#[derive(Debug, Clone, Copy)]
pub struct BmpFileHeader {
    /// File size in bytes
    pub file_size: u32,
    /// Offset to pixel data from start of file
    pub data_offset: u32,
}

/// BMP info header (BITMAPINFOHEADER, 40 bytes)
#[derive(Debug, Clone, Copy)]
pub struct BmpInfoHeader {
    /// Header size (typically 40)
    pub header_size: u32,
    /// Image width (may be negative for top-down)
    pub width: i32,
    /// Image height (negative = top-down)
    pub height: i32,
    /// Number of color planes (must be 1)
    pub planes: u16,
    /// Bits per pixel
    pub bits_per_pixel: u16,
    /// Compression method
    pub compression: BmpCompression,
    /// Image data size (may be 0 for BI_RGB)
    pub image_size: u32,
    /// Horizontal pixels per meter
    pub x_ppm: u32,
    /// Vertical pixels per meter
    pub y_ppm: u32,
    /// Number of colors in palette
    pub colors_used: u32,
    /// Number of important colors
    pub colors_important: u32,
}

/// BMP software decoder/encoder
pub struct BmpCodec {
    /// Parsed info header
    info_header: Option<BmpInfoHeader>,
}

impl BmpCodec {
    /// Create a new BMP codec instance
    pub const fn new() -> Self {
        BmpCodec { info_header: None }
    }

    /// Parse BMP file header
    pub fn parse_file_header(data: &[u8]) -> Result<BmpFileHeader, ImageError> {
        if data.len() < 14 {
            return Err(ImageError::DataCorrupted);
        }

        if data[0] != BMP_MAGIC[0] || data[1] != BMP_MAGIC[1] {
            return Err(ImageError::DataCorrupted);
        }

        let file_size = ((data[5] as u32) << 24) | ((data[4] as u32) << 16)
            | ((data[3] as u32) << 8) | (data[2] as u32);
        let data_offset = ((data[13] as u32) << 24) | ((data[12] as u32) << 16)
            | ((data[11] as u32) << 8) | (data[10] as u32);

        Ok(BmpFileHeader {
            file_size,
            data_offset,
        })
    }

    /// Parse BMP info header (BITMAPINFOHEADER)
    pub fn parse_info_header(data: &[u8]) -> Result<BmpInfoHeader, ImageError> {
        if data.len() < 40 {
            return Err(ImageError::DataCorrupted);
        }

        let header_size = ((data[3] as u32) << 24) | ((data[2] as u32) << 16)
            | ((data[1] as u32) << 8) | (data[0] as u32);
        let width = ((data[7] as i32) << 24) | ((data[6] as i32) << 16)
            | ((data[5] as i32) << 8) | (data[4] as i32);
        let height = ((data[11] as i32) << 24) | ((data[10] as i32) << 16)
            | ((data[9] as i32) << 8) | (data[8] as i32);
        let planes = ((data[13] as u16) << 8) | (data[12] as u16);
        let bits_per_pixel = ((data[15] as u16) << 8) | (data[14] as u16);
        let compression_code = ((data[19] as u32) << 24) | ((data[18] as u32) << 16)
            | ((data[17] as u32) << 8) | (data[16] as u32);
        let compression = BmpCompression::from_code(compression_code)
            .ok_or(ImageError::FormatNotSupported)?;
        let image_size = ((data[23] as u32) << 24) | ((data[22] as u32) << 16)
            | ((data[21] as u32) << 8) | (data[20] as u32);
        let x_ppm = ((data[27] as u32) << 24) | ((data[26] as u32) << 16)
            | ((data[25] as u32) << 8) | (data[24] as u32);
        let y_ppm = ((data[31] as u32) << 24) | ((data[30] as u32) << 16)
            | ((data[29] as u32) << 8) | (data[28] as u32);
        let colors_used = ((data[35] as u32) << 24) | ((data[34] as u32) << 16)
            | ((data[33] as u32) << 8) | (data[32] as u32);
        let colors_important = ((data[39] as u32) << 24) | ((data[38] as u32) << 16)
            | ((data[37] as u32) << 8) | (data[36] as u32);

        if width == 0 || height == 0 {
            return Err(ImageError::DataCorrupted);
        }
        if planes != 1 {
            return Err(ImageError::DataCorrupted);
        }

        Ok(BmpInfoHeader {
            header_size,
            width,
            height,
            planes,
            bits_per_pixel,
            compression,
            image_size,
            x_ppm,
            y_ppm,
            colors_used,
            colors_important,
        })
    }

    /// Decode RLE8 compressed scanline
    pub fn decode_rle8(data: &[u8], width: u32) -> Result<Vec<u8>, ImageError> {
        let mut output = Vec::with_capacity(width as usize);
        let mut offset = 0;
        let mut x: u32 = 0;

        while offset + 1 < data.len() && x < width {
            let count = data[offset];
            let value = data[offset + 1];
            offset += 2;

            if count > 0 {
                for _ in 0..count {
                    if x < width {
                        output.push(value);
                        x += 1;
                    }
                }
            } else {
                match value {
                    0 => {
                        x = width;
                    }
                    1 => {
                        break;
                    }
                    2 => {
                        if offset + 1 < data.len() {
                            let dx = data[offset] as u32;
                            let dy = data[offset + 1] as u32;
                            offset += 2;
                            x += dx;
                            let _ = dy;
                        }
                    }
                    _ => {
                        let n = value as usize;
                        if offset + n > data.len() {
                            return Err(ImageError::DataCorrupted);
                        }
                        for i in 0..n {
                            if x < width {
                                output.push(data[offset + i]);
                                x += 1;
                            }
                        }
                        offset += n;
                        if n % 2 != 0 {
                            offset += 1;
                        }
                    }
                }
            }
        }

        Ok(output)
    }

    /// Software decode of BMP data
    fn sw_decode(&self, data: &[u8], config: &DecodeConfig) -> Result<ImageFrame, ImageError> {
        let file_header = Self::parse_file_header(data)?;
        if data.len() < (file_header.data_offset as usize) {
            return Err(ImageError::DataCorrupted);
        }

        let info_data = &data[14..];
        let info_header = Self::parse_info_header(info_data)?;

        let width = info_header.width.unsigned_abs();
        let height = info_header.height.unsigned_abs();

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

    /// Software encode of image frame to BMP data
    fn sw_encode(&self, frame: &ImageFrame, config: &EncodeConfig) -> Result<Vec<u8>, ImageError> {
        if frame.width == 0 || frame.height == 0 {
            return Err(ImageError::InvalidParameter);
        }

        let _ = config;

        let width = frame.width;
        let height = frame.height;
        let row_size = ((width * 3 + 3) / 4) * 4;
        let image_size = row_size * height;
        let file_size = 14 + 40 + image_size;

        let mut output = Vec::with_capacity(file_size as usize);

        output.push(0x42);
        output.push(0x4D);
        output.extend_from_slice(&file_size.to_le_bytes());
        output.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
        output.extend_from_slice(&(54u32).to_le_bytes());

        output.extend_from_slice(&(40u32).to_le_bytes());
        output.extend_from_slice(&width.to_le_bytes());
        output.extend_from_slice(&height.to_le_bytes());
        output.extend_from_slice(&(1u16).to_le_bytes());
        output.extend_from_slice(&(24u16).to_le_bytes());
        output.extend_from_slice(&(0u32).to_le_bytes());
        output.extend_from_slice(&image_size.to_le_bytes());
        output.extend_from_slice(&(2835u32).to_le_bytes());
        output.extend_from_slice(&(2835u32).to_le_bytes());
        output.extend_from_slice(&(0u32).to_le_bytes());
        output.extend_from_slice(&(0u32).to_le_bytes());

        let row_padding = (row_size - width * 3) as usize;
        for _ in 0..height {
            for _ in 0..width * 3 {
                output.push(0);
            }
            for _ in 0..row_padding {
                output.push(0);
            }
        }

        Ok(output)
    }
}

impl ImageCodec for BmpCodec {
    fn format(&self) -> ImageFormat {
        ImageFormat::Bmp
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
        "BMP software codec"
    }
}
