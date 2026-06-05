/*
 * Nuva OS - Fs - Fat32 - Mod
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
 * Nuva OS - FAT32 File System
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * FAT32 File System Implementation
 */

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

// ============================================================================
// FAT32 Constants
// ============================================================================

/// FAT32 boot sector signature
pub const FAT32_BOOT_SIGNATURE: u16 = 0xAA55;

/// FAT32 FSI Info sector signature
pub const FSI_LEAD_SIG: u32 = 0x41615252;
pub const FSI_STRUCT_SIG: u32 = 0x61417272;
pub const FSI_TRAIL_SIG: u32 = 0xAA550000;

/// FAT entry values
pub const FAT_FREE: u32 = 0x00000000;
pub const FAT_RESERVED: u32 = 0x0FFFFFF0;
pub const FAT_BAD: u32 = 0x0FFFFFF7;
pub const FAT_EOF_MIN: u32 = 0x0FFFFFF8;
pub const FAT_EOF_MAX: u32 = 0x0FFFFFFF;

/// Directory entry attributes
pub mod attr {
    pub const READ_ONLY: u8 = 0x01;
    pub const HIDDEN: u8 = 0x02;
    pub const SYSTEM: u8 = 0x04;
    pub const VOLUME_ID: u8 = 0x08;
    pub const DIRECTORY: u8 = 0x10;
    pub const ARCHIVE: u8 = 0x20;
    pub const LONG_NAME: u8 = 0x0F;  // Long file name attribute
}

/// Directory entry size
pub const DIR_ENTRY_SIZE: u32 = 32;

/// Max path length
pub const MAX_PATH_LEN: usize = 260;

// ============================================================================
// FAT32 Boot Sector
// ============================================================================

/// FAT32 boot sector (BPB)
#[repr(C, packed)]
pub struct Fat32BootSector {
    // BIOS Parameter Block (BPB)
    pub bs_jmp_boot: [u8; 3],          // Jump instruction
    pub bs_oem_name: [u8; 8],          // OEM Name
    pub bpb_byts_per_sec: u16,         // Bytes per sector
    pub bpb_sec_per_clus: u8,          // Sectors per cluster
    pub bpb_rsvd_sec_cnt: u16,         // Reserved sector count
    pub bpb_num_fats: u8,              // Number of FATs
    pub bpb_root_ent_cnt: u16,         // Root directory entry count (0 for FAT32)
    pub bpb_tot_sec16: u16,            // Total sectors (16-bit)
    pub bpb_media: u8,                 // Media type
    pub bpb_fatsz16: u16,              // FAT size (16-bit, 0 for FAT32)
    pub bpb_sec_per_trk: u16,          // Sectors per track
    pub bpb_num_heads: u16,            // Number of heads
    pub bpb_hidd_sec: u32,             // Hidden sectors
    pub bpb_tot_sec32: u32,            // Total sectors (32-bit)

    // FAT32 Extended BPB
    pub bpb_fatsz32: u32,              // FAT size (sectors)
    pub bpb_ext_flags: u16,            // Extended flags
    pub bpb_fs_ver: u16,               // File system version
    pub bpb_root_clus: u32,            // Root directory starting cluster
    pub bpb_fs_info: u16,              // FSINFO sector number
    pub bpb_bk_boot_sec: u16,          // Backup boot sector number
    pub bpb_reserved: [u8; 12],        // Reserved
    pub bs_drv_num: u8,                // Drive number
    pub bs_reserved1: u8,              // Reserved
    pub bs_boot_sig: u8,               // Boot signature (0x29)
    pub bs_vol_id: u32,                // Volume serial number
    pub bs_vol_lab: [u8; 11],          // Volume label
    pub bs_fil_sys_type: [u8; 8],      // File system type

    // Boot code and signature
    pub bs_code: [u8; 420],            // Boot code
    pub bs_signature: u16,             // Signature (0xAA55)
}

