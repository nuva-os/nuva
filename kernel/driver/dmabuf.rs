/*
 * Nuva OS - Kernel - Driver - Dmabuf
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
 * Nuva OS - Kernel - DMA-BUF Framework
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * DMA buffer sharing framework for zero-copy buffer sharing
 * between devices.
 */

use crate::{pr_debug, pr_info};
use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// DMA-BUF Handle
pub type DmaBufHandle = u32;

/// DMA-BUF Sync Mode
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DmaBufSync {
    /// Start CPU access (read)
    StartRead = 0,
    /// End CPU access (read)
    EndRead = 1,
    /// Start CPU access (write)
    StartWrite = 2,
    /// End CPU access (write)
    EndWrite = 3,
}

/// DMA-BUF Flags
bitflags::bitflags! {
    #[repr(transparent)]
    pub struct DmaBufFlags: u32 {
        /// CPU access allowed
        const CPU_ACCESS = 1 << 0;
        /// DMA access allowed
        const DMA_ACCESS = 1 << 1;
        /// Kernel mapping exists
        const KERNEL_MAPPED = 1 << 2;
        /// User mapping exists
        const USER_MAPPED = 1 << 3;
        /// Contiguous memory
        const CONTIGUOUS = 1 << 4;
        /// Cached
        const CACHED = 1 << 5;
        /// Write combine
        const WRITE_COMBINE = 1 << 6;
        /// Uncached
        const UNCACHED = 1 << 7;
        /// Protected
        const PROTECTED = 1 << 8;
    }
}

/// DMA-BUF Attachment
#[repr(C)]
pub struct DmaBufAttachment {
    /// Attachment ID
    pub id: u32,
    /// Device ID
    pub dev_id: u32,
    /// DMA address
    pub dma_addr: u64,
    /// Size
    pub size: usize,
    /// Direction
    pub direction: DmaBufDirection,
    /// Private data
    pub priv_data: *mut core::ffi::c_void,
}

/// DMA-BUF Direction
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DmaBufDirection {
    /// To device
    ToDevice = 0,
    /// From device
    FromDevice = 1,
    /// Bidirectional
    Bidirectional = 2,
    /// None
    None = 3,
}

/// DMA-BUF Plane
#[repr(C)]
pub struct DmaBufPlane {
    /// DMA address
    pub dma_addr: u64,
    /// Size
    pub size: u32,
    /// Offset
    pub offset: u32,
    /// Bytes used
    pub bytes_used: u32,
    /// FD for export
    pub fd: i32,
}

/// DMA-BUF Info
#[repr(C)]
pub struct DmaBufInfo {
    /// Handle
    pub handle: DmaBufHandle,
    /// Size
    pub size: usize,
    /// Flags
    pub flags: DmaBufFlags,
    /// Number of attachments
    pub num_attachments: u32,
    /// Number of planes
    pub num_planes: u32,
    /// Exporter name
    pub exporter: [u8; 32],
    /// Creation timestamp
    pub timestamp: u64,
    /// Reference count
    pub ref_count: u32,
}

/// DMA-BUF Exporter Operations
pub struct DmaBufExporterOps {
    /// Map attachment
    pub attach: Option<unsafe extern "C" fn(*mut core::ffi::c_void, *mut DmaBufAttachment) -> i32>,
    /// Detach
    pub detach: Option<unsafe extern "C" fn(*mut core::ffi::c_void, *mut DmaBufAttachment)>,
    /// Map for DMA
    pub map_dma: Option<unsafe extern "C" fn(*mut core::ffi::c_void, *mut DmaBufAttachment) -> i32>,
    /// Unmap from DMA
    pub unmap_dma: Option<unsafe extern "C" fn(*mut core::ffi::c_void, *mut DmaBufAttachment)>,
    /// Release
    pub release: Option<unsafe extern "C" fn(*mut core::ffi::c_void)>,
    /// Begin CPU access
    pub begin_cpu_access:
        Option<unsafe extern "C" fn(*mut core::ffi::c_void, DmaBufDirection) -> i32>,
    /// End CPU access
    pub end_cpu_access:
        Option<unsafe extern "C" fn(*mut core::ffi::c_void, DmaBufDirection) -> i32>,
    /// Map for CPU (kernel)
    pub vmap: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> *mut core::ffi::c_void>,
    /// Unmap from CPU
    pub vunmap: Option<unsafe extern "C" fn(*mut core::ffi::c_void)>,
    /// Get scatter-gather table
    pub get_sg_table:
        Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> *mut core::ffi::c_void>,
    /// Mmap
    pub mmap: Option<
        unsafe extern "C" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, u64, usize) -> i32,
    >,
}

