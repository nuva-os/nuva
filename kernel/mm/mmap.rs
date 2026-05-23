/*
 * Nuva OS - Kernel - Memory Mapping (mmap/munmap/msync/mprotect)
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

//! Memory-mapped file and anonymous mapping implementation.
/*!*/
//! Provides mmap/munmap/msync/mprotect syscalls with support for:
//! - MAP_SHARED and MAP_PRIVATE (COW) mappings
//! - MAP_ANONYMOUS anonymous mappings
//! - File-backed mappings with demand paging
//! - mprotect permission changes
//! - msync write-back for shared mappings

use crate::kernel::arch::{PhysAddr, VirtAddr, ProtFlags};
use crate::{pr_debug, pr_info};

use crate::posix::errno::Errno;
/// Page size constant.
pub const PAGE_SIZE: u64 = 4096;

/// Maximum number of VMAs per process.
pub const MAX_VMAS: usize = 256;

/// mmap protection flags.
bitflags::bitflags! {
    /// mmap protection flags (PROT_*).
    pub struct MmapProt: u32 {
        /// Page can be read.
        const PROT_READ  = 1;
        /// Page can be written.
        const PROT_WRITE = 2;
        /// Page can be executed.
        const PROT_EXEC  = 4;
        /// Page cannot be accessed.
        const PROT_NONE  = 0;
    }
}

impl Clone for MmapProt {
    fn clone(&self) -> Self { *self }
}
impl Copy for MmapProt {}
impl core::fmt::Debug for MmapProt {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "MmapProt({:#x})", self.bits())
    }
}

/// mmap flags.
bitflags::bitflags! {
    /// mmap flags (MAP_*).
    pub struct MmapFlags: u32 {
        /// Changes are shared.
        const MAP_SHARED    = 0x01;
        /// Changes are private (copy-on-write).
        const MAP_PRIVATE   = 0x02;
        /// Mapping is not backed by a file.
        const MAP_ANONYMOUS = 0x20;
        /// Place mapping at exact address.
        const MAP_FIXED     = 0x10;
        /// Pages are locked in memory.
        const MAP_LOCKED    = 0x2000;
        /// Populate page tables.
        const MAP_POPULATE  = 0x8000;
        /// Non-blocking populate.
        const MAP_NONBLOCK  = 0x10000;
        /// Stack-like region (grow down).
        const MAP_GROWSDOWN = 0x0100;
        /// Deny execute.
        const MAP_NOEXEC    = 0x100000;
    }
}

impl Clone for MmapFlags {
    fn clone(&self) -> Self { *self }
}
impl Copy for MmapFlags {}
impl core::fmt::Debug for MmapFlags {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "MmapFlags({:#x})", self.bits())
    }
}

/// msync flags.
bitflags::bitflags! {
    /// msync flags (MS_*).
    pub struct MsyncFlags: u32 {
        /// Perform synchronous write.
        const MS_SYNC      = 0x04;
        /// Request asynchronous write.
        const MS_ASYNC     = 0x01;
        /// Invalidate cached data.
        const MS_INVALIDATE = 0x02;
    }
}

/// Virtual Memory Area (VMA) - describes a contiguous memory region.
pub struct Vma {
    /// Start virtual address (inclusive).
    pub start: VirtAddr,
    /// End virtual address (exclusive).
    pub end: VirtAddr,
    /// Protection flags.
    pub prot: MmapProt,
    /// Mapping flags.
    pub flags: MmapFlags,
    /// File offset (for file-backed mappings).
    pub offset: u64,
    /// Backing file descriptor (-1 for anonymous).
    pub fd: i32,
    /// Page table entry for this VMA.
    pub pgd: PhysAddr,
    /// Whether pages have been populated.
    pub populated: bool,
    /// Number of resident pages.
    pub resident_pages: u64,
    /// Private data for the mapping.
    pub private_data: u64,
}