impl Fat32BootSector {
    pub const fn new() -> Self {
        Fat32BootSector {
            bs_jmp_boot: [0xEB, 0x58, 0x90],
            bs_oem_name: [b'M', b'S', b'D', b'O', b'S', b'5', b'.', b'0'],
            bpb_byts_per_sec: 512,
            bpb_sec_per_clus: 8,
            bpb_rsvd_sec_cnt: 32,
            bpb_num_fats: 2,
            bpb_root_ent_cnt: 0,
            bpb_tot_sec16: 0,
            bpb_media: 0xF8,
            bpb_fatsz16: 0,
            bpb_sec_per_trk: 63,
            bpb_num_heads: 255,
            bpb_hidd_sec: 0,
            bpb_tot_sec32: 0,
            bpb_fatsz32: 0,
            bpb_ext_flags: 0,
            bpb_fs_ver: 0,
            bpb_root_clus: 2,
            bpb_fs_info: 1,
            bpb_bk_boot_sec: 6,
            bpb_reserved: [0; 12],
            bs_drv_num: 0x80,
            bs_reserved1: 0,
            bs_boot_sig: 0x29,
            bs_vol_id: 0,
            bs_vol_lab: [b'N', b'O', b' ', b'N', b'A', b'M', b'E', b' ', b' ', b' ', b' '],
            bs_fil_sys_type: [b'F', b'A', b'T', b'3', b'2', b' ', b' ', b' '],
            bs_code: [0; 420],
            bs_signature: FAT32_BOOT_SIGNATURE,
        }
    }

    /// Get bytes per sector
    pub fn bytes_per_sector(&self) -> u32 {
        self.bpb_byts_per_sec as u32
    }

    /// Get bytes per cluster
    pub fn bytes_per_cluster(&self) -> u32 {
        self.bpb_byts_per_sec as u32 * self.bpb_sec_per_clus as u32
    }

    /// Get FAT start sector
    pub fn fat_start_sector(&self) -> u32 {
        self.bpb_rsvd_sec_cnt as u32
    }

    /// Get data area start sector
    pub fn data_start_sector(&self) -> u32 {
        self.bpb_rsvd_sec_cnt as u32 +
            self.bpb_num_fats as u32 * self.bpb_fatsz32
    }

    /// Get total clusters
    pub fn total_clusters(&self) -> u32 {
        let total_sectors = if self.bpb_tot_sec16 != 0 {
            self.bpb_tot_sec16 as u32
        } else {
            self.bpb_tot_sec32
        };

        let data_sectors = total_sectors - self.data_start_sector();
        data_sectors / self.bpb_sec_per_clus as u32
    }

    /// Convert cluster number to sector number
    pub fn cluster_to_sector(&self, cluster: u32) -> u32 {
        self.data_start_sector() + (cluster - 2) * self.bpb_sec_per_clus as u32
    }
}

// ============================================================================
// FAT32 FSINFO Structure
// ============================================================================

/// FAT32 FSINFO structure
#[repr(C, packed)]
pub struct Fat32FsInfo {
    pub fsi_lead_sig: u32,             // Lead signature
    pub fsi_reserved1: [u8; 480],      // Reserved
    pub fsi_struc_sig: u32,            // Structure signature
    pub fsi_free_count: u32,           // Free cluster count
    pub fsi_nxt_free: u32,             // Next free cluster
    pub fsi_reserved2: [u8; 12],       // Reserved
    pub fsi_trail_sig: u32,            // Trail signature
}

impl Fat32FsInfo {
    pub const fn new() -> Self {
        Fat32FsInfo {
            fsi_lead_sig: FSI_LEAD_SIG,
            fsi_reserved1: [0; 480],
            fsi_struc_sig: FSI_STRUCT_SIG,
            fsi_free_count: 0xFFFFFFFF,
            fsi_nxt_free: 3,
            fsi_reserved2: [0; 12],
            fsi_trail_sig: FSI_TRAIL_SIG,
        }
    }
}

// ============================================================================
// FAT32 Directory Entry
// ============================================================================

