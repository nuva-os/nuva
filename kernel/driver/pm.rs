/*
 * Nuva OS - Kernel - Driver - Pm
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
 * Nuva OS - Kernel - Device Power Management
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Power management framework for device drivers.
 */

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

use super::{Device, PowerState};
use crate::pr_info;

/// PM Event Type
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PmEvent {
    /// System suspend
    Suspend = 0,
    /// System resume
    Resume = 1,
    /// Device freeze (suspend without wakeup)
    Freeze = 2,
    /// Device thaw (resume from freeze)
    Thaw = 3,
    /// Device power off
    PowerOff = 4,
    /// Device power on
    PowerOn = 5,
    /// Runtime suspend
    RuntimeSuspend = 6,
    /// Runtime resume
    RuntimeResume = 7,
    /// Runtime idle
    RuntimeIdle = 8,
}

/// PM State
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PmState {
    /// Active
    Active = 0,
    /// Idle
    Idle = 1,
    /// Suspended
    Suspended = 2,
    /// Frozen
    Frozen = 3,
    /// Power off
    Off = 4,
}

/// PM Flags
bitflags::bitflags! {
    #[repr(transparent)]
    pub struct PmFlags: u32 {
        /// Device can wake system
        const WAKEUP = 1 << 0;
        /// Device is wakeup source
        const WAKEUP_SOURCE = 1 << 1;
        /// Runtime PM enabled
        const RUNTIME_PM = 1 << 2;
        /// Auto suspend enabled
        const AUTO_SUSPEND = 1 << 3;
        /// No runtime suspend
        const NO_RUNTIME_SUSPEND = 1 << 4;
        /// No runtime resume
        const NO_RUNTIME_RESUME = 1 << 5;
        /// Smart suspend
        const SMART_SUSPEND = 1 << 6;
        /// May skip resume
        const MAY_SKIP_RESUME = 1 << 7;
    }
}

/// PM Operations
pub struct PmOps {
    /// Prepare for suspend
    pub prepare: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    /// Suspend device
    pub suspend: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    /// Suspend late
    pub suspend_late: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    /// Suspend noirq
    pub suspend_noirq: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,

    /// Resume noirq
    pub resume_noirq: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    /// Resume early
    pub resume_early: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    /// Resume device
    pub resume: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    /// Complete resume
    pub complete: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,

    /// Freeze
    pub freeze: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    /// Thaw
    pub thaw: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    /// Power off
    pub poweroff: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    /// Power on
    pub poweron: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,

    /// Runtime suspend
    pub runtime_suspend: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    /// Runtime resume
    pub runtime_resume: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    /// Runtime idle
    pub runtime_idle: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
}

/// PM Domain
pub struct PmDomain {
    /// Domain name
    pub name: [u8; 32],
    /// Domain ID
    pub domain_id: u32,
    /// Power on
    pub power_on: Option<unsafe extern "C" fn(u32) -> i32>,
    /// Power off
    pub power_off: Option<unsafe extern "C" fn(u32) -> i32>,
    /// Suspend
    pub suspend: Option<unsafe extern "C" fn(u32) -> i32>,
    /// Resume
    pub resume: Option<unsafe extern "C" fn(u32) -> i32>,
    /// Device count
    pub device_count: AtomicU32,
}

/// Runtime PM Data
pub struct RuntimePm {
    /// Current state
    pub state: AtomicU32,
    /// Status flags
    pub status: AtomicU32,
    /// Usage count
    pub usage_count: AtomicU32,
    /// Child count
    pub child_count: AtomicU32,
    /// Disable depth
    pub disable_depth: AtomicU32,
    /// Autosuspend delay (ms)
    pub autosuspend_delay: AtomicU32,
    /// Last busy timestamp
    pub last_busy: AtomicU64,
    /// Request pending
    pub request_pending: AtomicU32,
}

impl RuntimePm {
    pub const fn new() -> Self {
        RuntimePm {
            state: AtomicU32::new(PmState::Active as u32),
            status: AtomicU32::new(0),
            usage_count: AtomicU32::new(0),
            child_count: AtomicU32::new(0),
            disable_depth: AtomicU32::new(0),
            autosuspend_delay: AtomicU32::new(2000),
            last_busy: AtomicU64::new(0),
            request_pending: AtomicU32::new(0),
        }
    }