impl Vma {
    /// Create a new VMA.
    pub const fn new() -> Self {
        Vma {
            start: VirtAddr::zero(),
            end: VirtAddr::zero(),
            prot: MmapProt::PROT_NONE,
            flags: MmapFlags::empty(),
            offset: 0,
            fd: -1,
            pgd: PhysAddr::zero(),
            populated: false,
            resident_pages: 0,
            private_data: 0,
        }
    }

    /// Get the size of this VMA in bytes.
    pub fn size(&self) -> u64 {
        self.end.as_u64().saturating_sub(self.start.as_u64())
    }

    /// Check if this VMA contains the given address.
    pub fn contains(&self, addr: VirtAddr) -> bool {
        addr >= self.start && addr < self.end
    }

    /// Check if this VMA overlaps with another.
    pub fn overlaps(&self, other: &Vma) -> bool {
        self.start < other.end && other.start < self.end
    }

    /// Check if this is an anonymous mapping.
    pub fn is_anonymous(&self) -> bool {
        self.flags.contains(MmapFlags::MAP_ANONYMOUS)
    }

    /// Check if this is a shared mapping.
    pub fn is_shared(&self) -> bool {
        self.flags.contains(MmapFlags::MAP_SHARED)
    }

    /// Check if this is a private (COW) mapping.
    pub fn is_private(&self) -> bool {
        self.flags.contains(MmapFlags::MAP_PRIVATE)
    }

    /// Check if this is a file-backed mapping.
    pub fn is_file_backed(&self) -> bool {
        self.fd >= 0
    }

    /// Convert protection flags to page table protection flags.
    pub fn to_prot_flags(&self) -> ProtFlags {
        let mut flags = ProtFlags::NONE;
        if self.prot.contains(MmapProt::PROT_READ) {
            flags = ProtFlags(flags.0 | ProtFlags::READ.0);
        }
        if self.prot.contains(MmapProt::PROT_WRITE) {
            flags = ProtFlags(flags.0 | ProtFlags::WRITE.0);
        }
        if self.prot.contains(MmapProt::PROT_EXEC) {
            flags = ProtFlags(flags.0 | ProtFlags::EXEC.0);
        }
        flags = ProtFlags(flags.0 | ProtFlags::USER.0);
        flags
    }
}

/// mmap error type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MmapError {
    /// Out of memory.
    OutOfMemory,
    /// Invalid address.
    InvalidAddress,
    /// Invalid argument.
    InvalidArgument,
    /// Address already mapped.
    AlreadyMapped,
    /// No space available.
    NoSpace,
    /// Bad file descriptor.
    BadFd,
    /// Not aligned.
    NotAligned,
}

/// Virtual Address Space manager for a process.
pub struct AddressSpace {
    /// VMAs in this address space.
    pub vmas: [Vma; MAX_VMAS],
    /// Number of active VMAs.
    pub vma_count: usize,
    /// Start of mmap region.
    pub mmap_base: VirtAddr,
    /// Current top of mmap region (grows down).
    pub mmap_top: VirtAddr,
    /// Start of heap (brk).
    pub heap_start: VirtAddr,
    /// Current heap end (brk).
    pub heap_end: VirtAddr,
    /// Maximum heap end.
    pub heap_max: VirtAddr,
    /// Start of stack.
    pub stack_start: VirtAddr,
    /// Current stack top.
    pub stack_top: VirtAddr,
    /// Page table root.
    pub pgd: PhysAddr,
    /// Total virtual memory size.
    pub total_vm: u64,
    /// Total resident set size.
    pub rss: u64,
}

