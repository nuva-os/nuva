/*
 * Nuva OS - Kernel - LED Device Class
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Standard interface for LED devices.
 */

use crate::pr_info;
use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// LED Type
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LedType {
    /// Unknown
    Unknown = 0,
    /// Indicator LED
    Indicator = 1,
    /// Keyboard backlight
    Keyboard = 2,
    /// Screen backlight
    Backlight = 3,
    /// Power LED
    Power = 4,
    /// Charging LED
    Charging = 5,
    /// Notification LED
    Notification = 6,
    /// Camera flash
    Flash = 7,
    /// RGB LED
    Rgb = 8,
    /// Status LED
    Status = 9,
    /// WiFi LED
    Wifi = 10,
    /// Bluetooth LED
    Bluetooth = 11,
    /// Ethernet LED
    Ethernet = 12,
    /// Disk activity LED
    Disk = 13,
    /// User LED
    User = 14,
}

/// LED Flags
bitflags::bitflags! {
    #[repr(transparent)]
    pub struct LedFlags: u32 {
        /// Brightness can be set
        const BRIGHTNESS = 1 << 0;
        /// Hardware accelerated blink
        const HW_BLINK = 1 << 1;
        /// Timer trigger available
        const TIMER = 1 << 2;
        /// Oneshot trigger available
        const ONESHOT = 1 << 3;
        /// Pattern trigger available
        const PATTERN = 1 << 4;
        /// Multi-color LED
        const MULTI_COLOR = 1 << 5;
        /// Inverted (active low)
        const INVERTED = 1 << 6;
        /// Keep on during suspend
        const KEEP_ON = 1 << 7;
        /// Auto brightness
        const AUTO_BRIGHTNESS = 1 << 8;
    }
}

/// LED State
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LedState {
    /// Off
    Off = 0,
    /// On
    On = 1,
    /// Blinking
    Blink = 2,
    /// Breathing
    Breath = 3,
}

/// LED Brightness
#[repr(C)]
pub struct LedBrightness {
    /// Current brightness (0-255)
    pub current: u8,
    /// Maximum brightness
    pub max: u8,
}

impl Default for LedBrightness {
    fn default() -> Self {
        LedBrightness {
            current: 0,
            max: 255,
        }
    }
}

/// LED Color (for RGB LEDs)
#[repr(C)]
pub struct LedColor {
    /// Red (0-255)
    pub r: u8,
    /// Green (0-255)
    pub g: u8,
    /// Blue (0-255)
    pub b: u8,
}

impl Default for LedColor {
    fn default() -> Self {
        LedColor { r: 0, g: 0, b: 0 }
    }
}

impl LedColor {
    /// Create from RGB values
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        LedColor { r, g, b }
    }

    /// Create white
    pub fn white() -> Self {
        LedColor {
            r: 255,
            g: 255,
            b: 255,
        }
    }

    /// Create red
    pub fn red() -> Self {
        LedColor { r: 255, g: 0, b: 0 }
    }

    /// Create green
    pub fn green() -> Self {
        LedColor { r: 0, g: 255, b: 0 }
    }

    /// Create blue
    pub fn blue() -> Self {
        LedColor { r: 0, g: 0, b: 255 }
    }

    /// Create yellow
    pub fn yellow() -> Self {
        LedColor {
            r: 255,
            g: 255,
            b: 0,
        }
    }

    /// Create cyan
    pub fn cyan() -> Self {
        LedColor {
            r: 0,
            g: 255,
            b: 255,
        }
    }

    /// Create magenta
    pub fn magenta() -> Self {
        LedColor {
            r: 255,
            g: 0,
            b: 255,
        }
    }
}

/// LED Blink Parameters
#[repr(C)]
pub struct LedBlink {
    /// Delay on (ms)
    pub delay_on: u32,
    /// Delay off (ms)
    pub delay_off: u32,
}

impl Default for LedBlink {
    fn default() -> Self {
        LedBlink {
            delay_on: 500,
            delay_off: 500,
        }
    }
}

/// LED Pattern Step
#[repr(C)]
pub struct LedPatternStep {
    /// Brightness
    pub brightness: u8,
    /// Duration (ms)
    pub duration_ms: u32,
}

/// LED Pattern
#[repr(C)]
pub struct LedPattern {
    /// Steps
    pub steps: [LedPatternStep; 16],
    /// Number of steps
    pub num_steps: u8,
    /// Repeat count (0 = infinite)
    pub repeat: u8,
}

