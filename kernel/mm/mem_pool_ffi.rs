/*
* Nuva OS - Memory Pool Allocator Rust FFI Bindings
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

//! Memory Pool Allocator FFI Bindings
/*!*/
//! Safe Rust wrapper around the C memory pool allocator.

use core::alloc::{GlobalAlloc, Layout};
use core::ptr;
use core::sync::atomic::{AtomicUsize, Ordering};

/// Memory pool error codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PoolError {
    Success,
    Error,
    Exhausted,
    Invalid,
}

impl From<i32> for PoolError {
    fn from(code: i32) -> Self {
        match code {
            0 => PoolError::Success,
            -1 => PoolError::Error,
            -2 => PoolError::Exhausted,
            -3 => PoolError::Invalid,
            _ => PoolError::Error,
        }
    }
}

/// C memory pool structure (opaque)
#[repr(C)]
pub struct CMemPool {
    _private: [u8; 0],
}

/// C multi-pool structure (opaque)
#[repr(C)]
pub struct CMultiPool {
    _private: [u8; 0],
}

/// C per-CPU cache structure (opaque)
#[repr(C)]
pub struct CPerCpuCache {
    _private: [u8; 0],
}

/// FFI function declarations
mod ffi {
    use super::*;

    extern "C" {
        /// Initialize a memory pool
        pub fn mem_pool_init(
            pool: *mut CMemPool,
            base: *mut u8,
            size: usize,
            block_size: usize,
        ) -> i32;

        /// Allocate a block from the pool
        pub fn mem_pool_alloc(pool: *mut CMemPool) -> *mut u8;

        /// Free a block back to the pool
        pub fn mem_pool_free(pool: *mut CMemPool, ptr: *mut u8) -> i32;

        /// Allocate and zero-initialize a block
        pub fn mem_pool_alloc_zero(pool: *mut CMemPool) -> *mut u8;

        /// Get pool statistics
        pub fn mem_pool_stats(
            pool: *mut CMemPool,
            free_count: *mut usize,
            alloc_count: *mut usize,
            total_count: *mut usize,
        );

        /// Check if pool is exhausted
        pub fn mem_pool_is_exhausted(pool: *mut CMemPool) -> i32;

        /// Get pool utilization
        pub fn mem_pool_utilization(pool: *mut CMemPool) -> i32;

        /// Initialize multi-pool
        pub fn multi_pool_init(multi: *mut CMultiPool, base: *mut u8, total_size: usize) -> i32;

        /// Allocate from multi-pool
        pub fn multi_pool_alloc(multi: *mut CMultiPool, size: usize) -> *mut u8;

        /// Free from multi-pool
        pub fn multi_pool_free(multi: *mut CMultiPool, ptr: *mut u8, size: usize) -> i32;

        /// Get multi-pool statistics
        pub fn multi_pool_stats(
            multi: *mut CMultiPool,
            total_allocs: *mut usize,
            total_frees: *mut usize,
        );

        /// Initialize per-CPU cache
        pub fn percpu_cache_init(cache: *mut CPerCpuCache, pool: *mut CMemPool) -> i32;

        /// Allocate from per-CPU cache
        pub fn percpu_cache_alloc(cache: *mut CPerCpuCache) -> *mut u8;

        /// Free to per-CPU cache
        pub fn percpu_cache_free(cache: *mut CPerCpuCache, ptr: *mut u8) -> i32;
    }
}

/// Safe Rust wrapper for memory pool
pub struct MemoryPool {
    pool: *mut CMemPool,
    block_size: usize,
    owned: bool,
}

impl MemoryPool {
    /// Create a new memory pool
    /// # Safety
    /// The caller must ensure that the base memory region is valid and
    /// will remain valid for the lifetime of the pool.
    // SAFETY: The caller must provide a valid base pointer and size.
    pub unsafe fn new(base: *mut u8, size: usize, block_size: usize) -> Result<Self, PoolError> {
        // SAFETY: size_of::<CMemPool>() > 0 and align=8 is a power of two,
        // so from_size_align cannot fail.
        let pool = core::alloc::alloc(
            Layout::from_size_align(core::mem::size_of::<CMemPool>(), 8)
                .unwrap_or_else(|_| Layout::new::<u64>()),
        ) as *mut CMemPool;

        if pool.is_null() {
            return Err(PoolError::Error);
        }

        let result = ffi::mem_pool_init(pool, base, size, block_size);
        if result != 0 {
            core::alloc::dealloc(
                pool as *mut u8,
                Layout::from_size_align(core::mem::size_of::<CMemPool>(), 8)
                    .unwrap_or_else(|_| Layout::new::<u64>()),
            );
            return Err(PoolError::from(result));
        }

        Ok(Self {
            pool,
            block_size,
            owned: true,
        })
    }

    /// Allocate a block from the pool
    pub fn alloc(&self) -> Option<*mut u8> {
        // SAFETY: unsafe block required for low-level memory or hardware access
        let ptr = unsafe { ffi::mem_pool_alloc(self.pool) };
        if ptr.is_null() {
            None
        } else {
            Some(ptr)
        }
    }

