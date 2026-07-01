/*
 * Nuva OS - Kernel - Slab Allocator
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

use core::ptr;
use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/** L1 cache line size for SLAB object alignment */
const CACHE_LINE_SIZE: usize = 64;

/**
 * Slab cache structure.
 *
 * Manages allocation of fixed-size objects from slab pages.
 * Objects are aligned to L1 cache line size (64 bytes) to
 * prevent false sharing between frequently accessed objects
 * on different CPUs.
 */
#[repr(C, align(64))]
pub struct KmemCache {
    /** Cache name for debugging */
    pub name: &'static str,
    /** Size of each object (aligned to cache line) */
    pub object_size: usize,
    /** Number of objects */
    pub num_objects: usize,
    /** Free object list head */
    pub free_list: *mut u8,
    /** Number of allocated objects */
    pub active: AtomicU32,
    /** Total number of objects */
    pub total: AtomicU32,
}

impl KmemCache {
    /**
     * Create a new slab cache.
     *
     * Object size is aligned up to the L1 cache line size (64 bytes)
     * to prevent false sharing between adjacent objects accessed by
     * different CPUs.
     *
     * @param name: Cache name for debugging
     * @param object_size: Size of objects in this cache (will be cache-line aligned)
     * @return New KmemCache instance
     */
    pub const fn new(name: &'static str, object_size: usize) -> Self {
        // Align object size to cache line boundary
        let aligned_size =
            ((object_size + CACHE_LINE_SIZE - 1) / CACHE_LINE_SIZE) * CACHE_LINE_SIZE;
        KmemCache {
            name,
            object_size: aligned_size,
            num_objects: 0,
            free_list: ptr::null_mut(),
            active: AtomicU32::new(0),
            total: AtomicU32::new(0),
        }
    }

    /// Allocate an object from the cache
    /// @return Pointer to allocated object, or null on failure
    pub fn alloc(&mut self) -> *mut u8 {
        if self.free_list.is_null() {
            // Need to allocate a new slab
            if !self.grow() {
                return ptr::null_mut();
            }
        }

        // Pop from free list
        let obj = self.free_list;
        // SAFETY: The free_list pointer was set during grow() or from a
        // previous free() call. It points to a valid slab object. We read
        // the first word of the object which stores the next free pointer
        // (freelist linkage pattern). This is safe because:
        // 1. obj is non-null (checked above)
        // 2. obj points to a valid slab object (invariant of free_list)
        // 3. The first pointer-sized word stores the next pointer
        // SAFETY: obj is a valid non-null pointer to a slab object whose
        // first word stores the freelist next pointer. Reading it as a
        // *mut u8 is valid because slab objects are pointer-aligned.
        unsafe {
            self.free_list = *(obj as *const *mut u8);
        }

        self.active.fetch_add(1, Ordering::Relaxed);
        obj
    }

    /// Free an object back to the cache
    /// @param obj: Pointer to object to free
    pub fn free(&mut self, obj: *mut u8) {
        if obj.is_null() {
            return;
        }

        // Push to free list
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            *(obj as *mut *mut u8) = self.free_list;
        }
        self.free_list = obj;

        self.active.fetch_sub(1, Ordering::Relaxed);
    }

    /// Grow the cache by allocating a new slab page
    /// @return true on success, false on failure
    fn grow(&mut self) -> bool {
        // Allocate a page from buddy allocator
        let page = crate::kernel::mm::buddy::alloc_page();
        if page.is_null() {
            log_warn!("Slab: failed to allocate page for cache {}", self.name);
            return false;
        }

        // Get physical address of the page
        let page_phys = crate::kernel::mm::buddy::buddy().get_phys_addr(page);

        // Calculate number of objects that fit in a page
        let num = 4096 / self.object_size;

        // Initialize free list
        for i in 0..num {
            let obj = (page_phys + (i * self.object_size) as u64) as *mut u8;
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                // Add object to free list
                *(obj as *mut *mut u8) = self.free_list;
            }
            self.free_list = obj;
        }

        self.total.fetch_add(num as u32, Ordering::Relaxed);
        log_debug!("Slab: added {} objects to cache {}", num, self.name);
        true
    }
}

/// Slab allocator
/// Manages multiple slab caches for different object sizes.
pub struct SlabAllocator {
    caches: [Option<KmemCache>; 16],
}

impl SlabAllocator {
    pub const fn new() -> Self {
        SlabAllocator {
            caches: [
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None,
            ],
        }
    }

    /// Create a new cache
    /// @param name: Cache name
    /// @param size: Object size
    /// @return Cache index on success, None on failure
    pub fn create_cache(&mut self, name: &'static str, size: usize) -> Option<usize> {
        for (i, cache) in self.caches.iter_mut().enumerate() {
            if cache.is_none() {
                *cache = Some(KmemCache::new(name, size));
                return Some(i);
            }
        }
        None
    }

    /// Allocate an object from a cache
    /// @param cache_idx: Cache index
    /// @return Pointer to allocated object, or null on failure
    pub fn alloc(&mut self, cache_idx: usize) -> *mut u8 {
        if let Some(ref mut cache) = self.caches[cache_idx] {
            cache.alloc()
        } else {
            ptr::null_mut()
        }
    }

    /// Free an object to a cache
    /// @param cache_idx: Cache index
    /// @param obj: Pointer to object to free
    pub fn free(&mut self, cache_idx: usize, obj: *mut u8) {
        if let Some(ref mut cache) = self.caches[cache_idx] {
            cache.free(obj);
        }
    }
}

/// Global slab allocator instance
static SLAB_ALLOCATOR: crate::sync_oncelock::OnceLock<SlabAllocator> = crate::sync_oncelock::OnceLock::new();

