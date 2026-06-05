/*
 * Nuva OS - Kernel - Storage - Block
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
use crate::{pr_info};
/*
 * Nuva OS - Kernel - Block Device Driver Framework
 * 
 * Complete block device driver implementation.
 * 
 * Copyright (C) 2026 Nuva OS Team
 */

use core::sync::atomic::{AtomicU32, AtomicU64, AtomicBool, Ordering};
use alloc::vec::Vec;
use alloc::collections::BTreeMap;

/// Block device type
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockDeviceType {
    Unknown = 0,
    HardDisk = 1,
    Ssd = 2,
    Nvme = 3,
    Usb = 4,
    Cdrom = 5,
    Floppy = 6,
    Ramdisk = 7,
    Virtual = 8,
    Mmc = 9,
    Nand = 10,
}

/// Block device flags
pub mod blk_flags {
    pub const READONLY: u32 = 1 << 0;
    pub const REMOVABLE: u32 = 1 << 1;
    pub const WRITE_CACHE: u32 = 1 << 2;
    pub const ROTATIONAL: u32 = 1 << 3;
    pub const PARTITIONABLE: u32 = 1 << 4;
    pub const SYNC: u32 = 1 << 5;
}

/// Block I/O request type
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BioType {
    Read = 0,
    Write = 1,
    Flush = 2,
    Discard = 3,
    SecureErase = 4,
    WriteSame = 5,
    ZoneReset = 6,
}

/// Block I/O status
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BioStatus {
    Pending = 0,
    InProgress = 1,
    Complete = 2,
    Failed = 3,
}

/// Block I/O request
#[repr(C)]
pub struct BioRequest {
    /// Request ID
    pub id: u64,
    /// Request type
    pub bio_type: BioType,
    /// Status
    pub status: AtomicU32,
    /// Sector number
    pub sector: u64,
    /// Number of sectors
    pub nr_sectors: u32,
    /// Buffer pointer
    pub buffer: *mut u8,
    /// Buffer size
    pub buffer_size: usize,
    /// Bytes transferred
    pub bytes_done: AtomicU32,
    /// Error code
    pub error: AtomicI32,
    /// Flags
    pub flags: u32,
    /// Callback
    pub callback: Option<unsafe fn(*mut BioRequest)>,
}

impl BioRequest {
    pub fn new_read(sector: u64, nr_sectors: u32, buffer: *mut u8) -> Self {
        BioRequest {
            id: 0,
            bio_type: BioType::Read,
            status: AtomicU32::new(BioStatus::Pending as u32),
            sector,
            nr_sectors,
            buffer,
            buffer_size: (nr_sectors as usize) * 512,
            bytes_done: AtomicU32::new(0),
            error: AtomicI32::new(0),
            flags: 0,
            callback: None,
        }
    }
    
    pub fn new_write(sector: u64, nr_sectors: u32, buffer: *const u8) -> Self {
        BioRequest {
            id: 0,
            bio_type: BioType::Write,
            status: AtomicU32::new(BioStatus::Pending as u32),
            sector,
            nr_sectors,
            buffer: buffer as *mut u8,
            buffer_size: (nr_sectors as usize) * 512,
            bytes_done: AtomicU32::new(0),
            error: AtomicI32::new(0),
            flags: 0,
            callback: None,
        }
    }
    
    pub fn is_complete(&self) -> bool {
        self.status.load(Ordering::Acquire) == BioStatus::Complete as u32
    }
    
    pub fn is_failed(&self) -> bool {
        self.status.load(Ordering::Acquire) == BioStatus::Failed as u32
    }
}

/// AtomicI32 for error code
use core::sync::atomic::AtomicI32;

/// Block device operations
pub struct BlockDeviceOps {
    /// Open device
    pub open: Option<unsafe fn(*mut BlockDevice) -> Result<(), i32>>,
    /// Close device
    pub close: Option<unsafe fn(*mut BlockDevice) -> Result<(), i32>>,
    /// Submit I/O request
    pub submit_bio: Option<unsafe fn(*mut BlockDevice, *mut BioRequest) -> Result<(), i32>>,
    /// Get geometry
    pub get_geometry: Option<unsafe fn(*const BlockDevice, *mut DiskGeometry) -> Result<(), i32>>,
    /// Flush cache
    pub flush: Option<unsafe fn(*mut BlockDevice) -> Result<(), i32>>,
    /// Trim/discard
    pub discard: Option<unsafe fn(*mut BlockDevice, u64, u32) -> Result<(), i32>>,
    /// Reset device
    pub reset: Option<unsafe fn(*mut BlockDevice) -> Result<(), i32>>,
}

