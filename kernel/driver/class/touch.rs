/*
 * Nuva OS - Kernel - Driver - Class - Touch
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
/*
 * Nuva OS - Kernel - Touch Device Class
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Standard interface for touch screen devices.
 */

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// Touch Point
/// Represents a single touch point on the screen.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct TouchPoint {
    /// Touch ID (for multi-touch tracking)
    pub id: u32,
    /// X coordinate (pixels)
    pub x: u32,
    /// Y coordinate (pixels)
    pub y: u32,
    /// Pressure (0-255, 0 = released)
    pub pressure: u8,
    /// Touch major axis
    pub major: u16,
    /// Touch minor axis
    pub minor: u16,
    /// Orientation angle
    pub orientation: i16,
}

impl TouchPoint {
    pub const fn new() -> Self {
        TouchPoint {
            id: 0,
            x: 0,
            y: 0,
            pressure: 0,
            major: 0,
            minor: 0,
            orientation: 0,
        }
    }
}

/// Touch Event Type
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TouchEventType {
    /// Finger down
    Down = 0,
    /// Finger up
    Up = 1,
    /// Finger moved
    Move = 2,
    /// Multiple touch points changed
    Multi = 3,
    /// Frame sync
    Sync = 4,
}

/// Touch Event
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct TouchEvent {
    /// Event type
    pub event_type: TouchEventType,
    /// Timestamp (nanoseconds)
    pub timestamp: u64,
    /// Touch point
    pub point: TouchPoint,
    /// Number of active touches
    pub num_touches: u8,
    /// Reserved
    pub reserved: [u8; 3],
}

impl TouchEvent {
    pub fn new(event_type: TouchEventType, timestamp: u64, point: TouchPoint) -> Self {
        TouchEvent {
            event_type,
            timestamp,
            point,
            num_touches: 1,
            reserved: [0; 3],
        }
    }
}

/// Touch Device Configuration
#[repr(C)]
pub struct TouchConfig {
    /// Screen width (pixels)
    pub width: u32,
    /// Screen height (pixels)
    pub height: u32,
    /// Maximum number of simultaneous touches
    pub max_touches: u8,
    /// Sensitivity (0-100)
    pub sensitivity: u8,
    /// Pressure threshold
    pub pressure_threshold: u8,
    /// Swap X/Y axes
    pub swap_axes: bool,
    /// Invert X axis
    pub invert_x: bool,
    /// Invert Y axis
    pub invert_y: bool,
}

impl Default for TouchConfig {
    fn default() -> Self {
        TouchConfig {
            width: 1080,
            height: 1920,
            max_touches: 10,
            sensitivity: 50,
            pressure_threshold: 10,
            swap_axes: false,
            invert_x: false,
            invert_y: false,
        }
    }
}

/// Touch Device Statistics
pub struct TouchStats {
    /// Total touch events
    pub total_events: AtomicU64,
    /// Down events
    pub down_events: AtomicU64,
    /// Up events
    pub up_events: AtomicU64,
    /// Move events
    pub move_events: AtomicU64,
    /// Dropped events
    pub dropped_events: AtomicU64,
}

impl TouchStats {
    pub const fn new() -> Self {
        TouchStats {
            total_events: AtomicU64::new(0),
            down_events: AtomicU64::new(0),
            up_events: AtomicU64::new(0),
            move_events: AtomicU64::new(0),
            dropped_events: AtomicU64::new(0),
        }
    }
}

/// Touch Device Operations
pub struct TouchDeviceOps {
    /// Get touch points
    pub get_points:
        Option<unsafe extern "C" fn(*mut core::ffi::c_void, *mut TouchPoint, usize) -> i32>,
    /// Set configuration
    pub set_config: Option<unsafe extern "C" fn(*mut core::ffi::c_void, *const TouchConfig) -> i32>,
    /// Get configuration
    pub get_config: Option<unsafe extern "C" fn(*mut core::ffi::c_void, *mut TouchConfig) -> i32>,
    /// Reset device
    pub reset: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    /// Calibrate
    pub calibrate: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
}

/// Touch ioctl commands
pub mod touch_ioctl {
    /// Get touch points
    pub const GET_POINTS: u32 = 0x8001;
    /// Set configuration
    pub const SET_CONFIG: u32 = 0x8002;
    /// Get configuration
    pub const GET_CONFIG: u32 = 0x8003;
    /// Set sensitivity
    pub const SET_SENSITIVITY: u32 = 0x8004;
    /// Get sensitivity
    pub const GET_SENSITIVITY: u32 = 0x8005;
    /// Reset device
    pub const RESET: u32 = 0x8006;
    /// Calibrate
    pub const CALIBRATE: u32 = 0x8007;
    /// Get statistics
    pub const GET_STATS: u32 = 0x8008;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_touch_point_new() {
        let point = TouchPoint::new();
        assert_eq!(point.id, 0);
        assert_eq!(point.x, 0);
        assert_eq!(point.y, 0);
        assert_eq!(point.pressure, 0);
    }

    #[test]
    fn test_touch_event_type_values() {
        assert_eq!(TouchEventType::Down as i32, 0);
        assert_eq!(TouchEventType::Up as i32, 1);
        assert_eq!(TouchEventType::Move as i32, 2);
    }

    #[test]
    fn test_touch_config_default() {
        let config = TouchConfig::default();
        assert_eq!(config.width, 1080);
        assert_eq!(config.height, 1920);
        assert_eq!(config.max_touches, 10);
    }
}