    /// Get usage count
    pub fn get_usage_count(&self) -> u32 {
        self.usage_count.load(Ordering::Acquire)
    }

    /// Increment usage count
    pub fn get(&self) -> u32 {
        self.usage_count.fetch_add(1, Ordering::AcqRel) + 1
    }

    /// Decrement usage count
    pub fn put(&self) -> u32 {
        self.usage_count.fetch_sub(1, Ordering::AcqRel) - 1
    }

    /// Set autosuspend delay
    pub fn set_autosuspend_delay(&self, delay_ms: u32) {
        self.autosuspend_delay.store(delay_ms, Ordering::Release);
    }

    /// Mark last busy
    pub fn mark_last_busy(&self, timestamp: u64) {
        self.last_busy.store(timestamp, Ordering::Release);
    }

    /// Check if active
    pub fn is_active(&self) -> bool {
        self.state.load(Ordering::Acquire) == PmState::Active as u32
    }

    /// Check if suspended
    pub fn is_suspended(&self) -> bool {
        self.state.load(Ordering::Acquire) == PmState::Suspended as u32
    }
}

/// PM Statistics
pub struct PmStats {
    /// Suspend count
    pub suspend_count: AtomicU64,
    /// Resume count
    pub resume_count: AtomicU64,
    /// Runtime suspend count
    pub runtime_suspend_count: AtomicU64,
    /// Runtime resume count
    pub runtime_resume_count: AtomicU64,
    /// Total suspend time (ns)
    pub suspend_time_ns: AtomicU64,
    /// Total resume time (ns)
    pub resume_time_ns: AtomicU64,
    /// Last suspend time
    pub last_suspend: AtomicU64,
    /// Last resume time
    pub last_resume: AtomicU64,
}

impl PmStats {
    pub const fn new() -> Self {
        PmStats {
            suspend_count: AtomicU64::new(0),
            resume_count: AtomicU64::new(0),
            runtime_suspend_count: AtomicU64::new(0),
            runtime_resume_count: AtomicU64::new(0),
            suspend_time_ns: AtomicU64::new(0),
            resume_time_ns: AtomicU64::new(0),
            last_suspend: AtomicU64::new(0),
            last_resume: AtomicU64::new(0),
        }
    }
}

/// Device Power Management
pub struct DevicePm {
    /// PM operations
    pub ops: PmOps,
    /// Runtime PM
    pub runtime: RuntimePm,
    /// PM flags
    pub flags: AtomicU32,
    /// PM domain
    pub domain_id: u32,
    /// Wakeup IRQ
    pub wakeup_irq: u32,
    /// Statistics
    pub stats: PmStats,
}

impl DevicePm {
    pub fn new() -> Self {
        DevicePm {
            ops: PmOps {
                prepare: None,
                suspend: None,
                suspend_late: None,
                suspend_noirq: None,
                resume_noirq: None,
                resume_early: None,
                resume: None,
                complete: None,
                freeze: None,
                thaw: None,
                poweroff: None,
                poweron: None,
                runtime_suspend: None,
                runtime_resume: None,
                runtime_idle: None,
            },
            runtime: RuntimePm::new(),
            flags: AtomicU32::new(0),
            domain_id: 0,
            wakeup_irq: 0,
            stats: PmStats::new(),
        }
    }

    /// Suspend device
    pub fn suspend(&mut self, dev: *mut core::ffi::c_void) -> i32 {
        self.stats.suspend_count.fetch_add(1, Ordering::AcqRel);

        if let Some(suspend_fn) = self.ops.suspend {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe { suspend_fn(dev) }
        } else {
            0
        }
    }

    /// Resume device
    pub fn resume(&mut self, dev: *mut core::ffi::c_void) -> i32 {
        self.stats.resume_count.fetch_add(1, Ordering::AcqRel);

        if let Some(resume_fn) = self.ops.resume {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe { resume_fn(dev) }
        } else {
            0
        }
    }

    /// Runtime suspend
    pub fn runtime_suspend(&mut self, dev: *mut core::ffi::c_void) -> i32 {
        self.stats
            .runtime_suspend_count
            .fetch_add(1, Ordering::AcqRel);
        self.runtime
            .state
            .store(PmState::Suspended as u32, Ordering::Release);

        if let Some(suspend_fn) = self.ops.runtime_suspend {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe { suspend_fn(dev) }
        } else {
            0
        }
    }

