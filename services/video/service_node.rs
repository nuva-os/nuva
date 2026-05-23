/*
 * Nuva OS - SystemService - Video - Service Node
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

//! Video service node implementing CoreProcessingService trait.
//! Registered as "nuva.service.video" in the Nuva IPC framework.

use alloc::collections::BTreeMap;
use core::sync::atomic::{AtomicU64, Ordering};

use crate::services::core_processing::service_node::{
    CallerIdentity, CoreProcessingService, ServiceConfig, ServiceHealth, ServiceNodeId,
    ServiceStats, ServiceVersion,
};
use crate::services::core_processing::error::ServiceError;

use super::codec::CodecRegistry;
use super::error::{
    DecoderId, EncoderId, VideoDecodeConfig, VideoEncodeConfig, VideoError, VideoFormat,
    VideoPacket,
};
use super::format_detect::{detect_video_format, VideoDetectResult};
use super::frame_buffer::{DecodeResult, FrameBufferPool};
use super::hw_accel::HwVideoCodec;
use super::power::VideoPowerManager;
use super::sw_fallback::SwFallback;

/// Convert VideoError to ServiceError
impl From<VideoError> for ServiceError {
    fn from(e: VideoError) -> ServiceError {
        match e {
            VideoError::NotInitialized => ServiceError::NotInitialized,
            VideoError::OutOfMemory => ServiceError::OutOfMemory,
            VideoError::HardwareError => ServiceError::Specific(
                crate::services::core_processing::error::ServiceSpecificError::VideoHardwareError,
            ),
            VideoError::FormatNotSupported => ServiceError::Specific(
                crate::services::core_processing::error::ServiceSpecificError::VideoFormatNotSupported,
            ),
            VideoError::DataCorrupted => ServiceError::Specific(
                crate::services::core_processing::error::ServiceSpecificError::VideoDataCorrupted,
            ),
            VideoError::InvalidParameter => ServiceError::Specific(
                crate::services::core_processing::error::ServiceSpecificError::VideoInvalidParameter,
            ),
            _ => ServiceError::Specific(
                crate::services::core_processing::error::ServiceSpecificError::VideoHardwareError,
            ),
        }
    }
}

/// Acceleration path for codec operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoAccelPath {
    /// Hardware acceleration (GPU/NPU)
    Hardware = 0,
    /// Software fallback
    Software = 1,
}

/// Decoder instance state
struct DecoderInstance {
    /// Decoder ID
    id: DecoderId,
    /// Decode configuration
    config: VideoDecodeConfig,
    /// Owner PID
    owner_pid: u32,
    /// Acceleration path
    accel_path: VideoAccelPath,
}

/// Encoder instance state
struct EncoderInstance {
    /// Encoder ID
    id: EncoderId,
    /// Encode configuration
    config: VideoEncodeConfig,
    /// Owner PID
    owner_pid: u32,
    /// Acceleration path
    accel_path: VideoAccelPath,
}

/// Video service statistics
#[derive(Debug)]
pub struct VideoStats {
    /// Total decoders created
    pub total_decoders: AtomicU64,
    /// Total encoders created
    pub total_encoders: AtomicU64,
    /// Total frames decoded
    pub total_frames_decoded: AtomicU64,
    /// Total frames encoded
    pub total_frames_encoded: AtomicU64,
    /// Software fallback count
    pub fallback_count: AtomicU64,
}

impl VideoStats {
    /// Create zero-initialized stats
    pub const fn new() -> Self {
        VideoStats {
            total_decoders: AtomicU64::new(0),
            total_encoders: AtomicU64::new(0),
            total_frames_decoded: AtomicU64::new(0),
            total_frames_encoded: AtomicU64::new(0),
            fallback_count: AtomicU64::new(0),
        }
    }
}

/// Video service
pub struct VideoService {
    /// Service configuration
    config: ServiceConfig,
    /// Core service statistics
    stats: ServiceStats,
    /// Video-specific statistics
    video_stats: VideoStats,
    /// Codec registry
    codec_registry: CodecRegistry,
    /// Software fallback
    sw_fallback: SwFallback,
    /// Frame buffer pool
    frame_pool: FrameBufferPool,
    /// Power manager
    power_mgr: VideoPowerManager,
    /// Active decoder instances
    decoders: BTreeMap<u64, DecoderInstance>,
    /// Active encoder instances
    encoders: BTreeMap<u64, EncoderInstance>,
    /// Next decoder ID
    next_decoder_id: AtomicU64,
    /// Next encoder ID
    next_encoder_id: AtomicU64,
    /// Whether the service is initialized
    initialized: bool,
}

/// Default idle timeout for video codec: 5 seconds in microseconds
const DEFAULT_IDLE_TIMEOUT_US: u64 = 5_000_000;

impl VideoService {
    /// Create a new video service instance
    pub fn new() -> Self {
        let config = ServiceConfig {
            name: "nuva.service.video",
            version: ServiceVersion::new(1, 0, 0),
            max_concurrent_requests: 32,
            request_timeout_us: 33_333,
            hw_accel_available: true,
        };

        VideoService {
            config,
            stats: ServiceStats::new(),
            video_stats: VideoStats::new(),
            codec_registry: CodecRegistry::new(),
            sw_fallback: SwFallback::new(),
            frame_pool: FrameBufferPool::new(),
            power_mgr: VideoPowerManager::new(DEFAULT_IDLE_TIMEOUT_US),
            decoders: BTreeMap::new(),
            encoders: BTreeMap::new(),
            next_decoder_id: AtomicU64::new(1),
            next_encoder_id: AtomicU64::new(1),
            initialized: false,
        }
    }

    /// Create a decoder instance for the given format
    pub fn create_decoder(
        &mut self,
        caller: CallerIdentity,
        config: VideoDecodeConfig,
    ) -> Result<DecoderId, VideoError> {
        if !self.initialized {
            return Err(VideoError::NotInitialized);
        }

        let has_hw = self.codec_registry.select_hardware(config.format).is_some();
        let has_sw = self.codec_registry.select_software(config.format).is_some();

        if !has_hw && !has_sw && !self.sw_fallback.has_fallback(config.format) {
            return Err(VideoError::FormatNotSupported);
        }

        let accel_path = if config.hw_accel && has_hw {
            VideoAccelPath::Hardware
        } else {
            VideoAccelPath::Software
        };

        let decoder_id = DecoderId(self.next_decoder_id.fetch_add(1, Ordering::Relaxed));

        self.decoders.insert(decoder_id.0, DecoderInstance {
            id: decoder_id,
            config,
            owner_pid: caller.pid,
            accel_path,
        });

        self.video_stats.total_decoders.fetch_add(1, Ordering::Relaxed);
        self.power_mgr.decoder_created();

        log_debug!(
            "Created decoder id={} format={:?} accel={:?}",
            decoder_id.0,
            self.decoders.get(&decoder_id.0).map_or(VideoFormat::Unknown, |d| d.config.format),
            accel_path,
        );

        Ok(decoder_id)
    }

    /// Create an encoder instance for the given format
    pub fn create_encoder(
        &mut self,
        caller: CallerIdentity,
        config: VideoEncodeConfig,
    ) -> Result<EncoderId, VideoError> {
        if !self.initialized {
            return Err(VideoError::NotInitialized);
        }

        let has_hw = self.codec_registry.select_hardware(config.format).is_some();
        let has_sw = self.codec_registry.select_software(config.format).is_some();

        if !has_hw && !has_sw && !self.sw_fallback.has_fallback(config.format) {
            return Err(VideoError::FormatNotSupported);
        }

        let accel_path = if config.hw_accel && has_hw {
            VideoAccelPath::Hardware
        } else {
            VideoAccelPath::Software
        };

        let encoder_id = EncoderId(self.next_encoder_id.fetch_add(1, Ordering::Relaxed));

        self.encoders.insert(encoder_id.0, EncoderInstance {
            id: encoder_id,
            config,
            owner_pid: caller.pid,
            accel_path,
        });

        self.video_stats.total_encoders.fetch_add(1, Ordering::Relaxed);
        self.power_mgr.encoder_created();

        log_debug!(
            "Created encoder id={} format={:?} accel={:?}",
            encoder_id.0,
            self.encoders.get(&encoder_id.0).map_or(VideoFormat::Unknown, |e| e.config.format),
            accel_path,
        );

        Ok(encoder_id)
    }

    /// Decode a video packet using the specified decoder
    pub fn decode(
        &mut self,
        caller: CallerIdentity,
        decoder_id: DecoderId,
        packet: &VideoPacket,
    ) -> Result<DecodeResult, VideoError> {
        if !self.initialized {
            return Err(VideoError::NotInitialized);
        }

        let decoder = self.decoders.get(&decoder_id.0)
            .ok_or(VideoError::DecoderNotFound)?;

        if decoder.owner_pid != caller.pid {
            return Err(VideoError::InvalidParameter);
        }

        let format = decoder.config.format;
        let accel_path = decoder.accel_path;

        self.power_mgr.decode_started(0);

        let result = if accel_path == VideoAccelPath::Hardware {
            if let Some(codec) = self.codec_registry.select(format) {
                match codec.decode(packet) {
                    Ok(r) => Ok(r),
                    Err(_) => {
                        log_warn!("HW decode failed, falling back to SW for format={:?}", format);
                        self.video_stats.fallback_count.fetch_add(1, Ordering::Relaxed);
                        self.sw_fallback.decode(packet)
                    }
                }
            } else {
                self.sw_fallback.decode(packet)
            }
        } else {
            self.sw_fallback.decode(packet)
        };

        self.power_mgr.decode_completed();

        if let Ok(ref r) = result {
            self.video_stats.total_frames_decoded.fetch_add(
                r.frames.len() as u64,
                Ordering::Relaxed,
            );
        }

        result
    }

    /// Encode raw frame data using the specified encoder
    pub fn encode(
        &mut self,
        caller: CallerIdentity,
        encoder_id: EncoderId,
        frame_data: &[u8],
        stride: u32,
    ) -> Result<VideoPacket, VideoError> {
        if !self.initialized {
            return Err(VideoError::NotInitialized);
        }

        let encoder = self.encoders.get(&encoder_id.0)
            .ok_or(VideoError::EncoderNotFound)?;

        if encoder.owner_pid != caller.pid {
            return Err(VideoError::InvalidParameter);
        }

        let format = encoder.config.format;
        let width = encoder.config.width;
        let height = encoder.config.height;
        let accel_path = encoder.accel_path;

        self.power_mgr.encode_started(0);

        let result = if accel_path == VideoAccelPath::Hardware {
            if let Some(codec) = self.codec_registry.select(format) {
                match codec.encode(frame_data, width, height, stride) {
                    Ok(p) => Ok(p),
                    Err(_) => {
                        log_warn!("HW encode failed, falling back to SW for format={:?}", format);
                        self.video_stats.fallback_count.fetch_add(1, Ordering::Relaxed);
                        self.sw_fallback.encode(format, frame_data, width, height, stride)
                    }
                }
            } else {
                self.sw_fallback.encode(format, frame_data, width, height, stride)
            }
        } else {
            self.sw_fallback.encode(format, frame_data, width, height, stride)
        };

        self.power_mgr.encode_completed();

        if result.is_ok() {
            self.video_stats.total_frames_encoded.fetch_add(1, Ordering::Relaxed);
        }

        result
    }

    /// Detect video format from data
    pub fn detect_format(&self, data: &[u8]) -> Result<VideoDetectResult, VideoError> {
        detect_video_format(data)
    }

    /// Destroy a decoder instance
    pub fn destroy_decoder(
        &mut self,
        caller: CallerIdentity,
        decoder_id: DecoderId,
    ) -> Result<(), VideoError> {
        if !self.initialized {
            return Err(VideoError::NotInitialized);
        }

        let decoder = self.decoders.get(&decoder_id.0)
            .ok_or(VideoError::DecoderNotFound)?;

        if decoder.owner_pid != caller.pid {
            return Err(VideoError::InvalidParameter);
        }

        self.decoders.remove(&decoder_id.0);
        self.power_mgr.decoder_destroyed();

        log_debug!("Destroyed decoder id={}", decoder_id.0);
        Ok(())
    }

    /// Destroy an encoder instance
    pub fn destroy_encoder(
        &mut self,
        caller: CallerIdentity,
        encoder_id: EncoderId,
    ) -> Result<(), VideoError> {
        if !self.initialized {
            return Err(VideoError::NotInitialized);
        }

        let encoder = self.encoders.get(&encoder_id.0)
            .ok_or(VideoError::EncoderNotFound)?;

        if encoder.owner_pid != caller.pid {
            return Err(VideoError::InvalidParameter);
        }

        self.encoders.remove(&encoder_id.0);
        self.power_mgr.encoder_destroyed();

        log_debug!("Destroyed encoder id={}", encoder_id.0);
        Ok(())
    }

    /// Get the acceleration path for a decoder
    pub fn get_accel_path(&self, decoder_id: DecoderId) -> Result<VideoAccelPath, VideoError> {
        let decoder = self.decoders.get(&decoder_id.0)
            .ok_or(VideoError::DecoderNotFound)?;
        Ok(decoder.accel_path)
    }

    /// Get video-specific statistics
    pub fn get_stats(&self) -> &VideoStats {
        &self.video_stats
    }
}

impl CoreProcessingService for VideoService {
    fn config(&self) -> &ServiceConfig {
        &self.config
    }

    fn init(&mut self) -> Result<ServiceNodeId, ServiceError> {
        if self.initialized {
            return Err(ServiceError::Busy);
        }

        log_info!("Initializing Video service (nuva.service.video)");

        self.initialized = true;

        // SAFETY: converting reference to u64 for use as a unique ID within
        // this process. The reference remains valid as long as the service exists.
        let node_id = self as *const Self as u64;

        log_info!("Video service initialized, node_id={}", node_id);
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
            "Video service request: caller=({},{}) req_id={} len={}",
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

        log_info!("Shutting down Video service");

        self.decoders.clear();
        self.encoders.clear();

        self.initialized = false;
        Ok(())
    }

    fn health_check(&self) -> ServiceHealth {
        if !self.initialized {
            return ServiceHealth::NotInitialized;
        }
        let has_sw_fallback = self.decoders.values().any(|d| d.accel_path == VideoAccelPath::Software)
            || self.encoders.values().any(|e| e.accel_path == VideoAccelPath::Software);
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
