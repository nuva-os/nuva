/*
 * Nuva OS - Kernel - Include
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


/// KernelimaginarysimulatedAddressbaseaddress
pub const KERNEL_VA_BASE: u64 = 0xFFFFFF80_00000000;

/// KernelPhysicsAddressbaseaddress
pub const KERNEL_PA_BASE: u64 = 0x00080000;

/// KernelSizeLimit (128MB)
pub const KERNEL_SIZE: u64 = 128 * 1024 * 1024;

/// KernelimaginarysimulatedAddressEnd
pub const KERNEL_VA_END: u64 = KERNEL_VA_BASE + KERNEL_SIZE;

/// KernelPhysicsAddressEnd
pub const KERNEL_PA_END: u64 = KERNEL_PA_BASE + KERNEL_SIZE;

/// UseremptybetweenimaginarysimulatedAddressbaseaddress
pub const USER_VA_BASE: u64 = 0x00000000_00000000;

/// UseremptybetweenimaginarysimulatedAddressEnd (low 48 Bit)
pub const USER_VA_END: u64 = 0x00008000_00000000;

/// UseremptybetweenSize (128TB)
pub const USER_VA_SIZE: u64 = USER_VA_END - USER_VA_BASE;

/// pageSize (4KB)
pub const PAGE_SIZE: u64 = 4096;

/// largepageSize (2MB)
pub const HUGE_PAGE_SIZE: u64 = 2 * 1024 * 1024;

/// exceedlargepageSize (1GB)
pub const SUPER_PAGE_SIZE: u64 = 1024 * 1024 * 1024;

/// Page TableLevel
pub const PG_LEVEL_0: usize = 0; // PGD: 512GB per entry
pub const PG_LEVEL_1: usize = 1; // PUD: 1GB per entry
pub const PG_LEVEL_2: usize = 2; // PMD: 2MB per entry
pub const PG_LEVEL_3: usize = 3; // PTE: 4KB per entry

/// PerlevelPage table entrynumber
pub const PTRS_PER_TABLE: usize = 512;

/// imaginarysimulatedAddressBitnumber
pub const VA_BITS: u64 = 48;

/// PhysicsAddressBitnumber
pub const PA_BITS: u64 = 48;

/// KernelimaginarysimulatedAddressOffset
pub const KERNEL_VA_OFFSET: u64 = KERNEL_VA_BASE - KERNEL_PA_BASE;

/// PhysicsMemorybaseaddress (secondaryDeviceTreeGet)
pub const PHYS_MEMORY_BASE: u64 = 0x40000000; // 1GB

/// PhysicsMemorySize (falseset 8GB)
pub const PHYS_MEMORY_SIZE: u64 = 8 * 1024 * 1024 * 1024;

/// DMA Regionbaseaddress
pub const DMA_ZONE_BASE: u64 = 0x00000000;

/// DMA RegionSize (16MB)
pub const DMA_ZONE_SIZE: u64 = 16 * 1024 * 1024;

/// Normal Regionbaseaddress
pub const NORMAL_ZONE_BASE: u64 = DMA_ZONE_BASE + DMA_ZONE_SIZE;

/// Normal RegionSize
pub const NORMAL_ZONE_SIZE: u64 = PHYS_MEMORY_SIZE - DMA_ZONE_SIZE;

/// HighMem Regionbaseaddress (ifPhysicsMemory > 4GB)
pub const HIGHMEM_ZONE_BASE: u64 = 0x100000000; // 4GB

/// DeviceMemoryRegionbaseaddress
pub const DEVICE_MEMORY_BASE: u64 = 0x00000000_00000000;

/// DeviceMemoryRegionSize (1GB)
pub const DEVICE_MEMORY_SIZE: u64 = 1 * 1024 * 1024 * 1024;

/// MMIO Regionbaseaddress
pub const MMIO_BASE: u64 = 0xF0000000;

/// MMIO RegionSize (256MB)
pub const MMIO_SIZE: u64 = 256 * 1024 * 1024;

/// StackSize (64KB)
pub const STACK_SIZE: u64 = 64 * 1024;

/// StackAlignment
pub const STACK_ALIGN: u64 = 16;

/// KernelStackcount
pub const KERNEL_STACK_COUNT: usize = 8;

/// UserStackSize (8MB)
pub const USER_STACK_SIZE: u64 = 8 * 1024 * 1024;

/// HeapstartbeginAddress
pub const HEAP_START: u64 = KERNEL_VA_END;

/// HeapSize (1GB)
pub const HEAP_SIZE: u64 = 1024 * 1024 * 1024;

/// HeapEndAddress
pub const HEAP_END: u64 = HEAP_START + HEAP_SIZE;

/// VDSO baseaddress
pub const VDSO_BASE: u64 = 0x7FF00000_00000000;

/// VDSO Size (4MB)
pub const VDSO_SIZE: u64 = 4 * 1024 * 1024;

/// VVAR baseaddress
pub const VVAR_BASE: u64 = VDSO_BASE + VDSO_SIZE;

/// VVAR Size (4MB)
pub const VVAR_SIZE: u64 = 4 * 1024 * 1024;

/// Useremptybetween mmap baseaddress
pub const MMAP_BASE: u64 = 0x00004000_00000000;

/// Useremptybetween mmap End
pub const MMAP_END: u64 = USER_VA_END - USER_STACK_SIZE;

/// AddressconvertFunction

/// PhysicsAddressbranchimaginarysimulatedAddress
#[inline]
pub const fn phys_to_virt(pa: u64) -> u64 {
 pa + KERNEL_VA_OFFSET
}

/// imaginarysimulatedAddressbranchPhysicsAddress
#[inline]
pub const fn virt_to_phys(va: u64) -> u64 {
 va - KERNEL_VA_OFFSET
}

/// CheckifasKernelimaginarysimulatedAddress
#[inline]
pub const fn is_kernel_address(va: u64) -> bool {
 va >= KERNEL_VA_BASE && va < KERNEL_VA_END
}

/// CheckifasUserimaginarysimulatedAddress
#[inline]
pub const fn is_user_address(va: u64) -> bool {
 va < USER_VA_END
}

/// CheckifasDeviceAddress
#[inline]
pub const fn is_device_address(pa: u64) -> bool {
 pa >= DEVICE_MEMORY_BASE && pa < DEVICE_MEMORY_BASE + DEVICE_MEMORY_SIZE
}

/// Page alignment
#[inline]
pub const fn page_align(addr: u64) -> u64 {
 (addr + PAGE_SIZE - 1) & !(PAGE_SIZE - 1)
}

/// Page alignmentdirectiondownload
#[inline]
pub const fn page_align_down(addr: u64) -> u64 {
 addr & !(PAGE_SIZE - 1)
}

/// CheckifPage alignment
#[inline]
pub const fn is_page_aligned(addr: u64) -> bool {
 (addr & (PAGE_SIZE - 1)) == 0
}

/// Getpagesignal
#[inline]
pub const fn page_number(addr: u64) -> u64 {
 addr / PAGE_SIZE
}

/// secondarypagesignalGetAddress
#[inline]
pub const fn page_address(pfn: u64) -> u64 {
 pfn * PAGE_SIZE
}

/// Get PGD Index
#[inline]
pub const fn pgd_index(va: u64) -> usize {
 ((va >> 39) & 0x1FF) as usize
}

/// Get PUD Index
#[inline]
pub const fn pud_index(va: u64) -> usize {
 ((va >> 30) & 0x1FF) as usize
}

/// Get PMD Index
#[inline]
pub const fn pmd_index(va: u64) -> usize {
 ((va >> 21) & 0x1FF) as usize
}

/// Get PTE Index
#[inline]
pub const fn pte_index(va: u64) -> usize {
 ((va >> 12) & 0x1FF) as usize
}

/// GetpageinsideOffset
#[inline]
pub const fn page_offset(va: u64) -> u64 {
 va & (PAGE_SIZE - 1)
}

/// MemoryRegionstruct
#[derive(Debug, Clone, Copy)]
pub struct MemoryRegion {
 pub start: u64,
 pub end: u64,
 pub name: &'static str,
}

/// KernelMemoryRegion
pub const KERNEL_REGION: MemoryRegion = MemoryRegion {
 start: KERNEL_VA_BASE,
 end: KERNEL_VA_END,
 name: "Kernel",
};

/// UserMemoryRegion
pub const USER_REGION: MemoryRegion = MemoryRegion {
 start: USER_VA_BASE,
 end: USER_VA_END,
 name: "User",
};

/// DMA MemoryRegion
pub const DMA_REGION: MemoryRegion = MemoryRegion {
 start: DMA_ZONE_BASE,
 end: DMA_ZONE_BASE + DMA_ZONE_SIZE,
 name: "DMA",
};

/// Normal MemoryRegion
pub const NORMAL_REGION: MemoryRegion = MemoryRegion {
 start: NORMAL_ZONE_BASE,
 end: NORMAL_ZONE_BASE + NORMAL_ZONE_SIZE,
 name: "Normal",
};

/// MemoryType
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryType {
 /// DeviceMemory (nGnRnE)
 Device,
 /// DeviceMemory (nGnRE)
 DeviceNonCoherent,
 /// Memory (Caching)
 NormalNonCacheable,
 /// Memory (Caching)
 NormalCacheable,
 /// Memory (write)
 NormalWriteThrough,
}

/// MemoryProperty
#[derive(Debug, Clone, Copy)]
pub struct MemoryAttributes {
 pub mem_type: MemoryType,
 pub executable: bool,
 pub readable: bool,
 pub writable: bool,
 pub user_accessible: bool,
 pub global: bool,
}

impl MemoryAttributes {
 /// KernelCodeparagraphProperty
 pub const KERNEL_CODE: Self = MemoryAttributes {
 mem_type: MemoryType::NormalCacheable,
 executable: true,
 readable: true,
 writable: false,
 user_accessible: false,
 global: true,
 };
 
 /// KernelDataparagraphProperty
 pub const KERNEL_DATA: Self = MemoryAttributes {
 mem_type: MemoryType::NormalCacheable,
 executable: false,
 readable: true,
 writable: true,
 user_accessible: false,
 global: true,
 };
 
 /// UserCodeparagraphProperty
 pub const USER_CODE: Self = MemoryAttributes {
 mem_type: MemoryType::NormalCacheable,
 executable: true,
 readable: true,
 writable: false,
 user_accessible: true,
 global: false,
 };
 
 /// UserDataparagraphProperty
 pub const USER_DATA: Self = MemoryAttributes {
 mem_type: MemoryType::NormalCacheable,
 executable: false,
 readable: true,
 writable: true,
 user_accessible: true,
 global: false,
 };
 
 /// DeviceMemoryProperty
 pub const DEVICE: Self = MemoryAttributes {
 mem_type: MemoryType::Device,
 executable: false,
 readable: true,
 writable: true,
 user_accessible: false,
 global: true,
 };
}

#[cfg(test)]
mod tests {
 use super::*;
 
 #[test]
 fn test_address_conversion() {
 let pa = 0x100000u64;
 let va = phys_to_virt(pa);
 assert_eq!(va, pa + KERNEL_VA_OFFSET);
 assert_eq!(virt_to_phys(va), pa);
 }
 
 #[test]
 fn test_page_alignment() {
 assert_eq!(page_align(0x1000), 0x1000);
 assert_eq!(page_align(0x1001), 0x2000);
 assert_eq!(page_align_down(0x1FFF), 0x1000);
 assert!(is_page_aligned(0x1000));
 assert!(!is_page_aligned(0x1001));
 }
 
 #[test]
 fn test_page_indices() {
 let va = 0xFFFFFF80_00123456u64;
 assert_eq!(pgd_index(va), 0x1FF);
 assert_eq!(pud_index(va), 0x000);
 assert_eq!(pmd_index(va), 0x000);
 assert_eq!(pte_index(va), 0x123);
 assert_eq!(page_offset(va), 0x456);
 }
}