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


use core::sync::atomic::{AtomicU64, AtomicU32, Ordering};

/// pageSize (4KB)
pub const PAGE_SIZE: u64 = 4096;

/// pageOffsetBitnumber
pub const PAGE_SHIFT: u64 = 12;

/// PhysicsAddressType
pub type PhysAddr = u64;

/// imaginarysimulatedAddressType
pub type VirtAddr = u64;

/// pageFramesignal
pub type PageFrame = u64;

/// PhysicsAddresstopageFramesignal
#[inline(always)]
pub fn phys_to_pfn(phys: PhysAddr) -> PageFrame {
 phys >> PAGE_SHIFT
}

/// pageFramesignaltoPhysicsAddress
#[inline(always)]
pub fn pfn_to_phys(pfn: PageFrame) -> PhysAddr {
 pfn << PAGE_SHIFT
}

/// imaginarysimulatedAddresstopageFramesignal
#[inline(always)]
pub fn virt_to_pfn(virt: VirtAddr) -> PageFrame {
 virt >> PAGE_SHIFT
}

/// pageFramesignaltoimaginarysimulatedAddress
#[inline(always)]
pub fn pfn_to_virt(pfn: PageFrame) -> VirtAddr {
 pfn << PAGE_SHIFT
}

/// Page table entryFlag
pub mod pte_flags {
 pub const PRESENT: u64 = 1 << 0;
 pub const WRITABLE: u64 = 1 << 1;
 pub const USER: u64 = 1 << 2;
 pub const WRITE_THROUGH: u64 = 1 << 3;
 pub const CACHE_DISABLE: u64 = 1 << 4;
 pub const ACCESSED: u64 = 1 << 5;
 pub const DIRTY: u64 = 1 << 6;
 pub const HUGE: u64 = 1 << 7;
 pub const GLOBAL: u64 = 1 << 8;
 pub const NO_EXECUTE: u64 = 1 << 63;
}

/// Page table entry
#[derive(Debug, Clone, Copy)]
pub struct PageTableEntry {
 pub value: u64,
}

impl PageTableEntry {
 /// CreateemptyPage table entry
 pub const fn new() -> Self {
 PageTableEntry { value: 0 }
 }
 
 /// fromvalueCreate
 pub const fn from(value: u64) -> Self {
 PageTableEntry { value }
 }
 
 /// ifexist
 pub fn is_present(&self) -> bool {
 self.value & pte_flags::PRESENT != 0
 }
 
 /// ifcanwrite
 pub fn is_writable(&self) -> bool {
 self.value & pte_flags::WRITABLE != 0
 }
 
 /// ifUsercanaccess
 pub fn is_user(&self) -> bool {
 self.value & pte_flags::USER != 0
 }
 
 /// ifislargepage
 pub fn is_huge(&self) -> bool {
 self.value & pte_flags::HUGE != 0
 }
 
 /// GetPhysicsAddress
 pub fn get_phys(&self) -> PhysAddr {
 self.value & 0x000F_FFFF_FFFF_F000
 }
 
 /// SetPhysicsAddress
 pub fn set_phys(&mut self, phys: PhysAddr) {
 self.value = (self.value & 0xFFF) | (phys & 0x000F_FFFF_FFFF_F000);
 }
 
 /// SetFlag
 pub fn set_flags(&mut self, flags: u64) {
 self.value |= flags;
 }
 
 /// clearDivideFlag
 pub fn clear_flags(&mut self, flags: u64) {
 self.value &= !flags;
 }
}

/// MemoryRegion
pub struct MemoryRegion {
 /// startbeginPhysicsAddress
 pub start: PhysAddr,
 /// Size
 pub size: u64,
 /// Type
 pub region_type: MemoryRegionType,
}

/// MemoryRegion Type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryRegionType {
 /// canuseMemory
 Available = 1,
 /// protectedMemory
 Reserved = 2,
 /// ACPI Data
 AcpiData = 3,
 /// ACPI NVS
 AcpiNvs = 4,
 /// notcanuse
 Unusable = 5,
}

/// PhysicsMemoryManager
pub struct PhysMemManager {
 /// totalpagenumber
 pub total_pages: AtomicU64,
 /// emptyidlepagenumber
 pub free_pages: AtomicU64,
 /// alreadyusepagenumber
 pub used_pages: AtomicU64,
 /// totalMemorySize
 pub total_memory: AtomicU64,
 /// emptyidleMemorySize
 pub free_memory: AtomicU64,
}

