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

//! InputDeviceDriverImplementation
/*!*/
//! Provides unified interface for input devices such as keyboard, mouse, and touchpad.

use alloc::sync::Arc;
use alloc::string::String;
use alloc::collections::VecDeque;
use core::sync::atomic::{AtomicU32, Ordering};

use super::{DeviceOps, DeviceInfo, DeviceState, DeviceType, DriverError, InputSubType};

// ============================================================================
// InputEventDefinition
// ============================================================================

/// InputEventType
#[derive(Debug, Clone, Copy)]
pub enum InputEventType {
    /// Key event
    Key = 0,
    /// Relative coordinate event (mouse move)
    Relative = 1,
    /// Absolute coordinate event (touch screen)
    Absolute = 2,
    /// SynchronousEvent
    Sync = 3,
}

/// Key code
#[derive(Debug, Clone, Copy)]
pub enum KeyCode {
    /// Letter keys
    A = 30,
    B = 48,
    C = 46,
    D = 32,
    E = 18,
    F = 33,
    G = 34,
    H = 35,
    I = 23,
    J = 36,
    K = 37,
    L = 38,
    M = 50,
    N = 49,
    O = 24,
    P = 25,
    Q = 16,
    R = 19,
    S = 31,
    T = 20,
    U = 22,
    V = 47,
    W = 17,
    X = 45,
    Y = 21,
    Z = 44,
    
    /// Number keys
    Num0 = 11,
    Num1 = 2,
    Num2 = 3,
    Num3 = 4,
    Num4 = 5,
    Num5 = 6,
    Num6 = 7,
    Num7 = 8,
    Num8 = 9,
    Num9 = 10,
    
    /// Function keys
    Enter = 28,
    Escape = 1,
    Backspace = 14,
    Tab = 15,
    Space = 57,
    
    /// Direction keys
    Up = 103,
    Down = 108,
    Left = 105,
    Right = 106,
    
    /// Modifier keys
    LeftShift = 42,
    RightShift = 54,
    LeftCtrl = 29,
    RightCtrl = 97,
    LeftAlt = 56,
    RightAlt = 100,
    
    /// Mouse button keys
    MouseLeft = 272,
    MouseRight = 273,
    MouseMiddle = 274,
}

/// InputEvent
#[derive(Debug, Clone, Copy)]
pub struct InputEvent {
    /// EventType
    pub event_type: InputEventType,
    /// Event code
    pub code: u16,
    /// Event value
    pub value: i32,
    /// Timestamp (microseconds)
    pub timestamp_us: u64,
}

// ============================================================================
// InputDeviceConfig
// ============================================================================

/// InputDeviceConfig
#[derive(Debug, Clone)]
pub struct InputConfig {
    /// InputDeviceChildType
    pub sub_type: InputSubType,
    /// EventQueueSize
    pub event_queue_size: usize,
    /// Supported event types
    pub supported_events: u32,
    /// Key repeat delay (milliseconds)
    pub repeat_delay_ms: u32,
    /// Key repeat interval (milliseconds)
    pub repeat_interval_ms: u32,
}

// ============================================================================
// InputDeviceOperationInterface
// ============================================================================

/// Input device extended operation interface
pub trait InputOps: DeviceOps {
    /// GetInputConfig
    fn get_config(&self) -> &InputConfig;
    
    /// ReadInputEvent
    fn read_event(&mut self) -> Result<InputEvent, DriverError>;
    
    /// Read all events
    fn read_events(&mut self, events: &mut [InputEvent]) -> Result<usize, DriverError>;
    
    /// Get event count
    fn get_event_count(&self) -> usize;
    
    /// ClearEventQueue
    fn clear_events(&mut self);
    
    /// Set LED State（Keyboard）
    fn set_led(&mut self, led: u8, state: bool) -> Result<(), DriverError>;
    
    /// Get key state
    fn get_key_state(&self, key_code: KeyCode) -> bool;
}

// ============================================================================
// GeneralInputDeviceImplementation
// ============================================================================

/// GeneralInputDevice
pub struct GenericInput {
    /// DeviceInfo
    info: DeviceInfo,
    /// InputConfig
    config: InputConfig,
    /// EventQueue
    event_queue: VecDeque<InputEvent>,
    /// Key state table
    key_state: [bool; 256],
}

impl GenericInput {
    /// Create a new input device
    pub fn new(name: String, config: InputConfig) -> Self {
        Self {
            info: DeviceInfo {
                name,
                device_type: DeviceType::Input,
                id: super::DeviceId {
                    vendor_id: 0,
                    device_id: 0,
                    bus_type: 0,
                    bus_number: 0,
                },
                state: DeviceState::Uninitialized,
                driver_name: String::from("generic_input"),
                device_path: String::from("/dev/input0"),
                private_data_size: 0,
                flags: 0,
            },
            config,
            event_queue: VecDeque::new(),
            key_state: [false; 256],
        }
    }
    
    /// PushEvent
    pub fn push_event(&mut self, event: InputEvent) {
        if self.event_queue.len() >= self.config.event_queue_size {
            self.event_queue.pop_front();
        }
        
        // Update key state
        if event.event_type == InputEventType::Key {
            let key_index = event.code as usize;
            if key_index < 256 {
                self.key_state[key_index] = event.value > 0;
            }
        }
        
        self.event_queue.push_back(event);
    }
}

impl DeviceOps for GenericInput {
    fn get_info(&self) -> &DeviceInfo {
        &self.info
    }
    
    fn init(&mut self) -> Result<(), DriverError> {
        self.info.state = DeviceState::Initialized;
        Ok(())
    }
    
    fn deinit(&mut self) -> Result<(), DriverError> {
        self.info.state = DeviceState::Uninitialized;
        self.event_queue.clear();
        Ok(())
    }
    
