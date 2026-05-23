/*
 * Nuva OS - SystemService - Video - Format Detection
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

//! Video format auto-detection using magic bytes and bitstream analysis.
//! Identifies H.264, HEVC, VP9, and AV1 formats from file/stream headers.

use crate::services::core_processing::format_detect::{
    probe_format, FormatProbeResult, MagicEntry,
    FORMAT_H264, FORMAT_HEVC, FORMAT_VP9, FORMAT_AV1,
};

use super::error::{VideoError, VideoFormat};

/// IVF container header magic: "DKIF"
const IVF_MAGIC: [u8; 4] = [0x44, 0x4B, 0x49, 0x46];

/// WebM container header magic: "\x1A\x45\xDF\xA3"
const WEBM_MAGIC: [u8; 4] = [0x1A, 0x45, 0xDF, 0xA3];

/// MP4/MOV container magic (ftyp box)
const MP4_MAGIC: [u8; 4] = [0x66, 0x74, 0x79, 0x70];

/// Magic bytes table for video format detection
static VIDEO_MAGIC_ENTRIES: &[MagicEntry] = &[
    MagicEntry::new(FORMAT_H264, &[0x00, 0x00, 0x00, 0x01], 0),
    MagicEntry::new(FORMAT_HEVC, &[0x00, 0x00, 0x00, 0x01], 0),
];

/// Container type detected from file header
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoContainer {
    /// Raw bitstream (Annex B)
    RawBitstream = 0,
    /// IVF container (VP9/AV1)
    Ivf = 1,
    /// WebM container (Matroska subset)
    WebM = 2,
    /// MP4/MOV container
    Mp4 = 3,
    /// Unknown container
    Unknown = 255,
}

/// Video format detection result with container info
#[derive(Debug, Clone, Copy)]
pub struct VideoDetectResult {
    /// Detected video format
    pub format: VideoFormat,
    /// Detected container type
    pub container: VideoContainer,
    /// Detection confidence (0-100)
    pub confidence: u8,
}

/// Detect video container from header bytes
pub fn detect_container(data: &[u8]) -> VideoContainer {
    if data.len() < 8 {
        return VideoContainer::Unknown;
    }

    if data[0] == IVF_MAGIC[0]
        && data[1] == IVF_MAGIC[1]
        && data[2] == IVF_MAGIC[2]
        && data[3] == IVF_MAGIC[3]
    {
        return VideoContainer::Ivf;
    }

    if data[0] == WEBM_MAGIC[0]
        && data[1] == WEBM_MAGIC[1]
        && data[2] == WEBM_MAGIC[2]
        && data[3] == WEBM_MAGIC[3]
    {
        return VideoContainer::WebM;
    }

    if data.len() >= 12 {
        let ftyp_offset = 4;
        if data[ftyp_offset] == MP4_MAGIC[0]
            && data[ftyp_offset + 1] == MP4_MAGIC[1]
            && data[ftyp_offset + 2] == MP4_MAGIC[2]
            && data[ftyp_offset + 3] == MP4_MAGIC[3]
        {
            return VideoContainer::Mp4;
        }
    }

    if data.len() >= 4
        && data[0] == 0x00
        && data[1] == 0x00
        && (data[2] == 0x00 || data[2] == 0x01)
    {
        return VideoContainer::RawBitstream;
    }

    VideoContainer::Unknown
}

/// Detect video format from Annex B NAL header byte
fn detect_nal_format(data: &[u8]) -> Option<VideoFormat> {
    let start_codes: &[usize] = &[0];
    let _ = start_codes;

    let mut i = 0;
    while i + 3 < data.len() {
        if data[i] == 0 && data[i + 1] == 0 {
            let nal_start = if data[i + 2] == 1 {
                i + 3
            } else if i + 3 < data.len() && data[i + 2] == 0 && data[i + 3] == 1 {
                i + 4
            } else {
                i += 1;
                continue;
            };

            if nal_start >= data.len() {
                return None;
            }

            let first_byte = data[nal_start];
            let nal_type_h264 = first_byte & 0x1F;
            if nal_type_h264 == 7 || nal_type_h264 == 8 || nal_type_h264 == 5 {
                return Some(VideoFormat::H264);
            }

            if nal_start + 1 < data.len() {
                let nal_type_hevc = (first_byte >> 1) & 0x3F;
                if nal_type_hevc == 32 || nal_type_hevc == 33 || nal_type_hevc == 34 {
                    return Some(VideoFormat::Hevc);
                }
            }

            return None;
        }
        i += 1;
    }
    None
}

/// Detect VP9 format from IVF container
fn detect_ivf_format(data: &[u8]) -> Option<VideoFormat> {
    if data.len() < 32 {
        return None;
    }

    let codec_id = &data[8..12];
    if codec_id == b"VP90" {
        return Some(VideoFormat::Vp9);
    }
    if codec_id == b"AV01" {
        return Some(VideoFormat::Av1);
    }
    None
}

/// Detect video format from raw data/stream header
pub fn detect_video_format(data: &[u8]) -> Result<VideoDetectResult, VideoError> {
    if data.is_empty() {
        return Err(VideoError::InvalidParameter);
    }

    let container = detect_container(data);

    match container {
        VideoContainer::Ivf => {
            if let Some(format) = detect_ivf_format(data) {
                return Ok(VideoDetectResult {
                    format,
                    container,
                    confidence: 95,
                });
            }
        }
        VideoContainer::RawBitstream => {
            if let Some(format) = detect_nal_format(data) {
                return Ok(VideoDetectResult {
                    format,
                    container,
                    confidence: 90,
                });
            }
        }
        VideoContainer::WebM => {
            return Ok(VideoDetectResult {
                format: VideoFormat::Vp9,
                container,
                confidence: 60,
            });
        }
        VideoContainer::Mp4 => {
            if let Some(probe) = probe_format(data, VIDEO_MAGIC_ENTRIES) {
                return Ok(VideoDetectResult {
                    format: VideoFormat::from_format_id(probe.format_id),
                    container,
                    confidence: probe.confidence,
                });
            }
            return Ok(VideoDetectResult {
                format: VideoFormat::H264,
                container,
                confidence: 50,
            });
        }
        VideoContainer::Unknown => {}
    }

    if let Some(probe) = probe_format(data, VIDEO_MAGIC_ENTRIES) {
        return Ok(VideoDetectResult {
            format: VideoFormat::from_format_id(probe.format_id),
            container: VideoContainer::RawBitstream,
            confidence: probe.confidence,
        });
    }

    Err(VideoError::FormatNotSupported)
}
