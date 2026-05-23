/*
 * Nuva OS - Kernel - Memory Pool and Object Cache
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

use core::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, AtomicPtr, Ordering};
use alloc::alloc::{alloc, dealloc, Layout};
use alloc::vec::Vec;
use crate::{pr_info};

use crate::posix::errno::Errno;
/// Maximum objects per pool
const MEMPOOL_MAX_OBJECTS: u32 = 65536;

/// Maximum per-cpu cache slots
const MEMPOOL_PERCPU_SLOTS: usize = 64;

/// Cache line size for slab alignment
const SLAB_CACHE_LINE_SIZE: usize = 64;

/// Align size up to cache line boundary (64 bytes)
#[inline(always)]
const fn cache_line_aligned_size(size: u32) -> u32 {
    ((size as usize + (SLAB_CACHE_LINE_SIZE - 1)) & !(SLAB_CACHE_LINE_SIZE - 1)) as u32
}

/// Memory pool error type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemPoolError {
    /// Out of memory
    NoMem,
    /// Pool is full
    Full,
    /// Invalid argument
    Invalid,
    /// Object not found in pool
    NotFound,
}

/// Memory Pool
/// Pre-allocated object pool for fast allocation/deallocation.
/// Supports per-CPU caches for lock-free access on the local CPU.
/// Object sizes are aligned to cache line boundaries (64 bytes) to prevent
/// false sharing between adjacent objects.
#[repr(C, align(64))]
pub struct MemPool {
    /// Pool name
    pub name: [u8; 32],
    /// Object size in bytes
    pub object_size: u32,
    /// Object alignment
    pub align: u32,
    /// Number of pre-allocated objects
    pub capacity: u32,
    /// Allocated objects count
    pub allocated: AtomicU32,
    /// Pool enabled
    pub enabled: AtomicBool,
    /// Object storage array (pointers to allocated objects)
    pub objects: AtomicPtr<*mut u8>,
    /// Free list: stack of free slot indices
    pub free_stack: AtomicPtr<u32>,
    /// Free stack top
    pub free_top: AtomicU32,
    /// Per-CPU caches
    pub percpu_cache: [PerCpuObjCache; 256],
    /// Statistics
    pub stats: MemPoolStats,
}

/// Per-CPU object cache
/// Lock-free cache of pre-allocated objects for a single CPU.
pub struct PerCpuObjCache {
    /// Cached object pointers
    pub objects: [*mut u8; MEMPOOL_PERCPU_SLOTS],
    /// Number of cached objects
    pub count: AtomicU32,
    /// Batch size for refill/drain
    pub batch: u32,
    /// Hit count
    pub hit_count: AtomicU64,
    /// Miss count (fallback to global pool)
    pub miss_count: AtomicU64,
}

impl PerCpuObjCache {
    pub const fn new() -> Self {
        PerCpuObjCache {
            objects: [core::ptr::null_mut(); MEMPOOL_PERCPU_SLOTS],
            count: AtomicU32::new(0),
            batch: 8,
            hit_count: AtomicU64::new(0),
            miss_count: AtomicU64::new(0),
        }
    }

    /// Try to allocate from the per-CPU cache
    pub fn alloc(&mut self) -> *mut u8 {
        let count = self.count.load(Ordering::Acquire);
        if count == 0 {
            self.miss_count.fetch_add(1, Ordering::AcqRel);
            return core::ptr::null_mut();
        }
        let idx = (count - 1) as usize;
        let ptr = self.objects[idx];
        self.objects[idx] = core::ptr::null_mut();
        self.count.fetch_sub(1, Ordering::AcqRel);
        self.hit_count.fetch_add(1, Ordering::AcqRel);
        ptr
    }

    /// Try to free to the per-CPU cache
    /// @return: true if freed to cache, false if cache full
    pub fn free(&mut self, ptr: *mut u8) -> bool {
        let count = self.count.load(Ordering::Acquire);
        if count as usize >= MEMPOOL_PERCPU_SLOTS {
            return false;
        }
        self.objects[count as usize] = ptr;
        self.count.fetch_add(1, Ordering::AcqRel);
        true
    }

    /// Refill cache from a source of objects
    /// @param source: slice of object pointers to transfer
    /// @return: number of objects transferred
    pub fn refill(&mut self, source: &[*mut u8]) -> usize {
        let count = self.count.load(Ordering::Acquire);
        let available = MEMPOOL_PERCPU_SLOTS - count as usize;
        let transfer = source.len().min(available).min(self.batch as usize);
        for i in 0..transfer {
            self.objects[(count as usize) + i] = source[i];
        }
        self.count.fetch_add(transfer as u32, Ordering::AcqRel);
        transfer
    }
}

/// Memory pool statistics
pub struct MemPoolStats {
    /// Total allocations
    pub alloc_count: AtomicU64,
    /// Total frees
    pub free_count: AtomicU64,
    /// Cache hits
    pub cache_hits: AtomicU64,
    /// Cache misses
    pub cache_misses: AtomicU64,
    /// Failed allocations
    pub alloc_fails: AtomicU64,
}

impl MemPoolStats {
    pub const fn new() -> Self {
        MemPoolStats {
            alloc_count: AtomicU64::new(0),
            free_count: AtomicU64::new(0),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
            alloc_fails: AtomicU64::new(0),
        }
    }
}

impl MemPool {
    /// Create a new memory pool
    /// @param name: pool name
    /// @param object_size: size of each object in bytes
    /// @param align: object alignment
    /// @param capacity: number of pre-allocated objects
    pub fn create(
        name: &str,
        object_size: u32,
        align: u32,
        capacity: u32,
    ) -> Result<Self, MemPoolError> {
        if object_size == 0 || capacity == 0 {
            return Err(MemPoolError::Invalid);
        }

        let aligned_size = cache_line_aligned_size(object_size);
        let effective_align = if align == 0 { 8 } else { align };
        let final_align = if effective_align < SLAB_CACHE_LINE_SIZE as u32 {
            SLAB_CACHE_LINE_SIZE as u32
        } else {
            effective_align
        };

        #[cfg(feature = "debug")]
        {
            if aligned_size % SLAB_CACHE_LINE_SIZE as u32 != 0 {
                return Err(MemPoolError::Invalid);
            }
        }

        let mut name_arr = [0u8; 32];
        let name_bytes = name.as_bytes();
        let len = name_bytes.len().min(31);
        name_arr[..len].copy_from_slice(&name_bytes[..len]);

        let pool = MemPool {
            name: name_arr,
            object_size: aligned_size,
            align: final_align,
            capacity,
            allocated: AtomicU32::new(0),
            enabled: AtomicBool::new(true),
            objects: AtomicPtr::new(core::ptr::null_mut()),
            free_stack: AtomicPtr::new(core::ptr::null_mut()),
            free_top: AtomicU32::new(0),
            percpu_cache: core::array::from_fn(|_| PerCpuObjCache::new()),
            stats: MemPoolStats::new(),
        };

        Ok(pool)
    }

    /// Initialize pool: pre-allocate objects
    /// @return: 0 on success, negative errno on failure
    pub fn init(&mut self) -> i32 {
        let obj_layout = match Layout::from_size_align(
            self.object_size as usize,
            self.align as usize,
        ) {
            Ok(l) => l,
            Err(_) => return Errno::Einval.to_ret_i32(), // EINVAL
        };

        let cap = self.capacity as usize;

        // SAFETY: allocating array of object pointers
        let obj_arr = unsafe { alloc(Layout::array::<*mut u8>(cap).ok().unwrap_or(Layout::new::<*mut u8>())) };
        if obj_arr.is_null() {
            return Errno::Enomem.to_ret_i32(); // ENOMEM
        }

        // SAFETY: allocating free stack
        let stack = unsafe { alloc(Layout::array::<u32>(cap).ok().unwrap_or(Layout::new::<u32>())) };
        if stack.is_null() {
            // SAFETY: freeing previously allocated obj_arr
            unsafe { dealloc(obj_arr, Layout::array::<*mut u8>(cap).unwrap_or(Layout::new::<*mut u8>())); }
            return Errno::Enomem.to_ret_i32();
        }

        // SAFETY: writing to allocated arrays
        unsafe {
            let obj_ptrs = core::slice::from_raw_parts_mut(obj_arr as *mut *mut u8, cap);
            let free_slots = core::slice::from_raw_parts_mut(stack as *mut u32, cap);

            for i in 0..cap {
                let obj = alloc(obj_layout);
                if obj.is_null() {
                    // Free already-allocated objects
                    for j in 0..i {
                        dealloc(obj_ptrs[j], obj_layout);
                    }
                    dealloc(obj_arr, Layout::array::<*mut u8>(cap).unwrap_or(Layout::new::<*mut u8>()));
                    dealloc(stack, Layout::array::<u32>(cap).unwrap_or(Layout::new::<u32>()));
                    return Errno::Enomem.to_ret_i32();
                }
                obj_ptrs[i] = obj;
                free_slots[i] = i as u32;
            }
        }

        self.objects.store(obj_arr as *mut *mut u8, Ordering::Release);
        self.free_stack.store(stack as *mut u32, Ordering::Release);
        self.free_top.store(cap as u32, Ordering::Release);
        self.enabled.store(true, Ordering::Release);

        0
    }

    /// Allocate an object from the pool
    /// @param cpu_id: CPU ID for per-CPU cache lookup
    pub fn alloc(&mut self, cpu_id: u32) -> *mut u8 {
        if !self.enabled.load(Ordering::Acquire) {
            return core::ptr::null_mut();
        }

        // Try per-CPU cache first
        let cpu_idx = (cpu_id as usize).min(255);
        let ptr = self.percpu_cache[cpu_idx].alloc();
        if !ptr.is_null() {
            self.stats.cache_hits.fetch_add(1, Ordering::AcqRel);
            self.stats.alloc_count.fetch_add(1, Ordering::AcqRel);
            return ptr;
        }

        // Fallback to global free stack
        self.stats.cache_misses.fetch_add(1, Ordering::AcqRel);

        let top = self.free_top.load(Ordering::Acquire);
        if top == 0 {
            self.stats.alloc_fails.fetch_add(1, Ordering::AcqRel);
            return core::ptr::null_mut();
        }

        let obj_arr = self.objects.load(Ordering::Acquire);
        let stack = self.free_stack.load(Ordering::Acquire);
        if obj_arr.is_null() || stack.is_null() {
            return core::ptr::null_mut();
        }

        // SAFETY: accessing within allocated bounds
        unsafe {
            let idx = *stack.add((top - 1) as usize);
            let ptr = *obj_arr.add(idx as usize);
            self.free_top.store(top - 1, Ordering::Release);
            self.allocated.fetch_add(1, Ordering::AcqRel);
            self.stats.alloc_count.fetch_add(1, Ordering::AcqRel);
            ptr
        }
    }

    /// Free an object back to the pool
    /// @param ptr: pointer to the object
    /// @param cpu_id: CPU ID for per-CPU cache
    pub fn free(&mut self, ptr: *mut u8, cpu_id: u32) {
        if ptr.is_null() || !self.enabled.load(Ordering::Acquire) {
            return;
        }

        // Try per-CPU cache first
        let cpu_idx = (cpu_id as usize).min(255);
        if self.percpu_cache[cpu_idx].free(ptr) {
            self.stats.free_count.fetch_add(1, Ordering::AcqRel);
            return;
        }

        // Fallback: push to global free stack
        let stack = self.free_stack.load(Ordering::Acquire);
        let top = self.free_top.load(Ordering::Acquire);
        if stack.is_null() || top >= self.capacity {
            return;
        }

        // SAFETY: accessing within allocated bounds
        unsafe {
            // Find the object index by scanning (slow path)
            let obj_arr = self.objects.load(Ordering::Acquire);
            if obj_arr.is_null() {
                return;
            }
            for i in 0..self.capacity as usize {
                if *obj_arr.add(i) == ptr {
                    *stack.add(top as usize) = i as u32;
                    self.free_top.store(top + 1, Ordering::Release);
                    self.allocated.fetch_sub(1, Ordering::AcqRel);
                    self.stats.free_count.fetch_add(1, Ordering::AcqRel);
                    return;
                }
            }
        }
    }

    /// Destroy the memory pool and free all resources
    pub fn destroy(&mut self) {
        self.enabled.store(false, Ordering::Release);

        let obj_arr = self.objects.load(Ordering::Acquire);
        let stack = self.free_stack.load(Ordering::Acquire);
        let cap = self.capacity as usize;

        if obj_arr.is_null() {
            return;
        }

        let obj_layout = Layout::from_size_align(
            self.object_size as usize,
            self.align as usize,
        ).unwrap_or(Layout::new::<u8>());

        // SAFETY: freeing all allocated objects and arrays
        unsafe {
            let obj_ptrs = core::slice::from_raw_parts(obj_arr, cap);
            for i in 0..cap {
                if !obj_ptrs[i].is_null() {
                    dealloc(obj_ptrs[i], obj_layout);
                }
            }

            dealloc(obj_arr as *mut u8, Layout::array::<*mut u8>(cap).unwrap_or(Layout::new::<*mut u8>()));

            if !stack.is_null() {
                dealloc(stack as *mut u8, Layout::array::<u32>(cap).unwrap_or(Layout::new::<u32>()));
            }
        }

        self.objects.store(core::ptr::null_mut(), Ordering::Release);
        self.free_stack.store(core::ptr::null_mut(), Ordering::Release);
        self.free_top.store(0, Ordering::Release);
    }

    /// Get number of allocated objects
    pub fn in_use(&self) -> u32 {
        self.allocated.load(Ordering::Acquire)
    }

    /// Get pool capacity
    pub fn capacity(&self) -> u32 {
        self.capacity
    }
}

/// Object cache (optimized slab cache)
/// A lightweight wrapper around MemPool providing a simple
/// object cache interface with automatic per-CPU cache selection.
pub struct ObjectCache {
    /// Backing memory pool
    pub pool: MemPool,
    /// Constructor function
    pub ctor: Option<fn(*mut u8)>,
    /// Destructor function
    pub dtor: Option<fn(*mut u8)>,
}

impl ObjectCache {
    /// Create a new object cache
    pub fn create(
        name: &str,
        object_size: u32,
        align: u32,
        capacity: u32,
    ) -> Result<Self, MemPoolError> {
        let pool = MemPool::create(name, object_size, align, capacity)?;
        Ok(ObjectCache {
            pool,
            ctor: None,
            dtor: None,
        })
    }

    /// Set constructor
    pub fn set_ctor(&mut self, ctor: fn(*mut u8)) {
        self.ctor = Some(ctor);
    }

    /// Set destructor
    pub fn set_dtor(&mut self, dtor: fn(*mut u8)) {
        self.dtor = Some(dtor);
    }

    /// Initialize the cache
    pub fn init(&mut self) -> i32 {
        let rc = self.pool.init();
        if rc != 0 {
            return rc;
        }

        // Run constructor on all objects if set
        if let Some(ctor) = self.ctor {
            let obj_arr = self.pool.objects.load(Ordering::Acquire);
            if !obj_arr.is_null() {
                let cap = self.pool.capacity as usize;
                // SAFETY: accessing within allocated bounds
                unsafe {
                    let obj_ptrs = core::slice::from_raw_parts(obj_arr, cap);
                    for i in 0..cap {
                        if !obj_ptrs[i].is_null() {
                            ctor(obj_ptrs[i]);
                        }
                    }
                }
            }
        }

        0
    }

    /// Allocate an object from the cache
    pub fn alloc(&mut self, cpu_id: u32) -> *mut u8 {
        self.pool.alloc(cpu_id)
    }

    /// Free an object back to the cache
    pub fn free(&mut self, ptr: *mut u8, cpu_id: u32) {
        if let Some(dtor) = self.dtor {
            if !ptr.is_null() {
                dtor(ptr);
            }
        }
        self.pool.free(ptr, cpu_id);
    }

    /// Destroy the cache
    pub fn destroy(&mut self) {
        self.pool.destroy();
    }
}

/// Public API: Create a memory pool
pub fn mpool_create(
    name: &str,
    object_size: u32,
    align: u32,
    capacity: u32,
) -> Result<MemPool, MemPoolError> {
    MemPool::create(name, object_size, align, capacity)
}

/// Public API: Allocate from a memory pool
pub fn mpool_alloc(pool: &mut MemPool, cpu_id: u32) -> *mut u8 {
    pool.alloc(cpu_id)
}

/// Public API: Free to a memory pool
pub fn mpool_free(pool: &mut MemPool, ptr: *mut u8, cpu_id: u32) {
    pool.free(ptr, cpu_id);
}

/// Public API: Destroy a memory pool
pub fn mpool_destroy(pool: &mut MemPool) {
    pool.destroy();
}

/// Initialize memory pool subsystem
pub fn init_mempool_opt() {
    log_info!("Memory pool optimization subsystem initialized");
}
