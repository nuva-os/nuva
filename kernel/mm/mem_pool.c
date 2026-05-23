/*
 * Nuva OS - Optimized Memory Pool Allocator
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

/**
 * Optimized Memory Pool Allocator
 *
 * This module implements a high-performance, O(1) memory pool allocator
 * for kernel critical paths. Features:
 * - O(1) allocation and deallocation
 * - Zero fragmentation
 * - Thread-safe with atomic operations
 * - Cache-friendly design
 */

#include <stdint.h>
#include <stddef.h>
#include <string.h>
#include <stdatomic.h>

/* Memory pool structure */
typedef struct {
    uint8_t *base;              /* Base address of the pool */
    size_t size;                /* Total pool size in bytes */
    size_t block_size;          /* Size of each block */
    size_t num_blocks;          /* Total number of blocks */
    atomic_uint *free_list;     /* Free list (array of block indices) */
    atomic_size_t free_count;   /* Number of free blocks */
    atomic_size_t alloc_count;  /* Number of allocated blocks */
    uint32_t magic;             /* Magic number for validation */
} mem_pool_t;

/* Pool magic number */
#define MEM_POOL_MAGIC 0x4D454D50  /* "MEMP" */

/* Pool flags */
#define POOL_FLAG_LOCKFREE  0x01
#define POOL_FLAG_ZEROINIT  0x02

/* Error codes */
#define MEM_POOL_SUCCESS     0
#define MEM_POOL_ERROR      -1
#define MEM_POOL_EXHAUSTED  -2
#define MEM_POOL_INVALID    -3

/**
 * Initialize a memory pool
 *
 * @param pool Pointer to pool structure
 * @param base Base address for the pool memory
 * @param size Total size of the pool
 * @param block_size Size of each block
 * @return 0 on success, negative on error
 */
int mem_pool_init(mem_pool_t *pool, void *base, size_t size, size_t block_size) {
    if (pool == NULL || base == NULL || size == 0 || block_size == 0) {
        return MEM_POOL_INVALID;
    }

    /* Align block size to 8 bytes */
    block_size = (block_size + 7) & ~7;

    /* Calculate number of blocks */
    size_t num_blocks = size / block_size;
    if (num_blocks == 0) {
        return MEM_POOL_INVALID;
    }

    /* Allocate free list at the beginning of the pool */
    size_t free_list_size = num_blocks * sizeof(atomic_uint);
    atomic_uint *free_list = (atomic_uint *)base;

    /* Adjust base and size for free list */
    uint8_t *adjusted_base = (uint8_t *)base + free_list_size;
    size_t adjusted_size = size - free_list_size;
    num_blocks = adjusted_size / block_size;

    /* Initialize free list */
    for (size_t i = 0; i < num_blocks; i++) {
        atomic_store(&free_list[i], (unsigned int)i);
    }

    /* Initialize pool structure */
    pool->base = adjusted_base;
    pool->size = adjusted_size;
    pool->block_size = block_size;
    pool->num_blocks = num_blocks;
    pool->free_list = free_list;
    atomic_store(&pool->free_count, num_blocks);
    atomic_store(&pool->alloc_count, 0);
    pool->magic = MEM_POOL_MAGIC;

    return MEM_POOL_SUCCESS;
}

/**
 * Allocate a block from the pool
 *
 * @param pool Pointer to pool structure
 * @return Pointer to allocated block, or NULL if pool is exhausted
 */
void *mem_pool_alloc(mem_pool_t *pool) {
    if (pool == NULL || pool->magic != MEM_POOL_MAGIC) {
        return NULL;
    }

    /* Check if pool has free blocks */
    size_t free_count = atomic_load(&pool->free_count);
    if (free_count == 0) {
        return NULL;
    }

    /* Atomically decrement free count and get index */
    size_t index = atomic_fetch_sub(&pool->free_count, 1) - 1;
    if (index >= pool->num_blocks) {
        /* Race condition: restore count and return NULL */
        atomic_fetch_add(&pool->free_count, 1);
        return NULL;
    }

    /* Get block index from free list */
    unsigned int block = atomic_load(&pool->free_list[index]);

    /* Update allocation count */
    atomic_fetch_add(&pool->alloc_count, 1);

    /* Return pointer to block */
    return pool->base + block * pool->block_size;
}

/**
 * Free a block back to the pool
 *
 * @param pool Pointer to pool structure
 * @param ptr Pointer to block to free
 * @return 0 on success, negative on error
 */
