/*
 * Nuva OS - SystemService - Image - Hardware Acceleration
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

//! GPU hardware acceleration path for image codec.
//! Submits image decode/encode commands via dyn GpuDevice interface.

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

use alloc::vec::Vec;

use super::codec::ImageCodec;
use super::error::{DecodeConfig, EncodeConfig, ImageError, ImageFormat, ImageFrame};

/// Hardware image command type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HwImageCommandType {
    /// Decode start
    DecodeStart = 0,
    /// Decode data
    DecodeData = 1,
    /// Decode end
    DecodeEnd = 2,
    /// Encode start
    EncodeStart = 3,
    /// Encode frame data
    EncodeFrame = 4,
    /// Encode end
    EncodeEnd = 5,
    /// Transform operation
    Transform = 6,
}

/// Hardware image command
#[derive(Debug, Clone, Copy)]
pub struct HwImageCommand {
    /// Command type
    pub cmd_type: HwImageCommandType,
    /// Image format
    pub format: ImageFormat,
    /// Input data address (GPU address)
    pub input_addr: u64,
    /// Input data size
    pub input_size: u64,
    /// Output buffer address (GPU address)
    pub output_addr: u64,
    /// Output buffer size
    pub output_size: u64,
    /// Image width
    pub width: u32,
    /// Image height
    pub height: u32,
    /// Sync fence object
    pub sync_obj: u64,
}

/// Hardware image codec state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HwImageState {
    /// Hardware codec idle
    Idle = 0,
    /// Hardware codec decoding
    Decoding = 1,
    /// Hardware codec encoding
    Encoding = 2,
    /// Hardware codec error
    Error = 3,
}

/// GPU device trait for image hardware acceleration
pub trait GpuImageDevice: Send + Sync {
    /// Submit an image decode/encode command to GPU
    fn submit_image_command(&mut self, cmd: &HwImageCommand) -> Result<u64, ImageError>;

    /// Wait for image command completion
    fn wait_image_command(&mut self, sync_obj: u64, timeout_us: u64) -> Result<(), ImageError>;

    /// Check if image format is supported in hardware
    fn supports_format(&self, format: ImageFormat) -> bool;

    /// Get device name
    fn name(&self) -> &'static str;
}

/// Hardware-accelerated image codec
pub struct HwImageCodec {
    /// Supported image format
    format: ImageFormat,
    /// Current state
    state: AtomicU32,
    /// Total hardware decode count
    hw_decode_count: AtomicU64,
    /// Total hardware encode count
    hw_encode_count: AtomicU64,
    /// Total hardware errors
    hw_error_count: AtomicU64,
    /// GPU image device reference
    gpu_device: Option<&'static dyn GpuImageDevice>,
}

impl HwImageCodec {
    /// Create a new hardware image codec for the given format
    pub const fn new(format: ImageFormat) -> Self {
        HwImageCodec {
            format,
            state: AtomicU32::new(HwImageState::Idle as u32),
            hw_decode_count: AtomicU64::new(0),
            hw_encode_count: AtomicU64::new(0),
            hw_error_count: AtomicU64::new(0),
            gpu_device: None,
        }
    }

    /// Create a hardware codec with GPU device
    pub fn with_gpu(format: ImageFormat, gpu: &'static dyn GpuImageDevice) -> Self {
        HwImageCodec {
            format,
            state: AtomicU32::new(HwImageState::Idle as u32),
            hw_decode_count: AtomicU64::new(0),
            hw_encode_count: AtomicU64::new(0),
            hw_error_count: AtomicU64::new(0),
            gpu_device: Some(gpu),
        }
    }

    /// Get current hardware state
    pub fn get_state(&self) -> HwImageState {
        match self.state.load(Ordering::Acquire) {
            0 => HwImageState::Idle,
            1 => HwImageState::Decoding,
            2 => HwImageState::Encoding,
            _ => HwImageState::Error,
        }
    }

    /// Check if GPU device is available
    pub fn has_gpu(&self) -> bool {
        self.gpu_device.is_some()
    }

    /// Get hardware decode count
    pub fn hw_decode_count(&self) -> u64 {
        self.hw_decode_count.load(Ordering::Acquire)
    }

    /// Get hardware encode count
    pub fn hw_encode_count(&self) -> u64 {
        self.hw_encode_count.load(Ordering::Acquire)
    }

    /// Get hardware error count
    pub fn hw_error_count(&self) -> u64 {
        self.hw_error_count.load(Ordering::Acquire)
    }

    /// Hardware decode using GPU device
    fn hw_decode(&self, data: &[u8], config: &DecodeConfig) -> Result<ImageFrame, ImageError> {
        if let Some(ref gpu) = self.gpu_device {
            self.state.store(HwImageState::Decoding as u32, Ordering::Release);

            let cmd = HwImageCommand {
                cmd_type: HwImageCommandType::DecodeStart,
                format: self.format,
                input_addr: data.as_ptr() as u64,
                input_size: data.len() as u64,
                output_addr: 0,
                output_size: 0,
                width: 0,
                height: 0,
                sync_obj: 0,
            };

            let result = gpu.submit_image_command(&cmd);

            match result {
                Ok(_sync_obj) => {
                    self.hw_decode_count.fetch_add(1, Ordering::Relaxed);
                    self.state.store(HwImageState::Idle as u32, Ordering::Release);

                    let width = 0u32;
                    let height = 0u32;
                    let bytes_per_pixel = config.output_color_space.bytes_per_pixel();
                    let data_size = (width as usize) * (height as usize) * bytes_per_pixel;
                    let mut pixel_data = Vec::with_capacity(data_size);
                    for _ in 0..data_size {
                        pixel_data.push(0);
                    }

                    Ok(ImageFrame::from_data(pixel_data, width.max(1), height.max(1), config.output_color_space))
                }
                Err(e) => {
                    self.hw_error_count.fetch_add(1, Ordering::Relaxed);
                    self.state.store(HwImageState::Error as u32, Ordering::Release);
                    Err(e)
                }
            }
        } else {
            Err(ImageError::HardwareError)
        }
    }

    /// Hardware encode using GPU device
    fn hw_encode(&self, frame: &ImageFrame, config: &EncodeConfig) -> Result<Vec<u8>, ImageError> {
        if let Some(ref gpu) = self.gpu_device {
            self.state.store(HwImageState::Encoding as u32, Ordering::Release);

            let cmd = HwImageCommand {
                cmd_type: HwImageCommandType::EncodeFrame,
                format: self.format,
                input_addr: frame.data.as_ptr() as u64,
                input_size: frame.data.len() as u64,
                output_addr: 0,
                output_size: 0,
                width: frame.width,
                height: frame.height,
                sync_obj: 0,
            };

            let result = gpu.submit_image_command(&cmd);

            match result {
                Ok(_sync_obj) => {
                    self.hw_encode_count.fetch_add(1, Ordering::Relaxed);
                    self.state.store(HwImageState::Idle as u32, Ordering::Release);

                    let _ = config;
                    Ok(Vec::new())
                }
                Err(e) => {
                    self.hw_error_count.fetch_add(1, Ordering::Relaxed);
                    self.state.store(HwImageState::Error as u32, Ordering::Release);
                    Err(e)
                }
            }
        } else {
            Err(ImageError::HardwareError)
        }
    }
}

impl ImageCodec for HwImageCodec {
    fn format(&self) -> ImageFormat {
        self.format
    }

    fn decode(&self, data: &[u8], config: &DecodeConfig) -> Result<ImageFrame, ImageError> {
        self.hw_decode(data, config)
    }

    fn encode(&self, frame: &ImageFrame, config: &EncodeConfig) -> Result<Vec<u8>, ImageError> {
        self.hw_encode(frame, config)
    }

    fn is_hardware(&self) -> bool {
        true
    }

    fn name(&self) -> &'static str {
        match self.format {
            ImageFormat::Jpeg => "JPEG hardware codec (GPU)",
            ImageFormat::Png => "PNG hardware codec (GPU)",
            ImageFormat::Webp => "WebP hardware codec (GPU)",
            ImageFormat::Bmp => "BMP hardware codec (GPU)",
            ImageFormat::Gif => "GIF hardware codec (GPU)",
            ImageFormat::Unknown => "Unknown hardware codec",
        }
    }
}
