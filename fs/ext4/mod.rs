/*
 * Nuva OS - ext4 File System
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * ext4 file system implementation
 */

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

pub mod superblock;
pub mod inode;
pub mod extent;
pub mod journal;

// ============================================================================
// ext4 Constants
// ============================================================================

/// ext4 magic number
pub const EXT4_SUPER_MAGIC: u16 = 0xEF53;

/// Minimum block size
pub const EXT4_MIN_BLOCK_SIZE: u32 = 1024;

/// Maximum block size
pub const EXT4_MAX_BLOCK_SIZE: u32 = 65536;

/// Default block size
pub const EXT4_DEF_BLOCK_SIZE: u32 = 4096;

/// Minimum inode size
pub const EXT4_MIN_INODE_SIZE: u16 = 128;

/// Default inode size
pub const EXT4_DEF_INODE_SIZE: u16 = 256;

/// Blocks per group
pub const EXT4_BLOCKS_PER_GROUP: u32 = 32768;

/// Inodes per group
pub const EXT4_INODES_PER_GROUP: u32 = 8192;

/// Superblock offset
pub const EXT4_SUPERBLOCK_OFFSET: u64 = 1024;

/// Descriptor size
pub const EXT4_DESC_SIZE: u16 = 32;

/// Descriptor size (64-bit)
pub const EXT4_DESC_SIZE_64: u16 = 64;

// ============================================================================
// ext4 Feature Flags
// ============================================================================

/// Compatible features
pub mod compat_features {
    pub const DIR_PREALLOC: u32 = 0x0001;
    pub const IMAGIC_INODES: u32 = 0x0002;
    pub const HAS_JOURNAL: u32 = 0x0004;
    pub const EXT_ATTR: u32 = 0x0008;
    pub const RESIZE_INODE: u32 = 0x0010;
    pub const DIR_INDEX: u32 = 0x0020;
    pub const LAZY_BG: u32 = 0x0040;
    pub const EXCLUDE_INODE: u32 = 0x0080;
    pub const EXCLUDE_BITMAP: u32 = 0x0100;
    pub const SPARSE_SUPER2: u32 = 0x0200;
}

/// Read-only compatible features
pub mod ro_compat_features {
    pub const SPARSE_SUPER: u32 = 0x0001;
    pub const LARGE_FILE: u32 = 0x0002;
    pub const BTREE_DIR: u32 = 0x0004;
    pub const HUGE_FILE: u32 = 0x0008;
    pub const GDT_CSUM: u32 = 0x0010;
    pub const DIR_NLINK: u32 = 0x0020;
    pub const EXTRA_ISIZE: u32 = 0x0040;
    pub const HAS_SNAPSHOT: u32 = 0x0080;
    pub const QUOTA: u32 = 0x0100;
    pub const BIGALLOC: u32 = 0x0200;
    pub const METADATA_CSUM: u32 = 0x0400;
    pub const READONLY: u32 = 0x8000;
}

/// Incompatible features
pub mod incompat_features {
    pub const COMPRESSION: u32 = 0x0001;
    pub const FILETYPE: u32 = 0x0002;
    pub const RECOVER: u32 = 0x0004;
    pub const JOURNAL_DEV: u32 = 0x0008;
    pub const META_BG: u32 = 0x0010;
    pub const EXTENTS: u32 = 0x0040;
    pub const 64BIT: u32 = 0x0080;
    pub const MMP: u32 = 0x0100;
    pub const FLEX_BG: u32 = 0x0200;
    pub const EA_INODE: u32 = 0x0400;
    pub const DIRDATA: u32 = 0x1000;
    pub const BG_INCOMPAT: u32 = 0x2000;
    pub const LARGEDIR: u32 = 0x4000;
    pub const INLINE_DATA: u32 = 0x8000;
    pub const ENCRYPT: u32 = 0x10000;
}

// ============================================================================
// ext4 Inode Flags
// ============================================================================

