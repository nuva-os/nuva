/*
 * Nuva OS - Kernel - EEPROM Device Class
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Standard interface for EEPROM/Flash devices.
 */

use crate::{pr_debug, pr_info};
use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// EEPROM Type
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EepromType {
    /// Unknown
    Unknown = 0,
    /// EEPROM
    Eeprom = 1,
    /// Flash
    Flash = 2,
    /// FRAM
    Fram = 3,
    /// NVRAM
    Nvram = 4,
    /// OTP
    Otp = 5,
}

/// EEPROM Flags
bitflags::bitflags! {
    #[repr(transparent)]
    pub struct EepromFlags: u32 {
        /// Read only
        const READ_ONLY = 1 << 0;
        /// Write protect
        const WRITE_PROTECT = 1 << 1;
        /// Erase before write
        const ERASE_BEFORE_WRITE = 1 << 2;
        /// Page erase
        const PAGE_ERASE = 1 << 3;
        /// Sector erase
        const SECTOR_ERASE = 1 << 4;
        /// Block erase
        const BLOCK_ERASE = 1 << 5;
        /// Chip erase
        const CHIP_ERASE = 1 << 6;
        /// Lockable
        const LOCKABLE = 1 << 7;
        /// OTP area
        const OTP = 1 << 8;
    }
}

/// EEPROM Info
#[repr(C)]
pub struct EepromInfo {
    /// Device name
    pub name: [u8; 32],
    /// EEPROM type
    pub eeprom_type: EepromType,
    /// Total size (bytes)
    pub size: u32,
    /// Page size (bytes)
    pub page_size: u16,
    /// Sector size (bytes)
    pub sector_size: u32,
    /// Block size (bytes)
    pub block_size: u32,
    /// Address width (bits)
    pub addr_width: u8,
    /// Flags
    pub flags: EepromFlags,
    /// Write cycle time (ms)
    pub write_cycle_time: u16,
    /// Erase time (ms)
    pub erase_time: u32,
    /// Endurance (cycles)
    pub endurance: u32,
}

/// EEPROM Region
#[repr(C)]
pub struct EepromRegion {
    /// Region name
    pub name: [u8; 16],
    /// Offset
    pub offset: u32,
    /// Size
    pub size: u32,
    /// Flags
    pub flags: EepromFlags,
    /// Locked
    pub locked: bool,
}

/// EEPROM Device Operations
pub struct EepromDeviceOps {
    /// Read
    pub read: Option<unsafe extern "C" fn(*const core::ffi::c_void, u32, *mut u8, usize) -> i32>,
    /// Write
    pub write: Option<unsafe extern "C" fn(*mut core::ffi::c_void, u32, *const u8, usize) -> i32>,
    /// Erase
    pub erase: Option<unsafe extern "C" fn(*mut core::ffi::c_void, u32, usize) -> i32>,
    /// Get info
    pub get_info: Option<unsafe extern "C" fn(*const core::ffi::c_void, *mut EepromInfo) -> i32>,
    /// Lock region
    pub lock: Option<unsafe extern "C" fn(*mut core::ffi::c_void, u32, usize) -> i32>,
    /// Unlock region
    pub unlock: Option<unsafe extern "C" fn(*mut core::ffi::c_void, u32, usize) -> i32>,
    /// Is locked
    pub is_locked: Option<unsafe extern "C" fn(*const core::ffi::c_void, u32) -> bool>,
}

/// EEPROM ioctl commands
pub mod eeprom_ioctl {
    /// Read
    pub const READ: u32 = 0x6001;
    /// Write
    pub const WRITE: u32 = 0x6002;
    /// Erase
    pub const ERASE: u32 = 0x6003;
    /// Get info
    pub const GET_INFO: u32 = 0x6004;
    /// Lock
    pub const LOCK: u32 = 0x6005;
    /// Unlock
    pub const UNLOCK: u32 = 0x6006;
}

/// EEPROM Manager
pub struct EepromManager {
    /// EEPROM count
    eeprom_count: AtomicU32,
    /// Statistics
    stats: EepromStats,
}

/// EEPROM Statistics
pub struct EepromStats {
    /// Read count
    pub read_count: AtomicU64,
    /// Write count
    pub write_count: AtomicU64,
    /// Erase count
    pub erase_count: AtomicU64,
    /// Bytes read
    pub bytes_read: AtomicU64,
    /// Bytes written
    pub bytes_written: AtomicU64,
}

impl EepromStats {
    pub const fn new() -> Self {
        EepromStats {
            read_count: AtomicU64::new(0),
            write_count: AtomicU64::new(0),
            erase_count: AtomicU64::new(0),
            bytes_read: AtomicU64::new(0),
            bytes_written: AtomicU64::new(0),
        }
    }
}

impl EepromManager {
    pub const fn new() -> Self {
        EepromManager {
            eeprom_count: AtomicU32::new(0),
            stats: EepromStats::new(),
        }
    }

    /// Initialize
    pub fn init(&self) {
        log_info!("EEPROM manager initialized");
    }

    /// Register EEPROM
    pub fn register(&mut self) -> u32 {
        self.eeprom_count.fetch_add(1, Ordering::AcqRel)
    }

    /// Read
    pub fn read(&mut self, eeprom_id: u32, offset: u32, buf: &mut [u8]) -> i32 {
        self.stats.read_count.fetch_add(1, Ordering::AcqRel);
        self.stats
            .bytes_read
            .fetch_add(buf.len() as u64, Ordering::AcqRel);
        log_debug!(
            "eeprom_read: id={}, offset={}, len={}",
            eeprom_id,
            offset,
            buf.len()
        );
        buf.len() as i32
    }

    /// Write
    pub fn write(&mut self, eeprom_id: u32, offset: u32, data: &[u8]) -> i32 {
        self.stats.write_count.fetch_add(1, Ordering::AcqRel);
        self.stats
            .bytes_written
            .fetch_add(data.len() as u64, Ordering::AcqRel);
        log_debug!(
            "eeprom_write: id={}, offset={}, len={}",
            eeprom_id,
            offset,
            data.len()
        );
        data.len() as i32
    }

    /// Erase
    pub fn erase(&mut self, eeprom_id: u32, offset: u32, size: usize) -> i32 {
        self.stats.erase_count.fetch_add(1, Ordering::AcqRel);
        log_debug!(
            "eeprom_erase: id={}, offset={}, size={}",
            eeprom_id,
            offset,
            size
        );
        0
    }
}

/// Global EEPROM manager
static EEPROM_MANAGER: core::sync::OnceLock<EepromManager> = core::sync::OnceLock::new();

/// Get EEPROM manager
pub fn eeprom_manager() -> &'static EepromManager {
    EEPROM_MANAGER.get_or_init(EepromManager::new)
}

pub fn init_eeprom_manager() -> &'static EepromManager {
    EEPROM_MANAGER.get_or_init(EepromManager::new)
}

/// Initialize EEPROM manager
pub fn init_eeprom_manager() {
    let mgr = eeprom_manager();
    mgr.init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eeprom_type() {
        assert_eq!(EepromType::Eeprom as i32, 1);
        assert_eq!(EepromType::Flash as i32, 2);
        assert_eq!(EepromType::Fram as i32, 3);
    }

    #[test]
    fn test_eeprom_flags() {
        let flags = EepromFlags::READ_ONLY | EepromFlags::LOCKABLE;
        assert!(flags.contains(EepromFlags::READ_ONLY));
        assert!(flags.contains(EepromFlags::LOCKABLE));
    }
}
