/*
 * Nuva OS - Kernel - Thermal Framework
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Thermal management framework for temperature monitoring and cooling.
 */

use crate::{pr_debug, pr_info};
use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// Thermal Zone ID
pub type ThermalZoneId = u32;

/// Cooling Device ID
pub type CoolingDeviceId = u32;

/// Temperature (in millidegrees Celsius)
pub type Temperature = i32;

/// Thermal Trip Type
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThermalTripType {
    /// Active cooling
    Active = 0,
    /// Passive cooling
    Passive = 1,
    /// Critical shutdown
    Critical = 2,
    /// Hot
    Hot = 3,
}

/// Thermal Trip Point
#[repr(C)]
pub struct ThermalTrip {
    /// Trip type
    pub trip_type: ThermalTripType,
    /// Temperature (mC)
    pub temp: Temperature,
    /// Hysteresis (mC)
    pub hysteresis: Temperature,
    /// Cooling device ID
    pub cooling_id: CoolingDeviceId,
    /// Cooling state
    pub cooling_state: u32,
}

/// Thermal Zone Mode
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThermalZoneMode {
    /// Enabled
    Enabled = 0,
    /// Disabled
    Disabled = 1,
}

/// Thermal Zone Info
#[repr(C)]
pub struct ThermalZoneInfo {
    /// Zone name
    pub name: [u8; 32],
    /// Zone ID
    pub id: ThermalZoneId,
    /// Number of trip points
    pub num_trips: u8,
    /// Trip points
    pub trips: [ThermalTrip; 8],
    /// Mode
    pub mode: ThermalZoneMode,
    /// Passive delay (ms)
    pub passive_delay: u32,
    /// Polling delay (ms)
    pub polling_delay: u32,
    /// Current temperature (mC)
    pub temperature: Temperature,
    /// Last temperature
    pub last_temp: Temperature,
    /// Trend
    pub trend: ThermalTrend,
}

/// Thermal Trend
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThermalTrend {
    /// Not available
    NotAvailable = 0,
    /// Rising
    Rising = 1,
    /// Falling
    Falling = 2,
    /// Stable
    Stable = 3,
}

/// Thermal Zone Operations
pub struct ThermalZoneOps {
    /// Get temperature
    pub get_temp: Option<unsafe extern "C" fn(*const core::ffi::c_void) -> Temperature>,
    /// Get trend
    pub get_trend: Option<unsafe extern "C" fn(*const core::ffi::c_void, i32) -> ThermalTrend>,
    /// Set trip temp
    pub set_trip_temp:
        Option<unsafe extern "C" fn(*mut core::ffi::c_void, i32, Temperature) -> i32>,
    /// Get trip temp
    pub get_trip_temp: Option<unsafe extern "C" fn(*const core::ffi::c_void, i32) -> Temperature>,
    /// Get trip hyst
    pub get_trip_hyst: Option<unsafe extern "C" fn(*const core::ffi::c_void, i32) -> Temperature>,
    /// Set trip hyst
    pub set_trip_hyst:
        Option<unsafe extern "C" fn(*mut core::ffi::c_void, i32, Temperature) -> i32>,
    /// Set emul temp
    pub set_emul_temp: Option<unsafe extern "C" fn(*mut core::ffi::c_void, Temperature) -> i32>,
    /// Notify
    pub notify: Option<unsafe extern "C" fn(*mut core::ffi::c_void, i32)>,
}

/// Cooling Device Type
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoolingType {
    /// CPU frequency
    Cpufreq = 0,
    /// CPU idle
    CpuIdle = 1,
    /// GPU frequency
    Gpufreq = 2,
    /// Fan
    Fan = 3,
    /// LED
    Led = 4,
    /// Peltier
    Peltier = 5,
    /// GPIO fan
    GpioFan = 6,
    /// Clock
    Clock = 7,
    /// Power
    Power = 8,
    /// Devfreq
    Devfreq = 9,
    /// Custom
    Custom = 255,
}

/// Cooling Device Info
#[repr(C)]
pub struct CoolingDeviceInfo {
    /// Device name
    pub name: [u8; 32],
    /// Device ID
    pub id: CoolingDeviceId,
    /// Cooling type
    pub cooling_type: CoolingType,
    /// Minimum state
    pub min_state: u32,
    /// Maximum state
    pub max_state: u32,
    /// Current state
    pub current_state: u32,
    /// Default state
    pub default_state: u32,
}