/// Disk geometry
#[repr(C)]
pub struct DiskGeometry {
    pub cylinders: u32,
    pub heads: u8,
    pub sectors: u8,
    pub start: u64,
}

/// Block device
pub struct BlockDevice {
    /// Device ID
    pub id: u64,
    /// Device name
    pub name: [u8; 32],
    /// Device type
    pub dev_type: BlockDeviceType,
    /// Device number (major:minor)
    pub devno: u32,
    /// Block size
    pub block_size: AtomicU32,
    /// Number of blocks
    pub nr_blocks: u64,
    /// Capacity in bytes
    pub capacity: u64,
    /// Flags
    pub flags: AtomicU32,
    /// Operations
    pub ops: BlockDeviceOps,
    /// Reference count
    pub refs: AtomicU32,
    /// Open count
    pub open_count: AtomicU32,
    /// Private data
    pub private: *mut core::ffi::c_void,
    /// Queue depth
    pub queue_depth: u32,
    /// Pending requests
    pub pending: spin::Mutex<Vec<u64>>,
}

impl BlockDevice {
    pub fn new(name: &str, dev_type: BlockDeviceType, capacity: u64) -> Self {
        let mut name_buf = [0u8; 32];
        let len = name.as_bytes().len().min(31);
        name_buf[..len].copy_from_slice(&name.as_bytes()[..len]);
        
        BlockDevice {
            id: 0,
            name: name_buf,
            dev_type,
            devno: 0,
            block_size: AtomicU32::new(512),
            nr_blocks: capacity / 512,
            capacity,
            flags: AtomicU32::new(0),
            ops: BlockDeviceOps {
                open: None,
                close: None,
                submit_bio: None,
                get_geometry: None,
                flush: None,
                discard: None,
                reset: None,
            },
            refs: AtomicU32::new(1),
            open_count: AtomicU32::new(0),
            private: core::ptr::null_mut(),
            queue_depth: 32,
            pending: spin::Mutex::new(Vec::new()),
        }
    }
    
    /// Open device
    pub fn open(&mut self) -> Result<(), i32> {
        if let Some(open_fn) = self.ops.open {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe { open_fn(self as *mut BlockDevice) }
        } else {
            self.open_count.fetch_add(1, Ordering::AcqRel);
            Ok(())
        }
    }
    
    /// Close device
    pub fn close(&mut self) -> Result<(), i32> {
        if self.open_count.load(Ordering::Acquire) == 0 {
            return Err(-9); // EBADF
        }
        
        if let Some(close_fn) = self.ops.close {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe { close_fn(self as *mut BlockDevice) }
        } else {
            self.open_count.fetch_sub(1, Ordering::AcqRel);
            Ok(())
        }
    }
    
