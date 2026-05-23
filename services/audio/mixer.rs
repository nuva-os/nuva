/*
 * Nuva OS - SystemService - Audio - Mixer
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

//! Multi-stream audio mixing engine.
//! Mixes multiple PCM audio streams with per-stream gain control.
//! Output uses hard clipping to prevent overflow.

use alloc::vec::Vec;

use super::error::{AudioError, AudioStreamInfo, ChannelLayout, PcmBuffer, SampleFormat};

/// Maximum number of streams that can be mixed simultaneously
pub const MAX_MIXER_STREAMS: usize = 32;

/// A single stream input to the mixer
#[derive(Debug, Clone)]
pub struct MixerStream {
    /// PCM buffer for this stream
    pub buffer: PcmBuffer,
    /// Gain factor for this stream (1.0 = unity, 0.0 = silent)
    pub gain: f32,
}

impl MixerStream {
    /// Create a new mixer stream from a PCM buffer with unity gain
    pub fn new(buffer: PcmBuffer) -> Self {
        MixerStream { buffer, gain: 1.0 }
    }

    /// Create a new mixer stream with a specific gain
    pub fn with_gain(buffer: PcmBuffer, gain: f32) -> Self {
        MixerStream { buffer, gain }
    }
}

/// Audio mixer engine
pub struct AudioMixer {
    /// Output stream info
    output_info: AudioStreamInfo,
    /// Maximum number of streams
    max_streams: usize,
}

impl AudioMixer {
    /// Create a new audio mixer with the given output format
    pub fn new(output_info: AudioStreamInfo) -> Self {
        AudioMixer {
            output_info,
            max_streams: MAX_MIXER_STREAMS,
        }
    }

    /// Create a new audio mixer with custom max streams
    pub fn with_max_streams(output_info: AudioStreamInfo, max_streams: usize) -> Self {
        AudioMixer {
            output_info,
            max_streams: if max_streams == 0 { 1 } else { max_streams },
        }
    }

    /// Mix multiple audio streams into a single output buffer.
    /// Applies per-stream gain and hard clipping to prevent overflow.
    pub fn mix(&self, streams: &[MixerStream]) -> Result<PcmBuffer, AudioError> {
        if streams.is_empty() {
            return Ok(PcmBuffer::new(self.output_info));
        }

        if streams.len() > self.max_streams {
            return Err(AudioError::MixerOverflow);
        }

        let frame_size = self.output_info.frame_size();
        let bytes_per_sample = self.output_info.sample_format.bytes_per_sample();
        let channels = self.output_info.channel_layout.channel_count() as usize;

        // Determine output frame count (use the longest stream)
        let max_frames = streams
            .iter()
            .map(|s| s.buffer.frame_count as usize)
            .max()
            .unwrap_or(0);

        if max_frames == 0 {
            return Ok(PcmBuffer::new(self.output_info));
        }

        let output_size = max_frames * frame_size;
        let mut output_data = Vec::with_capacity(output_size);
        output_data.resize(output_size, 0u8);

        match bytes_per_sample {
            2 => self.mix_s16(streams, max_frames, channels, &mut output_data),
            4 => self.mix_f32(streams, max_frames, channels, &mut output_data),
            _ => self.mix_generic(streams, max_frames, frame_size, &mut output_data),
        }

        let mut result = PcmBuffer::from_data(output_data, self.output_info);
        if let Some(first) = streams.first() {
            result.pts_us = first.buffer.pts_us;
        }

        log_debug!(
            "Mixer: {} streams -> {} frames, {} channels",
            streams.len(),
            result.frame_count,
            channels
        );

        Ok(result)
    }

    /// Mix for 16-bit signed samples
    fn mix_s16(
        &self,
        streams: &[MixerStream],
        max_frames: usize,
        channels: usize,
        output: &mut [u8],
    ) {
        for frame in 0..max_frames {
            for ch in 0..channels {
                let mut mixed: f64 = 0.0;

                for stream in streams {
                    if frame >= stream.buffer.frame_count as usize {
                        continue;
                    }

                    let offset = frame * channels * 2 + ch * 2;
                    let sample = Self::read_s16(&stream.buffer.data, offset);
                    mixed += sample as f64 * stream.gain as f64;
                }

                // Hard clipping to i16 range
                let clipped = mixed.clamp(i16::MIN as f64, i16::MAX as f64) as i16;
                let out_offset = frame * channels * 2 + ch * 2;
                Self::write_s16(output, out_offset, clipped);
            }
        }
    }

    /// Mix for 32-bit float samples
    fn mix_f32(
        &self,
        streams: &[MixerStream],
        max_frames: usize,
        channels: usize,
        output: &mut [u8],
    ) {
        for frame in 0..max_frames {
            for ch in 0..channels {
                let mut mixed: f64 = 0.0;

                for stream in streams {
                    if frame >= stream.buffer.frame_count as usize {
                        continue;
                    }

                    let offset = frame * channels * 4 + ch * 4;
                    let sample = Self::read_f32(&stream.buffer.data, offset);
                    mixed += sample as f64 * stream.gain as f64;
                }

                // Clip to [-1.0, 1.0] for float output
                let clipped = mixed.clamp(-1.0, 1.0);
                let out_offset = frame * channels * 4 + ch * 4;
                Self::write_f32(output, out_offset, clipped as f32);
            }
        }
    }

    /// Generic byte-level mixing (fallback)
    fn mix_generic(
        &self,
        streams: &[MixerStream],
        max_frames: usize,
        frame_size: usize,
        output: &mut [u8],
    ) {
        for frame in 0..max_frames {
            for byte in 0..frame_size {
                let mut mixed: f64 = 0.0;
                let mut count = 0usize;

                for stream in streams {
                    if frame >= stream.buffer.frame_count as usize {
                        continue;
                    }

                    let offset = frame * frame_size + byte;
                    let sample = stream.buffer.data.get(offset).copied().unwrap_or(0);
                    mixed += sample as f64 * stream.gain as f64;
                    count += 1;
                }

                if count > 0 {
                    let clipped = mixed.clamp(0.0, 255.0) as u8;
                    let out_offset = frame * frame_size + byte;
                    if out_offset < output.len() {
                        output[out_offset] = clipped;
                    }
                }
            }
        }
    }

    /// Read a 16-bit signed little-endian sample
    fn read_s16(data: &[u8], offset: usize) -> i16 {
        if offset + 1 < data.len() {
            let lo = data[offset] as u16;
            let hi = data[offset + 1] as u16;
            (hi << 8 | lo) as i16
        } else {
            0
        }
    }

    /// Write a 16-bit signed little-endian sample
    fn write_s16(data: &mut [u8], offset: usize, value: i16) {
        if offset + 1 < data.len() {
            let v = value as u16;
            data[offset] = (v & 0xFF) as u8;
            data[offset + 1] = ((v >> 8) & 0xFF) as u8;
        }
    }

    /// Read a 32-bit float little-endian sample
    fn read_f32(data: &[u8], offset: usize) -> f32 {
        if offset + 3 < data.len() {
            let bytes = [
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ];
            // SAFETY: We have validated the offset bounds above
            f32::from_le_bytes(bytes)
        } else {
            0.0
        }
    }

    /// Write a 32-bit float little-endian sample
    fn write_f32(data: &mut [u8], offset: usize, value: f32) {
        if offset + 3 < data.len() {
            let bytes = value.to_le_bytes();
            data[offset] = bytes[0];
            data[offset + 1] = bytes[1];
            data[offset + 2] = bytes[2];
            data[offset + 3] = bytes[3];
        }
    }

    /// Get the output stream info
    pub const fn output_info(&self) -> &AudioStreamInfo {
        &self.output_info
    }

    /// Get the maximum number of streams
    pub const fn max_streams(&self) -> usize {
        self.max_streams
    }
}
