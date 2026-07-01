/*
 * Nuva OS - Kernel - Device - DriverPlugin
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
use crate::{pr_info};
/*
 * Nuva OS - Kernel - Driver Plugin Interface
 * 
 * Copyright (C) 2026 Nuva OS Team
 * 
 * Driver plugin interface for device adaptation.
 */

use core::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use crate::kernel::plugin::{Plugin, PluginType, PluginFlags, PluginOps, PluginInfo};
use crate::kernel::plugin::core::PluginMeta;

use crate::syslib::posix::errno::Errno;
use crate::kernel::error::Errno;
/// Device Match Type
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceMatchType {
    /// Device tree compatible
    OfDevice = 0,
    /// PCI vendor/device ID
    PciDevice = 1,
    /// USB vendor/product ID
    UsbDevice = 2,
    /// I2C device ID
    I2cDevice = 3,
    /// SPI device ID
    SpiDevice = 4,
    /// Platform device name
    PlatformDevice = 5,
    /// ACPI device
    AcpiDevice = 6,
    /// Custom match
    Custom = 7,
}

/// Device Match Entry
#[repr(C)]
pub struct DeviceMatchEntry {
    /// Match type
    pub match_type: DeviceMatchType,
    /// Compatible string (for OF)
    pub compatible: [u8; 128],
    /// Vendor ID (for PCI/USB)
    pub vendor: u16,
    /// Device ID (for PCI/USB)
    pub device: u16,
    /// Subsystem vendor
    pub subvendor: u16,
    /// Subsystem device
    pub subdevice: u16,
    /// Class mask
    pub class_mask: u32,
    /// Driver data
    pub driver_data: u64,
    /// Next entry
    pub next: *mut DeviceMatchEntry,
}

/// Device Match Table
pub struct DeviceMatchTable {
    /// Table name
    pub name: [u8; 64],
    /// Entries
    pub entries: *mut DeviceMatchEntry,
    /// Entry count
    pub count: u32,
}

/// Driver Operations
pub struct DriverOps {
    /// Probe device
    pub probe: Option<unsafe extern "C" fn(*mut DriverPlugin, *mut core::ffi::c_void) -> i32>,
    /// Remove device
    pub remove: Option<unsafe extern "C" fn(*mut DriverPlugin, *mut core::ffi::c_void) -> i32>,
    /// Suspend device
    pub suspend: Option<unsafe extern "C" fn(*mut DriverPlugin, *mut core::ffi::c_void) -> i32>,
    /// Resume device
    pub resume: Option<unsafe extern "C" fn(*mut DriverPlugin, *mut core::ffi::c_void) -> i32>,
    /// Shutdown device
    pub shutdown: Option<unsafe extern "C" fn(*mut DriverPlugin, *mut core::ffi::c_void)>,
    /// Reset device
    pub reset: Option<unsafe extern "C" fn(*mut DriverPlugin, *mut core::ffi::c_void) -> i32>,
}

/// Driver Capabilities
bitflags::bitflags! {
    #[repr(transparent)]
    pub struct DriverCaps: u64 {
        /// DMA support
        const DMA = 1 << 0;
        /// Interrupt support
        const IRQ = 1 << 1;
        /// Power management
        const PM = 1 << 2;
        /// Hotplug
        const HOTPLUG = 1 << 3;
        /// System sleep
        const SYSTEM_SLEEP = 1 << 4;
        /// Runtime PM
        const RUNTIME_PM = 1 << 5;
        /// Async probe
        const ASYNC_PROBE = 1 << 6;
        /// Multi-instance
        const MULTI_INSTANCE = 1 << 7;
        /// Shared IRQ
        const SHARED_IRQ = 1 << 8;
        /// MSI
        const MSI = 1 << 9;
        /// MSI-X
        const MSI_X = 1 << 10;
        /// IOMMU
        const IOMMU = 1 << 11;
    }
}

