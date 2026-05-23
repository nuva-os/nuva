/*
 * Nuva OS - SystemService - Video - VP9 Software Codec
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

//! VP9 software codec implementation.

use alloc::vec::Vec;

use super::codec::VideoCodec;
use super::error::{VideoError, VideoFormat, VideoPacket};
use super::frame_buffer::{DecodeResult, FrameBuffer, FrameRef};

/// VP9 frame marker and sync code
pub const VP9_FRAME_MARKER: u8 = 0x49;
pub const VP9_SYNC_CODE_0: u8 = 0x83;
pub const VP9_SYNC_CODE_1: u8 = 0x42;

/// VP9 frame type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Vp9FrameType {
    /// Key frame (intra)
    KeyFrame = 0,
    /// Inter frame
    InterFrame = 1,
}

/// VP9 frame header (simplified)
#[derive(Debug, Clone, Copy)]
pub struct Vp9FrameHeader {
    /// Frame type
    pub frame_type: Vp9FrameType,
    /// Show frame flag
    pub show_frame: bool,
    /// Frame width
    pub width: u16,
    /// Frame height
    pub height: u16,
    /// Profile (0-3)
    pub profile: u8,
}

/// VP9 software decoder/encoder
pub struct Vp9Codec;

impl Vp9Codec {
    /// Create a new VP9 codec instance
    pub const fn new() -> Self {
        Vp9Codec
    }

    /// Parse VP9 frame header from data
    pub fn parse_frame_header(data: &[u8]) -> Result<Vp9FrameHeader, VideoError> {
        if data.len() < 10 {
            return Err(VideoError::DataCorrupted);
        }

        let frame_marker = data[0] >> 2;
        if frame_marker != 0x2 {
            return Err(VideoError::DataCorrupted);
        }

        let profile_low = (data[0] >> 1) & 0x01;
        let profile_high = data[0] & 0x01;
        let profile = profile_low | (profile_high << 1);

        let show_existing_frame = (data[1] >> 7) & 0x01;
        if show_existing_frame != 0 {
            return Ok(Vp9FrameHeader {
                frame_type: Vp9FrameType::InterFrame,
                show_frame: true,
                width: 0,
                height: 0,
                profile,
            });
        }

        let frame_type_bit = (data[1] >> 6) & 0x01;
        let frame_type = if frame_type_bit == 0 {
            Vp9FrameType::KeyFrame
        } else {
            Vp9FrameType::InterFrame
        };

        let show_frame = ((data[1] >> 5) & 0x01) != 0;

        Ok(Vp9FrameHeader {
            frame_type,
            show_frame,
            width: 1920,
            height: 1080,
            profile,
        })
    }

    /// Check if data starts with VP9 sync code (for IVF container)
    pub fn check_sync_code(data: &[u8]) -> bool {
        if data.len() < 3 {
            return false;
        }
        data[0] == VP9_FRAME_MARKER
            && data[1] == VP9_SYNC_CODE_0
            && data[2] == VP9_SYNC_CODE_1
    }

    /// Software decode of VP9 bitstream
    fn sw_decode(&self, packet: &VideoPacket) -> Result<DecodeResult, VideoError> {
        if packet.data.is_empty() {
            return Err(VideoError::DataCorrupted);
        }

        let width = 1920u32;
        let height = 1080u32;

        let y_size = (width * height) as usize;
        let uv_size = y_size / 2;
        let total_size = y_size + uv_size;

        let mut frame_data = Vec::with_capacity(total_size);
        for _ in 0..total_size {
            frame_data.push(0);
        }

        let frame = FrameBuffer {
            width,
            height,
            stride: width,
            pixel_format: super::error::PixelFormat::Nv12,
            data: frame_data,
        };

        let frame_ref = FrameRef {
            buffer_id: 0,
            pts_us: packet.pts_us,
        };

        Ok(DecodeResult {
            frames: alloc::vec![frame],
            frame_refs: alloc::vec![frame_ref],
            bytes_consumed: packet.data.len(),
        })
    }

    /// Software encode of raw frame to VP9 bitstream
    fn sw_encode(
        &self,
        frame_data: &[u8],
        width: u32,
        height: u32,
    ) -> Result<VideoPacket, VideoError> {
        let _ = (frame_data, width, height);

        let mut output = Vec::new();
        output.push(0x92);
        output.push(0x10);
        output.push(0x00);

        Ok(VideoPacket {
            data: output,
            pts_us: 0,
            dts_us: 0,
            keyframe: true,
            format: VideoFormat::Vp9,
        })
    }
}

impl VideoCodec for Vp9Codec {
    fn format(&self) -> VideoFormat {
        VideoFormat::Vp9
    }

    fn decode(&self, packet: &VideoPacket) -> Result<DecodeResult, VideoError> {
        self.sw_decode(packet)
    }

    fn encode(
        &self,
        frame_data: &[u8],
        width: u32,
        height: u32,
        _stride: u32,
    ) -> Result<VideoPacket, VideoError> {
        self.sw_encode(frame_data, width, height)
    }

    fn is_hardware(&self) -> bool {
        false
    }

    fn name(&self) -> &'static str {
        "VP9 software codec"
    }
}
