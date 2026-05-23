/*
 * NPU Memory Pool Management
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Manages NPU-dedicated memory regions for model weights,
 * intermediate tensors, and input/output buffers.
 * Supports zero-copy CPU-NPU shared memory via DMA mapping.
 */

use core::sync::atomic::{AtomicU64, AtomicU32, AtomicBool, Ordering};
use alloc::vec::Vec;

use crate::kernel::mm::{PhysAddr, VirtAddr, PAGE_SIZE};
use crate::{pr_info, pr_warn};

/// Maximum NPU memory regions
pub const MAX_NPU_MEM_REGIONS: usize = 32;

/// Maximum DMA mappings
pub const MAX_DMA_MAPPINGS: usize = 64;

/// NPU memory error type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NpuMemError {
    /// Out of NPU memory
    OutOfMemory,
    /// Invalid address
    InvalidAddress,
    /// Region not found
    RegionNotFound,
    /// Already mapped
    AlreadyMapped,
    /// DMA mapping failed
    DmaMapFailed,
    /// DMA unmapping failed
    DmaUnmapFailed,
    /// Alignment error
    AlignmentError,
    /// Permission denied
    PermissionDenied,
}

/// NPU memory region descriptor
pub struct NpuMemRegion {
    /// Physical start address
    pub phys_start: PhysAddr,
    /// Virtual start address (NPU address space)
    pub virt_start: VirtAddr,
    /// Region size in bytes
    pub size: u64,
    /// Allocated bytes
    pub allocated: AtomicU64,
    /// Region ID
    pub id: u32,
    /// Is cacheable
    pub cacheable: bool,
}

impl NpuMemRegion {
    /// Create new region descriptor
    pub fn new(id: u32) -> Self {
        NpuMemRegion {
            phys_start: 0,
            virt_start: 0,
            size: 0,
            allocated: AtomicU64::new(0),
            id,
            cacheable: false,
        }
    }

    /// Available bytes
    pub fn available(&self) -> u64 {
        self.size.saturating_sub(self.allocated.load(Ordering::Acquire))
    }

    /// Utilization (0-100%)
    pub fn utilization(&self) -> u32 {
        if self.size == 0 {
            return 0;
        }
        let alloc = self.allocated.load(Ordering::Acquire);
        ((alloc * 100) / self.size) as u32
    }
}

/// DMA mapping entry
pub struct DmaMapping {
    /// CPU virtual address
    pub cpu_vaddr: VirtAddr,
    /// NPU device address
    pub npu_dev_addr: VirtAddr,
    /// Mapping size
    pub size: u64,
    /// Physical address (for DMA)
    pub phys_addr: PhysAddr,
    /// Is active
    pub active: AtomicBool,
}

impl DmaMapping {
    /// Create empty mapping
    pub const fn empty() -> Self {
        DmaMapping {
            cpu_vaddr: 0,
            npu_dev_addr: 0,
            size: 0,
            phys_addr: 0,
            active: AtomicBool::new(false),
        }
    }
}

/// NPU memory pool
/// Manages NPU-dedicated memory with allocation, deallocation,
/// zero-copy shared memory mapping, and DMA import.
pub struct NpuMemPool {
    /// Memory regions
    pub regions: [Option<NpuMemRegion>; MAX_NPU_MEM_REGIONS],
    /// Number of regions
    pub num_regions: u32,
    /// Total NPU memory size
    pub total_size: AtomicU64,
    /// Total allocated
    pub total_allocated: AtomicU64,
    /// DMA mappings
    pub dma_mappings: [DmaMapping; MAX_DMA_MAPPINGS],
    /// Number of DMA mappings
    pub num_dma_mappings: AtomicU32,
    /// Initialized
    pub initialized: AtomicBool,
}