/// Driver Plugin
pub struct DriverPlugin {
    /// Base plugin
    pub base: PluginMeta,
    /// Match table
    pub match_table: DeviceMatchTable,
    /// Driver operations
    pub driver_ops: DriverOps,
    /// Capabilities
    pub caps: DriverCaps,
    /// Bound devices
    pub devices: *mut BoundDevice,
    /// Device count
    pub dev_count: AtomicU32,
    /// Max devices
    pub max_devices: u32,
    /// Auto-bind
    pub auto_bind: AtomicBool,
}

/// Bound Device
pub struct BoundDevice {
    /// Device pointer
    pub device: *mut core::ffi::c_void,
    /// Match entry
    pub match_entry: *mut DeviceMatchEntry,
    /// Driver data
    pub driver_data: u64,
    /// State
    pub state: AtomicU32,
    /// Next
    pub next: *mut BoundDevice,
}

/// Device State
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceState {
    Unbound = 0,
    Probing = 1,
    Bound = 2,
    Failed = 3,
    Removing = 4,
}

impl DriverPlugin {
    pub fn new(name: &[u8]) -> Self {
        let mut name_arr = [0u8; 64];
        let len = name.len().min(63);
        name_arr[..len].copy_from_slice(&name[..len]);
        
        DriverPlugin {
            base: PluginMeta::new(0, core::str::from_utf8(name).unwrap_or("")),
            match_table: DeviceMatchTable {
                name: name_arr,
                entries: core::ptr::null_mut(),
                count: 0,
            },
            driver_ops: DriverOps {
                probe: None,
                remove: None,
                suspend: None,
                resume: None,
                shutdown: None,
                reset: None,
            },
            caps: DriverCaps::empty(),
            devices: core::ptr::null_mut(),
            dev_count: AtomicU32::new(0),
            max_devices: u32::MAX,
            auto_bind: AtomicBool::new(true),
        }
    }
    
    /// Add match entry
    pub fn add_match(&mut self, entry: *mut DeviceMatchEntry) {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            (*entry).next = self.match_table.entries;
            self.match_table.entries = entry;
        }
        self.match_table.count += 1;
    }
    
    /// Match device
    pub fn match_device(&self, device: *const core::ffi::c_void) -> Option<*mut DeviceMatchEntry> {
        let mut entry = self.match_table.entries;
        
        while !entry.is_null() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                if self.match_entry(entry, device) {
                    return Some(entry);
                }
                entry = (*entry).next;
            }
        }
        
        None
    }
    
    /// Match single entry
    fn match_entry(&self, entry: *const DeviceMatchEntry, device: *const core::ffi::c_void) -> bool {
        // TODO: Implement device matching based on device type
        let _ = (entry, device);
        false
    }
    
    /// Probe device
    pub fn probe_device(&mut self, device: *mut core::ffi::c_void) -> i32 {
        // Check if already bound
        if self.find_device(device).is_some() {
            return Errno::Ebusy.to_ret_i32(); // EBUSY
        }
        
        // Check max devices
        if self.dev_count.load(Ordering::Acquire) >= self.max_devices {
            return Errno::Enomem.to_ret_i32(); // ENOMEM
        }
        
        // Match device
        let match_entry = match self.match_device(device) {
            Some(e) => e,
            None => return Errno::Enodev.to_ret_i32(), // ENODEV
        };
        
        // Call probe
        if let Some(probe) = self.driver_ops.probe {
            // SAFETY: unsafe block required for low-level memory or hardware access
            let ret = unsafe { probe(self as *mut DriverPlugin, device) };
            if ret != 0 {
                return ret;
            }
        }
        
        // Add to bound devices
        let bound = BoundDevice {
            device,
            match_entry,
            // SAFETY: unsafe block required for low-level memory or hardware access
            driver_data: unsafe { (*match_entry).driver_data },
            state: AtomicU32::new(DeviceState::Bound as u32),
            next: self.devices,
        };
        
        // TODO: Allocate and add bound device
        
        self.dev_count.fetch_add(1, Ordering::AcqRel);
        0
    }
    
    /// Remove device
    pub fn remove_device(&mut self, device: *mut core::ffi::c_void) -> i32 {
        let bound = match self.find_device(device) {
            Some(d) => d,
            None => return Errno::Enodev.to_ret_i32(), // ENODEV
        };
        
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            (*bound).state.store(DeviceState::Removing as u32, Ordering::Release);
            
            if let Some(remove) = self.driver_ops.remove {
                let ret = remove(self as *mut DriverPlugin, device);
                if ret != 0 {
                    (*bound).state.store(DeviceState::Bound as u32, Ordering::Release);
                    return ret;
                }
            }
        }
        
        // Remove from list
        // TODO: Remove bound device
        
        self.dev_count.fetch_sub(1, Ordering::AcqRel);
        0
    }
    
    /// Find bound device
    fn find_device(&self, device: *mut core::ffi::c_void) -> Option<*mut BoundDevice> {
        let mut bound = self.devices;
        
        while !bound.is_null() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                if (*bound).device == device {
                    return Some(bound);
                }
                bound = (*bound).next;
            }
        }
        
        None
    }
    
    /// Suspend all devices
    pub fn suspend_all(&mut self) -> i32 {
        let mut bound = self.devices;
        
        while !bound.is_null() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                if let Some(suspend) = self.driver_ops.suspend {
                    let ret = suspend(self as *mut DriverPlugin, (*bound).device);
                    if ret != 0 {
                        return ret;
                    }
                }
                bound = (*bound).next;
            }
        }
        
        0
    }
    
    /// Resume all devices
    pub fn resume_all(&mut self) -> i32 {
        let mut bound = self.devices;
        
        while !bound.is_null() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                if let Some(resume) = self.driver_ops.resume {
                    let ret = resume(self as *mut DriverPlugin, (*bound).device);
                    if ret != 0 {
                        return ret;
                    }
                }
                bound = (*bound).next;
            }
        }
        
        0
    }
    
    /// Shutdown all devices
    pub fn shutdown_all(&mut self) {
        let mut bound = self.devices;
        
        while !bound.is_null() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                if let Some(shutdown) = self.driver_ops.shutdown {
                    shutdown(self as *mut DriverPlugin, (*bound).device);
                }
                bound = (*bound).next;
            }
        }
    }
}

