/*
 * Nuva OS - SystemService - Video
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

//! Video codec service for Nuva OS.
//! Provides GPU/NPU-accelerated video decode/encode with software fallback,
//! per-caller decoder/encoder instances, frame buffer management with
//! zero-copy shared memory transfer, format auto-detection,
//! and power coordination with the system power service.

pub use crate::{pr_emerg, pr_alert, pr_crit, pr_err, pr_warn, pr_notice, pr_info, pr_debug};

pub mod service_node;
pub mod codec;
pub mod h264;
pub mod hevc;
pub mod vp9;
pub mod av1;
pub mod hw_accel;
pub mod sw_fallback;
pub mod frame_buffer;
pub mod format_detect;
pub mod power;
pub mod error;

pub use service_node::{VideoService, VideoAccelPath, VideoStats};
pub use error::{
    VideoError, VideoFormat, VideoPacket, VideoDecodeConfig, VideoEncodeConfig,
    DecoderId, EncoderId, PixelFormat,
};
pub use codec::{VideoCodec, CodecRegistry};
pub use frame_buffer::{FrameBuffer, FrameRef, DecodeResult, FrameBufferPool};
pub use power::VideoPowerManager;
pub use hw_accel::HwVideoCodec;
pub use sw_fallback::SwFallback;

/// Initialize the video codec service
pub fn init_video_service() {
    log_info!("Video service module loaded");
    // The VideoService is instantiated and initialized by
    // the system services manager via CoreProcessingService::init()
}