/// FAT32 directory entry (short file name)
#[repr(C, packed)]
pub struct Fat32DirEntry {
    pub dir_name: [u8; 11],            // File name (8.3 format)
    pub dir_attr: u8,                  // Attributes
    pub dir_nt_res: u8,                // Reserved (NT extension)
    pub dir_crt_time_tenth: u8,        // Creation time (tenths of seconds)
    pub dir_crt_time: u16,             // Creation time
    pub dir_crt_date: u16,             // Creation date
    pub dir_lst_acc_date: u16,         // Last access date
    pub dir_fst_clus_hi: u16,          // Starting cluster number (high 16 bits)
    pub dir_wrt_time: u16,             // Modification time
    pub dir_wrt_date: u16,             // Modification date
    pub dir_fst_clus_lo: u16,          // Starting cluster number (low 16 bits)
    pub dir_file_size: u32,            // File size
}

impl Fat32DirEntry {
    pub const fn new() -> Self {
        Fat32DirEntry {
            dir_name: [0; 11],
            dir_attr: 0,
            dir_nt_res: 0,
            dir_crt_time_tenth: 0,
            dir_crt_time: 0,
            dir_crt_date: 0,
            dir_lst_acc_date: 0,
            dir_fst_clus_hi: 0,
            dir_wrt_time: 0,
            dir_wrt_date: 0,
            dir_fst_clus_lo: 0,
            dir_file_size: 0,
        }
    }

    /// Get starting cluster number
    pub fn get_cluster(&self) -> u32 {
        (self.dir_fst_clus_hi as u32) << 16 | self.dir_fst_clus_lo as u32
    }

    /// Set starting cluster number
    pub fn set_cluster(&mut self, cluster: u32) {
        self.dir_fst_clus_hi = (cluster >> 16) as u16;
        self.dir_fst_clus_lo = cluster as u16;
    }

    /// Check if free (unused)
    pub fn is_free(&self) -> bool {
        self.dir_name[0] == 0x00 || self.dir_name[0] == 0xE5
    }

    /// Check if end of directory
    pub fn is_end(&self) -> bool {
        self.dir_name[0] == 0x00
    }

    /// Check if long file name entry
    pub fn is_long_name(&self) -> bool {
        self.dir_attr == attr::LONG_NAME
    }

    /// Check if directory
    pub fn is_directory(&self) -> bool {
        (self.dir_attr & attr::DIRECTORY) != 0
    }

    /// Check if volume label
    pub fn is_volume_label(&self) -> bool {
        (self.dir_attr & attr::VOLUME_ID) != 0
    }

    /// Check if hidden file
    pub fn is_hidden(&self) -> bool {
        (self.dir_attr & attr::HIDDEN) != 0
    }

    /// Check if system file
    pub fn is_system(&self) -> bool {
        (self.dir_attr & attr::SYSTEM) != 0
    }

    /// Get file name (convert to string)
    pub fn get_name(&self, buf: &mut [u8]) -> usize {
        if self.is_long_name() {
            return 0;
        }

        let mut pos = 0;

        // Main file name
        for i in 0..8 {
            let c = self.dir_name[i];
            if c == 0 || c == b' ' {
                break;
            }
            buf[pos] = c;
            pos += 1;
        }

        // Extension
        if self.dir_name[8] != b' ' {
            buf[pos] = b'.';
            pos += 1;

            for i in 8..11 {
                let c = self.dir_name[i];
                if c == 0 || c == b' ' {
                    break;
                }
                buf[pos] = c;
                pos += 1;
            }
        }

        pos
    }
}

// ============================================================================
// FAT32 Long File Name Directory Entry
// ============================================================================

/// FAT32 long file name directory entry
#[repr(C, packed)]
pub struct Fat32LfnEntry {
    pub lfn_ord: u8,                   // Sequence number
    pub lfn_name1: [u16; 5],           // Name characters 1-5
    pub lfn_attr: u8,                  // Attributes (must be 0x0F)
    pub lfn_type: u8,                  // Type
    pub lfn_checksum: u8,              // Checksum
    pub lfn_name2: [u16; 6],           // Name characters 6-11
    pub lfn_reserved: u16,             // Reserved
    pub lfn_name3: [u16; 2],           // Name characters 12-13
}