/// Driver Plugin Builder
pub struct DriverPluginBuilder {
    name: [u8; 64],
    name_len: usize,
    caps: DriverCaps,
    max_devices: u32,
    auto_bind: bool,
    flags: PluginFlags,
}

impl DriverPluginBuilder {
    pub fn new(name: &[u8]) -> Self {
        let mut name_arr = [0u8; 64];
        let len = name.len().min(63);
        name_arr[..len].copy_from_slice(&name[..len]);
        
        DriverPluginBuilder {
            name: name_arr,
            name_len: len,
            caps: DriverCaps::empty(),
            max_devices: u32::MAX,
            auto_bind: true,
            flags: PluginFlags::AUTO_LOAD | PluginFlags::AUTO_ACTIVATE,
        }
    }
    
    pub fn with_caps(mut self, caps: DriverCaps) -> Self {
        self.caps = caps;
        self
    }
    
    pub fn with_max_devices(mut self, max: u32) -> Self {
        self.max_devices = max;
        self
    }
    
    pub fn with_auto_bind(mut self, auto: bool) -> Self {
        self.auto_bind = auto;
        self
    }
    
    pub fn with_flags(mut self, flags: PluginFlags) -> Self {
        self.flags = flags;
        self
    }
    
    pub fn build(self) -> DriverPlugin {
        let mut plugin = DriverPlugin::new(&self.name[..self.name_len]);
        plugin.caps = self.caps;
        plugin.max_devices = self.max_devices;
        plugin.auto_bind.store(self.auto_bind, Ordering::Release);
        plugin
    }
}

/// Driver Plugin Registry
pub struct DriverPluginRegistry {
    /// Drivers
    pub drivers: *mut DriverPlugin,
    /// Driver count
    pub count: AtomicU32,
}

impl DriverPluginRegistry {
    pub const fn new() -> Self {
        DriverPluginRegistry {
            drivers: core::ptr::null_mut(),
            count: AtomicU32::new(0),
        }
    }
    
