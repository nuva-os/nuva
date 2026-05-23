/*
 * Nuva OS - SystemService - Image
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

//! Image codec service for Nuva OS.
//! Provides GPU-accelerated image decode/encode with software fallback,
//! multi-format support (JPEG, PNG, WebP, BMP, GIF), format auto-detection,
//! progressive decode, image transform pipeline (scale, rotate, crop,
//! color space conversion) with Nearest/Bilinear/Lanczos3 resampling.

pub use crate::{pr_emerg, pr_alert, pr_crit, pr_err, pr_warn, pr_notice, pr_info, pr_debug};

pub mod service_node;
pub mod codec;
pub mod jpeg;
pub mod png;
pub mod webp;
pub mod bmp;
pub mod gif;
pub mod transform;
pub mod hw_accel;
pub mod sw_fallback;
pub mod format_detect;
pub mod error;

pub use service_node::{ImageService, ImageAccelPath, ImageStats};
pub use error::{
    ImageError, ImageFormat, ImageFrame, DecodeConfig, EncodeConfig,
    ColorSpace, ProgressiveSession, DecoderId, EncoderId,
};
pub use codec::{ImageCodec, CodecRegistry};
pub use transform::{ImageTransform, TransformPipeline, ResampleFilter, Rotation};
pub use hw_accel::HwImageCodec;
pub use sw_fallback::SwFallback;
pub use format_detect::ImageDetectResult;

/// Initialize the image codec service
pub fn init_image_service() {
    log_info!("Image service module loaded");
    // The ImageService is instantiated and initialized by
    // the system services manager via CoreProcessingService::init()
}