    /// Runtime resume
    pub fn runtime_resume(&mut self, dev: *mut core::ffi::c_void) -> i32 {
        self.stats
            .runtime_resume_count
            .fetch_add(1, Ordering::AcqRel);
        self.runtime
            .state
            .store(PmState::Active as u32, Ordering::Release);

        if let Some(resume_fn) = self.ops.runtime_resume {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe { resume_fn(dev) }
        } else {
            0
        }
    }

    /// Enable wakeup
    pub fn enable_wakeup(&mut self) {
        self.flags
            .fetch_or(PmFlags::WAKEUP.bits(), Ordering::AcqRel);
    }

    /// Disable wakeup
    pub fn disable_wakeup(&mut self) {
        self.flags
            .fetch_and(!PmFlags::WAKEUP.bits(), Ordering::AcqRel);
    }

    /// Check if wakeup enabled
    pub fn is_wakeup_enabled(&self) -> bool {
        (self.flags.load(Ordering::Acquire) & PmFlags::WAKEUP.bits()) != 0
    }
}

/// PM Manager
pub struct PmManager {
    /// System state
    system_state: AtomicU32,
    /// Suspend count
    suspend_count: AtomicU32,
    /// PM domains
    domain_count: AtomicU32,
    /// Statistics
    stats: PmStats,
}

impl PmManager {
    pub const fn new() -> Self {
        PmManager {
            system_state: AtomicU32::new(PmState::Active as u32),
            suspend_count: AtomicU32::new(0),
            domain_count: AtomicU32::new(0),
            stats: PmStats::new(),
        }
    }

    /// Initialize
    pub fn init(&self) {
        log_info!("PM manager initialized");
    }

    /// System suspend
    pub fn system_suspend(&mut self) -> i32 {
        self.suspend_count.fetch_add(1, Ordering::AcqRel);
        self.system_state
            .store(PmState::Suspended as u32, Ordering::Release);
        log_info!("System suspending...");
        0
    }

    /// System resume
    pub fn system_resume(&mut self) -> i32 {
        self.system_state
            .store(PmState::Active as u32, Ordering::Release);
        log_info!("System resuming...");
        0
    }

    /// Get system state
    pub fn get_system_state(&self) -> PmState {
        match self.system_state.load(Ordering::Acquire) {
            0 => PmState::Active,
            1 => PmState::Idle,
            2 => PmState::Suspended,
            3 => PmState::Frozen,
            4 => PmState::Off,
            _ => PmState::Active,
        }
    }
}

/// Global PM manager
static PM_MANAGER: core::sync::OnceLock<PmManager> = core::sync::OnceLock::new();

/// Get PM manager
pub fn pm_manager() -> &'static PmManager {
    PM_MANAGER.get_or_init(PmManager::new)
}

pub fn init_pm_manager() -> &'static PmManager {
    PM_MANAGER.get_or_init(PmManager::new)
}

/// Initialize PM manager
pub fn init_pm_manager() {
    let mgr = pm_manager();
    mgr.init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pm_event_values() {
        assert_eq!(PmEvent::Suspend as i32, 0);
        assert_eq!(PmEvent::Resume as i32, 1);
        assert_eq!(PmEvent::RuntimeSuspend as i32, 6);
    }

    #[test]
    fn test_pm_state_values() {
        assert_eq!(PmState::Active as i32, 0);
        assert_eq!(PmState::Suspended as i32, 2);
    }

    #[test]
    fn test_runtime_pm() {
        let rpm = RuntimePm::new();
        assert!(rpm.is_active());
        assert_eq!(rpm.get_usage_count(), 0);

        let count = rpm.get();
        assert_eq!(count, 1);
        assert_eq!(rpm.get_usage_count(), 1);

        let count = rpm.put();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_pm_flags() {
        let flags = PmFlags::WAKEUP | PmFlags::RUNTIME_PM;
        assert!(flags.contains(PmFlags::WAKEUP));
        assert!(flags.contains(PmFlags::RUNTIME_PM));
    }
}
