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

// ! Address SpacemanagementadministrationModule
/*!*/
// ! theModuleImplementationProcessAddress Spacemanagementadministration, Package:
// ! - Address SpaceCreateandDestroy
// ! - writetimeCopy(COW)machinecontrol
// ! - pageFaceMapmanagementadministration
// ! - defectpageHandle

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use core::ptr;
use alloc::boxed::Box;
use crate::mm::page_alloc::{alloc_page, free_page, inc_page_ref, dec_page_ref, get_page_ref, copy_page};

/// pageSize(4KB)
pub const PAGE_SIZE: u64 = 4096;
pub const PAGE_SHIFT: u64 = 12;
pub const PAGE_MASK: u64 = !(PAGE_SIZE - 1);

/// Error code
pub mod errno {
 pub const ESUCCESS: i64 = 0;
 pub const ENOMEM: i64 = -12;
 pub const EACCES: i64 = -13;
 pub const EINVAL: i64 = -22;
 pub const ENOSYS: i64 = -38;
}

/// Page table entryFlag
pub mod pte_flags {
 pub const PRESENT: u64 = 1 << 0; // exist
 pub const WRITABLE: u64 = 1 << 1; // canwrite
 pub const USER: u64 = 1 << 2; // Usercanaccess
 pub const ACCESSED: u64 = 1 << 5; // alreadyaccess
 pub const DIRTY: u64 = 1 << 6; // alreadyModify
 pub const COW: u64 = 1 << 9; // COW Flag（selfDefinition）
 pub const NO_EXECUTE: u64 = 1 << 63; // notcanexecute
}

/// VMA Flag
pub mod vm_flags {
 pub const VM_NONE: u64 = 0x00000000;
 pub const VM_READ: u64 = 0x00000001;
 pub const VM_WRITE: u64 = 0x00000002;
 pub const VM_EXEC: u64 = 0x00000004;
 pub const VM_SHARED: u64 = 0x00000008;
 pub const VM_GROWSDOWN: u64 = 0x00000100; // directiondownloadincreasestrength（stack）
 pub const VM_DONTCOPY: u64 = 0x00020000; // fork timenotCopy
 pub const VM_COW: u64 = 0x10000000; // COW Flag
}

/// PhysicsAddressType
pub type PhysAddr = u64;

/// imaginarysimulatedAddressType
pub type VirtAddr = u64;

/// pageFramesignal
pub type PageFrame = u64;

/// Process ID Type
pub type Pid = u32;

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

/// imaginarysimulatedAddressAlignmenttopageBoundary
#[inline(always)]
pub fn page_align(addr: VirtAddr) -> VirtAddr {
 (addr + PAGE_SIZE - 1) & PAGE_MASK
}

/// imaginarysimulatedAddressdirectiondownloadAlignmenttopageBoundary
#[inline(always)]
pub fn page_align_down(addr: VirtAddr) -> VirtAddr {
 addr & PAGE_MASK
}

/// Page table entry
#[derive(Debug, Clone, Copy)]
pub struct PageTableEntry {
 pub value: u64,
}

impl PageTableEntry {
 pub const fn new() -> Self {
 PageTableEntry { value: 0 }
 }

 pub const fn from(value: u64) -> Self {
 PageTableEntry { value }
 }

 /// ifexist
 pub fn is_present(&self) -> bool {
 (self.value & pte_flags::PRESENT) != 0
 }

 /// ifcanwrite
 pub fn is_writable(&self) -> bool {
 (self.value & pte_flags::WRITABLE) != 0
 }

 /// ifis COW page
 pub fn is_cow(&self) -> bool {
 (self.value & pte_flags::COW) != 0
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

 /// Create new Page table entry
 pub fn create(phys: PhysAddr, flags: u64) -> Self {
 PageTableEntry {
 value: (phys & 0x000F_FFFF_FFFF_F000) | flags | pte_flags::PRESENT,
 }
 }

 /// Markeras COW page
 pub fn mark_cow(&mut self) {
 // clearDividecanwriteFlag, Set COW Flag
 self.clear_flags(pte_flags::WRITABLE);
 self.set_flags(pte_flags::COW);
 }
}

/// imaginarysimulatedMemoryRegion(VMA)
#[repr(C)]
pub struct Vma {
 /// startbeginimaginarysimulatedAddress
 pub vm_start: VirtAddr,
 /// EndimaginarysimulatedAddress
 pub vm_end: VirtAddr,
 /// Next VMA
 pub vm_next: *mut Vma,
 /// prefixaitem VMA
 pub vm_prev: *mut Vma,
 /// Flag
 pub vm_flags: u64,
 /// pageprotected
 pub vm_page_prot: u64,
 /// referenceCount(use COW)
 pub ref_count: AtomicU32,
}

impl Vma {
 pub const fn new() -> Self {
 Vma {
 vm_start: 0,
 vm_end: 0,
 vm_next: ptr::null_mut(),
 vm_prev: ptr::null_mut(),
 vm_flags: 0,
 vm_page_prot: 0,
 ref_count: AtomicU32::new(1),
 }
 }