/// Cooling Device Operations
pub struct CoolingDeviceOps {
    /// Get max state
    pub get_max_state: Option<unsafe extern "C" fn(*const core::ffi::c_void) -> u32>,
    /// Get cur state
    pub get_cur_state: Option<unsafe extern "C" fn(*const core::ffi::c_void) -> u32>,
    /// Set cur state
    pub set_cur_state: Option<unsafe extern "C" fn(*mut core::ffi::c_void, u32) -> i32>,
    /// Get requested power
    pub get_requested_power:
        Option<unsafe extern "C" fn(*const core::ffi::c_void, *mut u32) -> i32>,
    /// State2power
    pub state2power: Option<unsafe extern "C" fn(*const core::ffi::c_void, u32, *mut u32) -> i32>,
    /// Power2state
    pub power2state: Option<unsafe extern "C" fn(*const core::ffi::c_void, u32, *mut u32) -> i32>,
}

/// Thermal Manager
pub struct ThermalManager {
    /// Zone count
    zone_count: AtomicU32,
    /// Cooling device count
    cooling_count: AtomicU32,
    /// Statistics
    stats: ThermalStats,
}

/// Thermal Statistics
pub struct ThermalStats {
    /// Temperature updates
    pub temp_updates: AtomicU64,
    /// Trip crossings
    pub trip_crossings: AtomicU64,
    /// Cooling events
    pub cooling_events: AtomicU64,
}

impl ThermalStats {
    pub const fn new() -> Self {
        ThermalStats {
            temp_updates: AtomicU64::new(0),
            trip_crossings: AtomicU64::new(0),
            cooling_events: AtomicU64::new(0),
        }
    }
}

impl ThermalManager {
    pub const fn new() -> Self {
        ThermalManager {
            zone_count: AtomicU32::new(0),
            cooling_count: AtomicU32::new(0),
            stats: ThermalStats::new(),
        }
    }

    /// Initialize
    pub fn init(&self) {
        log_info!("Thermal manager initialized");
    }

    /// Register thermal zone
    pub fn register_zone(&mut self) -> ThermalZoneId {
        self.zone_count.fetch_add(1, Ordering::AcqRel)
    }

    /// Register cooling device
    pub fn register_cooling(&mut self) -> CoolingDeviceId {
        self.cooling_count.fetch_add(1, Ordering::AcqRel)
    }

    /// Get temperature
    pub fn get_temp(&mut self, zone_id: ThermalZoneId) -> Temperature {
        self.stats.temp_updates.fetch_add(1, Ordering::AcqRel);
        log_debug!("thermal_get_temp: zone={}", zone_id);
        25000 // Default 25C
    }

    /// Set cooling state
    pub fn set_cooling_state(&mut self, cooling_id: CoolingDeviceId, state: u32) -> i32 {
        self.stats.cooling_events.fetch_add(1, Ordering::AcqRel);
        log_debug!("thermal_set_cooling: id={}, state={}", cooling_id, state);
        0
    }

    /// Update thermal zone
    pub fn update_zone(&mut self, zone_id: ThermalZoneId) -> i32 {
        log_debug!("thermal_update_zone: zone={}", zone_id);
        0
    }

    /// Get zone count
    pub fn get_zone_count(&self) -> u32 {
        self.zone_count.load(Ordering::Acquire)
    }

    /// Get cooling count
    pub fn get_cooling_count(&self) -> u32 {
        self.cooling_count.load(Ordering::Acquire)
    }
}

/// Global thermal manager
static THERMAL_MANAGER: core::sync::OnceLock<ThermalManager> = core::sync::OnceLock::new();

/// Get thermal manager
pub fn thermal_manager() -> &'static ThermalManager {
    THERMAL_MANAGER.get_or_init(ThermalManager::new)
}

pub fn init_thermal_manager() -> &'static ThermalManager {
    THERMAL_MANAGER.get_or_init(ThermalManager::new)
}

/// Initialize thermal manager
pub fn init_thermal_manager() {
    let mgr = thermal_manager();
    mgr.init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thermal_trip_type() {
        assert_eq!(ThermalTripType::Active as i32, 0);
        assert_eq!(ThermalTripType::Critical as i32, 2);
    }

    #[test]
    fn test_thermal_trend() {
        assert_eq!(ThermalTrend::Rising as i32, 1);
        assert_eq!(ThermalTrend::Falling as i32, 2);
    }

    #[test]
    fn test_cooling_type() {
        assert_eq!(CoolingType::Cpufreq as i32, 0);
        assert_eq!(CoolingType::Fan as i32, 3);
    }
}
