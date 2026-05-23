/*
 * Nuva OS - Kernel - Page Fault Handler
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

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// Page fault flags
pub mod fault_flags {
    pub const WRITE: u32 = 1 << 0;       /* Write fault */
    pub const USER: u32 = 1 << 1;        /* User mode fault */
    pub const INSTRUCTION: u32 = 1 << 2; /* Instruction fetch fault */
    pub const TRACE: u32 = 1 << 3;       /* Trace flag */
    pub const KILLABLE: u32 = 1 << 4;    /* Task can be killed */
    pub const TRIED: u32 = 1 << 5;       /* Already tried */
    pub const ALLOW_RETRY: u32 = 1 << 6; /* Allow retry */
    pub const RETRY_NOWAIT: u32 = 1 << 7; /* Retry without waiting */
    pub const NOWAIT: u32 = 1 << 8;      /* Don't wait */
    pub const UNSHARE: u32 = 1 << 9;     /* Unshare */
}

/// PTE flags for page table entries
pub mod pte_flags {
    pub const VALID: u64 = 1 << 0;       /* Page is valid */
    pub const TABLE: u64 = 1 << 1;       /* Page table entry */
    pub const USER: u64 = 1 << 6;        /* User accessible */
    pub const READONLY: u64 = 1 << 7;    /* Read-only (AP[1]) */
    pub const SHARED: u64 = 1 << 8;      /* Shareable */
    pub const AF: u64 = 1 << 10;         /* Access flag */
    pub const NX: u64 = 1 << 54;         /* No execute */
    pub const DIRTY: u64 = 1 << 55;      /* Dirty (software) */
    pub const COW: u64 = 1 << 56;        /* Copy-on-write (software) */
    pub const SWAP: u64 = 1 << 57;       /* Swapped out (software) */
}

/// Fault result enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaultResult {
    /// Success
    Success = 0,
    /// Need retry
    Retry = 1,
    /// Write protection violation
    WriteProtect = 2,
    /// Segmentation fault
    Segfault = 3,
    /// Bus error
    BusError = 4,
    /// Out of memory
    Oom = 5,
    /// Interrupted by signal
    Interrupted = 6,
}

/// Fault context structure
pub struct FaultContext {
    /// Fault address
    pub address: u64,
    /// Error code
    pub error_code: u32,
    /// Flags
    pub flags: u32,
    /// Page table entry
    pub pte: AtomicU64,
    /// Physical address of current page
    pub phys_addr: u64,
    /// VMA start address
    pub vma_start: u64,
    /// VMA end address
    pub vma_end: u64,
    /// VMA flags
    pub vma_flags: u32,
}

impl FaultContext {
    /// Create fault context
    pub fn new(address: u64, error_code: u32) -> Self {
        FaultContext {
            address,
            error_code,
            flags: 0,
            pte: AtomicU64::new(0),
            phys_addr: 0,
            vma_start: 0,
            vma_end: 0,
            vma_flags: 0,
        }
    }
    
    /// Check if write fault
    pub fn is_write(&self) -> bool {
        (self.error_code & fault_flags::WRITE) != 0
    }
    
    /// Check if user mode fault
    pub fn is_user(&self) -> bool {
        (self.error_code & fault_flags::USER) != 0
    }
    
    /// Check if instruction fetch fault
    pub fn is_instruction(&self) -> bool {
        (self.error_code & fault_flags::INSTRUCTION) != 0
    }
    
    /// Check if address is in VMA range
    pub fn in_vma(&self) -> bool {
        self.address >= self.vma_start && self.address < self.vma_end
    }
    
    /// Get current PTE value
    pub fn get_pte(&self) -> u64 {
        self.pte.load(Ordering::Acquire)
    }
    
    /// Set PTE value
    pub fn set_pte(&self, pte: u64) {
        self.pte.store(pte, Ordering::Release);
    }
    
    /// Check if PTE is COW
    pub fn is_cow_pte(&self) -> bool {
        (self.get_pte() & pte_flags::COW) != 0
    }
    
    /// Check if PTE is swapped out
    pub fn is_swap_pte(&self) -> bool {
        (self.get_pte() & pte_flags::SWAP) != 0
    }
}

/// COW (Copy-On-Write) handler
/// Handles copy-on-write page faults for memory efficiency.
pub struct CowHandler {
    /// Total COW faults
    pub cow_faults: AtomicU64,
    /// COW pages copied
    pub cow_copies: AtomicU64,
    /// COW pages shared (ref count was 1)
    pub cow_shared: AtomicU64,
    /// COW failures
    pub cow_failures: AtomicU64,
}