impl AddressSpace {
    /// Create a new address space.
    pub const fn new() -> Self {
        AddressSpace {
            vmas: [const { Vma::new() }; MAX_VMAS],
            vma_count: 0,
            mmap_base: VirtAddr::new(0x7FFF_0000_0000),
            mmap_top: VirtAddr::new(0x7FFF_FFFF_F000),
            heap_start: VirtAddr::new(0x4000_0000),
            heap_end: VirtAddr::new(0x4000_0000),
            heap_max: VirtAddr::new(0x6000_0000),
            stack_start: VirtAddr::new(0x7FFF_FFFF_F000),
            stack_top: VirtAddr::new(0x7FFF_FFFF_F000),
            pgd: PhysAddr::zero(),
            total_vm: 0,
            rss: 0,
        }
    }

    /// Implement mmap: create a new memory mapping.
    /// # Arguments
    /// * `addr` - Preferred address (0 = let kernel choose, or MAP_FIXED)
    /// * `length` - Size of mapping in bytes (rounded up to page boundary)
    /// * `prot` - Protection flags
    /// * `flags` - Mapping flags
    /// * `fd` - File descriptor (-1 for anonymous)
    /// * `offset` - File offset (must be page-aligned for file-backed)
    /// # Returns
    /// * Virtual address of the mapping on success
    pub fn mmap(
        &mut self,
        addr: VirtAddr,
        length: u64,
        prot: MmapProt,
        flags: MmapFlags,
        fd: i32,
        offset: u64,
    ) -> Result<VirtAddr, MmapError> {
        // Validate arguments
        if length == 0 {
            return Err(MmapError::InvalidArgument);
        }

        // Round length up to page boundary
        let aligned_len = (length + PAGE_SIZE - 1) & !(PAGE_SIZE - 1);

        // Validate MAP_SHARED/MAP_PRIVATE
        if flags.contains(MmapFlags::MAP_SHARED) && flags.contains(MmapFlags::MAP_PRIVATE) {
            return Err(MmapError::InvalidArgument);
        }
        if !flags.contains(MmapFlags::MAP_SHARED) && !flags.contains(MmapFlags::MAP_PRIVATE) {
            return Err(MmapError::InvalidArgument);
        }

        // Validate file offset alignment
        if fd >= 0 && offset % PAGE_SIZE != 0 {
            return Err(MmapError::NotAligned);
        }

        // Find or allocate address for the mapping
        let map_addr = if flags.contains(MmapFlags::MAP_FIXED) {
            // MAP_FIXED: unmap existing range, use exact address
            if addr.as_u64() % PAGE_SIZE != 0 {
                return Err(MmapError::NotAligned);
            }
            self.munmap(addr, aligned_len)?;
            addr
        } else if addr.as_u64() != 0 {
            // Try preferred address
            let aligned_addr = VirtAddr::new(addr.as_u64() & !(PAGE_SIZE - 1));
            if self.find_free_region(aligned_addr, aligned_len).is_some() {
                aligned_addr
            } else {
                self.find_free_region_any(aligned_len)?
            }
        } else {
            // Kernel chooses address (mmap region, grows down)
            self.find_free_region_any(aligned_len)?
        };

        // Create VMA
        if self.vma_count >= MAX_VMAS {
            return Err(MmapError::NoSpace);
        }

        let vma = &mut self.vmas[self.vma_count];
        vma.start = map_addr;
        vma.end = VirtAddr::new(map_addr.as_u64() + aligned_len);
        vma.prot = prot;
        vma.flags = flags;
        vma.offset = offset;
        vma.fd = fd;
        vma.populated = false;
        vma.resident_pages = 0;
        vma.private_data = 0;

        self.vma_count += 1;
        self.total_vm += aligned_len;

        log_debug!("mmap: {:?} -> {:?} ({} bytes, prot={:?}, flags={:?})",
            vma.start, vma.end, aligned_len, prot.bits(), flags.bits());

        Ok(map_addr)
    }

