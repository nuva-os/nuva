/*
 * Nuva OS - Kernel - Reset Controller Framework
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Reset control for device drivers.
 */

use crate::{pr_debug, pr_info};
use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// Reset ID
pub type ResetId = u32;

/// Reset Status
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResetStatus {
    /// Reset asserted
    Asserted = 0,
    /// Reset deasserted
    Deasserted = 1,
    /// Unknown
    Unknown = 2,
}

/// Reset Flags
bitflags::bitflags! {
    #[repr(transparent)]
    pub struct ResetFlags: u32 {
        /// Active low reset
        const ACTIVE_LOW = 1 << 0;
        /// Shared reset
        const SHARED = 1 << 1;
        /// Optional reset
        const OPTIONAL = 1 << 2;
        /// Exclusive reset
        const EXCLUSIVE = 1 << 3;
    }
}

/// Reset Control
#[repr(C)]
pub struct ResetControl {
    /// Reset ID
    pub id: ResetId,
    /// Controller ID
    pub controller_id: u32,
    /// Flags
    pub flags: ResetFlags,
    /// Reference count
    pub ref_count: AtomicU32,
    /// Status
    pub status: AtomicU32,
}

impl ResetControl {
    pub fn new(id: ResetId) -> Self {
        ResetControl {
            id,
            controller_id: 0,
            flags: ResetFlags::empty(),
            ref_count: AtomicU32::new(1),
            status: AtomicU32::new(ResetStatus::Deasserted as u32),
        }
    }
}

/// Reset Controller Operations
pub struct ResetControllerOps {
    /// Assert reset
    pub assert: Option<unsafe extern "C" fn(*mut core::ffi::c_void, ResetId) -> i32>,
    /// Deassert reset
    pub deassert: Option<unsafe extern "C" fn(*mut core::ffi::c_void, ResetId) -> i32>,
    /// Reset pulse
    pub reset: Option<unsafe extern "C" fn(*mut core::ffi::c_void, ResetId) -> i32>,
    /// Get status
    pub status: Option<unsafe extern "C" fn(*const core::ffi::c_void, ResetId) -> i32>,
}

/// Reset Controller
pub struct ResetController {
    /// Controller name
    pub name: [u8; 32],
    /// Controller ID
    pub id: u32,
    /// Number of resets
    pub nr_resets: u32,
    /// Operations
    pub ops: ResetControllerOps,
    /// Private data
    pub data: *mut core::ffi::c_void,
    /// Device ID
    pub device_id: u32,
}

/// Reset Manager
pub struct ResetManager {
    /// Controller count
    controller_count: AtomicU32,
    /// Statistics
    stats: ResetStats,
}

/// Reset Statistics
pub struct ResetStats {
    /// Assert count
    pub assert_count: AtomicU64,
    /// Deassert count
    pub deassert_count: AtomicU64,
    /// Reset count
    pub reset_count: AtomicU64,
}

impl ResetStats {
    pub const fn new() -> Self {
        ResetStats {
            assert_count: AtomicU64::new(0),
            deassert_count: AtomicU64::new(0),
            reset_count: AtomicU64::new(0),
        }
    }
}

impl ResetManager {
    pub const fn new() -> Self {
        ResetManager {
            controller_count: AtomicU32::new(0),
            stats: ResetStats::new(),
        }
    }

    /// Initialize
    pub fn init(&self) {
        log_info!("Reset manager initialized");
    }

    /// Register controller
    pub fn register_controller(&mut self, _ctrl: &ResetController) -> u32 {
        let id = self.controller_count.fetch_add(1, Ordering::AcqRel);
        id
    }

    /// Assert reset
    pub fn assert(&mut self, rstc: &mut ResetControl) -> i32 {
        self.stats.assert_count.fetch_add(1, Ordering::AcqRel);
        rstc.status
            .store(ResetStatus::Asserted as u32, Ordering::Release);
        log_debug!("reset_assert: id={}", rstc.id);
        0
    }

    /// Deassert reset
    pub fn deassert(&mut self, rstc: &mut ResetControl) -> i32 {
        self.stats.deassert_count.fetch_add(1, Ordering::AcqRel);
        rstc.status
            .store(ResetStatus::Deasserted as u32, Ordering::Release);
        log_debug!("reset_deassert: id={}", rstc.id);
        0
    }

    /// Reset pulse (assert then deassert)
    pub fn reset(&mut self, rstc: &mut ResetControl) -> i32 {
        self.stats.reset_count.fetch_add(1, Ordering::AcqRel);

        // Assert
        let ret = self.assert(rstc);
        if ret != 0 {
            return ret;
        }

        // Small delay (TODO: proper delay)
        for _ in 0..1000 {
            core::hint::spin_loop();
        }

        // Deassert
        let ret = self.deassert(rstc);
        if ret != 0 {
            return ret;
        }

        0
    }

    /// Get status
    pub fn status(&self, rstc: &ResetControl) -> ResetStatus {
        match rstc.status.load(Ordering::Acquire) {
            0 => ResetStatus::Asserted,
            1 => ResetStatus::Deasserted,
            _ => ResetStatus::Unknown,
        }
    }
}

/// Global reset manager
static RESET_MANAGER: core::sync::OnceLock<ResetManager> = core::sync::OnceLock::new();

/// Get reset manager
pub fn reset_manager() -> &'static ResetManager {
    RESET_MANAGER.get_or_init(ResetManager::new)
}

pub fn init_reset_manager() -> &'static ResetManager {
    RESET_MANAGER.get_or_init(ResetManager::new)
}

/// Initialize reset manager
pub fn init_reset_manager() {
    let mgr = reset_manager();
    mgr.init();
}

// Convenience functions

/// Assert reset
pub fn reset_assert(rstc: &mut ResetControl) -> i32 {
    reset_manager().assert(rstc)
}

/// Deassert reset
pub fn reset_deassert(rstc: &mut ResetControl) -> i32 {
    reset_manager().deassert(rstc)
}

/// Reset pulse
pub fn reset_reset(rstc: &mut ResetControl) -> i32 {
    reset_manager().reset(rstc)
}

/// Get status
pub fn reset_status(rstc: &ResetControl) -> ResetStatus {
    reset_manager().status(rstc)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reset_status_values() {
        assert_eq!(ResetStatus::Asserted as i32, 0);
        assert_eq!(ResetStatus::Deasserted as i32, 1);
    }

    #[test]
    fn test_reset_flags() {
        let flags = ResetFlags::ACTIVE_LOW | ResetFlags::SHARED;
        assert!(flags.contains(ResetFlags::ACTIVE_LOW));
        assert!(flags.contains(ResetFlags::SHARED));
    }

    #[test]
    fn test_reset_control() {
        let rstc = ResetControl::new(1);
        assert_eq!(rstc.id, 1);
        assert_eq!(rstc.ref_count.load(Ordering::Acquire), 1);
    }
}
