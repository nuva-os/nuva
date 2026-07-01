/*
 * Nuva OS - Kernel - Core - Mempool
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
use crate::{pr_info};
/*
 * Nuva OS - Kernel - Memory Pool
 * 
 * Copyright (C) 2026 Nuva OS Team
 * 
 * Kernel memory pool for efficient allocation.
 */

use core::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, AtomicPtr, Ordering};

use crate::syslib::posix::errno::Errno;
use crate::kernel::error::Errno;
/// Memory Pool Flags
bitflags::bitflags! {
    #[repr(transparent)]
    pub struct MempoolFlags: u32 {
        /// GFP flags mask
        const GFP_MASK = 0xFFFF;
        /// Pre-allocate minimum
        const MIN_PREALLOC = 1 << 16;
        /// Grow on demand
        const GROW = 1 << 17;
        /// Shrink when idle
        const SHRINK = 1 << 18;
        /// Thread safe
        const THREAD_SAFE = 1 << 19;
        /// Debug mode
        const DEBUG = 1 << 20;
        /// Zero on alloc
        const ZERO = 1 << 21;
        /// Poison on free
        const POISON = 1 << 22;
    }
}

/// Memory Pool Element
pub struct MempoolElement {
    /// Data pointer
    pub data: *mut u8,
    /// Pool reference
    pub pool: *mut Mempool,
    /// In use
    pub in_use: AtomicBool,
    /// Next element
    pub next: *mut MempoolElement,
}

/// Memory Pool Operations
pub struct MempoolOps {
    /// Allocate function
    pub alloc: Option<unsafe extern "C" fn(usize, u32) -> *mut u8>,
    /// Free function
    pub free: Option<unsafe extern "C" fn(*mut u8)>,
}

/// Memory Pool
pub struct Mempool {
    /// Pool name
    pub name: [u8; 32],
    /// Element size
    pub element_size: usize,
    /// Minimum elements
    pub min_nr: u32,
    /// Maximum elements
    pub max_nr: u32,
    /// Current elements
    pub curr_nr: AtomicU32,
    /// Allocated elements
    pub allocated: AtomicU32,
    /// Flags
    pub flags: MempoolFlags,
    /// Operations
    pub ops: MempoolOps,
    /// Private data
    pub priv_data: *mut core::ffi::c_void,
    /// Elements
    pub elements: *mut MempoolElement,
    /// Free list
    pub free_list: AtomicPtr<MempoolElement>,
    /// Lock
    pub lock: AtomicU32,
    /// Statistics
    pub stats: MempoolStats,
}

/// Memory Pool Statistics
pub struct MempoolStats {
    pub alloc_count: AtomicU64,
    pub free_count: AtomicU64,
    pub grow_count: AtomicU64,
    pub shrink_count: AtomicU64,
    pub wait_count: AtomicU64,
    pub fail_count: AtomicU64,
}

impl MempoolStats {
    pub const fn new() -> Self {
        MempoolStats {
            alloc_count: AtomicU64::new(0),
            free_count: AtomicU64::new(0),
            grow_count: AtomicU64::new(0),
            shrink_count: AtomicU64::new(0),
            wait_count: AtomicU64::new(0),
            fail_count: AtomicU64::new(0),
        }
    }
}

impl Mempool {
    pub fn new(name: &[u8], element_size: usize, min_nr: u32, max_nr: u32) -> Self {
        let mut name_arr = [0u8; 32];
        let len = name.len().min(31);
        name_arr[..len].copy_from_slice(&name[..len]);
        
        Mempool {
            name: name_arr,
            element_size,
            min_nr,
            max_nr,
            curr_nr: AtomicU32::new(0),
            allocated: AtomicU32::new(0),
            flags: MempoolFlags::GROW | MempoolFlags::THREAD_SAFE,
            ops: MempoolOps {
                alloc: None,
                free: None,
            },
            priv_data: core::ptr::null_mut(),
            elements: core::ptr::null_mut(),
            free_list: AtomicPtr::new(core::ptr::null_mut()),
            lock: AtomicU32::new(0),
            stats: MempoolStats::new(),
        }
    }
    