impl CowHandler {
    pub const fn new() -> Self {
        CowHandler {
            cow_faults: AtomicU64::new(0),
            cow_copies: AtomicU64::new(0),
            cow_shared: AtomicU64::new(0),
            cow_failures: AtomicU64::new(0),
        }
    }
    
    /// Handle COW page fault
    /// @param ctx: Fault context
    /// @param page_ref_count: Reference count of the page
    /// @return Fault result
    pub fn handle_cow(&self, ctx: &mut FaultContext, page_ref_count: u32) -> FaultResult {
        self.cow_faults.fetch_add(1, Ordering::AcqRel);
        
        log_debug!("COW fault at {:#x}, ref_count={}", ctx.address, page_ref_count);
        
        if page_ref_count > 1 {
            // Multiple references - need to copy
            self.do_cow_copy(ctx)
        } else {
            // Single reference - just make writable
            self.do_cow_share(ctx)
        }
    }
    
    /// Perform actual COW copy
    /// Allocates new page and copies content.
    fn do_cow_copy(&self, ctx: &mut FaultContext) -> FaultResult {
        // Step 1: Allocate new page
        let new_page = self.alloc_page();
        if new_page == 0 {
            self.cow_failures.fetch_add(1, Ordering::AcqRel);
            return FaultResult::Oom;
        }
        
        // Step 2: Copy page content
        let old_phys = ctx.phys_addr;
        self.copy_page_content(new_page, old_phys);
        
        // Step 3: Update page table entry
        let new_pte = self.make_writable_pte(new_page);
        ctx.set_pte(new_pte);
        
        // Step 4: Flush TLB for this address
        self.flush_tlb(ctx.address);
        
        // Step 5: Decrement old page reference count
        self.dec_page_ref(old_phys);
        
        self.cow_copies.fetch_add(1, Ordering::AcqRel);
        
        log_debug!("COW copy: old={:#x} -> new={:#x}", old_phys, new_page);
        
        FaultResult::Success
    }
    
    /// Make page writable without copying
    /// Used when page has single reference.
    fn do_cow_share(&self, ctx: &mut FaultContext) -> FaultResult {
        // Clear COW flag and make writable
        let old_pte = ctx.get_pte();
        let new_pte = (old_pte & !pte_flags::COW) & !pte_flags::READONLY;
        ctx.set_pte(new_pte);
        
        // Flush TLB
        self.flush_tlb(ctx.address);
        
        self.cow_shared.fetch_add(1, Ordering::AcqRel);
        
        log_debug!("COW share: made writable at {:#x}", ctx.address);
        
        FaultResult::Success
    }
    
    /// Allocate a new page from the page allocator
    fn alloc_page(&self) -> u64 {
        let page = super::alloc_page();
        if page.is_null() {
            return 0;
        }
        // SAFETY: page is freshly allocated from the allocator
        unsafe { (*page).phys_addr }
    }
    
    /// Copy page content using optimized memory copy
    fn copy_page_content(&self, dst: u64, src: u64) {
        const PAGE_OFFSET: u64 = 0xFFFF_0000_0000_0000;
        let dst_vaddr = (dst + PAGE_OFFSET) as *mut u8;
        let src_vaddr = (src + PAGE_OFFSET) as *const u8;
        // SAFETY: both addresses are valid kernel direct-mapped virtual addresses
        // and do not overlap; PAGE_SIZE bytes are available at each
        unsafe {
            core::ptr::copy_nonoverlapping(src_vaddr, dst_vaddr, 4096);
        }
    }
    
    /// Make writable PTE
    fn make_writable_pte(&self, phys: u64) -> u64 {
        phys | pte_flags::VALID | pte_flags::USER | pte_flags::AF | pte_flags::DIRTY
    }
    
    /// Flush TLB entry using architecture-specific operation
    fn flush_tlb(&self, addr: u64) {
        super::page_table::flush_tlb_addr(addr);
    }
    
    /// Decrement page reference count via mem_map lookup
    fn dec_page_ref(&self, phys: u64) {
        let page = super::mem_map::get_page(phys);
        if page.is_null() {
            return;
        }
        // SAFETY: page is from mem_map, valid Page structure
        unsafe {
            let old = (*page).ref_count.fetch_sub(1, Ordering::AcqRel);
            if old == 1 {
                super::free_page(page);
            }
        }
    }
    