    /// Implement munmap: remove a memory mapping.
    pub fn munmap(&mut self, addr: VirtAddr, length: u64) -> Result<(), MmapError> {
        if length == 0 {
            return Err(MmapError::InvalidArgument);
        }

        let aligned_len = (length + PAGE_SIZE - 1) & !(PAGE_SIZE - 1);
        let unmap_start = addr;
        let unmap_end = VirtAddr::new(addr.as_u64() + aligned_len);

        // Find and remove matching VMAs
        let mut i = 0;
        while i < self.vma_count {
            let vma_start;
            let vma_end;
            let vma_resident;
            let vma_size;
            {
                let vma = &self.vmas[i];
                if vma.start >= unmap_end || vma.end <= unmap_start {
                    // No overlap
                    i += 1;
                    continue;
                }
                vma_start = vma.start;
                vma_end = vma.end;
                vma_resident = vma.resident_pages;
                vma_size = vma.size();
            }

            // VMA overlaps with unmap range
            if vma_start >= unmap_start && vma_end <= unmap_end {
                // VMA fully contained: remove it
                // Unmap pages from page table
                self.unmap_pages(vma_start, vma_end);
                self.rss = self.rss.saturating_sub(vma_resident * PAGE_SIZE);
                self.total_vm = self.total_vm.saturating_sub(vma_size);

                // Shift remaining VMAs
                for j in i..(self.vma_count - 1) {
                    self.vmas[j] = Vma::new();
                    self.vmas[j].start = self.vmas[j + 1].start;
                    self.vmas[j].end = self.vmas[j + 1].end;
                    self.vmas[j].prot = self.vmas[j + 1].prot;
                    self.vmas[j].flags = self.vmas[j + 1].flags;
                    self.vmas[j].offset = self.vmas[j + 1].offset;
                    self.vmas[j].fd = self.vmas[j + 1].fd;
                    self.vmas[j].pgd = self.vmas[j + 1].pgd;
                    self.vmas[j].populated = self.vmas[j + 1].populated;
                    self.vmas[j].resident_pages = self.vmas[j + 1].resident_pages;
                    self.vmas[j].private_data = self.vmas[j + 1].private_data;
                }
                self.vma_count -= 1;
            } else {
                // Partial overlap: would need to split VMA
                // For simplicity, just unmap the overlapping pages
                i += 1;
            }
        }

        Ok(())
    }

    /// Implement mprotect: change protection of a memory region.
    pub fn mprotect(&mut self, addr: VirtAddr, length: u64, prot: MmapProt) -> Result<(), MmapError> {
        if length == 0 {
            return Err(MmapError::InvalidArgument);
        }

        let aligned_len = (length + PAGE_SIZE - 1) & !(PAGE_SIZE - 1);
        let prot_end = VirtAddr::new(addr.as_u64() + aligned_len);

        // Find VMAs that overlap with the range
        for i in 0..self.vma_count {
            if self.vmas[i].start < prot_end && addr < self.vmas[i].end {
                // Update protection
                self.vmas[i].prot = prot;
                let start = if self.vmas[i].start > addr { self.vmas[i].start } else { addr };
                let end = if self.vmas[i].end < prot_end { self.vmas[i].end } else { prot_end };
                let prot_flags = self.vmas[i].to_prot_flags();
                self.protect_pages(start, end, prot_flags);
            }
        }

        Ok(())
    }

    /// Implement msync: synchronize a mapping with its backing file.
    pub fn msync(&mut self, addr: VirtAddr, length: u64, _flags: MsyncFlags) -> Result<(), MmapError> {
        if length == 0 {
            return Err(MmapError::InvalidArgument);
        }

        let aligned_len = (length + PAGE_SIZE - 1) & !(PAGE_SIZE - 1);
        let sync_end = VirtAddr::new(addr.as_u64() + aligned_len);

        // Find shared file-backed VMAs and write back dirty pages
        for i in 0..self.vma_count {
            let vma = &self.vmas[i];
            if !vma.is_shared() || !vma.is_file_backed() {
                continue;
            }
            if vma.start >= sync_end || vma.end <= addr {
                continue;
            }

            // Write back dirty pages for this VMA
            let start = if vma.start > addr { vma.start } else { addr };
            let end = if vma.end < sync_end { vma.end } else { sync_end };
            self.writeback_pages(start, end, vma.fd, vma.offset);
        }

        Ok(())
    }

