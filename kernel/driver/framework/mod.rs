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

//! Nuva OS Unified Driver Framework
/*!*/
//! Provides unified device driver interface, supporting plug-in driver management.
/*!*/
//! # Core Features
/*!*/
//! - **Unified Interface**: All device driver implementations use unified interface
//! - **Plug-in System**: Support dynamic loading and unloading of drivers
//! - **Device Classification**: Manage by device type classification
//! - **Vendor Integration**: Provide standard interface for vendor implementation
//! - **Auto Discovery**: Automatically discover and load device drivers
/*!*/
//! # Device Types
/*!*/
//! - Display Device (Display)
//! - Camera Device (Camera)
//! - Bluetooth Device (Bluetooth)
//! - USB Device (USB)
//! - Input Device (Input: Keyboard, Mouse, Touchpad)
//! - NFC Device (NFC)
//! - Sensor Device (Sensor)
//! - WiFi Device (WiFi)
//! - Audio Device (Audio)
//! - Storage Device (Storage)

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use alloc::sync::Arc;
use alloc::string::String;
use alloc::vec::Vec;

// Nuva native async-first driver operation model
pub mod nv_operation;

// ============================================================================
// DeviceTypeDefinition
// ============================================================================

/// Device type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceType {
    /// Display device
    Display = 0,
    /// Camera device
    Camera = 1,
    /// Bluetooth device
    Bluetooth = 2,
    /// USB device
    Usb = 3,
    /// Input device (Keyboard, Mouse, Touchpad)
    Input = 4,
    /// NFC device
    Nfc = 5,
    /// Sensor device
    Sensor = 6,
    /// WiFi device
    Wifi = 7,
    /// Audio device
    Audio = 8,
    /// Storage device
    Storage = 9,
    /// Power device
    Power = 10,
    /// LED device
    Led = 11,
    /// Vibrator device
    Vibrator = 12,
    /// Backlight device
    Backlight = 13,
    /// EEPROM device
    Eeprom = 14,
    /// Touch device
    Touch = 15,
    /// Unknown device
    Unknown = 255,
}

/// Input device sub-type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputSubType {
    /// Keyboard
    Keyboard = 0,
    /// Mouse
    Mouse = 1,
    /// Touchpad
    Touchpad = 2,
    /// Touch screen
    Touchscreen = 3,
    /// Gamepad
    Gamepad = 4,
    /// Tablet
    Tablet = 5,
}

/// Device state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceState {
    /// Uninitialized
    Uninitialized = 0,
    /// Initialized
    Initialized = 1,
    /// Running
    Running = 2,
    /// Suspended
    Suspended = 3,
    /// Error state
    Error = 4,
    /// Removed
    Removed = 5,
}

// ============================================================================
// Device information structure
// ============================================================================

/// Device identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeviceId {
    /// Vendor ID
    pub vendor_id: u16,
    /// Device ID
    pub device_id: u16,
    /// Bus type
    pub bus_type: u8,
    /// Bus number
    pub bus_number: u8,
}

/// Device information
#[derive(Debug, Clone)]
pub struct DeviceInfo {
    /// Device name
    pub name: String,
    /// Device type
    pub device_type: DeviceType,
    /// Device ID
    pub id: DeviceId,
    /// Device state
    pub state: DeviceState,
    /// Driver name
    pub driver_name: String,
    /// Device path
    pub device_path: String,
    /// Private data size
    pub private_data_size: usize,
    /// Flag bits
    pub flags: u32,
}

// ============================================================================
// Unified Device Interface
// ============================================================================

/// Unified device operation interface
pub trait DeviceOps: Send + Sync {
    /// GetDeviceInfo
    fn get_info(&self) -> &DeviceInfo;
    
    /// InitializeDevice
    fn init(&mut self) -> Result<(), DriverError>;
    
    /// DeinitializeDevice
    fn deinit(&mut self) -> Result<(), DriverError>;
    