impl NpuMemPool {
    /// Create new NPU memory pool
    pub const fn new() -> Self {
        NpuMemPool {
            regions: [const { None }; MAX_NPU_MEM_REGIONS],
            num_regions: 0,
            total_size: AtomicU64::new(0),
            total_allocated: AtomicU64::new(0),
            dma_mappings: [const { DmaMapping::empty() }; MAX_DMA_MAPPINGS],
            num_dma_mappings: AtomicU32::new(0),
            initialized: AtomicBool::new(false),
        }
    }

    /// Initialize NPU memory pool
    /// @param npu_phys_base: Physical base address of NPU memory
    /// @param npu_mem_size: Size of NPU memory in bytes
    pub fn init(&mut self, npu_phys_base: PhysAddr, npu_mem_size: u64) {
        if self.initialized.load(Ordering::Acquire) {
            return;
        }

        let npu_virt_base: VirtAddr = npu_phys_base + 0xFFFF_0000_0000_0000;

        self.regions[0] = Some(NpuMemRegion {
            phys_start: npu_phys_base,
            virt_start: npu_virt_base,
            size: npu_mem_size,
            allocated: AtomicU64::new(0),
            id: 0,
            cacheable: false,
        });
        self.num_regions = 1;
        self.total_size.store(npu_mem_size, Ordering::Release);
        self.initialized.store(true, Ordering::Release);

        log_info!("NPU memory pool initialized: {} MB at phys 0x{:x}",
                 npu_mem_size / (1024 * 1024), npu_phys_base);
    }

    /// Allocate from NPU memory pool
    /// @param size: Requested allocation size in bytes
    /// @param alignment: Required alignment (must be power of 2)
    /// @return: NPU device virtual address, or error
    pub fn npu_mem_alloc(
        &mut self,
        size: u64,
        alignment: u64,
    ) -> Result<VirtAddr, NpuMemError> {
        if !self.initialized.load(Ordering::Acquire) {
            return Err(NpuMemError::InvalidAddress);
        }

        if size == 0 {
            return Err(NpuMemError::InvalidAddress);
        }

        let aligned_size = align_up(size, alignment.max(PAGE_SIZE));

        for i in 0..self.num_regions as usize {
            if let Some(ref region) = self.regions[i] {
                let available = region.available();
                if available >= aligned_size {
                    let offset = region.allocated.fetch_add(aligned_size, Ordering::AcqRel);
                    let addr = region.virt_start + offset;

                    self.total_allocated.fetch_add(aligned_size, Ordering::Relaxed);
                    return Ok(addr);
                }
            }
        }

        log_warn!("NPU memory: allocation failed ({} bytes requested)", size);
        Err(NpuMemError::OutOfMemory)
    }

    /// Free NPU memory
    /// @param addr: Address returned by npu_mem_alloc
    /// @param size: Size of the allocation
    pub fn npu_mem_free(
        &mut self,
        addr: VirtAddr,
        size: u64,
    ) -> Result<(), NpuMemError> {
        if !self.initialized.load(Ordering::Acquire) {
            return Err(NpuMemError::InvalidAddress);
        }

        let aligned_size = align_up(size, PAGE_SIZE);

        for i in 0..self.num_regions as usize {
            if let Some(ref region) = self.regions[i] {
                if addr >= region.virt_start && addr < region.virt_start + region.size {
                    region.allocated.fetch_sub(aligned_size, Ordering::AcqRel);
                    self.total_allocated.fetch_sub(aligned_size, Ordering::Relaxed);
                    return Ok(());
                }
            }
        }

        Err(NpuMemError::RegionNotFound)
    }

