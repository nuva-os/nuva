/*
 * Nuva OS - Kernel - Audio Device Class
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Standard interface for audio input/output devices.
 */

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// Audio Format
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioFormat {
    /// 8-bit unsigned PCM
    U8 = 0,
    /// 16-bit signed PCM (little-endian)
    S16LE = 1,
    /// 16-bit signed PCM (big-endian)
    S16BE = 2,
    /// 24-bit signed PCM (little-endian)
    S24LE = 3,
    /// 32-bit signed PCM (little-endian)
    S32LE = 4,
    /// 32-bit float
    Float = 5,
    /// Compressed (codec specific)
    Compressed = 6,
}

/// Audio Stream Direction
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioDirection {
    /// Playback
    Playback = 0,
    /// Capture
    Capture = 1,
}

/// Audio Stream Configuration
#[repr(C)]
pub struct AudioStreamConfig {
    /// Sample rate (Hz)
    pub sample_rate: u32,
    /// Number of channels
    pub channels: u8,
    /// Bits per sample
    pub bits_per_sample: u8,
    /// Audio format
    pub format: AudioFormat,
    /// Buffer size (frames)
    pub buffer_size: u32,
    /// Period size (frames)
    pub period_size: u32,
}

impl Default for AudioStreamConfig {
    fn default() -> Self {
        AudioStreamConfig {
            sample_rate: 48000,
            channels: 2,
            bits_per_sample: 16,
            format: AudioFormat::S16LE,
            buffer_size: 4096,
            period_size: 1024,
        }
    }
}

/// Audio Buffer
#[repr(C)]
pub struct AudioBuffer {
    /// Data pointer
    pub data: *mut u8,
    /// Size in bytes
    pub size: usize,
    /// Number of frames
    pub frames: u32,
    /// Timestamp
    pub timestamp: u64,
}

/// Audio Volume Control
#[repr(C)]
pub struct AudioVolume {
    /// Channel index (-1 for all channels)
    pub channel: i32,
    /// Volume value (0-100 or dB * 100)
    pub value: i32,
    /// Mute flag
    pub mute: bool,
}

/// Audio Device State
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioState {
    /// Closed
    Closed = 0,
    /// Open
    Open = 1,
    /// Prepared
    Prepared = 2,
    /// Running
    Running = 3,
    /// Paused
    Paused = 4,
    /// Draining
    Draining = 5,
}

/// Audio Device Statistics
pub struct AudioStats {
    /// Total bytes played/recorded
    pub total_bytes: AtomicU64,
    /// Buffer underruns
    pub underruns: AtomicU64,
    /// Buffer overruns
    pub overruns: AtomicU64,
    /// Sample rate adjustments
    pub rate_adjustments: AtomicU64,
}

impl AudioStats {
    pub const fn new() -> Self {
        AudioStats {
            total_bytes: AtomicU64::new(0),
            underruns: AtomicU64::new(0),
            overruns: AtomicU64::new(0),
            rate_adjustments: AtomicU64::new(0),
        }
    }
}

/// Audio Device Operations
pub struct AudioDeviceOps {
    /// Open stream
    pub open: Option<unsafe extern "C" fn(*mut core::ffi::c_void, AudioDirection) -> i32>,
    /// Close stream
    pub close: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    /// Prepare stream
    pub prepare: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    /// Start stream
    pub start: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    /// Stop stream
    pub stop: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    /// Pause stream
    pub pause: Option<unsafe extern "C" fn(*mut core::ffi::c_void, bool) -> i32>,
    /// Drain stream
    pub drain: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    /// Write audio data
    pub write: Option<unsafe extern "C" fn(*mut core::ffi::c_void, *const u8, usize) -> i32>,
    /// Read audio data
    pub read: Option<unsafe extern "C" fn(*mut core::ffi::c_void, *mut u8, usize) -> i32>,
    /// Set configuration
    pub set_config:
        Option<unsafe extern "C" fn(*mut core::ffi::c_void, *const AudioStreamConfig) -> i32>,
    /// Get configuration
    pub get_config:
        Option<unsafe extern "C" fn(*mut core::ffi::c_void, *mut AudioStreamConfig) -> i32>,
    /// Set volume
    pub set_volume: Option<unsafe extern "C" fn(*mut core::ffi::c_void, *const AudioVolume) -> i32>,
    /// Get volume
    pub get_volume: Option<unsafe extern "C" fn(*mut core::ffi::c_void, *mut AudioVolume) -> i32>,
}

/// Audio ioctl commands
pub mod audio_ioctl {
    /// Set stream configuration
    pub const SET_CONFIG: u32 = 0x9001;
    /// Get stream configuration
    pub const GET_CONFIG: u32 = 0x9002;
    /// Set volume
    pub const SET_VOLUME: u32 = 0x9003;
    /// Get volume
    pub const GET_VOLUME: u32 = 0x9004;
    /// Set mute
    pub const SET_MUTE: u32 = 0x9005;
    /// Get mute
    pub const GET_MUTE: u32 = 0x9006;
    /// Start stream
    pub const START: u32 = 0x9007;
    /// Stop stream
    pub const STOP: u32 = 0x9008;
    /// Pause stream
    pub const PAUSE: u32 = 0x9009;
    /// Resume stream
    pub const RESUME: u32 = 0x900A;
    /// Drain stream
    pub const DRAIN: u32 = 0x900B;
    /// Get delay (frames)
    pub const GET_DELAY: u32 = 0x900C;
    /// Get available space
    pub const GET_AVAIL: u32 = 0x900D;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_format_values() {
        assert_eq!(AudioFormat::U8 as i32, 0);
        assert_eq!(AudioFormat::S16LE as i32, 1);
        assert_eq!(AudioFormat::Float as i32, 5);
    }

    #[test]
    fn test_audio_direction_values() {
        assert_eq!(AudioDirection::Playback as i32, 0);
        assert_eq!(AudioDirection::Capture as i32, 1);
    }

    #[test]
    fn test_audio_stream_config_default() {
        let config = AudioStreamConfig::default();
        assert_eq!(config.sample_rate, 48000);
        assert_eq!(config.channels, 2);
        assert_eq!(config.bits_per_sample, 16);
    }

    #[test]
    fn test_audio_state_values() {
        assert_eq!(AudioState::Closed as i32, 0);
        assert_eq!(AudioState::Running as i32, 3);
    }
}
