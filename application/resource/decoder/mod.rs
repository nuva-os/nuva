/*
 * Nuva OS - Application - Resource - Decoder - Mod
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
/*
 * Nuva OS - Resource Decoder Bridge
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Bridges the application layer to the service layer for media decoding.
 * Delegates image decoding to services/image and audio decoding to
 * services/audio via the declarative resource manager.
 */

/// Image format enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    Png,
    Jpeg,
    WebP,
    Bmp,
    Gif,
    Unknown,
}

/// Audio format enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioFormat {
    Aac,
    Opus,
    Flac,
    Pcm,
    Unknown,
}

/// Font format enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontFormat {
    Ttf,
    Otf,
    Unknown,
}

/// Decoded image result.
#[derive(Debug, Clone)]
pub struct DecodedImage {
    pub width: u32,
    pub height: u32,
    pub format: ImageFormat,
    pub pixel_format: u32,
    pub data: &'static [u8],
}

/// Decoded audio result.
#[derive(Debug, Clone)]
pub struct DecodedAudio {
    pub sample_rate: u32,
    pub channels: u32,
    pub format: AudioFormat,
    pub data: &'static [u8],
    pub duration_ms: u32,
}

/// Detect image format from magic bytes.
pub fn detect_image_format(data: &[u8]) -> ImageFormat {
    if data.len() < 4 {
        return ImageFormat::Unknown;
    }
    match (data[0], data[1], data[2], data[3]) {
        (0x89, b'P', b'N', b'G') => ImageFormat::Png,
        (0xFF, 0xD8, 0xFF, _) => ImageFormat::Jpeg,
        (b'R', b'I', b'F', b'F') => ImageFormat::WebP,
        (b'B', b'M', _, _) => ImageFormat::Bmp,
        (b'G', b'I', b'F', b'8') => ImageFormat::Gif,
        _ => ImageFormat::Unknown,
    }
}

/// Detect audio format from magic bytes or container signature.
pub fn detect_audio_format(data: &[u8]) -> AudioFormat {
    if data.len() < 4 {
        return AudioFormat::Unknown;
    }
    match (data[0], data[1], data[2], data[3]) {
        (0xFF, 0xF1 | 0xF9, _, _) => AudioFormat::Aac,
        (b'O', b'p', b'u', b's') => AudioFormat::Opus,
        (b'f', b'L', b'a', b'C') => AudioFormat::Flac,
        (b'R', b'I', b'F', b'F') => AudioFormat::Pcm,
        _ => AudioFormat::Unknown,
    }
}

/// Detect font format from magic bytes.
pub fn detect_font_format(data: &[u8]) -> FontFormat {
    if data.len() < 4 {
        return FontFormat::Unknown;
    }
    match (data[0], data[1], data[2], data[3]) {
        (0x00, 0x01, 0x00, 0x00) => FontFormat::Ttf,
        (b'O', b'T', b'T', b'O') => FontFormat::Otf,
        (b't', b'r', b'u', b'e') => FontFormat::Ttf,
        _ => FontFormat::Unknown,
    }
}
