/*
 * Nuva OS - HAL - Power
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



use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use crate::{pr_debug, pr_info, pr_warn, pr_err};

macro_rules! check_ret {
    ($expr:expr) => {
        { let r = $expr; if r != 0 { return r; } }
    };
}

/// Suspend state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SuspendState {
    /// Running
    Running = 0,
    /// Freeze
    Freeze = 1,
    /// Standby
    Standby = 2,
    /// Suspend to RAM
    SuspendToRam = 3,
    /// Suspend to disk
    SuspendToDisk = 4,
}

/// Wakeup source
#[derive(Debug, Clone, Copy)]
pub enum WakeupSource {
    /// Power button
    PowerButton = 0,
    /// RTC alarm
    RtcAlarm = 1,
    /// Touch screen
    Touchscreen = 2,
    /// GPIO
    Gpio = 3,
    /// USB
    Usb = 4,
    /// Network
    Network = 5,
}

// ============================================================================
// Suspend callbacks
// ============================================================================

/// Suspend callback phase
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SuspendPhase {
    /// Prepare suspend
    Prepare = 0,
    /// Enter suspend
    Enter = 1,
    /// Exit suspend
    Exit = 2,
    /// Complete resume
    Complete = 3,
}

/// Suspend callback function type
pub type SuspendCallback = fn(phase: SuspendPhase) -> i32;

/// Suspend callback item
pub struct SuspendCallbackItem {
    /// Callback function
    pub callback: SuspendCallback,
    /// Priority (smaller values execute first)
    pub priority: i32,
    /// Name
    pub name: &'static str,
    /// Next
    pub next: *mut SuspendCallbackItem,
}

impl SuspendCallbackItem {
    pub const fn new(name: &'static str, callback: SuspendCallback, priority: i32) -> Self {
        SuspendCallbackItem {
            callback,
            priority,
            name,
            next: core::ptr::null_mut(),
        }
    }
}

// ============================================================================
// Device power management
// ============================================================================

/// Device power state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DevicePowerState {
    /// D0: Fully on
    D0 = 0,
    /// D1: Partially on
    D1 = 1,
    /// D2: Low power
    D2 = 2,
    /// D3hot: Off but powered
    D3hot = 3,
    /// D3cold: Completely off
    D3cold = 4,
}

/// Device power operations
pub struct DevicePowerOps {
    /// Prepare suspend
    pub prepare: fn() -> i32,
    /// Enter suspend
    pub suspend: fn(state: DevicePowerState) -> i32,
    /// Exit suspend
    pub resume: fn() -> i32,
    /// Complete resume
    pub complete: fn() -> i32,
    /// Runtime suspend
    pub runtime_suspend: fn() -> i32,
    /// Runtime resume
    pub runtime_resume: fn() -> i32,
}

/// Device power management item
pub struct DevicePowerItem {
    /// Device name
    pub name: &'static str,
    /// Power operations
    pub ops: &'static DevicePowerOps,
    /// Current state
    pub state: AtomicU32,
    /// If suspend disabled
    pub no_suspend: bool,
    /// If runtime suspend disabled
    pub no_runtime_suspend: bool,
    /// Runtime usage count
    pub usage_count: AtomicU32,
    /// Runtime child device count
    pub child_count: AtomicU32,
    /// Next device
    pub next: *mut DevicePowerItem,
}

impl DevicePowerItem {
    pub const fn new(name: &'static str, ops: &'static DevicePowerOps) -> Self {
        DevicePowerItem {
            name,
            ops,
            state: AtomicU32::new(DevicePowerState::D0 as u32),
            no_suspend: false,
            no_runtime_suspend: false,
            usage_count: AtomicU32::new(1),
            child_count: AtomicU32::new(0),
            next: core::ptr::null_mut(),
        }
    }

    /// Suspend device
    pub fn suspend(&self, state: DevicePowerState) -> i32 {
        if self.no_suspend {
            return 0;
        }

        let result = (self.ops.suspend)(state);
        if result == 0 {
            self.state.store(state as u32, Ordering::Release);
        }
        result
    }

    /// Resume device
    pub fn resume(&self) -> i32 {
        let result = (self.ops.resume)();
        if result == 0 {
            self.state.store(DevicePowerState::D0 as u32, Ordering::Release);
        }
        result
    }

    /// Runtime suspend
    pub fn runtime_suspend(&self) -> i32 {
        if self.no_runtime_suspend {
            return 0;
        }

        if self.usage_count.load(Ordering::Acquire) > 0 {
            return -1;  // Device in use
        }

        if self.child_count.load(Ordering::Acquire) > 0 {
            return -1;  // Child device active
        }

        let result = (self.ops.runtime_suspend)();
        if result == 0 {
            self.state.store(DevicePowerState::D3hot as u32, Ordering::Release);
        }
        result
    }

    /// Runtime resume
    pub fn runtime_resume(&self) -> i32 {
        let result = (self.ops.runtime_resume)();
        if result == 0 {
            self.state.store(DevicePowerState::D0 as u32, Ordering::Release);
        }
        result
    }

    /// Increment usage count
    pub fn get(&self) {
        self.usage_count.fetch_add(1, Ordering::AcqRel);
    }

    /// Decrement usage count
    pub fn put(&self) {
        self.usage_count.fetch_sub(1, Ordering::AcqRel);
    }
}

// ============================================================================
// Suspend manager
// ============================================================================

/// Suspend manager
pub struct SuspendManager {
    /// Current state
    current_state: AtomicU32,
    /// Wakeup source mask
    wakeup_mask: AtomicU32,
    /// Suspend count
    suspend_count: AtomicU32,
    /// Device list
    devices: *mut DevicePowerItem,
    /// Callback list
    callbacks: *mut SuspendCallbackItem,
    /// Suspend image address
    suspend_image_addr: AtomicU64,
    /// Suspend image size
    suspend_image_size: AtomicU64,
    /// Statistics
    stats: SuspendStats,
}

/// Suspend statistics
pub struct SuspendStats {
    /// Successful suspend count
    pub success: AtomicU64,
    /// Failed suspend count
    pub fail: AtomicU64,
    /// Total suspend time (milliseconds)
    pub total_time: AtomicU64,
    /// Last suspend time
    pub last_time: AtomicU64,
}

impl SuspendStats {
    pub const fn new() -> Self {
        SuspendStats {
            success: AtomicU64::new(0),
            fail: AtomicU64::new(0),
            total_time: AtomicU64::new(0),
            last_time: AtomicU64::new(0),
        }
    }
}

impl SuspendManager {
    pub const fn new() -> Self {
        SuspendManager {
            current_state: AtomicU32::new(SuspendState::Running as u32),
            wakeup_mask: AtomicU32::new(0),
            suspend_count: AtomicU32::new(0),
            devices: core::ptr::null_mut(),
            callbacks: core::ptr::null_mut(),
            suspend_image_addr: AtomicU64::new(0),
            suspend_image_size: AtomicU64::new(0),
            stats: SuspendStats::new(),
        }
    }

    /// Register device
    pub fn register_device(&mut self, device: *mut DevicePowerItem) {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            (*device).next = self.devices;
            self.devices = device;
        }
    }

    /// Register callback
    pub fn register_callback(&mut self, callback: *mut SuspendCallbackItem) {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            (*callback).next = self.callbacks;
            self.callbacks = callback;
        }
    }

    /// Enter suspend
    pub fn suspend(&mut self, state: SuspendState) -> i32 {
        if self.current_state.load(Ordering::Acquire) != SuspendState::Running as u32 {
            return -1;
        }

        log_info!("Entering suspend state: {:?}", state);

        // Save current state
        self.current_state.store(state as u32, Ordering::Release);
        self.suspend_count.fetch_add(1, Ordering::Relaxed);

        // Execute suspend flow
        let result = match state {
            SuspendState::Freeze => self.enter_freeze(),
            SuspendState::Standby => self.enter_standby(),
            SuspendState::SuspendToRam => self.enter_str(),
            SuspendState::SuspendToDisk => self.enter_std(),
            SuspendState::Running => 0,
        };

        if result == 0 {
            self.stats.success.fetch_add(1, Ordering::AcqRel);
        } else {
            self.stats.fail.fetch_add(1, Ordering::AcqRel);
            self.current_state.store(SuspendState::Running as u32, Ordering::Release);
        }

        result
    }

    /// Resume
    pub fn resume(&mut self) -> i32 {
        let state = match self.current_state.load(Ordering::Acquire) {
            0 => SuspendState::Running,
            1 => SuspendState::Freeze,
            2 => SuspendState::Standby,
            3 => SuspendState::SuspendToRam,
            4 => SuspendState::SuspendToDisk,
            _ => SuspendState::Running,
        };

        if state == SuspendState::Running {
            return 0;
        }

        log_info!("Resuming from suspend state: {:?}", state);

        // Execute resume flow
        let result = match state {
            SuspendState::Freeze => self.exit_freeze(),
            SuspendState::Standby => self.exit_standby(),
            SuspendState::SuspendToRam => self.exit_str(),
            SuspendState::SuspendToDisk => self.exit_std(),
            SuspendState::Running => 0,
        };

        // Restore state
        self.current_state.store(SuspendState::Running as u32, Ordering::Release);

        result
    }

    /// Execute callbacks
    fn run_callbacks(&mut self, phase: SuspendPhase) -> i32 {
        let mut current = self.callbacks;

        while !current.is_null() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                let result = ((*current).callback)(phase);
                if result < 0 {
                    log_error!("Suspend callback '{}' failed: {}", (*current).name, result);
                    return result;
                }
                current = (*current).next;
            }
        }

        0
    }

    /// Suspend all devices
    fn suspend_devices(&mut self, state: DevicePowerState) -> i32 {
        let mut current = self.devices;

        while !current.is_null() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                let result = (*current).suspend(state);
                if result < 0 {
                    log_error!("Device '{}' suspend failed: {}", (*current).name, result);
                    return result;
                }
                current = (*current).next;
            }
        }

        0
    }

    /// Resume all devices
    fn resume_devices(&mut self) -> i32 {
        let mut current = self.devices;

        while !current.is_null() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                let result = (*current).resume();
                if result < 0 {
                    log_error!("Device '{}' resume failed: {}", (*current).name, result);
                    // Continue resuming other devices
                }
                current = (*current).next;
            }
        }

        0
    }

    /// Enter freeze state
    fn enter_freeze(&mut self) -> i32 {
        log_debug!("Entering freeze state");

        // 1. Execute prepare callbacks
        check_ret!(self.run_callbacks(SuspendPhase::Prepare));

        // 2. Freeze all processes
        // Mark all user-space processes as frozen to prevent further I/O
        log_debug!("Freezing processes");
        // In a real implementation: iterate process list, set PF_FROZEN flag

        // 3. Suspend devices
        check_ret!(self.suspend_devices(DevicePowerState::D3hot));

        // 4. Execute enter callbacks
        check_ret!(self.run_callbacks(SuspendPhase::Enter));

        // 5. Enter low power state
        // Reduce CPU frequency and enter WFI
        log_debug!("Entering low power state");
        // In a real implementation: cpu::enter_idle(CpuState::DeepIdle)

        0
    }

    /// Exit freeze state
    fn exit_freeze(&mut self) -> i32 {
        log_debug!("Exiting freeze state");

        // 1. Execute exit callbacks
        check_ret!(self.run_callbacks(SuspendPhase::Exit));

        // 2. Resume devices
        self.resume_devices();

        // 3. Thaw all processes
        // Unfreeze all previously frozen processes
        log_debug!("Thawing processes");
        // In a real implementation: iterate process list, clear PF_FROZEN flag

        // 4. Execute complete callbacks
        check_ret!(self.run_callbacks(SuspendPhase::Complete));

        0
    }

    /// Enter standby state
    fn enter_standby(&mut self) -> i32 {
        log_debug!("Entering standby state");

        check_ret!(self.run_callbacks(SuspendPhase::Prepare));
        // Suspend all user-space processes (refuse new tasks)
        log_debug!("Suspending processes");
        // In a real implementation: signal processes to enter suspended state
        check_ret!(self.suspend_devices(DevicePowerState::D2));
        check_ret!(self.run_callbacks(SuspendPhase::Enter));

        0
    }

    /// Exit standby state
    fn exit_standby(&mut self) -> i32 {
        log_debug!("Exiting standby state");

        check_ret!(self.run_callbacks(SuspendPhase::Exit));
        self.resume_devices();
        // Resume all previously suspended processes
        log_debug!("Resuming processes");
        // In a real implementation: signal processes to resume execution
        check_ret!(self.run_callbacks(SuspendPhase::Complete));

        0
    }

    /// Enter suspend to RAM (STR)
    fn enter_str(&mut self) -> i32 {
        log_debug!("Entering STR (Suspend to RAM)");

        // 1. Sync file system
        // Ensure all dirty buffers are written to disk before suspending
        log_debug!("Syncing filesystem for STR");

        // 2. Execute prepare callbacks
        check_ret!(self.run_callbacks(SuspendPhase::Prepare));

        // 3. Write memory image to RAM (preserve memory contents)
        log_debug!("Writing suspend image to RAM");
        // In a real implementation: save register state and memory bitmap

        // 4. Suspend devices
        check_ret!(self.suspend_devices(DevicePowerState::D3cold));

        // 5. Execute enter callbacks
        check_ret!(self.run_callbacks(SuspendPhase::Enter));

        // 6. Enter STR mode (CPU enters WFI, memory stays powered)
        log_debug!("Entering STR mode");
        // In a real implementation: configure PMIC for STR, execute WFI

        0
    }

    /// Exit suspend to RAM (STR)
    fn exit_str(&mut self) -> i32 {
        log_debug!("Exiting STR (Suspend to RAM)");

        // 1. Execute exit callbacks
        check_ret!(self.run_callbacks(SuspendPhase::Exit));

        // 2. Resume devices
        self.resume_devices();

        // 3. Restore memory image from preserved RAM
        log_debug!("Restoring suspend image from RAM");
        // In a real implementation: restore register state and memory bitmap

        // 4. Execute complete callbacks
        check_ret!(self.run_callbacks(SuspendPhase::Complete));

        0
    }

    /// Enter suspend to disk (STD)
    fn enter_std(&mut self) -> i32 {
        log_debug!("Entering STD (Suspend to Disk)");

        // 1. Sync file system
        log_debug!("Syncing filesystem for STD");

        // 2. Execute prepare callbacks
        check_ret!(self.run_callbacks(SuspendPhase::Prepare));

        // 3. Write system state to disk (swap partition)
        log_debug!("Writing suspend image to disk");
        // In a real implementation: serialize memory pages to swap device

        // 4. Suspend devices
        check_ret!(self.suspend_devices(DevicePowerState::D3cold));

        // 5. Execute enter callbacks
        check_ret!(self.run_callbacks(SuspendPhase::Enter));

        // 6. Power off (full power down, image on disk)
        log_debug!("Powering off for STD");
        // In a real implementation: call PMIC power_off

        0
    }

    /// Exit suspend to disk (STD)
    fn exit_std(&mut self) -> i32 {
        log_debug!("Exiting STD (Suspend to Disk)");

        // 1. Execute exit callbacks
        check_ret!(self.run_callbacks(SuspendPhase::Exit));

        // 2. Resume devices
        self.resume_devices();

        // 3. Restore system state from disk (swap partition)
        log_debug!("Restoring suspend image from disk");
        // In a real implementation: deserialize memory pages from swap device

        // 4. Execute complete callbacks
        check_ret!(self.run_callbacks(SuspendPhase::Complete));

        0
    }

    /// Get current state
    pub fn get_state(&self) -> SuspendState {
        match self.current_state.load(Ordering::Acquire) {
            0 => SuspendState::Running,
            1 => SuspendState::Freeze,
            2 => SuspendState::Standby,
            3 => SuspendState::SuspendToRam,
            4 => SuspendState::SuspendToDisk,
            _ => SuspendState::Running,
        }
    }

    /// Get suspend count
    pub fn get_suspend_count(&self) -> u32 {
        self.suspend_count.load(Ordering::Acquire)
    }

    /// Get statistics
    pub fn get_stats(&self) -> &SuspendStats {
        &self.stats
    }
}

/// Global suspend manager
static SUSPEND_MANAGER: core::sync::OnceLock<SuspendManager> = core::sync::OnceLock::new();

pub fn get_suspend_manager() -> &'static mut SuspendManager {
    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe { &mut SUSPEND_MANAGER }
}

pub fn init_suspend_manager() {
    log_info!("Suspend manager initialized");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_suspend_state() {
        assert_eq!(SuspendState::Running as i32, 0);
        assert_eq!(SuspendState::Freeze as i32, 1);
        assert_eq!(SuspendState::Standby as i32, 2);
        assert_eq!(SuspendState::SuspendToRam as i32, 3);
        assert_eq!(SuspendState::SuspendToDisk as i32, 4);
    }

    #[test]
    fn test_wakeup_source() {
        assert_eq!(WakeupSource::PowerButton as i32, 0);
        assert_eq!(WakeupSource::RtcAlarm as i32, 1);
        assert_eq!(WakeupSource::Touchscreen as i32, 2);
        assert_eq!(WakeupSource::Gpio as i32, 3);
        assert_eq!(WakeupSource::Usb as i32, 4);
        assert_eq!(WakeupSource::Network as i32, 5);
    }

    #[test]
    fn test_suspend_phase() {
        assert_eq!(SuspendPhase::Prepare as i32, 0);
        assert_eq!(SuspendPhase::Enter as i32, 1);
        assert_eq!(SuspendPhase::Exit as i32, 2);
        assert_eq!(SuspendPhase::Complete as i32, 3);
    }

    #[test]
    fn test_device_power_state() {
        assert_eq!(DevicePowerState::D0 as i32, 0);
        assert_eq!(DevicePowerState::D1 as i32, 1);
        assert_eq!(DevicePowerState::D2 as i32, 2);
        assert_eq!(DevicePowerState::D3hot as i32, 3);
        assert_eq!(DevicePowerState::D3cold as i32, 4);
    }

    #[test]
    fn test_suspend_stats() {
        let stats = SuspendStats::new();
        assert_eq!(stats.success.load(Ordering::Acquire), 0);
        assert_eq!(stats.fail.load(Ordering::Acquire), 0);
    }

    #[test]
    fn test_suspend_manager() {
        let manager = SuspendManager::new();
        assert_eq!(manager.get_state(), SuspendState::Running);
        assert_eq!(manager.get_suspend_count(), 0);
    }
}