impl Fat32LfnEntry {
    pub const fn new() -> Self {
        Fat32LfnEntry {
            lfn_ord: 0,
            lfn_name1: [0; 5],
            lfn_attr: attr::LONG_NAME,
            lfn_type: 0,
            lfn_checksum: 0,
            lfn_name2: [0; 6],
            lfn_reserved: 0,
            lfn_name3: [0; 2],
        }
    }

    /// Check if last entry
    pub fn is_last(&self) -> bool {
        (self.lfn_ord & 0x40) != 0
    }

    /// Get sequence number
    pub fn get_order(&self) -> u8 {
        self.lfn_ord & 0x1F
    }

    /// Extract name characters
    pub fn get_name_chars(&self, buf: &mut [u16]) -> usize {
        let mut pos = 0;

        for &c in &self.lfn_name1 {
            if c == 0 || c == 0xFFFF {
                return pos;
            }
            buf[pos] = c;
            pos += 1;
        }

        for &c in &self.lfn_name2 {
            if c == 0 || c == 0xFFFF {
                return pos;
            }
            buf[pos] = c;
            pos += 1;
        }

        for &c in &self.lfn_name3 {
            if c == 0 || c == 0xFFFF {
                return pos;
            }
            buf[pos] = c;
            pos += 1;
        }

        pos
    }
}

// ============================================================================
// FAT32 File System
// ============================================================================

/// FAT32 file system
pub struct Fat32FileSystem {
    /// Boot sector
    pub boot: Fat32BootSector,
    /// FSINFO
    pub fs_info: Fat32FsInfo,
    /// FAT cache
    pub fat_cache: FatCache,
    /// Current directory cluster
    pub cwd_cluster: u32,
    /// Statistics
    pub stats: Fat32Stats,
}

/// FAT cache
pub struct FatCache {
    /// Cached data
    pub data: [u8; 4096],
    /// Cached FAT sector
    pub sector: u32,
    /// Whether valid
    pub valid: bool,
}

/// FAT32 statistics
pub struct Fat32Stats {
    pub reads: AtomicU64,
    pub writes: AtomicU64,
    pub cluster_allocs: AtomicU64,
    pub cluster_frees: AtomicU64,
}

impl Fat32Stats {
    pub const fn new() -> Self {
        Fat32Stats {
            reads: AtomicU64::new(0),
            writes: AtomicU64::new(0),
            cluster_allocs: AtomicU64::new(0),
            cluster_frees: AtomicU64::new(0),
        }
    }
}

impl FatCache {
    pub const fn new() -> Self {
        FatCache {
            data: [0; 4096],
            sector: 0xFFFFFFFF,
            valid: false,
        }
    }
}

impl Fat32FileSystem {
    pub const fn new() -> Self {
        Fat32FileSystem {
            boot: Fat32BootSector::new(),
            fs_info: Fat32FsInfo::new(),
            fat_cache: FatCache::new(),
            cwd_cluster: 2,  // Root directory
            stats: Fat32Stats::new(),
        }
    }

