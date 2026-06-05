/*
 * Nuva OS - Kernel - Driver - Gpio
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
 * Nuva OS - Kernel - GPIO Framework
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * GPIO management for device drivers.
 */

use crate::{pr_debug, pr_info};
use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// GPIO Number
pub type GpioNum = u32;

/// GPIO Direction
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpioDir {
    /// Input
    Input = 0,
    /// Output
    Output = 1,
}

/// GPIO Value
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpioValue {
    /// Low
    Low = 0,
    /// High
    High = 1,
}

impl GpioValue {
    pub fn from_bool(v: bool) -> Self {
        if v {
            GpioValue::High
        } else {
            GpioValue::Low
        }
    }

    pub fn to_bool(self) -> bool {
        self == GpioValue::High
    }
}

/// GPIO Flags
bitflags::bitflags! {
    #[repr(transparent)]
    pub struct GpioFlags: u32 {
        /// Active low
        const ACTIVE_LOW = 1 << 0;
        /// Open drain
        const OPEN_DRAIN = 1 << 1;
        /// Open source
        const OPEN_SOURCE = 1 << 2;
        /// Pull up
        const PULL_UP = 1 << 3;
        /// Pull down
        const PULL_DOWN = 1 << 4;
        /// No pull
        const NO_PULL = 1 << 5;
        /// Drive strength (mask)
        const DRIVE_MASK = 0xF << 6;
        /// Input debounce
        const INPUT_DEBOUNCE = 1 << 10;
    }
}

/// GPIO Trigger
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpioTrigger {
    /// No trigger
    None = 0,
    /// Rising edge
    Rising = 1,
    /// Falling edge
    Falling = 2,
    /// Both edges
    Both = 3,
    /// High level
    High = 4,
    /// Low level
    Low = 5,
}

/// GPIO Descriptor
#[repr(C)]
pub struct GpioDesc {
    /// GPIO number
    pub gpio: GpioNum,
    /// Controller ID
    pub controller_id: u32,
    /// Flags
    pub flags: GpioFlags,
    /// Direction
    pub dir: AtomicU32,
    /// Value
    pub value: AtomicU32,
    /// Label
    pub label: [u8; 32],
    /// Use count
    pub use_count: AtomicU32,
}

impl GpioDesc {
    pub fn new(gpio: GpioNum) -> Self {
        GpioDesc {
            gpio,
            controller_id: 0,
            flags: GpioFlags::empty(),
            dir: AtomicU32::new(GpioDir::Input as u32),
            value: AtomicU32::new(GpioValue::Low as u32),
            label: [0; 32],
            use_count: AtomicU32::new(0),
        }
    }

    /// Get direction
    pub fn get_direction(&self) -> GpioDir {
        match self.dir.load(Ordering::Acquire) {
            0 => GpioDir::Input,
            _ => GpioDir::Output,
        }
    }

    /// Get value
    pub fn get_value(&self) -> GpioValue {
        match self.value.load(Ordering::Acquire) {
            0 => GpioValue::Low,
            _ => GpioValue::High,
        }
    }

    /// Check active low
    pub fn is_active_low(&self) -> bool {
        self.flags.contains(GpioFlags::ACTIVE_LOW)
    }
}

/// GPIO Operations
pub struct GpioOps {
    /// Request GPIO
    pub request: Option<unsafe extern "C" fn(*mut core::ffi::c_void, GpioNum, *const u8) -> i32>,
    /// Free GPIO
    pub free: Option<unsafe extern "C" fn(*mut core::ffi::c_void, GpioNum)>,
    /// Get direction
    pub get_direction: Option<unsafe extern "C" fn(*const core::ffi::c_void, GpioNum) -> i32>,
    /// Set direction input
    pub direction_input: Option<unsafe extern "C" fn(*mut core::ffi::c_void, GpioNum) -> i32>,
    /// Set direction output
    pub direction_output: Option<unsafe extern "C" fn(*mut core::ffi::c_void, GpioNum, i32) -> i32>,
    /// Get value
    pub get: Option<unsafe extern "C" fn(*const core::ffi::c_void, GpioNum) -> i32>,
    /// Set value
    pub set: Option<unsafe extern "C" fn(*mut core::ffi::c_void, GpioNum, i32)>,
    /// Set debounce
    pub set_debounce: Option<unsafe extern "C" fn(*mut core::ffi::c_void, GpioNum, u32) -> i32>,
    /// Set config
    pub set_config: Option<unsafe extern "C" fn(*mut core::ffi::c_void, GpioNum, u64) -> i32>,
    /// To IRQ
    pub to_irq: Option<unsafe extern "C" fn(*const core::ffi::c_void, GpioNum) -> i32>,
}

/// GPIO Controller (Chip)
pub struct GpioChip {
    /// Controller name
    pub name: [u8; 32],
    /// Controller ID
    pub id: u32,
    /// Base GPIO number
    pub base: GpioNum,
    /// Number of GPIOs
    pub ngpio: u16,
    /// Operations
    pub ops: GpioOps,
    /// Private data
    pub data: *mut core::ffi::c_void,
    /// Parent device
    pub parent: u32,
    /// IRQ base
    pub irq_base: i32,
}

/// GPIO Manager
pub struct GpioManager {
    /// Chip count
    chip_count: AtomicU32,
    /// Statistics
    stats: GpioStats,
}

