/*
 * Nuva OS - Memory Pool Allocator Performance Tests
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
 * Memory Pool Allocator Performance Tests
 *
 * This module provides comprehensive benchmarks for the memory pool allocator,
 * measuring allocation/free latency, throughput, and fragmentation.
 */

#include <stdint.h>
#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include "mem_pool.c"

/* Performance test configuration */
#define TEST_ITERATIONS    1000000
#define TEST_WARMUP        10000
#define TEST_POOL_SIZE     (16 * 1024 * 1024)  /* 16 MB */
#define TEST_BLOCK_SIZE    64

/* Get high-resolution timestamp (nanoseconds) */
static inline uint64_t get_timestamp_ns(void) {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return (uint64_t)ts.tv_sec * 1000000000ULL + (uint64_t)ts.tv_nsec;
}

/* Calculate elapsed time in nanoseconds */
static inline uint64_t elapsed_ns(uint64_t start, uint64_t end) {
    return end - start;
}

/* Test result structure */
typedef struct {
    const char *name;
    uint64_t total_ns;
    uint64_t min_ns;
    uint64_t max_ns;
    double avg_ns;
    double ops_per_sec;
    size_t iterations;
} test_result_t;

/* Print test result */
static void print_result(const test_result_t *result) {
    printf("=== %s ===\n", result->name);
    printf("  Iterations: %zu\n", result->iterations);
    printf("  Total time: %lu ns (%.3f ms)\n", 
           (unsigned long)result->total_ns, 
           result->total_ns / 1000000.0);
    printf("  Min latency: %lu ns\n", (unsigned long)result->min_ns);
    printf("  Max latency: %lu ns\n", (unsigned long)result->max_ns);
    printf("  Avg latency: %.2f ns\n", result->avg_ns);
    printf("  Throughput: %.2f ops/sec\n", result->ops_per_sec);
    printf("\n");
}

/**
 * Test 1: Single-threaded allocation latency
 */
static void test_alloc_latency(mem_pool_t *pool, test_result_t *result) {
    uint64_t start, end, latency;
    uint64_t min_lat = UINT64_MAX;
    uint64_t max_lat = 0;
    uint64_t total = 0;
    void *ptrs[TEST_ITERATIONS];
    size_t count = 0;

    result->name = "Allocation Latency";
    result->iterations = TEST_ITERATIONS;

    /* Warmup */
    for (size_t i = 0; i < TEST_WARMUP; i++) {
        void *p = mem_pool_alloc(pool);
        if (p) mem_pool_free(pool, p);
    }

    /* Measure allocation latency */
    for (size_t i = 0; i < TEST_ITERATIONS; i++) {
        start = get_timestamp_ns();
        ptrs[i] = mem_pool_alloc(pool);
        end = get_timestamp_ns();

        if (ptrs[i] != NULL) {
            latency = elapsed_ns(start, end);
            total += latency;
            if (latency < min_lat) min_lat = latency;
            if (latency > max_lat) max_lat = latency;
            count++;
        }
    }

    /* Free all allocated blocks */
    for (size_t i = 0; i < TEST_ITERATIONS; i++) {
        if (ptrs[i] != NULL) {
            mem_pool_free(pool, ptrs[i]);
        }
    }

    result->total_ns = total;
    result->min_ns = min_lat;
    result->max_ns = max_lat;
    result->avg_ns = (double)total / count;
    result->ops_per_sec = (double)count * 1000000000.0 / total;
}

/**
 * Test 2: Single-threaded free latency
 */
static void test_free_latency(mem_pool_t *pool, test_result_t *result) {
    uint64_t start, end, latency;
    uint64_t min_lat = UINT64_MAX;
    uint64_t max_lat = 0;
    uint64_t total = 0;
    void *ptrs[TEST_ITERATIONS];
    size_t count = 0;

    result->name = "Free Latency";
    result->iterations = TEST_ITERATIONS;

    /* Pre-allocate all blocks */
    for (size_t i = 0; i < TEST_ITERATIONS; i++) {
        ptrs[i] = mem_pool_alloc(pool);
    }

    /* Warmup */
    for (size_t i = 0; i < TEST_WARMUP; i++) {
        void *p = mem_pool_alloc(pool);
        if (p) mem_pool_free(pool, p);
    }

    /* Measure free latency */
    for (size_t i = 0; i < TEST_ITERATIONS; i++) {
        if (ptrs[i] == NULL) continue;

        start = get_timestamp_ns();
        mem_pool_free(pool, ptrs[i]);
        end = get_timestamp_ns();

        latency = elapsed_ns(start, end);
        total += latency;
        if (latency < min_lat) min_lat = latency;
        if (latency > max_lat) max_lat = latency;
        count++;
    }

    result->total_ns = total;
    result->min_ns = min_lat;
    result->max_ns = max_lat;
    result->avg_ns = (double)total / count;
    result->ops_per_sec = (double)count * 1000000000.0 / total;
}

