/*
 * Nuva OS
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

// Input device HAL

/// Default input priority: touch-first
static mut INPUT_PRIORITY: u32 = 0;

/// Input device callback type
type InputCallback = Option<fn(InputEvent)>;

/// Global input event callback
static mut INPUT_CALLBACK: InputCallback = None;

/// Input device type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputDeviceType {
    /// Touch screen
    Touch,
    /// Keyboard
    Keyboard,
    /// Mouse/trackpad
    Mouse,
    /// Stylus
    Stylus,
    /// Gamepad
    Gamepad,
}

/// Input event
#[derive(Debug, Clone, Copy)]
pub struct InputEvent {
    /// Device ID
    pub device_id: u32,
    /// Event type
    pub event_type: InputDeviceType,
    /// X coordinate
    pub x: i32,
    /// Y coordinate
    pub y: i32,
    /// Z/pressure
    pub z: i32,
    /// Timestamp
    pub timestamp: u64,
}

/// Initialize input HAL
pub fn init_input_hal() {
    // SAFETY: Initializing static mut globals for input subsystem.
    // This is called once during system startup, no concurrent access.
    unsafe {
        INPUT_PRIORITY = 0;
        INPUT_CALLBACK = None;
    }
}

/// Set input priority (0 = touch-first, 1 = keyboard-first)
pub fn set_input_priority(priority: u32) {
    // SAFETY: Writing to INPUT_PRIORITY, a static mut global.
    // In a real system this would be protected by a spinlock.
    unsafe {
        INPUT_PRIORITY = priority;
    }
}

/// Get current input priority
pub fn get_input_priority() -> u32 {
    // SAFETY: Reading INPUT_PRIORITY, a static mut global.
    unsafe {
        INPUT_PRIORITY
    }
}

/// Register input event callback
pub fn register_input_callback(callback: fn(InputEvent)) {
    // SAFETY: Writing to INPUT_CALLBACK, a static mut global.
    unsafe {
        INPUT_CALLBACK = Some(callback);
    }
}

/// Dispatch input event to registered callback
pub fn dispatch_input_event(event: InputEvent) {
    // SAFETY: Reading INPUT_CALLBACK to invoke the registered handler.
    unsafe {
        if let Some(cb) = INPUT_CALLBACK {
            cb(event);
        }
    }
}