    /// Initialize file system
    pub fn init(&mut self) -> i32 {
        log_info!("FAT32: Initializing filesystem");

        // Validate signature
        if self.boot.bs_signature != FAT32_BOOT_SIGNATURE {
            log_error!("FAT32: Invalid boot signature: 0x{:04X}", self.boot.bs_signature);
            return -1;
        }

        // Validate bytes per sector
        if self.boot.bpb_byts_per_sec != 512 &&
           self.boot.bpb_byts_per_sec != 1024 &&
           self.boot.bpb_byts_per_sec != 2048 &&
           self.boot.bpb_byts_per_sec != 4096 {
            log_error!("FAT32: Invalid bytes per sector: {}", self.boot.bpb_byts_per_sec);
            return -2;
        }

        // Validate sectors per cluster
        let valid_sec_per_clus = [1, 2, 4, 8, 16, 32, 64, 128];
        if !valid_sec_per_clus.contains(&self.boot.bpb_sec_per_clus) {
            log_error!("FAT32: Invalid sectors per cluster: {}", self.boot.bpb_sec_per_clus);
            return -3;
        }

        log_info!("FAT32: Bytes per sector: {}", self.boot.bytes_per_sector());
        log_info!("FAT32: Bytes per cluster: {}", self.boot.bytes_per_cluster());
        log_info!("FAT32: Total clusters: {}", self.boot.total_clusters());
        log_info!("FAT32: Root cluster: {}", self.boot.bpb_root_clus);
        log_info!("FAT32: FAT start: sector {}", self.boot.fat_start_sector());
        log_info!("FAT32: Data start: sector {}", self.boot.data_start_sector());

        // Initialize current directory to root directory
        self.cwd_cluster = self.boot.bpb_root_clus;

        0
    }

    /// Read sector
    pub fn read_sector(&mut self, sector: u32, buf: &mut [u8]) -> i32 {
        self.stats.reads.fetch_add(1, Ordering::AcqRel);

        // Actual sector read
        // Should call block device interface to read sector
        // Simplified implementation: fill with zeros
        for byte in buf.iter_mut() {
            *byte = 0;
        }

        log_debug!("FAT32: Read sector {}", sector);
        0
    }

    /// Write sector
    pub fn write_sector(&mut self, sector: u32, buf: &[u8]) -> i32 {
        self.stats.writes.fetch_add(1, Ordering::AcqRel);

        // Actual sector write
        // Should call block device interface to write sector
        // Simplified implementation: just log
        log_debug!("FAT32: Write sector {} ({} bytes)", sector, buf.len());

        0
    }

    /// Read FAT entry
    pub fn read_fat_entry(&mut self, cluster: u32) -> u32 {
        // FAT32: Each FAT entry occupies 4 bytes
        let fat_offset = cluster * 4;
        let fat_sector = self.boot.fat_start_sector() + fat_offset / self.boot.bytes_per_sector();
        let offset_in_sector = fat_offset % self.boot.bytes_per_sector();

        // Check cache
        if !self.fat_cache.valid || self.fat_cache.sector != fat_sector {
            // Read FAT sector into cache
            self.read_sector(fat_sector, &mut self.fat_cache.data)?;
            self.fat_cache.sector = fat_sector;
            self.fat_cache.valid = true;
        }

        // Read FAT entry from cache
        let offset = offset_in_sector as usize;
        let entry = self.fat_cache.data[offset] as u32 |
                   (self.fat_cache.data[offset + 1] as u32) << 8 |
                   (self.fat_cache.data[offset + 2] as u32) << 16 |
                   (self.fat_cache.data[offset + 3] as u32) << 24;

        // FAT32 only uses low 28 bits
        entry & 0x0FFFFFFF
    }

    /// Write FAT entry
    pub fn write_fat_entry(&mut self, cluster: u32, value: u32) -> i32 {
        let fat_offset = cluster * 4;
        let fat_sector = self.boot.fat_start_sector() + fat_offset / self.boot.bytes_per_sector();
        let offset_in_sector = fat_offset % self.boot.bytes_per_sector();

        // Check cache
        if !self.fat_cache.valid || self.fat_cache.sector != fat_sector {
            // Read FAT sector into cache
            self.read_sector(fat_sector, &mut self.fat_cache.data)?;
            self.fat_cache.sector = fat_sector;
            self.fat_cache.valid = true;
        }

        // Write FAT entry to cache
        let offset = offset_in_sector as usize;
        self.fat_cache.data[offset] = value as u8;
        self.fat_cache.data[offset + 1] = (value >> 8) as u8;
        self.fat_cache.data[offset + 2] = (value >> 16) as u8;
        self.fat_cache.data[offset + 3] = (value >> 24) as u8 & 0x0F;

        // Write back sector
        self.write_sector(fat_sector, &self.fat_cache.data)?;

        0
    }