/**
 * Test 3: Alloc/free cycle latency
 */
static void test_cycle_latency(mem_pool_t *pool, test_result_t *result) {
    uint64_t start, end, latency;
    uint64_t min_lat = UINT64_MAX;
    uint64_t max_lat = 0;
    uint64_t total = 0;
    void *ptr;

    result->name = "Alloc/Free Cycle Latency";
    result->iterations = TEST_ITERATIONS;

    /* Warmup */
    for (size_t i = 0; i < TEST_WARMUP; i++) {
        ptr = mem_pool_alloc(pool);
        if (ptr) mem_pool_free(pool, ptr);
    }

    /* Measure alloc+free latency */
    for (size_t i = 0; i < TEST_ITERATIONS; i++) {
        start = get_timestamp_ns();
        ptr = mem_pool_alloc(pool);
        if (ptr) {
            mem_pool_free(pool, ptr);
        }
        end = get_timestamp_ns();

        latency = elapsed_ns(start, end);
        total += latency;
        if (latency < min_lat) min_lat = latency;
        if (latency > max_lat) max_lat = latency;
    }

    result->total_ns = total;
    result->min_ns = min_lat;
    result->max_ns = max_lat;
    result->avg_ns = (double)total / TEST_ITERATIONS;
    result->ops_per_sec = (double)TEST_ITERATIONS * 1000000000.0 / total;
}

/**
 * Test 4: Throughput test
 */
static void test_throughput(mem_pool_t *pool, test_result_t *result) {
    uint64_t start, end;
    void *ptrs[1024];
    size_t ops = 0;

    result->name = "Throughput";
    result->iterations = TEST_ITERATIONS;

    /* Warmup */
    for (size_t i = 0; i < TEST_WARMUP; i++) {
        void *p = mem_pool_alloc(pool);
        if (p) mem_pool_free(pool, p);
    }

    start = get_timestamp_ns();

    /* Perform many alloc/free operations */
    for (size_t i = 0; i < TEST_ITERATIONS / 1024; i++) {
        /* Allocate batch */
        for (size_t j = 0; j < 1024; j++) {
            ptrs[j] = mem_pool_alloc(pool);
            if (ptrs[j]) ops++;
        }

        /* Free batch */
        for (size_t j = 0; j < 1024; j++) {
            if (ptrs[j]) {
                mem_pool_free(pool, ptrs[j]);
                ops++;
            }
        }
    }

    end = get_timestamp_ns();

    result->total_ns = elapsed_ns(start, end);
    result->min_ns = 0;
    result->max_ns = 0;
    result->avg_ns = (double)result->total_ns / ops;
    result->ops_per_sec = (double)ops * 1000000000.0 / result->total_ns;
}

/**
 * Test 5: Fragmentation test
 */
static void test_fragmentation(mem_pool_t *pool, test_result_t *result) {
    void *ptrs[1024];
    size_t allocated = 0;
    size_t freed = 0;
    size_t fragmentation_events = 0;

    result->name = "Fragmentation";
    result->iterations = 1024;

    /* Allocate all blocks */
    for (size_t i = 0; i < 1024; i++) {
        ptrs[i] = mem_pool_alloc(pool);
        if (ptrs[i]) allocated++;
    }

    /* Free every other block */
    for (size_t i = 0; i < 1024; i += 2) {
        if (ptrs[i]) {
            mem_pool_free(pool, ptrs[i]);
            ptrs[i] = NULL;
            freed++;
        }
    }

    /* Try to allocate - should succeed without fragmentation */
    for (size_t i = 0; i < 1024; i += 2) {
        if (ptrs[i] == NULL) {
            ptrs[i] = mem_pool_alloc(pool);
            if (ptrs[i] == NULL) {
                fragmentation_events++;
            }
        }
    }

    /* Free all */
    for (size_t i = 0; i < 1024; i++) {
        if (ptrs[i]) {
            mem_pool_free(pool, ptrs[i]);
        }
    }

    result->total_ns = 0;
    result->min_ns = 0;
    result->max_ns = fragmentation_events;
    result->avg_ns = 0;
    result->ops_per_sec = 0;

    printf("=== Fragmentation Test ===\n");
    printf("  Allocated: %zu\n", allocated);
    printf("  Freed: %zu\n", freed);
    printf("  Fragmentation events: %zu\n", fragmentation_events);
    printf("  Result: %s\n", fragmentation_events == 0 ? "PASS (zero fragmentation)" : "FAIL");
    printf("\n");
}

/**
 * Test 6: Multi-pool performance
 */