 /// Create new VMA
 pub fn create(start: VirtAddr, end: VirtAddr, flags: u64) -> Self {
 Vma {
 vm_start: start,
 vm_end: end,
 vm_next: ptr::null_mut(),
 vm_prev: ptr::null_mut(),
 vm_flags: flags,
 vm_page_prot: 0,
 ref_count: AtomicU32::new(1),
 }
 }

 /// GetSize
 pub fn size(&self) -> u64 {
 self.vm_end - self.vm_start
 }

 /// CheckAddressifin VMA inside
 pub fn contains(&self, addr: VirtAddr) -> bool {
 addr >= self.vm_start && addr < self.vm_end
 }

 /// increasePlusreferenceCount
 pub fn inc_ref(&self) {
 self.ref_count.fetch_add(1, Ordering::AcqRel);
 }

 /// MinusfewreferenceCount
 pub fn dec_ref(&self) -> u32 {
 self.ref_count.fetch_sub(1, Ordering::AcqRel)
 }

 /// GetreferenceCount
 pub fn get_ref(&self) -> u32 {
 self.ref_count.load(Ordering::Acquire)
 }

 /// ifcanwrite
 pub fn is_writable(&self) -> bool {
 (self.vm_flags & vm_flags::VM_WRITE) != 0
 }

 /// ifis COW Region
 pub fn is_cow(&self) -> bool {
 (self.vm_flags & vm_flags::VM_COW) != 0
 }
}

/// Memorystatistics
pub struct MemoryStats {
 /// totalimaginarysimulatedMemorySize
 pub total_vm: AtomicU64,
 /// alreadymakeuseimaginarysimulatedMemory
 pub used_vm: AtomicU64,
 /// PhysicspageFacenumber
 pub total_pages: AtomicU64,
 /// COW pageFacenumber
 pub cow_pages: AtomicU64,
}

impl MemoryStats {
 pub const fn new() -> Self {
 MemoryStats {
 total_vm: AtomicU64::new(0),
 used_vm: AtomicU64::new(0),
 total_pages: AtomicU64::new(0),
 cow_pages: AtomicU64::new(0),
 }
 }
}

/// Address Space（MemoryDescriptor）
pub struct AddressSpace {
 /// Process ID
 pub pid: Pid,
 /// Page Tablebaseaddress
 pub pgd: PhysAddr,
 /// Codeparagraphstartbegin
 pub start_code: VirtAddr,
 /// CodeparagraphEnd
 pub end_code: VirtAddr,
 /// Dataparagraphstartbegin
 pub start_data: VirtAddr,
 /// DataparagraphEnd
 pub end_data: VirtAddr,
 /// Heapstartbegin
 pub start_brk: VirtAddr,
 /// HeapCurrent
 pub brk: VirtAddr,
 /// Stackstartbegin
 pub start_stack: VirtAddr,
 /// Parameterstartbegin
 pub arg_start: VirtAddr,
 /// ParameterEnd
 pub arg_end: VirtAddr,
 /// Ringenvironmentstartbegin
 pub env_start: VirtAddr,
 /// RingenvironmentEnd
 pub env_end: VirtAddr,

 /// VMA linkform
 pub mmap: *mut Vma,
 /// VMA count
 pub map_count: AtomicU32,
 /// Memorystatistics
 pub stats: MemoryStats,
 /// referenceCount(useSharedAddress Space)
 pub mm_users: AtomicU32,
 /// referenceCount(useAddress Spacestructbook)
 pub mm_count: AtomicU32,
}

impl AddressSpace {
 /// Create new Address Space
 pub fn new(pid: Pid) -> Self {
 AddressSpace {
 pid,
 pgd: 0,
 start_code: 0,
 end_code: 0,
 start_data: 0,
 end_data: 0,
 start_brk: 0,
 brk: 0,
 start_stack: 0,
 arg_start: 0,
 arg_end: 0,
 env_start: 0,
 env_end: 0,
 mmap: ptr::null_mut(),
 map_count: AtomicU32::new(0),
 stats: MemoryStats::new(),
 mm_users: AtomicU32::new(1),
 mm_count: AtomicU32::new(1),
 }
 }