    /// StartDevice
    fn start(&mut self) -> Result<(), DriverError>;
    
    /// StopDevice
    fn stop(&mut self) -> Result<(), DriverError>;
    
    /// SuspendDevice
    fn suspend(&mut self) -> Result<(), DriverError>;
    
    /// RecoveryDevice
    fn resume(&mut self) -> Result<(), DriverError>;
    
    /// ReadData
    fn read(&self, buffer: &mut [u8], offset: u64) -> Result<usize, DriverError>;
    
    /// WriteData
    fn write(&self, buffer: &[u8], offset: u64) -> Result<usize, DriverError>;
    
    /// IO Control
    fn ioctl(&mut self, cmd: u32, arg: u64) -> Result<u64, DriverError>;
    
    /// GetDeviceState
    fn get_state(&self) -> DeviceState;
    
    /// SetDeviceState
    fn set_state(&mut self, state: DeviceState);
}

// ============================================================================
// Driver Error Type
// ============================================================================

/// Driver error
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriverError {
    /// Device not found
    DeviceNotFound,
    /// Device already exists
    DeviceExists,
    /// Driver not found
    DriverNotFound,
    /// Driver already exists
    DriverExists,
    /// Initialize failed
    InitFailed,
    /// Operation failed
    OperationFailed,
    /// Invalid argument
    InvalidArgument,
    /// No memory
    NoMemory,
    /// Device busy
    DeviceBusy,
    /// Device not initialized
    NotInitialized,
    /// Permission denied
    PermissionDenied,
    /// Operation not supported
    NotSupported,
    /// Timeout
    Timeout,
    /// IO error
    IoError,
}

// ============================================================================
// Driver Plug-in Interface
// ============================================================================

/// Driver plug-in interface
pub trait DriverPlugin: Send + Sync {
    /// Get driver name
    fn get_name(&self) -> &str;
    
    /// Get driver version
    fn get_version(&self) -> &str;
    
    /// Get supported device types
    fn get_supported_types(&self) -> &[DeviceType];
    
    /// Probe device
    fn probe(&self, device_info: &DeviceInfo) -> Result<bool, DriverError>;
    
    /// Create device instance
    fn create_device(&self, device_info: &DeviceInfo) -> Result<Arc<dyn DeviceOps>, DriverError>;
    
    /// Destroy device instance
    fn destroy_device(&self, device: &Arc<dyn DeviceOps>) -> Result<(), DriverError>;
    
    /// Initialize driver
    fn init(&mut self) -> Result<(), DriverError>;
    
    /// Deinitialize driver
    fn deinit(&mut self) -> Result<(), DriverError>;
}

// ============================================================================
// DeviceManager
// ============================================================================

use spin::Mutex as SpinLock;

/// Device manager
pub struct DeviceManager {
    /// Registered devices
    devices: SpinLock<Vec<Arc<dyn DeviceOps>>>,
    /// Registered driver plug-ins
    drivers: SpinLock<Vec<Arc<dyn DriverPlugin>>>,
    /// Device count
    device_count: AtomicU32,
    /// Driver count
    driver_count: AtomicU32,
}

impl DeviceManager {
    /// Create new device manager
    pub fn new() -> Self {
        Self {
            devices: SpinLock::new(Vec::new()),
            drivers: SpinLock::new(Vec::new()),
            device_count: AtomicU32::new(0),
            driver_count: AtomicU32::new(0),
        }
    }
    
    /// Register driver plug-in
    pub fn register_driver(&self, driver: Arc<dyn DriverPlugin>) -> Result<(), DriverError> {
        let mut drivers = self.drivers.lock();
        
        // Check if already exists
        for existing in drivers.iter() {
            if existing.get_name() == driver.get_name() {
                return Err(DriverError::DriverExists);
            }
        }
        
        drivers.push(driver);
        self.driver_count.fetch_add(1, Ordering::AcqRel);
        Ok(())
    }
    