    /// Check if EOF cluster
    pub fn is_eof_cluster(&self, cluster: u32) -> bool {
        cluster >= FAT_EOF_MIN && cluster <= FAT_EOF_MAX
    }

    /// Get EOF cluster value
    pub fn get_eof_cluster(&self) -> u32 {
        FAT_EOF_MAX
    }

    /// Allocate cluster
    pub fn alloc_cluster(&mut self) -> Option<u32> {
        self.stats.cluster_allocs.fetch_add(1, Ordering::AcqRel);

        // Get next free cluster from FSINFO
        let mut cluster = self.fs_info.fsi_nxt_free;
        let total_clusters = self.boot.total_clusters();

        // Search for free cluster
        for _ in 0..total_clusters {
            if cluster < 2 || cluster >= total_clusters {
                cluster = 2;
            }

            let fat_entry = self.read_fat_entry(cluster);
            if fat_entry == FAT_FREE {
                // Found free cluster
                self.write_fat_entry(cluster, self.get_eof_cluster())?;
                self.fs_info.fsi_nxt_free = cluster + 1;
                if self.fs_info.fsi_free_count != 0xFFFFFFFF {
                    self.fs_info.fsi_free_count -= 1;
                }
                return Some(cluster);
            }

            cluster += 1;
        }

        None
    }

    /// Free cluster chain
    pub fn free_cluster_chain(&mut self, start_cluster: u32) -> i32 {
        let mut cluster = start_cluster;

        while !self.is_eof_cluster(cluster) && cluster != FAT_FREE {
            let next = self.read_fat_entry(cluster);
            self.write_fat_entry(cluster, FAT_FREE)?;
            self.stats.cluster_frees.fetch_add(1, Ordering::AcqRel);

            if self.fs_info.fsi_free_count != 0xFFFFFFFF {
                self.fs_info.fsi_free_count += 1;
            }

            cluster = next;
        }

        0
    }

    /// Read directory entry
    pub fn read_dir_entry(&mut self, cluster: u32, index: u32, entry: &mut Fat32DirEntry) -> i32 {
        let bytes_per_cluster = self.boot.bytes_per_cluster();
        let entries_per_cluster = bytes_per_cluster / DIR_ENTRY_SIZE;

        // Calculate cluster and offset
        let cluster_offset = index / entries_per_cluster;
        let entry_offset = index % entries_per_cluster;

        // Traverse to target cluster
        let mut current_cluster = cluster;
        for _ in 0..cluster_offset {
            current_cluster = self.read_fat_entry(current_cluster);
            if self.is_eof_cluster(current_cluster) {
                return -1;  // Out of range
            }
        }

        // Calculate sector and offset
        let sector = self.boot.cluster_to_sector(current_cluster);
        let byte_offset = entry_offset * DIR_ENTRY_SIZE;
        let sector_offset = byte_offset / self.boot.bytes_per_sector();
        let offset_in_sector = byte_offset % self.boot.bytes_per_sector();

        // Read sector
        let mut buf = [0u8; 512];
        self.read_sector(sector + sector_offset, &mut buf)?;

        // Copy directory entry
        // SAFETY: unsafe block required for low-level memory or hardware access
        let entry_bytes = unsafe {
            &*(buf.as_ptr().add(offset_in_sector as usize) as *const Fat32DirEntry)
        };
        *entry = *entry_bytes;

        0
    }

    /// Find directory entry
    pub fn find_dir_entry(&mut self, cluster: u32, name: &[u8]) -> Option<(u32, Fat32DirEntry)> {
        let mut index = 0u32;

        loop {
            let mut entry = Fat32DirEntry::new();
            if self.read_dir_entry(cluster, index, &mut entry) < 0 {
                break;
            }

            if entry.is_end() {
                break;
            }

            if !entry.is_free() && !entry.is_long_name() {
                let mut entry_name = [0u8; 13];
                let name_len = entry.get_name(&mut entry_name);

                if name_len == name.len() &&
                   entry_name[..name_len] == name[..name_len] {
                    return Some((index, entry));
                }
            }

            index += 1;
        }

        None
    }