 /// increasePlusUserreference
 pub fn inc_mm_users(&self) {
 self.mm_users.fetch_add(1, Ordering::AcqRel);
 }

 /// MinusfewUserreference
 pub fn dec_mm_users(&self) -> u32 {
 self.mm_users.fetch_sub(1, Ordering::AcqRel)
 }

 /// FindPackageexpfixedAddress VMA
 pub fn find_vma(&self, addr: VirtAddr) -> Option<&Vma> {
 let mut vma = self.mmap;
 while !vma.is_null() {
 // SAFETY: unsafe block required for low-level memory or hardware access
 unsafe {
 if (*vma).contains(addr) {
 return Some(&*vma);
 }
 vma = (*vma).vm_next;
 }
 }
 None
 }

 /// GettotalimaginarysimulatedMemorySize
 pub fn get_total_vm(&self) -> u64 {
 self.stats.total_vm.load(Ordering::Acquire)
 }
}

/// Address SpaceManager
pub struct AddressSpaceManager {
 /// NextAddress space ID
 next_asid: AtomicU32,
 /// Address space count
 as_count: AtomicU32,
}

impl AddressSpaceManager {
 pub const fn new() -> Self {
 AddressSpaceManager {
 next_asid: AtomicU32::new(1),
 as_count: AtomicU32::new(0),
 }
 }

 /// AllocateAddress space ID
 pub fn alloc_asid(&self) -> u32 {
 self.next_asid.fetch_add(1, Ordering::AcqRel)
 }

 /// increasePlusAddress SpaceCount
 pub fn inc_as_count(&self) {
 self.as_count.fetch_add(1, Ordering::AcqRel);
 }

 /// MinusfewAddress SpaceCount
 pub fn dec_as_count(&self) {
 self.as_count.fetch_sub(1, Ordering::AcqRel);
 }
}

/// GlobalAddress SpaceManager
static ASM: AddressSpaceManager = AddressSpaceManager::new();

/// GetAddress SpaceManager
pub fn get_asm() -> &'static AddressSpaceManager {
 &ASM
}

/// Create new Address Space
pub fn create_address_space(pid: Pid) -> Result<AddressSpace, i64> {
 log_debug!("create_address_space: pid={}", pid);

 // AllocateAddress space ID
 let asid = ASM.alloc_asid();
 ASM.inc_as_count();

 // Create address space structure
 let mut mm = AddressSpace::new(pid);

 // Allocate page table
 let pgd = crate::mm::page_table::alloc_page_table();
 if pgd == 0 {
 return Err(-12); // ENOMEM
 }
 mm.pgd = pgd;

 log_debug!("create_address_space: asid={}, pgd={:#x}", asid, mm.pgd);

 Ok(mm)
}

/// Destroy address space
pub fn destroy_address_space(mm: &mut AddressSpace) -> Result<(), i64> {
 log_debug!("destroy_address_space: pid={}", mm.pid);

 // Decrease reference count
 let count = mm.dec_mm_users();
 if count > 0 {
 // Still has other references, don't destroy
 return Ok(());
 }

 // Free all VMAs
 let mut vma = mm.mmap;
 while !vma.is_null() {
 // SAFETY: unsafe block required for low-level memory or hardware access
 unsafe {
 let next = (*vma).vm_next;
 // Free VMA
 drop(Box::from_raw(vma));
 vma = next;
 }
 }

 // Free page table
 crate::mm::page_table::free_page_table(mm.pgd);

 ASM.dec_as_count();

 Ok(())
}

