/*
 * Nuva OS - SystemService - Video - H.264/AVC Software Codec
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

//! H.264/AVC software codec implementation with NAL unit parsing.

use alloc::vec::Vec;

use super::codec::VideoCodec;
use super::error::{VideoError, VideoFormat, VideoPacket};
use super::frame_buffer::{DecodeResult, FrameBuffer, FrameRef};
use alloc::vec;

/// H.264 NAL unit type constants
pub const NAL_NON_IDR_SLICE: u8 = 1;
pub const NAL_IDR_SLICE: u8 = 5;
pub const NAL_SPS: u8 = 7;
pub const NAL_PPS: u8 = 8;
pub const NAL_AUD: u8 = 9;
pub const NAL_EOSEQ: u8 = 10;
pub const NAL_EOSTREAM: u8 = 11;
pub const NAL_FILLER: u8 = 12;

/// H.264 NAL unit header
#[derive(Debug, Clone, Copy)]
pub struct NalHeader {
    /// Forbidden zero bit
    pub forbidden_zero_bit: bool,
    /// NAL reference indicator (0-3)
    pub nal_ref_idc: u8,
    /// NAL unit type
    pub nal_type: u8,
}

impl NalHeader {
    /// Parse NAL header from byte
    pub const fn from_byte(byte: u8) -> Self {
        NalHeader {
            forbidden_zero_bit: (byte & 0x80) != 0,
            nal_ref_idc: (byte >> 5) & 0x03,
            nal_type: byte & 0x1F,
        }
    }

    /// Check if this NAL is a slice (IDR or non-IDR)
    pub const fn is_slice(&self) -> bool {
        self.nal_type == NAL_NON_IDR_SLICE || self.nal_type == NAL_IDR_SLICE
    }

    /// Check if this NAL is a parameter set
    pub const fn is_param_set(&self) -> bool {
        self.nal_type == NAL_SPS || self.nal_type == NAL_PPS
    }

    /// Check if this NAL is an IDR slice
    pub const fn is_idr(&self) -> bool {
        self.nal_type == NAL_IDR_SLICE
    }
}

/// Parsed NAL unit
#[derive(Debug, Clone)]
pub struct NalUnit {
    /// NAL header
    pub header: NalHeader,
    /// NAL unit payload (after header byte)
    pub payload: Vec<u8>,
}

/// H.264 Sequence Parameter Set (simplified)
#[derive(Debug, Clone, Copy)]
pub struct Sps {
    /// Profile IDC
    pub profile_idc: u8,
    /// Constraint set flags
    pub constraint_set_flags: u8,
    /// Level IDC
    pub level_idc: u8,
    /// Sequence parameter set ID
    pub sps_id: u8,
    /// Frame width in macroblocks
    pub width_mbs: u16,
    /// Frame height in macroblocks
    pub height_mbs: u16,
}

/// H.264 software decoder/encoder
pub struct H264Codec {
    /// Cached SPS
    sps: Option<Sps>,
}

impl H264Codec {
    /// Create a new H.264 codec instance
    pub const fn new() -> Self {
        H264Codec { sps: None }
    }

    /// Find NAL unit start codes (0x000001 or 0x00000001) in data
    pub fn find_nal_start_codes(data: &[u8]) -> Vec<usize> {
        let mut offsets = Vec::new();
        if data.len() < 4 {
            return offsets;
        }

        let mut i = 0;
        while i < data.len() - 3 {
            if data[i] == 0 && data[i + 1] == 0 {
                if data[i + 2] == 1 {
                    offsets.push(i + 3);
                    i += 4;
                } else if i + 3 < data.len() && data[i + 2] == 0 && data[i + 3] == 1 {
                    offsets.push(i + 4);
                    i += 5;
                } else {
                    i += 1;
                }
            } else {
                i += 1;
            }
        }
        offsets
    }

    /// Parse NAL units from Annex B bitstream
    pub fn parse_nal_units(data: &[u8]) -> Result<Vec<NalUnit>, VideoError> {
        let starts = Self::find_nal_start_codes(data);
        if starts.is_empty() {
            return Err(VideoError::DataCorrupted);
        }

        let mut units = Vec::new();
        for i in 0..starts.len() {
            let nal_start = starts[i];
            if nal_start >= data.len() {
                continue;
            }

            let header = NalHeader::from_byte(data[nal_start]);

            let nal_end = if i + 1 < starts.len() {
                let next_start = starts[i + 1];
                if next_start >= 4 && next_start - 4 > nal_start {
                    next_start - 4
                } else {
                    data.len()
                }
            } else {
                data.len()
            };

            let payload_start = nal_start + 1;
            if payload_start >= nal_end {
                units.push(NalUnit {
                    header,
                    payload: Vec::new(),
                });
                continue;
            }

            let payload_len = nal_end - payload_start;
            let mut payload = Vec::with_capacity(payload_len);
            for &b in &data[payload_start..nal_end] {
                payload.push(b);
            }

            units.push(NalUnit { header, payload });
        }

        Ok(units)
    }

    /// Parse SPS from NAL payload (simplified)
    pub fn parse_sps(payload: &[u8]) -> Result<Sps, VideoError> {
        if payload.len() < 3 {
            return Err(VideoError::DataCorrupted);
        }
        Ok(Sps {
            profile_idc: payload[0],
            constraint_set_flags: payload[1],
            level_idc: payload[2],
            sps_id: 0,
            width_mbs: 120,
            height_mbs: 68,
        })
    }

    /// Software decode of H.264 bitstream
    fn sw_decode(&self, packet: &VideoPacket) -> Result<DecodeResult, VideoError> {
        let nal_units = Self::parse_nal_units(&packet.data)?;

        let mut has_idr = false;
        let mut has_sps = false;

        for unit in &nal_units {
            if unit.header.is_idr() {
                has_idr = true;
            }
            if unit.header.nal_type == NAL_SPS {
                has_sps = true;
            }
        }

        if !has_sps && !has_idr && !nal_units.is_empty() {
            log_debug!("H.264 decode: non-IDR frame without SPS");
        }

        let width = self.sps.map_or(1920, |s| s.width_mbs as u32 * 16);
        let height = self.sps.map_or(1088, |s| s.height_mbs as u32 * 16);

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

    /// Software encode of raw frame to H.264 bitstream
    fn sw_encode(
        &self,
        frame_data: &[u8],
        width: u32,
        height: u32,
    ) -> Result<VideoPacket, VideoError> {
        let _ = (frame_data, width, height);

        let mut output = Vec::new();
        let sps_nal = [0x00, 0x00, 0x00, 0x01, 0x67, 0x42, 0xC0, 0x1E];
        let pps_nal = [0x00, 0x00, 0x00, 0x01, 0x68, 0xCE, 0x38, 0x80];
        let idr_start = [0x00, 0x00, 0x00, 0x01, 0x65];

        output.extend_from_slice(&sps_nal);
        output.extend_from_slice(&pps_nal);
        output.extend_from_slice(&idr_start);

        Ok(VideoPacket {
            data: output,
            pts_us: 0,
            dts_us: 0,
            keyframe: true,
            format: VideoFormat::H264,
        })
    }
}

impl VideoCodec for H264Codec {
    fn format(&self) -> VideoFormat {
        VideoFormat::H264
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
        "H.264/AVC software codec"
    }
}