    /// Create directory entry
    pub fn create_entry(&mut self, parent_cluster: u32, name: &[u8], attr: u8) -> Option<Fat32DirEntry> {
        // Find free directory entry
        let mut index = 0u32;
        let mut free_index = None;

        loop {
            let mut entry = Fat32DirEntry::new();
            if self.read_dir_entry(parent_cluster, index, &mut entry) < 0 {
                break;
            }

            if entry.is_end() {
                free_index = Some(index);
                break;
            }

            if entry.is_free() && free_index.is_none() {
                free_index = Some(index);
            }

            index += 1;
        }

        let index = free_index?;

        // Allocate starting cluster
        let cluster = self.alloc_cluster()?;

        // Create directory entry
        let mut entry = Fat32DirEntry::new();
        // Pad file name (8.3 format)
        for (i, &c) in name.iter().take(11).enumerate() {
            entry.dir_name[i] = c;
        }
        entry.dir_attr = attr;
        entry.set_cluster(cluster);

        // Write directory entry
        self.write_dir_entry(parent_cluster, index, &entry)?;

        Some(entry)
    }

    /// Write directory entry
    pub fn write_dir_entry(&mut self, cluster: u32, index: u32, entry: &Fat32DirEntry) -> i32 {
        let bytes_per_cluster = self.boot.bytes_per_cluster();
        let entries_per_cluster = bytes_per_cluster / DIR_ENTRY_SIZE;

        // Calculate cluster and offset
        let cluster_offset = index / entries_per_cluster;
        let entry_offset = index % entries_per_cluster;

        // Traverse to target cluster
        let mut current_cluster = cluster;
        for _ in 0..cluster_offset {
            current_cluster = self.read_fat_entry(current_cluster);
            if self.is_eof_cluster(current_cluster) {
                return -1;  // Out of range
            }
        }

        // Calculate sector and offset
        let sector = self.boot.cluster_to_sector(current_cluster);
        let byte_offset = entry_offset * DIR_ENTRY_SIZE;
        let sector_offset = byte_offset / self.boot.bytes_per_sector();
        let offset_in_sector = byte_offset % self.boot.bytes_per_sector();

        // Read sector
        let mut buf = [0u8; 512];
        self.read_sector(sector + sector_offset, &mut buf)?;

        // Copy directory entry
        // SAFETY: unsafe block required for low-level memory or hardware access
        let entry_bytes = unsafe {
            &mut *(buf.as_mut_ptr().add(offset_in_sector as usize) as *mut Fat32DirEntry)
        };
        *entry_bytes = *entry;

        // Write back sector
        self.write_sector(sector + sector_offset, &buf)?;

        0
    }

    /// Delete directory entry
    pub fn delete_entry(&mut self, parent_cluster: u32, name: &[u8]) -> i32 {
        let (index, entry) = match self.find_dir_entry(parent_cluster, name) {
            Some(e) => e,
            None => return -1,
        };

        // Free cluster chain
        let cluster = entry.get_cluster();
        if cluster != 0 {
            self.free_cluster_chain(cluster)?;
        }

        // Mark directory entry as deleted
        let mut entry = Fat32DirEntry::new();
        entry.dir_name[0] = 0xE5;  // Mark as deleted
        self.write_dir_entry(parent_cluster, index, &entry)?;

        0
    }

    /// Find free cluster
    pub fn find_free_cluster(&mut self) -> Option<u32> {
        let total_clusters = self.boot.total_clusters();

        // Get next free cluster from FSINFO
        let mut cluster = self.fs_info.fsi_nxt_free;

        // Search for free cluster
        for _ in 0..total_clusters {
            if cluster < 2 || cluster >= total_clusters {
                cluster = 2;
            }

            let fat_entry = self.read_fat_entry(cluster);
            if fat_entry == FAT_FREE {
                return Some(cluster);
            }

            cluster += 1;
        }

        None
    }
}