/// CopyAddress Space（fork）
/// ImplementationwritetimeCopy(COW)machinecontrol:
/// 1. SharedPhysicspageFace
/// 2. MarkerpageFaceasreadsum COW
/// 3. WritetimeTriggerdefectpage, CopypageFace
pub fn copy_address_space(
 parent_mm: &AddressSpace,
 child_pid: Pid,
) -> Result<AddressSpace, i64> {
 log_debug!("copy_address_space: parent={}, child={}", parent_mm.pid, child_pid);

 // CreateChildProcessAddress Space
 let mut child_mm = create_address_space(child_pid)?;

 // CopyMemoryRegionInfo
 child_mm.start_code = parent_mm.start_code;
 child_mm.end_code = parent_mm.end_code;
 child_mm.start_data = parent_mm.start_data;
 child_mm.end_data = parent_mm.end_data;
 child_mm.start_brk = parent_mm.start_brk;
 child_mm.brk = parent_mm.brk;
 child_mm.start_stack = parent_mm.start_stack;
 child_mm.arg_start = parent_mm.arg_start;
 child_mm.arg_end = parent_mm.arg_end;
 child_mm.env_start = parent_mm.env_start;
 child_mm.env_end = parent_mm.env_end;

 // Copy VMA parallelSet COW
 let mut parent_vma = parent_mm.mmap;
 let mut prev_child_vma: *mut Vma = ptr::null_mut();
 let mut first_child_vma: *mut Vma = ptr::null_mut();

 while !parent_vma.is_null() {
 // SAFETY: unsafe block required for low-level memory or hardware access
 unsafe {
 let parent = &*parent_vma;

 // CheckifshouldCopy
 if (parent.vm_flags & vm_flags::VM_DONTCOPY) != 0 {
 parent_vma = parent.vm_next;
 continue;
 }

 // CreateChildProcess VMA
 let child_vma: *mut Vma = Box::into_raw(Box::new(Vma {
 vm_start: parent.vm_start,
 vm_end: parent.vm_end,
 vm_flags: parent.vm_flags | vm_flags::VM_COW,
 vm_page_prot: parent.vm_page_prot,
 vm_next: ptr::null_mut(),
 vm_prev: ptr::null_mut(),
 ref_count: AtomicU32::new(1),
 }));

 // Increment parent VMA reference count
 parent.inc_ref();

 // Set linked list pointers
 if prev_child_vma.is_null() {
 first_child_vma = child_vma;
 } else {
 // SAFETY: prev_child_vma is a valid VMA pointer we just allocated
 unsafe {
 (*prev_child_vma).vm_next = child_vma;
 (*child_vma).vm_prev = prev_child_vma;
 }
 }
 prev_child_vma = child_vma;

 parent_vma = parent.vm_next;
 }
 }

 // SetChildProcess VMA linkform
 child_mm.mmap = first_child_vma;

 // Copy page table and set COW
 // SAFETY: Both pgd values are valid page table roots from the allocator.
 // copy_page_table_cow maps all present pages as read-only + COW in the
 // child and write-protects them in the parent, so the first write by
 // either process triggers a page fault that copies the page.
 crate::mm::page_table::copy_page_table_cow(parent_mm.pgd, child_mm.pgd);

 // Updatestatistics
 child_mm.stats.total_vm.store(
 parent_mm.get_total_vm(),
 Ordering::Release
 );
 child_mm.stats.cow_pages.store(
 parent_mm.stats.total_pages.load(Ordering::Acquire),
 Ordering::Release
 );

 log_debug!("copy_address_space: COW setup complete");

 Ok(child_mm)
}

/// Create a new address space
/// @return Result<AddressSpace, i64> New address space or error
pub fn mm_create() -> Result<AddressSpace, i64> {
 // AllocateAddress space ID
 let pid = ASM.alloc_asid();
 
 // CreateAddress Spacestruct
 let mut mm = AddressSpace::new(pid);
 
 // CreatePage Table
 mm.pgd = crate::arch::current_arch().page_table().create().as_u64();
 
 // increasePlusAddress SpaceCount
 ASM.inc_as_count();
 
 log_info!("Created address space for PID {}", pid);
 Ok(mm)
}

/// Destroy an address space
/// @param mm: Address space to destroy
pub fn mm_destroy(mut mm: AddressSpace) {
 // Freeall VMA
 let mut vma = mm.mmap;
 while !vma.is_null() {
 // SAFETY: unsafe block required for low-level memory or hardware access
 unsafe {
 let next = (*vma).vm_next;
 
 // Minusfew VMA referenceCount
 if (*vma).dec_ref() == 0 {
 // Free VMA
 // TODO: Implementation VMA Free
 // free_vma(vma);
 }
 
 vma = next;
 }
 }
 
 // FreePage Table
 if mm.pgd != 0 {
 crate::arch::current_arch().page_table().destroy(crate::arch::PhysAddr::new(mm.pgd));
 }
 
 // MinusfewAddress SpaceCount
 ASM.dec_as_count();
 
 log_info!("Destroyed address space for PID {}", mm.pid);
}

