/*
 * Nuva OS - Kernel - DMA Manager
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * DMA buffer management for device drivers.
 */

use crate::{pr_debug, pr_info};
use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// DMA Attributes
bitflags::bitflags! {
    #[repr(transparent)]
    pub struct DmaAttr: u32 {
        /// Coherent DMA (no cache maintenance needed)
        const COHERENT = 1 << 0;
        /// Streaming DMA (requires sync)
        const STREAMING = 1 << 1;
        /// Device read access
        const READ = 1 << 2;
        /// Device write access
        const WRITE = 1 << 3;
        /// Non-coherent (requires explicit sync)
        const NON_COHERENT = 1 << 4;
        /// Contiguous memory
        const CONTIGUOUS = 1 << 5;
        /// Uncached
        const UNCACHED = 1 << 6;
        /// Write combine
        const WRITE_COMBINE = 1 << 7;
    }
}

/// DMA Direction
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DmaDirection {
    /// To device
    ToDevice = 0,
    /// From device
    FromDevice = 1,
    /// Bidirectional
    Bidirectional = 2,
    /// None (for unmap)
    None = 3,
}

/// DMA Buffer
#[repr(C)]
pub struct DmaBuffer {
    /// Virtual address (CPU view)
    pub vaddr: *mut u8,
    /// Physical address
    pub paddr: u64,
    /// DMA address (device view, may differ with IOMMU)
    pub dma_addr: u64,
    /// Size in bytes
    pub size: usize,
    /// Attributes
    pub attr: DmaAttr,
    /// Direction
    pub direction: DmaDirection,
    /// Device ID
    pub device_id: u32,
    /// Reference count
    pub ref_count: AtomicU32,
}

impl DmaBuffer {
    /// Create a new DMA buffer descriptor
    pub fn new(vaddr: *mut u8, paddr: u64, dma_addr: u64, size: usize) -> Self {
        DmaBuffer {
            vaddr,
            paddr,
            dma_addr,
            size,
            attr: DmaAttr::empty(),
            direction: DmaDirection::Bidirectional,
            device_id: 0,
            ref_count: AtomicU32::new(1),
        }
    }

    /// Get virtual address
    pub fn as_ptr(&self) -> *mut u8 {
        self.vaddr
    }

    /// Get as slice
    pub fn as_slice(&self) -> &[u8] {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe { core::slice::from_raw_parts(self.vaddr, self.size) }
    }

    /// Get as mutable slice
    pub fn as_slice_mut(&mut self) -> &mut [u8] {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe { core::slice::from_raw_parts_mut(self.vaddr, self.size) }
    }
}

/// DMA Pool
pub struct DmaPool {
    /// Pool name
    pub name: [u8; 32],
    /// Buffer size
    pub buffer_size: usize,
    /// Alignment
    pub alignment: usize,
    /// Boundary (no crossing)
    pub boundary: usize,
    /// Attributes
    pub attr: DmaAttr,
    /// Number of buffers
    pub count: AtomicU32,
    /// Free count
    pub free_count: AtomicU32,
    /// Allocated count
    pub alloc_count: AtomicU64,
}

impl DmaPool {
    pub const fn new(_name: &[u8], buffer_size: usize, alignment: usize) -> Self {
        DmaPool {
            name: [0; 32],
            buffer_size,
            alignment,
            boundary: 0,
            attr: DmaAttr::COHERENT,
            count: AtomicU32::new(0),
            free_count: AtomicU32::new(0),
            alloc_count: AtomicU64::new(0),
        }
    }
}

/// DMA Statistics
pub struct DmaStats {
    /// Total allocations
    pub alloc_count: AtomicU64,
    /// Total frees
    pub free_count: AtomicU64,
    /// Coherent allocations
    pub coherent_count: AtomicU64,
    /// Streaming mappings
    pub streaming_count: AtomicU64,
    /// Total bytes allocated
    pub total_bytes: AtomicU64,
    /// Current bytes in use
    pub current_bytes: AtomicU64,
    /// Allocation failures
    pub failures: AtomicU64,
}