/// DMA-BUF Exporter Info
pub struct DmaBufExporter {
    /// Exporter name
    pub name: [u8; 32],
    /// Exporter ID
    pub id: u32,
    /// Operations
    pub ops: DmaBufExporterOps,
    /// Private data
    pub data: *mut core::ffi::c_void,
    /// Owner device
    pub owner: u32,
}

/// DMA-BUF Manager
pub struct DmaBufManager {
    /// Buffer count
    buf_count: AtomicU32,
    /// Exporter count
    exporter_count: AtomicU32,
    /// Statistics
    stats: DmaBufStats,
}

/// DMA-BUF Statistics
pub struct DmaBufStats {
    /// Total allocations
    pub alloc_count: AtomicU64,
    /// Total frees
    pub free_count: AtomicU64,
    /// Total bytes allocated
    pub total_bytes: AtomicU64,
    /// Current bytes in use
    pub current_bytes: AtomicU64,
    /// Import count
    pub import_count: AtomicU64,
    /// Export count
    pub export_count: AtomicU64,
}

impl DmaBufStats {
    pub const fn new() -> Self {
        DmaBufStats {
            alloc_count: AtomicU64::new(0),
            free_count: AtomicU64::new(0),
            total_bytes: AtomicU64::new(0),
            current_bytes: AtomicU64::new(0),
            import_count: AtomicU64::new(0),
            export_count: AtomicU64::new(0),
        }
    }
}

impl DmaBufManager {
    pub const fn new() -> Self {
        DmaBufManager {
            buf_count: AtomicU32::new(0),
            exporter_count: AtomicU32::new(0),
            stats: DmaBufStats::new(),
        }
    }

    /// Initialize
    pub fn init(&self) {
        log_info!("DMA-BUF manager initialized");
    }

    /// Register exporter
    pub fn register_exporter(&mut self, _exporter: &DmaBufExporter) -> u32 {
        let id = self.exporter_count.fetch_add(1, Ordering::AcqRel);
        id
    }

    /// Allocate DMA-BUF
    pub fn alloc(&mut self, size: usize, flags: DmaBufFlags) -> DmaBufHandle {
        self.stats.alloc_count.fetch_add(1, Ordering::AcqRel);
        self.stats
            .total_bytes
            .fetch_add(size as u64, Ordering::AcqRel);
        self.stats
            .current_bytes
            .fetch_add(size as u64, Ordering::AcqRel);

        let handle = self.buf_count.fetch_add(1, Ordering::AcqRel);

        log_debug!(
            "dmabuf_alloc: size={}, flags={:#x}, handle={}",
            size,
            flags.bits(),
            handle
        );
        handle
    }

    /// Free DMA-BUF
    pub fn free(&mut self, handle: DmaBufHandle, size: usize) {
        self.stats.free_count.fetch_add(1, Ordering::AcqRel);
        self.stats
            .current_bytes
            .fetch_sub(size as u64, Ordering::AcqRel);

        log_debug!("dmabuf_free: handle={}", handle);
    }

    /// Export DMA-BUF (get FD)
    pub fn export(&mut self, handle: DmaBufHandle) -> i32 {
        self.stats.export_count.fetch_add(1, Ordering::AcqRel);
        log_debug!("dmabuf_export: handle={}", handle);
        handle as i32
    }

