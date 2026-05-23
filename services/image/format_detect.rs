/*
 * Nuva OS - SystemService - Image - Format Detection
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

//! Image format auto-detection using magic bytes.
//! Identifies JPEG, PNG, WebP, BMP, and GIF formats from file/stream headers.

use crate::services::core_processing::format_detect::{
    probe_format, FormatProbeResult, MagicEntry,
    FORMAT_JPEG, FORMAT_PNG, FORMAT_WEBP, FORMAT_BMP, FORMAT_GIF,
};

use super::error::{ImageError, ImageFormat};

/// Magic bytes table for image format detection
static IMAGE_MAGIC_ENTRIES: &[MagicEntry] = &[
    MagicEntry::new(FORMAT_JPEG, &[0xFF, 0xD8, 0xFF], 0),
    MagicEntry::new(FORMAT_PNG, &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A], 0),
    MagicEntry::new(FORMAT_BMP, &[0x42, 0x4D], 0),
    MagicEntry::new(FORMAT_GIF, &[0x47, 0x49, 0x46], 0),
];

/// WebP RIFF header magic: "RIFF"
const RIFF_MAGIC: [u8; 4] = [0x52, 0x49, 0x46, 0x46];

/// WebP format magic: "WEBP"
const WEBP_MAGIC: [u8; 4] = [0x57, 0x45, 0x42, 0x50];

/// GIF version bytes for "87a" and "89a"
const GIF_VERSION_87A: [u8; 3] = [0x38, 0x37, 0x61];
const GIF_VERSION_89A: [u8; 3] = [0x38, 0x39, 0x61];

/// Image format detection result with additional metadata
#[derive(Debug, Clone, Copy)]
pub struct ImageDetectResult {
    /// Detected image format
    pub format: ImageFormat,
    /// Detection confidence (0-100)
    pub confidence: u8,
    /// Whether the image is likely animated
    pub animated: bool,
}

/// Check if data matches WebP format (RIFF....WEBP)
fn is_webp_format(data: &[u8]) -> bool {
    if data.len() < 12 {
        return false;
    }

    if data[0] != RIFF_MAGIC[0]
        || data[1] != RIFF_MAGIC[1]
        || data[2] != RIFF_MAGIC[2]
        || data[3] != RIFF_MAGIC[3]
    {
        return false;
    }

    if data[8] != WEBP_MAGIC[0]
        || data[9] != WEBP_MAGIC[1]
        || data[10] != WEBP_MAGIC[2]
        || data[11] != WEBP_MAGIC[3]
    {
        return false;
    }

    true
}

/// Check if data matches GIF format with version
fn is_gif_format(data: &[u8]) -> bool {
    if data.len() < 6 {
        return false;
    }

    if data[0] != 0x47 || data[1] != 0x49 || data[2] != 0x46 {
        return false;
    }

    let version = [data[3], data[4], data[5]];
    version == GIF_VERSION_87A || version == GIF_VERSION_89A
}

/// Detect if a GIF image is animated by checking for multiple graphic control blocks
fn detect_gif_animated(data: &[u8]) -> bool {
    if data.len() < 13 {
        return false;
    }

    let mut offset = 13;
    let _ = data[6..10].iter();
    let has_gct = (data[10] & 0x80) != 0;
    if has_gct {
        let gct_size = 1 << ((data[10] & 0x07) + 1);
        offset += gct_size * 3;
    }

    let mut block_count = 0u32;
    while offset < data.len() {
        let block_type = data[offset];
        if block_type == 0x3B {
            break;
        }
        if block_type == 0x21 {
            offset += 1;
            if offset >= data.len() {
                break;
            }
            offset += 1;
            while offset < data.len() {
                let sub_block_size = data[offset] as usize;
                offset += 1;
                if sub_block_size == 0 {
                    break;
                }
                offset += sub_block_size;
            }
        } else if block_type == 0x2C {
            block_count += 1;
            if block_count > 1 {
                return true;
            }
            if offset + 9 >= data.len() {
                break;
            }
            let lct_flag = (data[offset + 9] & 0x80) != 0;
            offset += 10;
            if lct_flag {
                if offset >= data.len() {
                    break;
                }
                let lct_size = 1 << ((data[offset - 1] & 0x07) + 1);
                offset += lct_size * 3;
            }
            if offset >= data.len() {
                break;
            }
            offset += 1;
            if offset >= data.len() {
                break;
            }
            let min_code_size = data[offset] as usize;
            offset += 1;
            let _ = min_code_size;
            while offset < data.len() {
                let sub_block_size = data[offset] as usize;
                offset += 1;
                if sub_block_size == 0 {
                    break;
                }
                offset += sub_block_size;
            }
        } else {
            break;
        }
    }

    false
}

/// Detect image format from raw data/stream header
pub fn detect_image_format(data: &[u8]) -> Result<ImageDetectResult, ImageError> {
    if data.is_empty() {
        return Err(ImageError::InvalidParameter);
    }

    if is_webp_format(data) {
        return Ok(ImageDetectResult {
            format: ImageFormat::Webp,
            confidence: 99,
            animated: false,
        });
    }

    if is_gif_format(data) {
        let animated = detect_gif_animated(data);
        return Ok(ImageDetectResult {
            format: ImageFormat::Gif,
            confidence: 99,
            animated,
        });
    }

    if let Some(probe) = probe_format(data, IMAGE_MAGIC_ENTRIES) {
        let format = ImageFormat::from_format_id(probe.format_id);
        return Ok(ImageDetectResult {
            format,
            confidence: probe.confidence,
            animated: false,
        });
    }

    Err(ImageError::FormatNotSupported)
}