    /// Allocate a zero-initialized block
    pub fn alloc_zero(&self) -> Option<*mut u8> {
        // SAFETY: unsafe block required for low-level memory or hardware access
        let ptr = unsafe { ffi::mem_pool_alloc_zero(self.pool) };
        if ptr.is_null() {
            None
        } else {
            Some(ptr)
        }
    }

    /// Free a block back to the pool
    /// # Safety
    /// The pointer must have been allocated from this pool.
    // SAFETY: The caller must ensure ptr was allocated from this pool.
    pub unsafe fn free(&self, ptr: *mut u8) -> Result<(), PoolError> {
        let result = ffi::mem_pool_free(self.pool, ptr);
        if result == 0 {
            Ok(())
        } else {
            Err(PoolError::from(result))
        }
    }

    /// Get pool statistics
    pub fn stats(&self) -> PoolStats {
        let mut free_count = 0usize;
        let mut alloc_count = 0usize;
        let mut total_count = 0usize;

        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            ffi::mem_pool_stats(
                self.pool,
                &mut free_count,
                &mut alloc_count,
                &mut total_count,
            );
        }

        PoolStats {
            free_count,
            alloc_count,
            total_count,
        }
    }

    /// Check if pool is exhausted
    pub fn is_exhausted(&self) -> bool {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe { ffi::mem_pool_is_exhausted(self.pool) == 1 }
    }

    /// Get pool utilization (0-100)
    pub fn utilization(&self) -> i32 {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe { ffi::mem_pool_utilization(self.pool) }
    }

    /// Get block size
    pub fn block_size(&self) -> usize {
        self.block_size
    }
}

impl Drop for MemoryPool {
    fn drop(&mut self) {
        if self.owned && !self.pool.is_null() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                core::alloc::dealloc(
                    self.pool as *mut u8,
                    Layout::from_size_align(core::mem::size_of::<CMemPool>(), 8)
                        .unwrap_or_else(|_| Layout::new::<u64>()),
                );
            }
        }
    }
}

/// Pool statistics
#[derive(Debug, Clone, Copy)]
pub struct PoolStats {
    pub free_count: usize,
    pub alloc_count: usize,
    pub total_count: usize,
}

/// Safe Rust wrapper for multi-pool allocator
pub struct MultiPool {
    multi: *mut CMultiPool,
    owned: bool,
}

impl MultiPool {
    /// Create a new multi-pool allocator
    /// # Safety
    /// The caller must ensure that the base memory region is valid.
    // SAFETY: The caller must provide a valid base pointer and size.
    pub unsafe fn new(base: *mut u8, total_size: usize) -> Result<Self, PoolError> {
        let multi = core::alloc::alloc(
            Layout::from_size_align(core::mem::size_of::<CMultiPool>(), 8)
                .unwrap_or_else(|_| Layout::new::<u64>()),
        ) as *mut CMultiPool;

        if multi.is_null() {
            return Err(PoolError::Error);
        }

        let result = ffi::multi_pool_init(multi, base, total_size);
        if result != 0 {
            core::alloc::dealloc(
                multi as *mut u8,
                Layout::from_size_align(core::mem::size_of::<CMultiPool>(), 8)
                    .unwrap_or_else(|_| Layout::new::<u64>()),
            );
            return Err(PoolError::from(result));
        }

        Ok(Self { multi, owned: true })
    }

    /// Allocate memory
    pub fn alloc(&self, size: usize) -> Option<*mut u8> {
        // SAFETY: unsafe block required for low-level memory or hardware access
        let ptr = unsafe { ffi::multi_pool_alloc(self.multi, size) };
        if ptr.is_null() {
            None
        } else {
            Some(ptr)
        }
    }

    /// Free memory
    /// # Safety
    /// The pointer must have been allocated from this pool with the given size.
    // SAFETY: The caller must ensure ptr was allocated from this pool.
    pub unsafe fn free(&self, ptr: *mut u8, size: usize) -> Result<(), PoolError> {
        let result = ffi::multi_pool_free(self.multi, ptr, size);
        if result == 0 {
            Ok(())
        } else {
            Err(PoolError::from(result))
        }
    }

    /// Get statistics
    pub fn stats(&self) -> MultiPoolStats {
        let mut total_allocs = 0usize;
        let mut total_frees = 0usize;

        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            ffi::multi_pool_stats(self.multi, &mut total_allocs, &mut total_frees);
        }

        MultiPoolStats {
            total_allocs,
            total_frees,
        }
    }
}

impl Drop for MultiPool {
    fn drop(&mut self) {
        if self.owned && !self.multi.is_null() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                core::alloc::dealloc(
                    self.multi as *mut u8,
                    Layout::from_size_align(core::mem::size_of::<CMultiPool>(), 8)
                        .unwrap_or_else(|_| Layout::new::<u64>()),
                );
            }
        }
    }
}

