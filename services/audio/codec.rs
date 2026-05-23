/*
 * Nuva OS - SystemService - Audio - Codec Registry
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

//! Audio codec trait and registry for managing available codecs.
//! The registry prefers hardware codecs over software implementations.

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use super::error::{AudioError, AudioFormat, AudioPacket, PcmBuffer};

/// Audio codec trait - implemented by all codec backends
pub trait AudioCodec: Send + Sync {
    /// Get the audio format this codec handles
    fn format(&self) -> AudioFormat;

    /// Decode an encoded audio packet into PCM samples
    fn decode(&self, packet: &AudioPacket) -> Result<PcmBuffer, AudioError>;

    /// Encode PCM samples into an encoded audio packet
    fn encode(&self, pcm: &PcmBuffer) -> Result<AudioPacket, AudioError>;

    /// Whether this is a hardware-accelerated codec
    fn is_hardware(&self) -> bool;

    /// Get codec name for logging
    fn name(&self) -> &'static str;
}

/// Codec entry in the registry
struct CodecEntry {
    /// Codec priority (lower = higher priority)
    priority: u32,
    /// Whether this is a hardware codec
    is_hw: bool,
}

/// Codec registry - manages available audio codecs
pub struct CodecRegistry {
    /// Registered codecs indexed by format then by registration order
    codecs: BTreeMap<AudioFormat, Vec<&'static dyn AudioCodec>>,
    /// Codec metadata for priority tracking
    entries: BTreeMap<AudioFormat, Vec<CodecEntry>>,
}

impl CodecRegistry {
    /// Create a new empty codec registry
    pub fn new() -> Self {
        CodecRegistry {
            codecs: BTreeMap::new(),
            entries: BTreeMap::new(),
        }
    }

    /// Register a codec with the registry
    pub fn register(&mut self, codec: &'static dyn AudioCodec) {
        let format = codec.format();
        let is_hw = codec.is_hardware();
        let priority = if is_hw { 0 } else { 100 };

        let codec_list = self.codecs.entry(format).or_insert_with(Vec::new);
        let entry_list = self.entries.entry(format).or_insert_with(Vec::new);

        codec_list.push(codec);
        entry_list.push(CodecEntry { priority, is_hw });

        log_info!(
            "Registered audio codec: {} (format={:?}, hw={})",
            codec.name(),
            format,
            is_hw
        );
    }

    /// Select the best codec for a given format.
    /// Hardware codecs are preferred over software implementations.
    pub fn select(&self, format: AudioFormat) -> Option<&'static dyn AudioCodec> {
        let codec_list = self.codecs.get(&format)?;
        let entry_list = self.entries.get(&format)?;

        let mut best_idx: usize = 0;
        let mut best_priority: u32 = u32::MAX;

        for (i, entry) in entry_list.iter().enumerate() {
            if entry.priority < best_priority {
                best_priority = entry.priority;
                best_idx = i;
            }
        }

        if best_idx < codec_list.len() {
            Some(codec_list[best_idx])
        } else {
            codec_list.first().copied()
        }
    }

    /// Select a hardware codec for the given format
    pub fn select_hardware(&self, format: AudioFormat) -> Option<&'static dyn AudioCodec> {
        let codec_list = self.codecs.get(&format)?;
        let entry_list = self.entries.get(&format)?;

        for (i, entry) in entry_list.iter().enumerate() {
            if entry.is_hw && i < codec_list.len() {
                return Some(codec_list[i]);
            }
        }
        None
    }

    /// Select a software codec for the given format
    pub fn select_software(&self, format: AudioFormat) -> Option<&'static dyn AudioCodec> {
        let codec_list = self.codecs.get(&format)?;
        let entry_list = self.entries.get(&format)?;

        for (i, entry) in entry_list.iter().enumerate() {
            if !entry.is_hw && i < codec_list.len() {
                return Some(codec_list[i]);
            }
        }
        None
    }

    /// Check if a format is supported
    pub fn is_format_supported(&self, format: AudioFormat) -> bool {
        self.codecs.get(&format).map_or(false, |l| !l.is_empty())
    }

    /// Get all supported formats
    pub fn supported_formats(&self) -> Vec<AudioFormat> {
        self.codecs.keys().copied().collect()
    }
}