impl PhysMemManager {
 pub const fn new() -> Self {
 PhysMemManager {
 total_pages: AtomicU64::new(0),
 free_pages: AtomicU64::new(0),
 used_pages: AtomicU64::new(0),
 total_memory: AtomicU64::new(0),
 free_memory: AtomicU64::new(0),
 }
 }
 
 /// Initialize
 pub fn init(&mut self, total_memory: u64) {
 let total_pages = total_memory / PAGE_SIZE;
 
 self.total_pages.store(total_pages, Ordering::Release);
 self.free_pages.store(total_pages, Ordering::Release);
 self.used_pages.store(0, Ordering::Release);
 self.total_memory.store(total_memory, Ordering::Release);
 self.free_memory.store(total_memory, Ordering::Release);
 
 log_info!("Physical memory manager initialized");
 log_info!(" Total memory: {} MB", total_memory / (1024 * 1024));
 log_info!(" Total pages: {}", total_pages);
 log_info!(" Page size: {} KB", PAGE_SIZE / 1024);
 }
 
 /// Allocateapage
 pub fn alloc_page(&self) -> Option<PhysAddr> {
 let free = self.free_pages.fetch_sub(1, Ordering::AcqRel);
 if free == 0 {
 self.free_pages.fetch_add(1, Ordering::AcqRel);
 return None;
 }
 
 self.used_pages.fetch_add(1, Ordering::AcqRel);
 self.free_memory.fetch_sub(PAGE_SIZE, Ordering::AcqRel);
 
 // TODO: Implementationrealactual pageAllocate
 
 Some(0)
 }
 
 /// Freeapage
 pub fn free_page(&self, _phys: PhysAddr) {
 self.free_pages.fetch_add(1, Ordering::AcqRel);
 self.used_pages.fetch_sub(1, Ordering::AcqRel);
 self.free_memory.fetch_add(PAGE_SIZE, Ordering::AcqRel);
 
 // TODO: Implementationrealactual pageFree
 }
 
 /// Allocatecontinuemanypage
 pub fn alloc_pages(&self, count: u64) -> Option<PhysAddr> {
 let free = self.free_pages.load(Ordering::Acquire);
 if free < count {
 return None;
 }
 
 self.free_pages.fetch_sub(count, Ordering::AcqRel);
 self.used_pages.fetch_add(count, Ordering::AcqRel);
 self.free_memory.fetch_sub(count * PAGE_SIZE, Ordering::AcqRel);
 
 // TODO: Implementationrealactual continuepageAllocate
 
 Some(0)
 }
 
 /// Freecontinuemanypage
 pub fn free_pages(&self, _phys: PhysAddr, count: u64) {
 self.free_pages.fetch_add(count, Ordering::AcqRel);
 self.used_pages.fetch_sub(count, Ordering::AcqRel);
 self.free_memory.fetch_add(count * PAGE_SIZE, Ordering::AcqRel);
 
 // TODO: Implementationrealactual continuepageFree
 }
 
 /// Gettotalpagenumber
 pub fn get_total_pages(&self) -> u64 {
 self.total_pages.load(Ordering::Acquire)
 }
 
 /// Getemptyidlepagenumber
 pub fn get_free_pages(&self) -> u64 {
 self.free_pages.load(Ordering::Acquire)
 }
 
 /// Getalreadyusepagenumber
 pub fn get_used_pages(&self) -> u64 {
 self.used_pages.load(Ordering::Acquire)
 }
 
 /// GetMemorymakeuserate
 pub fn get_usage_percent(&self) -> u32 {
 let total = self.total_pages.load(Ordering::Acquire);
 let used = self.used_pages.load(Ordering::Acquire);
 
 if total == 0 {
 return 0;
 }
 
 ((used * 100) / total) as u32
 }
}

/// imaginarysimulatedMemoryRegion (VMA)
pub struct Vma {
 /// startbeginimaginarysimulatedAddress
 pub start: VirtAddr,
 /// EndimaginarysimulatedAddress
 pub end: VirtAddr,
 /// Flag
 pub flags: u64,
 /// Next VMA
 pub next: *mut Vma,
}

impl Vma {
 /// Createnew VMA
 pub fn new(start: VirtAddr, end: VirtAddr, flags: u64) -> Self {
 Vma {
 start,
 end,
 flags,
 next: core::ptr::null_mut(),
 }
 }
 
