/*
 * Nuva OS - SystemService - CoreProcessing - Format Detection
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

//! Format detection framework using magic bytes matching.
//! Shared by video, audio, and image services.

/// Maximum magic bytes length for detection
pub const MAX_MAGIC_LEN: usize = 16;

/// Format probe result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FormatProbeResult {
    /// Detected format identifier
    pub format_id: u32,
    /// Confidence level (0-100)
    pub confidence: u8,
}

/// Magic bytes entry for format detection
pub struct MagicEntry {
    /// Format identifier
    pub format_id: u32,
    /// Magic bytes to match
    pub magic: [u8; MAX_MAGIC_LEN],
    /// Length of magic bytes (actual significant bytes)
    pub magic_len: u8,
    /// Offset in the data where magic appears
    pub offset: u8,
}

impl MagicEntry {
    /// Create a new magic entry
    pub const fn new(format_id: u32, magic: &[u8], offset: u8) -> Self {
        let mut buf = [0u8; MAX_MAGIC_LEN];
        let len = magic.len();
        let mut i = 0;
        while i < len && i < MAX_MAGIC_LEN {
            buf[i] = magic[i];
            i += 1;
        }
        MagicEntry {
            format_id,
            magic: buf,
            magic_len: len as u8,
            offset,
        }
    }
}

/// Probe data format using magic bytes
pub fn probe_format(data: &[u8], entries: &[MagicEntry]) -> Option<FormatProbeResult> {
    for entry in entries {
        let offset = entry.offset as usize;
        let len = entry.magic_len as usize;
        if data.len() < offset + len {
            continue;
        }
        let mut matched = true;
        for i in 0..len {
            if data[offset + i] != entry.magic[i] {
                matched = false;
                break;
            }
        }
        if matched {
            return Some(FormatProbeResult {
                format_id: entry.format_id,
                confidence: 100,
            });
        }
    }
    None
}

// Common format IDs shared across services
/// JPEG format ID
pub const FORMAT_JPEG: u32 = 1;
/// PNG format ID
pub const FORMAT_PNG: u32 = 2;
/// WebP format ID
pub const FORMAT_WEBP: u32 = 3;
/// BMP format ID
pub const FORMAT_BMP: u32 = 4;
/// GIF format ID
pub const FORMAT_GIF: u32 = 5;
/// H.264 format ID
pub const FORMAT_H264: u32 = 10;
/// HEVC format ID
pub const FORMAT_HEVC: u32 = 11;
/// VP9 format ID
pub const FORMAT_VP9: u32 = 12;
/// AV1 format ID
pub const FORMAT_AV1: u32 = 13;
/// AAC format ID
pub const FORMAT_AAC: u32 = 20;
/// Opus format ID
pub const FORMAT_OPUS: u32 = 21;
/// FLAC format ID
pub const FORMAT_FLAC: u32 = 22;
/// PCM format ID
pub const FORMAT_PCM: u32 = 23;