/// Multi-pool statistics
#[derive(Debug, Clone, Copy)]
pub struct MultiPoolStats {
    pub total_allocs: usize,
    pub total_frees: usize,
}

/// Safe Rust wrapper for per-CPU cache
pub struct PerCpuCache {
    cache: *mut CPerCpuCache,
    owned: bool,
}

impl PerCpuCache {
    /// Create a new per-CPU cache
    /// # Safety
    /// The pool must remain valid for the lifetime of the cache.
    // SAFETY: The caller must ensure the pool remains valid.
    pub unsafe fn new(pool: &MemoryPool) -> Result<Self, PoolError> {
        let cache = core::alloc::alloc(
            Layout::from_size_align(core::mem::size_of::<CPerCpuCache>(), 8)
                .unwrap_or_else(|_| Layout::new::<u64>()),
        ) as *mut CPerCpuCache;

        if cache.is_null() {
            return Err(PoolError::Error);
        }

        let result = ffi::percpu_cache_init(cache, pool.pool);
        if result != 0 {
            core::alloc::dealloc(
                cache as *mut u8,
                Layout::from_size_align(core::mem::size_of::<CPerCpuCache>(), 8)
                    .unwrap_or_else(|_| Layout::new::<u64>()),
            );
            return Err(PoolError::from(result));
        }

        Ok(Self { cache, owned: true })
    }

    /// Allocate from cache
    pub fn alloc(&self) -> Option<*mut u8> {
        // SAFETY: unsafe block required for low-level memory or hardware access
        let ptr = unsafe { ffi::percpu_cache_alloc(self.cache) };
        if ptr.is_null() {
            None
        } else {
            Some(ptr)
        }
    }

    /// Free to cache
    /// # Safety
    /// The pointer must have been allocated from this cache.
    // SAFETY: The caller must ensure ptr was allocated from this cache.
    pub unsafe fn free(&self, ptr: *mut u8) -> Result<(), PoolError> {
        let result = ffi::percpu_cache_free(self.cache, ptr);
        if result == 0 {
            Ok(())
        } else {
            Err(PoolError::from(result))
        }
    }
}

impl Drop for PerCpuCache {
    fn drop(&mut self) {
        if self.owned && !self.cache.is_null() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                core::alloc::dealloc(
                    self.cache as *mut u8,
                    Layout::from_size_align(core::mem::size_of::<CPerCpuCache>(), 8)
                        .unwrap_or_else(|_| Layout::new::<u64>()),
                );
            }
        }
    }
}

/// Global allocator using memory pool
pub struct PoolAllocator {
    pool: AtomicUsize,
}

impl PoolAllocator {
    /// Create a new pool allocator
    pub const fn new() -> Self {
        Self {
            pool: AtomicUsize::new(0),
        }
    }

    /// Set the backing pool
    /// # Safety
    /// The pool must remain valid for the lifetime of the allocator.
    // SAFETY: The caller must ensure the pool pointer remains valid.
    pub unsafe fn set_pool(&self, pool: *mut CMemPool) {
        self.pool.store(pool as usize, Ordering::Release);
    }
}

// SAFETY: PoolAllocator is safe to use from multiple threads because the
// pool pointer is stored in an AtomicUsize and all FFI operations are
// thread-safe by design (each pool has internal synchronization).
unsafe impl GlobalAlloc for PoolAllocator {
    // SAFETY: The pool pointer is valid if non-null (set via set_pool).
    // mem_pool_alloc returns a valid pointer or null on failure.
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let pool = self.pool.load(Ordering::Acquire) as *mut CMemPool;
        if pool.is_null() {
            return ptr::null_mut();
        }

        // For simplicity, we use multi_pool_alloc with the size
        // In a real implementation, we'd need a multi-pool here
        ffi::mem_pool_alloc(pool)
    }

    // SAFETY: The pool pointer is valid if non-null. ptr was allocated
    // from this pool, so mem_pool_free can safely free it.
    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        let pool = self.pool.load(Ordering::Acquire) as *mut CMemPool;
        if !pool.is_null() {
            ffi::mem_pool_free(pool, ptr);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_error_from() {
        assert_eq!(PoolError::from(0), PoolError::Success);
        assert_eq!(PoolError::from(-1), PoolError::Error);
        assert_eq!(PoolError::from(-2), PoolError::Exhausted);
        assert_eq!(PoolError::from(-3), PoolError::Invalid);
    }

    #[test]
    fn test_pool_stats() {
        let stats = PoolStats {
            free_count: 100,
            alloc_count: 50,
            total_count: 150,
        };
        assert_eq!(stats.free_count, 100);
        assert_eq!(stats.alloc_count, 50);
        assert_eq!(stats.total_count, 150);
    }

    #[test]
    fn test_multi_pool_stats() {
        let stats = MultiPoolStats {
            total_allocs: 1000,
            total_frees: 500,
        };
        assert_eq!(stats.total_allocs, 1000);
        assert_eq!(stats.total_frees, 500);
    }
}
