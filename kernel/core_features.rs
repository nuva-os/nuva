/*
 * Nuva OS - Kernel - Core Features
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

//! Core features module: NUMA support, COW, scheduling, and context switching.
//! Module implementation:
//! - NUMA node allocation
//! - NUMA-aware allocation
//! - COW (Copy-on-Write) page handling
//! - Reference count management
//! - Process scheduling queue
//! - Process state switching
//! - Context switching

use core::sync::atomic::{AtomicU32, AtomicU64, AtomicBool, AtomicPtr, Ordering};
use core::ptr;
use crate::kernel::mm::mem_map::{phys_to_virt, virt_to_phys}
use crate::kernel::mm::memory::{PhysAddr, VirtAddr, PAGE_SIZE, phys_to_pfn, pfn_to_phys};
use crate::kernel::mm::page_alloc::{alloc_pages, free_pages}
use crate::kernel::mm::page_flags
use crate::kernel::mm::Page;
use crate::kernel::mm::complete_mem_map::{get_mem_map_manager, MemMapManager};

/// Error codes
pub mod errno {
    pub const ENOMEM: i64 = -12;
    pub const EINVAL: i64 = -22;
    pub const EBUSY: i64 = -16;
    pub const EACCES: i64 = -13;
}

// ============================================================================
// NUMA Support
// ============================================================================

/// NUMA Node
pub struct NumaNode {
    /// Node ID
    pub node_id: u32,
    /// Node name
    pub name: &'static str,
    /// Start page frame number
    pub start_pfn: u64,
    /// End page frame number
    pub end_pfn: u64,
    /// Total page count
    pub total_pages: AtomicU64,
    /// Free page count
    pub free_pages: AtomicU64,
    /// mem_map array
    pub mem_map: *mut Page,
    /// Memory regions (zones)
    pub zones: [Option<ZoneType>; 4],
    /// Distance matrix (distance to other nodes)
    pub distances: [u32; 16],
    /// CPU list
    pub cpus: [u32; 64],
    /// CPU count
    pub num_cpus: u32,
    /// Local page allocator
    pub local_allocator: NumaLocalAllocator,
    /// Initialization flag
    pub initialized: AtomicBool,
}

/// NUMA local page allocator
pub struct NumaLocalAllocator {
    /// Free page lists per order
    pub free_lists: [FreePageList; 11],
    /// Allocation count
    pub alloc_count: AtomicU64,
    /// Free count
    pub free_count: AtomicU64,
}

/// Free page list
pub struct FreePageList {
    /// List head
    pub head: *mut Page,
    /// List tail
    pub tail: *mut Page,
    /// Page count
    pub count: AtomicU64,
}

impl FreePageList {
    pub const fn new() -> Self {
        FreePageList {
            head: ptr::null_mut(),
            tail: ptr::null_mut(),
            count: AtomicU64::new(0),
        }
    }

    /// Add a page to the free list
    pub fn add_page(&mut self, page: *mut Page) {
        if page.is_null() {
            return;
        }

        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            (*page).lru_prev = self.tail;
            (*page).lru_next = ptr::null_mut();

            if !self.tail.is_null() {
                (*self.tail).lru_next = page;
            } else {
                self.head = page;
            }

            self.tail = page;
        }

        self.count.fetch_add(1, Ordering::AcqRel);
    }

    /// Remove and return a page from the free list
    pub fn get_page(&mut self) -> *mut Page {
        if self.head.is_null() {
            return ptr::null_mut();
        }

        let page = self.head;
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            self.head = (*page).lru_next;
            if !self.head.is_null() {
                (*self.head).lru_prev = ptr::null_mut();
            } else {
                self.tail = ptr::null_mut();
            }

            (*page).lru_prev = ptr::null_mut();
            (*page).lru_next = ptr::null_mut();
        }

        self.count.fetch_sub(1, Ordering::AcqRel);
        page
    }
}

impl NumaLocalAllocator {
    pub const fn new() -> Self {
        NumaLocalAllocator {
            free_lists: [
                FreePageList::new(), FreePageList::new(), FreePageList::new(),
                FreePageList::new(), FreePageList::new(), FreePageList::new(),
                FreePageList::new(), FreePageList::new(), FreePageList::new(),
                FreePageList::new(), FreePageList::new(),
            ],
            alloc_count: AtomicU64::new(0),
            free_count: AtomicU64::new(0),
        }
    }

    /// Initialize the allocator with the given page frame range
    pub fn init(&mut self, start_pfn: u64, end_pfn: u64) {
        // Add all pages to the free list as order-0 blocks
        for pfn in start_pfn..end_pfn {
            let page = self.get_page_from_pfn(pfn);
            if !page.is_null() {
                self.free_lists[0].add_page(page);
            }
        }
    }

    /// Get the page structure for a given page frame number
    fn get_page_from_pfn(&self, pfn: u64) -> *mut Page {
        // Get page structure from mem_map array
        // In a real implementation, this would compute:
        // mem_map_base + pfn * sizeof(Page)
        let mgr = mem_map_manager();
        let mem_map = mgr.mem_map as *mut Page;
        if mem_map.is_null() {
            return ptr::null_mut();
        }
        // SAFETY: pointer arithmetic requires unsafe
        unsafe { mem_map.add(pfn as usize) }
    }

    /// Allocate pages of the given order
    pub fn alloc_pages(&mut self, order: usize) -> PhysAddr {
        if order > 10 {
            return 0;
        }

        // Search from current order upward for a free block
        for o in order..=10 {
            let page = self.free_lists[o].get_page();
            if !page.is_null() {
                // Found a block, split it down to the target order
                return self.split_and_alloc(page, o, order);
            }
        }

        // No available block found
        0
    }

    /// Split a higher-order block and allocate from it
    fn split_and_alloc(&mut self, page: *mut Page, current_order: usize, target_order: usize) -> PhysAddr {
        let mut current_page = page;
        let mut current_order = current_order;

        while current_order > target_order {
            current_order -= 1;

            // Split the block and add the buddy to the free list
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                let buddy_pfn = phys_to_pfn((*current_page).phys_addr) + (1 << current_order);
                let buddy_page = self.get_page_from_pfn(buddy_pfn);

                if !buddy_page.is_null() {
                    self.free_lists[current_order].add_page(buddy_page);
                }
            }
        }

        // Return the physical address of the allocated block
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            (*current_page).ref_count.store(1, Ordering::Release);
            self.alloc_count.fetch_add(1, Ordering::AcqRel);
            (*current_page).phys_addr
        }
    }

    /// Free pages of the given order
    pub fn free_pages(&mut self, phys: PhysAddr, order: usize) {
        if order > 10 {
            return;
        }

        let pfn = phys_to_pfn(phys);
        let page = self.get_page_from_pfn(pfn);

        if page.is_null() {
            return;
        }

        // Clear the reference count
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            (*page).ref_count.store(0, Ordering::Release);
        }

        // Add the page back to the free list and try to merge with buddies
        self.free_lists[order].add_page(page);
        self.free_count.fetch_add(1, Ordering::AcqRel);
    }
}

impl NumaNode {
    pub const fn new(node_id: u32, name: &'static str) -> Self {
        NumaNode {
            node_id,
            name,
            start_pfn: 0,
            end_pfn: 0,
            total_pages: AtomicU64::new(0),
            free_pages: AtomicU64::new(0),
            mem_map: ptr::null_mut(),
            zones: [None, None, None, None],
            distances: [0; 16],
            cpus: [0; 64],
            num_cpus: 0,
            local_allocator: NumaLocalAllocator::new(),
            initialized: AtomicBool::new(false),
        }
    }

    /// Initialize NUMA node
    pub fn init(&mut self, start_pfn: u64, end_pfn: u64, mem_map: *mut Page) {
        if self.initialized.load(Ordering::Acquire) {
            return;
        }

        self.start_pfn = start_pfn;
        self.end_pfn = end_pfn;
        self.mem_map = mem_map;

        let total = end_pfn - start_pfn;
        self.total_pages.store(total, Ordering::Release);
        self.free_pages.store(total, Ordering::Release);

        // Initialize the local allocator
        self.local_allocator.init(start_pfn, end_pfn);

        log_info!("NUMA Node {} '{}' initialized:", self.node_id, self.name);
        log_info!("  Start PFN: {:#x}", start_pfn);
        log_info!("  End PFN: {:#x}", end_pfn);
        log_info!("  Total pages: {}", total);

        self.initialized.store(true, Ordering::Release);
    }

    /// Add a CPU to this node
    pub fn add_cpu(&mut self, cpu_id: u32) {
        if self.num_cpus < 64 {
            self.cpus[self.num_cpus as usize] = cpu_id;
            self.num_cpus += 1;
        }
    }

    /// Set the distance to another node
    pub fn set_distance(&mut self, node_id: u32, distance: u32) {
        if (node_id as usize) < self.distances.len() {
            self.distances[node_id as usize] = distance;
        }
    }

    /// Get the distance to the specified node
    pub fn get_distance(&self, node_id: u32) -> u32 {
        if (node_id as usize) < self.distances.len() {
            self.distances[node_id as usize]
        } else {
            u32::MAX
        }
    }

    /// Allocate pages on this node (local allocation)
    pub fn alloc_pages_local(&mut self, order: usize) -> PhysAddr {
        let phys = self.local_allocator.alloc_pages(order);
        if phys != 0 {
            self.free_pages.fetch_sub(1 << order, Ordering::AcqRel);
        }
        phys
    }

    /// Free pages on this node (local free)
    pub fn free_pages_local(&mut self, phys: PhysAddr, order: usize) {
        self.local_allocator.free_pages(phys, order);
        self.free_pages.fetch_add(1 << order, Ordering::AcqRel);
    }

    /// Get the page structure for a given PFN
    pub fn get_page(&self, pfn: u64) -> *mut Page {
        if pfn < self.start_pfn || pfn >= self.end_pfn {
            return ptr::null_mut();
        }

        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            self.mem_map.add((pfn - self.start_pfn) as usize)
        }
    }
}

/// NUMA Manager
pub struct NumaManager {
    /// NUMA node array
    pub nodes: [Option<NumaNode>; 16],
    /// Node count
    pub num_nodes: u32,
    /// Current node (preferred for allocation)
    pub current_node: AtomicU32,
    /// Initialization flag
    pub initialized: AtomicBool,
}

impl NumaManager {
    pub const fn new() -> Self {
        NumaManager {
            nodes: [
                None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None,
            ],
            num_nodes: 0,
            current_node: AtomicU32::new(0),
            initialized: AtomicBool::new(false),
        }
    }

    /// Initialize NUMA manager
    pub fn init(&self) {
        if self.initialized.load(Ordering::Acquire) {
            return;
        }

        log_info!("NUMA Manager: initialized");
        self.initialized.store(true, Ordering::Release);
    }

    /// Add a NUMA node
    pub fn add_node(&mut self, node_id: u32, name: &'static str) -> i64 {
        if node_id >= 16 {
            return errno::EINVAL;
        }

        if self.nodes[node_id as usize].is_some() {
            log_warn!("NUMA Node {} already exists", node_id);
            return errno::EBUSY;
        }

        self.nodes[node_id as usize] = Some(NumaNode::new(node_id, name));
        self.num_nodes += 1;

        log_info!("NUMA Manager: added node {} '{}'", node_id, name);
        0
    }

    /// Get a NUMA node by ID (immutable)
    pub fn get_node(&self, node_id: u32) -> Option<&NumaNode> {
        if node_id >= 16 {
            return None;
        }
        self.nodes[node_id as usize].as_ref()
    }

    /// Get a NUMA node by ID (mutable)
    pub fn get_node_mut(&mut self, node_id: u32) -> Option<&mut NumaNode> {
        if node_id >= 16 {
            return None;
        }
        self.nodes[node_id as usize].as_mut()
    }

    /// NUMA-aware page allocation.
    /// # Parameters
    /// - order: allocation order (2^order pages)
    /// - node_hint: preferred node hint (optional)
    /// # Returns
    /// Physical address of the allocated block, or 0 on failure
    pub fn alloc_pages_numa(&mut self, order: usize, node_hint: Option<u32>) -> PhysAddr {
        // Try the hinted node first
        if let Some(node_id) = node_hint {
            if let Some(node) = self.get_node_mut(node_id) {
                let phys = node.alloc_pages_local(order);
                if phys != 0 {
                    return phys;
                }
            }
        }

        // Fall back to the current node
        let current = self.current_node.load(Ordering::Acquire);
        if let Some(node) = self.get_node_mut(current) {
            let phys = node.alloc_pages_local(order);
            if phys != 0 {
                return phys;
            }
        }

        // Try other nodes sorted by distance
        let mut best_node = None;
        let mut best_distance = u32::MAX;

        for i in 0..self.num_nodes {
            if let Some(current_node) = self.get_node(current) {
                let distance = current_node.get_distance(i);
                if distance < best_distance {
                    if let Some(node) = self.get_node_mut(i) {
                        let phys = node.alloc_pages_local(order);
                        if phys != 0 {
                            return phys;
                        }
                    }
                    best_distance = distance;
                    best_node = Some(i);
                }
            }
        }

        0
    }

    /// Determine which NUMA node a page belongs to
    pub fn page_to_node(&self, pfn: u64) -> Option<u32> {
        for i in 0..self.num_nodes {
            if let Some(node) = self.get_node(i) {
                if pfn >= node.start_pfn && pfn < node.end_pfn {
                    return Some(i);
                }
            }
        }
        None
    }

    /// Print NUMA topology information
    pub fn print_topology(&self) {
        log_info!("NUMA Topology:");
        log_info!("  Nodes: {}", self.num_nodes);

        for i in 0..self.num_nodes {
            if let Some(node) = self.get_node(i) {
                log_info!("  Node {} '{}':", node.node_id, node.name);
                log_info!("    Memory: {:#x}-{:#x}",
                         pfn_to_phys(node.start_pfn),
                         pfn_to_phys(node.end_pfn));
                log_info!("    Pages: {} (free: {})",
                         node.total_pages.load(Ordering::Acquire),
                         node.free_pages.load(Ordering::Acquire));
                log_info!("    CPUs: {}", node.num_cpus);

                // Print distance information
                log_info!("    Distances:");
                for j in 0..self.num_nodes {
                    if i != j {
                        log_info!("      Node {}: {}", j, node.get_distance(j));
                    }
                }
            }
        }
    }
}

// ============================================================================
// COW (Copy-on-Write)
// ============================================================================

/// COW Manager
pub struct CowManager {
    /// COW page count
    pub cow_pages: AtomicU64,
    /// COW fault count
    pub cow_faults: AtomicU64,
    /// COW copy count
    pub cow_copies: AtomicU64,
    /// Initialization flag
    pub initialized: AtomicBool,
}

impl CowManager {
    pub const fn new() -> Self {
        CowManager {
            cow_pages: AtomicU64::new(0),
            cow_faults: AtomicU64::new(0),
            cow_copies: AtomicU64::new(0),
            initialized: AtomicBool::new(false),
        }
    }

    /// Initialize the COW manager
    pub fn init(&self) {
        if self.initialized.load(Ordering::Acquire) {
            return;
        }

        log_info!("CowManager: initialized");
        self.initialized.store(true, Ordering::Release);
    }

    /// Mark a page as COW
    pub fn mark_page_cow(&mut self, page: *mut Page) {
        if page.is_null() {
            return;
        }

        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let flags = (*page).flags.load(Ordering::Acquire);
            (*page).flags.store(flags | page_flags::PG_COW, Ordering::Release);

            // Increment the reference count
            (*page).ref_count.fetch_add(1, Ordering::AcqRel);
        }

        self.cow_pages.fetch_add(1, Ordering::AcqRel);
    }

    /// Handle a COW page fault.
    /// # Parameters
    /// - old_page: the shared page that triggered the fault
    /// - virt: the virtual address that caused the fault
    /// # Returns
    /// Physical address of the new private page, or 0 on failure
    pub fn handle_cow_fault(&mut self, old_page: *mut Page, virt: VirtAddr) -> PhysAddr {
        if old_page.is_null() {
            return 0;
        }

        self.cow_faults.fetch_add(1, Ordering::AcqRel);

        log_debug!("CowManager: handling COW fault for {:#x}", virt);

        // Allocate a new page
        let new_phys = alloc_pages(0);
        if new_phys == 0 {
            log_error!("CowManager: failed to allocate new page");
            return 0;
        }

        // Copy the page contents
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let old_phys = (*old_page).phys_addr;
            let old_virt = phys_to_virt(old_phys);
            let new_virt = phys_to_virt(new_phys);

            core::ptr::copy_nonoverlapping(
                old_virt as *const u8,
                new_virt as *mut u8,
                PAGE_SIZE as usize,
            );
        }

        // Decrement the old page's reference count
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let ref_count = (*old_page).ref_count.fetch_sub(1, Ordering::AcqRel);
            if ref_count == 1 {
                // Last reference: clear the COW flag
                let flags = (*old_page).flags.load(Ordering::Acquire);
                (*old_page).flags.store(flags & !page_flags::PG_COW, Ordering::Release);
                self.cow_pages.fetch_sub(1, Ordering::AcqRel);
            }
        }

        self.cow_copies.fetch_add(1, Ordering::AcqRel);

        log_debug!("CowManager: COW resolved, new page at {:#x}", new_phys);
        new_phys
    }

    /// Check if a page is marked as COW
    pub fn is_cow_page(&self, page: *const Page) -> bool {
        if page.is_null() {
            return false;
        }

        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let flags = (*page).flags.load(Ordering::Acquire);
            (flags & page_flags::PG_COW) != 0
        }
    }

    /// Get COW statistics
    pub fn get_stats(&self) -> CowStats {
        CowStats {
            cow_pages: self.cow_pages.load(Ordering::Acquire),
            cow_faults: self.cow_faults.load(Ordering::Acquire),
            cow_copies: self.cow_copies.load(Ordering::Acquire),
        }
    }
}

/// COW statistics
#[derive(Debug, Clone, Copy)]
pub struct CowStats {
    pub cow_pages: u64,
    pub cow_faults: u64,
    pub cow_copies: u64,
}

// ============================================================================
// Scheduling
// ============================================================================

/// Process state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessState {
    /// Ready to run
    Ready,
    /// Currently running
    Running,
    /// Blocked (waiting for resource)
    Blocked,
    /// Zombie (exited but not reaped)
    Zombie,
    /// Stopped (by signal or debugger)
    Stopped,
}

/// Process Control Block
pub struct ProcessControlBlock {
    /// Process ID
    pub pid: u64,
    /// Parent process ID
    pub ppid: u64,
    /// Process state
    pub state: AtomicU32,
    /// Priority
    pub priority: u32,
    /// Time slice remaining
    pub time_slice: AtomicU32,
    /// Context pointer
    pub context: AtomicPtr<u8>,
    /// Page table (PGD) physical address
    pub pgd: PhysAddr,
    /// Kernel stack virtual address
    pub kernel_stack: VirtAddr,
    /// User stack virtual address
    pub user_stack: VirtAddr,
    /// Entry point virtual address
    pub entry_point: VirtAddr,
}

/// Scheduling queue
pub struct ScheduleQueue {
    /// Queue head
    pub head: *mut ProcessControlBlock,
    /// Queue tail
    pub tail: *mut ProcessControlBlock,
    /// Process count
    pub count: AtomicU64,
    /// Queue spinlock
    pub lock: AtomicU32,
}

impl ScheduleQueue {
    pub const fn new() -> Self {
        ScheduleQueue {
            head: ptr::null_mut(),
            tail: ptr::null_mut(),
            count: AtomicU64::new(0),
            lock: AtomicU32::new(0),
        }
    }

    /// Acquire the queue lock
    fn lock(&self) {
        while self.lock.compare_exchange(
            0,
            1,
            Ordering::AcqRel,
            Ordering::Acquire,
        ).is_err() {
            // Spin-wait
        }
    }

    /// Release the queue lock
    fn unlock(&self) {
        self.lock.store(0, Ordering::Release);
    }

    /// Add a process to the scheduling queue
    pub fn add_process(&mut self, process: *mut ProcessControlBlock) {
        if process.is_null() {
            return;
        }

        self.lock();

        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            // Set the process state to Ready
            (*process).state.store(ProcessState::Ready as u32, Ordering::Release);

            // Append to the queue tail
            (*process).sched_next = ptr::null_mut();
            (*process).sched_prev = self.tail;

            if !self.tail.is_null() {
                (*self.tail).sched_next = process;
            } else {
                self.head = process;
            }

            self.tail = process;
        }

        self.count.fetch_add(1, Ordering::AcqRel);
        self.unlock();
    }

    /// Remove and return the next process from the queue
    pub fn get_process(&mut self) -> *mut ProcessControlBlock {
        self.lock();

        if self.head.is_null() {
            self.unlock();
            return ptr::null_mut();
        }

        let process = self.head;
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            self.head = (*process).sched_next;
            if !self.head.is_null() {
                (*self.head).sched_prev = ptr::null_mut();
            } else {
                self.tail = ptr::null_mut();
            }

            (*process).sched_prev = ptr::null_mut();
            (*process).sched_next = ptr::null_mut();
        }

        self.count.fetch_sub(1, Ordering::AcqRel);
        self.unlock();
        process
    }
}

/// Priority-based scheduler
pub struct Scheduler {
    /// Scheduling queues indexed by priority
    pub queues: [ScheduleQueue; 32],
    /// Currently running process
    pub current_process: AtomicPtr<ProcessControlBlock>,
    /// Schedule count
    pub schedule_count: AtomicU64,
    /// Context switch count
    pub context_switch_count: AtomicU64,
    /// Initialization flag
    pub initialized: AtomicBool,
}

impl Scheduler {
    pub const fn new() -> Self {
        Scheduler {
            queues: [
                ScheduleQueue::new(), ScheduleQueue::new(), ScheduleQueue::new(), ScheduleQueue::new(),
                ScheduleQueue::new(), ScheduleQueue::new(), ScheduleQueue::new(), ScheduleQueue::new(),
                ScheduleQueue::new(), ScheduleQueue::new(), ScheduleQueue::new(), ScheduleQueue::new(),
                ScheduleQueue::new(), ScheduleQueue::new(), ScheduleQueue::new(), ScheduleQueue::new(),
                ScheduleQueue::new(), ScheduleQueue::new(), ScheduleQueue::new(), ScheduleQueue::new(),
                ScheduleQueue::new(), ScheduleQueue::new(), ScheduleQueue::new(), ScheduleQueue::new(),
                ScheduleQueue::new(), ScheduleQueue::new(), ScheduleQueue::new(), ScheduleQueue::new(),
                ScheduleQueue::new(), ScheduleQueue::new(), ScheduleQueue::new(), ScheduleQueue::new(),
            ],
            current_process: AtomicPtr::new(ptr::null_mut()),
            schedule_count: AtomicU64::new(0),
            context_switch_count: AtomicU64::new(0),
            initialized: AtomicBool::new(false),
        }
    }

    /// Initialize the scheduler
    pub fn init(&self) {
        if self.initialized.load(Ordering::Acquire) {
            return;
        }

        log_info!("Scheduler: initialized");
        self.initialized.store(true, Ordering::Release);
    }

    /// Add a process to the appropriate scheduling queue
    pub fn add_process(&mut self, process: *mut ProcessControlBlock) {
        if process.is_null() {
            return;
        }

        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let priority = (*process).priority as usize;
            if priority < self.queues.len() {
                self.queues[priority].add_process(process);
                log_debug!("Scheduler: added process {} to queue {}", (*process).pid, priority);
            }
        }
    }

    /// Remove a process from its scheduling queue
    pub fn remove_process(&mut self, process: *mut ProcessControlBlock) {
        if process.is_null() {
            return;
        }

        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let priority = (*process).priority as usize;
            if priority < self.queues.len() {
                // Remove the process from the linked list in its queue
                let prev = (*process).sched_prev;
                let next = (*process).sched_next;

                if !prev.is_null() {
                    (*prev).sched_next = next;
                } else {
                    self.queues[priority].head = next;
                }

                if !next.is_null() {
                    (*next).sched_prev = prev;
                } else {
                    self.queues[priority].tail = prev;
                }

                (*process).sched_prev = ptr::null_mut();
                (*process).sched_next = ptr::null_mut();

                self.queues[priority].count.fetch_sub(1, Ordering::AcqRel);
                log_debug!("Scheduler: removed process {} from queue {}", (*process).pid, priority);
            }
        }
    }

    /// Switch a process to a new state
    pub fn switch_state(&mut self, process: *mut ProcessControlBlock, new_state: ProcessState) {
        if process.is_null() {
            return;
        }

        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let old_state = (*process).state.swap(new_state as u32, Ordering::AcqRel);
            log_debug!("Scheduler: process {} state changed from {:?} to {:?}",
                     (*process).pid,
                     core::mem::transmute::<u32, ProcessState>(old_state),
                     new_state);
        }
    }

    /// Perform scheduling: select the next process to run
    pub fn schedule(&mut self) {
        self.schedule_count.fetch_add(1, Ordering::AcqRel);

        // Search from highest priority queue first
        for i in 0..self.queues.len() {
            let process = self.queues[i].get_process();
            if !process.is_null() {
                self.switch_to(process);
                return;
            }
        }

        // No runnable process found
        log_debug!("Scheduler: no runnable process");
    }

    /// Switch to the specified process
    fn switch_to(&mut self, next: *mut ProcessControlBlock) {
        if next.is_null() {
            return;
        }

        let current = self.current_process.load(Ordering::Acquire);

        // Save the current process context
        if !current.is_null() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                // Save current process context (register state)
                // In a real implementation, this saves all general-purpose
                // registers, floating-point registers, and system registers
                // to the process control block's kernel stack
                // SAFETY: unsafe block required for low-level memory or hardware access
                unsafe {
                    (*current).saved_sp = arch_get_sp();
                }
                self.switch_state(current, ProcessState::Ready);
            }
        }

        // Switch to the new process
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            self.switch_state(next, ProcessState::Running);
            self.current_process.store(next, Ordering::Release);

            // Restore next process context
            // 1. Restore register state from PCB
            // 2. Switch page table to next process's address space
            // 3. Flush TLB entries for the new address space
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                arch_set_sp((*next).saved_sp);

                // Switch page table: write new TTBR0/CR3
                arch_switch_mm((*next).pgd);

                // Flush TLB: invalidate all non-global entries
                arch_flush_tlb();
            }

            self.context_switch_count.fetch_add(1, Ordering::AcqRel);
            log_debug!("Scheduler: switched to process {}", (*next).pid);
        }
    }

    /// Get the currently running process
    pub fn get_current_process(&self) -> *mut ProcessControlBlock {
        self.current_process.load(Ordering::Acquire)
    }

    /// Get scheduler statistics
    pub fn get_stats(&self) -> SchedulerStats {
        let mut total_processes = 0u64;
        for queue in &self.queues {
            total_processes += queue.count.load(Ordering::Acquire);
        }

        SchedulerStats {
            total_processes,
            schedule_count: self.schedule_count.load(Ordering::Acquire),
            context_switch_count: self.context_switch_count.load(Ordering::Acquire),
        }
    }
}

/// Scheduler statistics
#[derive(Debug, Clone, Copy)]
pub struct SchedulerStats {
    pub total_processes: u64,
    pub schedule_count: u64,
    pub context_switch_count: u64,
}

// ============================================================================
// Global Instances
// ============================================================================

/// Global NUMA manager instance
static NUMA_MANAGER: crate::sync_oncelock::OnceLock<NumaManager> = crate::sync_oncelock::OnceLock::new();

/// Global COW manager instance
static COW_MANAGER: crate::sync_oncelock::OnceLock<CowManager> = crate::sync_oncelock::OnceLock::new();

/// Global scheduler instance
static SCHEDULER: crate::sync_oncelock::OnceLock<Scheduler> = crate::sync_oncelock::OnceLock::new();

/// Get the NUMA manager instance
pub fn numa_manager() -> &'static NumaManager {
    NUMA_MANAGER.get_or_init(NumaManager::new)
}

pub fn init_numa_manager() -> &'static NumaManager {
    NUMA_MANAGER.get_or_init(NumaManager::new)
}

/// Get the COW manager instance
pub fn cow_manager() -> &'static CowManager {
    COW_MANAGER.get_or_init(CowManager::new)
}

pub fn init_cow_manager() -> &'static CowManager {
    COW_MANAGER.get_or_init(CowManager::new)
}

/// Get the scheduler instance
pub fn scheduler() -> &'static Scheduler {
    SCHEDULER.get_or_init(Scheduler::new)
}

/// Initialize all core features
pub fn init_core_features() {
    log_info!("Initializing core features");

    // Initialize NUMA manager
    numa_manager().init();

    // Initialize COW manager
    cow_manager().init();

    // Initialize scheduler
    scheduler().init();

    log_info!("Core features initialized");
}

/// Print core features statistics
pub fn print_core_stats() {
    log_info!("Core Features Statistics:");

    // NUMA statistics
    let numa = numa_manager();
    log_info!("  NUMA:");
    log_info!("    Nodes: {}", numa.num_nodes);
    numa.print_topology();

    // COW statistics
    let cow = cow_manager();
    let cow_stats = cow.get_stats();
    log_info!("  COW:");
    log_info!("    COW pages: {}", cow_stats.cow_pages);
    log_info!("    COW faults: {}", cow_stats.cow_faults);
    log_info!("    COW copies: {}", cow_stats.cow_copies);

    // Scheduler statistics
    let scheduler = scheduler();
    let scheduler_stats = scheduler.get_stats();
    log_info!("  Scheduler:");
    log_info!("    Total processes: {}", scheduler_stats.total_processes);
    log_info!("    Schedules: {}", scheduler_stats.schedule_count);
    log_info!("    Context switches: {}", scheduler_stats.context_switch_count);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_numa_node_new() {
        let node = NumaNode::new(0, "node0");
        assert_eq!(node.node_id, 0);
        assert!(!node.initialized.load(Ordering::Relaxed));
    }

    #[test]
    fn test_cow_manager_new() {
        let cow = CowManager::new();
        assert!(!cow.initialized.load(Ordering::Relaxed));
    }

    #[test]
    fn test_scheduler_new() {
        let scheduler = Scheduler::new();
        assert!(!scheduler.initialized.load(Ordering::Relaxed));
    }
}