pub mod inode_flags {
    pub const SECURE_FL: u32 = 0x00000001;
    pub const SYMLINK_FL: u32 = 0x00000002;
    pub const UNRM_FL: u32 = 0x00000004;
    pub const COMPR_FL: u32 = 0x00000008;
    pub const SYNC_FL: u32 = 0x00000010;
    pub const IMMUTABLE_FL: u32 = 0x00000020;
    pub const APPEND_FL: u32 = 0x00000040;
    pub const NODUMP_FL: u32 = 0x00000080;
    pub const NOATIME_FL: u32 = 0x00000100;
    pub const COMPRBLK_FL: u32 = 0x00000200;
    pub const DIRTY_FL: u32 = 0x00000400;
    pub const COMPRMODE_FL: u32 = 0x00000800;
    pub const JOURNAL_DATA_FL: u32 = 0x00004000;
    pub const NOCOMPR_FL: u32 = 0x00008000;
    pub const INDEX_FL: u32 = 0x00001000;
    pub const TOPDIR_FL: u32 = 0x00020000;
    pub const HUGE_FL: u32 = 0x00040000;
    pub const EXTENTS_FL: u32 = 0x00080000;
    pub const EA_INODE_FL: u32 = 0x00200000;
    pub const EOFBLOCKS_FL: u32 = 0x00400000;
    pub const INLINE_FL: u32 = 0x10000000;
    pub const PROJINHERIT_FL: u32 = 0x20000000;
    pub const ENCRYPT_FL: u32 = 0x08000000;
    pub const USER_VISIBLE_FL: u32 = 0x003DFFFF;
    pub const USER_MODIFIABLE_FL: u32 = 0x003801FF;
}

// ============================================================================
// ext4 File System Structure
// ============================================================================

/// ext4 file system
pub struct Ext4FileSystem {
    /// Superblock
    pub sb: Ext4SuperBlock,
    /// Block group descriptors
    pub group_desc: [Ext4GroupDesc; 1024],
    /// Number of block groups
    pub num_groups: u32,
    /// Block size
    pub block_size: u32,
    /// Inode size
    pub inode_size: u16,
    /// Block bitmap cache
    pub block_bitmap_cache: BlockBitmapCache,
    /// Inode bitmap cache
    pub inode_bitmap_cache: InodeBitmapCache,
    /// Journal
    pub journal: Option<Ext4Journal>,
    /// Statistics
    pub stats: Ext4Stats,
}

/// ext4 statistics
pub struct Ext4Stats {
    pub reads: AtomicU64,
    pub writes: AtomicU64,
    pub inode_reads: AtomicU64,
    pub inode_writes: AtomicU64,
    pub block_allocs: AtomicU64,
    pub block_frees: AtomicU64,
    pub inode_allocs: AtomicU64,
    pub inode_frees: AtomicU64,
}

impl Ext4Stats {
    pub const fn new() -> Self {
        Ext4Stats {
            reads: AtomicU64::new(0),
            writes: AtomicU64::new(0),
            inode_reads: AtomicU64::new(0),
            inode_writes: AtomicU64::new(0),
            block_allocs: AtomicU64::new(0),
            block_frees: AtomicU64::new(0),
            inode_allocs: AtomicU64::new(0),
            inode_frees: AtomicU64::new(0),
        }
    }
}

impl Ext4FileSystem {
    pub const fn new() -> Self {
        Ext4FileSystem {
            sb: Ext4SuperBlock::new(),
            group_desc: [Ext4GroupDesc::new(); 1024],
            num_groups: 0,
            block_size: EXT4_DEF_BLOCK_SIZE,
            inode_size: EXT4_DEF_INODE_SIZE,
            block_bitmap_cache: BlockBitmapCache::new(),
            inode_bitmap_cache: InodeBitmapCache::new(),
            journal: None,
            stats: Ext4Stats::new(),
        }
    }