/// Get reference to global slab allocator
pub fn slab_allocator() -> &'static SlabAllocator {
    SLAB_ALLOCATOR.get_or_init(SlabAllocator::new)
}

/// Initialize slab allocator
pub fn init_slab() {
    log_info!("Slab allocator initialized");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kmem_cache_new() {
        let cache = KmemCache::new("test_cache", 64);

        assert_eq!(cache.name, "test_cache");
        assert_eq!(cache.object_size, 64);
        assert_eq!(cache.num_objects, 0);
        assert_eq!(cache.active.load(Ordering::Relaxed), 0);
        assert_eq!(cache.total.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_kmem_cache_small_objects() {
        let cache = KmemCache::new("small", 16);

        assert_eq!(cache.object_size, 16);
    }

    #[test]
    fn test_kmem_cache_large_objects() {
        let cache = KmemCache::new("large", 1024);

        assert_eq!(cache.object_size, 1024);
    }

    #[test]
    fn test_slab_allocator_new() {
        let allocator = SlabAllocator::new();

        // All caches should be None
        for i in 0..16 {
            assert!(allocator.caches[i].is_none());
        }
    }

    #[test]
    fn test_slab_allocator_create_cache() {
        let mut allocator = SlabAllocator::new();

        let idx1 = allocator.create_cache("cache1", 32);
        assert!(idx1.is_some());
        assert_eq!(idx1.unwrap(), 0);

        let idx2 = allocator.create_cache("cache2", 64);
        assert!(idx2.is_some());
        assert_eq!(idx2.unwrap(), 1);

        let idx3 = allocator.create_cache("cache3", 128);
        assert!(idx3.is_some());
        assert_eq!(idx3.unwrap(), 2);
    }

    #[test]
    fn test_slab_allocator_create_cache_full() {
        let mut allocator = SlabAllocator::new();

        // Fill all caches
        for i in 0..16 {
            let idx = allocator.create_cache("cache", 32);
            assert!(idx.is_some());
        }

        // Creating another should fail
        let idx = allocator.create_cache("overflow", 32);
        assert!(idx.is_none());
    }

    #[test]
    fn test_slab_allocator_alloc_invalid_cache() {
        let mut allocator = SlabAllocator::new();

        // Allocating from non-existent cache
        let ptr = allocator.alloc(0);
        assert!(ptr.is_null());
    }

    #[test]
    fn test_slab_allocator_free_null() {
        let mut allocator = SlabAllocator::new();
        allocator.create_cache("test", 32);

        // Freeing null should not panic
        allocator.free(0, ptr::null_mut());
    }

    #[test]
    fn test_kmem_cache_free_null() {
        let mut cache = KmemCache::new("test", 32);

        // Freeing null should not panic
        cache.free(ptr::null_mut());
    }

    #[test]
    fn test_kmem_cache_active_count() {
        let mut cache = KmemCache::new("test", 32);

        assert_eq!(cache.active.load(Ordering::Relaxed), 0);

        // Simulate allocation
        cache.active.fetch_add(1, Ordering::Relaxed);
        assert_eq!(cache.active.load(Ordering::Relaxed), 1);

        cache.active.fetch_add(1, Ordering::Relaxed);
        assert_eq!(cache.active.load(Ordering::Relaxed), 2);

        // Simulate free
        cache.active.fetch_sub(1, Ordering::Relaxed);
        assert_eq!(cache.active.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_kmem_cache_total_count() {
        let mut cache = KmemCache::new("test", 32);

        assert_eq!(cache.total.load(Ordering::Relaxed), 0);

        // Simulate growth
        cache.total.fetch_add(128, Ordering::Relaxed);
        assert_eq!(cache.total.load(Ordering::Relaxed), 128);
    }

    #[test]
    fn test_slab_allocator_multiple_caches() {
        let mut allocator = SlabAllocator::new();

        // Create caches of different sizes
        let sizes = [16, 32, 64, 128, 256, 512, 1024, 2048];
        let mut indices = Vec::new();

        for (i, &size) in sizes.iter().enumerate() {
            let name = match i {
                0 => "size_16",
                1 => "size_32",
                2 => "size_64",
                3 => "size_128",
                4 => "size_256",
                5 => "size_512",
                6 => "size_1024",
                _ => "size_2048",
            };
            let idx = allocator.create_cache(name, size);
            assert!(idx.is_some());
            indices.push(idx.unwrap());
        }

        // Verify all indices are different
        for i in 0..indices.len() {
            for j in (i + 1)..indices.len() {
                assert_ne!(indices[i], indices[j]);
            }
        }
    }

    #[test]
    fn test_kmem_cache_object_sizes() {
        // Test various object sizes
        let sizes = [8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096];

        for &size in &sizes {
            let cache = KmemCache::new("test", size);
            assert_eq!(cache.object_size, size);
        }
    }

    #[test]
    fn test_slab_allocator_cache_names() {
        let mut allocator = SlabAllocator::new();

        allocator.create_cache("task_struct", 512);
        allocator.create_cache("inode", 256);
        allocator.create_cache("dentry", 128);

        // Verify caches were created
        assert!(allocator.caches[0].is_some());
        assert!(allocator.caches[1].is_some());
        assert!(allocator.caches[2].is_some());

        // Verify names
        assert_eq!(allocator.caches[0].as_ref().unwrap().name, "task_struct");
        assert_eq!(allocator.caches[1].as_ref().unwrap().name, "inode");
        assert_eq!(allocator.caches[2].as_ref().unwrap().name, "dentry");
    }
}
