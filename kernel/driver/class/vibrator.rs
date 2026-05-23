/*
 * Nuva OS - Kernel - Vibrator Device Class
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Standard interface for vibrator/haptic devices.
 */

use crate::pr_info;
use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// Vibrator State
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VibratorState {
    /// Off
    Off = 0,
    /// Vibrating
    On = 1,
    /// Playing pattern
    Pattern = 2,
}

/// Vibrator Capabilities
bitflags::bitflags! {
    #[repr(transparent)]
    pub struct VibratorCaps: u32 {
        /// Simple on/off
        const SIMPLE = 1 << 0;
        /// Variable intensity
        const INTENSITY = 1 << 1;
        /// Timed vibration
        const TIMED = 1 << 2;
        /// Pattern support
        const PATTERN = 1 << 3;
        /// Amplitude control
        const AMPLITUDE = 1 << 4;
        /// Frequency control
        const FREQUENCY = 1 << 5;
        /// Direction control
        const DIRECTION = 1 << 6;
    }
}

/// Vibrator Effect
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VibratorEffect {
    /// Single vibration
    Single = 0,
    /// Double click
    DoubleClick = 1,
    /// Tick
    Tick = 2,
    /// Heavy click
    HeavyClick = 3,
    /// Soft bump
    SoftBump = 4,
    /// Hard bump
    HardBump = 5,
    /// Success
    Success = 6,
    /// Failure
    Failure = 7,
    /// Warning
    Warning = 8,
    /// Keyboard press
    KeyPress = 9,
    /// Keyboard release
    KeyRelease = 10,
    /// Long press
    LongPress = 11,
    /// Virtual key
    VirtualKey = 12,
    /// Multi press
    MultiPress = 13,
    /// Custom
    Custom = 255,
}

/// Vibrator Pattern Step
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VibratorPatternStep {
    /// Amplitude (0-255)
    pub amplitude: u8,
    /// Duration (ms)
    pub duration_ms: u16,
}

/// Vibrator Pattern
#[repr(C)]
pub struct VibratorPattern {
    /// Pattern steps
    pub steps: [VibratorPatternStep; 32],
    /// Number of steps
    pub num_steps: u8,
    /// Repeat count (0 = infinite)
    pub repeat: u8,
}

impl VibratorPattern {
    /// Create a simple pattern
    pub fn simple(amplitude: u8, duration_ms: u16) -> Self {
        let mut pattern = VibratorPattern {
            steps: [VibratorPatternStep {
                amplitude: 0,
                duration_ms: 0,
            }; 32],
            num_steps: 1,
            repeat: 0,
        };
        pattern.steps[0] = VibratorPatternStep {
            amplitude,
            duration_ms,
        };
        pattern
    }

    /// Create a double click pattern
    pub fn double_click(amplitude: u8) -> Self {
        let mut pattern = VibratorPattern {
            steps: [VibratorPatternStep {
                amplitude: 0,
                duration_ms: 0,
            }; 32],
            num_steps: 4,
            repeat: 0,
        };
        pattern.steps[0] = VibratorPatternStep {
            amplitude,
            duration_ms: 50,
        };
        pattern.steps[1] = VibratorPatternStep {
            amplitude: 0,
            duration_ms: 50,
        };
        pattern.steps[2] = VibratorPatternStep {
            amplitude,
            duration_ms: 50,
        };
        pattern.steps[3] = VibratorPatternStep {
            amplitude: 0,
            duration_ms: 0,
        };
        pattern
    }
}

/// Vibrator Info
#[repr(C)]
pub struct VibratorInfo {
    /// Vibrator name
    pub name: [u8; 32],
    /// Capabilities
    pub caps: VibratorCaps,
    /// Maximum intensity (0-255)
    pub max_intensity: u8,
    /// Maximum duration (ms)
    pub max_duration_ms: u32,
    /// Minimum duration (ms)
    pub min_duration_ms: u16,
    /// Current state
    pub state: VibratorState,
    /// Number of effects supported
    pub num_effects: u8,
}

/// Vibrator Device Operations
pub struct VibratorDeviceOps {
    /// Initialize
    pub init: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    /// Deinitialize
    pub deinit: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,

    /// Vibrate with duration
    pub vibrate: Option<unsafe extern "C" fn(*mut core::ffi::c_void, u32) -> i32>,
    /// Vibrate with intensity and duration
    pub vibrate_intensity: Option<unsafe extern "C" fn(*mut core::ffi::c_void, u8, u32) -> i32>,
    /// Play effect
    pub play_effect: Option<unsafe extern "C" fn(*mut core::ffi::c_void, VibratorEffect) -> i32>,
    /// Play pattern
    pub play_pattern:
        Option<unsafe extern "C" fn(*mut core::ffi::c_void, *const VibratorPattern) -> i32>,