    /// Initialize the file system
    pub fn init(&mut self) -> i32 {
        log_info!("ext4: Initializing filesystem");

        // Validate magic number
        if self.sb.s_magic != EXT4_SUPER_MAGIC {
            log_error!("ext4: Invalid magic number: 0x{:04X}", self.sb.s_magic);
            return -1;
        }

        // Calculate block size
        self.block_size = 1024 << self.sb.s_log_block_size;

        // Calculate inode size
        self.inode_size = if self.sb.s_inode_size == 0 {
            EXT4_DEF_INODE_SIZE
        } else {
            self.sb.s_inode_size
        };

        // Calculate number of block groups
        let blocks_count = self.sb.s_blocks_count_lo as u64 |
                          ((self.sb.s_blocks_count_hi as u64) << 32);
        let blocks_per_group = self.sb.s_blocks_per_group as u64;
        self.num_groups = ((blocks_count + blocks_per_group - 1) / blocks_per_group) as u32;

        log_info!("ext4: Block size: {} bytes", self.block_size);
        log_info!("ext4: Inode size: {} bytes", self.inode_size);
        log_info!("ext4: Block groups: {}", self.num_groups);
        log_info!("ext4: Total blocks: {}", blocks_count);
        log_info!("ext4: Total inodes: {}", self.sb.s_inodes_count);

        // Check if journal recovery is needed
        if (self.sb.s_feature_incompat & incompat_features::RECOVER) != 0 {
            log_info!("ext4: Journal recovery needed");
            // Journal recovery: replay the journal log
            // 1. Read the journal superblock
            // 2. Find the last committed transaction
            // 3. Replay all uncommitted transactions
            // 4. Clear the RECOVER feature flag
            // In a real implementation, this calls the journal module
            // crate::kernel::journal::recover(self.device);
        }

        0
    }

    /// Read a block from the device
    pub fn read_block(&mut self, block: u64, buf: &mut [u8]) -> i32 {
        self.stats.reads.fetch_add(1, Ordering::AcqRel);

        // TODO: Implement actual block read via device I/O
        let _ = (block, buf);
        0
    }

    /// Write a block to the device
    pub fn write_block(&mut self, block: u64, buf: &[u8]) -> i32 {
        self.stats.writes.fetch_add(1, Ordering::AcqRel);

        // TODO: Implement actual block write via device I/O
        let _ = (block, buf);
        0
    }

    /// Read an inode from the device
    pub fn read_inode(&mut self, ino: u32, inode: &mut Ext4Inode) -> i32 {
        self.stats.inode_reads.fetch_add(1, Ordering::AcqRel);

        if ino == 0 || ino > self.sb.s_inodes_count {
            return -1;
        }

        // Calculate the block group containing this inode
        let inodes_per_group = self.sb.s_inodes_per_group;
        let group = (ino - 1) / inodes_per_group;
        let local_ino = (ino - 1) % inodes_per_group;

        // Get the block group descriptor
        let gd = &self.group_desc[group as usize];

        // Calculate the inode table starting block
        let inode_table_block = gd.bg_inode_table_lo as u64 |
                               ((gd.bg_inode_table_hi as u64) << 32);

        // Calculate the inode offset within the table
        let inode_offset = local_ino as u64 * self.inode_size as u64;
        let block_offset = inode_offset / self.block_size as u64;
        let offset_in_block = inode_offset % self.block_size as u64;

        // Read the block containing the inode
        let block = inode_table_block + block_offset;

        // TODO: Read block and parse inode from the buffer
        let _ = (block, offset_in_block, inode);

        0
    }

    /// Write an inode to the device
    pub fn write_inode(&mut self, ino: u32, inode: &Ext4Inode) -> i32 {
        self.stats.inode_writes.fetch_add(1, Ordering::AcqRel);

        // TODO: Implement inode write to device
        let _ = (ino, inode);
        0
    }

    /// Allocate a new block
    pub fn alloc_block(&mut self, goal: u64) -> Option<u64> {
        self.stats.block_allocs.fetch_add(1, Ordering::AcqRel);

        // Start searching from the target block group
        let goal_group = if goal != 0 {
            (goal / self.sb.s_blocks_per_group as u64) as u32
        } else {
            0
        };

        // Traverse block groups to find a free block
        for i in 0..self.num_groups {
            let group = (goal_group + i) % self.num_groups;

            if let Some(block) = self.alloc_block_in_group(group) {
                return Some(block);
            }
        }

        None
    }

    /// Allocate a block in the specified block group
    fn alloc_block_in_group(&mut self, group: u32) -> Option<u64> {
        let gd = &self.group_desc[group as usize];

        // Check if there are free blocks in this group
        if gd.bg_free_blocks_count_lo == 0 {
            return None;
        }

        // TODO: Find free block in the block bitmap
        None
    }

