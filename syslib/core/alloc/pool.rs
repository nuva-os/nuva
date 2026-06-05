/*
 * Nuva OS - Syslib - Core - Alloc - Pool
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
/*
 * Memory Pool - High-Performance Memory Allocation
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * This module implements efficient memory pool allocation
 * for fixed-size blocks with minimal fragmentation.
 */

use core::ptr;
use core::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};
use alloc::vec::Vec;
use alloc::boxed::Box;
use spin::Mutex;

/// Memory pool for fixed-size blocks
/// Provides fast allocation and deallocation of fixed-size
/// memory blocks with minimal overhead.
pub struct MemoryPool {
    /// Block size
    block_size: usize,

    /// Number of blocks per chunk
    blocks_per_chunk: usize,

    /// Free list
    free_list: AtomicPtr<FreeBlock>,

    /// Chunks
    chunks: Mutex<Vec<Chunk>>,

    /// Statistics
    stats: PoolStats,
}

impl MemoryPool {
    /// Create new memory pool
    /// @param block_size: Size of each block
    /// @param initial_blocks: Initial number of blocks
    pub fn new(block_size: usize, initial_blocks: usize) -> Self {
        let blocks_per_chunk = 64; // Configurable
        let initial_chunks = (initial_blocks + blocks_per_chunk - 1) / blocks_per_chunk;

        let mut pool = Self {
            block_size: block_size.max(core::mem::size_of::<FreeBlock>()),
            blocks_per_chunk,
            free_list: AtomicPtr::new(ptr::null_mut()),
            chunks: Mutex::new(Vec::new()),
            stats: PoolStats::default(),
        };

        // Allocate initial chunks
        for _ in !0..initial_chunks {
            if let Err(_) = pool.allocate_chunk() {
                break;
            }
        }

        pool
    }

    /// Allocate a block
    /// @return: Pointer to block, or null if out of memory
    pub fn alloc(&self) -> *mut u8 {
        // Try to pop from free list
        loop {
            let head = self.free_list.load(Ordering::Acquire);

            if head.is_null() {
                // Free list is empty, try to allocate new chunk
                if let Err(_) = self.allocate_chunk() {
                    return ptr::null_mut();
                }
                continue;
            }

            // SAFETY: unsafe block required for low-level memory or hardware access
            let next = unsafe { (*head).next };

            if self.free_list.compare_exchange_weak(
                head,
                next,
                Ordering::Release,
                Ordering::Relaxed,
            ).is_ok() {
                self.stats.allocated.fetch_add(1, Ordering::Relaxed);
                return head as *mut u8;
            }
        }
    }

    /// Free a block
    /// @param ptr: Pointer to block
    pub fn free(&self, ptr: *mut u8) {
        if ptr.is_null() {
            return;
        }

        // Push to free list
        let block = ptr as *mut FreeBlock;
        loop {
            let head = self.free_list.load(Ordering::Acquire);
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe { (*block).next = head; }

            if self.free_list.compare_exchange_weak(
                head,
                block,
                Ordering::Release,
                Ordering::Relaxed,
            ).is_ok() {
                self.stats.allocated.fetch_sub(1, Ordering::Relaxed);
                return;
            }
        }
    }

    /// Allocate a new chunk
    fn allocate_chunk(&self) -> Result<(), ()> {
        // Allocate chunk memory
        let chunk_size = self.block_size * self.blocks_per_chunk;
        let mut chunk_memory = Vec::with_capacity(chunk_size);
        chunk_memory.resize(chunk_size, 0);
        let chunk_ptr = chunk_memory.as_mut_ptr();

        // Create chunk
        let chunk = Chunk {
            memory: chunk_memory.into_boxed_slice(),
            base: chunk_ptr,
        };

        // Add blocks to free list
        for i in 0..self.blocks_per_chunk {
            // SAFETY: unsafe block required for low-level memory or hardware access
            let block_ptr = unsafe { chunk_ptr.add(i * self.block_size) } as *mut FreeBlock;
            loop {
                let head = self.free_list.load(Ordering::Acquire);
                // SAFETY: unsafe block required for low-level memory or hardware access
                unsafe { (*block_ptr).next = head; }

                if self.free_list.compare_exchange_weak(
                    head,
                    block_ptr,
                    Ordering::Release,
                    Ordering::Relaxed,
                ).is_ok() {
                    break;
                }
            }
        }

        // Add chunk to list
        let mut chunks = self.chunks.lock();
        chunks.push(chunk);
        self.stats.chunks.fetch_add(1, Ordering::Relaxed);
        self.stats.capacity.fetch_add(self.blocks_per_chunk, Ordering::Relaxed);

        Ok(())
    }

    /// Get block size
    pub fn block_size(&self) -> usize {
        self.block_size
    }

    /// Get capacity
    pub fn capacity(&self) -> usize {
        self.stats.capacity.load(Ordering::Relaxed)
    }

    /// Get allocated count
    pub fn allocated(&self) -> usize {
        self.stats.allocated.load(Ordering::Relaxed)
    }

    /// Get available count
    pub fn available(&self) -> usize {
        self.capacity() - self.allocated()
    }
}

/// Free block node
struct FreeBlock {
    next: *mut FreeBlock,
}

/// Memory chunk
struct Chunk {
    /// Chunk memory
    memory: Box<[u8]>,

    /// Base pointer
    base: *mut u8,
}

/// Pool statistics
struct PoolStats {
    /// Total chunks
    chunks: AtomicUsize,

    /// Total capacity
    capacity: AtomicUsize,

    /// Currently allocated
    allocated: AtomicUsize,
}

