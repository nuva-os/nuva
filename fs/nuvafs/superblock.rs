/*
 * Nuva OS - Nuva OS
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

//! NuvaFS Superblock

use core::sync::atomic::{AtomicU64, AtomicU32, Ordering};

/// NuvaFS magic number
pub const NuvaFS_MAGIC: u32 = 0x4E56_4653; // "NVFS"

/// NuvaFS version
pub const NuvaFS_VERSION_MAJOR: u16 = 1;
pub const NuvaFS_VERSION_MINOR: u16 = 0;

/// Feature flags
pub const FEATURE_COMPRESSION: u64 = 1 << 0;
pub const FEATURE_ENCRYPTION: u64 = 1 << 1;
pub const FEATURE_JOURNAL: u64 = 1 << 2;
pub const FEATURE_SNAPSHOT: u64 = 1 << 3;
pub const FEATURE_DEDUP: u64 = 1 << 4;
pub const FEATURE_EXTENTS: u64 = 1 << 5;

/// Block size options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockSize {
    B4K = 4096,
    B8K = 8192,
    B16K = 16384,
    B32K = 32768,
    B64K = 65536,
}

impl BlockSize {
    pub fn from_u32(size: u32) -> Option<Self> {
        match size {
            4096 => Some(Self::B4K),
            8192 => Some(Self::B8K),
            16384 => Some(Self::B16K),
            32768 => Some(Self::B32K),
            65536 => Some(Self::B64K),
            _ => None,
        }
    }

    pub fn as_u32(&self) -> u32 {
        *self as u32
    }

    pub fn shift(&self) -> u32 {
        match self {
            Self::B4K => 12,
            Self::B8K => 13,
            Self::B16K => 14,
            Self::B32K => 15,
            Self::B64K => 16,
        }
    }
}

/// Compression algorithm
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CompressionAlgo {
    None = 0,
    LZ4 = 1,
    ZSTD = 2,
    GZIP = 3,
}

/// Encryption algorithm
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum EncryptionAlgo {
    None = 0,
    AES256XTS = 1,
    AES256GCM = 2,
}

/// NuvaFS superblock
#[derive(Debug, Clone)]
#[repr(C, packed)]
pub struct NuvaSuperblock {
    /// Magic number
    pub magic: u32,

    /// Version
    pub version_major: u16,
    pub version_minor: u16,

    /// Block size
    pub block_size: u32,

    /// Block shift bits
    pub block_shift: u8,

    /// Block group size (block count)
    pub blocks_per_group: u32,

    /// Total block count
    pub total_blocks: u64,

    /// Free block count
    pub free_blocks: AtomicU64,

    /// Total inode count
    pub total_inodes: u64,

    /// Free inode count
    pub free_inodes: AtomicU64,

    /// Root directory inode
    pub root_ino: u64,

    /// Journal area start block
    pub journal_start: u64,

    /// Journal area block count
    pub journal_blocks: u32,

    /// Feature flags
    pub features: u64,

    /// Compression algorithm
    pub compression: u8,

    /// Encryption algorithm
    pub encryption: u8,

    /// Reserved
    pub reserved: [u8; 6],

    /// UUID
    pub uuid: [u8; 16],

    /// Volume label
    pub label: [u8; 64],

    /// Creation time
    pub create_time: u64,

    /// Mount time
    pub mount_time: AtomicU64,

    /// Mount count
    pub mount_count: AtomicU32,

    /// Maximum mount count
    pub max_mount_count: u32,

    /// Last write time
    pub last_write_time: AtomicU64,

    /// State flags
    pub state: AtomicU32,

    /// Error handling
    pub errors: u8,

    /// Checksum
    pub checksum: u32,
}

/// Superblock state
pub const SB_STATE_CLEAN: u32 = 0;
pub const SB_STATE_DIRTY: u32 = 1;
pub const SB_STATE_ERROR: u32 = 2;

/// Error handling mode
pub const ERRORS_CONTINUE: u8 = 0;
pub const ERRORS_REMOUNT_RO: u8 = 1;
pub const ERRORS_PANIC: u8 = 2;

impl NuvaSuperblock {
    pub fn new(block_size: BlockSize, total_blocks: u64) -> Self {
        // Get current timestamp (using architecture-specific time counter)
        let now = Self::get_current_time();

        Self {
            magic: NuvaFS_MAGIC,
            version_major: NuvaFS_VERSION_MAJOR,
            version_minor: NuvaFS_VERSION_MINOR,
            block_size: block_size.as_u32(),
            block_shift: block_size.shift() as u8,
            blocks_per_group: 8192,
            total_blocks,
            free_blocks: AtomicU64::new(total_blocks - 100), // Reserve some blocks
            total_inodes: total_blocks / 4,
            free_inodes: AtomicU64::new(total_blocks / 4 - 10),
            root_ino: 2,
            journal_start: 1,
            journal_blocks: 4096,
            features: FEATURE_JOURNAL | FEATURE_EXTENTS | FEATURE_COMPRESSION,
            compression: CompressionAlgo::LZ4 as u8,
            encryption: EncryptionAlgo::None as u8,
            reserved: [0; 6],
            uuid: [0; 16],
            label: [0; 64],
            create_time: now,
            mount_time: AtomicU64::new(now),
            mount_count: AtomicU32::new(0),
            max_mount_count: 100,
            last_write_time: AtomicU64::new(now),
            state: AtomicU32::new(SB_STATE_CLEAN),
            errors: ERRORS_REMOUNT_RO,
            checksum: 0,
        }
    }

    /// Get current timestamp (Unix timestamp, seconds)
    /// Use architecture-specific time counter to get current time
    #[inline]
    fn get_current_time() -> u64 {
        // SAFETY: Reading time counter is a safe read-only operation
        unsafe {
            #[cfg(target_arch = "x86_64")]
            {
                let tsc: u64;
                core::arch::asm!(
                    "rdtsc",
                    out("rax") tsc,
                    out("rdx") _,
                    options(nomem, nostack),
                );
                // Assume TSC frequency is 1GHz, convert to seconds
                // Actual frequency should be calibrated at boot
                tsc / 1_000_000_000
            }

            #[cfg(target_arch = "aarch64")]
            {
                let cntvct: u64;
                core::arch::asm!(
                    "mrs {}, cntvct_el0",
                    out(reg) cntvct,
                    options(nomem, nostack),
                );
                cntvct / 1_000_000_000
            }

            #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
            {
                0
            }
        }
    }

    pub fn is_valid(&self) -> bool {
        self.magic == NuvaFS_MAGIC
    }

    pub fn has_feature(&self, feature: u64) -> bool {
        self.features & feature != 0
    }

    pub fn mark_dirty(&self) {
        self.state.store(SB_STATE_DIRTY, Ordering::Relaxed);
    }

    pub fn mark_clean(&self) {
        self.state.store(SB_STATE_CLEAN, Ordering::Relaxed);
    }

    pub fn is_clean(&self) -> bool {
        self.state.load(Ordering::Relaxed) == SB_STATE_CLEAN
    }

    pub fn block_to_addr(&self, block: u64) -> u64 {
        block << self.block_shift
    }

    pub fn addr_to_block(&self, addr: u64) -> u64 {
        addr >> self.block_shift
    }
}

/// Block group descriptor
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct BlockGroupDesc {
    /// Block bitmap block
    pub block_bitmap: u64,

    /// Inode bitmap block
    pub inode_bitmap: u64,

    /// Inode table start block
    pub inode_table: u64,

    /// Free block count
    pub free_blocks: u32,

    /// Free inode count
    pub free_inodes: u32,

    /// Used directory count
    pub used_dirs: u32,

    /// Checksum
    pub checksum: u32,
}