    /// Free a block
    pub fn free_block(&mut self, block: u64) -> i32 {
        self.stats.block_frees.fetch_add(1, Ordering::AcqRel);

        // Calculate the block group containing this block
        let group = (block / self.sb.s_blocks_per_group as u64) as u32;
        let local_block = block % self.sb.s_blocks_per_group as u64;

        // TODO: Clear the bit in the block bitmap
        let _ = (group, local_block);

        0
    }

    /// Allocate a new inode
    pub fn alloc_inode(&mut self, parent: u32, mode: u32) -> Option<u32> {
        self.stats.inode_allocs.fetch_add(1, Ordering::AcqRel);

        // Calculate the block group of the parent directory
        let parent_group = if parent > 0 {
            (parent - 1) / self.sb.s_inodes_per_group
        } else {
            0
        };

        // Traverse block groups to find a free inode
        for i in 0..self.num_groups {
            let group = (parent_group + i) % self.num_groups;

            if let Some(ino) = self.alloc_inode_in_group(group) {
                // Initialize the new inode
                let _ = mode;
                return Some(ino);
            }
        }

        None
    }

    /// Allocate an inode in the specified block group
    fn alloc_inode_in_group(&mut self, group: u32) -> Option<u32> {
        let gd = &self.group_desc[group as usize];

        // Check if there are free inodes in this group
        if gd.bg_free_inodes_count_lo == 0 {
            return None;
        }

        // TODO: Find free inode in the inode bitmap
        None
    }

    /// Free an inode
    pub fn free_inode(&mut self, ino: u32) -> i32 {
        self.stats.inode_frees.fetch_add(1, Ordering::AcqRel);

        // Calculate the block group
        let group = (ino - 1) / self.sb.s_inodes_per_group;
        let local_ino = (ino - 1) % self.sb.s_inodes_per_group;

        // TODO: Clear the bit in the inode bitmap
        let _ = (group, local_ino);

        0
    }
}

// ============================================================================
// ext4 Superblock
// ============================================================================