impl DmaStats {
    pub const fn new() -> Self {
        DmaStats {
            alloc_count: AtomicU64::new(0),
            free_count: AtomicU64::new(0),
            coherent_count: AtomicU64::new(0),
            streaming_count: AtomicU64::new(0),
            total_bytes: AtomicU64::new(0),
            current_bytes: AtomicU64::new(0),
            failures: AtomicU64::new(0),
        }
    }
}

/// DMA Manager
pub struct DmaManager {
    /// Statistics
    pub stats: DmaStats,
    /// Coherent pool
    coherent_pool: DmaPool,
    /// IOMMU present
    has_iommu: bool,
}

impl DmaManager {
    pub const fn new() -> Self {
        DmaManager {
            stats: DmaStats::new(),
            coherent_pool: DmaPool::new(b"coherent", 4096, 4096),
            has_iommu: false,
        }
    }

    /// Initialize DMA manager
    pub fn init(&self) {
        log_info!("DMA manager initialized");
        if self.has_iommu {
            log_info!("  IOMMU: enabled");
        }
    }

    /// Allocate coherent DMA buffer
    /// @param size: Buffer size
    /// @param align: Alignment requirement
    /// @return DMA buffer or error
    pub fn alloc_coherent(&mut self, size: usize, align: usize) -> Result<DmaBuffer, i32> {
        self.stats.alloc_count.fetch_add(1, Ordering::AcqRel);
        self.stats.coherent_count.fetch_add(1, Ordering::AcqRel);

        // TODO: Actual allocation
        // 1. Allocate contiguous physical memory
        // 2. Map to kernel virtual address space
        // 3. Get DMA address (consider IOMMU)

        log_debug!("dma_alloc_coherent: size={}, align={}", size, align);

        // Placeholder - return error for now
        self.stats.failures.fetch_add(1, Ordering::AcqRel);
        Err(-12) // ENOMEM
    }

    /// Free coherent DMA buffer
    pub fn free_coherent(&mut self, buf: &DmaBuffer) {
        self.stats.free_count.fetch_add(1, Ordering::AcqRel);
        self.stats
            .current_bytes
            .fetch_sub(buf.size as u64, Ordering::AcqRel);

        // TODO: Actual free
        log_debug!("dma_free_coherent: size={}", buf.size);
    }

    /// Map single buffer for streaming DMA
    pub fn map_single(
        &mut self,
        vaddr: *mut u8,
        size: usize,
        dir: DmaDirection,
    ) -> Result<u64, i32> {
        self.stats.streaming_count.fetch_add(1, Ordering::AcqRel);

        // TODO: Actual mapping
        // 1. Get physical address
        // 2. Flush/invalidate caches as needed
        // 3. Get DMA address (consider IOMMU)

        log_debug!("dma_map_single: size={}, dir={:?}", size, dir);

        // Placeholder - return physical address
        Ok(vaddr as u64)
    }

    /// Unmap streaming DMA buffer
    pub fn unmap_single(&mut self, dma_addr: u64, size: usize, dir: DmaDirection) {
        // TODO: Actual unmap
        // 1. Invalidate/flush caches as needed
        // 2. Unmap from IOMMU if present

        log_debug!("dma_unmap_single: dma_addr={:#x}, size={}", dma_addr, size);
    }

    /// Sync DMA buffer for CPU access
    pub fn sync_for_cpu(&self, dma_addr: u64, size: usize, dir: DmaDirection) {
        // TODO: Cache maintenance
        // For FROM_DEVICE: invalidate CPU cache
        // For BIDIRECTIONAL: invalidate CPU cache

        log_debug!("dma_sync_for_cpu: dma_addr={:#x}, size={}", dma_addr, size);
    }

    /// Sync DMA buffer for device access
    pub fn sync_for_device(&self, dma_addr: u64, size: usize, dir: DmaDirection) {
        // TODO: Cache maintenance
        // For TO_DEVICE: clean CPU cache
        // For BIDIRECTIONAL: clean CPU cache

        log_debug!(
            "dma_sync_for_device: dma_addr={:#x}, size={}",
            dma_addr,
            size
        );
    }

