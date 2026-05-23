/*
 * Nuva OS - Kernel - Power Device Class
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Standard interface for power management devices (battery, charger, etc.).
 */

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// Power Source Type
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerSourceType {
    /// Unknown
    Unknown = 0,
    /// Battery
    Battery = 1,
    /// AC line
    AcLine = 2,
    /// USB
    Usb = 3,
    /// Wireless
    Wireless = 4,
    /// Solar
    Solar = 5,
}

/// Battery Status
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatteryStatus {
    /// Unknown
    Unknown = 0,
    /// Charging
    Charging = 1,
    /// Discharging
    Discharging = 2,
    /// Not charging
    NotCharging = 3,
    /// Full
    Full = 4,
}

/// Battery Health
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatteryHealth {
    /// Unknown
    Unknown = 0,
    /// Good
    Good = 1,
    /// Overheat
    Overheat = 2,
    /// Dead
    Dead = 3,
    /// Over voltage
    OverVoltage = 4,
    /// Unspecified failure
    UnspecifiedFailure = 5,
    /// Cold
    Cold = 6,
    /// Watchdog timer expire
    WatchdogTimer = 7,
    /// Safe mode
    SafeMode = 8,
}

/// Battery Information
#[repr(C)]
pub struct BatteryInfo {
    /// Technology (e.g., "Li-ion")
    pub technology: [u8; 16],
    /// Manufacturer
    pub manufacturer: [u8; 32],
    /// Model name
    pub model: [u8; 32],
    /// Serial number
    pub serial: [u8; 32],
    /// Design capacity (uAh)
    pub design_capacity: u32,
    /// Last full capacity (uAh)
    pub full_capacity: u32,
    /// Design voltage (uV)
    pub design_voltage: u32,
    /// Cycle count
    pub cycle_count: u32,
}

/// Battery Status Data
#[repr(C)]
pub struct BatteryStatusData {
    /// Current capacity (uAh)
    pub capacity: u32,
    /// Capacity percent (0-100)
    pub capacity_percent: u8,
    /// Current voltage (uV)
    pub voltage: u32,
    /// Current current (uA, negative = discharging)
    pub current: i32,
    /// Temperature (0.1°C)
    pub temperature: i16,
    /// Status
    pub status: BatteryStatus,
    /// Health
    pub health: BatteryHealth,
    /// Time to empty (seconds)
    pub time_to_empty: u32,
    /// Time to full (seconds)
    pub time_to_full: u32,
}

/// Charger Type
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChargerType {
    /// Unknown
    Unknown = 0,
    /// None
    None = 1,
    /// Standard USB (500mA)
    Usb = 2,
    /// USB DCP (1.5A)
    UsbDcp = 3,
    /// USB CDP (1.5A)
    UsbCdp = 4,
    /// USB ACA
    UsbAca = 5,
    /// AC adapter
    Ac = 6,
    /// Wireless
    Wireless = 7,
    /// Fast charger
    Fast = 8,
}

/// Charger Status
#[repr(C)]
pub struct ChargerStatus {
    /// Charger type
    pub charger_type: ChargerType,
    /// Online
    pub online: bool,
    /// Input current limit (uA)
    pub input_current: u32,
    /// Charging current (uA)
    pub charge_current: u32,
    /// Charging voltage (uV)
    pub charge_voltage: u32,
    /// Maximum current (uA)
    pub max_current: u32,
    /// Maximum voltage (uV)
    pub max_voltage: u32,
}

/// Power Event Type
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerEventType {
    /// AC connected
    AcConnected = 0,
    /// AC disconnected
    AcDisconnected = 1,
    /// Battery status changed
    BatteryChanged = 2,
    /// Battery low warning
    BatteryLow = 3,
    /// Battery critical
    BatteryCritical = 4,
    /// Charging started
    ChargingStarted = 5,
    /// Charging stopped
    ChargingStopped = 6,
    /// Charger type changed
    ChargerChanged = 7,
}

/// Power Event
#[repr(C)]
pub struct PowerEvent {
    /// Event type
    pub event_type: PowerEventType,
    /// Timestamp (nanoseconds)
    pub timestamp: u64,
    /// Source ID
    pub source_id: u32,
    /// Value (capacity, voltage, etc.)
    pub value: u32,
}

/// Power Device Operations
pub struct PowerDeviceOps {
    // Battery operations
    /// Get battery info
    pub get_battery_info:
        Option<unsafe extern "C" fn(*mut core::ffi::c_void, *mut BatteryInfo) -> i32>,
    /// Get battery status
    pub get_battery_status:
        Option<unsafe extern "C" fn(*mut core::ffi::c_void, *mut BatteryStatusData) -> i32>,

    // Charger operations
    /// Get charger status
    pub get_charger_status:
        Option<unsafe extern "C" fn(*mut core::ffi::c_void, *mut ChargerStatus) -> i32>,
    /// Set input current limit
    pub set_input_current: Option<unsafe extern "C" fn(*mut core::ffi::c_void, u32) -> i32>,
    /// Set charge current
    pub set_charge_current: Option<unsafe extern "C" fn(*mut core::ffi::c_void, u32) -> i32>,
    /// Set charge voltage
    pub set_charge_voltage: Option<unsafe extern "C" fn(*mut core::ffi::c_void, u32) -> i32>,
    /// Enable/disable charging
    pub set_charging: Option<unsafe extern "C" fn(*mut core::ffi::c_void, bool) -> i32>,

    // Power source operations
    /// Get power source type
    pub get_source_type: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> PowerSourceType>,
    /// Check if online
    pub is_online: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> bool>,
}

/// Power ioctl commands
pub mod power_ioctl {
    /// Get battery info
    pub const GET_BATTERY_INFO: u32 = 0xC001;
    /// Get battery status
    pub const GET_BATTERY_STATUS: u32 = 0xC002;
    /// Get charger status
    pub const GET_CHARGER_STATUS: u32 = 0xC003;
    /// Set input current limit
    pub const SET_INPUT_CURRENT: u32 = 0xC004;
    /// Set charge current
    pub const SET_CHARGE_CURRENT: u32 = 0xC005;
    /// Set charge voltage
    pub const SET_CHARGE_VOLTAGE: u32 = 0xC006;
    /// Enable charging
    pub const ENABLE_CHARGING: u32 = 0xC007;
    /// Disable charging
    pub const DISABLE_CHARGING: u32 = 0xC008;
    /// Get power source type
    pub const GET_SOURCE_TYPE: u32 = 0xC009;
    /// Check online
    pub const IS_ONLINE: u32 = 0xC00A;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_power_source_type_values() {
        assert_eq!(PowerSourceType::Battery as i32, 1);
        assert_eq!(PowerSourceType::AcLine as i32, 2);
        assert_eq!(PowerSourceType::Usb as i32, 3);
    }

    #[test]
    fn test_battery_status_values() {
        assert_eq!(BatteryStatus::Charging as i32, 1);
        assert_eq!(BatteryStatus::Discharging as i32, 2);
        assert_eq!(BatteryStatus::Full as i32, 4);
    }

    #[test]
    fn test_charger_type_values() {
        assert_eq!(ChargerType::Usb as i32, 2);
        assert_eq!(ChargerType::Ac as i32, 6);
        assert_eq!(ChargerType::Fast as i32, 8);
    }
}