/// ext4 superblock
#[repr(C, packed)]
pub struct Ext4SuperBlock {
    pub s_inodes_count: u32,           // Total inode count
    pub s_blocks_count_lo: u32,        // Total block count (low 32 bits)
    pub s_r_blocks_count_lo: u32,      // Reserved block count (low 32 bits)
    pub s_free_blocks_count_lo: u32,   // Free block count (low 32 bits)
    pub s_free_inodes_count: u32,      // Free inode count
    pub s_first_data_block: u32,       // First data block
    pub s_log_block_size: u32,         // Block size = 1024 << s_log_block_size
    pub s_log_cluster_size: u32,       // Cluster size
    pub s_blocks_per_group: u32,       // Blocks per group
    pub s_clusters_per_group: u32,     // Clusters per group
    pub s_inodes_per_group: u32,       // Inodes per group
    pub s_mtime: u32,                  // Last mount time
    pub s_wtime: u32,                  // Last write time
    pub s_mnt_count: u16,              // Mount count
    pub s_max_mnt_count: u16,          // Max mount count
    pub s_magic: u16,                  // Magic number
    pub s_state: u16,                  // File system state
    pub s_errors: u16,                 // Error handling method
    pub s_minor_rev_level: u16,        // Minor version number
    pub s_lastcheck: u32,              // Last check time
    pub s_checkinterval: u32,          // Check interval
    pub s_creator_os: u32,             // Creator OS
    pub s_rev_level: u32,              // Version number
    pub s_def_resuid: u16,             // Default reserved UID
    pub s_def_resgid: u16,             // Default reserved GID
    pub s_first_ino: u32,              // First non-reserved inode
    pub s_inode_size: u16,             // Inode size
    pub s_block_group_nr: u16,         // Block group number
    pub s_feature_compat: u32,         // Compatible features
    pub s_feature_incompat: u32,       // Incompatible features
    pub s_feature_ro_compat: u32,      // Read-only compatible features
    pub s_uuid: [u8; 16],              // UUID
    pub s_volume_name: [u8; 16],       // Volume name
    pub s_last_mounted: [u8; 64],      // Last mount point
    pub s_algorithm_usage_bitmap: u32, // Compression algorithm
    pub s_prealloc_blocks: u8,         // Preallocate block count
    pub s_prealloc_dir_blocks: u8,     // Directory preallocate block count
    pub s_reserved_gdt_blocks: u16,    // Reserved GDT block count
    pub s_journal_uuid: [u8; 16],      // Journal UUID
    pub s_journal_inum: u32,           // Journal inode
    pub s_journal_dev: u32,            // Journal device
    pub s_last_orphan: u32,            // Last orphan inode
    pub s_hash_seed: [u32; 4],         // HTREE hash seed
    pub s_def_hash_version: u8,        // Default hash version
    pub s_jnl_backup_type: u8,         // Journal backup type
    pub s_desc_size: u16,              // Descriptor size
    pub s_default_mount_opts: u32,     // Default mount options
    pub s_first_meta_bg: u32,          // First meta block group
    pub s_mkfs_time: u32,              // Creation time
    pub s_jnl_blocks: [u32; 17],       // Journal blocks
    pub s_blocks_count_hi: u32,        // Total block count (high 32 bits)
    pub s_r_blocks_count_hi: u32,      // Reserved block count (high 32 bits)
    pub s_free_blocks_count_hi: u32,   // Free block count (high 32 bits)
    pub s_min_extra_isize: u16,        // Min extra inode size
    pub s_want_extra_isize: u16,       // Desired extra inode size
    pub s_flags: u32,                  // Flags
    pub s_raid_stride: u16,            // RAID stride
    pub s_mmp_update_interval: u16,    // MMP update interval
    pub s_mmp_block: u64,              // MMP block
    pub s_raid_stripe_width: u32,      // RAID stripe width
    pub s_log_groups_per_flex: u8,     // Flex block group size
    pub s_checksum_type: u8,           // Checksum type
    pub s_encryption_level: u8,        // Encryption level
    pub s_reserved_pad: u8,            // Reserved
    pub s_kbytes_written: u64,         // Written byte count
    pub s_snapshot_inum: u32,          // Snapshot inode
    pub s_snapshot_id: u32,            // Snapshot ID
    pub s_snapshot_r_blocks_count: u64,// Snapshot reserved block count
    pub s_snapshot_list: u32,          // Snapshot list
    pub s_error_count: u32,            // Error count
    pub s_first_error_time: u32,       // First error time
    pub s_first_error_ino: u32,        // First error inode
    pub s_first_error_block: u64,      // First error block
    pub s_first_error_func: [u8; 32],  // First error function
    pub s_first_error_line: u32,       // First error line number
    pub s_last_error_time: u32,        // Last error time
    pub s_last_error_ino: u32,         // Last error inode
    pub s_last_error_line: u16,        // Last error line number
    pub s_last_error_trans: u16,       // Last error transaction
    pub s_last_error_block: u64,       // Last error block
    pub s_last_error_func: [u8; 32],   // Last error function
    pub s_mount_opts: [u8; 64],        // Mount options
    pub s_usr_quota_inum: u32,         // User quota inode
    pub s_grp_quota_inum: u32,         // Group quota inode
    pub s_overhead_blocks: u32,        // Overhead block count
    pub s_backup_bgs: [u32; 2],        // Backup block groups
    pub s_encrypt_algos: [u8; 4],      // Encryption algorithms
    pub s_encrypt_pw_salt: [u8; 16],   // Encryption password salt
    pub s_lpf_ino: u32,                // LPF inode
    pub s_prj_quota_inum: u32,         // Project quota inode
    pub s_checksum_seed: u32,          // Checksum seed
    pub s_reserved: [u8; 98],          // Reserved
    pub s_checksum: u32,               // Checksum
}