    /// Get COW statistics
    pub fn get_stats(&self) -> (u64, u64, u64, u64) {
        (
            self.cow_faults.load(Ordering::Acquire),
            self.cow_copies.load(Ordering::Acquire),
            self.cow_shared.load(Ordering::Acquire),
            self.cow_failures.load(Ordering::Acquire),
        )
    }
}

/// Page fault handler
pub struct PageFaultHandler {
    /// Total fault count
    pub fault_count: AtomicU64,
    /// User mode faults
    pub user_faults: AtomicU64,
    /// Kernel mode faults
    pub kernel_faults: AtomicU64,
    /// Write faults
    pub write_faults: AtomicU64,
    /// Read faults
    pub read_faults: AtomicU64,
    /// Success count
    pub success_count: AtomicU64,
    /// Failure count
    pub fail_count: AtomicU64,
    /// COW handler
    pub cow_handler: CowHandler,
    /// Swap-in count
    pub swapin_count: AtomicU64,
}

impl PageFaultHandler {
    pub const fn new() -> Self {
        PageFaultHandler {
            fault_count: AtomicU64::new(0),
            user_faults: AtomicU64::new(0),
            kernel_faults: AtomicU64::new(0),
            write_faults: AtomicU64::new(0),
            read_faults: AtomicU64::new(0),
            success_count: AtomicU64::new(0),
            fail_count: AtomicU64::new(0),
            cow_handler: CowHandler::new(),
            swapin_count: AtomicU64::new(0),
        }
    }
    
    /// Initialize
    pub fn init(&mut self) {
        log_info!("Page fault handler initialized");
        log_info!("  COW support: enabled");
    }
    
    /// Handle page fault
    /// Main entry point for all page faults.
    pub fn handle_fault(&mut self, ctx: &mut FaultContext) -> FaultResult {
        self.fault_count.fetch_add(1, Ordering::AcqRel);
        
        // Update statistics
        if ctx.is_user() {
            self.user_faults.fetch_add(1, Ordering::AcqRel);
        } else {
            self.kernel_faults.fetch_add(1, Ordering::AcqRel);
        }
        
        if ctx.is_write() {
            self.write_faults.fetch_add(1, Ordering::AcqRel);
        } else {
            self.read_faults.fetch_add(1, Ordering::AcqRel);
        }
        
        // Process the fault
        let result = self.do_page_fault(ctx);
        
        if result == FaultResult::Success {
            self.success_count.fetch_add(1, Ordering::AcqRel);
        } else {
            self.fail_count.fetch_add(1, Ordering::AcqRel);
        }
        
        result
    }
    
    /// Process page fault
    fn do_page_fault(&mut self, ctx: &mut FaultContext) -> FaultResult {
        // Step 1: Find VMA for the address
        if !self.find_vma(ctx) {
            log_warn!("No VMA for address {:#x}", ctx.address);
            return FaultResult::Segfault;
        }
        
        // Step 2: Check permissions
        if !self.check_permissions(ctx) {
            log_warn!("Permission denied for address {:#x}", ctx.address);
            return FaultResult::WriteProtect;
        }
        
        // Step 3: Get current PTE
        self.get_pte(ctx);
        
        // Step 4: Handle based on PTE state
        if ctx.is_swap_pte() {
            // Page is swapped out
            self.handle_swapin(ctx)
        } else if ctx.is_cow_pte() && ctx.is_write() {
            // COW page with write fault
            let ref_count = self.get_page_ref_count(ctx.phys_addr);
            self.cow_handler.handle_cow(ctx, ref_count)
        } else if ctx.get_pte() == 0 {
            // Page not present - anonymous mapping
            self.handle_anon_fault(ctx)
        } else {
            // Normal fault - just update access flags
            self.handle_access_fault(ctx)
        }
    }
    
    /// Find VMA for address using VMA subsystem
    fn find_vma(&self, ctx: &mut FaultContext) -> bool {
        // SAFETY: accessing current process mm is safe in fault context
        let mm = unsafe { current_mm() };
        if mm.is_null() {
            ctx.vma_start = 0x1000;
            ctx.vma_end = 0x7FFF_FFFF_F000;
            ctx.vma_flags = 0x7;
            return true;
        }
        // SAFETY: mm pointer from current_mm is valid if non-null
        let vma = unsafe { find_vma_in_mm(mm, ctx.address) };
        if vma.is_null() {
            return false;
        }
        // SAFETY: vma pointer from find_vma_in_mm is valid if non-null
        unsafe {
            ctx.vma_start = (*vma).vm_start;
            ctx.vma_end = (*vma).vm_end;
            ctx.vma_flags = (*vma).vm_flags;
        }
        true
    }
    
