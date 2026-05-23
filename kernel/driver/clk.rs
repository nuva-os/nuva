/*
 * Nuva OS - Kernel - Clock Framework
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Clock management for device drivers.
 */

use crate::{pr_debug, pr_info};
use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// Clock Type
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClkType {
    /// Fixed rate clock
    Fixed = 0,
    /// Gate clock
    Gate = 1,
    /// Mux clock
    Mux = 2,
    /// Divider clock
    Divider = 3,
    /// Composite clock
    Composite = 4,
    /// PLL
    Pll = 5,
}

/// Clock Flags
bitflags::bitflags! {
    #[repr(transparent)]
    pub struct ClkFlags: u32 {
        /// Clock is critical (cannot be disabled)
        const CRITICAL = 1 << 0;
        /// Set rate parent
        const SET_RATE_PARENT = 1 << 1;
        /// Set rate no reparent
        const SET_RATE_NO_REPARENT = 1 << 2;
        /// Get rate no cache
        const GET_RATE_NOCACHE = 1 << 3;
        /// Set rate ungate
        const SET_RATE_UNGATE = 1 << 4;
        /// Set parent gate
        const SET_PARENT_GATE = 1 << 5;
        /// Enable on set rate
        const ENABLE_ON_SET_RATE = 1 << 6;
        /// Recalc rates
        const RECALC_RATES = 1 << 7;
        /// Is root
        const IS_ROOT = 1 << 8;
        /// Is basic
        const IS_BASIC = 1 << 9;
    }
}

/// Clock Rate
pub type ClkRate = u64;

/// Clock ID
pub type ClkId = u32;

/// Clock Info
#[repr(C)]
pub struct ClkInfo {
    /// Clock name
    pub name: [u8; 32],
    /// Clock ID
    pub id: ClkId,
    /// Clock type
    pub clk_type: ClkType,
    /// Flags
    pub flags: ClkFlags,
    /// Current rate (Hz)
    pub rate: ClkRate,
    /// Parent clock ID
    pub parent_id: ClkId,
    /// Enable count
    pub enable_count: AtomicU32,
    /// Prepare count
    pub prepare_count: AtomicU32,
}

/// Clock Operations
pub struct ClkOps {
    /// Prepare clock
    pub prepare: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    /// Unprepare clock
    pub unprepare: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    /// Is prepared
    pub is_prepared: Option<unsafe extern "C" fn(*const core::ffi::c_void) -> bool>,
    /// Enable clock
    pub enable: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    /// Disable clock
    pub disable: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    /// Is enabled
    pub is_enabled: Option<unsafe extern "C" fn(*const core::ffi::c_void) -> bool>,
    /// Recalculate rate
    pub recalc_rate: Option<unsafe extern "C" fn(*const core::ffi::c_void, ClkRate) -> ClkRate>,
    /// Round rate
    pub round_rate:
        Option<unsafe extern "C" fn(*const core::ffi::c_void, ClkRate, *mut ClkRate) -> i64>,
    /// Set rate
    pub set_rate: Option<unsafe extern "C" fn(*mut core::ffi::c_void, ClkRate, ClkRate) -> i32>,
    /// Get parent
    pub get_parent: Option<unsafe extern "C" fn(*const core::ffi::c_void) -> u8>,
    /// Set parent
    pub set_parent: Option<unsafe extern "C" fn(*mut core::ffi::c_void, u8) -> i32>,
    /// Get parent count
    pub get_num_parents: Option<unsafe extern "C" fn(*const core::ffi::c_void) -> u8>,
}

/// Fixed Rate Clock
#[repr(C)]
pub struct FixedClk {
    /// Fixed rate
    pub rate: ClkRate,
}

/// Gate Clock
#[repr(C)]
pub struct GateClk {
    /// Register address
    pub reg: u64,
    /// Bit index
    pub bit_idx: u8,
    /// Gate flags
    pub gate_flags: u8,
}

/// Divider Clock
#[repr(C)]
pub struct DividerClk {
    /// Register address
    pub reg: u64,
    /// Shift
    pub shift: u8,
    /// Width
    pub width: u8,
    /// Max divider
    pub max_div: u16,
    /// Divider flags
    pub div_flags: u8,
    /// Table (optional)
    pub table: *const u8,
}

/// Mux Clock
#[repr(C)]
pub struct MuxClk {
    /// Register address
    pub reg: u64,
    /// Shift
    pub shift: u8,
    /// Width
    pub width: u8,
    /// Mux flags
    pub mux_flags: u8,
    /// Parent IDs
    pub parents: [ClkId; 8],
    /// Parent count
    pub num_parents: u8,
}

/// Clock Manager
pub struct ClkManager {
    /// Clock count
    clock_count: AtomicU32,
    /// Statistics
    stats: ClkStats,
}

/// Clock Statistics
pub struct ClkStats {
    /// Enable count
    pub enable_count: AtomicU64,
    /// Disable count
    pub disable_count: AtomicU64,
    /// Set rate count
    pub set_rate_count: AtomicU64,
    /// Set parent count
    pub set_parent_count: AtomicU64,
}