int mem_pool_free(mem_pool_t *pool, void *ptr) {
    if (pool == NULL || pool->magic != MEM_POOL_MAGIC) {
        return MEM_POOL_INVALID;
    }

    if (ptr == NULL) {
        return MEM_POOL_SUCCESS;
    }

    /* Check if pointer is within pool range */
    uint8_t *block = (uint8_t *)ptr;
    if (block < pool->base || block >= pool->base + pool->size) {
        return MEM_POOL_INVALID;
    }

    /* Calculate block index */
    size_t offset = block - pool->base;
    if (offset % pool->block_size != 0) {
        return MEM_POOL_INVALID;
    }
    unsigned int block_index = (unsigned int)(offset / pool->block_size);

    /* Atomically increment free count and get index */
    size_t index = atomic_fetch_add(&pool->free_count, 1);

    /* Check for overflow */
    if (index >= pool->num_blocks) {
        atomic_fetch_sub(&pool->free_count, 1);
        return MEM_POOL_ERROR;
    }

    /* Add block to free list */
    atomic_store(&pool->free_list[index], block_index);

    /* Update allocation count */
    atomic_fetch_sub(&pool->alloc_count, 1);

    return MEM_POOL_SUCCESS;
}

/**
 * Allocate and zero-initialize a block
 *
 * @param pool Pointer to pool structure
 * @return Pointer to allocated block, or NULL if pool is exhausted
 */
void *mem_pool_alloc_zero(mem_pool_t *pool) {
    void *ptr = mem_pool_alloc(pool);
    if (ptr != NULL) {
        memset(ptr, 0, pool->block_size);
    }
    return ptr;
}

/**
 * Get pool statistics
 *
 * @param pool Pointer to pool structure
 * @param free_count Output: number of free blocks
 * @param alloc_count Output: number of allocated blocks
 * @param total_count Output: total number of blocks
 */
void mem_pool_stats(mem_pool_t *pool, size_t *free_count, size_t *alloc_count, size_t *total_count) {
    if (pool == NULL || pool->magic != MEM_POOL_MAGIC) {
        if (free_count) *free_count = 0;
        if (alloc_count) *alloc_count = 0;
        if (total_count) *total_count = 0;
        return;
    }

    if (free_count) *free_count = atomic_load(&pool->free_count);
    if (alloc_count) *alloc_count = atomic_load(&pool->alloc_count);
    if (total_count) *total_count = pool->num_blocks;
}

/**
 * Check if pool is exhausted
 *
 * @param pool Pointer to pool structure
 * @return 1 if exhausted, 0 if not, negative on error
 */
int mem_pool_is_exhausted(mem_pool_t *pool) {
    if (pool == NULL || pool->magic != MEM_POOL_MAGIC) {
        return MEM_POOL_INVALID;
    }
    return atomic_load(&pool->free_count) == 0 ? 1 : 0;
}

/**
 * Get pool utilization
 *
 * @param pool Pointer to pool structure
 * @return Utilization percentage (0-100), or negative on error
 */
int mem_pool_utilization(mem_pool_t *pool) {
    if (pool == NULL || pool->magic != MEM_POOL_MAGIC) {
        return MEM_POOL_INVALID;
    }
    size_t alloc_count = atomic_load(&pool->alloc_count);
    return (int)((alloc_count * 100) / pool->num_blocks);
}

/* Multi-size pool allocator */
#define MAX_POOL_SIZES 8

typedef struct {
    mem_pool_t pools[MAX_POOL_SIZES];
    size_t num_pools;
    size_t size_classes[MAX_POOL_SIZES];
    atomic_size_t total_allocs;
    atomic_size_t total_frees;
} multi_pool_t;

/* Common size classes */
static const size_t DEFAULT_SIZE_CLASSES[MAX_POOL_SIZES] = {
    32,     /* 32 bytes */
    64,     /* 64 bytes */
    128,    /* 128 bytes */
    256,    /* 256 bytes */
    512,    /* 512 bytes */
    1024,   /* 1 KB */
    2048,   /* 2 KB */
    4096    /* 4 KB */
};

/**
 * Initialize a multi-size pool allocator
 *
 * @param multi Pointer to multi-pool structure
 * @param base Base address for all pools
 * @param total_size Total size for all pools
 * @return 0 on success, negative on error
 */
int multi_pool_init(multi_pool_t *multi, void *base, size_t total_size) {
    if (multi == NULL || base == NULL || total_size == 0) {
        return MEM_POOL_INVALID;
    }

    /* Calculate size per pool */
    size_t size_per_pool = total_size / MAX_POOL_SIZES;
    uint8_t *current_base = (uint8_t *)base;

    /* Initialize each pool */
    for (size_t i = 0; i < MAX_POOL_SIZES; i++) {
        int result = mem_pool_init(&multi->pools[i], current_base, size_per_pool, DEFAULT_SIZE_CLASSES[i]);
        if (result != MEM_POOL_SUCCESS) {
            return result;
        }
        current_base += size_per_pool;
        multi->size_classes[i] = DEFAULT_SIZE_CLASSES[i];
    }

    multi->num_pools = MAX_POOL_SIZES;
    atomic_store(&multi->total_allocs, 0);
    atomic_store(&multi->total_frees, 0);

    return MEM_POOL_SUCCESS;
}

