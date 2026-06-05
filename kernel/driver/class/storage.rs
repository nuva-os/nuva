/*
 * Nuva OS - Kernel - Driver - Class - Storage
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
 * Nuva OS - Kernel - Storage Device Class
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Standard interface for storage devices (eMMC, SD, NVMe, etc.).
 */

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// Storage Type
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageType {
    /// Unknown
    Unknown = 0,
    /// SD card
    Sd = 1,
    /// eMMC
    Emmc = 2,
    /// NVMe
    Nvme = 3,
    /// UFS
    Ufs = 4,
    /// SATA
    Sata = 5,
    /// SPI NOR Flash
    SpiNor = 6,
    /// SPI NAND Flash
    SpiNand = 7,
    /// Raw NAND
    RawNand = 8,
    /// Virtual
    Virtual = 9,
}

/// Storage State
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageState {
    /// Not present
    NotPresent = 0,
    /// Present
    Present = 1,
    /// Ready
    Ready = 2,
    /// Busy
    Busy = 3,
    /// Error
    Error = 4,
    /// Suspended
    Suspended = 5,
}

/// Storage Flags
bitflags::bitflags! {
    #[repr(transparent)]
    pub struct StorageFlags: u32 {
        /// Removable
        const REMOVABLE = 1 << 0;
        /// Read only
        const READ_ONLY = 1 << 1;
        /// Write protected
        const WRITE_PROTECT = 1 << 2;
        /// Bootable
        const BOOTABLE = 1 << 3;
        /// RPMB present
        const RPMB = 1 << 4;
        /// Cache enabled
        const CACHE = 1 << 5;
        /// TRIM supported
        const TRIM = 1 << 6;
        /// Discard supported
        const DISCARD = 1 << 7;
        /// Secure erase
        const SECURE_ERASE = 1 << 8;
        /// Sanitize
        const SANITIZE = 1 << 9;
        /// FUA (Force Unit Access)
        const FUA = 1 << 10;
    }
}

/// Storage Info
#[repr(C)]
pub struct StorageInfo {
    /// Device name
    pub name: [u8; 32],
    /// Storage type
    pub storage_type: StorageType,
    /// State
    pub state: StorageState,
    /// Flags
    pub flags: StorageFlags,
    /// Total capacity (bytes)
    pub capacity: u64,
    /// Sector size (bytes)
    pub sector_size: u32,
    /// Block size (bytes)
    pub block_size: u32,
    /// Max read speed (KB/s)
    pub max_read_speed: u32,
    /// Max write speed (KB/s)
    pub max_write_speed: u32,
    /// Manufacturer ID
    pub manfid: u16,
    /// OEM ID
    pub oemid: u16,
    /// Product name
    pub prod_name: [u8; 8],
    /// Product revision
    pub prod_rev: u8,
    /// Serial number
    pub serial: u32,
    /// Firmware version
    pub fwrev: [u8; 8],
    /// Lifetime (0-100%, 0xFF unknown)
    pub lifetime: u8,
    /// Health status (0-100%)
    pub health: u8,
    /// Temperature (Celsius, 0x8000 = unknown)
    pub temperature: i16,
}

/// Storage Partition
#[repr(C)]
pub struct StoragePartition {
    /// Partition ID
    pub id: u8,
    /// Partition type
    pub part_type: PartitionType,
    /// Start sector
    pub start: u64,
    /// Size in sectors
    pub size: u64,
    /// Flags
    pub flags: PartitionFlags,
    /// Name
    pub name: [u8; 36],
}

/// Partition Type
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartitionType {
    /// None
    None = 0,
    /// Bootloader
    Bootloader = 1,
    /// Boot
    Boot = 2,
    /// System
    System = 3,
    /// Data
    Data = 4,
    /// Cache
    Cache = 5,
    /// Recovery
    Recovery = 6,
    /// Misc
    Misc = 7,
    /// Metadata
    Metadata = 8,
    /// Vendor
    Vendor = 9,
    /// RPMB
    Rpmb = 10,
    /// User
    User = 11,
}