    /// Lock
    fn lock(&self) {
        if self.flags.contains(MempoolFlags::THREAD_SAFE) {
            while self.lock.compare_exchange(0, 1, Ordering::AcqRel, Ordering::Acquire).is_err() {
                core::hint::spin_loop();
            }
        }
    }
    
    /// Unlock
    fn unlock(&self) {
        if self.flags.contains(MempoolFlags::THREAD_SAFE) {
            self.lock.store(0, Ordering::Release);
        }
    }
    
    /// Initialize pool
    pub fn init(&self) -> i32 {
        // Pre-allocate minimum elements
        for _ in 0..self.min_nr {
            if self.grow().is_err() {
                return Errno::Enomem.to_ret_i32(); // ENOMEM
            }
        }
        
        0
    }
    
    /// Grow pool
    fn grow(&mut self) -> Result<*mut MempoolElement, i32> {
        let curr = self.curr_nr.load(Ordering::Acquire);
        
        if curr >= self.max_nr {
            return Err(-12);
        }
        
        // Allocate element
        let data = if let Some(alloc) = self.ops.alloc {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe { alloc(self.element_size, 0) }
        } else {
            // Default allocation
            core::ptr::null_mut()
        };
        
        if data.is_null() {
            return Err(-12);
        }
        
        // Zero if requested
        if self.flags.contains(MempoolFlags::ZERO) {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                core::ptr::write_bytes(data, 0, self.element_size);
            }
        }
        
        // Create element
        let elem = MempoolElement {
            data,
            pool: self as *mut Mempool,
            in_use: AtomicBool::new(false),
            next: core::ptr::null_mut(),
        };
        
        // TODO: Allocate element structure
        
        self.curr_nr.fetch_add(1, Ordering::AcqRel);
        self.stats.grow_count.fetch_add(1, Ordering::AcqRel);
        
        // Return placeholder
        Err(-12)
    }
    
    /// Shrink pool
    fn shrink(&mut self) {
        let curr = self.curr_nr.load(Ordering::Acquire);
        
        if curr <= self.min_nr {
            return;
        }
        
        // Check if can shrink
        let free = self.free_list.load(Ordering::Acquire);
        if free.is_null() {
            return;
        }
        
        // Remove from free list
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            self.free_list.store((*free).next, Ordering::Release);
            
            // Free data
            if let Some(free_fn) = self.ops.free {
                free_fn((*free).data);
            }
        }
        
        self.curr_nr.fetch_sub(1, Ordering::AcqRel);
        self.stats.shrink_count.fetch_add(1, Ordering::AcqRel);
    }
    
    /// Allocate from pool
    pub fn alloc(&mut self, gfp_flags: u32) -> *mut u8 {
        self.lock();
        
        // Try to get from free list
        let mut elem = self.free_list.load(Ordering::Acquire);
        
        if elem.is_null() {
            // Try to grow
            if self.flags.contains(MempoolFlags::GROW) {
                if self.grow().is_ok() {
                    elem = self.free_list.load(Ordering::Acquire);
                }
            }
            
            if elem.is_null() {
                self.stats.fail_count.fetch_add(1, Ordering::AcqRel);
                self.unlock();
                return core::ptr::null_mut();
            }
        }
        
        // Remove from free list
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            self.free_list.store((*elem).next, Ordering::Release);
            (*elem).in_use.store(true, Ordering::Release);
            
            self.allocated.fetch_add(1, Ordering::AcqRel);
            self.stats.alloc_count.fetch_add(1, Ordering::AcqRel);
            
            let data = (*elem).data;
            
            // Zero if requested
            if self.flags.contains(MempoolFlags::ZERO) {
                core::ptr::write_bytes(data, 0, self.element_size);
            }
            
            self.unlock();
            data
        }
    }
    
    /// Free to pool
    pub fn free(&mut self, data: *mut u8) {
        if data.is_null() {
            return;
        }
        
        self.lock();
        
        // Find element
        let mut elem = self.elements;
        while !elem.is_null() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                if (*elem).data == data {
                    // Poison if requested
                    if self.flags.contains(MempoolFlags::POISON) {
                        core::ptr::write_bytes(data, 0xDE, self.element_size);
                    }
                    
                    // Add to free list
                    (*elem).in_use.store(false, Ordering::Release);
                    (*elem).next = self.free_list.load(Ordering::Acquire);
                    self.free_list.store(elem, Ordering::Release);
                    
                    self.allocated.fetch_sub(1, Ordering::AcqRel);
                    self.stats.free_count.fetch_add(1, Ordering::AcqRel);
                    
                    // Try to shrink
                    if self.flags.contains(MempoolFlags::SHRINK) {
                        self.shrink();
                    }
                    
                    self.unlock();
                    return;
                }
                elem = (*elem).next;
            }
        }
        
        // Not found, just free
        if let Some(free_fn) = self.ops.free {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe { free_fn(data); }
        }
        
        self.unlock();
    }
    
    /// Get element
    pub fn get(&mut self) -> *mut u8 {
        self.alloc(0)
    }
    
    /// Put element
    pub fn put(&mut self, data: *mut u8) {
        self.free(data);
    }
    
    /// Get allocated count
    pub fn allocated_count(&self) -> u32 {
        self.allocated.load(Ordering::Acquire)
    }
    
    /// Get available count
    pub fn available_count(&self) -> u32 {
        self.curr_nr.load(Ordering::Acquire) - self.allocated.load(Ordering::Acquire)
    }
}