impl Ext4SuperBlock {
    pub const fn new() -> Self {
        Ext4SuperBlock {
            s_inodes_count: 0,
            s_blocks_count_lo: 0,
            s_r_blocks_count_lo: 0,
            s_free_blocks_count_lo: 0,
            s_free_inodes_count: 0,
            s_first_data_block: 0,
            s_log_block_size: 2,  // 4096 bytes
            s_log_cluster_size: 0,
            s_blocks_per_group: EXT4_BLOCKS_PER_GROUP,
            s_clusters_per_group: 0,
            s_inodes_per_group: EXT4_INODES_PER_GROUP,
            s_mtime: 0,
            s_wtime: 0,
            s_mnt_count: 0,
            s_max_mnt_count: 20,
            s_magic: EXT4_SUPER_MAGIC,
            s_state: 0,
            s_errors: 0,
            s_minor_rev_level: 0,
            s_lastcheck: 0,
            s_checkinterval: 0,
            s_creator_os: 0,
            s_rev_level: 1,
            s_def_resuid: 0,
            s_def_resgid: 0,
            s_first_ino: 11,
            s_inode_size: EXT4_DEF_INODE_SIZE,
            s_block_group_nr: 0,
            s_feature_compat: 0,
            s_feature_incompat: incompat_features::EXTENTS | incompat_features::FILETYPE,
            s_feature_ro_compat: ro_compat_features::SPARSE_SUPER | ro_compat_features::LARGE_FILE,
            s_uuid: [0; 16],
            s_volume_name: [0; 16],
            s_last_mounted: [0; 64],
            s_algorithm_usage_bitmap: 0,
            s_prealloc_blocks: 0,
            s_prealloc_dir_blocks: 0,
            s_reserved_gdt_blocks: 0,
            s_journal_uuid: [0; 16],
            s_journal_inum: 0,
            s_journal_dev: 0,
            s_last_orphan: 0,
            s_hash_seed: [0; 4],
            s_def_hash_version: 0,
            s_jnl_backup_type: 0,
            s_desc_size: EXT4_DESC_SIZE,
            s_default_mount_opts: 0,
            s_first_meta_bg: 0,
            s_mkfs_time: 0,
            s_jnl_blocks: [0; 17],
            s_blocks_count_hi: 0,
            s_r_blocks_count_hi: 0,
            s_free_blocks_count_hi: 0,
            s_min_extra_isize: 0,
            s_want_extra_isize: 0,
            s_flags: 0,
            s_raid_stride: 0,
            s_mmp_update_interval: 0,
            s_mmp_block: 0,
            s_raid_stripe_width: 0,
            s_log_groups_per_flex: 0,
            s_checksum_type: 0,
            s_encryption_level: 0,
            s_reserved_pad: 0,
            s_kbytes_written: 0,
            s_snapshot_inum: 0,
            s_snapshot_id: 0,
            s_snapshot_r_blocks_count: 0,
            s_snapshot_list: 0,
            s_error_count: 0,
            s_first_error_time: 0,
            s_first_error_ino: 0,
            s_first_error_block: 0,
            s_first_error_func: [0; 32],
            s_first_error_line: 0,
            s_last_error_time: 0,
            s_last_error_ino: 0,
            s_last_error_line: 0,
            s_last_error_trans: 0,
            s_last_error_block: 0,
            s_last_error_func: [0; 32],
            s_mount_opts: [0; 64],
            s_usr_quota_inum: 0,
            s_grp_quota_inum: 0,
            s_overhead_blocks: 0,
            s_backup_bgs: [0; 2],
            s_encrypt_algos: [0; 4],
            s_encrypt_pw_salt: [0; 16],
            s_lpf_ino: 0,
            s_prj_quota_inum: 0,
            s_checksum_seed: 0,
            s_reserved: [0; 98],
            s_checksum: 0,
        }
    }
}

// ============================================================================
// ext4 Block Group Descriptor
// ============================================================================

