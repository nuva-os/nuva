/*
 * Nuva OS - SystemService - Image - Software Fallback
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

//! Software fallback path for image codec.
//! When hardware acceleration fails or is unavailable, this module
//! dispatches to the appropriate software codec implementation.

use core::sync::atomic::{AtomicU64, Ordering};

use alloc::vec::Vec;

use super::codec::ImageCodec;
use super::error::{DecodeConfig, EncodeConfig, ImageError, ImageFormat, ImageFrame};
use super::bmp::BmpCodec;
use super::gif::GifCodec;
use super::jpeg::JpegCodec;
use super::png::PngCodec;
use super::webp::WebpCodec;

/// Software fallback manager for image codec
pub struct SwFallback {
    /// JPEG software codec
    jpeg: JpegCodec,
    /// PNG software codec
    png: PngCodec,
    /// WebP software codec
    webp: WebpCodec,
    /// BMP software codec
    bmp: BmpCodec,
    /// GIF software codec
    gif: GifCodec,
    /// Fallback invocation count
    fallback_count: AtomicU64,
}

impl SwFallback {
    /// Create a new software fallback manager
    pub const fn new() -> Self {
        SwFallback {
            jpeg: JpegCodec::new(),
            png: PngCodec::new(),
            webp: WebpCodec::new(),
            bmp: BmpCodec::new(),
            gif: GifCodec::new(),
            fallback_count: AtomicU64::new(0),
        }
    }

    /// Get the software codec for a given format
    pub fn get_codec(&self, format: ImageFormat) -> Option<&dyn ImageCodec> {
        match format {
            ImageFormat::Jpeg => Some(&self.jpeg as &dyn ImageCodec),
            ImageFormat::Png => Some(&self.png as &dyn ImageCodec),
            ImageFormat::Webp => Some(&self.webp as &dyn ImageCodec),
            ImageFormat::Bmp => Some(&self.bmp as &dyn ImageCodec),
            ImageFormat::Gif => Some(&self.gif as &dyn ImageCodec),
            ImageFormat::Unknown => None,
        }
    }

    /// Decode using software fallback for the given format
    pub fn decode(
        &self,
        data: &[u8],
        format: ImageFormat,
        config: &DecodeConfig,
    ) -> Result<ImageFrame, ImageError> {
        self.fallback_count.fetch_add(1, Ordering::Relaxed);

        let codec = self.get_codec(format)
            .ok_or(ImageError::FormatNotSupported)?;

        log_debug!(
            "Software fallback decode for format={:?}",
            format
        );

        codec.decode(data, config)
    }

    /// Encode using software fallback for the given format
    pub fn encode(
        &self,
        frame: &ImageFrame,
        format: ImageFormat,
        config: &EncodeConfig,
    ) -> Result<Vec<u8>, ImageError> {
        self.fallback_count.fetch_add(1, Ordering::Relaxed);

        let codec = self.get_codec(format)
            .ok_or(ImageError::FormatNotSupported)?;

        log_debug!(
            "Software fallback encode for format={:?}",
            format
        );

        codec.encode(frame, config)
    }

    /// Get total fallback count
    pub fn fallback_count(&self) -> u64 {
        self.fallback_count.load(Ordering::Acquire)
    }

    /// Check if a format has a software fallback available
    pub fn has_fallback(&self, format: ImageFormat) -> bool {
        matches!(
            format,
            ImageFormat::Jpeg | ImageFormat::Png | ImageFormat::Webp
            | ImageFormat::Bmp | ImageFormat::Gif
        )
    }
}
