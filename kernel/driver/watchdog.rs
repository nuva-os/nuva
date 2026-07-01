/*
 * Nuva OS - Kernel - Driver - Watchdog
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
 * Nuva OS - Kernel - Watchdog Framework
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Watchdog timer framework for system monitoring.
 */

use crate::{pr_debug, pr_info};
use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// Watchdog ID
pub type WatchdogId = u32;

/// Watchdog Status
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WatchdogStatus {
    /// Unknown
    Unknown = 0,
    /// Active
    Active = 1,
    /// Inactive
    Inactive = 2,
}

/// Watchdog Info
#[repr(C)]
pub struct WatchdogInfo {
    /// Driver name
    pub name: [u8; 32],
    /// Firmware version
    pub firmware_version: u32,
    /// Identity string
    pub identity: [u8; 32],
    /// Options
    pub options: WatchdogOptions,
    /// Timeout range (min, max) in seconds
    pub min_timeout: u32,
    pub max_timeout: u32,
    /// Current timeout
    pub timeout: u32,
    /// Pretimeout
    pub pretimeout: u32,
    /// Time left
    pub time_left: u32,
    /// Status
    pub status: WatchdogStatus,
    /// Boot status
    pub boot_status: u32,
}

/// Watchdog Options
bitflags::bitflags! {
    #[repr(transparent)]
    pub struct WatchdogOptions: u32 {
        /// Disable if explicitly closed
        const DISCLOSENABLE = 1 << 0;
        /// Keep alive ping
        const KEEPALIVEPING = 1 << 1;
        /// Magic close character
        const MAGICCLOSE = 1 << 2;
        /// Set timeout
        const SETTIMEOUT = 1 << 3;
        /// Get timeout
        const GETTIMEOUT = 1 << 4;
        /// Set pretimeout
        const SETPRETIMEOUT = 1 << 5;
        /// Get pretimeout
        const GETPRETIMEOUT = 1 << 6;
        /// Get time left
        const GETTIMELEFT = 1 << 7;
        /// Get boot status
        const GETBOOTSTATUS = 1 << 8;
        /// No way out
        const NO_WAY_OUT = 1 << 9;
        /// Externally active
        const EXTERN = 1 << 10;
        /// Card reset
        const CARDRESET = 1 << 11;
        /// Power under voltage
        const POWEROVER = 1 << 12;
        /// Power over voltage
        const POWERUNDER = 1 << 13;
        /// Overheat
        const OVERHEAT = 1 << 14;
    }
}

/// Watchdog Operations
pub struct WatchdogOps {
    /// Start watchdog
    pub start: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    /// Stop watchdog
    pub stop: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    /// Ping (keep alive)
    pub ping: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    /// Set timeout
    pub set_timeout: Option<unsafe extern "C" fn(*mut core::ffi::c_void, u32) -> i32>,
    /// Get timeout
    pub get_timeout: Option<unsafe extern "C" fn(*const core::ffi::c_void) -> u32>,
    /// Set pretimeout
    pub set_pretimeout: Option<unsafe extern "C" fn(*mut core::ffi::c_void, u32) -> i32>,
    /// Get pretimeout
    pub get_pretimeout: Option<unsafe extern "C" fn(*const core::ffi::c_void) -> u32>,
    /// Get time left
    pub get_time_left: Option<unsafe extern "C" fn(*const core::ffi::c_void) -> u32>,
    /// Get boot status
    pub get_boot_status: Option<unsafe extern "C" fn(*const core::ffi::c_void) -> u32>,
    /// Restart
    pub restart: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    /// Refuse unregister
    pub refuse_unregister: Option<unsafe extern "C" fn(*const core::ffi::c_void) -> bool>,
}

/// Watchdog Device
pub struct WatchdogDevice {
    /// Device name
    pub name: [u8; 32],
    /// Watchdog ID
    pub id: WatchdogId,
    /// Operations
    pub ops: WatchdogOps,
    /// Private data
    pub data: *mut core::ffi::c_void,
    /// Parent device
    pub parent: u32,
    /// Info
    pub info: WatchdogInfo,
    /// Timeout
    pub timeout: AtomicU32,
    /// Pretimeout
    pub pretimeout: AtomicU32,
    /// Last keepalive
    pub last_keepalive: AtomicU64,
    /// Status
    pub status: AtomicU32,
    /// Open count
    pub open_count: AtomicU32,
}

/// Watchdog Manager
pub struct WatchdogManager {
    /// Watchdog count
    wdt_count: AtomicU32,
    /// Statistics
    stats: WatchdogStats,
}

/// Watchdog Statistics
pub struct WatchdogStats {
    /// Start count
    pub start_count: AtomicU64,
    /// Stop count
    pub stop_count: AtomicU64,
    /// Ping count
    pub ping_count: AtomicU64,
    /// Timeout count
    pub timeout_count: AtomicU64,
}

