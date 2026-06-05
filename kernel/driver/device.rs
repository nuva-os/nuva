/*
 * Nuva OS - Kernel - Driver - Device
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
 * Nuva OS - Kernel - Driver Framework
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 */

use crate::{pr_debug, pr_info};
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU32, AtomicU64, AtomicU8, Ordering};
use spin::Mutex as SpinLock;

/// Device number type
pub type DevNo = u64;

/// Device ID type
pub type DeviceId = u32;

/// Extract major device number
#[inline(always)]
pub fn major(dev: DevNo) -> u32 {
    (dev >> 20) as u32
}

/// Extract minor device number
#[inline(always)]
pub fn minor(dev: DevNo) -> u32 {
    (dev & 0xFFFFF) as u32
}

/// Compose device number from major and minor
#[inline(always)]
pub fn mkdev(major: u32, minor: u32) -> DevNo {
    ((major as DevNo) << 20) | (minor as DevNo)
}

/// Device type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceType {
    /// Character device
    Char = 0,
    /// Block device
    Block = 1,
    /// Network device
    Net = 2,
    /// Input device
    Input = 3,
    /// Display device
    Display = 4,
    /// Audio device
    Audio = 5,
    /// Bus device
    Bus = 6,
}

/// Device state (inspired by HDF state machine)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceState {
    /// Not initialized
    Uninitialized = 0,
    /// Initialized
    Initialized = 1,
    /// Bound to driver (HDF style)
    Bound = 2,
    /// Running
    Running = 3,
    /// Suspended
    Suspended = 4,
    /// Stopped
    Stopped = 5,
    /// Error state
    Error = 6,
}

/// Device flags
bitflags::bitflags! {
    #[repr(transparent)]
    pub struct DeviceFlags: u32 {
        /// Readable
        const READABLE = 1 << 0;
        /// Writable
        const WRITABLE = 1 << 1;
        /// Executable
        const EXECUTABLE = 1 << 2;
        /// Removable
        const REMOVABLE = 1 << 3;
        /// Solid State Drive
        const SSD = 1 << 4;
        /// Hot-pluggable
        const HOTPLUG = 1 << 5;
        /// Mounted
        const MOUNTED = 1 << 6;
        /// Primary device
        const PRIMARY = 1 << 7;
    }
}

/// Device property value (inspired by I/O Kit PropertyTable)
#[derive(Debug, Clone)]
pub enum DeviceProperty {
    /// String value
    String(String),
    /// Integer value
    Integer(i64),
    /// Boolean value
    Bool(bool),
    /// Binary data
    Data(Vec<u8>),
}

impl DeviceProperty {
    /// Get as string if applicable
    pub fn as_string(&self) -> Option<&str> {
        match self {
            DeviceProperty::String(s) => Some(s),
            _ => None,
        }
    }

    /// Get as integer if applicable
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            DeviceProperty::Integer(i) => Some(*i),
            _ => None,
        }
    }

    /// Check if this property matches another
    pub fn matches(&self, other: &DeviceProperty) -> bool {
        match (self, other) {
            (DeviceProperty::String(a), DeviceProperty::String(b)) => a == b,
            (DeviceProperty::Integer(a), DeviceProperty::Integer(b)) => a == b,
            (DeviceProperty::Bool(a), DeviceProperty::Bool(b)) => a == b,
            (DeviceProperty::Data(a), DeviceProperty::Data(b)) => a == b,
            _ => false,
        }
    }
}

/// Device property table (inspired by I/O Kit)
#[derive(Debug, Clone, Default)]
pub struct PropertyTable {
    properties: BTreeMap<String, DeviceProperty>,
}

impl PropertyTable {
    pub fn new() -> Self {
        Self {
            properties: BTreeMap::new(),
        }
    }

    /// Set a property
    /// @param key: Property name
    /// @param value: Property value
    pub fn set(&mut self, key: impl Into<String>, value: DeviceProperty) {
        self.properties.insert(key.into(), value);
    }

    /// Get a property
    /// @param key: Property name
    /// @return Reference to property if found
    pub fn get(&self, key: &str) -> Option<&DeviceProperty> {
        self.properties.get(key)
    }

