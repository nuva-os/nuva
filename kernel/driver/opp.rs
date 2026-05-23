/*
 * Nuva OS - Kernel - OPP (Operating Performance Points) Framework
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * OPP framework for performance state management.
 */

use crate::{pr_debug, pr_info};
use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// OPP ID
pub type OppId = u32;

/// OPP Level
pub type OppLevel = u32;

/// OPP (Operating Performance Point)
#[repr(C)]
pub struct Opp {
    /// OPP ID
    pub id: OppId,
    /// Level (performance level)
    pub level: OppLevel,
    /// Frequency (Hz)
    pub freq: u64,
    /// Voltage (uV)
    pub u_volt: u32,
    /// Target voltage (uV)
    pub u_volt_target: u32,
    /// Minimum voltage (uV)
    pub u_volt_min: u32,
    /// Maximum voltage (uV)
    pub u_volt_max: u32,
    /// Current (uA)
    pub u_amp: u32,
    /// Power (uW)
    pub u_watt: u32,
    /// Clock latency (ns)
    pub clock_latency_ns: u64,
    /// Flags
    pub flags: OppFlags,
    /// Availability
    pub available: bool,
    /// Supplies count
    pub supplies: u8,
}

/// OPP Flags
bitflags::bitflags! {
    #[repr(transparent)]
    pub struct OppFlags: u32 {
        /// Dynamic
        const DYNAMIC = 1 << 0;
        /// Turbo
        const TURBO = 1 << 1;
        /// Shared
        const SHARED = 1 << 2;
        /// Performance
        const PERFORMANCE = 1 << 3;
        /// Power
        const POWER = 1 << 4;
    }
}

/// OPP Table
#[repr(C)]
pub struct OppTable {
    /// Table name
    pub name: [u8; 32],
    /// Table ID
    pub id: u32,
    /// OPPs
    pub opps: [Opp; 16],
    /// Number of OPPs
    pub num_opps: u8,
    /// Current OPP index
    pub current_opp: u8,
    /// Supported hardware
    pub supported_hw: u32,
    /// Supported hardware count
    pub supported_hw_count: u8,
    /// Prop name
    pub prop_name: [u8; 32],
    /// Regulator count
    pub regulator_count: u8,
    /// Clock name
    pub clk_name: [u8; 32],
    /// Flags
    pub flags: OppTableFlags,
    /// Is shared
    pub is_shared: bool,
    /// Reference count
    pub ref_count: AtomicU32,
}

/// OPP Table Flags
bitflags::bitflags! {
    #[repr(transparent)]
    pub struct OppTableFlags: u32 {
        /// Has frequency
        const HAS_FREQ = 1 << 0;
        /// Has voltage
        const HAS_VOLTAGE = 1 << 1;
        /// Has current
        const HAS_CURRENT = 1 << 2;
        /// Has power
        const HAS_POWER = 1 << 3;
        /// Has level
        const HAS_LEVEL = 1 << 4;
        /// Has latency
        const HAS_LATENCY = 1 << 5;
        /// Is read only
        const READ_ONLY = 1 << 6;
    }
}

/// OPP Config
#[repr(C)]
pub struct OppConfig {
    /// Regulator names
    pub regulator_names: [[u8; 32]; 4],
    /// Number of regulators
    pub num_regulators: u8,
    /// Clock name
    pub clk_name: [u8; 32],
    /// Prop name
    pub prop_name: [u8; 32],
    /// Supported hardware
    pub supported_hw: u32,
    /// Supported hardware count
    pub supported_hw_count: u8,
    /// Flags
    pub flags: u32,
}

/// OPP Search Type
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OppSearchType {
    /// Exact match
    Exact = 0,
    /// Higher than or equal
    Higher = 1,
    /// Lower than or equal
    Lower = 2,
    /// Ceiling
    Ceiling = 3,
    /// Floor
    Floor = 4,
}

/// OPP Manager
pub struct OppManager {
    /// Table count
    table_count: AtomicU32,
    /// Statistics
    stats: OppStats,
}