impl ClkStats {
    pub const fn new() -> Self {
        ClkStats {
            enable_count: AtomicU64::new(0),
            disable_count: AtomicU64::new(0),
            set_rate_count: AtomicU64::new(0),
            set_parent_count: AtomicU64::new(0),
        }
    }
}

impl ClkManager {
    pub const fn new() -> Self {
        ClkManager {
            clock_count: AtomicU32::new(0),
            stats: ClkStats::new(),
        }
    }

    /// Initialize
    pub fn init(&self) {
        log_info!("Clock manager initialized");
    }

    /// Register clock
    pub fn register(&mut self, _clk: &ClkInfo) -> ClkId {
        let id = self.clock_count.fetch_add(1, Ordering::AcqRel);
        id
    }

    /// Enable clock
    pub fn enable(&mut self, clk_id: ClkId) -> i32 {
        self.stats.enable_count.fetch_add(1, Ordering::AcqRel);
        log_debug!("clk_enable: id={}", clk_id);
        0
    }

    /// Disable clock
    pub fn disable(&mut self, clk_id: ClkId) -> i32 {
        self.stats.disable_count.fetch_add(1, Ordering::AcqRel);
        log_debug!("clk_disable: id={}", clk_id);
        0
    }

    /// Get clock rate
    pub fn get_rate(&self, clk_id: ClkId) -> ClkRate {
        log_debug!("clk_get_rate: id={}", clk_id);
        0
    }

    /// Set clock rate
    pub fn set_rate(&mut self, clk_id: ClkId, rate: ClkRate) -> i32 {
        self.stats.set_rate_count.fetch_add(1, Ordering::AcqRel);
        log_debug!("clk_set_rate: id={}, rate={}", clk_id, rate);
        0
    }

    /// Round rate
    pub fn round_rate(&self, clk_id: ClkId, rate: ClkRate) -> ClkRate {
        log_debug!("clk_round_rate: id={}, rate={}", clk_id, rate);
        rate
    }

    /// Set parent
    pub fn set_parent(&mut self, clk_id: ClkId, parent_id: ClkId) -> i32 {
        self.stats.set_parent_count.fetch_add(1, Ordering::AcqRel);
        log_debug!("clk_set_parent: id={}, parent={}", clk_id, parent_id);
        0
    }

    /// Get parent
    pub fn get_parent(&self, clk_id: ClkId) -> ClkId {
        log_debug!("clk_get_parent: id={}", clk_id);
        0
    }

    /// Prepare clock
    pub fn prepare(&mut self, clk_id: ClkId) -> i32 {
        log_debug!("clk_prepare: id={}", clk_id);
        0
    }

    /// Unprepare clock
    pub fn unprepare(&mut self, clk_id: ClkId) -> i32 {
        log_debug!("clk_unprepare: id={}", clk_id);
        0
    }
}

/// Global clock manager
static CLK_MANAGER: core::sync::OnceLock<ClkManager> = core::sync::OnceLock::new();

/// Get clock manager
pub fn clk_manager() -> &'static ClkManager {
    CLK_MANAGER.get_or_init(ClkManager::new)
}

pub fn init_clk_manager() -> &'static ClkManager {
    CLK_MANAGER.get_or_init(ClkManager::new)
}

/// Initialize clock manager
pub fn init_clk_manager() {
    let mgr = clk_manager();
    mgr.init();
}

// Convenience functions

/// Enable clock
pub fn clk_enable(clk_id: ClkId) -> i32 {
    clk_manager().enable(clk_id)
}

/// Disable clock
pub fn clk_disable(clk_id: ClkId) -> i32 {
    clk_manager().disable(clk_id)
}

/// Get clock rate
pub fn clk_get_rate(clk_id: ClkId) -> ClkRate {
    clk_manager().get_rate(clk_id)
}

/// Set clock rate
pub fn clk_set_rate(clk_id: ClkId, rate: ClkRate) -> i32 {
    clk_manager().set_rate(clk_id, rate)
}

/// Round rate
pub fn clk_round_rate(clk_id: ClkId, rate: ClkRate) -> ClkRate {
    clk_manager().round_rate(clk_id, rate)
}

/// Set parent
pub fn clk_set_parent(clk_id: ClkId, parent_id: ClkId) -> i32 {
    clk_manager().set_parent(clk_id, parent_id)
}

/// Get parent
pub fn clk_get_parent(clk_id: ClkId) -> ClkId {
    clk_manager().get_parent(clk_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clk_type_values() {
        assert_eq!(ClkType::Fixed as i32, 0);
        assert_eq!(ClkType::Gate as i32, 1);
        assert_eq!(ClkType::Pll as i32, 5);
    }

    #[test]
    fn test_clk_flags() {
        let flags = ClkFlags::CRITICAL | ClkFlags::IS_ROOT;
        assert!(flags.contains(ClkFlags::CRITICAL));
        assert!(flags.contains(ClkFlags::IS_ROOT));
    }

    #[test]
    fn test_fixed_clk() {
        let clk = FixedClk { rate: 24_000_000 };
        assert_eq!(clk.rate, 24_000_000);
    }
}