 /// GetSize
 pub fn size(&self) -> u64 {
 self.end - self.start
 }
 
 /// ifPackageAddress
 pub fn contains(&self, addr: VirtAddr) -> bool {
 addr >= self.start && addr < self.end
 }
}

/// Address Space
pub struct AddressSpace {
 /// Page Tablebaseaddress
 pub pgd: PhysAddr,
 /// VMA linkformHead
 pub vma_head: *mut Vma,
 /// VMA count
 pub vma_count: AtomicU32,
 /// totalimaginarysimulatedMemorySize
 pub total_vm: AtomicU64,
 /// LockedMemorySize
 pub locked_vm: AtomicU64,
}

impl AddressSpace {
 /// CreatenewAddress Space
 pub fn new(pgd: PhysAddr) -> Self {
 AddressSpace {
 pgd,
 vma_head: core::ptr::null_mut(),
 vma_count: AtomicU32::new(0),
 total_vm: AtomicU64::new(0),
 locked_vm: AtomicU64::new(0),
 }
 }
 
 /// add VMA
 pub fn add_vma(&mut self, vma: &mut Vma) {
 vma.next = self.vma_head;
 self.vma_head = vma as *mut Vma;
 self.vma_count.fetch_add(1, Ordering::AcqRel);
 self.total_vm.fetch_add(vma.size(), Ordering::AcqRel);
 }
 
 /// FindPackageAddress VMA
 pub fn find_vma(&self, addr: VirtAddr) -> Option<&Vma> {
 let mut current = self.vma_head;
 
 while !current.is_null() {
 // SAFETY: unsafe block required for low-level memory or hardware access
 unsafe {
 let vma = &*current;
 if vma.contains(addr) {
 return Some(vma);
 }
 current = vma.next;
 }
 }
 
 None
 }
}

/// GlobalPhysicsMemoryManager
static PHYS_MEM: crate::sync_oncelock::OnceLock<PhysMemManager> = crate::sync_oncelock::OnceLock::new();

pub fn phys_mem() -> &'static PhysMemManager {
    PHYS_MEM.get_or_init(PhysMemManager::new)
}

pub fn init_phys_mem(total_memory: u64) {
 let mem = phys_mem();
 mem.init(total_memory);
}

#[cfg(test)]
mod tests {
 use super::*;

 #[test]
 fn test_page_constants() {
 assert_eq!(PAGE_SIZE, 4096);
 assert_eq!(PAGE_SHIFT, 12);
 }

 #[test]
 fn test_phys_to_pfn() {
 assert_eq!(phys_to_pfn(0), 0);
 assert_eq!(phys_to_pfn(4096), 1);
 assert_eq!(phys_to_pfn(8192), 2);
 assert_eq!(phys_to_pfn(4096 * 100), 100);
 }

 #[test]
 fn test_pfn_to_phys() {
 assert_eq!(pfn_to_phys(0), 0);
 assert_eq!(pfn_to_phys(1), 4096);
 assert_eq!(pfn_to_phys(2), 8192);
 }

 #[test]
 fn test_virt_to_pfn() {
 assert_eq!(virt_to_pfn(0), 0);
 assert_eq!(virt_to_pfn(4096), 1);
 }

 #[test]
 fn test_pfn_to_virt() {
 assert_eq!(pfn_to_virt(0), 0);
 assert_eq!(pfn_to_virt(1), 4096);
 }

 #[test]
 fn test_pte_flags() {
 assert_eq!(pte_flags::PRESENT, 1 << 0);
 assert_eq!(pte_flags::WRITABLE, 1 << 1);
 assert_eq!(pte_flags::USER, 1 << 2);
 assert_eq!(pte_flags::NO_EXECUTE, 1 << 63);
 }

 #[test]
 fn test_page_table_entry_new() {
 let pte = PageTableEntry::new();
 assert_eq!(pte.value, 0);
 assert!(!pte.is_present());
 assert!(!pte.is_writable());
 assert!(!pte.is_user());
 }

 #[test]
 fn test_page_table_entry_flags() {
 let mut pte = PageTableEntry::new();

 pte.set_flags(pte_flags::PRESENT | pte_flags::WRITABLE);

 assert!(pte.is_present());
 assert!(pte.is_writable());
 assert!(!pte.is_user());

 pte.clear_flags(pte_flags::WRITABLE);
 assert!(!pte.is_writable());
 assert!(pte.is_present());
 }