/// Partition Flags
bitflags::bitflags! {
    #[repr(transparent)]
    pub struct PartitionFlags: u32 {
        /// Active
        const ACTIVE = 1 << 0;
        /// Read only
        const READ_ONLY = 1 << 1;
        /// Hidden
        const HIDDEN = 1 << 2;
        /// Critical
        const CRITICAL = 1 << 3;
    }
}

/// Storage Request
#[repr(C)]
pub struct StorageRequest {
    /// Request ID
    pub id: u64,
    /// Command
    pub cmd: StorageCmd,
    /// Sector start
    pub sector: u64,
    /// Number of sectors
    pub nr_sectors: u32,
    /// Buffer
    pub buffer: *mut u8,
    /// Buffer size
    pub buf_size: usize,
    /// Flags
    pub flags: RequestFlags,
    /// Status
    pub status: RequestStatus,
    /// Error
    pub error: i32,
}

/// Storage Command
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageCmd {
    /// Read
    Read = 0,
    /// Write
    Write = 1,
    /// Flush
    Flush = 2,
    /// Discard
    Discard = 3,
    /// Secure erase
    SecureErase = 4,
    /// Sanitize
    Sanitize = 5,
    /// Reset
    Reset = 6,
}

/// Request Flags
bitflags::bitflags! {
    #[repr(transparent)]
    pub struct RequestFlags: u32 {
        /// Sync
        const SYNC = 1 << 0;
        /// FUA
        const FUA = 1 << 1;
        /// Read ahead
        const READ_AHEAD = 1 << 2;
        /// Write back
        const WRITE_BACK = 1 << 3;
        /// Fail fast
        const FAIL_FAST = 1 << 4;
        /// Quiet
        const QUIET = 1 << 5;
        /// Priority high
        const PRIO_HIGH = 1 << 6;
        /// Priority low
        const PRIO_LOW = 1 << 7;
    }
}

/// Request Status
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestStatus {
    /// Pending
    Pending = 0,
    /// In progress
    InProgress = 1,
    /// Completed
    Completed = 2,
    /// Failed
    Failed = 3,
    /// Cancelled
    Cancelled = 4,
}

/// Storage Device Operations
pub struct StorageDeviceOps {
    /// Initialize
    pub init: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    /// Deinitialize
    pub deinit: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    /// Get info
    pub get_info: Option<unsafe extern "C" fn(*const core::ffi::c_void, *mut StorageInfo) -> i32>,
    /// Submit request
    pub submit: Option<unsafe extern "C" fn(*mut core::ffi::c_void, *mut StorageRequest) -> i32>,
    /// Complete request
    pub complete: Option<unsafe extern "C" fn(*mut core::ffi::c_void, *mut StorageRequest) -> i32>,
    /// Flush
    pub flush: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    /// Reset
    pub reset: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
}

/// Storage ioctl commands
pub mod storage_ioctl {
    /// Get info
    pub const GET_INFO: u32 = 0x7001;
    /// Read
    pub const READ: u32 = 0x7002;
    /// Write
    pub const WRITE: u32 = 0x7003;
    /// Flush
    pub const FLUSH: u32 = 0x7004;
    /// Discard
    pub const DISCARD: u32 = 0x7005;
    /// Secure erase
    pub const SECURE_ERASE: u32 = 0x7006;
    /// Reset
    pub const RESET: u32 = 0x7007;
    /// Get partitions
    pub const GET_PARTITIONS: u32 = 0x7008;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_type() {
        assert_eq!(StorageType::Emmc as i32, 2);
        assert_eq!(StorageType::Nvme as i32, 3);
        assert_eq!(StorageType::Ufs as i32, 4);
    }

    #[test]
    fn test_storage_state() {
        assert_eq!(StorageState::Ready as i32, 2);
        assert_eq!(StorageState::Error as i32, 4);
    }

    #[test]
    fn test_storage_flags() {
        let flags = StorageFlags::REMOVABLE | StorageFlags::BOOTABLE;
        assert!(flags.contains(StorageFlags::REMOVABLE));
        assert!(flags.contains(StorageFlags::BOOTABLE));
    }

    #[test]
    fn test_storage_cmd() {
        assert_eq!(StorageCmd::Read as i32, 0);
        assert_eq!(StorageCmd::Write as i32, 1);
        assert_eq!(StorageCmd::Flush as i32, 2);
    }
}
