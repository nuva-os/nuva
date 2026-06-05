/*
 * Nuva OS - Kernel - Driver - Freq
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
 * Nuva OS - Kernel - Frequency Framework
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Frequency scaling framework for DVFS (Dynamic Voltage Frequency Scaling).
 */

use crate::{pr_debug, pr_info};
use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// Freq Device ID
pub type FreqDeviceId = u32;

/// Frequency (Hz)
pub type Frequency = u64;

/// Freq Profile
#[repr(C)]
pub struct FreqProfile {
    /// Profile name
    pub name: [u8; 32],
    /// Target frequency callback
    pub target: Option<unsafe extern "C" fn(*mut core::ffi::c_void, Frequency) -> i32>,
    /// Get frequency callback
    pub get: Option<unsafe extern "C" fn(*const core::ffi::c_void) -> Frequency>,
    /// Get max frequency
    pub get_max: Option<unsafe extern "C" fn(*const core::ffi::c_void) -> Frequency>,
    /// Get min frequency
    pub get_min: Option<unsafe extern "C" fn(*const core::ffi::c_void) -> Frequency>,
    /// Suspend
    pub suspend: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    /// Resume
    pub resume: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    /// Exit
    pub exit: Option<unsafe extern "C" fn(*mut core::ffi::c_void)>,
    /// Driver data
    pub driver_data: *mut core::ffi::c_void,
}

/// Freq Info
#[repr(C)]
pub struct FreqInfo {
    /// Device name
    pub name: [u8; 32],
    /// Device ID
    pub id: FreqDeviceId,
    /// Current frequency (Hz)
    pub cur_freq: Frequency,
    /// Minimum frequency (Hz)
    pub min_freq: Frequency,
    /// Maximum frequency (Hz)
    pub max_freq: Frequency,
    /// Policy
    pub policy: FreqPolicy,
    /// Number of frequencies
    pub num_freqs: u32,
    /// Transition latency (ns)
    pub transition_latency: u64,
    /// Flags
    pub flags: FreqFlags,
}

/// Freq Policy
#[repr(C)]
pub struct FreqPolicy {
    /// Policy name
    pub name: [u8; 16],
    /// Minimum frequency
    pub min: Frequency,
    /// Maximum frequency
    pub max: Frequency,
    /// Current frequency
    pub cur: Frequency,
    /// Governor name
    pub governor: [u8; 16],
    /// Policy flags
    pub flags: u32,
    /// CPU mask (for CPU freq)
    pub cpus: u64,
}

/// Freq Flags
bitflags::bitflags! {
    #[repr(transparent)]
    pub struct FreqFlags: u32 {
        /// Need update limits
        const NEED_UPDATE_LIMITS = 1 << 0;
        /// Fast switch possible
        const FAST_SWITCH = 1 << 1;
        /// Async notification
        const ASYNC_NOTIFICATION = 1 << 2;
        /// Shared type any
        const SHARED_TYPE_ANY = 1 << 3;
        /// Shared type all
        const SHARED_TYPE_ALL = 1 << 4;
        /// Inconsistent freq
        const INCONSISTENT_FREQ = 1 << 5;
        /// Have governor per policy
        const HAVE_GOVERNOR_PER_POLICY = 1 << 6;
        /// Is slow path
        const IS_SLOW_PATH = 1 << 7;
    }
}

/// Freq Governor
pub struct FreqGovernor {
    /// Governor name
    pub name: [u8; 16],
    /// Governor ID
    pub id: u32,
    /// Init
    pub init: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    /// Exit
    pub exit: Option<unsafe extern "C" fn(*mut core::ffi::c_void)>,
    /// Start
    pub start: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    /// Stop
    pub stop: Option<unsafe extern "C" fn(*mut core::ffi::c_void)>,
    /// Limits
    pub limits: Option<unsafe extern "C" fn(*mut core::ffi::c_void)>,
    /// Owner
    pub owner: u32,
    /// Flags
    pub flags: GovernorFlags,
}

/// Governor Flags
bitflags::bitflags! {
    #[repr(transparent)]
    pub struct GovernorFlags: u32 {
        /// Dynamic sampling
        const DYNAMIC_SAMPLING = 1 << 0;
        /// Need update
        const NEED_UPDATE = 1 << 1;
    }
}

/// Freq Stats
#[repr(C)]
pub struct FreqStats {
    /// Frequency table
    pub freq_table: [Frequency; 16],
    /// Time in state (ms)
    pub time_in_state: [u64; 16],
    /// Number of frequencies
    pub num_freqs: u8,
    /// Last index
    pub last_index: u8,
    /// Last time
    pub last_time: u64,
    /// Total transitions
    pub total_trans: u64,
}