/// Memory Pool Manager
pub struct MempoolManager {
    /// Pools
    pub pools: *mut Mempool,
    /// Pool count
    pub pool_count: AtomicU32,
    /// Statistics
    pub stats: MempoolMgrStats,
}

/// Memory Pool Manager Statistics
pub struct MempoolMgrStats {
    pub total_pools: AtomicU32,
    pub total_elements: AtomicU64,
    pub total_allocated: AtomicU64,
}

impl MempoolMgrStats {
    pub const fn new() -> Self {
        MempoolMgrStats {
            total_pools: AtomicU32::new(0),
            total_elements: AtomicU64::new(0),
            total_allocated: AtomicU64::new(0),
        }
    }
}

impl MempoolManager {
    pub const fn new() -> Self {
        MempoolManager {
            pools: core::ptr::null_mut(),
            pool_count: AtomicU32::new(0),
            stats: MempoolMgrStats::new(),
        }
    }
    
    /// Initialize
    pub fn init(&self) {
        // Create standard pools
        self.create_standard_pools();
        
        log_info!("Memory pool manager initialized");
    }
    
    /// Create standard pools
    fn create_standard_pools(&mut self) {
        // Small object pool (64 bytes)
        let small_pool = Mempool::new(b"small", 64, 256, 1024);
        // TODO: Register
        
        // Medium object pool (256 bytes)
        let medium_pool = Mempool::new(b"medium", 256, 128, 512);
        // TODO: Register
        
        // Large object pool (1024 bytes)
        let large_pool = Mempool::new(b"large", 1024, 64, 256);
        // TODO: Register
        
        // Page pool (4096 bytes)
        let page_pool = Mempool::new(b"page", 4096, 32, 128);
        // TODO: Register
        
        let _ = (small_pool, medium_pool, large_pool, page_pool);
    }
    
    /// Create pool
    pub fn create(&mut self, name: &[u8], element_size: usize, min_nr: u32, max_nr: u32) -> Result<*mut Mempool, i32> {
        let pool = Mempool::new(name, element_size, min_nr, max_nr);
        
        // TODO: Allocate and initialize pool
        
        self.pool_count.fetch_add(1, Ordering::AcqRel);
        self.stats.total_pools.fetch_add(1, Ordering::AcqRel);
        
        Err(-12)
    }
    
    /// Destroy pool
    pub fn destroy(&mut self, pool: *mut Mempool) {
        if pool.is_null() {
            return;
        }
        
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            // Free all elements
            let mut elem = (*pool).elements;
            while !elem.is_null() {
                if let Some(free_fn) = (*pool).ops.free {
                    free_fn((*elem).data);
                }
                elem = (*elem).next;
            }
        }
        
        self.pool_count.fetch_sub(1, Ordering::AcqRel);
    }
    
    /// Find pool by name
    pub fn find(&self, name: &[u8]) -> Option<*mut Mempool> {
        let mut pool = self.pools;
        
        while !pool.is_null() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                let pool_name = &(*pool).name;
                if pool_name[..name.len()] == *name {
                    return Some(pool);
                }
                pool = (*pool).elements as *mut Mempool;
            }
        }
        
        None
    }
}