static void test_multi_pool(multi_pool_t *multi, test_result_t *result) {
    uint64_t start, end;
    void *ptrs[TEST_ITERATIONS];
    size_t sizes[] = {32, 64, 128, 256, 512, 1024, 2048, 4096};
    size_t num_sizes = sizeof(sizes) / sizeof(sizes[0]);
    size_t ops = 0;

    result->name = "Multi-Pool Throughput";
    result->iterations = TEST_ITERATIONS;

    start = get_timestamp_ns();

    /* Allocate with various sizes */
    for (size_t i = 0; i < TEST_ITERATIONS; i++) {
        size_t size = sizes[i % num_sizes];
        ptrs[i] = multi_pool_alloc(multi, size);
        if (ptrs[i]) ops++;
    }

    /* Free all */
    for (size_t i = 0; i < TEST_ITERATIONS; i++) {
        if (ptrs[i]) {
            size_t size = sizes[i % num_sizes];
            multi_pool_free(multi, ptrs[i], size);
            ops++;
        }
    }

    end = get_timestamp_ns();

    result->total_ns = elapsed_ns(start, end);
    result->min_ns = 0;
    result->max_ns = 0;
    result->avg_ns = (double)result->total_ns / ops;
    result->ops_per_sec = (double)ops * 1000000000.0 / result->total_ns;
}

/**
 * Test 7: Per-CPU cache performance
 */
static void test_percpu_cache(percpu_cache_t *cache, test_result_t *result) {
    uint64_t start, end, latency;
    uint64_t min_lat = UINT64_MAX;
    uint64_t max_lat = 0;
    uint64_t total = 0;
    void *ptr;

    result->name = "Per-CPU Cache Latency";
    result->iterations = TEST_ITERATIONS;

    /* Warmup */
    for (size_t i = 0; i < TEST_WARMUP; i++) {
        ptr = percpu_cache_alloc(cache);
        if (ptr) percpu_cache_free(cache, ptr);
    }

    /* Measure alloc+free latency */
    for (size_t i = 0; i < TEST_ITERATIONS; i++) {
        start = get_timestamp_ns();
        ptr = percpu_cache_alloc(cache);
        if (ptr) {
            percpu_cache_free(cache, ptr);
        }
        end = get_timestamp_ns();

        latency = elapsed_ns(start, end);
        total += latency;
        if (latency < min_lat) min_lat = latency;
        if (latency > max_lat) max_lat = latency;
    }

    result->total_ns = total;
    result->min_ns = min_lat;
    result->max_ns = max_lat;
    result->avg_ns = (double)total / TEST_ITERATIONS;
    result->ops_per_sec = (double)TEST_ITERATIONS * 1000000000.0 / total;
}

/**
 * Main test runner
 */
int main(void) {
    printf("Nuva OS Memory Pool Allocator Performance Tests\n");
    printf("================================================\n\n");

    /* Allocate memory for pools */
    uint8_t *pool_mem = malloc(TEST_POOL_SIZE);
    uint8_t *multi_mem = malloc(TEST_POOL_SIZE);
    if (!pool_mem || !multi_mem) {
        printf("Failed to allocate test memory\n");
        return 1;
    }

    /* Initialize pool */
    mem_pool_t pool;
    if (mem_pool_init(&pool, pool_mem, TEST_POOL_SIZE, TEST_BLOCK_SIZE) != 0) {
        printf("Failed to initialize memory pool\n");
        free(pool_mem);
        free(multi_mem);
        return 1;
    }

    /* Initialize multi-pool */
    multi_pool_t multi;
    if (multi_pool_init(&multi, multi_mem, TEST_POOL_SIZE) != 0) {
        printf("Failed to initialize multi-pool\n");
        free(pool_mem);
        free(multi_mem);
        return 1;
    }

    /* Initialize per-CPU cache */
    percpu_cache_t cache;
    if (percpu_cache_init(&cache, &pool) != 0) {
        printf("Failed to initialize per-CPU cache\n");
        free(pool_mem);
        free(multi_mem);
        return 1;
    }

    /* Run tests */
    test_result_t result;

    printf("--- Single Pool Tests ---\n\n");
    
    test_alloc_latency(&pool, &result);
    print_result(&result);

    test_free_latency(&pool, &result);
    print_result(&result);

    test_cycle_latency(&pool, &result);
    print_result(&result);

    test_throughput(&pool, &result);
    print_result(&result);

    test_fragmentation(&pool, &result);

    printf("--- Multi-Pool Tests ---\n\n");

    test_multi_pool(&multi, &result);
    print_result(&result);

    printf("--- Per-CPU Cache Tests ---\n\n");

    test_percpu_cache(&cache, &result);
    print_result(&result);

    /* Print summary */
    printf("=== Summary ===\n");
    printf("Target: Allocation < 10ns, Free < 10ns\n");
    printf("Target: Zero fragmentation\n");
    printf("Target: Throughput > 10M ops/sec\n");
    printf("\n");

    /* Cleanup */
    free(pool_mem);
    free(multi_mem);

    printf("All tests completed.\n");
    return 0;
}