    /// Check permissions
    fn check_permissions(&self, ctx: &FaultContext) -> bool {
        // Check write permission
        if ctx.is_write() {
            let writable = (ctx.vma_flags & 0x2) != 0;
            if !writable {
                return false;
            }
        }
        
        // Check execute permission
        if ctx.is_instruction() {
            let executable = (ctx.vma_flags & 0x1) != 0;
            if !executable {
                return false;
            }
        }
        
        true
    }
    
    /// Get PTE for address via page table walk
    fn get_pte(&self, ctx: &mut FaultContext) {
        let pgd = unsafe { current_pgd() };
        if pgd == 0 {
            return;
        }
        let pte_val = walk_page_table(pgd, ctx.address);
        ctx.set_pte(pte_val);
        ctx.phys_addr = pte_val & 0x000F_FFFF_FFFF_F000;
    }
    
    /// Get page reference count from mem_map
    fn get_page_ref_count(&self, phys: u64) -> u32 {
        let page = super::mem_map::get_page(phys);
        if page.is_null() {
            return 0;
        }
        // SAFETY: page from mem_map is valid
        unsafe { (*page).ref_count.load(Ordering::Acquire) }
    }
    
    /// Handle anonymous page fault
    fn handle_anon_fault(&mut self, ctx: &mut FaultContext) -> FaultResult {
        log_debug!("Anonymous fault at {:#x}", ctx.address);
        
        // Allocate new page
        let page = self.alloc_zeroed_page();
        if page == 0 {
            return FaultResult::Oom;
        }
        
        // Create PTE
        let mut pte = page | pte_flags::VALID | pte_flags::USER | pte_flags::AF;
        
        // Set COW flag if this is a shared mapping
        if self.is_shared_mapping(ctx) {
            pte |= pte_flags::COW | pte_flags::READONLY;
        }
        
        // Add write permission if writable and not shared
        if !ctx.is_write() && !self.is_shared_mapping(ctx) {
            pte &= !pte_flags::READONLY;
        }
        
        ctx.set_pte(pte);
        
        FaultResult::Success
    }
    
    /// Handle access fault (update flags)
    fn handle_access_fault(&mut self, ctx: &mut FaultContext) -> FaultResult {
        let mut pte = ctx.get_pte();
        
        // Set access flag
        pte |= pte_flags::AF;
        
        // Set dirty flag if write
        if ctx.is_write() {
            pte |= pte_flags::DIRTY;
        }
        
        ctx.set_pte(pte);
        
        FaultResult::Success
    }
    
    /// Handle swap-in
    fn handle_swapin(&mut self, ctx: &mut FaultContext) -> FaultResult {
        self.swapin_count.fetch_add(1, Ordering::AcqRel);
        
        log_debug!("Swap-in fault at {:#x}", ctx.address);
        
        // Step 1: Get swap entry from PTE
        let swap_entry = self.get_swap_entry(ctx);
        
        // Step 2: Allocate new page
        let page = self.alloc_zeroed_page();
        if page == 0 {
            return FaultResult::Oom;
        }
        
        // Step 3: Read from swap
        self.swap_read(swap_entry, page);
        
        // Step 4: Update PTE
        let pte = page | pte_flags::VALID | pte_flags::USER | pte_flags::AF;
        ctx.set_pte(pte);
        
        // Step 5: Free swap entry
        self.free_swap_entry(swap_entry);
        
        FaultResult::Success
    }
    
    /// Check if mapping is shared (VM_SHARED flag)
    fn is_shared_mapping(&self, ctx: &FaultContext) -> bool {
        (ctx.vma_flags & 0x10) != 0
    }
    
    /// Allocate zeroed page from page allocator
    fn alloc_zeroed_page(&self) -> u64 {
        let page = super::alloc_page();
        if page.is_null() {
            return 0;
        }
        // SAFETY: page is freshly allocated, valid for writing
        unsafe {
            let vaddr = (*page).phys_addr + 0xFFFF_0000_0000_0000;
            core::ptr::write_bytes(vaddr as *mut u8, 0u8, 4096);
            (*page).phys_addr
        }
    }
    
