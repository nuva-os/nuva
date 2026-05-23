/*
 * Nuva OS - SystemService - Video - Software Fallback
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

//! Software fallback path for video codec.
//! When hardware acceleration fails or is unavailable, this module
//! dispatches to the appropriate software codec implementation.

use core::sync::atomic::{AtomicU64, Ordering};

use super::codec::VideoCodec;
use super::error::{VideoError, VideoFormat, VideoPacket};
use super::frame_buffer::DecodeResult;
use super::h264::H264Codec;
use super::hevc::HevcCodec;
use super::vp9::Vp9Codec;
use super::av1::Av1Codec;

/// Software fallback manager for video codec
pub struct SwFallback {
    /// H.264 software codec
    h264: H264Codec,
    /// HEVC software codec
    hevc: HevcCodec,
    /// VP9 software codec
    vp9: Vp9Codec,
    /// AV1 software codec
    av1: Av1Codec,
    /// Fallback invocation count
    fallback_count: AtomicU64,
}

impl SwFallback {
    /// Create a new software fallback manager
    pub const fn new() -> Self {
        SwFallback {
            h264: H264Codec::new(),
            hevc: HevcCodec::new(),
            vp9: Vp9Codec::new(),
            av1: Av1Codec::new(),
            fallback_count: AtomicU64::new(0),
        }
    }

    /// Get the software codec for a given format
    pub fn get_codec(&self, format: VideoFormat) -> Option<&dyn VideoCodec> {
        match format {
            VideoFormat::H264 => Some(&self.h264 as &dyn VideoCodec),
            VideoFormat::Hevc => Some(&self.hevc as &dyn VideoCodec),
            VideoFormat::Vp9 => Some(&self.vp9 as &dyn VideoCodec),
            VideoFormat::Av1 => Some(&self.av1 as &dyn VideoCodec),
            VideoFormat::Unknown => None,
        }
    }

    /// Decode using software fallback for the given format
    pub fn decode(&self, packet: &VideoPacket) -> Result<DecodeResult, VideoError> {
        self.fallback_count.fetch_add(1, Ordering::Relaxed);

        let codec = self.get_codec(packet.format)
            .ok_or(VideoError::FormatNotSupported)?;

        log_debug!(
            "Software fallback decode for format={:?}",
            packet.format
        );

        codec.decode(packet)
    }

    /// Encode using software fallback for the given format
    pub fn encode(
        &self,
        format: VideoFormat,
        frame_data: &[u8],
        width: u32,
        height: u32,
        stride: u32,
    ) -> Result<VideoPacket, VideoError> {
        self.fallback_count.fetch_add(1, Ordering::Relaxed);

        let codec = self.get_codec(format)
            .ok_or(VideoError::FormatNotSupported)?;

        log_debug!(
            "Software fallback encode for format={:?}",
            format
        );

        codec.encode(frame_data, width, height, stride)
    }

    /// Get total fallback count
    pub fn fallback_count(&self) -> u64 {
        self.fallback_count.load(Ordering::Acquire)
    }

    /// Check if a format has a software fallback available
    pub fn has_fallback(&self, format: VideoFormat) -> bool {
        matches!(format, VideoFormat::H264 | VideoFormat::Hevc | VideoFormat::Vp9 | VideoFormat::Av1)
    }
}
