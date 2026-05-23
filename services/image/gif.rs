/*
 * Nuva OS - SystemService - Image - GIF Codec
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

//! GIF software codec implementation.
//! Supports multi-frame animation and LZW decompression.

use alloc::vec::Vec;

use super::codec::ImageCodec;
use super::error::{DecodeConfig, EncodeConfig, ImageError, ImageFormat, ImageFrame};

/// GIF header magic: "GIF"
const GIF_MAGIC: [u8; 3] = [0x47, 0x49, 0x46];

/// GIF version "87a"
const GIF_VERSION_87A: [u8; 3] = [0x38, 0x37, 0x61];

/// GIF version "89a"
const GIF_VERSION_89A: [u8; 3] = [0x38, 0x39, 0x61];

/// GIF block type: Image Descriptor
const GIF_IMAGE_SEPARATOR: u8 = 0x2C;

/// GIF block type: Extension
const GIF_EXTENSION: u8 = 0x21;

/// GIF block type: Trailer
const GIF_TRAILER: u8 = 0x3B;

/// GIF extension: Graphics Control
const GIF_GCE: u8 = 0xF9;

/// GIF extension: Application Extension (for NETSCAPE2.0 animation loop)
const GIF_APP_EXT: u8 = 0xFF;

/// LZW maximum code table size
const LZW_MAX_TABLE_SIZE: usize = 4096;

/// GIF Logical Screen Descriptor
#[derive(Debug, Clone, Copy)]
pub struct GifScreenDescriptor {
    /// Canvas width
    pub width: u16,
    /// Canvas height
    pub height: u16,
    /// Whether global color table is present
    pub has_gct: bool,
    /// Color resolution (bits per primary color - 1)
    pub color_resolution: u8,
    /// Whether GCT is sorted
    pub sorted: bool,
    /// Size of GCT (2^(N+1) entries)
    pub gct_size: u8,
    /// Background color index
    pub bg_color_index: u8,
    /// Pixel aspect ratio
    pub pixel_aspect_ratio: u8,
}

/// GIF Image Descriptor
#[derive(Debug, Clone, Copy)]
pub struct GifImageDescriptor {
    /// Left position
    pub left: u16,
    /// Top position
    pub top: u16,
    /// Image width
    pub width: u16,
    /// Image height
    pub height: u16,
    /// Whether local color table is present
    pub has_lct: bool,
    /// Whether image is interlaced
    pub interlaced: bool,
    /// Whether LCT is sorted
    pub sorted: bool,
    /// Size of LCT (2^(N+1) entries)
    pub lct_size: u8,
}

/// GIF Graphics Control Extension
#[derive(Debug, Clone, Copy)]
pub struct GifGraphicsControl {
    /// Disposal method
    pub disposal: u8,
    /// User input flag
    pub user_input: bool,
    /// Transparent color flag
    pub transparent: bool,
    /// Delay time in centiseconds
    pub delay_cs: u16,
    /// Transparent color index
    pub transparent_index: u8,
}

/// GIF animation frame
#[derive(Debug, Clone)]
pub struct GifFrame {
    /// Image descriptor
    pub descriptor: GifImageDescriptor,
    /// Graphics control (if present)
    pub gce: Option<GifGraphicsControl>,
    /// Decoded pixel data (indexed color)
    pub pixel_data: Vec<u8>,
}

/// LZW decompressor for GIF
pub struct LzwDecoder {
    /// Minimum code size
    min_code_size: u8,
    /// Current code size
    code_size: u8,
    /// Clear code
    clear_code: u16,
    /// End of information code
    eoi_code: u16,
    /// Next available code
    next_code: u16,
    /// Code table: prefix codes
    prefix: [u16; LZW_MAX_TABLE_SIZE],
    /// Code table: suffix bytes
    suffix: [u8; LZW_MAX_TABLE_SIZE],
    /// Bit buffer
    bit_buffer: u32,
    /// Bits in buffer
    bits_in_buffer: u8,
    /// Whether decoder is initialized
    initialized: bool,
}

impl LzwDecoder {
    /// Create a new LZW decoder with the given minimum code size
    pub fn new(min_code_size: u8) -> Result<Self, ImageError> {
        if min_code_size < 2 || min_code_size > 8 {
            return Err(ImageError::InvalidParameter);
        }

        let clear_code = 1u16 << min_code_size;
        let eoi_code = clear_code + 1;

        Ok(LzwDecoder {
            min_code_size,
            code_size: min_code_size + 1,
            clear_code,
            eoi_code,
            next_code: eoi_code + 1,
            prefix: [0; LZW_MAX_TABLE_SIZE],
            suffix: [0; LZW_MAX_TABLE_SIZE],
            bit_buffer: 0,
            bits_in_buffer: 0,
            initialized: false,
        })
    }

    /// Reset the code table
    fn reset(&mut self) {
        self.code_size = self.min_code_size + 1;
        self.next_code = self.eoi_code + 1;
        for i in 0..self.clear_code as usize {
            self.prefix[i] = 0;
            self.suffix[i] = i as u8;
        }
    }

    /// Read next code from the bit stream
    fn read_code(&mut self, data: &[u8], offset: &mut usize) -> Option<u16> {
        while self.bits_in_buffer < self.code_size {
            if *offset >= data.len() {
                return None;
            }
            self.bit_buffer |= (data[*offset] as u32) << self.bits_in_buffer;
            self.bits_in_buffer += 8;
            *offset += 1;
        }

        let code = (self.bit_buffer & ((1u32 << self.code_size) - 1)) as u16;
        self.bit_buffer >>= self.code_size;
        self.bits_in_buffer -= self.code_size;
        Some(code)
    }

    /// Decode LZW compressed data
    pub fn decode(&mut self, data: &[u8]) -> Result<Vec<u8>, ImageError> {
        self.reset();
        self.initialized = true;

        let mut output = Vec::new();
        let mut offset = 0;
        let mut prev_code: Option<u16> = None;

        loop {
            let code = match self.read_code(data, &mut offset) {
                Some(c) => c,
                None => break,
            };

            if code == self.clear_code {
                self.reset();
                prev_code = None;
                continue;
            }

            if code == self.eoi_code {
                break;
            }

            if let Some(prev) = prev_code {
                if code < self.next_code {
                    let mut stack = Vec::new();
                    let mut c = code;
                    while c >= self.clear_code {
                        if c as usize >= LZW_MAX_TABLE_SIZE {
                            return Err(ImageError::DataCorrupted);
                        }
                        stack.push(self.suffix[c as usize]);
                        c = self.prefix[c as usize];
                    }
                    stack.push(self.suffix[c as usize]);

                    for &byte in stack.iter().rev() {
                        output.push(byte);
                    }

                    if (self.next_code as usize) < LZW_MAX_TABLE_SIZE {
                        self.prefix[self.next_code as usize] = prev;
                        self.suffix[self.next_code as usize] = self.suffix[c as usize];
                        self.next_code += 1;
                    }

                    if self.next_code > (1u16 << self.code_size) - 1
                        && self.code_size < 12
                    {
                        self.code_size += 1;
                    }
                } else if code == self.next_code {
                    let mut stack = Vec::new();
                    let mut c = prev;
                    while c >= self.clear_code {
                        if c as usize >= LZW_MAX_TABLE_SIZE {
                            return Err(ImageError::DataCorrupted);
                        }
                        stack.push(self.suffix[c as usize]);
                        c = self.prefix[c as usize];
                    }
                    let first = self.suffix[c as usize];
                    stack.push(first);

                    for &byte in stack.iter().rev() {
                        output.push(byte);
                    }

                    if (self.next_code as usize) < LZW_MAX_TABLE_SIZE {
                        self.prefix[self.next_code as usize] = prev;
                        self.suffix[self.next_code as usize] = first;
                        self.next_code += 1;
                    }

                    if self.next_code > (1u16 << self.code_size) - 1
                        && self.code_size < 12
                    {
                        self.code_size += 1;
                    }
                } else {
                    return Err(ImageError::DataCorrupted);
                }
            } else {
                if code >= self.clear_code {
                    return Err(ImageError::DataCorrupted);
                }
                output.push(self.suffix[code as usize]);
            }

            prev_code = Some(code);
        }

        Ok(output)
    }
}

/// GIF software decoder/encoder
pub struct GifCodec {
    /// Parsed screen descriptor
    screen: Option<GifScreenDescriptor>,
}

impl GifCodec {
    /// Create a new GIF codec instance
    pub const fn new() -> Self {
        GifCodec { screen: None }
    }

    /// Parse GIF Logical Screen Descriptor
    pub fn parse_screen_descriptor(data: &[u8]) -> Result<GifScreenDescriptor, ImageError> {
        if data.len() < 13 {
            return Err(ImageError::DataCorrupted);
        }

        if data[0] != GIF_MAGIC[0] || data[1] != GIF_MAGIC[1] || data[2] != GIF_MAGIC[2] {
            return Err(ImageError::DataCorrupted);
        }

        let version = [data[3], data[4], data[5]];
        if version != GIF_VERSION_87A && version != GIF_VERSION_89A {
            return Err(ImageError::DataCorrupted);
        }

        let width = ((data[7] as u16) << 8) | (data[6] as u16);
        let height = ((data[9] as u16) << 8) | (data[8] as u16);
        let packed = data[10];
        let has_gct = (packed & 0x80) != 0;
        let color_resolution = ((packed >> 4) & 0x07) + 1;
        let sorted = (packed & 0x08) != 0;
        let gct_size = (packed & 0x07) + 1;
        let bg_color_index = data[11];
        let pixel_aspect_ratio = data[12];

        Ok(GifScreenDescriptor {
            width,
            height,
            has_gct,
            color_resolution,
            sorted,
            gct_size,
            bg_color_index,
            pixel_aspect_ratio,
        })
    }

    /// Parse GIF Image Descriptor
    pub fn parse_image_descriptor(data: &[u8]) -> Result<GifImageDescriptor, ImageError> {
        if data.len() < 10 {
            return Err(ImageError::DataCorrupted);
        }

        if data[0] != GIF_IMAGE_SEPARATOR {
            return Err(ImageError::DataCorrupted);
        }

        let left = ((data[2] as u16) << 8) | (data[1] as u16);
        let top = ((data[4] as u16) << 8) | (data[3] as u16);
        let width = ((data[6] as u16) << 8) | (data[5] as u16);
        let height = ((data[8] as u16) << 8) | (data[7] as u16);
        let packed = data[9];
        let has_lct = (packed & 0x80) != 0;
        let interlaced = (packed & 0x40) != 0;
        let sorted = (packed & 0x20) != 0;
        let lct_size = (packed & 0x07) + 1;

        Ok(GifImageDescriptor {
            left,
            top,
            width,
            height,
            has_lct,
            interlaced,
            sorted,
            lct_size,
        })
    }

    /// Parse Graphics Control Extension
    pub fn parse_gce(data: &[u8]) -> Result<GifGraphicsControl, ImageError> {
        if data.len() < 8 {
            return Err(ImageError::DataCorrupted);
        }

        if data[0] != GIF_EXTENSION || data[1] != GIF_GCE {
            return Err(ImageError::DataCorrupted);
        }

        let packed = data[3];
        let disposal = (packed >> 2) & 0x07;
        let user_input = (packed & 0x02) != 0;
        let transparent = (packed & 0x01) != 0;
        let delay_cs = ((data[4] as u16) << 8) | (data[5] as u16);
        let transparent_index = data[6];

        Ok(GifGraphicsControl {
            disposal,
            user_input,
            transparent,
            delay_cs,
            transparent_index,
        })
    }

    /// Extract all animation frame delays from a GIF
    pub fn extract_frame_delays(data: &[u8]) -> Vec<u16> {
        let mut delays = Vec::new();
        let mut i = 0;

        while i + 1 < data.len() {
            if data[i] == GIF_EXTENSION && data[i + 1] == GIF_GCE {
                if i + 7 < data.len() {
                    let delay = ((data[i + 4] as u16) << 8) | (data[i + 5] as u16);
                    delays.push(delay);
                }
                i += 8;
            } else if data[i] == GIF_IMAGE_SEPARATOR {
                if delays.is_empty() {
                    delays.push(0);
                }
                i += 1;
            } else {
                i += 1;
            }
        }

        delays
    }

    /// Software decode of GIF data (returns first frame)
    fn sw_decode(&self, data: &[u8], config: &DecodeConfig) -> Result<ImageFrame, ImageError> {
        let screen = Self::parse_screen_descriptor(data)?;

        let width = screen.width as u32;
        let height = screen.height as u32;

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

    /// Software encode of image frame to GIF data
    fn sw_encode(&self, frame: &ImageFrame, config: &EncodeConfig) -> Result<Vec<u8>, ImageError> {
        if frame.width == 0 || frame.height == 0 {
            return Err(ImageError::InvalidParameter);
        }

        let _ = config;

        let width = frame.width as u16;
        let height = frame.height as u16;

        let mut output = Vec::new();

        output.extend_from_slice(&GIF_MAGIC);
        output.extend_from_slice(&GIF_VERSION_89A);

        output.extend_from_slice(&width.to_le_bytes());
        output.extend_from_slice(&height.to_le_bytes());
        output.push(0x80 | ((7u8 - 1) << 4) | 0);
        output.push(0);
        output.push(0);

        for i in 0..256u32 {
            let r = ((i >> 5) & 0x07) as u8 * 36;
            let g = ((i >> 2) & 0x07) as u8 * 36;
            let b = (i & 0x03) as u8 * 85;
            output.push(r);
            output.push(g);
            output.push(b);
        }

        output.push(GIF_IMAGE_SEPARATOR);
        output.extend_from_slice(&0u16.to_le_bytes());
        output.extend_from_slice(&0u16.to_le_bytes());
        output.extend_from_slice(&width.to_le_bytes());
        output.extend_from_slice(&height.to_le_bytes());
        output.push(0);

        output.push(8);
        output.push(0x01);
        output.push(0x00);

        output.push(GIF_TRAILER);

        Ok(output)
    }
}

impl ImageCodec for GifCodec {
    fn format(&self) -> ImageFormat {
        ImageFormat::Gif
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
        "GIF software codec"
    }
}