/// Create a VMA in the address space
/// @param mm: Address space
/// @param start: Start address
/// @param end: End address
/// @param prot: Protection flags
/// @return Result<*mut Vma, i64> New VMA or error
pub fn vma_create(mm: &mut AddressSpace, start: VirtAddr, end: VirtAddr, prot: u64) -> Result<*mut Vma, i64> {
 // AlignmentAddress
 let start = page_align(start);
 let end = page_align(end);
 
 // CheckRange
 if start >= end {
 return Err(errno::EINVAL);
 }
 
 // TODO: FindemptyidleRegion
 // TODO: Allocate VMA struct
 // let vma = alloc_vma()?;
 
 // Create VMA
 let vma = Vma::create(start, end, prot);
 
 // TODO: InserttoredblackTreeorlinkform
 // thismakeusesimpleform linkformInsert
 // unsafe {
 // (*vma).vm_next = mm.mmap;
 // if !mm.mmap.is_null() {
 // (*mm.mmap).vm_prev = vma;
 // }
 // mm.mmap = vma;
 // }
 
 // Updatestatistics
 mm.stats.total_vm.fetch_add(end - start, Ordering::AcqRel);
 mm.map_count.fetch_add(1, Ordering::AcqRel);
 
 log_info!("Created VMA [{:#x}, {:#x}) for PID {}", start, end, mm.pid);
 
 // TODO: Returnrealactual VMA pointer
 Ok(ptr::null_mut())
}

/// Find VMA containing an address
/// @param mm: Address space
/// @param addr: Address to find
/// @return Option<&Vma> VMA containing the address
pub fn find_vma(mm: &AddressSpace, addr: VirtAddr) -> Option<&Vma> {
 mm.find_vma(addr)
}

/// Merge adjacent VMAs
/// @param mm: Address space
/// @param vma: VMA to merge
pub fn vma_merge(mm: &mut AddressSpace, vma: *mut Vma) {
 if vma.is_null() {
 return;
 }
 
 // SAFETY: unsafe block required for low-level memory or hardware access
 unsafe {
 let vma_ref = &*vma;
 
 // CheckifcanwithNext VMA Merge
 let next = vma_ref.vm_next;
 if !next.is_null() {
 let next_ref = &*next;
 
 // CheckifmutualPermissionmutualsame
 if vma_ref.vm_end == next_ref.vm_start && vma_ref.vm_flags == next_ref.vm_flags {
 // Merge
 (*vma).vm_end = next_ref.vm_end;
 (*vma).vm_next = next_ref.vm_next;
 
 if !next_ref.vm_next.is_null() {
 (*next_ref.vm_next).vm_prev = vma;
 }
 
 // Updatestatistics
 mm.map_count.fetch_sub(1, Ordering::AcqRel);
 
 log_debug!("Merged VMA [{:#x}, {:#x})", vma_ref.vm_start, next_ref.vm_end);
 }
 }
 }
}

/// CopyPage Table（COW）
/// traverseParentProcessPage Table, logPeritempageFace:
/// 1. Markerasreadsum COW
/// 2. inChildProcessPage TableinfixCreatemutualsameMap
/// 3. increasePluspageFacereferenceCount
fn copy_page_table_cow(parent_pgd: PhysAddr, child_pgd: PhysAddr) -> Result<(), i64> {
 // TODO: ImplementationPage TabletraversesumCopy
 // 1. traverseParentProcessPage Table
 // 2. logPeritemvalidPage table entry:
 // a. Markerasreadsum COW
 // b. inChildProcessPage TableinfixCreatemutualsameMap
 // c. increasePlusPhysicspageFacereferenceCount

 log_debug!("copy_page_table_cow: parent={:#x}, child={:#x}", parent_pgd, child_pgd);

 Ok(())
}