    /// Read blocks
    pub fn read(&mut self, sector: u64, nr_sectors: u32, buffer: *mut u8) -> Result<usize, i32> {
        let mut req = BioRequest::new_read(sector, nr_sectors, buffer);
        
        if let Some(submit_fn) = self.ops.submit_bio {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                submit_fn(self as *mut BlockDevice, &mut req as *mut BioRequest)?;
                
                // Wait for completion
                while !req.is_complete() && !req.is_failed() {
                    // TODO: Yield CPU
                }
                
                if req.is_failed() {
                    return Err(req.error.load(Ordering::Acquire));
                }
                
                Ok(req.bytes_done.load(Ordering::Acquire) as usize)
            }
        } else {
            Err(-95) // EOPNOTSUPP
        }
    }
    
    /// Write blocks
    pub fn write(&mut self, sector: u64, nr_sectors: u32, buffer: *const u8) -> Result<usize, i32> {
        let mut req = BioRequest::new_write(sector, nr_sectors, buffer);
        
        if let Some(submit_fn) = self.ops.submit_bio {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                submit_fn(self as *mut BlockDevice, &mut req as *mut BioRequest)?;
                
                while !req.is_complete() && !req.is_failed() {
                    // TODO: Yield CPU
                }
                
                if req.is_failed() {
                    return Err(req.error.load(Ordering::Acquire));
                }
                
                Ok(req.bytes_done.load(Ordering::Acquire) as usize)
            }
        } else {
            Err(-95)
        }
    }
    
    /// Flush cache
    pub fn flush(&mut self) -> Result<(), i32> {
        if let Some(flush_fn) = self.ops.flush {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe { flush_fn(self as *mut BlockDevice) }
        } else {
            Ok(())
        }
    }
    
    /// Get geometry
    pub fn get_geometry(&self) -> Result<DiskGeometry, i32> {
        let mut geom = DiskGeometry {
            cylinders: 0,
            heads: 0,
            sectors: 0,
            start: 0,
        };
        
        if let Some(get_geom_fn) = self.ops.get_geometry {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe { get_geom_fn(self, &mut geom as *mut DiskGeometry)?; }
        }
        
        Ok(geom)
    }
    
    /// Check if read-only
    pub fn is_readonly(&self) -> bool {
        self.flags.load(Ordering::Acquire) & blk_flags::READONLY != 0
    }
    
    /// Check if removable
    pub fn is_removable(&self) -> bool {
        self.flags.load(Ordering::Acquire) & blk_flags::REMOVABLE != 0
    }
}

/// Partition entry
#[repr(C)]
pub struct Partition {
    pub partno: u8,
    pub start_sector: u64,
    pub nr_sectors: u64,
    pub type_id: u8,
    pub flags: u32,
    pub name: [u8; 32],
}

/// Block device manager
pub struct BlockDeviceManager {
    devices: spin::Mutex<BTreeMap<u64, BlockDevice>>,
    next_id: AtomicU64,
    next_devno: AtomicU32,
}

impl BlockDeviceManager {
    pub const fn new() -> Self {
        BlockDeviceManager {
            devices: spin::Mutex::new(BTreeMap::new()),
            next_id: AtomicU64::new(1),
            next_devno: AtomicU32::new(0),
        }
    }
    
    /// Register block device
    pub fn register(&self, mut device: BlockDevice) -> Result<u64, i32> {
        let id = self.next_id.fetch_add(1, Ordering::AcqRel);
        let devno = self.next_devno.fetch_add(1, Ordering::AcqRel);
        
        device.id = id;
        device.devno = devno;

        log_info!("Block device registered: {} (capacity: {} bytes)",
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe { core::str::from_utf8_unchecked(&device.name) },
            device.capacity
        );

        self.devices.lock().insert(id, device);
        
        Ok(id)
    }
    
    /// Unregister block device
    pub fn unregister(&self, id: u64) -> Result<(), i32> {
        if self.devices.lock().remove(&id).is_some() {
            log_info!("Block device unregistered: {}", id);
            Ok(())
        } else {
            Err(-2) // ENOENT
        }
    }
    
    /// Get device by ID
    pub fn get(&self, id: u64) -> Option<&mut BlockDevice> {
        // Note: lifetime issues, need proper locking in real impl
        None
    }
    
    /// Get device by name
    pub fn get_by_name(&self, name: &str) -> Option<u64> {
        let devices = self.devices.lock();
        for (_, dev) in devices.iter() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            let dev_name = unsafe { core::str::from_utf8_unchecked(&dev.name) };
            if dev_name.starts_with(name) {
                return Some(dev.id);
            }
        }
        None
    }
    
    /// List all devices
    pub fn list(&self) -> Vec<u64> {
        self.devices.lock().keys().copied().collect()
    }
}

impl Default for BlockDeviceManager {
    fn default() -> Self { Self::new() }
}

/// Global block device manager
static BLK_DEV_MANAGER: core::sync::OnceLock<BlockDeviceManager> = core::sync::OnceLock::new();

/// Get block device manager
pub fn blk_dev_manager() -> &'static BlockDeviceManager {
    BLK_DEV_MANAGER.get_or_init(BlockDeviceManager::new)
}

pub fn init_blk_dev_manager() -> &'static BlockDeviceManager {
    BLK_DEV_MANAGER.get_or_init(BlockDeviceManager::new)
}

/// Initialize block device subsystem
pub fn init_block_device() {
    log_info!("Block device subsystem initialized");
}
