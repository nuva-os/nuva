/*
 * Nuva OS - Kernel - Input Subsystem
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Input subsystem for handling input devices and events.
 */

use crate::{pr_debug, pr_info};
use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// Input Device ID
pub type InputDeviceId = u32;

/// Input Event Type
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputEventType {
    /// Synchronization
    Sync = 0x00,
    /// Key
    Key = 0x01,
    /// Relative
    Relative = 0x02,
    /// Absolute
    Absolute = 0x03,
    /// Misc
    Misc = 0x04,
    /// Switch
    Switch = 0x05,
    /// LED
    Led = 0x11,
    /// Sound
    Sound = 0x12,
    /// Repeat
    Repeat = 0x14,
    /// Force feedback
    Ff = 0x15,
    /// Power
    Power = 0x16,
    /// Force feedback status
    FfStatus = 0x17,
    /// Max
    Max = 0x1f,
}

/// Input Event
#[repr(C)]
pub struct InputEvent {
    /// Time (seconds)
    pub time_sec: u64,
    /// Time (microseconds)
    pub time_usec: u64,
    /// Event type
    pub event_type: u16,
    /// Event code
    pub code: u16,
    /// Event value
    pub value: i32,
}

impl InputEvent {
    /// Create key event
    pub fn key(code: u16, value: i32) -> Self {
        InputEvent {
            time_sec: 0,
            time_usec: 0,
            event_type: InputEventType::Key as u16,
            code,
            value,
        }
    }

    /// Create relative event
    pub fn relative(code: u16, value: i32) -> Self {
        InputEvent {
            time_sec: 0,
            time_usec: 0,
            event_type: InputEventType::Relative as u16,
            code,
            value,
        }
    }

    /// Create absolute event
    pub fn absolute(code: u16, value: i32) -> Self {
        InputEvent {
            time_sec: 0,
            time_usec: 0,
            event_type: InputEventType::Absolute as u16,
            code,
            value,
        }
    }

    /// Create sync event
    pub fn sync() -> Self {
        InputEvent {
            time_sec: 0,
            time_usec: 0,
            event_type: InputEventType::Sync as u16,
            code: 0,
            value: 0,
        }
    }
}

/// Input Device Info
#[repr(C)]
pub struct InputDeviceInfo {
    /// Device name
    pub name: [u8; 64],
    /// Device ID
    pub id: InputDeviceId,
    /// Bustype
    pub bustype: u16,
    /// Vendor
    pub vendor: u16,
    /// Product
    pub product: u16,
    /// Version
    pub version: u16,
    /// Capabilities
    pub caps: InputCaps,
    /// Number of keys
    pub num_keys: u16,
    /// Number of relative axes
    pub num_rel: u16,
    /// Number of absolute axes
    pub num_abs: u16,
    /// Number of LEDs
    pub num_leds: u16,
    /// Number of force feedback effects
    pub num_ff: u16,
}

/// Input Capabilities
bitflags::bitflags! {
    #[repr(transparent)]
    pub struct InputCaps: u32 {
        /// Keys/buttons
        const KEY = 1 << 0;
        /// Relative axes
        const REL = 1 << 1;
        /// Absolute axes
        const ABS = 1 << 2;
        /// Misc
        const MISC = 1 << 3;
        /// Switch
        const SW = 1 << 4;
        /// LED
        const LED = 1 << 5;
        /// Sound
        const SND = 1 << 6;
        /// Repeat
        const REP = 1 << 7;
        /// Force feedback
        const FF = 1 << 8;
        /// Power
        const PWR = 1 << 9;
    }
}

/// Input Absolute Info
#[repr(C)]
pub struct InputAbsInfo {
    /// Maximum value
    pub maximum: i32,
    /// Minimum value
    pub minimum: i32,
    /// Fuzz
    pub fuzz: i32,
    /// Flat
    pub flat: i32,
    /// Resolution
    pub resolution: i32,
    /// Current value
    pub value: i32,
}

/// Input Device Operations
pub struct InputDeviceOps {
    /// Open
    pub open: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    /// Close
    pub close: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    /// Event
    pub event: Option<unsafe extern "C" fn(*mut core::ffi::c_void, *const InputEvent) -> i32>,
    /// Get key
    pub get_key: Option<unsafe extern "C" fn(*const core::ffi::c_void, u16) -> i32>,
    /// Set key
    pub set_key: Option<unsafe extern "C" fn(*mut core::ffi::c_void, u16, i32) -> i32>,
    /// Get abs
    pub get_abs:
        Option<unsafe extern "C" fn(*const core::ffi::c_void, u16, *mut InputAbsInfo) -> i32>,
    /// Set abs
    pub set_abs:
        Option<unsafe extern "C" fn(*mut core::ffi::c_void, u16, *const InputAbsInfo) -> i32>,
    /// FF upload
    pub ff_upload:
        Option<unsafe extern "C" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> i32>,
    /// FF erase
    pub ff_erase:
        Option<unsafe extern "C" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> i32>,
}