    /// Import DMA-BUF (from FD)
    pub fn import(&mut self, fd: i32) -> DmaBufHandle {
        self.stats.import_count.fetch_add(1, Ordering::AcqRel);
        log_debug!("dmabuf_import: fd={}", fd);
        fd as DmaBufHandle
    }

    /// Sync DMA-BUF
    pub fn sync(&self, handle: DmaBufHandle, sync: DmaBufSync) -> i32 {
        log_debug!("dmabuf_sync: handle={}, sync={:?}", handle, sync);
        0
    }

    /// Map DMA-BUF for device
    pub fn map(&mut self, handle: DmaBufHandle, dev_id: u32, direction: DmaBufDirection) -> u64 {
        log_debug!(
            "dmabuf_map: handle={}, dev={}, dir={:?}",
            handle,
            dev_id,
            direction
        );
        0
    }

    /// Unmap DMA-BUF
    pub fn unmap(&mut self, handle: DmaBufHandle, dev_id: u32) {
        log_debug!("dmabuf_unmap: handle={}, dev={}", handle, dev_id);
    }

    /// Get info
    pub fn get_info(&self, handle: DmaBufHandle) -> DmaBufInfo {
        DmaBufInfo {
            handle,
            size: 0,
            flags: DmaBufFlags::empty(),
            num_attachments: 0,
            num_planes: 1,
            exporter: [0; 32],
            timestamp: 0,
            ref_count: 1,
        }
    }

    /// Get statistics
    pub fn get_stats(&self) -> (u64, u64, u64, u64) {
        (
            self.stats.alloc_count.load(Ordering::Acquire),
            self.stats.free_count.load(Ordering::Acquire),
            self.stats.total_bytes.load(Ordering::Acquire),
            self.stats.current_bytes.load(Ordering::Acquire),
        )
    }
}

/// Global DMA-BUF manager
static DMABUF_MANAGER: crate::sync_oncelock::OnceLock<DmaBufManager> = crate::sync_oncelock::OnceLock::new();

/// Get DMA-BUF manager
pub fn dmabuf_manager() -> &'static DmaBufManager {
    DMABUF_MANAGER.get_or_init(DmaBufManager::new)
}

/// Initialize DMA-BUF manager
pub fn init_dmabuf_manager() {
    let mgr = dmabuf_manager();
    mgr.init();
}

// Convenience functions

/// Allocate DMA-BUF
pub fn dmabuf_alloc(size: usize, flags: DmaBufFlags) -> DmaBufHandle {
    dmabuf_manager().alloc(size, flags)
}

/// Free DMA-BUF
pub fn dmabuf_free(handle: DmaBufHandle, size: usize) {
    dmabuf_manager().free(handle, size);
}

/// Export DMA-BUF
pub fn dmabuf_export(handle: DmaBufHandle) -> i32 {
    dmabuf_manager().export(handle)
}

/// Import DMA-BUF
pub fn dmabuf_import(fd: i32) -> DmaBufHandle {
    dmabuf_manager().import(fd)
}

/// Sync DMA-BUF
pub fn dmabuf_sync(handle: DmaBufHandle, sync: DmaBufSync) -> i32 {
    dmabuf_manager().sync(handle, sync)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dmabuf_sync() {
        assert_eq!(DmaBufSync::StartRead as i32, 0);
        assert_eq!(DmaBufSync::EndWrite as i32, 3);
    }

    #[test]
    fn test_dmabuf_direction() {
        assert_eq!(DmaBufDirection::ToDevice as i32, 0);
        assert_eq!(DmaBufDirection::Bidirectional as i32, 2);
    }

    #[test]
    fn test_dmabuf_flags() {
        let flags = DmaBufFlags::CPU_ACCESS | DmaBufFlags::DMA_ACCESS;
        assert!(flags.contains(DmaBufFlags::CPU_ACCESS));
        assert!(flags.contains(DmaBufFlags::DMA_ACCESS));
    }
}