    /// Register driver
    pub fn register(&mut self, driver: *mut DriverPlugin) -> i32 {
        if driver.is_null() {
            return Errno::Einval.to_ret_i32();
        }
        
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            (*driver).base.next = (*self.drivers).base.next;
            self.drivers = driver;
        }
        
        self.count.fetch_add(1, Ordering::AcqRel);
        0
    }
    
    /// Find driver for device
    pub fn find_driver(&self, device: *const core::ffi::c_void) -> Option<*mut DriverPlugin> {
        let mut driver = self.drivers;
        
        while !driver.is_null() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                if (*driver).match_device(device).is_some() {
                    return Some(driver);
                }
                driver = (*driver).base.next as *mut DriverPlugin;
            }
        }
        
        None
    }
    
    /// Probe device with all drivers
    pub fn probe_device(&mut self, device: *mut core::ffi::c_void) -> i32 {
        if let Some(driver) = self.find_driver(device) {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe { (*driver).probe_device(device) }
        } else {
            -19 // ENODEV
        }
    }
}

/// Global driver registry
static DRIVER_REGISTRY: crate::sync_oncelock::OnceLock<DriverPluginRegistry> = crate::sync_oncelock::OnceLock::new();

/// Get driver registry
pub fn driver_registry() -> &'static DriverPluginRegistry {
    DRIVER_REGISTRY.get_or_init(DriverPluginRegistry::new)
}

pub fn init_driver_registry() -> &'static DriverPluginRegistry {
    DRIVER_REGISTRY.get_or_init(DriverPluginRegistry::new)
}

/// Initialize driver plugin system
pub fn init_driver_plugin() {
    let reg = driver_registry();
    let _ = reg;
    log_info!("Driver plugin system initialized");
}

// Convenience macros

/// Define driver plugin
#[macro_export]
macro_rules! define_driver_plugin {
    ($name:ident, $probe:expr, $remove:expr) => {
        static mut $name: $crate::kernel::plugin::driver_plugin::DriverPlugin = {
            let mut driver = $crate::kernel::plugin::driver_plugin::DriverPlugin::new(
                stringify!($name).as_bytes(),
            );
            driver.driver_ops.probe = Some($probe);
            driver.driver_ops.remove = Some($remove);
            driver
        };
    };
}

/// Add OF match
#[macro_export]
macro_rules! driver_of_match {
    ($driver:ident, $compatible:expr, $data:expr) => {
        {
            static mut MATCH_ENTRY: $crate::kernel::plugin::driver_plugin::DeviceMatchEntry = {
                let mut entry = $crate::kernel::plugin::driver_plugin::DeviceMatchEntry {
                    match_type: $crate::kernel::plugin::driver_plugin::DeviceMatchType::OfDevice,
                    compatible: [0; 128],
                    vendor: 0,
                    device: 0,
                    subvendor: 0,
                    subdevice: 0,
                    class_mask: 0,
                    driver_data: $data,
                    next: core::ptr::null_mut(),
                };
                let compat = $compatible.as_bytes();
                let len = compat.len().min(127);
                entry.compatible[..len].copy_from_slice(&compat[..len]);
                entry
            };
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe { $driver.add_match(&mut MATCH_ENTRY); }
        }
    };
}

/// Add PCI match
#[macro_export]
macro_rules! driver_pci_match {
    ($driver:ident, $vendor:expr, $device:expr, $data:expr) => {
        {
            static mut MATCH_ENTRY: $crate::kernel::plugin::driver_plugin::DeviceMatchEntry = {
                $crate::kernel::plugin::driver_plugin::DeviceMatchEntry {
                    match_type: $crate::kernel::plugin::driver_plugin::DeviceMatchType::PciDevice,
                    compatible: [0; 128],
                    vendor: $vendor,
                    device: $device,
                    subvendor: 0xFFFF,
                    subdevice: 0xFFFF,
                    class_mask: 0,
                    driver_data: $data,
                    next: core::ptr::null_mut(),
                }
            };
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe { $driver.add_match(&mut MATCH_ENTRY); }
        }
    };
}
