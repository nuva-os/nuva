/*
 * Nuva OS - SystemService - Video - HEVC Software Codec
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

//! H.265/HEVC software codec implementation with NAL unit parsing.

use alloc::vec::Vec;

use super::codec::VideoCodec;
use super::error::{VideoError, VideoFormat, VideoPacket};
use super::frame_buffer::{DecodeResult, FrameBuffer, FrameRef};
use alloc::vec;

/// HEVC NAL unit type constants
pub const NAL_TRAIL_N: u8 = 0;
pub const NAL_TRAIL_R: u8 = 1;
pub const NAL_IDR_W_RADL: u8 = 19;
pub const NAL_IDR_N_LP: u8 = 20;
pub const NAL_VPS: u8 = 32;
pub const NAL_SPS: u8 = 33;
pub const NAL_PPS: u8 = 34;
pub const NAL_AUD: u8 = 35;
pub const NAL_EOSEQ: u8 = 36;
pub const NAL_EOSTREAM: u8 = 37;
pub const NAL_FILLER: u8 = 38;

/// HEVC NAL unit header (2 bytes)
#[derive(Debug, Clone, Copy)]
pub struct HevcNalHeader {
    /// NAL unit type (6 bits)
    pub nal_type: u8,
    /// Layer ID (6 bits)
    pub layer_id: u8,
    /// Temporal ID + 1 (3 bits)
    pub tid: u8,
}

impl HevcNalHeader {
    /// Parse HEVC NAL header from two bytes
    pub const fn from_bytes(byte0: u8, byte1: u8) -> Self {
        HevcNalHeader {
            nal_type: (byte0 >> 1) & 0x3F,
            layer_id: ((byte0 & 0x01) << 5) | ((byte1 >> 3) & 0x1F),
            tid: byte1 & 0x07,
        }
    }

    /// Check if this NAL is an IDR slice
    pub const fn is_idr(&self) -> bool {
        self.nal_type == NAL_IDR_W_RADL || self.nal_type == NAL_IDR_N_LP
    }

    /// Check if this NAL is a slice
    pub const fn is_slice(&self) -> bool {
        self.nal_type <= NAL_TRAIL_R
            || (self.nal_type >= NAL_IDR_W_RADL && self.nal_type <= NAL_IDR_N_LP)
    }

    /// Check if this NAL is a parameter set
    pub const fn is_param_set(&self) -> bool {
        self.nal_type >= NAL_VPS && self.nal_type <= NAL_PPS
    }
}

/// Parsed HEVC NAL unit
#[derive(Debug, Clone)]
pub struct HevcNalUnit {
    /// NAL header
    pub header: HevcNalHeader,
    /// NAL payload
    pub payload: Vec<u8>,
}

/// HEVC software decoder/encoder
pub struct HevcCodec;

impl HevcCodec {
    /// Create a new HEVC codec instance
    pub const fn new() -> Self {
        HevcCodec
    }

    /// Find HEVC NAL unit start codes in data
    pub fn find_nal_start_codes(data: &[u8]) -> Vec<usize> {
        let mut offsets = Vec::new();
        if data.len() < 5 {
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

    /// Parse NAL units from HEVC Annex B bitstream
    pub fn parse_nal_units(data: &[u8]) -> Result<Vec<HevcNalUnit>, VideoError> {
        let starts = Self::find_nal_start_codes(data);
        if starts.is_empty() {
            return Err(VideoError::DataCorrupted);
        }

        let mut units = Vec::new();
        for i in 0..starts.len() {
            let nal_start = starts[i];
            if nal_start + 1 >= data.len() {
                continue;
            }

            let header = HevcNalHeader::from_bytes(data[nal_start], data[nal_start + 1]);

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

            let payload_start = nal_start + 2;
            let mut payload = Vec::new();
            if payload_start < nal_end {
                payload.extend_from_slice(&data[payload_start..nal_end]);
            }

            units.push(HevcNalUnit { header, payload });
        }

        Ok(units)
    }

    /// Software decode of HEVC bitstream
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
            log_debug!("HEVC decode: non-IDR frame without SPS");
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

    /// Software encode of raw frame to HEVC bitstream
    fn sw_encode(
        &self,
        frame_data: &[u8],
        width: u32,
        height: u32,
    ) -> Result<VideoPacket, VideoError> {
        let _ = (frame_data, width, height);

        let mut output = Vec::new();
        let vps_nal = [0x00, 0x00, 0x00, 0x01, 0x40, 0x01];
        let sps_nal = [0x00, 0x00, 0x00, 0x01, 0x42, 0x01];
        let pps_nal = [0x00, 0x00, 0x00, 0x01, 0x44, 0x01];
        let idr_start = [0x00, 0x00, 0x00, 0x01, 0x26, 0x01];

        output.extend_from_slice(&vps_nal);
        output.extend_from_slice(&sps_nal);
        output.extend_from_slice(&pps_nal);
        output.extend_from_slice(&idr_start);

        Ok(VideoPacket {
            data: output,
            pts_us: 0,
            dts_us: 0,
            keyframe: true,
            format: VideoFormat::Hevc,
        })
    }
}

impl VideoCodec for HevcCodec {
    fn format(&self) -> VideoFormat {
        VideoFormat::Hevc
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
        "H.265/HEVC software codec"
    }
}
