/*
 * Nuva OS - Kernel - MFD (Multi-Function Device) Framework
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * MFD core for devices that contain multiple sub-devices.
 */

use crate::{pr_debug, pr_info};
use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// MFD Cell (sub-device)
#[repr(C)]
pub struct MfdCell {
    /// Cell name
    pub name: [u8; 32],
    /// Cell ID
    pub id: u32,
    /// Compatible string
    pub compatible: [u8; 32],
    /// Platform data
    pub platform_data: *mut core::ffi::c_void,
    /// Platform data size
    pub pdata_size: usize,
    /// Resources
    pub resources: *mut MfdResource,
    /// Number of resources
    pub num_resources: u8,
    /// Parent device
    pub parent: u32,
    /// OF match table
    pub of_compatible: *const u8,
    /// ACPI match table
    pub acpi_match: *const u8,
    /// PM ops
    pm_ops: *const core::ffi::c_void,
    /// Enable mask
    pub enable_mask: u32,
    /// Enable register
    pub enable_reg: u32,
    /// Disable mask
    pub disable_mask: u32,
    /// Disable register
    pub disable_reg: u32,
    /// Flags
    pub flags: MfdCellFlags,
}

/// MFD Cell Flags
bitflags::bitflags! {
    #[repr(transparent)]
    pub struct MfdCellFlags: u32 {
        /// Use platform data
        const USE_PLATFORM_DATA = 1 << 0;
        /// Use OF match
        const USE_OF = 1 << 1;
        /// Use ACPI match
        const USE_ACPI = 1 << 2;
        /// Enable on probe
        const ENABLE_ON_PROBE = 1 << 3;
        /// Disable on remove
        const DISABLE_ON_REMOVE = 1 << 4;
        /// IRQ shared
        const IRQ_SHARED = 1 << 5;
        /// Clock always on
        const CLOCK_ALWAYS_ON = 1 << 6;
    }
}

/// MFD Resource
#[repr(C)]
pub struct MfdResource {
    /// Resource type
    pub res_type: MfdResourceType,
    /// Start
    pub start: u64,
    /// End
    pub end: u64,
    /// Flags
    pub flags: u32,
    /// Name
    pub name: [u8; 16],
}

/// MFD Resource Type
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MfdResourceType {
    /// I/O port
    Io = 1,
    /// Memory
    Mem = 2,
    /// IRQ
    Irq = 3,
    /// DMA
    Dma = 4,
    /// Bus
    Bus = 5,
}

/// MFD Device
pub struct MfdDevice {
    /// Device name
    pub name: [u8; 32],
    /// Device ID
    pub id: u32,
    /// Parent device
    pub parent: u32,
    /// Cells
    pub cells: *mut MfdCell,
    /// Number of cells
    pub num_cells: u8,
    /// Private data
    pub data: *mut core::ffi::c_void,
    /// IRQ base
    pub irq_base: i32,
    /// IO base
    pub io_base: u64,
    /// Mem base
    pub mem_base: u64,
    /// Flags
    pub flags: MfdDeviceFlags,
}

/// MFD Device Flags
bitflags::bitflags! {
    #[repr(transparent)]
    pub struct MfdDeviceFlags: u32 {
        /// IRQ domain
        const IRQ_DOMAIN = 1 << 0;
        /// Regmap
        const REGMAP = 1 << 1;
        /// Syscon
        const SYSCON = 1 << 2;
        /// PM supported
        const PM = 1 << 3;
    }
}

/// MFD Operations
pub struct MfdOps {
    /// Probe
    pub probe: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    /// Remove
    pub remove: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    /// Suspend
    pub suspend: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    /// Resume
    pub resume: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    /// Shutdown
    pub shutdown: Option<unsafe extern "C" fn(*mut core::ffi::c_void)>,
    /// Enable cell
    pub enable_cell: Option<unsafe extern "C" fn(*mut core::ffi::c_void, *const MfdCell) -> i32>,
    /// Disable cell
    pub disable_cell: Option<unsafe extern "C" fn(*mut core::ffi::c_void, *const MfdCell) -> i32>,
}