    fn start(&mut self) -> Result<(), DriverError> {
        if self.info.state != DeviceState::Initialized {
            return Err(DriverError::NotInitialized);
        }
        self.info.state = DeviceState::Running;
        Ok(())
    }
    
    fn stop(&mut self) -> Result<(), DriverError> {
        self.info.state = DeviceState::Initialized;
        Ok(())
    }
    
    fn suspend(&mut self) -> Result<(), DriverError> {
        self.info.state = DeviceState::Suspended;
        Ok(())
    }
    
    fn resume(&mut self) -> Result<(), DriverError> {
        self.info.state = DeviceState::Running;
        Ok(())
    }
    
    fn read(&self, buffer: &mut [u8], offset: u64) -> Result<usize, DriverError> {
        // Read event data
        // TODO: Implement event serialization
        Err(DriverError::NotSupported)
    }
    
    fn write(&self, buffer: &[u8], offset: u64) -> Result<usize, DriverError> {
        // Input device does not typically support write
        Err(DriverError::NotSupported)
    }
    
    fn ioctl(&mut self, cmd: u32, arg: u64) -> Result<u64, DriverError> {
        match cmd {
            0 => Ok(self.get_event_count() as u64),
            _ => Err(DriverError::NotSupported),
        }
    }
    
    fn get_state(&self) -> DeviceState {
        self.info.state
    }
    
    fn set_state(&mut self, state: DeviceState) {
        self.info.state = state;
    }
}

impl InputOps for GenericInput {
    fn get_config(&self) -> &InputConfig {
        &self.config
    }
    
    fn read_event(&mut self) -> Result<InputEvent, DriverError> {
        self.event_queue.pop_front().ok_or(DriverError::DeviceNotFound)
    }
    
    fn read_events(&mut self, events: &mut [InputEvent]) -> Result<usize, DriverError> {
        let mut count = 0;
        
        for event in events.iter_mut() {
            if let Some(e) = self.event_queue.pop_front() {
                *event = e;
                count += 1;
            } else {
                break;
            }
        }
        
        if count == 0 {
            Err(DriverError::DeviceNotFound)
        } else {
            Ok(count)
        }
    }
    
    fn get_event_count(&self) -> usize {
        self.event_queue.len()
    }
    
    fn clear_events(&mut self) {
        self.event_queue.clear();
    }
    
    fn set_led(&mut self, led: u8, state: bool) -> Result<(), DriverError> {
        // TODO: Implement LED control
        Ok(())
    }
    
    fn get_key_state(&self, key_code: KeyCode) -> bool {
        let key_index = key_code as usize;
        if key_index < 256 {
            self.key_state[key_index]
        } else {
            false
        }
    }
}

// ============================================================================
// KeyboardDevice
// ============================================================================

/// KeyboardDevice
pub type Keyboard = GenericInput;

impl Keyboard {
    /// CreateKeyboardDevice
    pub fn new_keyboard(name: String) -> Self {
        let config = InputConfig {
            sub_type: InputSubType::Keyboard,
            event_queue_size: 256,
            supported_events: 0x1,  // Support key events
            repeat_delay_ms: 500,
            repeat_interval_ms: 50,
        };
        
        Self::new(name, config)
    }
}

// ============================================================================
// MouseDevice
// ============================================================================

/// MouseDevice
pub type Mouse = GenericInput;

impl Mouse {
    /// CreateMouseDevice
    pub fn new_mouse(name: String) -> Self {
        let config = InputConfig {
            sub_type: InputSubType::Mouse,
            event_queue_size: 512,
            supported_events: 0x3,  // Support key and relative coordinate events
            repeat_delay_ms: 0,
            repeat_interval_ms: 0,
        };
        
        Self::new(name, config)
    }
    
    /// Move mouse
    pub fn move_cursor(&mut self, dx: i32, dy: i32) {
        let event = InputEvent {
            event_type: InputEventType::Relative,
            code: 0,  // X axis
            value: dx,
            timestamp_us: 0,  // TODO: Get current time
        };
        self.push_event(event);
        
        let event = InputEvent {
            event_type: InputEventType::Relative,
            code: 1,  // Y axis
            value: dy,
            timestamp_us: 0,
        };
        self.push_event(event);
    }
    
    /// Click mouse button
    pub fn click(&mut self, button: KeyCode) {
        let event = InputEvent {
            event_type: InputEventType::Key,
            code: button as u16,
            value: 1,  // Press
            timestamp_us: 0,
        };
        self.push_event(event);
        
        let event = InputEvent {
            event_type: InputEventType::Key,
            code: button as u16,
            value: 0,  // Release
            timestamp_us: 0,
        };
        self.push_event(event);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_keyboard() {
        let mut keyboard = Keyboard::new_keyboard(String::from("test_keyboard"));
        
        assert_eq!(keyboard.init(), Ok(()));
        assert_eq!(keyboard.start(), Ok(()));
        
        // Simulate key press
        let event = InputEvent {
            event_type: InputEventType::Key,
            code: KeyCode::A as u16,
            value: 1,
            timestamp_us: 0,
        };
        keyboard.push_event(event);
        
        assert_eq!(keyboard.get_event_count(), 1);
        assert!(keyboard.get_key_state(KeyCode::A));
    }
    
    #[test]
    fn test_mouse() {
        let mut mouse = Mouse::new_mouse(String::from("test_mouse"));
        
        assert_eq!(mouse.init(), Ok(()));
        assert_eq!(mouse.start(), Ok(()));
        
        // Move mouse
        mouse.move_cursor(10, 20);
        assert_eq!(mouse.get_event_count(), 2);
        
        // Click left button
        mouse.click(KeyCode::MouseLeft);
        assert_eq!(mouse.get_event_count(), 4);
    }
}