    /// Map scatter-gather list
    pub fn map_sg(&mut self, sg: &mut [DmaBuffer], dir: DmaDirection) -> Result<usize, i32> {
        let mut mapped = 0;

        for entry in sg.iter_mut() {
            match self.map_single(entry.vaddr, entry.size, dir) {
                Ok(dma_addr) => {
                    entry.dma_addr = dma_addr;
                    entry.direction = dir;
                    mapped += 1;
                }
                Err(_) => break,
            }
        }

        Ok(mapped)
    }

    /// Unmap scatter-gather list
    pub fn unmap_sg(&mut self, sg: &[DmaBuffer], dir: DmaDirection) {
        for entry in sg.iter() {
            self.unmap_single(entry.dma_addr, entry.size, dir);
        }
    }

    /// Get statistics
    pub fn get_stats(&self) -> (u64, u64, u64, u64) {
        (
            self.stats.alloc_count.load(Ordering::Acquire),
            self.stats.free_count.load(Ordering::Acquire),
            self.stats.total_bytes.load(Ordering::Acquire),
            self.stats.failures.load(Ordering::Acquire),
        )
    }
}

/// Global DMA manager
static DMA_MANAGER: core::sync::OnceLock<DmaManager> = core::sync::OnceLock::new();

/// Get DMA manager
pub fn dma_manager() -> &'static DmaManager {
    DMA_MANAGER.get_or_init(DmaManager::new)
}

pub fn init_dma_manager() -> &'static DmaManager {
    DMA_MANAGER.get_or_init(DmaManager::new)
}

/// Initialize DMA manager
pub fn init_dma_manager() {
    let mgr = dma_manager();
    mgr.init();
}

// Convenience functions

/// Allocate coherent DMA buffer
pub fn dma_alloc_coherent(size: usize, align: usize) -> Result<DmaBuffer, i32> {
    dma_manager().alloc_coherent(size, align)
}

/// Free coherent DMA buffer
pub fn dma_free_coherent(buf: &DmaBuffer) {
    dma_manager().free_coherent(buf);
}

/// Map single buffer for streaming DMA
pub fn dma_map_single(vaddr: *mut u8, size: usize, dir: DmaDirection) -> Result<u64, i32> {
    dma_manager().map_single(vaddr, size, dir)
}

/// Unmap streaming DMA buffer
pub fn dma_unmap_single(dma_addr: u64, size: usize, dir: DmaDirection) {
    dma_manager().unmap_single(dma_addr, size, dir);
}

/// Sync for CPU
pub fn dma_sync_for_cpu(dma_addr: u64, size: usize, dir: DmaDirection) {
    dma_manager().sync_for_cpu(dma_addr, size, dir);
}

/// Sync for device
pub fn dma_sync_for_device(dma_addr: u64, size: usize, dir: DmaDirection) {
    dma_manager().sync_for_device(dma_addr, size, dir);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dma_attr() {
        let attr = DmaAttr::COHERENT | DmaAttr::READ | DmaAttr::WRITE;
        assert!(attr.contains(DmaAttr::COHERENT));
        assert!(attr.contains(DmaAttr::READ));
        assert!(attr.contains(DmaAttr::WRITE));
    }

    #[test]
    fn test_dma_direction_values() {
        assert_eq!(DmaDirection::ToDevice as i32, 0);
        assert_eq!(DmaDirection::FromDevice as i32, 1);
        assert_eq!(DmaDirection::Bidirectional as i32, 2);
    }

    #[test]
    fn test_dma_buffer_new() {
        let buf = DmaBuffer::new(core::ptr::null_mut(), 0x1000, 0x1000, 4096);
        assert_eq!(buf.size, 4096);
        assert_eq!(buf.paddr, 0x1000);
        assert_eq!(buf.dma_addr, 0x1000);
    }

    #[test]
    fn test_dma_pool_new() {
        let pool = DmaPool::new(b"test", 4096, 64);
        assert_eq!(pool.buffer_size, 4096);
        assert_eq!(pool.alignment, 64);
    }
}