    /// Unregister driver plug-in
    pub fn unregister_driver(&self, name: &str) -> Result<(), DriverError> {
        let mut drivers = self.drivers.lock();
        
        let index = drivers.iter().position(|d| d.get_name() == name);
        
        if let Some(i) = index {
            drivers.remove(i);
            self.driver_count.fetch_sub(1, Ordering::AcqRel);
            Ok(())
        } else {
            Err(DriverError::DriverNotFound)
        }
    }
    
    /// RegisterDevice
    pub fn register_device(&self, device: Arc<dyn DeviceOps>) -> Result<u32, DriverError> {
        let mut devices = self.devices.lock();
        
        let device_id = self.device_count.fetch_add(1, Ordering::AcqRel);
        devices.push(device);
        
        Ok(device_id)
    }
    
    /// UnregisterDevice
    pub fn unregister_device(&self, device_id: u32) -> Result<(), DriverError> {
        let mut devices = self.devices.lock();
        
        if (device_id as usize) < devices.len() {
            devices.remove(device_id as usize);
            self.device_count.fetch_sub(1, Ordering::AcqRel);
            Ok(())
        } else {
            Err(DriverError::DeviceNotFound)
        }
    }
    
    /// Auto probe device
    pub fn probe_devices(&self) -> Result<Vec<Arc<dyn DeviceOps>>, DriverError> {
        let drivers = self.drivers.lock();
        let mut detected_devices = Vec::new();
        
        // Traverse all drivers, probe device
        for driver in drivers.iter() {
            // TODO: Implement device probe logic
            // Need to traverse system buses (PCI, USB, I2C, etc.) to probe devices
        }
        
        Ok(detected_devices)
    }
    
    /// GetDevice
    pub fn get_device(&self, device_id: u32) -> Option<Arc<dyn DeviceOps>> {
        let devices = self.devices.lock();
        
        if (device_id as usize) < devices.len() {
            Some(devices[device_id as usize].clone())
        } else {
            None
        }
    }
    
    /// Get devices by type
    pub fn get_devices_by_type(&self, device_type: DeviceType) -> Vec<Arc<dyn DeviceOps>> {
        let devices = self.devices.lock();
        
        devices.iter()
            .filter(|d| d.get_info().device_type == device_type)
            .cloned()
            .collect()
    }
    
    /// Get device count
    pub fn get_device_count(&self) -> u32 {
        self.device_count.load(Ordering::Acquire)
    }
    
    /// Get driver count
    pub fn get_driver_count(&self) -> u32 {
        self.driver_count.load(Ordering::Acquire)
    }
}

// ============================================================================
// Global Device Manager
// ============================================================================

/// Global device manager
pub static DEVICE_MANAGER: DeviceManager = DeviceManager {
    devices: SpinLock::new(Vec::new()),
    drivers: SpinLock::new(Vec::new()),
    device_count: AtomicU32::new(0),
    driver_count: AtomicU32::new(0),
};

// ============================================================================
// Helper Functions
// ============================================================================

/// Initialize driver framework
pub fn init_driver_framework() {
    // Initialize device manager
    // Register built-in drivers
    
    // TODO: Register all built-in drivers
}

/// Register vendor driver
pub fn register_vendor_driver(driver: Arc<dyn DriverPlugin>) -> Result<(), DriverError> {
    DEVICE_MANAGER.register_driver(driver)
}

/// Auto probe and load devices
pub fn auto_probe_devices() -> Result<Vec<Arc<dyn DeviceOps>>, DriverError> {
    DEVICE_MANAGER.probe_devices()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_device_type() {
        assert_eq!(DeviceType::Display as u8, 0);
        assert_eq!(DeviceType::Camera as u8, 1);
        assert_eq!(DeviceType::Bluetooth as u8, 2);
    }
    
    #[test]
    fn test_device_manager() {
        let manager = DeviceManager::new();
        assert_eq!(manager.get_device_count(), 0);
        assert_eq!(manager.get_driver_count(), 0);
    }
}