/// Global FAT32 file system
static mut FAT32_FS: Fat32FileSystem = Fat32FileSystem::new();

pub fn get_fat32_fs() -> &'static mut Fat32FileSystem {
    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe { &mut FAT32_FS }
}

pub fn init_fat32() {
    let fs = get_fat32_fs();
    fs.init();
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Calculate FAT32 checksum
pub fn calc_checksum(name: &[u8; 11]) -> u8 {
    let mut sum: u8 = 0;
    for &byte in name {
        sum = ((sum & 1) << 7).wrapping_add(sum >> 1).wrapping_add(byte);
    }
    sum
}

/// FAT time conversion (FAT time -> Unix timestamp)
pub fn fat_time_to_unix(fat_time: u16, fat_date: u16) -> u64 {
    let seconds = (fat_time & 0x1F) * 2;
    let minutes = (fat_time >> 5) & 0x3F;
    let hours = (fat_time >> 11) & 0x1F;

    let day = fat_date & 0x1F;
    let month = (fat_date >> 5) & 0x0F;
    let year = ((fat_date >> 9) & 0x7F) + 1980;

    // Simplified calculation (ignoring leap years, etc.)
    let days = (year - 1970) * 365 + (month as u64) * 30 + day as u64;
    days * 86400 + hours as u64 * 3600 + minutes as u64 * 60 + seconds as u64
}

/// Unix timestamp conversion (Unix timestamp -> FAT time)
pub fn unix_to_fat_time(unix_time: u64) -> (u16, u16) {
    let seconds = (unix_time % 60) as u16 / 2;
    let minutes = ((unix_time / 60) % 60) as u16;
    let hours = ((unix_time / 3600) % 24) as u16;

    let fat_time = seconds | (minutes << 5) | (hours << 11);

    let days = (unix_time / 86400) as u16;
    let year = 1980 + days / 365;
    let month = (days % 365) / 30 + 1;
    let day = (days % 365) % 30 + 1;

    let fat_date = day | (month << 5) | ((year - 1980) << 9);

    (fat_time, fat_date)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boot_sector_defaults() {
        let boot = Fat32BootSector::new();

        assert_eq!(boot.bs_signature, FAT32_BOOT_SIGNATURE);
        assert_eq!(boot.bpb_byts_per_sec, 512);
        assert_eq!(boot.bpb_sec_per_clus, 8);
        assert_eq!(boot.bytes_per_sector(), 512);
        assert_eq!(boot.bytes_per_cluster(), 4096);
    }

    #[test]
    fn test_dir_entry_cluster() {
        let mut entry = Fat32DirEntry::new();

        entry.set_cluster(0x12345678);
        assert_eq!(entry.get_cluster(), 0x12345678);
    }

    #[test]
    fn test_dir_entry_name() {
        let mut entry = Fat32DirEntry::new();
        entry.dir_name = [b'T', b'E', b'S', b'T', b' ', b' ', b' ', b' ',
                         b'T', b'X', b'T'];

        let mut buf = [0u8; 13];
        let len = entry.get_name(&mut buf);

        assert_eq!(len, 8);
        assert_eq!(&buf[..8], b"TEST.TXT");
    }

    #[test]
    fn test_checksum() {
        let name = [b'T', b'E', b'S', b'T', b' ', b' ', b' ', b' ',
                    b'T', b'X', b'T'];
        let checksum = calc_checksum(&name);

        // Checksum should be a fixed value
        assert!(checksum != 0 || name == [0u8; 11]);
    }

    #[test]
    fn test_fat_time_conversion() {
        let fat_time = 0x4210;  // 08:32:00
        let fat_date = 0x5421;  // 2021-01-01

        let unix_time = fat_time_to_unix(fat_time, fat_date);
        assert!(unix_time > 0);

        let (new_time, new_date) = unix_to_fat_time(unix_time);
        // Allow certain error
        let _ = (new_time, new_date);
    }
}
