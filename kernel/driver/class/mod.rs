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

// Nuva OS - Kernel - Device Class Module
// Standard device class implementations.
// # Input Devices
// - touch: Touch screen devices
// - input: Keyboard, mouse, and other input devices
// - sensor: Various sensors (accelerometer, gyroscope, etc.)
// - camera: Camera/video capture devices
// # Output Devices
// - display: Display/graphics devices
// - led: LED indicator devices
// - backlight: Backlight devices
// - vibrator: Vibrator/haptic devices
// # Audio Devices
// - audio: Audio input/output devices
// # Power Devices
// - power: Battery and power management devices
// # Storage Devices
// - storage: eMMC, SD, NVMe, UFS, etc.
// - eeprom: EEPROM/Flash devices
// # Bus Devices
// - usb: USB and Type-C devices
// # Wireless Devices
// - bluetooth: Bluetooth devices
// - wifi: WiFi network devices
// - nfc: NFC devices

// Input devices
// Re-export print macros from crate root
pub use crate::{pr_alert, pr_crit, pr_debug, pr_emerg, pr_err, pr_info, pr_notice, pr_warn};
pub mod camera;
pub mod input;
pub mod sensor;
pub mod touch;

// Output devices
pub mod backlight;
pub mod display;
pub mod led;
pub mod vibrator;

// Audio devices
pub mod audio;

// Power devices
pub mod power;

// Storage devices
pub mod eeprom;
pub mod storage;

// Bus devices
pub mod usb;

// Wireless devices
pub mod bluetooth;
pub mod nfc;
pub mod wifi;

// Re-export input device classes
pub use camera::{
    CameraBuffer, CameraControl, CameraDeviceOps, CameraFormatDesc, CameraPixelFormat,
};
pub use input::{InputCapabilities, InputDeviceOps, InputEvent, KeyCode};
pub use sensor::{SensorConfig, SensorDeviceOps, SensorEvent, SensorType};
pub use touch::{TouchConfig, TouchDeviceOps, TouchEvent, TouchPoint};

// Re-export output device classes
pub use backlight::{BacklightDeviceOps, BacklightProps, BacklightState, BacklightType};
pub use display::{
    ConnectorType, DisplayBuffer, DisplayDeviceOps, DisplayInfo, DisplayMode, PixelFormat,
};
pub use led::{LedBlink, LedColor, LedDeviceOps, LedInfo, LedState, LedType};
pub use vibrator::{
    VibratorDeviceOps, VibratorEffect, VibratorInfo, VibratorPattern, VibratorState,
};

// Re-export audio device classes
pub use audio::{AudioBuffer, AudioDeviceOps, AudioFormat, AudioStreamConfig};

// Re-export power device classes
pub use power::{BatteryInfo, BatteryStatusData, ChargerStatus, PowerDeviceOps};

// Re-export storage device classes
pub use eeprom::{EepromDeviceOps, EepromFlags, EepromInfo, EepromRegion, EepromType};
pub use storage::{
    StorageDeviceOps, StorageInfo, StoragePartition, StorageRequest, StorageState, StorageType,
};

// Re-export bus device classes
pub use usb::{TypeCPortStatus, UsbDevice, UsbDeviceOps, UsbTransfer};

// Re-export wireless device classes
pub use bluetooth::{BtAddress, BtAdvParams, BtConnection, BtDeviceOps, BtState};
pub use nfc::{NfcData, NfcDeviceOps, NfcProtocol, NfcSeInfo, NfcState, NfcTarget};
pub use wifi::{
    WifiBssid, WifiConnectParams, WifiDeviceOps, WifiMode, WifiScanResult, WifiSsid, WifiState,
};