impl Default for PoolStats {
    fn default() -> Self {
        Self {
            chunks: AtomicUsize::new(0),
            capacity: AtomicUsize::new(0),
            allocated: AtomicUsize::new(0),
        }
    }
}

/// Memory pool manager
/// Manages multiple memory pools for different sizes.
pub struct PoolManager {
    /// Size classes
    size_classes: Vec<usize>,

    /// Memory pools
    pools: Vec<MemoryPool>,

    /// Configuration
    config: PoolManagerConfig,
    pub stats: PoolStats,
}

impl PoolManager {
    /// Create new pool manager
    /// @param config: Manager configuration
    pub fn new(config: PoolManagerConfig) -> Self {
        // Define size classes (powers of 2)
        let size_classes: Vec<usize> = (4..=12)
            .map(|i| 1 << i) // 16, 32, 64, 128, 256, 512, 1024, 2048, 4096
            .collect();

        // Create pools for each size class
        let pools: Vec<MemoryPool> = size_classes
            .iter()
            .map(|&size| MemoryPool::new(size, config.initial_blocks))
            .collect();

        Self {
            size_classes,
            pools,
            config,
                stats: PoolStats::default(),
            }
    }

    /// Allocate memory
    /// @param size: Requested size
    /// @return: Pointer to memory
    pub fn alloc(&self, size: usize) -> *mut u8 {
        if size == 0 {
            return ptr::null_mut();
        }

        // Find appropriate pool
        for (i, &class_size) in self.size_classes.iter().enumerate() {
            if size <= class_size {
                return self.pools[i].alloc();
            }
        }

        // Size too large for pool, allocate directly from system allocator
        let layout = alloc::alloc::Layout::from_size_align(size, 8).unwrap_or_else(|_| alloc::alloc::Layout::new::<u8>());
        // SAFETY: unsafe block required for low-level memory or hardware access
        let ptr = unsafe { alloc::alloc::alloc(layout) };
        if ptr.is_null() {
            ptr::null_mut()
        } else {
            self.stats.allocated.fetch_add(1, Ordering::Relaxed);
            ptr
        }
    }

    /// Free memory
    /// @param ptr: Pointer to memory
    /// @param size: Size of memory
    pub fn free(&self, ptr: *mut u8, size: usize) {
        if ptr.is_null() || size == 0 {
            return;
        }

        // Find appropriate pool
        for (i, &class_size) in self.size_classes.iter().enumerate() {
            if size <= class_size {
                self.pools[i].free(ptr);
                return;
            }
        }

        // Large allocation, free directly to system allocator
        let layout = alloc::alloc::Layout::from_size_align(size, 8).unwrap_or_else(|_| alloc::alloc::Layout::new::<u8>());
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe { alloc::alloc::dealloc(ptr, layout); }
        self.stats.allocated.fetch_sub(1, Ordering::Relaxed);
    }

    /// Get total capacity
    pub fn total_capacity(&self) -> usize {
        self.pools.iter().map(|p| p.capacity()).sum()
    }

    /// Get total allocated
    pub fn total_allocated(&self) -> usize {
        self.pools.iter().map(|p| p.allocated()).sum()
    }
}

/// Pool manager configuration
#[derive(Debug, Clone)]
pub struct PoolManagerConfig {
    /// Initial blocks per pool
    pub initial_blocks: usize,

    /// Maximum pool size
    pub max_pool_size: usize,

    /// Enable statistics
    pub enable_stats: bool,
}

impl Default for PoolManagerConfig {
    fn default() -> Self {
        Self {
            initial_blocks: 64,
            max_pool_size: 1024 * 1024, // 1MB
            enable_stats: true,
        }
    }
}

/// RAII guard for pool allocation
pub struct PoolBox<'a> {
    /// Pointer to memory
    ptr: *mut u8,

    /// Size
    size: usize,

    /// Pool reference
    pool: &'a MemoryPool,
}

impl<'a> PoolBox<'a> {
    /// Create new pool box
    /// @param pool: Memory pool
    /// @return: Pool box or None if allocation failed
    pub fn new(pool: &'a MemoryPool) -> Option<Self> {
        let ptr = pool.alloc();
        if ptr.is_null() {
            None
        } else {
            Some(Self {
                ptr,
                size: pool.block_size(),
                pool,
            })
        }
    }

    /// Get pointer
    pub fn as_ptr(&self) -> *mut u8 {
        self.ptr
    }

    /// Get size
    pub fn size(&self) -> usize {
        self.size
    }
}

impl<'a> Drop for PoolBox<'a> {
    fn drop(&mut self) {
        self.pool.free(self.ptr);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_pool() {
        let pool = MemoryPool::new(64, 16);

        let ptr1 = pool.alloc();
        let ptr2 = pool.alloc();
        let ptr3 = pool.alloc();

        assert!(!ptr1.is_null());
        assert!(!ptr2.is_null());
        assert!(!ptr3.is_null());
        assert_ne!(ptr1, ptr2);
        assert_ne!(ptr2, ptr3);

        assert_eq!(pool.allocated(), 3);

        pool.free(ptr1);
        pool.free(ptr2);
        pool.free(ptr3);

        assert_eq!(pool.allocated(), 0);
    }

    #[test]
    fn test_pool_manager() {
        let config = PoolManagerConfig::default();
        let manager = PoolManager::new(config);

        let ptr1 = manager.alloc(32);
        let ptr2 = manager.alloc(128);
        let ptr3 = manager.alloc(1024);

        assert!(!ptr1.is_null());
        assert!(!ptr2.is_null());
        assert!(!ptr3.is_null());

        manager.free(ptr1, 32);
        manager.free(ptr2, 128);
        manager.free(ptr3, 1024);
    }
}