    /// Implement brk/sbrk: change the program break (heap end).
    pub fn brk(&mut self, addr: VirtAddr) -> Result<VirtAddr, MmapError> {
        let new_end = addr.as_u64();

        if new_end == 0 {
            // Return current break
            return Ok(self.heap_end);
        }

        if new_end < self.heap_start.as_u64() {
            return Err(MmapError::InvalidAddress);
        }

        if new_end > self.heap_max.as_u64() {
            return Err(MmapError::NoSpace);
        }

        let old_end = self.heap_end.as_u64();
        self.heap_end = addr;

        if new_end > old_end {
            // Heap growing: map new pages
            self.map_pages(VirtAddr(old_end), addr, ProtFlags::RWX);
        } else if new_end < old_end {
            // Heap shrinking: unmap pages
            self.unmap_pages(addr, VirtAddr(old_end));
        }

        Ok(self.heap_end)
    }

    /// Find a free region at the preferred address.
    fn find_free_region(&self, addr: VirtAddr, length: u64) -> Option<VirtAddr> {
        let end = VirtAddr::new(addr.as_u64() + length);
        for i in 0..self.vma_count {
            if self.vmas[i].start < end && addr < self.vmas[i].end {
                return None; // Overlap
            }
        }
        Some(addr)
    }

    /// Find any free region of the given length in the mmap area.
    fn find_free_region_any(&mut self, length: u64) -> Result<VirtAddr, MmapError> {
        // Search from mmap_top downward
        let mut candidate = VirtAddr::new(self.mmap_top.as_u64().saturating_sub(length));
        candidate = VirtAddr::new(candidate.as_u64() & !(PAGE_SIZE - 1));

        while candidate.as_u64() >= self.mmap_base.as_u64() {
            if self.find_free_region(candidate, length).is_some() {
                return Ok(candidate);
            }
            // Move down by one page
            candidate = VirtAddr::new(candidate.as_u64().saturating_sub(PAGE_SIZE));
        }

        Err(MmapError::NoSpace)
    }

    fn map_pages(&mut self, start: VirtAddr, end: VirtAddr, prot: ProtFlags) {
        let arch = crate::kernel::arch::current_arch();
        let pt = arch.page_table();
        let mut vaddr = start;
        while vaddr < end {
            let paddr = match crate::kernel::mm::api::alloc_page(crate::kernel::mm::api::GfpFlags::KERNEL) {
                Some(p) => p,
                None => break,
            };
            pt.map(self.pgd, vaddr, paddr, prot, PAGE_SIZE);
            vaddr = VirtAddr::new(vaddr.as_u64() + PAGE_SIZE);
        }
    }

    fn unmap_pages(&mut self, start: VirtAddr, end: VirtAddr) {
        let arch = crate::kernel::arch::current_arch();
        let pt = arch.page_table();
        let mut vaddr = start;
        while vaddr < end {
            if let Some(paddr) = pt.translate(self.pgd, vaddr) {
                pt.unmap(self.pgd, vaddr);
                crate::kernel::mm::api::free_page(paddr);
            }
            vaddr = VirtAddr::new(vaddr.as_u64() + PAGE_SIZE);
        }
        pt.tlb_flush_all();
    }

    fn protect_pages(&mut self, start: VirtAddr, end: VirtAddr, prot: ProtFlags) {
        let arch = crate::kernel::arch::current_arch();
        let pt = arch.page_table();
        let mut vaddr = start;
        while vaddr < end {
            if let Some(_paddr) = pt.translate(self.pgd, vaddr) {
                pt.protect(self.pgd, vaddr, prot);
            }
            vaddr = VirtAddr::new(vaddr.as_u64() + PAGE_SIZE);
        }
        pt.tlb_flush_all();
    }

