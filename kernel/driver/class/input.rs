/*
 * Nuva OS - Kernel - Driver - Class - Input
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
 * Nuva OS - Kernel - Input Device Class
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Standard interface for input devices (keyboard, mouse, joystick).
 */

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// Input Event Type
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputEventType {
    /// Key pressed/released
    Key = 0,
    /// Relative motion
    Relative = 1,
    /// Absolute position
    Absolute = 2,
    /// Device switch
    Switch = 3,
    /// LED state
    Led = 4,
    /// Sound
    Sound = 5,
    /// Repeat
    Repeat = 6,
    /// Force feedback
    ForceFeedback = 7,
    /// Device status
    Status = 8,
}

/// Standard input key codes (POSIX/Unix compatible).
/// Numeric values are preserved for driver compatibility.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyCode {
    // Special keys
    Reserved = 0,
    Esc = 1,

    // Number keys
    Key1 = 2,
    Key2 = 3,
    Key3 = 4,
    Key4 = 5,
    Key5 = 6,
    Key6 = 7,
    Key7 = 8,
    Key8 = 9,
    Key9 = 10,
    Key0 = 11,

    // Letter keys (subset)
    A = 30,
    B = 48,
    C = 46,
    D = 32,
    E = 18,
    // ... more keys

    // Function keys
    F1 = 59,
    F2 = 60,
    F3 = 61,
    F4 = 62,
    F5 = 63,
    F6 = 64,
    F7 = 65,
    F8 = 66,
    F9 = 67,
    F10 = 68,
    F11 = 69,
    F12 = 70,

    // Navigation keys
    Up = 103,
    Down = 108,
    Left = 105,
    Right = 106,
    Home = 102,
    End = 107,
    PageUp = 104,
    PageDown = 109,

    // Modifier keys
    LeftShift = 42,
    RightShift = 54,
    LeftCtrl = 29,
    RightCtrl = 97,
    LeftAlt = 56,
    RightAlt = 100,
    LeftMeta = 125,
    RightMeta = 126,

    // Special keys
    Enter = 28,
    Backspace = 14,
    Tab = 15,
    Space = 57,
    Delete = 111,
    Insert = 110,

    // Mouse buttons
    MouseLeft = 272,
    MouseRight = 273,
    MouseMiddle = 274,

    // Power/sleep
    Power = 116,
    Sleep = 142,
    Wakeup = 143,
}

/// Relative Axis
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelativeAxis {
    X = 0,
    Y = 1,
    Z = 2,
    Wheel = 8,
    HWheel = 9,
}

/// Absolute Axis
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AbsoluteAxis {
    X = 0,
    Y = 1,
    Z = 2,
    Pressure = 24,
    Distance = 25,
    TiltX = 26,
    TiltY = 27,
}

/// Input Event
#[repr(C)]
pub struct InputEvent {
    /// Timestamp (seconds)
    pub time_sec: u64,
    /// Timestamp (microseconds)
    pub time_usec: u64,
    /// Event type
    pub event_type: InputEventType,
    /// Code (key code, axis, etc.)
    pub code: u16,
    /// Value
    pub value: i32,
}

impl InputEvent {
    pub fn new(event_type: InputEventType, code: u16, value: i32) -> Self {
        InputEvent {
            time_sec: 0,
            time_usec: 0,
            event_type,
            code,
            value,
        }
    }

    /// Create key event
    pub fn key_event(key: KeyCode, pressed: bool) -> Self {
        Self::new(InputEventType::Key, key as u16, if pressed { 1 } else { 0 })
    }

    /// Create relative motion event
    pub fn relative_event(axis: RelativeAxis, value: i32) -> Self {
        Self::new(InputEventType::Relative, axis as u16, value)
    }

    /// Create absolute position event
    pub fn absolute_event(axis: AbsoluteAxis, value: i32) -> Self {
        Self::new(InputEventType::Absolute, axis as u16, value)
    }
}

/// Input Device Capabilities
bitflags::bitflags! {
    #[repr(transparent)]
    pub struct InputCapabilities: u32 {
        /// Keyboard
        const KEYBOARD = 1 << 0;
        /// Mouse
        const MOUSE = 1 << 1;
        /// Joystick
        const JOYSTICK = 1 << 2;
        /// Touchpad
        const TOUCHPAD = 1 << 3;
        /// Trackball
        const TRACKBALL = 1 << 4;
        /// Tablet
        const TABLET = 1 << 5;
        /// Multi-touch
        const MULTI_TOUCH = 1 << 6;
    }
}