    /// Check if this table matches another
    /// @param other: Table to match against
    /// @return true if all properties match
    pub fn matches(&self, other: &PropertyTable) -> bool {
        for (key, value) in &self.properties {
            if let Some(other_value) = other.get(key) {
                if !value.matches(other_value) {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    }
}

/// Device operations
pub struct DeviceOps {
    /// Open device
    pub open: Option<fn(&Device) -> i32>,
    /// Close device
    pub close: Option<fn(&Device) -> i32>,
    /// Read from device
    pub read: Option<fn(&Device, &mut [u8]) -> i32>,
    /// Write to device
    pub write: Option<fn(&Device, &[u8]) -> i32>,
    /// I/O control
    pub ioctl: Option<fn(&Device, u32, u64) -> i32>,
    /// Memory map
    pub mmap: Option<fn(&Device, u64, usize) -> u64>,
    /// Poll for events
    pub poll: Option<fn(&Device) -> u32>,
}

impl Default for DeviceOps {
    fn default() -> Self {
        Self {
            open: None,
            close: None,
            read: None,
            write: None,
            ioctl: None,
            mmap: None,
            poll: None,
        }
    }
}

/// Device structure
/// Represents a device in the system.
pub struct Device {
    /// Device ID
    pub id: DeviceId,
    /// Device number
    pub dev_no: DevNo,
    /// Device type
    pub dev_type: DeviceType,
    /// Device name
    pub name: [u8; 32],
    /// Current state
    pub state: AtomicU32,
    /// Device flags
    pub flags: AtomicU32,
    /// Device operations
    pub ops: DeviceOps,
    /// Private driver data
    pub private_data: u64,
    /// Parent device
    pub parent: *mut Device,
    /// Bound driver
    pub driver: *mut Driver,
    /// Reference count
    pub ref_count: AtomicU32,
    /// Property table (inspired by I/O Kit)
    pub properties: SpinLock<PropertyTable>,
    /// Power state (inspired by HDF)
    pub power_state: AtomicU8,
}

impl Device {
    /// Create a new device
    /// @param id: Device ID
    /// @param dev_no: Device number
    /// @param dev_type: Device type
    /// @param name: Device name
    /// @return New Device instance
    pub fn new(id: DeviceId, dev_no: DevNo, dev_type: DeviceType, name: &[u8]) -> Self {
        let mut device = Device {
            id,
            dev_no,
            dev_type,
            name: [0; 32],
            state: AtomicU32::new(DeviceState::Uninitialized as u32),
            flags: AtomicU32::new(0),
            ops: DeviceOps::default(),
            private_data: 0,
            parent: core::ptr::null_mut(),
            driver: core::ptr::null_mut(),
            ref_count: AtomicU32::new(1),
            properties: SpinLock::new(PropertyTable::new()),
            power_state: AtomicU8::new(super::PowerState::On as u8),
        };

        let len = name.len().min(31);
        device.name[..len].copy_from_slice(&name[..len]);

        device
    }

    /// Get current device state
    pub fn get_state(&self) -> DeviceState {
        match self.state.load(Ordering::Acquire) {
            0 => DeviceState::Uninitialized,
            1 => DeviceState::Initialized,
            2 => DeviceState::Bound,
            3 => DeviceState::Running,
            4 => DeviceState::Suspended,
            5 => DeviceState::Stopped,
            _ => DeviceState::Error,
        }
    }

    /// Set device state
    /// @param state: New state
    pub fn set_state(&self, state: DeviceState) {
        self.state.store(state as u32, Ordering::Release);
    }

    /// Get current power state
    pub fn get_power_state(&self) -> super::PowerState {
        match self.power_state.load(Ordering::Acquire) {
            0 => super::PowerState::On,
            1 => super::PowerState::Sleep,
            2 => super::PowerState::Suspend,
            _ => super::PowerState::Off,
        }
    }

    /// Set power state
    /// @param state: New power state
    pub fn set_power_state(&self, state: super::PowerState) {
        self.power_state.store(state as u8, Ordering::Release);
    }

    /// Set a property (inspired by I/O Kit)
    /// @param key: Property name
    /// @param value: Property value
    pub fn set_property(&self, key: impl Into<String>, value: DeviceProperty) {
        self.properties.lock().set(key, value);
    }

    /// Get a property
    /// @param key: Property name
    /// @return Property value if found
    pub fn get_property(&self, key: &str) -> Option<DeviceProperty> {
        self.properties.lock().get(key).cloned()
    }

    /// Check if device matches a property table
    /// @param table: Table to match against
    /// @return true if matches
    pub fn matches(&self, table: &PropertyTable) -> bool {
        self.properties.lock().matches(table)
    }

    /// Open the device
    pub fn open(&self) -> i32 {
        if let Some(open) = self.ops.open {
            open(self)
        } else {
            0
        }
    }

    /// Close the device
    pub fn close(&self) -> i32 {
        if let Some(close) = self.ops.close {
            close(self)
        } else {
            0
        }
    }

    /// Read from device
    /// @param buf: Buffer to read into
    /// @return Number of bytes read, or negative error
    pub fn read(&self, buf: &mut [u8]) -> i32 {
        if let Some(read) = self.ops.read {
            read(self, buf)
        } else {
            -1
        }
    }

    /// Write to device
    /// @param buf: Buffer to write from
    /// @return Number of bytes written, or negative error
    pub fn write(&self, buf: &[u8]) -> i32 {
        if let Some(write) = self.ops.write {
            write(self, buf)
        } else {
            -1
        }
    }

    /// I/O control
    /// @param cmd: Command number
    /// @param arg: Command argument
    /// @return Result code
    pub fn ioctl(&self, cmd: u32, arg: u64) -> i32 {
        if let Some(ioctl) = self.ops.ioctl {
            ioctl(self, cmd, arg)
        } else {
            -1
        }
    }
}

/// Driver match table (inspired by I/O Kit matching)
pub struct DriverMatchTable {
    /// Properties to match
    pub match_table: PropertyTable,
    /// Priority (inspired by HDF priority)
    pub priority: u32,
}

impl DriverMatchTable {
    pub fn new() -> Self {
        Self {
            match_table: PropertyTable::new(),
            priority: 100,
        }
    }

    /// Check if device matches
    /// @param device: Device to check
    /// @return true if matches
    pub fn matches(&self, device: &Device) -> bool {
        device.matches(&self.match_table)
    }

    /// Calculate match score (inspired by I/O Kit probe score)
    /// @param device: Device to score
    /// @return Match score, or -1 if no match
    pub fn match_score(&self, device: &Device) -> i32 {
        if self.matches(device) {
            // Higher priority = higher score
            (1000 - self.priority) as i32
        } else {
            -1
        }
    }
}

impl Default for DriverMatchTable {
    fn default() -> Self {
        Self::new()
    }
}

/// Driver structure
/// Represents a device driver in the system.
pub struct Driver {
    /// Driver name
    pub name: [u8; 32],
    /// Supported device type
    pub dev_type: DeviceType,
    /// Probe device (returns match score, inspired by I/O Kit)
    pub probe: Option<fn(&mut Device) -> i32>,
    /// Remove device
    pub remove: Option<fn(&mut Device) -> i32>,
    /// Suspend device (inspired by HDF power_manage)
    pub suspend: Option<fn(&Device) -> i32>,
    /// Resume device
    pub resume: Option<fn(&Device) -> i32>,
    /// Initialize driver
    pub init: Option<fn() -> i32>,
    /// Cleanup driver
    pub cleanup: Option<fn()>,
    /// Device list head
    pub devices: *mut Device,
    /// Number of bound devices
    pub device_count: AtomicU32,
    /// Match table (inspired by I/O Kit)
    pub match_table: DriverMatchTable,
}

impl Driver {
    /// Create a new driver
    /// @param name: Driver name
    /// @param dev_type: Supported device type
    /// @return New Driver instance
    pub fn new(name: &[u8], dev_type: DeviceType) -> Self {
        let mut driver = Driver {
            name: [0; 32],
            dev_type,
            probe: None,
            remove: None,
            suspend: None,
            resume: None,
            init: None,
            cleanup: None,
            devices: core::ptr::null_mut(),
            device_count: AtomicU32::new(0),
            match_table: DriverMatchTable::new(),
        };

        let len = name.len().min(31);
        driver.name[..len].copy_from_slice(&name[..len]);

        driver
    }

    /// Set match table
    /// @param table: Match table to use
    /// @return Self for chaining
    pub fn with_match_table(mut self, table: DriverMatchTable) -> Self {
        self.match_table = table;
        self
    }

    /// Set priority
    /// @param priority: Priority value (lower = higher priority)
    /// @return Self for chaining
    pub fn with_priority(mut self, priority: u32) -> Self {
        self.match_table.priority = priority;
        self
    }

    /// Register the driver
    pub fn register(&mut self) -> i32 {
        if let Some(init) = self.init {
            init()
        } else {
            0
        }
    }

    /// Unregister the driver
    pub fn unregister(&mut self) {
        if let Some(cleanup) = self.cleanup {
            cleanup();
        }
    }

    /// Probe a device
    /// @param device: Device to probe
    /// @return Match score
    pub fn probe(&self, device: &mut Device) -> i32 {
        if let Some(probe) = self.probe {
            probe(device)
        } else {
            // Use match table to calculate score
            self.match_table.match_score(device)
        }
    }

    /// Check if driver matches device
    /// @param device: Device to check
    /// @return true if matches
    pub fn matches(&self, device: &Device) -> bool {
        device.dev_type == self.dev_type && self.match_table.matches(device)
    }
}

/// Device Manager (inspired by HDF DriverManager and I/O Kit IoRegistry)
/// Manages all devices and drivers in the system.
pub struct DeviceManager {
    /// Number of registered devices
    pub device_count: AtomicU32,
    /// Number of registered drivers
    pub driver_count: AtomicU32,
    /// Next device ID to allocate
    pub next_device_id: AtomicU32,
    /// Device registry (inspired by I/O Kit registry)
    devices: SpinLock<BTreeMap<DeviceId, Arc<Device>>>,
    /// Driver list
    drivers: SpinLock<Vec<Arc<Driver>>>,
}

impl DeviceManager {
    pub const fn new() -> Self {
        DeviceManager {
            device_count: AtomicU32::new(0),
            driver_count: AtomicU32::new(0),
            next_device_id: AtomicU32::new(1),
            devices: SpinLock::new(BTreeMap::new()),
            drivers: SpinLock::new(Vec::new()),
        }
    }

    /// Initialize device manager
    pub fn init(&self) {
        log_info!("Device manager initialized");
    }

    /// Register a device
    /// @param device: Device to register
    /// @return Assigned device ID
    pub fn register_device(&self, device: Arc<Device>) -> DeviceId {
        let id = self.next_device_id.fetch_add(1, Ordering::AcqRel);
        self.device_count.fetch_add(1, Ordering::AcqRel);

        self.devices.lock().insert(id, device);

        log_debug!("Registered device: id={}", id);

        // Try to match a driver
        self.match_driver(id);

        id
    }

    /// Unregister a device
    /// @param id: Device ID to unregister
    pub fn unregister_device(&self, id: DeviceId) {
        if self.devices.lock().remove(&id).is_some() {
            self.device_count.fetch_sub(1, Ordering::AcqRel);
        }
    }

    /// Register a driver
    /// @param driver: Driver to register
    pub fn register_driver(&self, driver: Arc<Driver>) {
        self.driver_count.fetch_add(1, Ordering::AcqRel);
        self.drivers.lock().push(driver);
    }

    /// Unregister a driver
    /// @param name: Driver name
    pub fn unregister_driver(&self, name: &[u8]) {
        let mut drivers = self.drivers.lock();
        if let Some(pos) = drivers
            .iter()
            .position(|d| d.name[..name.len().min(32)] == name[..name.len().min(32)])
        {
            drivers.remove(pos);
            self.driver_count.fetch_sub(1, Ordering::AcqRel);
        }
    }

    /// Match driver for device (inspired by HDF and I/O Kit matching)
    /// @param device_id: Device ID to match
    fn match_driver(&self, device_id: DeviceId) {
        let devices = self.devices.lock();
        let device = match devices.get(&device_id) {
            Some(d) => d,
            None => return,
        };

        let drivers = self.drivers.lock();

        // Find all matching drivers and calculate scores
        let mut best_match: Option<(i32, &Arc<Driver>)> = None;

        for driver in drivers.iter() {
            if driver.matches(device) {
                let score = driver.match_table.match_score(device);
                if let Some((best_score, _)) = best_match {
                    if score > best_score {
                        best_match = Some((score, driver));
                    }
                } else {
                    best_match = Some((score, driver));
                }
            }
        }

        if let Some((score, driver)) = best_match {
            log_debug!(
                "Matched driver {} for device {} (score={})",
                core::str::from_utf8(&driver.name).unwrap_or(""),
                device_id,
                score
            );
        }
    }

    /// Get device by ID
    /// @param id: Device ID
    /// @return Device if found
    pub fn get_device(&self, id: DeviceId) -> Option<Arc<Device>> {
        self.devices.lock().get(&id).cloned()
    }

    /// Get devices by type
    /// @param dev_type: Device type to filter
    /// @return List of matching devices
    pub fn get_devices_by_type(&self, dev_type: DeviceType) -> Vec<Arc<Device>> {
        self.devices
            .lock()
            .values()
            .filter(|d| d.dev_type == dev_type)
            .cloned()
            .collect()
    }

    /// Get number of registered devices
    pub fn get_device_count(&self) -> u32 {
        self.device_count.load(Ordering::Acquire)
    }

    /// Get number of registered drivers
    pub fn get_driver_count(&self) -> u32 {
        self.driver_count.load(Ordering::Acquire)
    }

    /// System power management (inspired by HDF)
    /// @param state: Power state to set
    pub fn system_power_manage(&self, state: super::PowerState) {
        let devices = self.devices.lock();
        for (_, device) in devices.iter() {
            device.set_power_state(state);
        }
        log_info!("System power state changed to {:?}", state);
    }
}

/// Global device manager instance
static DEVICE_MANAGER: core::sync::OnceLock<DeviceManager> = core::sync::OnceLock::new();

/// Get reference to global device manager
pub fn device_manager() -> &'static DeviceManager {
    DEVICE_MANAGER.get_or_init(DeviceManager::new)
}

pub fn init_device_manager() -> &'static DeviceManager {
    DEVICE_MANAGER.get_or_init(DeviceManager::new)
}

/// Initialize device manager
pub fn init_device_manager() {
    let dm = device_manager();
    dm.init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_major_minor() {
        let dev = mkdev(10, 5);
        assert_eq!(major(dev), 10);
        assert_eq!(minor(dev), 5);
    }

    #[test]
    fn test_device_properties() {
        let dev = Device::new(1, 0, DeviceType::Char, b"test");

        dev.set_property("vendor", DeviceProperty::String("Intel".into()));
        dev.set_property("rev", DeviceProperty::Integer(1));

        assert_eq!(
            dev.get_property("vendor").unwrap().as_string(),
            Some("Intel")
        );
        assert_eq!(dev.get_property("rev").unwrap().as_integer(), Some(1));
    }

    #[test]
    fn test_driver_matching() {
        let driver = Driver::new(b"test_drv", DeviceType::Char).with_priority(10);

        let dev = Device::new(1, 0, DeviceType::Char, b"test");

        // Type matches
        assert!(driver.matches(&dev));
    }

    #[test]
    fn test_device_state_transitions() {
        let dev = Device::new(1, 0, DeviceType::Char, b"test");

        assert_eq!(dev.get_state(), DeviceState::Uninitialized);

        dev.set_state(DeviceState::Initialized);
        assert_eq!(dev.get_state(), DeviceState::Initialized);

        dev.set_state(DeviceState::Bound);
        assert_eq!(dev.get_state(), DeviceState::Bound);

        dev.set_state(DeviceState::Running);
        assert_eq!(dev.get_state(), DeviceState::Running);
    }

    #[test]
    fn test_power_state() {
        let dev = Device::new(1, 0, DeviceType::Char, b"test");

        assert_eq!(dev.get_power_state(), super::PowerState::On);

        dev.set_power_state(super::PowerState::Suspend);
        assert_eq!(dev.get_power_state(), super::PowerState::Suspend);
    }
}