/// Global mempool manager
static MEMPOOL_MANAGER: crate::sync_oncelock::OnceLock<MempoolManager> = crate::sync_oncelock::OnceLock::new();

/// Get mempool manager
pub fn mempool_manager() -> &'static MempoolManager {
    MEMPOOL_MANAGER.get_or_init(MempoolManager::new)
}

pub fn init_mempool_manager() -> &'static MempoolManager {
    MEMPOOL_MANAGER.get_or_init(MempoolManager::new)
}

/// Initialize mempool
pub fn init_mempool() {
    let mgr = mempool_manager();
    mgr.init();
}

// Convenience functions

/// Create mempool
pub fn mempool_create(name: &[u8], element_size: usize, min_nr: u32, max_nr: u32) -> Result<*mut Mempool, i32> {
    mempool_manager().create(name, element_size, min_nr, max_nr)
}

/// Destroy mempool
pub fn mempool_destroy(pool: *mut Mempool) {
    mempool_manager().destroy(pool);
}

/// Allocate from pool
pub fn mempool_alloc(pool: *mut Mempool) -> *mut u8 {
    if pool.is_null() {
        return core::ptr::null_mut();
    }
    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe { (*pool).alloc(0) }
}

/// Free to pool
pub fn mempool_free(pool: *mut Mempool, data: *mut u8) {
    if pool.is_null() {
        return;
    }
    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe { (*pool).free(data); }
}

/// Slab Allocator
pub struct SlabAllocator {
    /// Object size
    pub object_size: usize,
    /// Objects per slab
    pub objects_per_slab: u32,
    /// Slabs
    pub slabs: *mut Slab,
    /// Partial slabs
    pub partial: *mut Slab,
    /// Full slabs
    pub full: *mut Slab,
    /// Free slabs
    pub free: *mut Slab,
    /// Total slabs
    pub total_slabs: AtomicU32,
    /// Total objects
    pub total_objects: AtomicU32,
    /// Free objects
    pub free_objects: AtomicU32,
}

/// Slab
pub struct Slab {
    /// Memory
    pub memory: *mut u8,
    /// In use count
    pub in_use: AtomicU32,
    /// Free list
    pub free_list: *mut u8,
    /// Next slab
    pub next: *mut Slab,
    /// Prev slab
    pub prev: *mut Slab,
}

impl SlabAllocator {
    pub fn new(object_size: usize) -> Self {
        let objects_per_slab = (4096 - core::mem::size_of::<Slab>()) / object_size;
        
        SlabAllocator {
            object_size,
            objects_per_slab: objects_per_slab as u32,
            slabs: core::ptr::null_mut(),
            partial: core::ptr::null_mut(),
            full: core::ptr::null_mut(),
            free: core::ptr::null_mut(),
            total_slabs: AtomicU32::new(0),
            total_objects: AtomicU32::new(0),
            free_objects: AtomicU32::new(0),
        }
    }
    
    /// Allocate object
    pub fn alloc(&mut self) -> *mut u8 {
        // Try partial slabs first
        if !self.partial.is_null() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                let slab = self.partial;
                let obj = (*slab).free_list;
                
                if !obj.is_null() {
                    (*slab).free_list = *(obj as *const *mut u8);
                    (*slab).in_use.fetch_add(1, Ordering::AcqRel);
                    self.free_objects.fetch_sub(1, Ordering::AcqRel);
                    
                    // Move to full if needed
                    if (*slab).in_use.load(Ordering::Acquire) == self.objects_per_slab {
                        self.partial = (*slab).next;
                        (*slab).next = self.full;
                        self.full = slab;
                    }
                    
                    return obj;
                }
            }
        }
        
        // Need new slab
        // TODO: Allocate new slab
        
        core::ptr::null_mut()
    }
    
    /// Free object
    pub fn free(&mut self, obj: *mut u8) {
        if obj.is_null() {
            return;
        }
        
        // Find slab
        // TODO: Find slab containing object
        
        self.free_objects.fetch_add(1, Ordering::AcqRel);
    }
}