/// Freq Manager
pub struct FreqManager {
    /// Device count
    dev_count: AtomicU32,
    /// Governor count
    gov_count: AtomicU32,
    /// Statistics
    stats: FreqMgrStats,
}

/// Freq Manager Statistics
pub struct FreqMgrStats {
    /// Frequency changes
    pub freq_changes: AtomicU64,
    /// Governor changes
    pub gov_changes: AtomicU64,
    /// Devices registered
    pub devices: AtomicU64,
}

impl FreqMgrStats {
    pub const fn new() -> Self {
        FreqMgrStats {
            freq_changes: AtomicU64::new(0),
            gov_changes: AtomicU64::new(0),
            devices: AtomicU64::new(0),
        }
    }
}

impl FreqManager {
    pub const fn new() -> Self {
        FreqManager {
            dev_count: AtomicU32::new(0),
            gov_count: AtomicU32::new(0),
            stats: FreqMgrStats::new(),
        }
    }

    /// Initialize
    pub fn init(&self) {
        log_info!("Freq manager initialized");
    }

    /// Register device
    pub fn register_device(&mut self, _profile: &FreqProfile) -> FreqDeviceId {
        self.stats.devices.fetch_add(1, Ordering::AcqRel);
        self.dev_count.fetch_add(1, Ordering::AcqRel)
    }

    /// Unregister device
    pub fn unregister_device(&mut self, dev_id: FreqDeviceId) {
        log_debug!("freq_unregister: id={}", dev_id);
    }

    /// Register governor
    pub fn register_governor(&mut self, _gov: &FreqGovernor) -> u32 {
        self.gov_count.fetch_add(1, Ordering::AcqRel)
    }

    /// Set frequency
    pub fn set_freq(&mut self, dev_id: FreqDeviceId, freq: Frequency) -> i32 {
        self.stats.freq_changes.fetch_add(1, Ordering::AcqRel);
        log_debug!("freq_set: id={}, freq={}", dev_id, freq);
        0
    }

    /// Get frequency
    pub fn get_freq(&self, dev_id: FreqDeviceId) -> Frequency {
        log_debug!("freq_get: id={}", dev_id);
        0
    }

    /// Set governor
    pub fn set_governor(&mut self, dev_id: FreqDeviceId, gov_name: &[u8]) -> i32 {
        self.stats.gov_changes.fetch_add(1, Ordering::AcqRel);
        log_debug!("freq_set_governor: id={}, gov={:?}", dev_id, gov_name);
        0
    }

    /// Get info
    pub fn get_info(&self, dev_id: FreqDeviceId) -> FreqInfo {
        FreqInfo {
            name: [0; 32],
            id: dev_id,
            cur_freq: 0,
            min_freq: 0,
            max_freq: 0,
            policy: FreqPolicy {
                name: [0; 16],
                min: 0,
                max: 0,
                cur: 0,
                governor: [0; 16],
                flags: 0,
                cpus: 0,
            },
            num_freqs: 0,
            transition_latency: 0,
            flags: FreqFlags::empty(),
        }
    }

    /// Suspend
    pub fn suspend(&mut self, dev_id: FreqDeviceId) -> i32 {
        log_debug!("freq_suspend: id={}", dev_id);
        0
    }

    /// Resume
    pub fn resume(&mut self, dev_id: FreqDeviceId) -> i32 {
        log_debug!("freq_resume: id={}", dev_id);
        0
    }
}

/// Global freq manager
static FREQ_MANAGER: core::sync::OnceLock<FreqManager> = core::sync::OnceLock::new();

/// Get freq manager
pub fn freq_manager() -> &'static FreqManager {
    FREQ_MANAGER.get_or_init(FreqManager::new)
}

pub fn init_freq_manager() -> &'static FreqManager {
    FREQ_MANAGER.get_or_init(FreqManager::new)
}

/// Initialize freq manager
pub fn init_freq_manager() {
    let mgr = freq_manager();
    mgr.init();
}

// Convenience functions

/// Set frequency
pub fn freq_set(dev_id: FreqDeviceId, freq: Frequency) -> i32 {
    freq_manager().set_freq(dev_id, freq)
}

/// Get frequency
pub fn freq_get(dev_id: FreqDeviceId) -> Frequency {
    freq_manager().get_freq(dev_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_freq_flags() {
        let flags = FreqFlags::FAST_SWITCH | FreqFlags::ASYNC_NOTIFICATION;
        assert!(flags.contains(FreqFlags::FAST_SWITCH));
        assert!(flags.contains(FreqFlags::ASYNC_NOTIFICATION));
    }

    #[test]
    fn test_governor_flags() {
        let flags = GovernorFlags::DYNAMIC_SAMPLING;
        assert!(flags.contains(GovernorFlags::DYNAMIC_SAMPLING));
    }
}
