/*
 * Nuva OS - SystemService - Image - Service Node
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

//! Image service node implementing CoreProcessingService trait.
//! Registered as "nuva.service.image" in the Nuva IPC framework.

use alloc::collections::BTreeMap;
use core::sync::atomic::{AtomicU64, Ordering};

use crate::services::core_processing::service_node::{
    CallerIdentity, CoreProcessingService, ServiceConfig, ServiceHealth, ServiceNodeId,
    ServiceStats, ServiceVersion,
};
use crate::services::core_processing::error::ServiceError;

use super::codec::CodecRegistry;
use super::error::{
    DecodeConfig, EncodeConfig, ImageError, ImageFormat, ImageFrame,
    DecoderId, EncoderId, ProgressiveSession,
};
use super::format_detect::{detect_image_format, ImageDetectResult};
use super::hw_accel::HwImageCodec;
use super::sw_fallback::SwFallback;
use super::transform::TransformPipeline;
use alloc::vec::Vec;

/// Convert ImageError to ServiceError
impl From<ImageError> for ServiceError {
    fn from(e: ImageError) -> ServiceError {
        match e {
            ImageError::NotInitialized => ServiceError::NotInitialized,
            ImageError::OutOfMemory => ServiceError::OutOfMemory,
            ImageError::HardwareError => ServiceError::Specific(
                crate::services::core_processing::error::ServiceSpecificError::ImageFormatNotSupported,
            ),
            ImageError::FormatNotSupported => ServiceError::Specific(
                crate::services::core_processing::error::ServiceSpecificError::ImageFormatNotSupported,
            ),
            ImageError::DataCorrupted => ServiceError::Specific(
                crate::services::core_processing::error::ServiceSpecificError::ImageDataCorrupted,
            ),
            ImageError::ColorSpaceNotSupported => ServiceError::Specific(
                crate::services::core_processing::error::ServiceSpecificError::ImageColorSpaceNotSupported,
            ),
            ImageError::SizeLimitExceeded => ServiceError::Specific(
                crate::services::core_processing::error::ServiceSpecificError::ImageSizeLimitExceeded,
            ),
            ImageError::InvalidParameter => ServiceError::Specific(
                crate::services::core_processing::error::ServiceSpecificError::ImageInvalidParameter,
            ),
            _ => ServiceError::Specific(
                crate::services::core_processing::error::ServiceSpecificError::ImageFormatNotSupported,
            ),
        }
    }
}

/// Acceleration path for codec operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageAccelPath {
    /// Hardware acceleration (GPU)
    Hardware = 0,
    /// Software fallback
    Software = 1,
}

/// Decoder instance state
struct DecoderInstance {
    /// Decoder ID
    id: DecoderId,
    /// Decode configuration
    config: DecodeConfig,
    /// Owner PID
    owner_pid: u32,
    /// Acceleration path
    accel_path: ImageAccelPath,
}

/// Encoder instance state
struct EncoderInstance {
    /// Encoder ID
    id: EncoderId,
    /// Encode configuration
    config: EncodeConfig,
    /// Owner PID
    owner_pid: u32,
    /// Acceleration path
    accel_path: ImageAccelPath,
}

/// Image service statistics
#[derive(Debug)]
pub struct ImageStats {
    /// Total decoders created
    pub total_decoders: AtomicU64,
    /// Total encoders created
    pub total_encoders: AtomicU64,
    /// Total frames decoded
    pub total_frames_decoded: AtomicU64,
    /// Total frames encoded
    pub total_frames_encoded: AtomicU64,
    /// Total transform operations
    pub total_transforms: AtomicU64,
    /// Software fallback count
    pub fallback_count: AtomicU64,
}

impl ImageStats {
    /// Create zero-initialized stats
    pub const fn new() -> Self {
        ImageStats {
            total_decoders: AtomicU64::new(0),
            total_encoders: AtomicU64::new(0),
            total_frames_decoded: AtomicU64::new(0),
            total_frames_encoded: AtomicU64::new(0),
            total_transforms: AtomicU64::new(0),
            fallback_count: AtomicU64::new(0),
        }
    }
}

/// Image service
pub struct ImageService {
    /// Service configuration
    config: ServiceConfig,
    /// Core service statistics
    stats: ServiceStats,
    /// Image-specific statistics
    image_stats: ImageStats,
    /// Codec registry
    codec_registry: CodecRegistry,
    /// Software fallback
    sw_fallback: SwFallback,
    /// Active decoder instances
    decoders: BTreeMap<u64, DecoderInstance>,
    /// Active encoder instances
    encoders: BTreeMap<u64, EncoderInstance>,
    /// Progressive decode sessions
    progressive_sessions: BTreeMap<u64, ProgressiveSession>,
    /// Next decoder ID
    next_decoder_id: AtomicU64,
    /// Next encoder ID
    next_encoder_id: AtomicU64,
    /// Next progressive session ID
    next_session_id: AtomicU64,
    /// Whether the service is initialized
    initialized: bool,
}

impl ImageService {
    /// Create a new image service instance
    pub fn new() -> Self {
        let config = ServiceConfig {
            name: "nuva.service.image",
            version: ServiceVersion::new(1, 0, 0),
            max_concurrent_requests: 64,
            request_timeout_us: 50_000,
            hw_accel_available: true,
        };

        ImageService {
            config,
            stats: ServiceStats::new(),
            image_stats: ImageStats::new(),
            codec_registry: CodecRegistry::new(),
            sw_fallback: SwFallback::new(),
            decoders: BTreeMap::new(),
            encoders: BTreeMap::new(),
            progressive_sessions: BTreeMap::new(),
            next_decoder_id: AtomicU64::new(1),
            next_encoder_id: AtomicU64::new(1),
            next_session_id: AtomicU64::new(1),
            initialized: false,
        }
    }

    /// Decode image data using the specified decoder
    pub fn decode(
        &mut self,
        caller: CallerIdentity,
        decoder_id: DecoderId,
        data: &[u8],
    ) -> Result<ImageFrame, ImageError> {
        if !self.initialized {
            return Err(ImageError::NotInitialized);
        }

        let decoder = self.decoders.get(&decoder_id.0)
            .ok_or(ImageError::DecoderNotFound)?;

        if decoder.owner_pid != caller.pid {
            return Err(ImageError::InvalidParameter);
        }

        let format = decoder.config.format;
        let accel_path = decoder.accel_path;

        let result = if accel_path == ImageAccelPath::Hardware {
            if let Some(codec) = self.codec_registry.select(format) {
                match codec.decode(data, &decoder.config) {
                    Ok(frame) => Ok(frame),
                    Err(_) => {
                        log_warn!("HW decode failed, falling back to SW for format={:?}", format);
                        self.image_stats.fallback_count.fetch_add(1, Ordering::Relaxed);
                        self.sw_fallback.decode(data, format, &decoder.config)
                    }
                }
            } else {
                self.sw_fallback.decode(data, format, &decoder.config)
            }
        } else {
            self.sw_fallback.decode(data, format, &decoder.config)
        };

        if result.is_ok() {
            self.image_stats.total_frames_decoded.fetch_add(1, Ordering::Relaxed);
        }

        result
    }

    /// Encode image frame using the specified encoder
    pub fn encode(
        &mut self,
        caller: CallerIdentity,
        encoder_id: EncoderId,
        frame: &ImageFrame,
    ) -> Result<alloc::vec::Vec<u8>, ImageError> {
        if !self.initialized {
            return Err(ImageError::NotInitialized);
        }

        let encoder = self.encoders.get(&encoder_id.0)
            .ok_or(ImageError::EncoderNotFound)?;

        if encoder.owner_pid != caller.pid {
            return Err(ImageError::InvalidParameter);
        }

        let format = encoder.config.format;
        let accel_path = encoder.accel_path;

        let result = if accel_path == ImageAccelPath::Hardware {
            if let Some(codec) = self.codec_registry.select(format) {
                match codec.encode(frame, &encoder.config) {
                    Ok(data) => Ok(data),
                    Err(_) => {
                        log_warn!("HW encode failed, falling back to SW for format={:?}", format);
                        self.image_stats.fallback_count.fetch_add(1, Ordering::Relaxed);
                        self.sw_fallback.encode(frame, format, &encoder.config)
                    }
                }
            } else {
                self.sw_fallback.encode(frame, format, &encoder.config)
            }
        } else {
            self.sw_fallback.encode(frame, format, &encoder.config)
        };

        if result.is_ok() {
            self.image_stats.total_frames_encoded.fetch_add(1, Ordering::Relaxed);
        }

        result
    }

    /// Apply a transform pipeline to an image frame
    pub fn transform(
        &self,
        frame: &ImageFrame,
        pipeline: &TransformPipeline,
    ) -> Result<ImageFrame, ImageError> {
        if !self.initialized {
            return Err(ImageError::NotInitialized);
        }

        let result = pipeline.apply(frame);

        if result.is_ok() {
            self.image_stats.total_transforms.fetch_add(1, Ordering::Relaxed);
        }

        result
    }

    /// Detect image format from data
    pub fn detect_format(&self, data: &[u8]) -> Result<ImageDetectResult, ImageError> {
        detect_image_format(data)
    }

    /// Start a progressive decode session
    pub fn progressive_decode_start(
        &mut self,
        format: ImageFormat,
    ) -> Result<ProgressiveSession, ImageError> {
        if !self.initialized {
            return Err(ImageError::NotInitialized);
        }

        let total_passes = match format {
            ImageFormat::Jpeg => 10,
            ImageFormat::Png => 7,
            _ => 1,
        };

        let session_id = self.next_session_id.fetch_add(1, Ordering::Relaxed);
        let mut session = ProgressiveSession::new(session_id, format, total_passes);
        session.state = super::error::ProgressiveState::Scanning;

        self.progressive_sessions.insert(session_id, session.clone());

        Ok(session)
    }

    /// Feed data to a progressive decode session
    pub fn progressive_decode_feed(
        &mut self,
        session_id: u64,
        data: &[u8],
    ) -> Result<(ImageFrame, ProgressiveSession), ImageError> {
        if !self.initialized {
            return Err(ImageError::NotInitialized);
        }

        let session = self.progressive_sessions.get_mut(&session_id)
            .ok_or(ImageError::InvalidParameter)?;

        session.bytes_consumed += data.len();

        if session.current_pass < session.total_passes {
            session.current_pass += 1;
            session.quality = ((session.current_pass * 100) / session.total_passes) as u8;
        }

        if session.current_pass >= session.total_passes {
            session.state = super::error::ProgressiveState::Complete;
        } else {
            session.state = super::error::ProgressiveState::NeedMoreData;
        }

        let frame = ImageFrame::new(1, 1, super::error::ColorSpace::Rgba8);
        let session_copy = session.clone();

        Ok((frame, session_copy))
    }

    /// Create a decoder instance for the given format
    pub fn create_decoder(
        &mut self,
        caller: CallerIdentity,
        config: DecodeConfig,
    ) -> Result<DecoderId, ImageError> {
        if !self.initialized {
            return Err(ImageError::NotInitialized);
        }

        let has_hw = self.codec_registry.select_hardware(config.format).is_some();
        let has_sw = self.codec_registry.select_software(config.format).is_some();

        if !has_hw && !has_sw && !self.sw_fallback.has_fallback(config.format) {
            return Err(ImageError::FormatNotSupported);
        }

        let accel_path = if config.hw_accel && has_hw {
            ImageAccelPath::Hardware
        } else {
            ImageAccelPath::Software
        };

        let decoder_id = DecoderId(self.next_decoder_id.fetch_add(1, Ordering::Relaxed));

        self.decoders.insert(decoder_id.0, DecoderInstance {
            id: decoder_id,
            config,
            owner_pid: caller.pid,
            accel_path,
        });

        self.image_stats.total_decoders.fetch_add(1, Ordering::Relaxed);

        log_debug!(
            "Created image decoder id={} format={:?} accel={:?}",
            decoder_id.0,
            self.decoders.get(&decoder_id.0).map_or(ImageFormat::Unknown, |d| d.config.format),
            accel_path,
        );

        Ok(decoder_id)
    }

    /// Create an encoder instance for the given format
    pub fn create_encoder(
        &mut self,
        caller: CallerIdentity,
        config: EncodeConfig,
    ) -> Result<EncoderId, ImageError> {
        if !self.initialized {
            return Err(ImageError::NotInitialized);
        }

        let has_hw = self.codec_registry.select_hardware(config.format).is_some();
        let has_sw = self.codec_registry.select_software(config.format).is_some();

        if !has_hw && !has_sw && !self.sw_fallback.has_fallback(config.format) {
            return Err(ImageError::FormatNotSupported);
        }

        let accel_path = if config.hw_accel && has_hw {
            ImageAccelPath::Hardware
        } else {
            ImageAccelPath::Software
        };

        let encoder_id = EncoderId(self.next_encoder_id.fetch_add(1, Ordering::Relaxed));

        self.encoders.insert(encoder_id.0, EncoderInstance {
            id: encoder_id,
            config,
            owner_pid: caller.pid,
            accel_path,
        });

        self.image_stats.total_encoders.fetch_add(1, Ordering::Relaxed);

        log_debug!(
            "Created image encoder id={} format={:?} accel={:?}",
            encoder_id.0,
            self.encoders.get(&encoder_id.0).map_or(ImageFormat::Unknown, |e| e.config.format),
            accel_path,
        );

        Ok(encoder_id)
    }

    /// Destroy a decoder instance
    pub fn destroy_decoder(
        &mut self,
        caller: CallerIdentity,
        decoder_id: DecoderId,
    ) -> Result<(), ImageError> {
        if !self.initialized {
            return Err(ImageError::NotInitialized);
        }

        let decoder = self.decoders.get(&decoder_id.0)
            .ok_or(ImageError::DecoderNotFound)?;

        if decoder.owner_pid != caller.pid {
            return Err(ImageError::InvalidParameter);
        }

        self.decoders.remove(&decoder_id.0);
        log_debug!("Destroyed image decoder id={}", decoder_id.0);
        Ok(())
    }

    /// Destroy an encoder instance
    pub fn destroy_encoder(
        &mut self,
        caller: CallerIdentity,
        encoder_id: EncoderId,
    ) -> Result<(), ImageError> {
        if !self.initialized {
            return Err(ImageError::NotInitialized);
        }

        let encoder = self.encoders.get(&encoder_id.0)
            .ok_or(ImageError::EncoderNotFound)?;

        if encoder.owner_pid != caller.pid {
            return Err(ImageError::InvalidParameter);
        }

        self.encoders.remove(&encoder_id.0);
        log_debug!("Destroyed image encoder id={}", encoder_id.0);
        Ok(())
    }

    /// Get the acceleration path for a decoder
    pub fn get_accel_path(&self, decoder_id: DecoderId) -> Result<ImageAccelPath, ImageError> {
        let decoder = self.decoders.get(&decoder_id.0)
            .ok_or(ImageError::DecoderNotFound)?;
        Ok(decoder.accel_path)
    }

    /// Get image-specific statistics
    pub fn get_stats(&self) -> &ImageStats {
        &self.image_stats
    }

    /// Get a reference to the codec registry
    pub fn codec_registry(&self) -> &CodecRegistry {
        &self.codec_registry
    }

    /// Get a mutable reference to the codec registry
    pub fn codec_registry_mut(&mut self) -> &mut CodecRegistry {
        &mut self.codec_registry
    }
}

impl CoreProcessingService for ImageService {
    fn config(&self) -> &ServiceConfig {
        &self.config
    }

    fn init(&mut self) -> Result<ServiceNodeId, ServiceError> {
        if self.initialized {
            return Err(ServiceError::Busy);
        }

        log_info!("Initializing Image service (nuva.service.image)");

        self.initialized = true;

        // SAFETY: converting reference to u64 for use as a unique ID within
        // this process. The reference remains valid as long as the service exists.
        let node_id = self as *const Self as u64;

        log_info!("Image service initialized, node_id={}", node_id);
        Ok(node_id)
    }

    fn handle_request(
        &mut self,
        caller: CallerIdentity,
        request_id: u64,
        payload: &[u8],
    ) -> Result<(), ServiceError> {
        if !self.initialized {
            return Err(ServiceError::NotInitialized);
        }

        self.stats.record_request(0);
        log_debug!(
            "Image service request: caller=({},{}) req_id={} len={}",
            caller.pid,
            caller.uid,
            request_id,
            payload.len()
        );

        self.stats.complete_request();
        Ok(())
    }

    fn shutdown(&mut self) -> Result<(), ServiceError> {
        if !self.initialized {
            return Err(ServiceError::NotInitialized);
        }

        log_info!("Shutting down Image service");

        self.decoders.clear();
        self.encoders.clear();
        self.progressive_sessions.clear();

        self.initialized = false;
        Ok(())
    }

    fn health_check(&self) -> ServiceHealth {
        if !self.initialized {
            return ServiceHealth::NotInitialized;
        }
        let has_sw_fallback = self.decoders.values().any(|d| d.accel_path == ImageAccelPath::Software)
            || self.encoders.values().any(|e| e.accel_path == ImageAccelPath::Software);
        if has_sw_fallback {
            ServiceHealth::Degraded
        } else {
            ServiceHealth::Healthy
        }
    }

    fn stats(&self) -> &ServiceStats {
        &self.stats
    }
}
