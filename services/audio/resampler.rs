/*
 * Nuva OS - SystemService - Audio - Resampler
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

//! Sample rate conversion (resampling) for audio streams.
//! Supports Linear and Sinc interpolation quality modes.

use alloc::vec::Vec;

use super::error::{AudioError, AudioStreamInfo, ChannelLayout, PcmBuffer, SampleFormat};

/// Resample quality level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResampleQuality {
    /// Linear interpolation - fast, lower quality
    Linear = 0,
    /// Sinc interpolation - slower, high quality
    Sinc = 1,
}

/// Sample rate converter
pub struct Resampler {
    /// Source sample rate
    src_rate: u32,
    /// Target sample rate
    dst_rate: u32,
    /// Quality mode
    quality: ResampleQuality,
    /// Number of channels
    channels: u32,
    /// Current fractional position (fixed-point 16.16)
    position: u64,
    /// Sinc filter table (for Sinc quality)
    sinc_table: Vec<i16>,
}

/// Fixed-point fractional bits for resampler position
const FRAC_BITS: u32 = 16;
/// Fixed-point scale (1.0 in fixed-point)
const FRAC_SCALE: u64 = 1u64 << FRAC_BITS;

/// Sinc filter half-length (number of taps on each side)
const SINC_HALF_TAPS: usize = 16;

impl Resampler {
    /// Create a new resampler
    pub fn new(
        src_rate: u32,
        dst_rate: u32,
        channels: u32,
        quality: ResampleQuality,
    ) -> Result<Self, AudioError> {
        if src_rate == 0 || dst_rate == 0 {
            return Err(AudioError::InvalidParameter);
        }
        if channels == 0 || channels > 8 {
            return Err(AudioError::InvalidParameter);
        }

        let sinc_table = if quality == ResampleQuality::Sinc {
            Self::build_sinc_table(SINC_HALF_TAPS)
        } else {
            Vec::new()
        };

        Ok(Resampler {
            src_rate,
            dst_rate,
            quality,
            channels,
            position: 0,
            sinc_table,
        })
    }

    /// Build a windowed sinc filter table
    fn build_sinc_table(half_taps: usize) -> Vec<i16> {
        let table_size = half_taps * 4;
        let mut table = Vec::with_capacity(table_size);

        for i in 0..table_size {
            let x = (i as f64 / 4.0) - (half_taps as f64);
            let sinc_val = if x.abs() < 1e-10 {
                1.0
            } else {
                let pi_x = core::f64::consts::PI * x;
                (pi_x.sin() / pi_x) * Self::blackman_window(i, table_size)
            };
            let quantized = (sinc_val * 32767.0).round() as i16;
            table.push(quantized);
        }

        table
    }

    /// Blackman window function
    fn blackman_window(n: usize, size: usize) -> f64 {
        let w = 2.0 * core::f64::consts::PI * n as f64 / size as f64;
        0.42 - 0.5 * w.cos() + 0.08 * (2.0 * w).cos()
    }

    /// Resample a PCM buffer to the target sample rate
    pub fn resample(&mut self, input: &PcmBuffer) -> Result<PcmBuffer, AudioError> {
        if input.data.is_empty() {
            return Ok(PcmBuffer::new(AudioStreamInfo::new(
                self.dst_rate,
                input.info.sample_format,
                input.info.channel_layout,
            )));
        }

        if input.info.sample_rate != self.src_rate {
            return Err(AudioError::InvalidParameter);
        }

        let src_frames = input.frame_count as u64;
        let ratio = (self.src_rate as u64 * FRAC_SCALE) / self.dst_rate as u64;
        let dst_frames = ((src_frames * self.dst_rate as u64) / self.src_rate as u64) as usize;

        let dst_info = AudioStreamInfo::new(
            self.dst_rate,
            input.info.sample_format,
            input.info.channel_layout,
        );

        let frame_size = input.info.frame_size();
        let output_size = dst_frames * frame_size;
        let mut output_data = Vec::with_capacity(output_size);

        match self.quality {
            ResampleQuality::Linear => {
                self.resample_linear(
                    &input.data,
                    src_frames as usize,
                    ratio,
                    dst_frames,
                    frame_size,
                    &mut output_data,
                );
            }
            ResampleQuality::Sinc => {
                self.resample_sinc(
                    &input.data,
                    src_frames as usize,
                    ratio,
                    dst_frames,
                    frame_size,
                    &mut output_data,
                );
            }
        }

        self.position = 0;

        let mut buffer = PcmBuffer::from_data(output_data, dst_info);
        buffer.pts_us = input.pts_us;

        log_debug!(
            "Resample: {} frames @ {}Hz -> {} frames @ {}Hz, quality={:?}",
            src_frames,
            self.src_rate,
            buffer.frame_count,
            self.dst_rate,
            self.quality
        );

        Ok(buffer)
    }

    /// Linear interpolation resampling
    fn resample_linear(
        &mut self,
        input: &[u8],
        src_frames: usize,
        ratio: u64,
        dst_frames: usize,
        frame_size: usize,
        output: &mut Vec<u8>,
    ) {
        output.resize(dst_frames * frame_size, 0u8);

        let channels = self.channels as usize;
        let bytes_per_sample = if frame_size > 0 && channels > 0 {
            frame_size / channels
        } else {
            2
        };

        for dst_idx in 0..dst_frames {
            let pos = self.position + (dst_idx as u64) * ratio;
            let int_pos = (pos >> FRAC_BITS) as usize;
            let frac = (pos & (FRAC_SCALE - 1)) as u32;
            let frac_weight = frac as f64 / FRAC_SCALE as f64;

            let src_idx0 = int_pos.min(src_frames.saturating_sub(1));
            let src_idx1 = (int_pos + 1).min(src_frames.saturating_sub(1));

            let offset0 = src_idx0 * frame_size;
            let offset1 = src_idx1 * frame_size;
            let dst_offset = dst_idx * frame_size;

            if bytes_per_sample == 2 {
                for ch in 0..channels {
                    let s0 = Self::read_s16(input, offset0 + ch * 2);
                    let s1 = Self::read_s16(input, offset1 + ch * 2);
                    let interpolated = s0 as f64 * (1.0 - frac_weight) + s1 as f64 * frac_weight;
                    Self::write_s16(output, dst_offset + ch * 2, interpolated as i16);
                }
            } else {
                // Byte-level copy for non-16-bit formats
                for byte in 0..frame_size {
                    let b0 = input.get(offset0 + byte).copied().unwrap_or(0);
                    let b1 = input.get(offset1 + byte).copied().unwrap_or(0);
                    let interp = b0 as f64 * (1.0 - frac_weight) + b1 as f64 * frac_weight;
                    if dst_offset + byte < output.len() {
                        output[dst_offset + byte] = interp as u8;
                    }
                }
            }
        }
    }

    /// Sinc interpolation resampling
    fn resample_sinc(
        &mut self,
        input: &[u8],
        src_frames: usize,
        ratio: u64,
        dst_frames: usize,
        frame_size: usize,
        output: &mut Vec<u8>,
    ) {
        output.resize(dst_frames * frame_size, 0u8);

        let channels = self.channels as usize;
        let bytes_per_sample = if frame_size > 0 && channels > 0 {
            frame_size / channels
        } else {
            2
        };

        for dst_idx in 0..dst_frames {
            let pos = self.position + (dst_idx as u64) * ratio;
            let int_pos = (pos >> FRAC_BITS) as i64;
            let frac = (pos & (FRAC_SCALE - 1)) as f64 / FRAC_SCALE as f64;

            let dst_offset = dst_idx * frame_size;

            for ch in 0..channels {
                let mut sum: f64 = 0.0;

                for tap in -(SINC_HALF_TAPS as i64)..=(SINC_HALF_TAPS as i64) {
                    let src_idx = int_pos + tap;
                    if src_idx < 0 || src_idx >= src_frames as i64 {
                        continue;
                    }

                    let s = if bytes_per_sample == 2 {
                        Self::read_s16(input, src_idx as usize * frame_size + ch * 2) as f64
                    } else {
                        input.get(src_idx as usize * frame_size + ch).copied().unwrap_or(0) as f64
                    };

                    let filter_pos = (tap as f64 + frac).abs();
                    let sinc_val = if filter_pos < 1e-10 {
                        1.0
                    } else {
                        let pi_x = core::f64::consts::PI * filter_pos;
                        pi_x.sin() / pi_x
                    };

                    sum += s * sinc_val * Self::blackman_window_continuous(filter_pos);
                }

                if bytes_per_sample == 2 {
                    Self::write_s16(output, dst_offset + ch * 2, sum.round() as i16);
                } else if dst_offset + ch < output.len() {
                    output[dst_offset + ch] = sum.round().clamp(0.0, 255.0) as u8;
                }
            }
        }
    }

    /// Blackman window for continuous position
    fn blackman_window_continuous(x: f64) -> f64 {
        let half = SINC_HALF_TAPS as f64;
        if x > half {
            return 0.0;
        }
        let normalized = (x + half) / (2.0 * half);
        let w = 2.0 * core::f64::consts::PI * normalized;
        0.42 - 0.5 * w.cos() + 0.08 * (2.0 * w).cos()
    }

    /// Read a 16-bit signed little-endian sample
    fn read_s16(data: &[u8], offset: usize) -> i16 {
        if offset + 1 < data.len() {
            // SAFETY: bounds checked above
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

    /// Get the source sample rate
    pub const fn src_rate(&self) -> u32 {
        self.src_rate
    }

    /// Get the destination sample rate
    pub const fn dst_rate(&self) -> u32 {
        self.dst_rate
    }

    /// Get the quality mode
    pub const fn quality(&self) -> ResampleQuality {
        self.quality
    }

    /// Reset the resampler state
    pub fn reset(&mut self) {
        self.position = 0;
    }
}
