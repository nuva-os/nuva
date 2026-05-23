/*
 * Nuva OS - SystemService - Audio
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

//! Audio codec service for Nuva OS.
//! Provides multi-format audio decode/encode (AAC, Opus, FLAC, PCM),
//! sample rate conversion, multi-stream mixing with clipping,
//! per-stream volume control, and power coordination.

pub use crate::{pr_emerg, pr_alert, pr_crit, pr_err, pr_warn, pr_notice, pr_info, pr_debug};

pub mod service_node;
pub mod codec;
pub mod aac;
pub mod opus;
pub mod flac;
pub mod pcm;
pub mod resampler;
pub mod mixer;
pub mod volume;
pub mod power;
pub mod error;

pub use service_node::{AudioService, AudioStats};
pub use error::{
    AudioError, AudioFormat, AudioStreamInfo, PcmBuffer, AudioPacket,
    AudioDecodeConfig, AudioEncodeConfig, DecoderId, EncoderId,
    SampleFormat, ChannelLayout,
};
pub use codec::{AudioCodec, CodecRegistry};
pub use resampler::{Resampler, ResampleQuality};
pub use mixer::{AudioMixer, MixerStream, MAX_MIXER_STREAMS};
pub use volume::VolumeManager;
pub use power::AudioPowerManager;

/// Initialize the audio codec service
pub fn init_audio_service() {
    log_info!("Audio service module loaded");
    // The AudioService is instantiated and initialized by
    // the system services manager via CoreProcessingService::init()
}