/**
 * Allocate from multi-size pool
 *
 * @param multi Pointer to multi-pool structure
 * @param size Requested size
 * @return Pointer to allocated block, or NULL if no suitable pool has space
 */
void *multi_pool_alloc(multi_pool_t *multi, size_t size) {
    if (multi == NULL || size == 0) {
        return NULL;
    }

    /* Find suitable pool */
    for (size_t i = 0; i < multi->num_pools; i++) {
        if (size <= multi->size_classes[i]) {
            void *ptr = mem_pool_alloc(&multi->pools[i]);
            if (ptr != NULL) {
                atomic_fetch_add(&multi->total_allocs, 1);
                return ptr;
            }
        }
    }

    return NULL;
}

/**
 * Free from multi-size pool
 *
 * @param multi Pointer to multi-pool structure
 * @param ptr Pointer to free
 * @param size Size of the allocation
 * @return 0 on success, negative on error
 */
int multi_pool_free(multi_pool_t *multi, void *ptr, size_t size) {
    if (multi == NULL || ptr == NULL) {
        return MEM_POOL_INVALID;
    }

    /* Find the pool this allocation came from */
    for (size_t i = 0; i < multi->num_pools; i++) {
        if (size <= multi->size_classes[i]) {
            int result = mem_pool_free(&multi->pools[i], ptr);
            if (result == MEM_POOL_SUCCESS) {
                atomic_fetch_add(&multi->total_frees, 1);
                return MEM_POOL_SUCCESS;
            }
        }
    }

    return MEM_POOL_INVALID;
}

/**
 * Get multi-pool statistics
 *
 * @param multi Pointer to multi-pool structure
 * @param total_allocs Output: total allocations
 * @param total_frees Output: total frees
 */
void multi_pool_stats(multi_pool_t *multi, size_t *total_allocs, size_t *total_frees) {
    if (multi == NULL) {
        if (total_allocs) *total_allocs = 0;
        if (total_frees) *total_frees = 0;
        return;
    }

    if (total_allocs) *total_allocs = atomic_load(&multi->total_allocs);
    if (total_frees) *total_frees = atomic_load(&multi->total_frees);
}

/* Per-CPU cache for even faster allocation */
typedef struct {
    void *local_cache[64];     /* Local cache of free blocks */
    size_t cache_count;        /* Number of cached blocks */
    size_t cache_capacity;     /* Cache capacity */
    mem_pool_t *backing_pool;  /* Backing pool */
} percpu_cache_t;

/**
 * Initialize per-CPU cache
 *
 * @param cache Pointer to cache structure
 * @param pool Backing pool
 * @return 0 on success, negative on error
 */
int percpu_cache_init(percpu_cache_t *cache, mem_pool_t *pool) {
    if (cache == NULL || pool == NULL) {
        return MEM_POOL_INVALID;
    }

    cache->cache_count = 0;
    cache->cache_capacity = 64;
    cache->backing_pool = pool;
    memset(cache->local_cache, 0, sizeof(cache->local_cache));

    return MEM_POOL_SUCCESS;
}

/**
 * Allocate from per-CPU cache
 *
 * @param cache Pointer to cache structure
 * @return Pointer to allocated block, or NULL if exhausted
 */
void *percpu_cache_alloc(percpu_cache_t *cache) {
    if (cache == NULL) {
        return NULL;
    }

    /* Try local cache first */
    if (cache->cache_count > 0) {
        cache->cache_count--;
        return cache->local_cache[cache->cache_count];
    }

    /* Refill from backing pool */
    for (size_t i = 0; i < cache->cache_capacity / 2; i++) {
        void *ptr = mem_pool_alloc(cache->backing_pool);
        if (ptr == NULL) {
            break;
        }
        cache->local_cache[cache->cache_count++] = ptr;
    }

    /* Try again */
    if (cache->cache_count > 0) {
        cache->cache_count--;
        return cache->local_cache[cache->cache_count];
    }

    return NULL;
}

/**
 * Free to per-CPU cache
 *
 * @param cache Pointer to cache structure
 * @param ptr Pointer to free
 * @return 0 on success, negative on error
 */
int percpu_cache_free(percpu_cache_t *cache, void *ptr) {
    if (cache == NULL || ptr == NULL) {
        return MEM_POOL_INVALID;
    }

    /* Add to local cache if not full */
    if (cache->cache_count < cache->cache_capacity) {
        cache->local_cache[cache->cache_count++] = ptr;
        return MEM_POOL_SUCCESS;
    }

    /* Flush half to backing pool */
    for (size_t i = 0; i < cache->cache_capacity / 2; i++) {
        mem_pool_free(cache->backing_pool, cache->local_cache[--cache->cache_count]);
    }

    /* Add to local cache */
    cache->local_cache[cache->cache_count++] = ptr;

    return MEM_POOL_SUCCESS;
}