/// GPIO Statistics
pub struct GpioStats {
    /// Request count
    pub request_count: AtomicU64,
    /// Free count
    pub free_count: AtomicU64,
    /// Get count
    pub get_count: AtomicU64,
    /// Set count
    pub set_count: AtomicU64,
}

impl GpioStats {
    pub const fn new() -> Self {
        GpioStats {
            request_count: AtomicU64::new(0),
            free_count: AtomicU64::new(0),
            get_count: AtomicU64::new(0),
            set_count: AtomicU64::new(0),
        }
    }
}

impl GpioManager {
    pub const fn new() -> Self {
        GpioManager {
            chip_count: AtomicU32::new(0),
            stats: GpioStats::new(),
        }
    }

    /// Initialize
    pub fn init(&self) {
        log_info!("GPIO manager initialized");
    }

    /// Register chip
    pub fn register_chip(&mut self, _chip: &GpioChip) -> u32 {
        let id = self.chip_count.fetch_add(1, Ordering::AcqRel);
        id
    }

    /// Request GPIO
    pub fn request(&mut self, gpio: GpioNum, label: &[u8]) -> i32 {
        self.stats.request_count.fetch_add(1, Ordering::AcqRel);
        log_debug!("gpio_request: gpio={}, label={:?}", gpio, label);
        0
    }

    /// Free GPIO
    pub fn free(&mut self, gpio: GpioNum) {
        self.stats.free_count.fetch_add(1, Ordering::AcqRel);
        log_debug!("gpio_free: gpio={}", gpio);
    }

    /// Set direction input
    pub fn direction_input(&mut self, gpio: GpioNum) -> i32 {
        log_debug!("gpio_direction_input: gpio={}", gpio);
        0
    }

    /// Set direction output
    pub fn direction_output(&mut self, gpio: GpioNum, value: GpioValue) -> i32 {
        log_debug!("gpio_direction_output: gpio={}, value={:?}", gpio, value);
        0
    }

    /// Get value
    pub fn get_value(&mut self, gpio: GpioNum) -> GpioValue {
        self.stats.get_count.fetch_add(1, Ordering::AcqRel);
        log_debug!("gpio_get_value: gpio={}", gpio);
        GpioValue::Low
    }

    /// Set value
    pub fn set_value(&mut self, gpio: GpioNum, value: GpioValue) {
        self.stats.set_count.fetch_add(1, Ordering::AcqRel);
        log_debug!("gpio_set_value: gpio={}, value={:?}", gpio, value);
    }

    /// Convert to IRQ
    pub fn to_irq(&self, gpio: GpioNum) -> i32 {
        log_debug!("gpio_to_irq: gpio={}", gpio);
        gpio as i32
    }
}

/// Global GPIO manager
static GPIO_MANAGER: core::sync::OnceLock<GpioManager> = core::sync::OnceLock::new();

/// Get GPIO manager
pub fn gpio_manager() -> &'static GpioManager {
    GPIO_MANAGER.get_or_init(GpioManager::new)
}

pub fn init_gpio_manager() -> &'static GpioManager {
    GPIO_MANAGER.get_or_init(GpioManager::new)
}

/// Initialize GPIO manager
pub fn init_gpio_manager() {
    let mgr = gpio_manager();
    mgr.init();
}

// Convenience functions

/// Request GPIO
pub fn gpio_request(gpio: GpioNum, label: &[u8]) -> i32 {
    gpio_manager().request(gpio, label)
}

/// Free GPIO
pub fn gpio_free(gpio: GpioNum) {
    gpio_manager().free(gpio);
}

/// Set direction input
pub fn gpio_direction_input(gpio: GpioNum) -> i32 {
    gpio_manager().direction_input(gpio)
}

/// Set direction output
pub fn gpio_direction_output(gpio: GpioNum, value: GpioValue) -> i32 {
    gpio_manager().direction_output(gpio, value)
}

/// Get value
pub fn gpio_get_value(gpio: GpioNum) -> GpioValue {
    gpio_manager().get_value(gpio)
}

/// Set value
pub fn gpio_set_value(gpio: GpioNum, value: GpioValue) {
    gpio_manager().set_value(gpio, value);
}

/// Convert to IRQ
pub fn gpio_to_irq(gpio: GpioNum) -> i32 {
    gpio_manager().to_irq(gpio)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpio_value() {
        assert_eq!(GpioValue::Low as i32, 0);
        assert_eq!(GpioValue::High as i32, 1);

        assert!(GpioValue::High.to_bool());
        assert!(!GpioValue::Low.to_bool());

        assert_eq!(GpioValue::from_bool(true), GpioValue::High);
        assert_eq!(GpioValue::from_bool(false), GpioValue::Low);
    }

    #[test]
    fn test_gpio_flags() {
        let flags = GpioFlags::ACTIVE_LOW | GpioFlags::PULL_UP;
        assert!(flags.contains(GpioFlags::ACTIVE_LOW));
        assert!(flags.contains(GpioFlags::PULL_UP));
    }

    #[test]
    fn test_gpio_desc() {
        let desc = GpioDesc::new(42);
        assert_eq!(desc.gpio, 42);
        assert_eq!(desc.get_direction(), GpioDir::Input);
        assert_eq!(desc.get_value(), GpioValue::Low);
    }
}