    /// Create CPU-NPU shared memory mapping (zero-copy)
    /// Maps NPU memory into CPU address space so both can
    /// access the same physical memory without copying.
    /// @param npu_addr: NPU device address
    /// @param size: Mapping size
    /// @return: CPU virtual address for the mapping
    pub fn npu_mem_map(
        &mut self,
        npu_addr: VirtAddr,
        size: u64,
    ) -> Result<VirtAddr, NpuMemError> {
        if !self.initialized.load(Ordering::Acquire) {
            return Err(NpuMemError::InvalidAddress);
        }

        let phys_addr = npu_addr_to_phys(npu_addr);

        // SAFETY: FFI call to map device memory into CPU address space
        let cpu_vaddr = unsafe {
            npu_mem_map_ffi(phys_addr, size)
        };

        if cpu_vaddr == 0 {
            return Err(NpuMemError::DmaMapFailed);
        }

        Ok(cpu_vaddr)
    }

    /// Import CPU memory into NPU (DMA mapping)
    /// Maps CPU-allocated memory into NPU address space
    /// for zero-copy access. Used for input/output tensors
    /// allocated on the CPU side.
    /// @param cpu_vaddr: CPU virtual address
    /// @param size: Mapping size
    /// @return: NPU device address for the imported memory
    pub fn npu_mem_import(
        &mut self,
        cpu_vaddr: VirtAddr,
        size: u64,
    ) -> Result<VirtAddr, NpuMemError> {
        if !self.initialized.load(Ordering::Acquire) {
            return Err(NpuMemError::InvalidAddress);
        }

        let num = self.num_dma_mappings.load(Ordering::Acquire);
        if num as usize >= MAX_DMA_MAPPINGS {
            return Err(NpuMemError::DmaMapFailed);
        }

        // SAFETY: FFI calls to get physical address and create DMA mapping
        let phys_addr = unsafe {
            npu_virt_to_phys_ffi(cpu_vaddr)
        };

        let npu_dev_addr = unsafe {
            npu_dma_map_ffi(phys_addr, size)
        };

        if npu_dev_addr == 0 {
            return Err(NpuMemError::DmaMapFailed);
        }

        let idx = num as usize;
        self.dma_mappings[idx] = DmaMapping {
            cpu_vaddr,
            npu_dev_addr,
            size,
            phys_addr,
            active: AtomicBool::new(true),
        };
        self.num_dma_mappings.fetch_add(1, Ordering::Release);

        Ok(npu_dev_addr)
    }

    /// Unimport (unmap) previously imported CPU memory
    pub fn npu_mem_unimport(
        &mut self,
        npu_dev_addr: VirtAddr,
    ) -> Result<(), NpuMemError> {
        let num = self.num_dma_mappings.load(Ordering::Acquire);

        for i in 0..num as usize {
            if self.dma_mappings[i].npu_dev_addr == npu_dev_addr
                && self.dma_mappings[i].active.load(Ordering::Acquire)
            {
                self.dma_mappings[i].active.store(false, Ordering::Release);

                // SAFETY: FFI call to unmap DMA
                unsafe {
                    npu_dma_unmap_ffi(self.dma_mappings[i].phys_addr, self.dma_mappings[i].size);
                }

                return Ok(());
            }
        }

        Err(NpuMemError::RegionNotFound)
    }

    /// Get total NPU memory statistics
    pub fn stats(&self) -> (u64, u64, u32) {
        (
            self.total_size.load(Ordering::Acquire),
            self.total_allocated.load(Ordering::Acquire),
            self.num_dma_mappings.load(Ordering::Acquire),
        )
    }

    /// Get utilization percentage
    pub fn utilization(&self) -> u32 {
        let total = self.total_size.load(Ordering::Acquire);
        if total == 0 {
            return 0;
        }
        let alloc = self.total_allocated.load(Ordering::Acquire);
        ((alloc * 100) / total) as u32
    }
}

/// Align value up to alignment
fn align_up(value: u64, alignment: u64) -> u64 {
    if alignment == 0 {
        return value;
    }
    (value + alignment - 1) & !(alignment - 1)
}

/// Convert NPU virtual address to physical
fn npu_addr_to_phys(npu_vaddr: VirtAddr) -> PhysAddr {
    const NPU_VIRT_OFFSET: u64 = 0xFFFF_0000_0000_0000;
    if npu_vaddr >= NPU_VIRT_OFFSET {
        npu_vaddr - NPU_VIRT_OFFSET
    } else {
        npu_vaddr
    }
}

