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

//! Driver Framework
//! # Device Classes
//! The driver framework supports the following device classes:
//! - Touch: Touch screen devices
//! - Audio: Audio input/output devices
//! - Sensor: Various sensors (accelerometer, gyroscope, etc.)
//! - Input: Keyboard, mouse, and other input devices
//! - Power: Battery and power management devices
//! - USB: USB and Type-C devices
//! # C Driver Integration
//! Vendor C library drivers can be integrated using the C ABI adapter:
//! ```c
//! CDriverInfo info = {
//! .name = "vendor_driver",
//! .abi_version = DDF_ABI_VERSION,
//! .ops = { ... },
//! };
//! driver_register(&info);
//! ```

// Re-export print macros from crate root
pub use crate::{pr_alert, pr_crit, pr_debug, pr_emerg, pr_err, pr_info, pr_notice, pr_warn};
pub mod block;
pub mod char;
pub mod declarative;
pub mod declarative_pm;
pub mod device;
pub mod dma;
pub mod dt;
pub mod event;
pub mod irq;
pub mod matching;

// Infrastructure modules
pub mod clk;
pub mod dmabuf;
pub mod freq;
pub mod gpio;
pub mod i2c;
pub mod icc;
pub mod input_subsys;
pub mod mfd;
pub mod opp;
pub mod phy;
pub mod pinctrl;
pub mod pm;
pub mod pwm;
pub mod regulator;
pub mod reset;
pub mod rtc;
pub mod spi;
pub mod thermal;
pub mod watchdog;

// C ABI adapter for vendor drivers
pub mod adapter;

// Device class implementations
pub mod class;

// Driver implementations
pub mod r#impl;

// Re-export main types
pub use block::{BlockDevice, BlockDeviceOps};
pub use char::{CharDevice, CharDeviceOps};
pub use device::{Device, DeviceFlags, DeviceManager, DeviceType, Driver};
pub use dma::{DmaAttr, DmaBuffer, DmaDirection, DmaManager};
pub use dt::{DeviceTree, DeviceTreeNode};
pub use event::{DeviceEvent, EventManager, EventType};
pub use irq::{IrqController as IrqChip, IrqDesc, IrqManager};

// Re-export infrastructure types
pub use clk::{ClkFlags, ClkId, ClkInfo, ClkManager, ClkOps, ClkRate, ClkType};
pub use dmabuf::{DmaBufFlags, DmaBufInfo, DmaBufManager, DmaBufSync};
pub use freq::{FreqInfo, FreqManager, FreqPolicy, FreqProfile, Frequency};
pub use gpio::{GpioChip, GpioDesc, GpioDir, GpioFlags, GpioManager, GpioValue};
pub use i2c::{I2cAdapter, I2cAddr, I2cClient, I2cFlags, I2cManager, I2cMsg};
pub use icc::{Bandwidth, IccManager, IccNode, IccPath, IccProvider};
pub use input_subsys::{InputCaps, InputDeviceInfo, InputEvent, InputSubsystem};
pub use mfd::{MfdCell, MfdDevice, MfdManager, MfdResource};
pub use opp::{Opp, OppFlags, OppLevel, OppManager, OppTable};
pub use phy::{PhyDevice, PhyInfo, PhyLink, PhyManager, PhyMode, PhyState};
pub use pinctrl::{PinConfig, PinFunction, PinGroup, PinState, PinctrlDev, PinctrlManager};
pub use pm::{DevicePm, PmEvent, PmFlags, PmManager, PmState, RuntimePm};
pub use pwm::{PwmChip, PwmDevice, PwmManager, PwmPolarity, PwmState};
pub use regulator::{
    RegulatorDesc, RegulatorDev, RegulatorManager, RegulatorMode, RegulatorStatus,
};
pub use reset::{ResetControl, ResetController, ResetFlags, ResetManager, ResetStatus};
pub use rtc::{RtcAlarm, RtcDevice, RtcFeatures, RtcManager, RtcTime};
pub use spi::{SpiController, SpiDevice, SpiFlags, SpiManager, SpiMode, SpiTransfer};
pub use thermal::{CoolingDeviceInfo, Temperature, ThermalManager, ThermalTrip, ThermalZoneInfo};
pub use watchdog::{WatchdogDevice, WatchdogInfo, WatchdogManager, WatchdogOptions};

// Re-export C ABI types
pub use adapter::{
    CCallbackTable, CDeviceClass, CDeviceContext, CDriverAdapter, CDriverInfo, CDriverOps,
    DDF_ABI_VERSION,
};

// Re-export device class types
// Input devices
pub use class::camera::{
    CameraBuffer, CameraControl, CameraDeviceOps, CameraFormatDesc, CameraPixelFormat,
};
pub use class::input::{InputCapabilities, InputDeviceOps, InputEvent as ClassInputEvent, KeyCode};
pub use class::sensor::{SensorConfig, SensorDeviceOps, SensorEvent, SensorType};
pub use class::touch::{TouchConfig, TouchDeviceOps, TouchEvent, TouchPoint};

// Output devices
pub use class::backlight::{BacklightDeviceOps, BacklightProps, BacklightState, BacklightType};
pub use class::display::{
    ConnectorType, DisplayBuffer, DisplayDeviceOps, DisplayInfo, DisplayMode, PixelFormat,
};
pub use class::led::{LedBlink, LedColor, LedDeviceOps, LedInfo, LedState, LedType};
pub use class::vibrator::{
    VibratorDeviceOps, VibratorEffect, VibratorInfo, VibratorPattern, VibratorState,
};

// Audio devices
pub use class::audio::{AudioBuffer, AudioDeviceOps, AudioFormat, AudioStreamConfig};

// Power devices
pub use class::power::{BatteryInfo, BatteryStatusData, ChargerStatus, PowerDeviceOps};

// Storage devices
pub use class::eeprom::{EepromDeviceOps, EepromFlags, EepromInfo, EepromRegion, EepromType};
pub use class::storage::{
    StorageDeviceOps, StorageInfo, StoragePartition, StorageRequest, StorageState, StorageType,
};

// Bus devices
pub use class::usb::{TypeCPortStatus, UsbDevice, UsbDeviceOps, UsbTransfer};

// Wireless devices
pub use class::bluetooth::{BtAddress, BtAdvParams, BtConnection, BtDeviceOps, BtState};
pub use class::nfc::{NfcData, NfcDeviceOps, NfcProtocol, NfcSeInfo, NfcState, NfcTarget};
pub use class::wifi::{
    WifiBssid, WifiConnectParams, WifiDeviceOps, WifiMode, WifiScanResult, WifiSsid, WifiState,
};

/// Driver error type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriverError {
    /// Device not found
    DeviceNotFound,
    /// Driver not found
    DriverNotFound,
    /// No matching driver
    NoMatchingDriver,
    /// Initialization failed
    InitFailed,
    /// Device is busy
    DeviceBusy,
    /// Invalid argument
    InvalidArgument,
    /// Out of memory
    NoMemory,
    /// Permission denied
    PermissionDenied,
    /// Operation not supported
    NotSupported,
    /// I/O error
    IoError,
}

/// Power state enumeration
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerState {
    /// Device is on
    On = 0,
    /// Device is sleeping
    Sleep = 1,
    /// Device is suspended
    Suspend = 2,
    /// Device is off
    Off = 3,
}

impl Default for PowerState {
    fn default() -> Self {
        Self::On
    }
}