/// ext4 block group descriptor
#[repr(C, packed)]
pub struct Ext4GroupDesc {
    pub bg_block_bitmap_lo: u32,       // Block bitmap block (low 32 bits)
    pub bg_inode_bitmap_lo: u32,       // Inode bitmap block (low 32 bits)
    pub bg_inode_table_lo: u32,        // Inode table starting block (low 32 bits)
    pub bg_free_blocks_count_lo: u16,  // Free block count (low 16 bits)
    pub bg_free_inodes_count_lo: u16,  // Free inode count (low 16 bits)
    pub bg_used_dirs_count_lo: u16,    // Used directory count (low 16 bits)
    pub bg_flags: u16,                 // Flags
    pub bg_exclude_bitmap_lo: u32,     // Exclude bitmap (low 32 bits)
    pub bg_block_bitmap_csum_lo: u16,  // Block bitmap checksum (low 16 bits)
    pub bg_inode_bitmap_csum_lo: u16,  // Inode bitmap checksum (low 16 bits)
    pub bg_itable_unused_lo: u16,      // Unused inode table entries (low 16 bits)
    pub bg_checksum: u16,              // Checksum
    pub bg_block_bitmap_hi: u32,       // Block bitmap block (high 32 bits)
    pub bg_inode_bitmap_hi: u32,       // Inode bitmap block (high 32 bits)
    pub bg_inode_table_hi: u32,        // Inode table starting block (high 32 bits)
    pub bg_free_blocks_count_hi: u16,  // Free block count (high 16 bits)
    pub bg_free_inodes_count_hi: u16,  // Free inode count (high 16 bits)
    pub bg_used_dirs_count_hi: u16,    // Used directory count (high 16 bits)
    pub bg_itable_unused_hi: u16,      // Unused inode table entries (high 16 bits)
    pub bg_exclude_bitmap_hi: u32,     // Exclude bitmap (high 32 bits)
    pub bg_block_bitmap_csum_hi: u16,  // Block bitmap checksum (high 16 bits)
    pub bg_inode_bitmap_csum_hi: u16,  // Inode bitmap checksum (high 16 bits)
    pub bg_reserved: u32,              // Reserved
}

impl Ext4GroupDesc {
    pub const fn new() -> Self {
        Ext4GroupDesc {
            bg_block_bitmap_lo: 0,
            bg_inode_bitmap_lo: 0,
            bg_inode_table_lo: 0,
            bg_free_blocks_count_lo: 0,
            bg_free_inodes_count_lo: 0,
            bg_used_dirs_count_lo: 0,
            bg_flags: 0,
            bg_exclude_bitmap_lo: 0,
            bg_block_bitmap_csum_lo: 0,
            bg_inode_bitmap_csum_lo: 0,
            bg_itable_unused_lo: 0,
            bg_checksum: 0,
            bg_block_bitmap_hi: 0,
            bg_inode_bitmap_hi: 0,
            bg_inode_table_hi: 0,
            bg_free_blocks_count_hi: 0,
            bg_free_inodes_count_hi: 0,
            bg_used_dirs_count_hi: 0,
            bg_itable_unused_hi: 0,
            bg_exclude_bitmap_hi: 0,
            bg_block_bitmap_csum_hi: 0,
            bg_inode_bitmap_csum_hi: 0,
            bg_reserved: 0,
        }
    }
}

// ============================================================================
// ext4 Inode
// ============================================================================

/// ext4 inode
#[repr(C, packed)]
pub struct Ext4Inode {
    pub i_mode: u16,                   // File mode
    pub i_uid: u16,                    // User ID (low 16 bits)
    pub i_size_lo: u32,                // File size (low 32 bits)
    pub i_atime: u32,                  // Access time
    pub i_ctime: u32,                  // State change time
    pub i_mtime: u32,                  // Modification time
    pub i_dtime: u32,                  // Deletion time
    pub i_gid: u16,                    // Group ID (low 16 bits)
    pub i_links_count: u16,            // Link count
    pub i_blocks_lo: u32,              // Block count (low 32 bits)
    pub i_flags: u32,                  // Inode flags
    pub i_osd1: u32,                   // OS-specific field 1
    pub i_block: [u32; 15],            // Block pointers / extent tree
    pub i_generation: u32,             // Generation number
    pub i_file_acl_lo: u32,            // File ACL (low 32 bits)
    pub i_size_hi: u32,                // File size (high 32 bits)
    pub i_obso_faddr: u32,             // Obsolete field
    pub i_obso_fblocks: u32,           // Obsolete field
    pub i_obso_ffragment: u32,         // Obsolete field
    pub i_obso_fuid: u32,              // Obsolete field
    pub i_uid_hi: u16,                 // User ID (high 16 bits)
    pub i_gid_hi: u16,                 // Group ID (high 16 bits)
    pub i_checksum_lo: u16,            // Checksum (low 16 bits)
    pub i_reserved: u16,               // Reserved
    pub i_extra_isize: u16,            // Extra inode size
    pub i_checksum_hi: u16,            // Checksum (high 16 bits)
    pub i_ctime_extra: u32,            // Creation time extra
    pub i_mtime_extra: u32,            // Modification time extra
    pub i_atime_extra: u32,            // Access time extra
    pub i_crtime: u32,                 // Creation time
    pub i_crtime_extra: u32,           // Creation time extra
    pub i_version_hi: u32,             // Version (high 32 bits)
    pub i_projid: u32,                 // Project ID
}