/// FFI declarations
extern "C" {
    fn npu_mem_map_ffi(phys_addr: PhysAddr, size: u64) -> VirtAddr;
    fn npu_virt_to_phys_ffi(vaddr: VirtAddr) -> PhysAddr;
    fn npu_dma_map_ffi(phys_addr: PhysAddr, size: u64) -> VirtAddr;
    fn npu_dma_unmap_ffi(phys_addr: PhysAddr, size: u64);
}

/// Global NPU memory pool
static NPU_MEM_POOL: core::sync::OnceLock<NpuMemPool> = core::sync::OnceLock::new();

/// Get global NPU memory pool
pub fn npu_mem_pool() -> &'static NpuMemPool {
    NPU_MEM_POOL.get_or_init(NpuMemPool::new)
}

/// Initialize NPU memory pool
pub fn init_npu_mem_pool(phys_base: PhysAddr, size: u64) {
    // SAFETY: init is called once during boot, before concurrent access
    let pool: &mut NpuMemPool = unsafe {
        &mut *core::ptr::from_ref(NPU_MEM_POOL.get_or_init(NpuMemPool::new)).cast_mut()
    };
    pool.init(phys_base, size);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_npu_mem_pool_new() {
        let pool = NpuMemPool::new();
        assert_eq!(pool.num_regions, 0);
    }

    #[test]
    fn test_npu_mem_region() {
        let region = NpuMemRegion::new(0);
        assert_eq!(region.available(), 0);
        assert_eq!(region.utilization(), 0);
    }

    #[test]
    fn test_npu_mem_pool_init() {
        let mut pool = NpuMemPool::new();
        pool.init(0x8000_0000, 1024 * 1024 * 256);
        assert!(pool.initialized.load(Ordering::Relaxed));
        assert_eq!(pool.num_regions, 1);
    }

    #[test]
    fn test_npu_mem_alloc() {
        let mut pool = NpuMemPool::new();
        pool.init(0x8000_0000, 1024 * 1024 * 256);

        let result = pool.npu_mem_alloc(4096, 4096);
        assert!(result.is_ok());
    }

    #[test]
    fn test_npu_mem_alloc_oom() {
        let mut pool = NpuMemPool::new();
        pool.init(0x8000_0000, 4096);

        let result = pool.npu_mem_alloc(8192, 4096);
        assert_eq!(result.err(), Some(NpuMemError::OutOfMemory));
    }

    #[test]
    fn test_npu_mem_free() {
        let mut pool = NpuMemPool::new();
        pool.init(0x8000_0000, 1024 * 1024 * 256);

        let addr = pool.npu_mem_alloc(4096, 4096).unwrap();
        let result = pool.npu_mem_free(addr, 4096);
        assert!(result.is_ok());
    }

    #[test]
    fn test_align_up() {
        assert_eq!(align_up(100, 128), 128);
        assert_eq!(align_up(128, 128), 128);
        assert_eq!(align_up(129, 128), 256);
        assert_eq!(align_up(0, 4096), 0);
    }

    #[test]
    fn test_npu_addr_to_phys() {
        let vaddr: VirtAddr = 0xFFFF_0000_0000_0000 + 0x8000_0000;
        let phys = npu_addr_to_phys(vaddr);
        assert_eq!(phys, 0x8000_0000);
    }

    #[test]
    fn test_dma_mapping_empty() {
        let m = DmaMapping::empty();
        assert_eq!(m.cpu_vaddr, 0);
        assert!(!m.active.load(Ordering::Relaxed));
    }

    #[test]
    fn test_utilization() {
        let mut pool = NpuMemPool::new();
        pool.init(0x8000_0000, 1024 * 1024 * 256);
        assert_eq!(pool.utilization(), 0);
    }
}