/// Input Handler
pub struct InputHandler {
    /// Handler name
    pub name: [u8; 32],
    /// Match device table
    pub id_table: *const InputDeviceId,
    /// Connect
    pub connect:
        Option<unsafe extern "C" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> i32>,
    /// Disconnect
    pub disconnect: Option<unsafe extern "C" fn(*mut core::ffi::c_void, *mut core::ffi::c_void)>,
    /// Event
    pub event: Option<unsafe extern "C" fn(*mut core::ffi::c_void, *const InputEvent)>,
    /// Filter
    pub filter: Option<unsafe extern "C" fn(*mut core::ffi::c_void, *const InputEvent) -> bool>,
}

/// Input Subsystem
pub struct InputSubsystem {
    /// Device count
    dev_count: AtomicU32,
    /// Handler count
    handler_count: AtomicU32,
    /// Statistics
    stats: InputStats,
}

/// Input Statistics
pub struct InputStats {
    /// Events generated
    pub events: AtomicU64,
    /// Events dropped
    pub dropped: AtomicU64,
    /// Devices registered
    pub devices: AtomicU64,
}

impl InputStats {
    pub const fn new() -> Self {
        InputStats {
            events: AtomicU64::new(0),
            dropped: AtomicU64::new(0),
            devices: AtomicU64::new(0),
        }
    }
}

impl InputSubsystem {
    pub const fn new() -> Self {
        InputSubsystem {
            dev_count: AtomicU32::new(0),
            handler_count: AtomicU32::new(0),
            stats: InputStats::new(),
        }
    }

    /// Initialize
    pub fn init(&mut self) {
        log_info!("Input subsystem initialized");
    }

    /// Register device
    pub fn register_device(&mut self) -> InputDeviceId {
        self.stats.devices.fetch_add(1, Ordering::AcqRel);
        self.dev_count.fetch_add(1, Ordering::AcqRel)
    }

    /// Unregister device
    pub fn unregister_device(&mut self, dev_id: InputDeviceId) {
        log_debug!("input_unregister_device: id={}", dev_id);
    }

    /// Register handler
    pub fn register_handler(&mut self) -> u32 {
        self.handler_count.fetch_add(1, Ordering::AcqRel)
    }

    /// Event
    pub fn event(&mut self, dev_id: InputDeviceId, event: &InputEvent) -> i32 {
        self.stats.events.fetch_add(1, Ordering::AcqRel);
        log_debug!(
            "input_event: dev={}, type={}, code={}, value={}",
            dev_id,
            event.event_type,
            event.code,
            event.value
        );
        0
    }

    /// Grab device
    pub fn grab(&mut self, dev_id: InputDeviceId) -> i32 {
        log_debug!("input_grab: dev={}", dev_id);
        0
    }

    /// Ungrab device
    pub fn ungrab(&mut self, dev_id: InputDeviceId) {
        log_debug!("input_ungrab: dev={}", dev_id);
    }

    /// Get device count
    pub fn get_device_count(&self) -> u32 {
        self.dev_count.load(Ordering::Acquire)
    }
}

/// Global input subsystem
static INPUT_SUBSYSTEM: core::sync::OnceLock<InputSubsystem> = core::sync::OnceLock::new();

/// Get input subsystem
pub fn input_subsystem() -> &'static InputSubsystem {
    INPUT_SUBSYSTEM.get_or_init(InputSubsystem::new)
}

/// Initialize input subsystem
pub fn init_input_subsystem() {
    let subsys = get_input_subsystem();
    subsys.init();
}

// Convenience functions

/// Input event
pub fn input_event(dev_id: InputDeviceId, event: &InputEvent) -> i32 {
    get_input_subsystem().event(dev_id, event)
}

/// Report key
pub fn input_report_key(dev_id: InputDeviceId, code: u16, value: i32) {
    let event = InputEvent::key(code, value);
    get_input_subsystem().event(dev_id, &event);
}

/// Report relative
pub fn input_report_rel(dev_id: InputDeviceId, code: u16, value: i32) {
    let event = InputEvent::relative(code, value);
    get_input_subsystem().event(dev_id, &event);
}

/// Report absolute
pub fn input_report_abs(dev_id: InputDeviceId, code: u16, value: i32) {
    let event = InputEvent::absolute(code, value);
    get_input_subsystem().event(dev_id, &event);
}

/// Sync
pub fn input_sync(dev_id: InputDeviceId) {
    let event = InputEvent::sync();
    get_input_subsystem().event(dev_id, &event);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_event_type() {
        assert_eq!(InputEventType::Key as u16, 0x01);
        assert_eq!(InputEventType::Absolute as u16, 0x03);
    }

    #[test]
    fn test_input_event() {
        let event = InputEvent::key(0x1C, 1); // Enter key pressed
        assert_eq!(event.event_type, InputEventType::Key as u16);
        assert_eq!(event.code, 0x1C);
        assert_eq!(event.value, 1);
    }

    #[test]
    fn test_input_caps() {
        let caps = InputCaps::KEY | InputCaps::REL | InputCaps::ABS;
        assert!(caps.contains(InputCaps::KEY));
        assert!(caps.contains(InputCaps::REL));
        assert!(caps.contains(InputCaps::ABS));
    }
}