impl Ext4Inode {
    pub const fn new() -> Self {
        Ext4Inode {
            i_mode: 0,
            i_uid: 0,
            i_size_lo: 0,
            i_atime: 0,
            i_ctime: 0,
            i_mtime: 0,
            i_dtime: 0,
            i_gid: 0,
            i_links_count: 0,
            i_blocks_lo: 0,
            i_flags: 0,
            i_osd1: 0,
            i_block: [0; 15],
            i_generation: 0,
            i_file_acl_lo: 0,
            i_size_hi: 0,
            i_obso_faddr: 0,
            i_obso_fblocks: 0,
            i_obso_ffragment: 0,
            i_obso_fuid: 0,
            i_uid_hi: 0,
            i_gid_hi: 0,
            i_checksum_lo: 0,
            i_reserved: 0,
            i_extra_isize: 0,
            i_checksum_hi: 0,
            i_ctime_extra: 0,
            i_mtime_extra: 0,
            i_atime_extra: 0,
            i_crtime: 0,
            i_crtime_extra: 0,
            i_version_hi: 0,
            i_projid: 0,
        }
    }

    /// Get the file size as a 64-bit value
    pub fn get_size(&self) -> u64 {
        self.i_size_lo as u64 | ((self.i_size_hi as u64) << 32)
    }

    /// Set the file size from a 64-bit value
    pub fn set_size(&mut self, size: u64) {
        self.i_size_lo = size as u32;
        self.i_size_hi = (size >> 32) as u32;
    }

    /// Check if the inode uses extents
    pub fn has_extents(&self) -> bool {
        (self.i_flags & inode_flags::EXTENTS_FL) != 0
    }

    /// Check if the inode is a directory
    pub fn is_dir(&self) -> bool {
        (self.i_mode & 0xF000) == 0x4000
    }

    /// Check if the inode is a regular file
    pub fn is_regular(&self) -> bool {
        (self.i_mode & 0xF000) == 0x8000
    }

    /// Check if the inode is a symbolic link
    pub fn is_symlink(&self) -> bool {
        (self.i_mode & 0xF000) == 0xA000
    }
}

// ============================================================================
// Bitmap Cache
// ============================================================================

/// Block bitmap cache
pub struct BlockBitmapCache {
    pub cache: [u8; 4096],
    pub group: u32,
    pub valid: bool,
}

impl BlockBitmapCache {
    pub const fn new() -> Self {
        BlockBitmapCache {
            cache: [0; 4096],
            group: 0xFFFFFFFF,
            valid: false,
        }
    }
}

/// Inode bitmap cache
pub struct InodeBitmapCache {
    pub cache: [u8; 4096],
    pub group: u32,
    pub valid: bool,
}

impl InodeBitmapCache {
    pub const fn new() -> Self {
        InodeBitmapCache {
            cache: [0; 4096],
            group: 0xFFFFFFFF,
            valid: false,
        }
    }
}

// ============================================================================
// ext4 Journal
// ============================================================================

/// ext4 journal
pub struct Ext4Journal {
    pub j_inode: u32,
    pub j_dev: u32,
    pub j_maxlen: u32,
    pub j_first: u32,
    pub j_head: u32,
    pub j_tail: u32,
    pub j_transaction: Option<Ext4Transaction>,
}

/// ext4 transaction
pub struct Ext4Transaction {
    pub t_tid: u64,
    pub t_state: u8,
    pub t_buffers: u32,
}

/// Global ext4 file system instance
static mut EXT4_FS: Ext4FileSystem = Ext4FileSystem::new();

/// Get the global ext4 file system instance
pub fn get_ext4_fs() -> &'static mut Ext4FileSystem {
    // SAFETY: Single-threaded kernel initialization; access is synchronized externally.
    unsafe { &mut EXT4_FS }
}

/// Initialize the ext4 file system
pub fn init_ext4() {
    let fs = get_ext4_fs();
    fs.init();
}