/// MFD Driver
pub struct MfdDriver {
    /// Driver name
    pub name: [u8; 32],
    /// Operations
    pub ops: MfdOps,
    /// ID table
    pub id_table: *const MfdId,
    /// OF match table
    pub of_match_table: *const u8,
    /// ACPI match table
    pub acpi_match_table: *const u8,
}

/// MFD ID
#[repr(C)]
pub struct MfdId {
    /// Name
    pub name: [u8; 32],
    /// Driver data
    pub driver_data: *mut core::ffi::c_void,
}

/// MFD Manager
pub struct MfdManager {
    /// Device count
    dev_count: AtomicU32,
    /// Statistics
    stats: MfdStats,
}

/// MFD Statistics
pub struct MfdStats {
    /// Register count
    pub register_count: AtomicU64,
    /// Probe count
    pub probe_count: AtomicU64,
    /// Cell count
    pub cell_count: AtomicU64,
}

impl MfdStats {
    pub const fn new() -> Self {
        MfdStats {
            register_count: AtomicU64::new(0),
            probe_count: AtomicU64::new(0),
            cell_count: AtomicU64::new(0),
        }
    }
}

impl MfdManager {
    pub const fn new() -> Self {
        MfdManager {
            dev_count: AtomicU32::new(0),
            stats: MfdStats::new(),
        }
    }

    /// Initialize
    pub fn init(&self) {
        log_info!("MFD manager initialized");
    }

    /// Register MFD device
    pub fn register_device(&mut self, mfd: &MfdDevice) -> i32 {
        self.stats.register_count.fetch_add(1, Ordering::AcqRel);
        self.stats
            .cell_count
            .fetch_add(mfd.num_cells as u64, Ordering::AcqRel);

        let id = self.dev_count.fetch_add(1, Ordering::AcqRel);
        log_debug!(
            "mfd_register: name={:?}, id={}, cells={}",
            &mfd.name[..],
            id,
            mfd.num_cells
        );
        0
    }

    /// Unregister MFD device
    pub fn unregister_device(&mut self, mfd: &MfdDevice) {
        log_debug!("mfd_unregister: name={:?}", &mfd.name[..]);
    }

    /// Add devices from MFD
    pub fn add_devices(&mut self, mfd: &MfdDevice) -> i32 {
        log_debug!("mfd_add_devices: cells={}", mfd.num_cells);
        mfd.num_cells as i32
    }

    /// Remove devices from MFD
    pub fn remove_devices(&mut self, mfd: &MfdDevice) {
        log_debug!("mfd_remove_devices: cells={}", mfd.num_cells);
    }

    /// Enable cell
    pub fn enable_cell(&mut self, mfd: &MfdDevice, cell: &MfdCell) -> i32 {
        log_debug!("mfd_enable_cell: cell={:?}", &cell.name[..]);
        0
    }

    /// Disable cell
    pub fn disable_cell(&mut self, mfd: &MfdDevice, cell: &MfdCell) -> i32 {
        log_debug!("mfd_disable_cell: cell={:?}", &cell.name[..]);
        0
    }
}

/// Global MFD manager
static MFD_MANAGER: core::sync::OnceLock<MfdManager> = core::sync::OnceLock::new();

/// Get MFD manager
pub fn mfd_manager() -> &'static MfdManager {
    MFD_MANAGER.get_or_init(MfdManager::new)
}

pub fn init_mfd_manager() -> &'static MfdManager {
    MFD_MANAGER.get_or_init(MfdManager::new)
}

/// Initialize MFD manager
pub fn init_mfd_manager() {
    let mgr = mfd_manager();
    mgr.init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mfd_resource_type() {
        assert_eq!(MfdResourceType::Io as i32, 1);
        assert_eq!(MfdResourceType::Mem as i32, 2);
        assert_eq!(MfdResourceType::Irq as i32, 3);
    }

    #[test]
    fn test_mfd_cell_flags() {
        let flags = MfdCellFlags::USE_OF | MfdCellFlags::ENABLE_ON_PROBE;
        assert!(flags.contains(MfdCellFlags::USE_OF));
        assert!(flags.contains(MfdCellFlags::ENABLE_ON_PROBE));
    }
}