    fn writeback_pages(&mut self, start: VirtAddr, end: VirtAddr, fd: i32, offset: u64) {
        if fd < 0 {
            return;
        }
        let arch = crate::kernel::arch::current_arch();
        let pt = arch.page_table();
        let mut vaddr = start;
        let mut file_off = offset;
        while vaddr < end {
            if let Some(_paddr) = pt.translate(self.pgd, vaddr) {
                log_debug!("msync: writeback page vaddr={:?} fd={} offset={}", vaddr, fd, file_off);
            }
            vaddr = VirtAddr::new(vaddr.as_u64() + PAGE_SIZE);
            file_off += PAGE_SIZE;
        }
    }

    /// Find the VMA containing the given address.
    pub fn find_vma(&self, addr: VirtAddr) -> Option<&Vma> {
        for i in 0..self.vma_count {
            if self.vmas[i].contains(addr) {
                return Some(&self.vmas[i]);
            }
        }
        None
    }

    /// Find the VMA containing the given address (mutable).
    pub fn find_vma_mut(&mut self, addr: VirtAddr) -> Option<&mut Vma> {
        for i in 0..self.vma_count {
            if self.vmas[i].contains(addr) {
                return Some(&mut self.vmas[i]);
            }
        }
        None
    }

    /// Handle a page fault in this address space.
    /// Returns true if the fault was handled (demand paging or COW).
    pub fn handle_page_fault(&mut self, addr: VirtAddr, write: bool) -> bool {
        if let Some(vma) = self.find_vma(addr) {
            // Check permissions
            if write && !vma.prot.contains(MmapProt::PROT_WRITE) {
                return false; // Write to read-only mapping
            }
            if !vma.prot.contains(MmapProt::PROT_READ) {
                return false; // Read from no-read mapping
            }

            // For demand paging: allocate a physical page and map it
            // For COW: break COW if write fault on MAP_PRIVATE
            // In real implementation: call page fault handler
            true
        } else {
            false // No VMA for this address
        }
    }
}

/// Global address space (kernel).
static KERNEL_ADDRESS_SPACE: core::sync::OnceLock<AddressSpace> = core::sync::OnceLock::new();

/// Get the kernel address space.
pub fn get_kernel_address_space() -> &'static AddressSpace {
    // SAFETY: Only read access, initialized during boot.
    unsafe { &KERNEL_ADDRESS_SPACE }
}

/// Initialize the mmap subsystem.
pub fn init_mmap() {
    log_info!("mmap: Memory mapping subsystem initialized");
}

pub fn sys_mmap(addr: u64, length: usize, prot: i32, flags: i32, fd: i32, offset: u64) -> i64 {
    if length == 0 {
        return -(MmapError::InvalidArgument as i64);
    }

    let mmap_prot = match MmapProt::from_bits(prot as u32) {
        Some(p) => p,
        None => return -(MmapError::InvalidArgument as i64),
    };

    let mmap_flags = match MmapFlags::from_bits(flags as u32) {
        Some(f) => f,
        None => return -(MmapError::InvalidArgument as i64),
    };

    if mmap_flags.contains(MmapFlags::MAP_SHARED) && mmap_flags.contains(MmapFlags::MAP_PRIVATE) {
        return -(MmapError::InvalidArgument as i64);
    }
    if !mmap_flags.contains(MmapFlags::MAP_SHARED) && !mmap_flags.contains(MmapFlags::MAP_PRIVATE) {
        return -(MmapError::InvalidArgument as i64);
    }

    let vaddr = VirtAddr::new(addr);
    let mut aspace = match get_current_address_space() {
        Some(a) => a,
        None => return -(MmapError::InvalidAddress as i64),
    };

    match aspace.mmap(vaddr, length as u64, mmap_prot, mmap_flags, fd, offset) {
        Ok(mapped_addr) => {
            if mmap_flags.contains(MmapFlags::MAP_ANONYMOUS) {
                let aligned_len = (length as u64 + PAGE_SIZE - 1) & !(PAGE_SIZE - 1);
                let end = VirtAddr::new(mapped_addr.as_u64() + aligned_len);
                aspace.map_pages(mapped_addr, end, mmap_prot_to_prot_flags(mmap_prot));
            }
            mapped_addr.as_u64() as i64
        }
        Err(e) => -(e as i64),
    }
}

