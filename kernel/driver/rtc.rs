/*
 * Nuva OS - Kernel - Driver - Rtc
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
 * Nuva OS - Kernel - RTC Framework
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Real-Time Clock framework for time keeping.
 */

use crate::{pr_debug, pr_info};
use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// RTC ID
pub type RtcId = u32;

/// RTC Time
#[repr(C)]
pub struct RtcTime {
    /// Seconds (0-59)
    pub sec: u8,
    /// Minutes (0-59)
    pub min: u8,
    /// Hours (0-23)
    pub hour: u8,
    /// Day of month (1-31)
    pub mday: u8,
    /// Month (1-12)
    pub mon: u8,
    /// Year (0-99, relative to 1900 or 2000)
    pub year: u8,
    /// Day of week (0-6, Sunday=0)
    pub wday: u8,
    /// Day of year (1-366)
    pub yday: u16,
    /// Is daylight saving time
    pub isdst: i8,
}

impl RtcTime {
    /// Create from Unix timestamp
    pub fn from_unix(ts: u64) -> Self {
        // Simplified conversion
        let secs = ts % 86400;
        let days = ts / 86400;

        RtcTime {
            sec: (secs % 60) as u8,
            min: ((secs / 60) % 60) as u8,
            hour: ((secs / 3600) % 24) as u8,
            mday: ((days % 31) + 1) as u8,
            mon: (((days / 31) % 12) + 1) as u8,
            year: (((days / 365) % 100) as u8),
            wday: ((days + 4) % 7) as u8,
            yday: (days % 366) as u16,
            isdst: 0,
        }
    }

    /// Convert to Unix timestamp
    pub fn to_unix(&self) -> u64 {
        let mut ts = 0u64;
        ts += self.sec as u64;
        ts += (self.min as u64) * 60;
        ts += (self.hour as u64) * 3600;
        ts += (self.yday as u64) * 86400;
        ts += (self.year as u64) * 365 * 86400;
        ts
    }
}

/// RTC Alarm
#[repr(C)]
pub struct RtcAlarm {
    /// Alarm ID
    pub id: u32,
    /// Alarm time
    pub time: RtcTime,
    /// Enabled
    pub enabled: bool,
    /// Pending
    pub pending: bool,
    /// Alarm type
    pub alarm_type: RtcAlarmType,
}

/// RTC Alarm Type
bitflags::bitflags! {
    #[repr(transparent)]
    pub struct RtcAlarmType: u32 {
        /// Time match
        const TIME = 1 << 0;
        /// Wake alarm
        const WAKE = 1 << 1;
        /// Auto increment
        const AUTO = 1 << 2;
    }
}

/// RTC Features
bitflags::bitflags! {
    #[repr(transparent)]
    pub struct RtcFeatures: u32 {
        /// Read time
        const READ_TIME = 1 << 0;
        /// Set time
        const SET_TIME = 1 << 1;
        /// Read alarm
        const READ_ALARM = 1 << 2;
        /// Set alarm
        const SET_ALARM = 1 << 3;
        /// Alarm interrupt
        const ALARM_INT = 1 << 4;
        /// Update interrupt
        const UPDATE_INT = 1 << 5;
        /// Wake alarm
        const WAKE_ALARM = 1 << 6;
        /// NVRAM
        const NVRAM = 1 << 7;
        /// Battery backup
        const BATTERY = 1 << 8;
    }
}

/// RTC Info
#[repr(C)]
pub struct RtcInfo {
    /// RTC name
    pub name: [u8; 32],
    /// Features
    pub features: RtcFeatures,
    /// Maximum alarm count
    pub max_alarms: u8,
    /// NVRAM size
    pub nvram_size: u16,
    /// Range max (Unix timestamp)
    pub range_max: u64,
    /// Alarm offset resolution (seconds)
    pub alarm_offset_resolution: u32,
}

/// RTC Operations
pub struct RtcOps {
    /// Read time
    pub read_time: Option<unsafe extern "C" fn(*const core::ffi::c_void, *mut RtcTime) -> i32>,
    /// Set time
    pub set_time: Option<unsafe extern "C" fn(*mut core::ffi::c_void, *const RtcTime) -> i32>,
    /// Read alarm
    pub read_alarm:
        Option<unsafe extern "C" fn(*const core::ffi::c_void, u32, *mut RtcAlarm) -> i32>,
    /// Set alarm
    pub set_alarm: Option<unsafe extern "C" fn(*mut core::ffi::c_void, *const RtcAlarm) -> i32>,
    /// Alarm enable
    pub alarm_irq_enable: Option<unsafe extern "C" fn(*mut core::ffi::c_void, u32, bool) -> i32>,
    /// Read callback
    pub read_callback: Option<unsafe extern "C" fn(*const core::ffi::c_void) -> u32>,
    /// Set callback
    pub set_callback: Option<unsafe extern "C" fn(*mut core::ffi::c_void, u32) -> i32>,
    /// NVRAM read
    pub nvram_read:
        Option<unsafe extern "C" fn(*const core::ffi::c_void, *mut u8, usize, usize) -> i32>,
    /// NVRAM write
    pub nvram_write:
        Option<unsafe extern "C" fn(*mut core::ffi::c_void, *const u8, usize, usize) -> i32>,
}

