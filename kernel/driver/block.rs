/*
 * Nuva OS - Kernel - Kernel
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

use crate::pr_info;
use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// Block device ID
pub type BlockDeviceId = u32;

/// Sector number
pub type Sector = u64;

/// Block device flags
pub mod block_flags {
    pub const READABLE: u32 = 1 << 0; // Readable
    pub const WRITABLE: u32 = 1 << 1; // Writable
    pub const REMOVABLE: u32 = 1 << 2; // Removable
    pub const READ_ONLY: u32 = 1 << 3; // Read-only
    pub const SYNC: u32 = 1 << 4; // Synchronous write
    pub const ROTATIONAL: u32 = 1 << 5; // Rotational device
    pub const SSD: u32 = 1 << 6; // Solid state drive
    pub const PARTITION: u32 = 1 << 7; // Partition
}

/// Block device operations
pub struct BlockDeviceOps {
    /// Read
    pub read: fn(dev: &BlockDevice, sector: Sector, buf: &mut [u8]) -> i64,
    /// Write
    pub write: fn(dev: &BlockDevice, sector: Sector, buf: &[u8]) -> i64,
    /// Flush
    pub flush: fn(dev: &BlockDevice) -> i32,
    /// IO control
    pub ioctl: fn(dev: &BlockDevice, cmd: u32, arg: u64) -> i32,
}

/// Block device
pub struct BlockDevice {
    /// Device ID
    pub dev_id: BlockDeviceId,
    /// Device name
    pub name: [u8; 32],
    /// Flags
    pub flags: AtomicU32,
    /// Sector size
    pub sector_size: u32,
    /// Sector count
    pub sector_count: u64,
    /// Block size
    pub block_size: u32,
    /// Operations
    pub ops: Option<BlockDeviceOps>,
    /// Private data
    pub private: u64,
    /// Reference count
    pub ref_count: AtomicU32,
    /// Read count
    pub read_count: AtomicU64,
    /// Write count
    pub write_count: AtomicU64,
    /// Read bytes
    pub read_bytes: AtomicU64,
    /// Write bytes
    pub write_bytes: AtomicU64,
}

impl BlockDevice {
    /// Create block device
    pub fn new(dev_id: BlockDeviceId, name: &[u8]) -> Self {
        let mut dev = BlockDevice {
            dev_id,
            name: [0; 32],
            flags: AtomicU32::new(block_flags::READABLE | block_flags::WRITABLE),
            sector_size: 512,
            sector_count: 0,
            block_size: 4096,
            ops: None,
            private: 0,
            ref_count: AtomicU32::new(0),
            read_count: AtomicU64::new(0),
            write_count: AtomicU64::new(0),
            read_bytes: AtomicU64::new(0),
            write_bytes: AtomicU64::new(0),
        };

        let len = name.len().min(31);
        dev.name[..len].copy_from_slice(&name[..len]);

        dev
    }

    /// Get device name
    pub fn get_name(&self) -> &[u8] {
        let mut len = 0;
        for i in 0..32 {
            if self.name[i] == 0 {
                break;
            }
            len = i + 1;
        }
        &self.name[..len]
    }

    /// Get capacity (bytes)
    pub fn get_capacity(&self) -> u64 {
        self.sector_count * self.sector_size as u64
    }

    /// Read sectors
    pub fn read(&self, sector: Sector, buf: &mut [u8]) -> i64 {
        if let Some(ref ops) = self.ops {
            let result = (ops.read)(self, sector, buf);

            if result > 0 {
                self.read_count.fetch_add(1, Ordering::AcqRel);
                self.read_bytes.fetch_add(result as u64, Ordering::AcqRel);
            }

            result
        } else {
            -1
        }
    }

    /// Write sectors
    pub fn write(&self, sector: Sector, buf: &[u8]) -> i64 {
        if let Some(ref ops) = self.ops {
            let result = (ops.write)(self, sector, buf);

            if result > 0 {
                self.write_count.fetch_add(1, Ordering::AcqRel);
                self.write_bytes.fetch_add(result as u64, Ordering::AcqRel);
            }

            result
        } else {
            -1
        }
    }

    /// Flush
    pub fn flush(&self) -> i32 {
        if let Some(ref ops) = self.ops {
            (ops.flush)(self)
        } else {
            0
        }
    }

    /// IO Control
    pub fn ioctl(&self, cmd: u32, arg: u64) -> i32 {
        if let Some(ref ops) = self.ops {
            (ops.ioctl)(self, cmd, arg)
        } else {
            -1
        }
    }

    /// Check if readable
    pub fn is_readable(&self) -> bool {
        (self.flags.load(Ordering::Acquire) & block_flags::READABLE) != 0
    }

    /// Check if writable
    pub fn is_writable(&self) -> bool {
        let flags = self.flags.load(Ordering::Acquire);
        (flags & block_flags::WRITABLE) != 0 && (flags & block_flags::READ_ONLY) == 0
    }

    /// Check if removable
    pub fn is_removable(&self) -> bool {
        (self.flags.load(Ordering::Acquire) & block_flags::REMOVABLE) != 0
    }

    /// Check if is SSD
    pub fn is_ssd(&self) -> bool {
        (self.flags.load(Ordering::Acquire) & block_flags::SSD) != 0
    }

    /// Increase reference
    pub fn get(&self) {
        self.ref_count.fetch_add(1, Ordering::AcqRel);
    }

    /// Decrease reference
    pub fn put(&self) {
        self.ref_count.fetch_sub(1, Ordering::AcqRel);
    }
}

/// Partition information
#[derive(Clone, Copy)]
pub struct Partition {
    /// Partition number
    pub part_num: u32,
    /// Start sector
    pub start_sector: Sector,
    /// Sector count
    pub sector_count: u64,
    /// Partition type
    pub part_type: u8,
    /// Flags
    pub flags: u8,
}

/// Disk partition table
pub struct PartitionTable {
    /// Partition array
    pub partitions: [Option<Partition>; 16],
    /// Partition count
    pub count: u32,
}

impl PartitionTable {
    pub const fn new() -> Self {
        PartitionTable {
            partitions: [None; 16],
            count: 0,
        }
    }

    /// Add partition
    pub fn add(&mut self, part: Partition) -> bool {
        if self.count >= 16 {
            return false;
        }

        self.partitions[self.count as usize] = Some(part);
        self.count += 1;
        true
    }

    /// Get partition
    pub fn get(&self, num: u32) -> Option<&Partition> {
        for i in 0..self.count as usize {
            if let Some(ref part) = self.partitions[i] {
                if part.part_num == num {
                    return Some(part);
                }
            }
        }
        None
    }
}

/// Block device manager
pub struct BlockDeviceManager {
    /// Device count
    pub device_count: AtomicU32,
    /// Next device ID
    pub next_dev_id: AtomicU32,
    /// Total read count
    pub total_reads: AtomicU64,
    /// Total write count
    pub total_writes: AtomicU64,
}

impl BlockDeviceManager {
    pub const fn new() -> Self {
        BlockDeviceManager {
            device_count: AtomicU32::new(0),
            next_dev_id: AtomicU32::new(1),
            total_reads: AtomicU64::new(0),
            total_writes: AtomicU64::new(0),
        }
    }

    /// Initialize
    pub fn init(&self) {
        log_info!("Block device manager initialized");
    }

    /// RegisterDevice
    pub fn register(&mut self, _dev: &mut BlockDevice) -> BlockDeviceId {
        let dev_id = self.next_dev_id.fetch_add(1, Ordering::AcqRel);
        self.device_count.fetch_add(1, Ordering::AcqRel);

        log_info!("Registered block device: dev_id={}", dev_id);

        dev_id
    }

    /// Get device count
    pub fn get_device_count(&self) -> u32 {
        self.device_count.load(Ordering::Acquire)
    }
}

/// Global block device manager
static BLOCK_DEVICE_MANAGER: crate::sync_oncelock::OnceLock<BlockDeviceManager> = crate::sync_oncelock::OnceLock::new();

pub fn block_device_manager() -> &'static BlockDeviceManager {
    BLOCK_DEVICE_MANAGER.get_or_init(BlockDeviceManager::new)
}

pub fn init_block_device_manager() -> &'static BlockDeviceManager {
    BLOCK_DEVICE_MANAGER.get_or_init(BlockDeviceManager::new)
}

pub fn init_block_device() {
    let mgr = block_device_manager();
    mgr.init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_flags() {
        assert_eq!(block_flags::READABLE, 1 << 0);
        assert_eq!(block_flags::WRITABLE, 1 << 1);
        assert_eq!(block_flags::REMOVABLE, 1 << 2);
        assert_eq!(block_flags::READ_ONLY, 1 << 3);
        assert_eq!(block_flags::SYNC, 1 << 4);
        assert_eq!(block_flags::ROTATIONAL, 1 << 5);
        assert_eq!(block_flags::SSD, 1 << 6);
        assert_eq!(block_flags::PARTITION, 1 << 7);
    }

    #[test]
    fn test_block_device_new() {
        let dev = BlockDevice::new(1, b"sda");

        assert_eq!(dev.dev_id, 1);
        assert_eq!(dev.get_name(), b"sda");
        assert_eq!(dev.sector_size, 512);
        assert_eq!(dev.block_size, 4096);
        assert!(dev.is_readable());
        assert!(dev.is_writable());
    }

    #[test]
    fn test_block_device_name() {
        let dev = BlockDevice::new(1, b"nvme0n1");

        assert_eq!(dev.get_name(), b"nvme0n1");
    }

    #[test]
    fn test_block_device_capacity() {
        let mut dev = BlockDevice::new(1, b"sda");

        dev.sector_count = 1953525168; // 1TB
        dev.sector_size = 512;

        assert_eq!(dev.get_capacity(), 1000204886016); // ~1TB
    }

    #[test]
    fn test_block_device_flags() {
        let dev = BlockDevice::new(1, b"sda");

        // Default readable and writable
        assert!(dev.is_readable());
        assert!(dev.is_writable());
        assert!(!dev.is_removable());
        assert!(!dev.is_ssd());

        // Set SSD Flag
        dev.flags.fetch_or(block_flags::SSD, Ordering::Relaxed);
        assert!(dev.is_ssd());

        // Set removable flag
        dev.flags
            .fetch_or(block_flags::REMOVABLE, Ordering::Relaxed);
        assert!(dev.is_removable());
    }

    #[test]
    fn test_block_device_read_only() {
        let dev = BlockDevice::new(1, b"sda");

        assert!(dev.is_writable());

        // Set read-only flag
        dev.flags
            .fetch_or(block_flags::READ_ONLY, Ordering::Relaxed);
        assert!(!dev.is_writable());
    }

    #[test]
    fn test_block_device_ref_count() {
        let dev = BlockDevice::new(1, b"sda");

        assert_eq!(dev.ref_count.load(Ordering::Relaxed), 0);

        dev.get();
        assert_eq!(dev.ref_count.load(Ordering::Relaxed), 1);

        dev.get();
        assert_eq!(dev.ref_count.load(Ordering::Relaxed), 2);

        dev.put();
        assert_eq!(dev.ref_count.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_block_device_read_without_ops() {
        let dev = BlockDevice::new(1, b"sda");
        let mut buf = [0u8; 512];

        let result = dev.read(0, &mut buf);
        assert_eq!(result, -1);
    }

    #[test]
    fn test_block_device_write_without_ops() {
        let dev = BlockDevice::new(1, b"sda");
        let buf = [0u8; 512];

        let result = dev.write(0, &buf);
        assert_eq!(result, -1);
    }

    #[test]
    fn test_block_device_flush_without_ops() {
        let dev = BlockDevice::new(1, b"sda");

        let result = dev.flush();
        assert_eq!(result, 0);
    }

    #[test]
    fn test_block_device_ioctl_without_ops() {
        let dev = BlockDevice::new(1, b"sda");

        let result = dev.ioctl(0, 0);
        assert_eq!(result, -1);
    }

    #[test]
    fn test_partition_new() {
        let part = Partition {
            part_num: 1,
            start_sector: 2048,
            sector_count: 1953523120,
            part_type: 0x83,
            flags: 0,
        };

        assert_eq!(part.part_num, 1);
        assert_eq!(part.start_sector, 2048);
        assert_eq!(part.sector_count, 1953523120);
        assert_eq!(part.part_type, 0x83);
    }

    #[test]
    fn test_partition_table_new() {
        let pt = PartitionTable::new();

        assert_eq!(pt.count, 0);
    }

    #[test]
    fn test_partition_table_add() {
        let mut pt = PartitionTable::new();

        let part1 = Partition {
            part_num: 1,
            start_sector: 2048,
            sector_count: 1000000,
            part_type: 0x83,
            flags: 0,
        };

        let result = pt.add(part1);
        assert!(result);
        assert_eq!(pt.count, 1);

        let part2 = Partition {
            part_num: 2,
            start_sector: 1002048,
            sector_count: 2000000,
            part_type: 0x83,
            flags: 0,
        };

        let result = pt.add(part2);
        assert!(result);
        assert_eq!(pt.count, 2);
    }

    #[test]
    fn test_partition_table_get() {
        let mut pt = PartitionTable::new();

        let part = Partition {
            part_num: 1,
            start_sector: 2048,
            sector_count: 1000000,
            part_type: 0x83,
            flags: 0,
        };

        pt.add(part);

        let found = pt.get(1);
        assert!(found.is_some());
        assert_eq!(found.unwrap().start_sector, 2048);

        let not_found = pt.get(2);
        assert!(not_found.is_none());
    }

    #[test]
    fn test_partition_table_max_partitions() {
        let mut pt = PartitionTable::new();

        // Add 16 partitions
        for i in 0..16 {
            let part = Partition {
                part_num: i + 1,
                start_sector: (i as u64 + 1) * 1000000,
                sector_count: 1000000,
                part_type: 0x83,
                flags: 0,
            };
            assert!(pt.add(part));
        }

        assert_eq!(pt.count, 16);

        // The 17th should fail
        let part = Partition {
            part_num: 17,
            start_sector: 0,
            sector_count: 0,
            part_type: 0,
            flags: 0,
        };
        assert!(!pt.add(part));
    }

    #[test]
    fn test_block_device_manager_new() {
        let mgr = BlockDeviceManager::new();

        assert_eq!(mgr.get_device_count(), 0);
        assert_eq!(mgr.total_reads.load(Ordering::Relaxed), 0);
        assert_eq!(mgr.total_writes.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_block_device_manager_register() {
        let mut mgr = BlockDeviceManager::new();
        let mut dev = BlockDevice::new(0, b"sda");

        let id1 = mgr.register(&mut dev);
        assert_eq!(id1, 1);
        assert_eq!(mgr.get_device_count(), 1);

        let id2 = mgr.register(&mut dev);
        assert_eq!(id2, 2);
        assert_eq!(mgr.get_device_count(), 2);
    }

    #[test]
    fn test_block_device_stats() {
        let dev = BlockDevice::new(1, b"sda");

        assert_eq!(dev.read_count.load(Ordering::Relaxed), 0);
        assert_eq!(dev.write_count.load(Ordering::Relaxed), 0);
        assert_eq!(dev.read_bytes.load(Ordering::Relaxed), 0);
        assert_eq!(dev.write_bytes.load(Ordering::Relaxed), 0);
    }
}

static mut CURRENT_IO_SCHEDULER: u32 = 0;

pub fn set_io_scheduler(scheduler: u32) {
    // Supported schedulers: 0=Noop, 1=Deadline, 2=CFQ
    if scheduler <= 2 {
        // SAFETY: single-threaded block device initialization context
        unsafe {
            CURRENT_IO_SCHEDULER = scheduler;
        }
    }
}

pub fn get_io_scheduler() -> u32 {
    // SAFETY: read-only access to scheduler ID
    unsafe { CURRENT_IO_SCHEDULER }
}