    /// Get swap entry from PTE
    fn get_swap_entry(&self, ctx: &FaultContext) -> u64 {
        // Swap entry stored in PTE with high bits masked
        ctx.get_pte() & 0x00FF_FFFF_FFFF_F000
    }
    
    /// Read page from swap
    fn swap_read(&self, _entry: u64, _page: u64) {
        // TODO: Actual swap read
    }
    
    /// Free swap entry
    fn free_swap_entry(&self, _entry: u64) {
        // TODO: Actual swap free
    }
    
    /// Get fault count
    pub fn get_fault_count(&self) -> u64 {
        self.fault_count.load(Ordering::Acquire)
    }
    
    /// Get success rate
    pub fn get_success_rate(&self) -> u32 {
        let success = self.success_count.load(Ordering::Acquire);
        let total = self.fault_count.load(Ordering::Acquire);
        
        if total == 0 {
            return 0;
        }
        
        ((success * 100) / total) as u32
    }
    
    /// Print statistics
    pub fn print_stats(&self) {
        log_info!("Page Fault Statistics:");
        log_info!("  Total faults: {}", self.fault_count.load(Ordering::Acquire));
        log_info!("  User faults: {}", self.user_faults.load(Ordering::Acquire));
        log_info!("  Kernel faults: {}", self.kernel_faults.load(Ordering::Acquire));
        log_info!("  Write faults: {}", self.write_faults.load(Ordering::Acquire));
        log_info!("  Read faults: {}", self.read_faults.load(Ordering::Acquire));
        log_info!("  Success rate: {}%", self.get_success_rate());
        
        let (cow_faults, cow_copies, cow_shared, cow_failures) = self.cow_handler.get_stats();
        log_info!("  COW faults: {}", cow_faults);
        log_info!("  COW copies: {}", cow_copies);
        log_info!("  COW shared: {}", cow_shared);
        log_info!("  COW failures: {}", cow_failures);
        log_info!("  Swap-ins: {}", self.swapin_count.load(Ordering::Acquire));
    }
}

/// Global page fault handler
static PAGE_FAULT_HANDLER: core::sync::OnceLock<PageFaultHandler> = core::sync::OnceLock::new();

/// Get page fault handler
pub fn page_fault_handler() -> &'static PageFaultHandler {
    PAGE_FAULT_HANDLER.get_or_init(PageFaultHandler::new)
}

/// Initialize page fault handler
pub fn init_page_fault() {
    let handler = get_page_fault_handler();
    handler.init();
}

/// Handle user space page fault
pub fn do_user_page_fault(address: u64, error_code: u32) -> FaultResult {
    let mut ctx = FaultContext::new(address, error_code);
    ctx.flags |= fault_flags::USER;
    
    get_page_fault_handler().handle_fault(&mut ctx)
}

/// Handle kernel space page fault
pub fn do_kernel_page_fault(address: u64, error_code: u32) -> FaultResult {
    let mut ctx = FaultContext::new(address, error_code);
    
    get_page_fault_handler().handle_fault(&mut ctx)
}

/// ARM64 specific page fault handler
/// Called from exception vector.
/// @param far: Fault Address Register
/// @param esr: Exception Syndrome Register
pub fn arm64_handle_page_fault(far: u64, esr: u32) {
    // Extract fault type from ESR
    let iss = esr & 0x1FFFFFF;
    let is_write = (iss >> 6) & 1 == 1;
    let is_instruction = (esr >> 26) == 0x22;  /* Instruction Abort */
    let is_user = (esr >> 4) & 1 == 0;  /* From EL0 */
    
    // Build error code
    let mut error_code = 0u32;
    if is_write {
        error_code |= fault_flags::WRITE;
    }
    if is_user {
        error_code |= fault_flags::USER;
    }
    if is_instruction {
        error_code |= fault_flags::INSTRUCTION;
    }
    
    // Handle the fault
    let result = if is_user {
        do_user_page_fault(far, error_code)
    } else {
        do_kernel_page_fault(far, error_code)
    };
    
    // Handle result
    match result {
        FaultResult::Success => {
            // Fault handled, return to user
        }
        FaultResult::Segfault => {
            log_warn!("Segmentation fault at {:#x}", far);
            // TODO: Send SIGSEGV to current process
        }
        FaultResult::WriteProtect => {
            log_warn!("Write protection fault at {:#x}", far);
            // TODO: Send SIGBUS to current process
        }
        FaultResult::Oom => {
            log_warn!("Out of memory at {:#x}", far);
            // TODO: Trigger OOM killer
        }
        _ => {
            log_warn!("Unhandled fault at {:#x}: {:?}", far, result);
        }
    }
}