/// RTC Device
pub struct RtcDevice {
    /// Device name
    pub name: [u8; 32],
    /// RTC ID
    pub id: RtcId,
    /// Operations
    pub ops: RtcOps,
    /// Private data
    pub data: *mut core::ffi::c_void,
    /// Parent device
    pub parent: u32,
    /// Info
    pub info: RtcInfo,
    /// Open count
    pub open_count: AtomicU32,
    /// IRQ
    pub irq: i32,
}

/// RTC Manager
pub struct RtcManager {
    /// RTC count
    rtc_count: AtomicU32,
    /// Statistics
    stats: RtcStats,
}

/// RTC Statistics
pub struct RtcStats {
    /// Read count
    pub read_count: AtomicU64,
    /// Write count
    pub write_count: AtomicU64,
    /// Alarm count
    pub alarm_count: AtomicU64,
}

impl RtcStats {
    pub const fn new() -> Self {
        RtcStats {
            read_count: AtomicU64::new(0),
            write_count: AtomicU64::new(0),
            alarm_count: AtomicU64::new(0),
        }
    }
}

impl RtcManager {
    pub const fn new() -> Self {
        RtcManager {
            rtc_count: AtomicU32::new(0),
            stats: RtcStats::new(),
        }
    }

    /// Initialize
    pub fn init(&self) {
        log_info!("RTC manager initialized");
    }

    /// Register RTC
    pub fn register(&mut self, _rtc: &RtcDevice) -> RtcId {
        let id = self.rtc_count.fetch_add(1, Ordering::AcqRel);
        id
    }

    /// Read time
    pub fn read_time(&mut self, rtc_id: RtcId) -> RtcTime {
        self.stats.read_count.fetch_add(1, Ordering::AcqRel);
        log_debug!("rtc_read_time: id={}", rtc_id);
        RtcTime::from_unix(0)
    }

    /// Set time
    pub fn set_time(&mut self, rtc_id: RtcId, time: &RtcTime) -> i32 {
        self.stats.write_count.fetch_add(1, Ordering::AcqRel);
        log_debug!("rtc_set_time: id={}", rtc_id);
        0
    }

    /// Read alarm
    pub fn read_alarm(&mut self, rtc_id: RtcId, alarm_id: u32) -> RtcAlarm {
        log_debug!("rtc_read_alarm: id={}, alarm={}", rtc_id, alarm_id);
        RtcAlarm {
            id: alarm_id,
            time: RtcTime::from_unix(0),
            enabled: false,
            pending: false,
            alarm_type: RtcAlarmType::TIME,
        }
    }

    /// Set alarm
    pub fn set_alarm(&mut self, rtc_id: RtcId, alarm: &RtcAlarm) -> i32 {
        self.stats.alarm_count.fetch_add(1, Ordering::AcqRel);
        log_debug!("rtc_set_alarm: id={}, alarm={}", rtc_id, alarm.id);
        0
    }

    /// Get info
    pub fn get_info(&self, rtc_id: RtcId) -> RtcInfo {
        RtcInfo {
            name: [0; 32],
            features: RtcFeatures::READ_TIME | RtcFeatures::SET_TIME,
            max_alarms: 1,
            nvram_size: 0,
            range_max: u64::MAX,
            alarm_offset_resolution: 1,
        }
    }
}

/// Global RTC manager
static RTC_MANAGER: core::sync::OnceLock<RtcManager> = core::sync::OnceLock::new();

/// Get RTC manager
pub fn rtc_manager() -> &'static RtcManager {
    RTC_MANAGER.get_or_init(RtcManager::new)
}

pub fn init_rtc_manager() -> &'static RtcManager {
    RTC_MANAGER.get_or_init(RtcManager::new)
}

/// Initialize RTC manager
pub fn init_rtc_manager() {
    let mgr = rtc_manager();
    mgr.init();
}

// Convenience functions

/// Read RTC time
pub fn rtc_read_time(rtc_id: RtcId) -> RtcTime {
    rtc_manager().read_time(rtc_id)
}

/// Set RTC time
pub fn rtc_set_time(rtc_id: RtcId, time: &RtcTime) -> i32 {
    rtc_manager().set_time(rtc_id, time)
}

/// Read RTC alarm
pub fn rtc_read_alarm(rtc_id: RtcId, alarm_id: u32) -> RtcAlarm {
    rtc_manager().read_alarm(rtc_id, alarm_id)
}

/// Set RTC alarm
pub fn rtc_set_alarm(rtc_id: RtcId, alarm: &RtcAlarm) -> i32 {
    rtc_manager().set_alarm(rtc_id, alarm)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rtc_time() {
        let time = RtcTime::from_unix(0);
        assert_eq!(time.sec, 0);
        assert_eq!(time.min, 0);
        assert_eq!(time.hour, 0);
    }

    #[test]
    fn test_rtc_features() {
        let features = RtcFeatures::READ_TIME | RtcFeatures::SET_TIME;
        assert!(features.contains(RtcFeatures::READ_TIME));
        assert!(features.contains(RtcFeatures::SET_TIME));
    }

    #[test]
    fn test_rtc_alarm_type() {
        let alarm_type = RtcAlarmType::TIME | RtcAlarmType::WAKE;
        assert!(alarm_type.contains(RtcAlarmType::TIME));
        assert!(alarm_type.contains(RtcAlarmType::WAKE));
    }
}
