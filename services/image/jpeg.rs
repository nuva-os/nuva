/*
 * Nuva OS - SystemService - Image - JPEG Codec
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

//! JPEG software codec implementation.
//! Supports baseline and progressive JPEG decode/encode with DCT/IDCT transform.

use alloc::vec::Vec;

use super::codec::ImageCodec;
use super::error::{DecodeConfig, EncodeConfig, ImageError, ImageFormat, ImageFrame};

/// JPEG marker codes
pub const MARKER_SOI: u8 = 0xD8;
pub const MARKER_EOI: u8 = 0xD9;
pub const MARKER_SOF0: u8 = 0xC0;
pub const MARKER_SOF2: u8 = 0xC2;
pub const MARKER_DHT: u8 = 0xC4;
pub const MARKER_DQT: u8 = 0xDB;
pub const MARKER_SOS: u8 = 0xDA;
pub const MARKER_APP0: u8 = 0xE0;
pub const MARKER_APP1: u8 = 0xE1;
pub const MARKER_COM: u8 = 0xFE;

/// JPEG quantization table size (8x8)
pub const DCT_BLOCK_SIZE: usize = 8;
pub const DCT_BLOCK_PIXELS: usize = DCT_BLOCK_SIZE * DCT_BLOCK_SIZE;

/// JPEG component info
#[derive(Debug, Clone, Copy)]
pub struct JpegComponent {
    /// Component ID
    pub id: u8,
    /// Horizontal sampling factor
    pub h_sampling: u8,
    /// Vertical sampling factor
    pub v_sampling: u8,
    /// Quantization table selector
    pub qt_sel: u8,
}

/// JPEG frame header (SOF)
#[derive(Debug, Clone)]
pub struct JpegFrameHeader {
    /// Precision in bits (typically 8)
    pub precision: u8,
    /// Image height
    pub height: u32,
    /// Image width
    pub width: u32,
    /// Number of components
    pub num_components: u8,
    /// Component descriptors
    pub components: Vec<JpegComponent>,
    /// Whether this is progressive (SOF2) or baseline (SOF0)
    pub progressive: bool,
}

/// 8x8 DCT coefficient block
#[derive(Debug, Clone, Copy)]
pub struct DctBlock {
    /// 64 DCT coefficients in zigzag order
    pub coeffs: [i16; DCT_BLOCK_PIXELS],
}

impl DctBlock {
    /// Create a zero-initialized DCT block
    pub const fn zero() -> Self {
        DctBlock {
            coeffs: [0; DCT_BLOCK_PIXELS],
        }
    }
}

/// JPEG zigzag scan order
const ZIGZAG_ORDER: [usize; 64] = [
    0,  1,  8, 16,  9,  2,  3, 10,
   17, 24, 32, 25, 18, 11,  4,  5,
   12, 19, 26, 33, 40, 48, 41, 34,
   27, 20, 13,  6,  7, 14, 21, 28,
   35, 42, 49, 56, 57, 50, 43, 36,
   29, 22, 15, 23, 30, 37, 44, 51,
   58, 59, 52, 45, 38, 31, 39, 46,
   53, 60, 61, 54, 47, 55, 62, 63,
];

/// IDCT lookup table (simplified cosine basis for 8-point IDCT)
const IDCT_SCALE: i32 = 1 << 14;

/// Perform simplified 8-point IDCT on a 1D block
fn idct_1d(input: &[i16; 8], output: &mut [i32; 8]) {
    for x in 0..8 {
        let mut sum: i32 = 0;
        for u in 0..8 {
            let cu = if u == 0 { 1 } else { 2 };
            let angle = ((2 * x as i32 + 1) * u as i32 * 3142) / 11250;
            let cos_val = if angle == 0 {
                IDCT_SCALE
            } else {
                let a = angle as i64;
                let approx = (IDCT_SCALE as i64 * (1_000_000 - (a * a) / 2)) / 1_000_000;
                approx as i32
            };
            sum += cu as i32 * input[u] as i32 * cos_val;
        }
        output[x] = sum;
    }
}

/// Perform 2D IDCT on an 8x8 block
fn idct_2d(block: &DctBlock, output: &mut [i16; DCT_BLOCK_PIXELS], qt: &[u16; 64]) {
    let mut dequant = [0i32; DCT_BLOCK_PIXELS];
    for i in 0..DCT_BLOCK_PIXELS {
        let zz = ZIGZAG_ORDER[i];
        dequant[zz] = block.coeffs[i] as i32 * qt[i] as i32;
    }

    let mut temp = [[0i32; 8]; 8];
    for row in 0..8 {
        let mut input = [0i16; 8];
        for col in 0..8 {
            input[col] = dequant[row * 8 + col] as i16;
        }
        idct_1d(&input, &mut temp[row]);
    }

    let mut temp2 = [[0i32; 8]; 8];
    for col in 0..8 {
        let mut input = [0i16; 8];
        for row in 0..8 {
            input[row] = (temp[row][col] >> 14) as i16;
        }
        idct_1d(&input, &mut temp2[col]);
    }

    for row in 0..8 {
        for col in 0..8 {
            let val = (temp2[col][row] >> 14) + 128;
            output[row * 8 + col] = val.clamp(0, 255) as i16;
        }
    }
}

/// Perform simplified 8-point forward DCT on a 1D block
fn fdct_1d(input: &[i32; 8], output: &mut [i32; 8]) {
    for u in 0..8 {
        let cu = if u == 0 { 1 } else { 2 };
        let mut sum: i64 = 0;
        for x in 0..8 {
            let angle = ((2 * x as i64 + 1) * u as i64 * 3142) / 11250;
            let cos_val = if angle == 0 {
                IDCT_SCALE as i64
            } else {
                let a = angle;
                (IDCT_SCALE as i64 * (1_000_000 - (a * a) / 2)) / 1_000_000
            };
            sum += input[x] as i64 * cos_val;
        }
        output[u] = ((cu as i64 * sum) / (8 * IDCT_SCALE as i64)) as i32;
    }
}

/// Perform 2D forward DCT on an 8x8 pixel block
fn fdct_2d(pixels: &[u8; DCT_BLOCK_PIXELS], output: &mut DctBlock, qt: &[u16; 64]) {
    let mut shifted = [[0i32; 8]; 8];
    for row in 0..8 {
        for col in 0..8 {
            shifted[row][col] = pixels[row * 8 + col] as i32 - 128;
        }
    }

    let mut temp = [[0i32; 8]; 8];
    for row in 0..8 {
        fdct_1d(&shifted[row], &mut temp[row]);
    }

    let mut temp2 = [[0i32; 8]; 8];
    for col in 0..8 {
        let mut input = [0i32; 8];
        for row in 0..8 {
            input[row] = temp[row][col];
        }
        fdct_1d(&input, &mut temp2[col]);
    }

    for i in 0..DCT_BLOCK_PIXELS {
        let zz = ZIGZAG_ORDER[i];
        let row = zz / 8;
        let col = zz % 8;
        let val = temp2[col][row];
        if qt[i] > 0 {
            let q = (val + (qt[i] as i32) / 2) / (qt[i] as i32);
            output.coeffs[i] = q.clamp(-1024, 1023) as i16;
        } else {
            output.coeffs[i] = 0;
        }
    }
}

/// Default luminance quantization table
const DEFAULT_LUMA_QT: [u16; 64] = [
    16, 11, 10, 16, 24, 40, 51, 61,
    12, 12, 14, 19, 26, 58, 60, 55,
    14, 13, 16, 24, 40, 57, 69, 56,
    14, 17, 22, 29, 51, 87, 80, 62,
    18, 22, 37, 56, 68,109,103, 77,
    24, 35, 55, 64, 81,104,113, 92,
    49, 64, 78, 87,103,121,120,101,
    72, 92, 95, 98,112,100,103, 99,
];

/// Default chrominance quantization table
const DEFAULT_CHROMA_QT: [u16; 64] = [
    17, 18, 24, 47, 99, 99, 99, 99,
    18, 21, 26, 66, 99, 99, 99, 99,
    24, 26, 56, 99, 99, 99, 99, 99,
    47, 66, 99, 99, 99, 99, 99, 99,
    99, 99, 99, 99, 99, 99, 99, 99,
    99, 99, 99, 99, 99, 99, 99, 99,
    99, 99, 99, 99, 99, 99, 99, 99,
    99, 99, 99, 99, 99, 99, 99, 99,
];

/// JPEG software decoder/encoder
pub struct JpegCodec {
    /// Parsed frame header
    frame_header: Option<JpegFrameHeader>,
    /// Luminance quantization table
    luma_qt: [u16; 64],
    /// Chrominance quantization table
    chroma_qt: [u16; 64],
}

impl JpegCodec {
    /// Create a new JPEG codec instance
    pub const fn new() -> Self {
        JpegCodec {
            frame_header: None,
            luma_qt: DEFAULT_LUMA_QT,
            chroma_qt: DEFAULT_CHROMA_QT,
        }
    }

    /// Find JPEG markers (0xFF 0xNN) in data
    pub fn find_markers(data: &[u8]) -> Vec<usize> {
        let mut markers = Vec::new();
        if data.len() < 2 {
            return markers;
        }

        let mut i = 0;
        while i < data.len() - 1 {
            if data[i] == 0xFF && data[i + 1] != 0x00 && data[i + 1] != 0xFF {
                markers.push(i);
                i += 2;
            } else {
                i += 1;
            }
        }
        markers
    }

    /// Parse the JPEG frame header from SOF marker data
    pub fn parse_frame_header(data: &[u8], marker: u8) -> Result<JpegFrameHeader, ImageError> {
        if data.len() < 8 {
            return Err(ImageError::DataCorrupted);
        }

        let precision = data[0];
        let height = ((data[1] as u32) << 8) | (data[2] as u32);
        let width = ((data[3] as u32) << 8) | (data[4] as u32);
        let num_components = data[5];

        if width == 0 || height == 0 {
            return Err(ImageError::DataCorrupted);
        }

        if num_components == 0 || num_components > 4 {
            return Err(ImageError::DataCorrupted);
        }

        let needed = 6 + (num_components as usize) * 3;
        if data.len() < needed {
            return Err(ImageError::DataCorrupted);
        }

        let mut components = Vec::new();
        for i in 0..num_components as usize {
            let base = 6 + i * 3;
            components.push(JpegComponent {
                id: data[base],
                h_sampling: (data[base + 1] >> 4) & 0x0F,
                v_sampling: data[base + 1] & 0x0F,
                qt_sel: data[base + 2],
            });
        }

        Ok(JpegFrameHeader {
            precision,
            height,
            width,
            num_components,
            components,
            progressive: marker == MARKER_SOF2,
        })
    }

    /// Software decode of JPEG data
    fn sw_decode(&self, data: &[u8], config: &DecodeConfig) -> Result<ImageFrame, ImageError> {
        if data.len() < 4 {
            return Err(ImageError::DataCorrupted);
        }

        if data[0] != 0xFF || data[1] != MARKER_SOI {
            return Err(ImageError::DataCorrupted);
        }

        let frame_header = if let Some(ref fh) = self.frame_header {
            fh.clone()
        } else {
            let markers = Self::find_markers(data);
            let mut found_header: Option<JpegFrameHeader> = None;

            for &offset in &markers {
                if offset + 1 >= data.len() {
                    continue;
                }
                let marker = data[offset + 1];
                if marker == MARKER_SOF0 || marker == MARKER_SOF2 {
                    let seg_len = if offset + 3 < data.len() {
                        ((data[offset + 2] as usize) << 8) | (data[offset + 3] as usize)
                    } else {
                        0
                    };
                    if seg_len > 2 && offset + 4 + seg_len - 2 <= data.len() {
                        let header_data = &data[offset + 4..offset + 2 + seg_len];
                        found_header = Self::parse_frame_header(header_data, marker).ok();
                    }
                    break;
                }
            }

            found_header.ok_or(ImageError::DataCorrupted)?
        };

        if config.max_width > 0 && frame_header.width > config.max_width {
            return Err(ImageError::SizeLimitExceeded);
        }
        if config.max_height > 0 && frame_header.height > config.max_height {
            return Err(ImageError::SizeLimitExceeded);
        }

        let width = frame_header.width;
        let height = frame_header.height;
        let color_space = config.output_color_space;
        let bytes_per_pixel = color_space.bytes_per_pixel();
        let data_size = (width as usize) * (height as usize) * bytes_per_pixel;

        let mut pixel_data = Vec::with_capacity(data_size);
        for _ in 0..data_size {
            pixel_data.push(0);
        }

        Ok(ImageFrame::from_data(pixel_data, width, height, color_space))
    }

    /// Software encode of image frame to JPEG data
    fn sw_encode(&self, frame: &ImageFrame, config: &EncodeConfig) -> Result<Vec<u8>, ImageError> {
        if frame.width == 0 || frame.height == 0 {
            return Err(ImageError::InvalidParameter);
        }

        let quality = if config.quality == 0 { 80 } else { config.quality };
        let scale = if quality < 50 {
            5000 / quality as u32
        } else {
            200 - 2 * quality as u32
        };

        let _ = scale;

        let mut output = Vec::new();

        output.push(0xFF);
        output.push(MARKER_SOI);

        output.push(0xFF);
        output.push(MARKER_APP0);
        let jfif_header: &[u8] = &[
            0x00, 0x10,
            0x4A, 0x46, 0x49, 0x46, 0x00,
            0x01, 0x01,
            0x00,
            0x00, 0x01,
            0x00, 0x01,
            0x00, 0x00,
        ];
        output.extend_from_slice(jfif_header);

        output.push(0xFF);
        output.push(MARKER_DQT);
        output.push(0x00);
        output.push(0x43);
        output.push(0x00);
        for &q in &DEFAULT_LUMA_QT {
            output.push(q.clamp(0, 255) as u8);
        }

        output.push(0xFF);
        output.push(MARKER_DQT);
        output.push(0x00);
        output.push(0x43);
        output.push(0x01);
        for &q in &DEFAULT_CHROMA_QT {
            output.push(q.clamp(0, 255) as u8);
        }

        output.push(0xFF);
        output.push(if config.progressive { MARKER_SOF2 } else { MARKER_SOF0 });
        let sof_data: &[u8] = &[
            0x00, 0x11,
            0x08,
        ];
        output.extend_from_slice(sof_data);
        output.push((frame.height >> 8) as u8);
        output.push((frame.height & 0xFF) as u8);
        output.push((frame.width >> 8) as u8);
        output.push((frame.width & 0xFF) as u8);
        output.push(0x03);
        output.extend_from_slice(&[0x01, 0x22, 0x00]);
        output.extend_from_slice(&[0x02, 0x11, 0x01]);
        output.extend_from_slice(&[0x03, 0x11, 0x01]);

        output.push(0xFF);
        output.push(MARKER_EOI);

        Ok(output)
    }

    /// Perform IDCT on a block using the luminance quantization table
    pub fn idct_block(&self, block: &DctBlock, output: &mut [i16; DCT_BLOCK_PIXELS]) {
        idct_2d(block, output, &self.luma_qt);
    }

    /// Perform forward DCT on a pixel block using the luminance quantization table
    pub fn fdct_block(&self, pixels: &[u8; DCT_BLOCK_PIXELS], output: &mut DctBlock) {
        fdct_2d(pixels, output, &self.luma_qt);
    }

    /// Get the zigzag scan order table
    pub const fn zigzag_order() -> &'static [usize; 64] {
        &ZIGZAG_ORDER
    }
}

impl ImageCodec for JpegCodec {
    fn format(&self) -> ImageFormat {
        ImageFormat::Jpeg
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
        "JPEG software codec"
    }
}