impl WatchdogStats {
    pub const fn new() -> Self {
        WatchdogStats {
            start_count: AtomicU64::new(0),
            stop_count: AtomicU64::new(0),
            ping_count: AtomicU64::new(0),
            timeout_count: AtomicU64::new(0),
        }
    }
}

impl WatchdogManager {
    pub const fn new() -> Self {
        WatchdogManager {
            wdt_count: AtomicU32::new(0),
            stats: WatchdogStats::new(),
        }
    }

    /// Initialize
    pub fn init(&self) {
        log_info!("Watchdog manager initialized");
    }

    /// Register watchdog
    pub fn register(&mut self, _wdt: &WatchdogDevice) -> WatchdogId {
        let id = self.wdt_count.fetch_add(1, Ordering::AcqRel);
        id
    }

    /// Start watchdog
    pub fn start(&mut self, wdt_id: WatchdogId) -> i32 {
        self.stats.start_count.fetch_add(1, Ordering::AcqRel);
        log_debug!("watchdog_start: id={}", wdt_id);
        0
    }

    /// Stop watchdog
    pub fn stop(&mut self, wdt_id: WatchdogId) -> i32 {
        self.stats.stop_count.fetch_add(1, Ordering::AcqRel);
        log_debug!("watchdog_stop: id={}", wdt_id);
        0
    }

    /// Ping watchdog
    pub fn ping(&mut self, wdt_id: WatchdogId) -> i32 {
        self.stats.ping_count.fetch_add(1, Ordering::AcqRel);
        log_debug!("watchdog_ping: id={}", wdt_id);
        0
    }

    /// Set timeout
    pub fn set_timeout(&mut self, wdt_id: WatchdogId, timeout: u32) -> i32 {
        self.stats.timeout_count.fetch_add(1, Ordering::AcqRel);
        log_debug!("watchdog_set_timeout: id={}, timeout={}", wdt_id, timeout);
        0
    }

    /// Get timeout
    pub fn get_timeout(&self, wdt_id: WatchdogId) -> u32 {
        log_debug!("watchdog_get_timeout: id={}", wdt_id);
        30 // Default 30 seconds
    }

    /// Get time left
    pub fn get_time_left(&self, wdt_id: WatchdogId) -> u32 {
        log_debug!("watchdog_get_time_left: id={}", wdt_id);
        30
    }

    /// Get info
    pub fn get_info(&self, wdt_id: WatchdogId) -> WatchdogInfo {
        WatchdogInfo {
            name: [0; 32],
            firmware_version: 0,
            identity: [0; 32],
            options: WatchdogOptions::SETTIMEOUT | WatchdogOptions::KEEPALIVEPING,
            min_timeout: 1,
            max_timeout: 65535,
            timeout: 30,
            pretimeout: 0,
            time_left: 30,
            status: WatchdogStatus::Inactive,
            boot_status: 0,
        }
    }
}

/// Global watchdog manager
static WATCHDOG_MANAGER: crate::sync_oncelock::OnceLock<WatchdogManager> = crate::sync_oncelock::OnceLock::new();

/// Get watchdog manager
pub fn watchdog_manager() -> &'static WatchdogManager {
    WATCHDOG_MANAGER.get_or_init(WatchdogManager::new)
}

/// Initialize watchdog manager
pub fn init_watchdog_manager() {
    let mgr = watchdog_manager();
    mgr.init();
}

// Convenience functions

/// Start watchdog
pub fn watchdog_start(wdt_id: WatchdogId) -> i32 {
    watchdog_manager().start(wdt_id)
}

/// Stop watchdog
pub fn watchdog_stop(wdt_id: WatchdogId) -> i32 {
    watchdog_manager().stop(wdt_id)
}

/// Ping watchdog
pub fn watchdog_ping(wdt_id: WatchdogId) -> i32 {
    watchdog_manager().ping(wdt_id)
}

/// Set timeout
pub fn watchdog_set_timeout(wdt_id: WatchdogId, timeout: u32) -> i32 {
    watchdog_manager().set_timeout(wdt_id, timeout)
}

/// Get timeout
pub fn watchdog_get_timeout(wdt_id: WatchdogId) -> u32 {
    watchdog_manager().get_timeout(wdt_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_watchdog_status() {
        assert_eq!(WatchdogStatus::Active as i32, 1);
        assert_eq!(WatchdogStatus::Inactive as i32, 2);
    }

    #[test]
    fn test_watchdog_options() {
        let opts = WatchdogOptions::SETTIMEOUT | WatchdogOptions::KEEPALIVEPING;
        assert!(opts.contains(WatchdogOptions::SETTIMEOUT));
        assert!(opts.contains(WatchdogOptions::KEEPALIVEPING));
    }
}