    /// Stop vibration
    pub stop: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    /// Pause
    pub pause: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    /// Resume
    pub resume: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,

    /// Get state
    pub get_state: Option<unsafe extern "C" fn(*const core::ffi::c_void) -> VibratorState>,
    /// Get info
    pub get_info: Option<unsafe extern "C" fn(*const core::ffi::c_void, *mut VibratorInfo) -> i32>,

    /// Set amplitude
    pub set_amplitude: Option<unsafe extern "C" fn(*mut core::ffi::c_void, u8) -> i32>,
    /// Set frequency
    pub set_frequency: Option<unsafe extern "C" fn(*mut core::ffi::c_void, u32) -> i32>,
}

/// Vibrator ioctl commands
pub mod vibrator_ioctl {
    /// Vibrate
    pub const VIBRATE: u32 = 0x1001;
    /// Vibrate with intensity
    pub const VIBRATE_INTENSITY: u32 = 0x1002;
    /// Play effect
    pub const PLAY_EFFECT: u32 = 0x1003;
    /// Play pattern
    pub const PLAY_PATTERN: u32 = 0x1004;
    /// Stop
    pub const STOP: u32 = 0x1005;
    /// Pause
    pub const PAUSE: u32 = 0x1006;
    /// Resume
    pub const RESUME: u32 = 0x1007;
    /// Get state
    pub const GET_STATE: u32 = 0x1008;
    /// Get info
    pub const GET_INFO: u32 = 0x1009;
    /// Set amplitude
    pub const SET_AMPLITUDE: u32 = 0x100A;
    /// Set frequency
    pub const SET_FREQUENCY: u32 = 0x100B;
}

/// Vibrator Manager
pub struct VibratorManager {
    /// Vibrator count
    vibrator_count: AtomicU32,
    /// Statistics
    stats: VibratorStats,
}

/// Vibrator Statistics
pub struct VibratorStats {
    /// Vibration count
    pub vibrate_count: AtomicU64,
    /// Total duration (ms)
    pub total_duration: AtomicU64,
    /// Stop count
    pub stop_count: AtomicU64,
}

impl VibratorStats {
    pub const fn new() -> Self {
        VibratorStats {
            vibrate_count: AtomicU64::new(0),
            total_duration: AtomicU64::new(0),
            stop_count: AtomicU64::new(0),
        }
    }
}

impl VibratorManager {
    pub const fn new() -> Self {
        VibratorManager {
            vibrator_count: AtomicU32::new(0),
            stats: VibratorStats::new(),
        }
    }

    /// Initialize
    pub fn init(&self) {
        log_info!("Vibrator manager initialized");
    }

    /// Register vibrator
    pub fn register_vibrator(&mut self) -> u32 {
        self.vibrator_count.fetch_add(1, Ordering::AcqRel)
    }

    /// Get vibrator count
    pub fn get_vibrator_count(&self) -> u32 {
        self.vibrator_count.load(Ordering::Acquire)
    }
}

/// Global vibrator manager
static VIBRATOR_MANAGER: core::sync::OnceLock<VibratorManager> = core::sync::OnceLock::new();

/// Get vibrator manager
pub fn vibrator_manager() -> &'static VibratorManager {
    VIBRATOR_MANAGER.get_or_init(VibratorManager::new)
}

pub fn init_vibrator_manager() -> &'static VibratorManager {
    VIBRATOR_MANAGER.get_or_init(VibratorManager::new)
}

/// Initialize vibrator manager
pub fn init_vibrator_manager() {
    let mgr = vibrator_manager();
    mgr.init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vibrator_state() {
        assert_eq!(VibratorState::Off as i32, 0);
        assert_eq!(VibratorState::On as i32, 1);
    }

    #[test]
    fn test_vibrator_effect() {
        assert_eq!(VibratorEffect::DoubleClick as i32, 1);
        assert_eq!(VibratorEffect::KeyPress as i32, 9);
    }

    #[test]
    fn test_vibrator_pattern() {
        let pattern = VibratorPattern::simple(128, 100);
        assert_eq!(pattern.num_steps, 1);
        assert_eq!(pattern.steps[0].amplitude, 128);
        assert_eq!(pattern.steps[0].duration_ms, 100);
    }

    #[test]
    fn test_vibrator_caps() {
        let caps = VibratorCaps::INTENSITY | VibratorCaps::TIMED;
        assert!(caps.contains(VibratorCaps::INTENSITY));
        assert!(caps.contains(VibratorCaps::TIMED));
    }
}
