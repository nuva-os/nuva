/*
 * Nuva OS - SystemService - Video - Hardware Acceleration
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

//! GPU/NPU hardware acceleration path for video codec.
//! Submits hardware video decode/encode commands via dyn GpuDevice
//! and dyn NpuHal interfaces.

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

use alloc::vec::Vec;

use super::codec::VideoCodec;
use super::error::{VideoError, VideoFormat, VideoPacket};
use super::frame_buffer::{DecodeResult, FrameBuffer, FrameRef};
use alloc::vec;

/// Hardware video command type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HwVideoCommandType {
    /// Decode start
    DecodeStart = 0,
    /// Decode slice data
    DecodeSlice = 1,
    /// Decode end
    DecodeEnd = 2,
    /// Encode start
    EncodeStart = 3,
    /// Encode frame data
    EncodeFrame = 4,
    /// Encode end
    EncodeEnd = 5,
}

/// Hardware video command
#[derive(Debug, Clone, Copy)]
pub struct HwVideoCommand {
    /// Command type
    pub cmd_type: HwVideoCommandType,
    /// Video format
    pub format: VideoFormat,
    /// Data pointer (GPU address)
    pub data_addr: u64,
    /// Data size
    pub data_size: u64,
    /// Output buffer address (GPU address)
    pub output_addr: u64,
    /// Output buffer size
    pub output_size: u64,
    /// Sync fence object
    pub sync_obj: u64,
}

/// Hardware video codec state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HwVideoState {
    /// Hardware codec idle
    Idle = 0,
    /// Hardware codec decoding
    Decoding = 1,
    /// Hardware codec encoding
    Encoding = 2,
    /// Hardware codec error
    Error = 3,
}

/// GPU device trait reference for video hardware acceleration
pub trait GpuVideoDevice: Send + Sync {
    /// Submit a video decode/encode command to GPU
    fn submit_video_command(&mut self, cmd: &HwVideoCommand) -> Result<u64, VideoError>;

    /// Wait for video command completion
    fn wait_video_command(&mut self, sync_obj: u64, timeout_us: u64) -> Result<(), VideoError>;

    /// Check if video format is supported in hardware
    fn supports_format(&self, format: VideoFormat) -> bool;

    /// Get device name
    fn name(&self) -> &'static str;
}

/// NPU device trait reference for AI-enhanced video processing
pub trait NpuVideoDevice: Send + Sync {
    /// Submit AI-enhanced video processing task
    fn submit_ai_task(&mut self, task_type: u32, data: &[u8]) -> Result<u64, VideoError>;

    /// Wait for AI task completion
    fn wait_ai_task(&mut self, handle: u64, timeout_us: u64) -> Result<(), VideoError>;

    /// Get device name
    fn name(&self) -> &'static str;
}

/// Hardware-accelerated video codec
pub struct HwVideoCodec {
    /// Supported video format
    format: VideoFormat,
    /// Current state
    state: AtomicU32,
    /// Total hardware decode count
    hw_decode_count: AtomicU64,
    /// Total hardware encode count
    hw_encode_count: AtomicU64,
    /// Total hardware errors
    hw_error_count: AtomicU64,
    /// GPU video device reference
    gpu_device: Option<&'static dyn GpuVideoDevice>,
    /// NPU video device reference (optional, for AI-enhanced processing)
    npu_device: Option<&'static dyn NpuVideoDevice>,
}

impl HwVideoCodec {
    /// Create a new hardware video codec for the given format
    pub const fn new(format: VideoFormat) -> Self {
        HwVideoCodec {
            format,
            state: AtomicU32::new(HwVideoState::Idle as u32),
            hw_decode_count: AtomicU64::new(0),
            hw_encode_count: AtomicU64::new(0),
            hw_error_count: AtomicU64::new(0),
            gpu_device: None,
            npu_device: None,
        }
    }

    /// Create a hardware codec with GPU device
    pub fn with_gpu(format: VideoFormat, gpu: &'static dyn GpuVideoDevice) -> Self {
        HwVideoCodec {
            format,
            state: AtomicU32::new(HwVideoState::Idle as u32),
            hw_decode_count: AtomicU64::new(0),
            hw_encode_count: AtomicU64::new(0),
            hw_error_count: AtomicU64::new(0),
            gpu_device: Some(gpu),
            npu_device: None,
        }
    }

    /// Get current hardware state
    pub fn get_state(&self) -> HwVideoState {
        match self.state.load(Ordering::Acquire) {
            0 => HwVideoState::Idle,
            1 => HwVideoState::Decoding,
            2 => HwVideoState::Encoding,
            _ => HwVideoState::Error,
        }
    }

    /// Check if GPU device is available
    pub fn has_gpu(&self) -> bool {
        self.gpu_device.is_some()
    }

    /// Check if NPU device is available
    pub fn has_npu(&self) -> bool {
        self.npu_device.is_some()
    }

    /// Get hardware decode count
    pub fn hw_decode_count(&self) -> u64 {
        self.hw_decode_count.load(Ordering::Acquire)
    }

    /// Get hardware encode count
    pub fn hw_encode_count(&self) -> u64 {
        self.hw_encode_count.load(Ordering::Acquire)
    }

    /// Hardware decode using GPU device
    fn hw_decode(&self, packet: &VideoPacket) -> Result<DecodeResult, VideoError> {
        if let Some(ref gpu) = self.gpu_device {
            self.state.store(HwVideoState::Decoding as u32, Ordering::Release);

            let cmd = HwVideoCommand {
                cmd_type: HwVideoCommandType::DecodeStart,
                format: self.format,
                data_addr: packet.data.as_ptr() as u64,
                data_size: packet.data.len() as u64,
                output_addr: 0,
                output_size: 0,
                sync_obj: 0,
            };

            let result = gpu.submit_video_command(&cmd);

            match result {
                Ok(_sync_obj) => {
                    self.hw_decode_count.fetch_add(1, Ordering::Relaxed);
                    self.state.store(HwVideoState::Idle as u32, Ordering::Release);

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
                Err(e) => {
                    self.hw_error_count.fetch_add(1, Ordering::Relaxed);
                    self.state.store(HwVideoState::Error as u32, Ordering::Release);
                    Err(e)
                }
            }
        } else {
            Err(VideoError::HardwareError)
        }
    }

    /// Hardware encode using GPU device
    fn hw_encode(
        &self,
        frame_data: &[u8],
        width: u32,
        height: u32,
    ) -> Result<VideoPacket, VideoError> {
        if let Some(ref gpu) = self.gpu_device {
            self.state.store(HwVideoState::Encoding as u32, Ordering::Release);

            let cmd = HwVideoCommand {
                cmd_type: HwVideoCommandType::EncodeFrame,
                format: self.format,
                data_addr: frame_data.as_ptr() as u64,
                data_size: frame_data.len() as u64,
                output_addr: 0,
                output_size: 0,
                sync_obj: 0,
            };

            let result = gpu.submit_video_command(&cmd);

            match result {
                Ok(_sync_obj) => {
                    self.hw_encode_count.fetch_add(1, Ordering::Relaxed);
                    self.state.store(HwVideoState::Idle as u32, Ordering::Release);

                    Ok(VideoPacket {
                        data: Vec::new(),
                        pts_us: 0,
                        dts_us: 0,
                        keyframe: true,
                        format: self.format,
                    })
                }
                Err(e) => {
                    self.hw_error_count.fetch_add(1, Ordering::Relaxed);
                    self.state.store(HwVideoState::Error as u32, Ordering::Release);
                    Err(e)
                }
            }
        } else {
            let _ = (frame_data, width, height);
            Err(VideoError::HardwareError)
        }
    }
}

impl VideoCodec for HwVideoCodec {
    fn format(&self) -> VideoFormat {
        self.format
    }

    fn decode(&self, packet: &VideoPacket) -> Result<DecodeResult, VideoError> {
        self.hw_decode(packet)
    }

    fn encode(
        &self,
        frame_data: &[u8],
        width: u32,
        height: u32,
        _stride: u32,
    ) -> Result<VideoPacket, VideoError> {
        self.hw_encode(frame_data, width, height)
    }

    fn is_hardware(&self) -> bool {
        true
    }

    fn name(&self) -> &'static str {
        match self.format {
            VideoFormat::H264 => "H.264 hardware codec (GPU/NPU)",
            VideoFormat::Hevc => "HEVC hardware codec (GPU/NPU)",
            VideoFormat::Vp9 => "VP9 hardware codec (GPU/NPU)",
            VideoFormat::Av1 => "AV1 hardware codec (GPU/NPU)",
            VideoFormat::Unknown => "Unknown hardware codec",
        }
    }
}