/// OPP Statistics
pub struct OppStats {
    /// Set count
    pub set_count: AtomicU64,
    /// Get count
    pub get_count: AtomicU64,
    /// Tables registered
    pub tables: AtomicU64,
}

impl OppStats {
    pub const fn new() -> Self {
        OppStats {
            set_count: AtomicU64::new(0),
            get_count: AtomicU64::new(0),
            tables: AtomicU64::new(0),
        }
    }
}

impl OppManager {
    pub const fn new() -> Self {
        OppManager {
            table_count: AtomicU32::new(0),
            stats: OppStats::new(),
        }
    }

    /// Initialize
    pub fn init(&self) {
        log_info!("OPP manager initialized");
    }

    /// Add OPP table
    pub fn add_table(&mut self, _table: &OppTable) -> u32 {
        self.stats.tables.fetch_add(1, Ordering::AcqRel);
        self.table_count.fetch_add(1, Ordering::AcqRel)
    }

    /// Remove OPP table
    pub fn remove_table(&mut self, table_id: u32) {
        log_debug!("opp_remove_table: id={}", table_id);
    }

    /// Find OPP by frequency
    pub fn find_freq(&self, table_id: u32, freq: u64, search_type: OppSearchType) -> Option<Opp> {
        self.stats.get_count.fetch_add(1, Ordering::AcqRel);
        log_debug!(
            "opp_find_freq: table={}, freq={}, type={:?}",
            table_id,
            freq,
            search_type
        );
        None
    }

    /// Find OPP by level
    pub fn find_level(&self, table_id: u32, level: OppLevel) -> Option<Opp> {
        log_debug!("opp_find_level: table={}, level={}", table_id, level);
        None
    }

    /// Set OPP
    pub fn set_opp(&mut self, table_id: u32, opp: &Opp) -> i32 {
        self.stats.set_count.fetch_add(1, Ordering::AcqRel);
        log_debug!("opp_set: table={}, opp={}", table_id, opp.id);
        0
    }

    /// Get current OPP
    pub fn get_current(&self, table_id: u32) -> Option<Opp> {
        log_debug!("opp_get_current: table={}", table_id);
        None
    }

    /// Enable OPP
    pub fn enable(&mut self, table_id: u32, opp_id: OppId) -> i32 {
        log_debug!("opp_enable: table={}, opp={}", table_id, opp_id);
        0
    }

    /// Disable OPP
    pub fn disable(&mut self, table_id: u32, opp_id: OppId) -> i32 {
        log_debug!("opp_disable: table={}, opp={}", table_id, opp_id);
        0
    }

    /// Adjust available OPPs
    pub fn adjust_available(&mut self, table_id: u32, min_freq: u64, max_freq: u64) -> i32 {
        log_debug!(
            "opp_adjust: table={}, min={}, max={}",
            table_id,
            min_freq,
            max_freq
        );
        0
    }
}

/// Global OPP manager
static OPP_MANAGER: core::sync::OnceLock<OppManager> = core::sync::OnceLock::new();

/// Get OPP manager
pub fn opp_manager() -> &'static OppManager {
    OPP_MANAGER.get_or_init(OppManager::new)
}

pub fn init_opp_manager() -> &'static OppManager {
    OPP_MANAGER.get_or_init(OppManager::new)
}

/// Initialize OPP manager
pub fn init_opp_manager() {
    let mgr = opp_manager();
    mgr.init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opp_flags() {
        let flags = OppFlags::TURBO | OppFlags::PERFORMANCE;
        assert!(flags.contains(OppFlags::TURBO));
        assert!(flags.contains(OppFlags::PERFORMANCE));
    }

    #[test]
    fn test_opp_table_flags() {
        let flags = OppTableFlags::HAS_FREQ | OppTableFlags::HAS_VOLTAGE;
        assert!(flags.contains(OppTableFlags::HAS_FREQ));
        assert!(flags.contains(OppTableFlags::HAS_VOLTAGE));
    }

    #[test]
    fn test_opp_search_type() {
        assert_eq!(OppSearchType::Exact as i32, 0);
        assert_eq!(OppSearchType::Higher as i32, 1);
        assert_eq!(OppSearchType::Floor as i32, 4);
    }
}