 #[test]
 fn test_page_table_entry_phys() {
 let mut pte = PageTableEntry::new();

 pte.set_phys(0x1000);
 assert_eq!(pte.get_phys(), 0x1000);

 pte.set_phys(0x1234000);
 assert_eq!(pte.get_phys(), 0x1234000);
 }

 #[test]
 fn test_memory_region_type() {
 assert_eq!(MemoryRegionType::Available as u32, 1);
 assert_eq!(MemoryRegionType::Reserved as u32, 2);
 assert_eq!(MemoryRegionType::AcpiData as u32, 3);
 assert_eq!(MemoryRegionType::AcpiNvs as u32, 4);
 assert_eq!(MemoryRegionType::Unusable as u32, 5);
 }

 #[test]
 fn test_phys_mem_manager_new() {
 let mem = PhysMemManager::new();

 assert_eq!(mem.get_total_pages(), 0);
 assert_eq!(mem.get_free_pages(), 0);
 assert_eq!(mem.get_used_pages(), 0);
 }

 #[test]
 fn test_phys_mem_manager_init() {
 let mut mem = PhysMemManager::new();

 // Initialize 1GB Memory
 mem.init(1024 * 1024 * 1024);

 let expected_pages = (1024 * 1024 * 1024) / PAGE_SIZE;
 assert_eq!(mem.get_total_pages(), expected_pages);
 assert_eq!(mem.get_free_pages(), expected_pages);
 assert_eq!(mem.get_used_pages(), 0);
 }

 #[test]
 fn test_phys_mem_manager_alloc_page() {
 let mut mem = PhysMemManager::new();
 mem.init(1024 * 1024 * 1024);

 let initial_free = mem.get_free_pages();

 let page = mem.alloc_page();
 assert!(page.is_some());
 assert_eq!(mem.get_free_pages(), initial_free - 1);
 assert_eq!(mem.get_used_pages(), 1);
 }

 #[test]
 fn test_phys_mem_manager_free_page() {
 let mut mem = PhysMemManager::new();
 mem.init(1024 * 1024 * 1024);

 let page = mem.alloc_page().unwrap();
 let used = mem.get_used_pages();

 mem.free_page(page);

 assert_eq!(mem.get_used_pages(), used - 1);
 }

 #[test]
 fn test_phys_mem_manager_alloc_pages() {
 let mut mem = PhysMemManager::new();
 mem.init(1024 * 1024 * 1024);

 let pages = mem.alloc_pages(10);
 assert!(pages.is_some());
 assert_eq!(mem.get_used_pages(), 10);
 }

 #[test]
 fn test_phys_mem_manager_alloc_pages_insufficient() {
 let mut mem = PhysMemManager::new();
 mem.init(4096 * 5); // finite 5 page

 let pages = mem.alloc_pages(10); // Request 10 page
 assert!(pages.is_none());
 }

 #[test]
 fn test_phys_mem_manager_usage_percent() {
 let mut mem = PhysMemManager::new();
 mem.init(4096 * 100); // 100 page

 assert_eq!(mem.get_usage_percent(), 0);

 mem.alloc_pages(50);
 assert_eq!(mem.get_usage_percent(), 50);

 mem.alloc_pages(25);
 assert_eq!(mem.get_usage_percent(), 75);
 }

 #[test]
 fn test_vma_new() {
 let vma = Vma::new(0x1000, 0x2000, 0);

 assert_eq!(vma.start, 0x1000);
 assert_eq!(vma.end, 0x2000);
 assert_eq!(vma.size(), 0x1000);
 }

 #[test]
 fn test_vma_contains() {
 let vma = Vma::new(0x1000, 0x2000, 0);

 assert!(!vma.contains(0x0FFF));
 assert!(vma.contains(0x1000));
 assert!(vma.contains(0x1500));
 assert!(vma.contains(0x1FFF));
 assert!(!vma.contains(0x2000));
 }

 #[test]
 fn test_address_space_new() {
 let aspace = AddressSpace::new(0x1000);

 assert_eq!(aspace.pgd, 0x1000);
 assert_eq!(aspace.vma_count.load(Ordering::Relaxed), 0);
 assert_eq!(aspace.total_vm.load(Ordering::Relaxed), 0);
 }
}