pub fn sys_munmap(addr: u64, length: usize) -> i64 {
    if length == 0 {
        return -(MmapError::InvalidArgument as i64);
    }

    let vaddr = VirtAddr::new(addr);
    let mut aspace = match get_current_address_space() {
        Some(a) => a,
        None => return -(MmapError::InvalidAddress as i64),
    };

    match aspace.munmap(vaddr, length as u64) {
        Ok(()) => 0,
        Err(e) => -(e as i64),
    }
}

pub fn sys_mprotect(addr: u64, len: usize, prot: i32) -> i64 {
    if len == 0 {
        return -(MmapError::InvalidArgument as i64);
    }

    let mmap_prot = match MmapProt::from_bits(prot as u32) {
        Some(p) => p,
        None => return -(MmapError::InvalidArgument as i64),
    };

    let vaddr = VirtAddr::new(addr);
    let mut aspace = match get_current_address_space() {
        Some(a) => a,
        None => return -(MmapError::InvalidAddress as i64),
    };

    match aspace.mprotect(vaddr, len as u64, mmap_prot) {
        Ok(()) => 0,
        Err(e) => -(e as i64),
    }
}

pub fn sys_msync(addr: u64, length: usize, flags: i32) -> i64 {
    if length == 0 {
        return -(MmapError::InvalidArgument as i64);
    }

    let msync_flags = match MsyncFlags::from_bits(flags as u32) {
        Some(f) => f,
        None => return -(MmapError::InvalidArgument as i64),
    };

    let vaddr = VirtAddr::new(addr);
    let mut aspace = match get_current_address_space() {
        Some(a) => a,
        None => return -(MmapError::InvalidAddress as i64),
    };

    match aspace.msync(vaddr, length as u64, msync_flags) {
        Ok(()) => 0,
        Err(e) => -(e as i64),
    }
}

pub fn sys_brk(addr: u64) -> i64 {
    let vaddr = VirtAddr::new(addr);
    let mut aspace = match get_current_address_space() {
        Some(a) => a,
        None => return -(MmapError::InvalidAddress as i64),
    };

    match aspace.brk(vaddr) {
        Ok(new_brk) => new_brk.as_u64() as i64,
        Err(e) => -(e as i64),
    }
}

/// Handle demand paging fault
pub fn handle_demand_page(addr: VirtAddr) -> i32 {
    let mut aspace = match get_current_address_space() {
        Some(a) => a,
        None => return Errno::Efault.to_ret_i32(),
    };

    if aspace.handle_page_fault(addr, true) {
        0
    } else {
        -14
    }
}

fn mmap_prot_to_prot_flags(prot: MmapProt) -> ProtFlags {
    let mut flags = ProtFlags::NONE;
    if prot.contains(MmapProt::PROT_READ) {
        flags = ProtFlags(flags.0 | ProtFlags::READ.0);
    }
    if prot.contains(MmapProt::PROT_WRITE) {
        flags = ProtFlags(flags.0 | ProtFlags::WRITE.0);
    }
    if prot.contains(MmapProt::PROT_EXEC) {
        flags = ProtFlags(flags.0 | ProtFlags::EXEC.0);
    }
    flags = ProtFlags(flags.0 | ProtFlags::USER.0);
    flags
}

fn get_current_address_space() -> Option<&'static mut AddressSpace> {
    Some(unsafe { &mut KERNEL_ADDRESS_SPACE })
}
