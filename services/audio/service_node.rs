/*
 * Nuva OS - SystemService - Audio - Service Node
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

//! Audio service node implementing CoreProcessingService trait.
//! Registered as "nuva.service.audio" in the Nuva IPC framework.

use alloc::collections::BTreeMap;
use core::sync::atomic::{AtomicU64, Ordering};

use crate::services::core_processing::service_node::{
    CallerIdentity, CoreProcessingService, ServiceConfig, ServiceHealth, ServiceNodeId,
    ServiceStats, ServiceVersion,
};
use crate::services::core_processing::error::ServiceError;

use super::codec::CodecRegistry;
use super::error::{
    AudioDecodeConfig, AudioEncodeConfig, AudioError, AudioFormat, AudioPacket, DecoderId,
    EncoderId, PcmBuffer,
};
use super::mixer::{AudioMixer, MixerStream};
use super::power::AudioPowerManager;
use super::resampler::{ResampleQuality, Resampler};
use super::volume::VolumeManager;

/// Convert AudioError to ServiceError
impl From<AudioError> for ServiceError {
    fn from(e: AudioError) -> ServiceError {
        match e {
            AudioError::NotInitialized => ServiceError::NotInitialized,
            AudioError::OutOfMemory => ServiceError::OutOfMemory,
            AudioError::FormatNotSupported => ServiceError::Specific(
                crate::services::core_processing::error::ServiceSpecificError::AudioFormatNotSupported,
            ),
            AudioError::DataCorrupted => ServiceError::Specific(
                crate::services::core_processing::error::ServiceSpecificError::AudioDataCorrupted,
            ),
            AudioError::LatencyExceeded => ServiceError::Specific(
                crate::services::core_processing::error::ServiceSpecificError::AudioLatencyExceeded,
            ),
            AudioError::InvalidParameter => ServiceError::Specific(
                crate::services::core_processing::error::ServiceSpecificError::AudioInvalidParameter,
            ),
            _ => ServiceError::Specific(
                crate::services::core_processing::error::ServiceSpecificError::AudioFormatNotSupported,
            ),
        }
    }
}

/// Decoder instance state
struct DecoderInstance {
    /// Decoder ID
    id: DecoderId,
    /// Decode configuration
    config: AudioDecodeConfig,
    /// Owner PID
    owner_pid: u32,
}

/// Encoder instance state
struct EncoderInstance {
    /// Encoder ID
    id: EncoderId,
    /// Encode configuration
    config: AudioEncodeConfig,
    /// Owner PID
    owner_pid: u32,
}

/// Audio service statistics
#[derive(Debug)]
pub struct AudioStats {
    /// Total decoders created
    pub total_decoders: AtomicU64,
    /// Total encoders created
    pub total_encoders: AtomicU64,
    /// Total frames decoded
    pub total_frames_decoded: AtomicU64,
    /// Total frames encoded
    pub total_frames_encoded: AtomicU64,
    /// Total resample operations
    pub total_resamples: AtomicU64,
    /// Total mix operations
    pub total_mixes: AtomicU64,
}

impl AudioStats {
    /// Create zero-initialized stats
    pub const fn new() -> Self {
        AudioStats {
            total_decoders: AtomicU64::new(0),
            total_encoders: AtomicU64::new(0),
            total_frames_decoded: AtomicU64::new(0),
            total_frames_encoded: AtomicU64::new(0),
            total_resamples: AtomicU64::new(0),
            total_mixes: AtomicU64::new(0),
        }
    }
}

/// Audio service
pub struct AudioService {
    /// Service configuration
    config: ServiceConfig,
    /// Core service statistics
    stats: ServiceStats,
    /// Audio-specific statistics
    audio_stats: AudioStats,
    /// Codec registry
    codec_registry: CodecRegistry,
    /// Volume manager
    volume_mgr: VolumeManager,
    /// Power manager
    power_mgr: AudioPowerManager,
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

/// Default idle timeout for audio codec: 3 seconds in microseconds
const DEFAULT_IDLE_TIMEOUT_US: u64 = 3_000_000;

impl AudioService {
    /// Create a new audio service instance
    pub fn new() -> Self {
        let config = ServiceConfig {
            name: "nuva.service.audio",
            version: ServiceVersion::new(1, 0, 0),
            max_concurrent_requests: 64,
            request_timeout_us: 10_000,
            hw_accel_available: true,
        };

        AudioService {
            config,
            stats: ServiceStats::new(),
            audio_stats: AudioStats::new(),
            codec_registry: CodecRegistry::new(),
            volume_mgr: VolumeManager::new(),
            power_mgr: AudioPowerManager::new(DEFAULT_IDLE_TIMEOUT_US),
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
        config: AudioDecodeConfig,
    ) -> Result<DecoderId, AudioError> {
        if !self.initialized {
            return Err(AudioError::NotInitialized);
        }

        if !self.codec_registry.is_format_supported(config.format) {
            return Err(AudioError::FormatNotSupported);
        }

        let decoder_id = DecoderId(self.next_decoder_id.fetch_add(1, Ordering::Relaxed));

        self.decoders.insert(decoder_id.0, DecoderInstance {
            id: decoder_id,
            config,
            owner_pid: caller.pid,
        });

        self.audio_stats.total_decoders.fetch_add(1, Ordering::Relaxed);
        self.power_mgr.decoder_created();

        log_debug!(
            "Created audio decoder id={} format={:?}",
            decoder_id.0,
            self.decoders.get(&decoder_id.0).map_or(AudioFormat::Unknown, |d| d.config.format),
        );

        Ok(decoder_id)
    }

    /// Create an encoder instance for the given format
    pub fn create_encoder(
        &mut self,
        caller: CallerIdentity,
        config: AudioEncodeConfig,
    ) -> Result<EncoderId, AudioError> {
        if !self.initialized {
            return Err(AudioError::NotInitialized);
        }

        if !self.codec_registry.is_format_supported(config.format) {
            return Err(AudioError::FormatNotSupported);
        }

        let encoder_id = EncoderId(self.next_encoder_id.fetch_add(1, Ordering::Relaxed));

        self.encoders.insert(encoder_id.0, EncoderInstance {
            id: encoder_id,
            config,
            owner_pid: caller.pid,
        });

        self.audio_stats.total_encoders.fetch_add(1, Ordering::Relaxed);
        self.power_mgr.encoder_created();

        log_debug!(
            "Created audio encoder id={} format={:?}",
            encoder_id.0,
            self.encoders.get(&encoder_id.0).map_or(AudioFormat::Unknown, |e| e.config.format),
        );

        Ok(encoder_id)
    }

    /// Decode an audio packet using the specified decoder
    pub fn decode(
        &mut self,
        caller: CallerIdentity,
        decoder_id: DecoderId,
        packet: &AudioPacket,
    ) -> Result<PcmBuffer, AudioError> {
        if !self.initialized {
            return Err(AudioError::NotInitialized);
        }

        let decoder = self.decoders.get(&decoder_id.0)
            .ok_or(AudioError::DecoderNotFound)?;

        if decoder.owner_pid != caller.pid {
            return Err(AudioError::InvalidParameter);
        }

        let format = decoder.config.format;

        self.power_mgr.decode_started(0);

        let result = if let Some(codec) = self.codec_registry.select(format) {
            codec.decode(packet)
        } else {
            Err(AudioError::CodecNotFound)
        };

        self.power_mgr.decode_completed();

        if let Ok(ref buf) = result {
            self.audio_stats.total_frames_decoded.fetch_add(
                buf.frame_count as u64,
                Ordering::Relaxed,
            );
        }

        result
    }

    /// Encode PCM samples using the specified encoder
    pub fn encode(
        &mut self,
        caller: CallerIdentity,
        encoder_id: EncoderId,
        pcm: &PcmBuffer,
    ) -> Result<AudioPacket, AudioError> {
        if !self.initialized {
            return Err(AudioError::NotInitialized);
        }

        let encoder = self.encoders.get(&encoder_id.0)
            .ok_or(AudioError::EncoderNotFound)?;

        if encoder.owner_pid != caller.pid {
            return Err(AudioError::InvalidParameter);
        }

        let format = encoder.config.format;

        self.power_mgr.encode_started(0);

        let result = if let Some(codec) = self.codec_registry.select(format) {
            codec.encode(pcm)
        } else {
            Err(AudioError::CodecNotFound)
        };

        self.power_mgr.encode_completed();

        if result.is_ok() {
            self.audio_stats.total_frames_encoded.fetch_add(
                pcm.frame_count as u64,
                Ordering::Relaxed,
            );
        }

        result
    }

    /// Resample a PCM buffer to a target sample rate
    pub fn resample(
        &mut self,
        input: &PcmBuffer,
        dst_rate: u32,
        quality: ResampleQuality,
    ) -> Result<PcmBuffer, AudioError> {
        if !self.initialized {
            return Err(AudioError::NotInitialized);
        }

        if dst_rate == 0 {
            return Err(AudioError::InvalidParameter);
        }

        let channels = input.info.channel_layout.channel_count();
        let mut resampler = Resampler::new(
            input.info.sample_rate,
            dst_rate,
            channels,
            quality,
        )?;

        self.power_mgr.resampler_created();
        let result = resampler.resample(input);
        self.power_mgr.resampler_destroyed();

        if result.is_ok() {
            self.audio_stats.total_resamples.fetch_add(1, Ordering::Relaxed);
        }

        result
    }

    /// Mix multiple audio streams
    pub fn mix(&self, streams: &[MixerStream]) -> Result<PcmBuffer, AudioError> {
        if !self.initialized {
            return Err(AudioError::NotInitialized);
        }

        if streams.is_empty() {
            return Ok(PcmBuffer::new(super::error::AudioStreamInfo::new(
                48000,
                super::error::SampleFormat::S16Le,
                super::error::ChannelLayout::Stereo,
            )));
        }

        let output_info = streams[0].buffer.info;
        let mixer = AudioMixer::new(output_info);
        let result = mixer.mix(streams);

        if result.is_ok() {
            // Cannot update stats (mix is &self), but operation succeeded
        }

        result
    }

    /// Set volume for a stream
    pub fn set_volume(
        &self,
        stream_id: super::volume::VolumeStreamId,
        gain: f32,
    ) -> Result<(), AudioError> {
        self.volume_mgr.set_volume(stream_id, gain)
    }

    /// Destroy a decoder instance
    pub fn destroy_decoder(
        &mut self,
        caller: CallerIdentity,
        decoder_id: DecoderId,
    ) -> Result<(), AudioError> {
        if !self.initialized {
            return Err(AudioError::NotInitialized);
        }

        let decoder = self.decoders.get(&decoder_id.0)
            .ok_or(AudioError::DecoderNotFound)?;

        if decoder.owner_pid != caller.pid {
            return Err(AudioError::InvalidParameter);
        }

        self.decoders.remove(&decoder_id.0);
        self.power_mgr.decoder_destroyed();

        log_debug!("Destroyed audio decoder id={}", decoder_id.0);
        Ok(())
    }

    /// Destroy an encoder instance
    pub fn destroy_encoder(
        &mut self,
        caller: CallerIdentity,
        encoder_id: EncoderId,
    ) -> Result<(), AudioError> {
        if !self.initialized {
            return Err(AudioError::NotInitialized);
        }

        let encoder = self.encoders.get(&encoder_id.0)
            .ok_or(AudioError::EncoderNotFound)?;

        if encoder.owner_pid != caller.pid {
            return Err(AudioError::InvalidParameter);
        }

        self.encoders.remove(&encoder_id.0);
        self.power_mgr.encoder_destroyed();

        log_debug!("Destroyed audio encoder id={}", encoder_id.0);
        Ok(())
    }

    /// Get audio-specific statistics
    pub fn get_stats(&self) -> &AudioStats {
        &self.audio_stats
    }

    /// Get a reference to the volume manager
    pub fn volume_manager(&self) -> &VolumeManager {
        &self.volume_mgr
    }

    /// Get a mutable reference to the volume manager
    pub fn volume_manager_mut(&mut self) -> &mut VolumeManager {
        &mut self.volume_mgr
    }

    /// Get a reference to the codec registry
    pub fn codec_registry(&self) -> &CodecRegistry {
        &self.codec_registry
    }

    /// Get a mutable reference to the codec registry
    pub fn codec_registry_mut(&mut self) -> &CodecRegistry {
        &mut self.codec_registry
    }
}

impl CoreProcessingService for AudioService {
    fn config(&self) -> &ServiceConfig {
        &self.config
    }

    fn init(&mut self) -> Result<ServiceNodeId, ServiceError> {
        if self.initialized {
            return Err(ServiceError::Busy);
        }

        log_info!("Initializing Audio service (nuva.service.audio)");

        self.initialized = true;

        // SAFETY: converting reference to u64 for use as a unique ID within
        // this process. The reference remains valid as long as the service exists.
        let node_id = self as *const Self as u64;

        log_info!("Audio service initialized, node_id={}", node_id);
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
            "Audio service request: caller=({},{}) req_id={} len={}",
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

        log_info!("Shutting down Audio service");

        self.decoders.clear();
        self.encoders.clear();

        self.initialized = false;
        Ok(())
    }

    fn health_check(&self) -> ServiceHealth {
        if !self.initialized {
            return ServiceHealth::NotInitialized;
        }
        ServiceHealth::Healthy
    }

    fn stats(&self) -> &ServiceStats {
        &self.stats
    }
}
