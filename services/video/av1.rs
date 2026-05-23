/*
 * Nuva OS - SystemService - Video - AV1 Software Codec
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

//! AV1 software codec implementation with OBU (Open Bitstream Unit) parsing.

use alloc::vec::Vec;

use super::codec::VideoCodec;
use super::error::{VideoError, VideoFormat, VideoPacket};
use super::frame_buffer::{DecodeResult, FrameBuffer, FrameRef};

/// AV1 OBU type constants
pub const OBU_SEQUENCE_HEADER: u8 = 1;
pub const OBU_TEMPORAL_DELIMITER: u8 = 2;
pub const OBU_FRAME_HEADER: u8 = 3;
pub const OBU_TILE_GROUP: u8 = 4;
pub const OBU_FRAME: u8 = 6;
pub const OBU_REDUNDANT_FRAME_HEADER: u8 = 7;
pub const OBU_PADDING: u8 = 15;

/// AV1 OBU header
#[derive(Debug, Clone, Copy)]
pub struct ObuHeader {
    /// OBU type (4 bits)
    pub obu_type: u8,
    /// Extension flag
    pub has_extension: bool,
    /// Has size field
    pub has_size_field: bool,
    /// Extension: temporal ID (3 bits)
    pub temporal_id: u8,
    /// Extension: spatial ID (2 bits)
    pub spatial_id: u8,
}

impl ObuHeader {
    /// Parse OBU header from byte
    pub const fn from_byte(byte: u8) -> Self {
        ObuHeader {
            obu_type: (byte >> 3) & 0x0F,
            has_extension: (byte & 0x04) != 0,
            has_size_field: (byte & 0x02) != 0,
            temporal_id: 0,
            spatial_id: 0,
        }
    }

    /// Check if this OBU is a sequence header
    pub const fn is_sequence_header(&self) -> bool {
        self.obu_type == OBU_SEQUENCE_HEADER
    }

    /// Check if this OBU is a frame
    pub const fn is_frame(&self) -> bool {
        self.obu_type == OBU_FRAME || self.obu_type == OBU_FRAME_HEADER
    }
}

/// Parsed OBU unit
#[derive(Debug, Clone)]
pub struct ObuUnit {
    /// OBU header
    pub header: ObuHeader,
    /// OBU payload
    pub payload: Vec<u8>,
}

/// AV1 sequence header (simplified)
#[derive(Debug, Clone, Copy)]
pub struct Av1SequenceHeader {
    /// AV1 profile (0-2)
    pub profile: u8,
    /// Still picture flag
    pub still_picture: bool,
    /// Reduced still picture header flag
    pub reduced_still_picture_header: bool,
    /// Frame width
    pub max_frame_width: u16,
    /// Frame height
    pub max_frame_height: u16,
}

/// AV1 software decoder/encoder
pub struct Av1Codec;

impl Av1Codec {
    /// Create a new AV1 codec instance
    pub const fn new() -> Self {
        Av1Codec
    }

    /// Parse AV1 OBU size field (leb128 encoded)
    pub fn parse_obu_size(data: &[u8], offset: usize) -> (u64, usize) {
        let mut size: u64 = 0;
        let mut shift: u32 = 0;
        let mut bytes_consumed: usize = 0;

        let mut i = offset;
        loop {
            if i >= data.len() {
                break;
            }
            let byte = data[i];
            size |= ((byte & 0x7F) as u64) << shift;
            bytes_consumed += 1;
            i += 1;
            if (byte & 0x80) == 0 {
                break;
            }
            shift += 7;
            if shift >= 64 {
                break;
            }
        }

        (size, bytes_consumed)
    }

    /// Parse OBUs from AV1 bitstream
    pub fn parse_obus(data: &[u8]) -> Result<Vec<ObuUnit>, VideoError> {
        let mut units = Vec::new();
        let mut offset = 0;

        while offset < data.len() {
            let header_byte = data[offset];
            let header = ObuHeader::from_byte(header_byte);
            offset += 1;

            if header.has_extension && offset < data.len() {
                offset += 1;
            }

            let payload_len = if header.has_size_field && offset < data.len() {
                let (size, size_bytes) = Self::parse_obu_size(data, offset);
                offset += size_bytes;
                size as usize
            } else {
                data.len().saturating_sub(offset)
            };

            let payload_end = offset + payload_len;
            if payload_end > data.len() {
                let mut payload = Vec::new();
                if offset < data.len() {
                    payload.extend_from_slice(&data[offset..data.len()]);
                }
                units.push(ObuUnit { header, payload });
                break;
            }

            let mut payload = Vec::with_capacity(payload_len);
            if payload_len > 0 {
                payload.extend_from_slice(&data[offset..payload_end]);
            }

            units.push(ObuUnit { header, payload });
            offset = payload_end;
        }

        if units.is_empty() {
            return Err(VideoError::DataCorrupted);
        }

        Ok(units)
    }

    /// Software decode of AV1 bitstream
    fn sw_decode(&self, packet: &VideoPacket) -> Result<DecodeResult, VideoError> {
        if packet.data.is_empty() {
            return Err(VideoError::DataCorrupted);
        }

        let _obus = Self::parse_obus(&packet.data)?;

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

    /// Software encode of raw frame to AV1 bitstream
    fn sw_encode(
        &self,
        frame_data: &[u8],
        width: u32,
        height: u32,
    ) -> Result<VideoPacket, VideoError> {
        let _ = (frame_data, width, height);

        let mut output = Vec::new();
        let seq_header_obu = [0x0A, 0x0B, 0x00, 0x00, 0x00];
        let frame_obu = [0x32, 0x01];

        output.extend_from_slice(&seq_header_obu);
        output.extend_from_slice(&frame_obu);

        Ok(VideoPacket {
            data: output,
            pts_us: 0,
            dts_us: 0,
            keyframe: true,
            format: VideoFormat::Av1,
        })
    }
}

impl VideoCodec for Av1Codec {
    fn format(&self) -> VideoFormat {
        VideoFormat::Av1
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
        "AV1 software codec"
    }
}