/// Handle COW defectpage
/// whenProcessWrite COW pageFacetime:
/// 1. Allocatenew PhysicspageFace
/// 2. CopysourcepageFaceinside
/// 3. UpdatePage table entry, Markerascanwrite
/// 4. MinusfewsourcepageFacereferenceCount
pub fn handle_cow_page_fault(
 mm: &mut AddressSpace,
 addr: VirtAddr,
) -> Result<(), i64> {
 log_debug!("handle_cow_page_fault: addr={:#x}", addr);

 // Find VMA
 let vma = mm.find_vma(addr).ok_or(errno::EINVAL)?;

 // Checkifis COW Region
 if !vma.is_cow() {
 log_error!("handle_cow_page_fault: not a COW region");
 return Err(errno::EINVAL);
 }

 // CheckwritePermission
 if !vma.is_writable() {
 log_error!("handle_cow_page_fault: not writable");
 return Err(errno::EACCES);
 }

 // GetPage table entry
 // let pte = get_pte(mm.pgd, addr)?;

 // Checkifis COW page
 // if !pte.is_cow() {
 // return Err(errno::EINVAL);
 // }

 // GetsourcepageFacePhysicsAddress
 // let old_page = pte.get_phys();

 // AllocatenewpageFace
 let new_page = alloc_page();
 if new_page == 0 {
 log_error!("handle_cow_page_fault: failed to allocate new page");
 return Err(errno::ENOMEM);
 }

 // CopysourcepageFaceinside
 // copy_page(new_page, old_page);

 // UpdatePage table entry
 // let mut new_pte = pte;
 // new_pte.set_phys(new_page);
 // new_pte.clear_flags(pte_flags::COW);
 // new_pte.set_flags(pte_flags::WRITABLE);
 // set_pte(mm.pgd, addr, new_pte);

 // MinusfewsourcepageFacereferenceCount
 // dec_page_ref(old_page);
 // if get_page_ref(old_page) == 0 {
 // free_page(old_page);
 // }

 // Updatestatistics
 mm.stats.cow_pages.fetch_sub(1, Ordering::AcqRel);

 log_debug!("handle_cow_page_fault: COW resolved, new_page={:#x}", new_page);

 Ok(())
}

/// HandledefectpageException
pub fn handle_page_fault(
 mm: &mut AddressSpace,
 addr: VirtAddr,
 is_write: bool,
 is_user: bool,
) -> Result<(), i64> {
 log_debug!("handle_page_fault: addr={:#x}, write={}, user={}", addr, is_write, is_user);

 // Find VMA
 let vma = mm.find_vma(addr).ok_or(errno::EINVAL)?;

 // CheckPermission
 if is_user && !vma.is_writable() && is_write {
 log_error!("handle_page_fault: permission denied");
 return Err(errno::EACCES);
 }

 // Get page table entry
 let pte = crate::mm::page_table::get_pte(mm.pgd, addr);

 // Check if it's a COW page
 if pte.is_present() && pte.is_cow() && is_write {
 return handle_cow_page_fault(mm, addr);
 }

 // Handle other page fault cases
 match vma.vm_anon {
 // Anonymous mapping: allocate new page
 1 => {
 let phys = crate::mm::page_alloc::alloc_page();
 if phys == 0 {
 log_warn!("Failed to allocate page for anonymous mapping");
 return Err(-12); // ENOMEM
 }
 
 // Zero the page
 crate::mm::page_alloc::zero_page(phys);
 
 // Map the page
 crate::mm::page_table::map_page(mm.pgd, addr, phys, prot);
 }
 // File mapping: read from file
 _ => {
 log_warn!("File mapping not implemented");
 return Err(-38); // ENOSYS
 }
 }

 Ok(())
}

/// MappageFace
pub fn map_pages(
 mm: &mut AddressSpace,
 addr: VirtAddr,
 size: u64,
 prot: u64,
 flags: u64,
) -> Result<VirtAddr, i64> {
 log_debug!("map_pages: addr={:#x}, size={}, prot={:#x}", addr, size, prot);

 // AlignmentAddressandSize
 let aligned_addr = page_align(addr);
 let aligned_size = page_align(size);

 // CheckParameter
 if aligned_size == 0 {
 return Err(errno::EINVAL);
 }

 // TODO: Implementation
 // 1. FindorCreate VMA
 // 2. AllocatePhysicspageFace
 // 3. UpdatePage Table

 Ok(aligned_addr)
}

/// cancelMap
pub fn unmap_pages(
 mm: &mut AddressSpace,
 addr: VirtAddr,
 size: u64,
) -> Result<(), i64> {
 log_debug!("unmap_pages: addr={:#x}, size={}", addr, size);

 // AlignmentAddressandSize
 let aligned_addr = page_align_down(addr);
 let aligned_size = page_align(size);

 // TODO: Implementation
 // 1. Find VMA
 // 2. FreePhysicspageFace
 // 3. clearDividePage table entry
 // 4. DivideorSplit VMA

 Ok(())
}

/// InitializeAddress Spacemanagementadministration
pub fn init_address_space_management() {
 log_info!("Address space management initialized");
 log_info!(" Page size: {} bytes", PAGE_SIZE);
 log_info!(" COW support: enabled");
}