/// Input device trait — composable interface replacing C function pointer table.
/// Each method provides a default no-op implementation for optional operations.
pub trait InputDevice: Send + Sync {
    /// Get the next input event. Returns 0 on success, negative errno on failure.
    fn get_event(&self, _event: &mut InputEvent) -> i32 {
        -1
    }

    /// Set LED state. Returns 0 on success, negative errno on failure.
    fn set_led(&self, _led: u16, _state: bool) -> i32 {
        -1
    }

    /// Get LED state.
    fn get_led(&self, _led: u16) -> bool {
        false
    }

    /// Set repeat rate (delay, period in ms). Returns 0 on success.
    fn set_repeat(&self, _delay: u32, _period: u32) -> i32 {
        -1
    }

    /// Grab or ungrab the device. Returns 0 on success.
    fn grab(&self, _exclusive: bool) -> i32 {
        -1
    }
}

/// Input Device Operations (legacy C function pointer table).
#[deprecated(since = "0.2.0", note = "Use InputDevice trait instead")]
pub struct InputDeviceOps {
    /// Get event
    pub get_event: Option<unsafe extern "C" fn(*mut core::ffi::c_void, *mut InputEvent) -> i32>,
    /// Set LED state
    pub set_led: Option<unsafe extern "C" fn(*mut core::ffi::c_void, u16, bool) -> i32>,
    /// Get LED state
    pub get_led: Option<unsafe extern "C" fn(*mut core::ffi::c_void, u16) -> bool>,
    /// Set repeat rate
    pub set_repeat: Option<unsafe extern "C" fn(*mut core::ffi::c_void, u32, u32) -> i32>,
    /// Grab device
    pub grab: Option<unsafe extern "C" fn(*mut core::ffi::c_void, bool) -> i32>,
}

/// Input device ioctl commands — type-safe enumeration replacing hardcoded u32 constants.
/// Values are preserved for ABI compatibility with the io_uring and VFS layers.
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputIoctlCmd {
    /// Get device capabilities
    GetCaps = 0xB001,
    /// Get key state
    GetKey = 0xB002,
    /// Set LED
    SetLed = 0xB003,
    /// Get LED
    GetLed = 0xB004,
    /// Set repeat rate
    SetRepeat = 0xB005,
    /// Grab device
    Grab = 0xB006,
    /// Ungrab device
    Ungrab = 0xB007,
}

impl TryFrom<u32> for InputIoctlCmd {
    type Error = u32;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0xB001 => Ok(InputIoctlCmd::GetCaps),
            0xB002 => Ok(InputIoctlCmd::GetKey),
            0xB003 => Ok(InputIoctlCmd::SetLed),
            0xB004 => Ok(InputIoctlCmd::GetLed),
            0xB005 => Ok(InputIoctlCmd::SetRepeat),
            0xB006 => Ok(InputIoctlCmd::Grab),
            0xB007 => Ok(InputIoctlCmd::Ungrab),
            _ => Err(value),
        }
    }
}

/// Input ioctl commands (legacy constants module).
#[deprecated(since = "0.2.0", note = "Use InputIoctlCmd enum instead")]
pub mod input_ioctl {
    /// Get device capabilities
    pub const GET_CAPS: u32 = super::InputIoctlCmd::GetCaps as u32;
    /// Get key state
    pub const GET_KEY: u32 = super::InputIoctlCmd::GetKey as u32;
    /// Set LED
    pub const SET_LED: u32 = super::InputIoctlCmd::SetLed as u32;
    /// Get LED
    pub const GET_LED: u32 = super::InputIoctlCmd::GetLed as u32;
    /// Set repeat rate
    pub const SET_REPEAT: u32 = super::InputIoctlCmd::SetRepeat as u32;
    /// Grab device
    pub const GRAB: u32 = super::InputIoctlCmd::Grab as u32;
    /// Ungrab device
    pub const UNGRAB: u32 = super::InputIoctlCmd::Ungrab as u32;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_code_values() {
        assert_eq!(KeyCode::Esc as u16, 1);
        assert_eq!(KeyCode::Enter as u16, 28);
        assert_eq!(KeyCode::Up as u16, 103);
    }

    #[test]
    fn test_input_event_key() {
        let event = InputEvent::key_event(KeyCode::A, true);
        assert_eq!(event.event_type, InputEventType::Key);
        assert_eq!(event.code, KeyCode::A as u16);
        assert_eq!(event.value, 1);
    }

    #[test]
    fn test_input_event_relative() {
        let event = InputEvent::relative_event(RelativeAxis::X, 10);
        assert_eq!(event.event_type, InputEventType::Relative);
        assert_eq!(event.code, RelativeAxis::X as u16);
        assert_eq!(event.value, 10);
    }
}