/// VMA info structure for page fault lookup
#[repr(C)]
pub struct VmaInfo {
    /// Start virtual address
    pub vm_start: u64,
    /// End virtual address
    pub vm_end: u64,
    /// VMA flags (VM_READ|VM_WRITE|VM_EXEC|VM_SHARED)
    pub vm_flags: u32,
    /// VMA private data
    pub vm_private: u64,
}

/// Get current process mm_struct pointer
/// Returns null if no current process (early boot)
fn current_mm() -> *mut u8 {
    // TODO: Get from current task struct when scheduler is available
    // For now, return null to use default fallback
    core::ptr::null_mut()
}

/// Find VMA containing address in mm_struct
fn find_vma_in_mm(_mm: *mut u8, _addr: u64) -> *mut VmaInfo {
    // TODO: Walk VMA red-black tree in mm_struct
    // Will be implemented with process management integration
    core::ptr::null_mut()
}

/// Get current PGD (page global directory) physical address
fn current_pgd() -> u64 {
    // TODO: Get from current task's mm->pgd
    0
}

/// Walk page table to find PTE for a virtual address
/// @param pgd: Physical address of PGD
/// @param vaddr: Virtual address to look up
/// @return PTE value (0 if not found)
fn walk_page_table(pgd: u64, vaddr: u64) -> u64 {
    if pgd == 0 {
        return 0;
    }

    let page_offset: u64 = 0xFFFF_0000_0000_0000;
    let mut table_vaddr = (pgd + page_offset) as *const u64;
    let pte_shift: u64 = 12;
    let bits_per_level: u64 = 9;
    let entries_per_level: u64 = 512;

    for level in (1..4u64).rev() {
        let shift = pte_shift + bits_per_level * level;
        let idx = ((vaddr >> shift) & (entries_per_level - 1)) as usize;

        // SAFETY: table_vaddr from PGD/intermediate table, idx within bounds
        let entry = unsafe { *table_vaddr.add(idx) };
        if entry == 0 {
            return 0;
        }

        let next_phys = entry & 0x000F_FFFF_FFFF_F000;
        table_vaddr = (next_phys + page_offset) as *const u64;
    }

    let idx = ((vaddr >> pte_shift) & (entries_per_level - 1)) as usize;
    // SAFETY: final PTE table, idx within bounds
    unsafe { *table_vaddr.add(idx) }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_fault_flags() {
        assert_eq!(fault_flags::WRITE, 1);
        assert_eq!(fault_flags::USER, 2);
        assert_eq!(fault_flags::INSTRUCTION, 4);
    }
    
    #[test]
    fn test_pte_flags() {
        assert_eq!(pte_flags::VALID, 1);
        assert_eq!(pte_flags::COW, 1 << 56);
        assert_eq!(pte_flags::SWAP, 1 << 57);
    }
    
    #[test]
    fn test_fault_context_new() {
        let ctx = FaultContext::new(0x1000, fault_flags::WRITE);
        assert_eq!(ctx.address, 0x1000);
        assert!(ctx.is_write());
        assert!(!ctx.is_user());
    }
    
    #[test]
    fn test_cow_handler_new() {
        let handler = CowHandler::new();
        let (faults, copies, shared, failures) = handler.get_stats();
        assert_eq!(faults, 0);
        assert_eq!(copies, 0);
        assert_eq!(shared, 0);
        assert_eq!(failures, 0);
    }
    
    #[test]
    fn test_page_fault_handler_new() {
        let handler = PageFaultHandler::new();
        assert_eq!(handler.get_fault_count(), 0);
        assert_eq!(handler.get_success_rate(), 0);
    }
    
    #[test]
    fn test_fault_result_values() {
        assert_eq!(FaultResult::Success as i32, 0);
        assert_eq!(FaultResult::Segfault as i32, 3);
        assert_eq!(FaultResult::Oom as i32, 5);
    }

    #[test]
    fn test_vma_info() {
        let vma = VmaInfo {
            vm_start: 0x1000,
            vm_end: 0x2000,
            vm_flags: 0x7,
            vm_private: 0,
        };
        assert_eq!(vma.vm_start, 0x1000);
        assert_eq!(vma.vm_end, 0x2000);
    }
}