/// LED Device Info
#[repr(C)]
pub struct LedInfo {
    /// LED name
    pub name: [u8; 32],
    /// LED type
    pub led_type: LedType,
    /// Flags
    pub flags: LedFlags,
    /// Brightness info
    pub brightness: LedBrightness,
    /// Current state
    pub state: LedState,
    /// Color (for RGB LEDs)
    pub color: LedColor,
    /// Blink parameters
    pub blink: LedBlink,
}

/// LED Device Operations
pub struct LedDeviceOps {
    /// Set brightness
    pub set_brightness: Option<unsafe extern "C" fn(*mut core::ffi::c_void, u8) -> i32>,
    /// Get brightness
    pub get_brightness: Option<unsafe extern "C" fn(*const core::ffi::c_void) -> u8>,
    /// Set color (RGB)
    pub set_color: Option<unsafe extern "C" fn(*mut core::ffi::c_void, u8, u8, u8) -> i32>,
    /// Get color (RGB)
    pub get_color:
        Option<unsafe extern "C" fn(*const core::ffi::c_void, *mut u8, *mut u8, *mut u8)>,
    /// Set blink
    pub set_blink: Option<unsafe extern "C" fn(*mut core::ffi::c_void, u32, u32) -> i32>,
    /// Set pattern
    pub set_pattern: Option<unsafe extern "C" fn(*mut core::ffi::c_void, *const LedPattern) -> i32>,
    /// Turn on
    pub on: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    /// Turn off
    pub off: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    /// Toggle
    pub toggle: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
}

/// LED ioctl commands
pub mod led_ioctl {
    /// Set brightness
    pub const SET_BRIGHTNESS: u32 = 0xF001;
    /// Get brightness
    pub const GET_BRIGHTNESS: u32 = 0xF002;
    /// Set color
    pub const SET_COLOR: u32 = 0xF003;
    /// Get color
    pub const GET_COLOR: u32 = 0xF004;
    /// Set blink
    pub const SET_BLINK: u32 = 0xF005;
    /// Set pattern
    pub const SET_PATTERN: u32 = 0xF006;
    /// Turn on
    pub const ON: u32 = 0xF007;
    /// Turn off
    pub const OFF: u32 = 0xF008;
    /// Toggle
    pub const TOGGLE: u32 = 0xF009;
    /// Get info
    pub const GET_INFO: u32 = 0xF00A;
}

/// LED Manager
pub struct LedManager {
    /// LED count
    led_count: AtomicU32,
    /// Statistics
    stats: LedStats,
}

/// LED Statistics
pub struct LedStats {
    /// Set count
    pub set_count: AtomicU64,
    /// Blink count
    pub blink_count: AtomicU64,
}

impl LedStats {
    pub const fn new() -> Self {
        LedStats {
            set_count: AtomicU64::new(0),
            blink_count: AtomicU64::new(0),
        }
    }
}

impl LedManager {
    pub const fn new() -> Self {
        LedManager {
            led_count: AtomicU32::new(0),
            stats: LedStats::new(),
        }
    }

    /// Initialize
    pub fn init(&self) {
        log_info!("LED manager initialized");
    }

    /// Register LED
    pub fn register_led(&mut self) -> u32 {
        self.led_count.fetch_add(1, Ordering::AcqRel)
    }

    /// Get LED count
    pub fn get_led_count(&self) -> u32 {
        self.led_count.load(Ordering::Acquire)
    }
}

/// Global LED manager
static LED_MANAGER: core::sync::OnceLock<LedManager> = core::sync::OnceLock::new();

/// Get LED manager
pub fn led_manager() -> &'static LedManager {
    LED_MANAGER.get_or_init(LedManager::new)
}

pub fn init_led_manager() -> &'static LedManager {
    LED_MANAGER.get_or_init(LedManager::new)
}

/// Initialize LED manager
pub fn init_led_manager() {
    let mgr = led_manager();
    mgr.init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_led_type() {
        assert_eq!(LedType::Backlight as i32, 3);
        assert_eq!(LedType::Rgb as i32, 8);
    }

    #[test]
    fn test_led_color() {
        let red = LedColor::red();
        assert_eq!(red.r, 255);
        assert_eq!(red.g, 0);
        assert_eq!(red.b, 0);

        let white = LedColor::white();
        assert_eq!(white.r, 255);
        assert_eq!(white.g, 255);
        assert_eq!(white.b, 255);
    }

    #[test]
    fn test_led_flags() {
        let flags = LedFlags::BRIGHTNESS | LedFlags::HW_BLINK;
        assert!(flags.contains(LedFlags::BRIGHTNESS));
        assert!(flags.contains(LedFlags::HW_BLINK));
    }
}
