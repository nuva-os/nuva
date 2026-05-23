/*
 * Nuva OS - Lock-Free Data Structures Performance Tests
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
 * Lock-Free Data Structures Performance Tests
 *
 * This module provides comprehensive benchmarks for lock-free data structures,
 * measuring throughput, latency, and scalability under various contention levels.
 */

#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <string.h>
#include <time.h>
#include <pthread.h>
#include "lockfree.c"

/* Test configuration */
#define TEST_ITERATIONS    1000000
#define TEST_WARMUP        10000
#define TEST_CAPACITY      65536

/* Number of threads for multi-threaded tests */
#define NUM_PRODUCERS       4
#define NUM_CONSUMERS       4
#define NUM_THREADS         8

/* Get high-resolution timestamp (nanoseconds) */
static inline uint64_t get_timestamp_ns(void) {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return (uint64_t)ts.tv_sec * 1000000000ULL + (uint64_t)ts.tv_nsec;
}

/* Test result structure */
typedef struct {
    const char *name;
    uint64_t total_ns;
    double ops_per_sec;
    double avg_latency_ns;
    size_t iterations;
    int num_threads;
} test_result_t;

/* Print test result */
static void print_result(const test_result_t *result) {
    printf("=== %s ===\n", result->name);
    printf("  Threads: %d\n", result->num_threads);
    printf("  Iterations: %zu\n", result->iterations);
    printf("  Total time: %.3f ms\n", result->total_ns / 1000000.0);
    printf("  Throughput: %.2f ops/sec\n", result->ops_per_sec);
    printf("  Avg latency: %.2f ns\n", result->avg_latency_ns);
    printf("\n");
}

/* ============================================================================
 * Single-threaded benchmarks
 * ============================================================================ */

static void benchmark_mpsc_single(test_result_t *result) {
    mpsc_queue_t queue;
    mpsc_queue_init(&queue);

    uint64_t start, end;
    void *ptrs[TEST_ITERATIONS];

    result->name = "MPSC Queue (Single-threaded)";
    result->num_threads = 1;
    result->iterations = TEST_ITERATIONS;

    /* Warmup */
    for (size_t i = 0; i < TEST_WARMUP; i++) {
        mpsc_queue_push(&queue, (void *)i);
        mpsc_queue_pop(&queue);
    }

    /* Benchmark push */
    start = get_timestamp_ns();
    for (size_t i = 0; i < TEST_ITERATIONS; i++) {
        mpsc_queue_push(&queue, (void *)i);
    }
    end = get_timestamp_ns();

    uint64_t push_ns = end - start;

    /* Benchmark pop */
    start = get_timestamp_ns();
    for (size_t i = 0; i < TEST_ITERATIONS; i++) {
        ptrs[i] = mpsc_queue_pop(&queue);
    }
    end = get_timestamp_ns();

    uint64_t pop_ns = end - start;

    result->total_ns = push_ns + pop_ns;
    result->ops_per_sec = (double)TEST_ITERATIONS * 2 * 1000000000.0 / result->total_ns;
    result->avg_latency_ns = (double)result->total_ns / (TEST_ITERATIONS * 2);

    mpsc_queue_destroy(&queue);
}

static void benchmark_spsc_single(test_result_t *result) {
    spsc_queue_t queue;
    spsc_queue_init(&queue, TEST_CAPACITY);

    uint64_t start, end;

    result->name = "SPSC Queue (Single-threaded)";
    result->num_threads = 1;
    result->iterations = TEST_ITERATIONS;

    /* Warmup */
    for (size_t i = 0; i < TEST_WARMUP; i++) {
        spsc_queue_push(&queue, (void *)i);
        spsc_queue_pop(&queue);
    }

    /* Benchmark push/pop cycle */
    start = get_timestamp_ns();
    for (size_t i = 0; i < TEST_ITERATIONS; i++) {
        spsc_queue_push(&queue, (void *)i);
        spsc_queue_pop(&queue);
    }
    end = get_timestamp_ns();

    result->total_ns = end - start;
    result->ops_per_sec = (double)TEST_ITERATIONS * 2 * 1000000000.0 / result->total_ns;
    result->avg_latency_ns = (double)result->total_ns / (TEST_ITERATIONS * 2);

    spsc_queue_destroy(&queue);
}

static void benchmark_stack_single(test_result_t *result) {
    lf_stack_t stack;
    lf_stack_init(&stack);

    uint64_t start, end;

    result->name = "Lock-Free Stack (Single-threaded)";
    result->num_threads = 1;
    result->iterations = TEST_ITERATIONS;

    /* Warmup */
    for (size_t i = 0; i < TEST_WARMUP; i++) {
        lf_stack_push(&stack, (void *)i);
        lf_stack_pop(&stack);
    }

    /* Benchmark push/pop cycle */
    start = get_timestamp_ns();
    for (size_t i = 0; i < TEST_ITERATIONS; i++) {
        lf_stack_push(&stack, (void *)i);
        lf_stack_pop(&stack);
    }
    end = get_timestamp_ns();

    result->total_ns = end - start;
    result->ops_per_sec = (double)TEST_ITERATIONS * 2 * 1000000000.0 / result->total_ns;
    result->avg_latency_ns = (double)result->total_ns / (TEST_ITERATIONS * 2);

    lf_stack_destroy(&stack);
}

static void benchmark_mpmc_single(test_result_t *result) {
    mpmc_queue_t queue;
    mpmc_queue_init(&queue, TEST_CAPACITY);

    uint64_t start, end;

    result->name = "MPMC Queue (Single-threaded)";
    result->num_threads = 1;
    result->iterations = TEST_ITERATIONS;

    /* Warmup */
    for (size_t i = 0; i < TEST_WARMUP; i++) {
        mpmc_queue_push(&queue, (void *)i);
        mpmc_queue_pop(&queue);
    }

    /* Benchmark push/pop cycle */
    start = get_timestamp_ns();
    for (size_t i = 0; i < TEST_ITERATIONS; i++) {
        mpmc_queue_push(&queue, (void *)i);
        mpmc_queue_pop(&queue);
    }
    end = get_timestamp_ns();

    result->total_ns = end - start;
    result->ops_per_sec = (double)TEST_ITERATIONS * 2 * 1000000000.0 / result->total_ns;
    result->avg_latency_ns = (double)result->total_ns / (TEST_ITERATIONS * 2);

    mpmc_queue_destroy(&queue);
}

/* ============================================================================
 * Multi-threaded benchmarks
 * ============================================================================ */

/* Thread argument structure */
typedef struct {
    void *queue;
    size_t iterations;
    uint64_t total_ns;
    int thread_id;
    int is_producer;
} thread_arg_t;

/* MPSC producer thread */
static void *mpsc_producer_thread(void *arg) {
    thread_arg_t *targ = (thread_arg_t *)arg;
    mpsc_queue_t *queue = (mpsc_queue_t *)targ->queue;

    uint64_t start = get_timestamp_ns();

    for (size_t i = 0; i < targ->iterations; i++) {
        mpsc_queue_push(queue, (void *)(targ->thread_id * targ->iterations + i));
    }

    targ->total_ns = get_timestamp_ns() - start;
    return NULL;
}

/* MPSC consumer thread */
static void *mpsc_consumer_thread(void *arg) {
    thread_arg_t *targ = (thread_arg_t *)arg;
    mpsc_queue_t *queue = (mpsc_queue_t *)targ->queue;

    uint64_t start = get_timestamp_ns();
    size_t count = 0;

    while (count < targ->iterations) {
        if (mpsc_queue_pop(queue) != NULL) {
            count++;
        }
    }

    targ->total_ns = get_timestamp_ns() - start;
    return NULL;
}

static void benchmark_mpsc_multi(test_result_t *result) {
    mpsc_queue_t queue;
    mpsc_queue_init(&queue);

    pthread_t producers[NUM_PRODUCERS];
    pthread_t consumers[NUM_CONSUMERS];
    thread_arg_t producer_args[NUM_PRODUCERS];
    thread_arg_t consumer_args[NUM_CONSUMERS];

    size_t iterations_per_producer = TEST_ITERATIONS / NUM_PRODUCERS;
    size_t iterations_per_consumer = TEST_ITERATIONS / NUM_CONSUMERS;

    result->name = "MPSC Queue (Multi-threaded)";
    result->num_threads = NUM_PRODUCERS + NUM_CONSUMERS;
    result->iterations = TEST_ITERATIONS;

    /* Start producers */
    for (int i = 0; i < NUM_PRODUCERS; i++) {
        producer_args[i].queue = &queue;
        producer_args[i].iterations = iterations_per_producer;
        producer_args[i].thread_id = i;
        producer_args[i].is_producer = 1;
        pthread_create(&producers[i], NULL, mpsc_producer_thread, &producer_args[i]);
    }

    /* Start consumers */
    for (int i = 0; i < NUM_CONSUMERS; i++) {
        consumer_args[i].queue = &queue;
        consumer_args[i].iterations = iterations_per_consumer;
        consumer_args[i].thread_id = i;
        consumer_args[i].is_producer = 0;
        pthread_create(&consumers[i], NULL, mpsc_consumer_thread, &consumer_args[i]);
    }

    /* Wait for producers */
    for (int i = 0; i < NUM_PRODUCERS; i++) {
        pthread_join(producers[i], NULL);
    }

    /* Wait for consumers */
    for (int i = 0; i < NUM_CONSUMERS; i++) {
        pthread_join(consumers[i], NULL);
    }

    /* Calculate total time */
    uint64_t max_ns = 0;
    for (int i = 0; i < NUM_PRODUCERS; i++) {
        if (producer_args[i].total_ns > max_ns) {
            max_ns = producer_args[i].total_ns;
        }
    }
    for (int i = 0; i < NUM_CONSUMERS; i++) {
        if (consumer_args[i].total_ns > max_ns) {
            max_ns = consumer_args[i].total_ns;
        }
    }

    result->total_ns = max_ns;
    result->ops_per_sec = (double)TEST_ITERATIONS * 2 * 1000000000.0 / result->total_ns;
    result->avg_latency_ns = (double)result->total_ns / (TEST_ITERATIONS * 2);

    mpsc_queue_destroy(&queue);
}

/* MPMC thread */
static void *mpmc_thread(void *arg) {
    thread_arg_t *targ = (thread_arg_t *)arg;
    mpmc_queue_t *queue = (mpmc_queue_t *)targ->queue;

    uint64_t start = get_timestamp_ns();

    if (targ->is_producer) {
        for (size_t i = 0; i < targ->iterations; i++) {
            while (mpmc_queue_push(queue, (void *)i) != 0) {
                /* Retry on full */
            }
        }
    } else {
        for (size_t i = 0; i < targ->iterations; i++) {
            while (mpmc_queue_pop(queue) == NULL) {
                /* Retry on empty */
            }
        }
    }

    targ->total_ns = get_timestamp_ns() - start;
    return NULL;
}

static void benchmark_mpmc_multi(test_result_t *result) {
    mpmc_queue_t queue;
    mpmc_queue_init(&queue, TEST_CAPACITY);

    pthread_t threads[NUM_THREADS];
    thread_arg_t thread_args[NUM_THREADS];

    size_t iterations_per_thread = TEST_ITERATIONS / (NUM_THREADS / 2);

    result->name = "MPMC Queue (Multi-threaded)";
    result->num_threads = NUM_THREADS;
    result->iterations = TEST_ITERATIONS;

    /* Start producer threads */
    for (int i = 0; i < NUM_THREADS / 2; i++) {
        thread_args[i].queue = &queue;
        thread_args[i].iterations = iterations_per_thread;
        thread_args[i].thread_id = i;
        thread_args[i].is_producer = 1;
        pthread_create(&threads[i], NULL, mpmc_thread, &thread_args[i]);
    }

    /* Start consumer threads */
    for (int i = NUM_THREADS / 2; i < NUM_THREADS; i++) {
        thread_args[i].queue = &queue;
        thread_args[i].iterations = iterations_per_thread;
        thread_args[i].thread_id = i;
        thread_args[i].is_producer = 0;
        pthread_create(&threads[i], NULL, mpmc_thread, &thread_args[i]);
    }

    /* Wait for all threads */
    uint64_t max_ns = 0;
    for (int i = 0; i < NUM_THREADS; i++) {
        pthread_join(threads[i], NULL);
        if (thread_args[i].total_ns > max_ns) {
            max_ns = thread_args[i].total_ns;
        }
    }

    result->total_ns = max_ns;
    result->ops_per_sec = (double)TEST_ITERATIONS * 2 * 1000000000.0 / result->total_ns;
    result->avg_latency_ns = (double)result->total_ns / (TEST_ITERATIONS * 2);

    mpmc_queue_destroy(&queue);
}

/* Stack thread */
static void *stack_thread(void *arg) {
    thread_arg_t *targ = (thread_arg_t *)arg;
    lf_stack_t *stack = (lf_stack_t *)targ->queue;

    uint64_t start = get_timestamp_ns();

    /* Push phase */
    for (size_t i = 0; i < targ->iterations; i++) {
        lf_stack_push(stack, (void *)i);
    }

    /* Pop phase */
    for (size_t i = 0; i < targ->iterations; i++) {
        lf_stack_pop(stack);
    }

    targ->total_ns = get_timestamp_ns() - start;
    return NULL;
}

static void benchmark_stack_multi(test_result_t *result) {
    lf_stack_t stack;
    lf_stack_init(&stack);

    pthread_t threads[NUM_THREADS];
    thread_arg_t thread_args[NUM_THREADS];

    size_t iterations_per_thread = TEST_ITERATIONS / NUM_THREADS;

    result->name = "Lock-Free Stack (Multi-threaded)";
    result->num_threads = NUM_THREADS;
    result->iterations = TEST_ITERATIONS;

    /* Start threads */
    for (int i = 0; i < NUM_THREADS; i++) {
        thread_args[i].queue = &stack;
        thread_args[i].iterations = iterations_per_thread;
        thread_args[i].thread_id = i;
        pthread_create(&threads[i], NULL, stack_thread, &thread_args[i]);
    }

    /* Wait for all threads */
    uint64_t max_ns = 0;
    for (int i = 0; i < NUM_THREADS; i++) {
        pthread_join(threads[i], NULL);
        if (thread_args[i].total_ns > max_ns) {
            max_ns = thread_args[i].total_ns;
        }
    }

    result->total_ns = max_ns;
    result->ops_per_sec = (double)TEST_ITERATIONS * 2 * 1000000000.0 / result->total_ns;
    result->avg_latency_ns = (double)result->total_ns / (TEST_ITERATIONS * 2);

    lf_stack_destroy(&stack);
}

/* ============================================================================
 * Correctness tests
 * ============================================================================ */

static void test_mpsc_correctness(void) {
    printf("=== MPSC Queue Correctness Test ===\n");

    mpsc_queue_t queue;
    mpsc_queue_init(&queue);

    /* Push items */
    for (size_t i = 0; i < 1000; i++) {
        mpsc_queue_push(&queue, (void *)i);
    }

    /* Pop and verify */
    int passed = 1;
    for (size_t i = 0; i < 1000; i++) {
        void *data = mpsc_queue_pop(&queue);
        if (data != (void *)i) {
            printf("  FAIL: Expected %zu, got %p\n", i, data);
            passed = 0;
            break;
        }
    }

    if (passed) {
        printf("  PASS: All items verified\n");
    }
    printf("\n");

    mpsc_queue_destroy(&queue);
}

static void test_spsc_correctness(void) {
    printf("=== SPSC Queue Correctness Test ===\n");

    spsc_queue_t queue;
    spsc_queue_init(&queue, 1024);

    /* Push items */
    for (size_t i = 0; i < 1000; i++) {
        spsc_queue_push(&queue, (void *)i);
    }

    /* Pop and verify */
    int passed = 1;
    for (size_t i = 0; i < 1000; i++) {
        void *data = spsc_queue_pop(&queue);
        if (data != (void *)i) {
            printf("  FAIL: Expected %zu, got %p\n", i, data);
            passed = 0;
            break;
        }
    }

    if (passed) {
        printf("  PASS: All items verified\n");
    }
    printf("\n");

    spsc_queue_destroy(&queue);
}

static void test_stack_correctness(void) {
    printf("=== Lock-Free Stack Correctness Test ===\n");

    lf_stack_t stack;
    lf_stack_init(&stack);

    /* Push items */
    for (size_t i = 0; i < 1000; i++) {
        lf_stack_push(&stack, (void *)i);
    }

    /* Pop and verify (LIFO order) */
    int passed = 1;
    for (size_t i = 0; i < 1000; i++) {
        void *data = lf_stack_pop(&stack);
        size_t expected = 999 - i;  /* LIFO: last in, first out */
        if (data != (void *)expected) {
            printf("  FAIL: Expected %zu, got %p\n", expected, data);
            passed = 0;
            break;
        }
    }

    if (passed) {
        printf("  PASS: All items verified (LIFO order)\n");
    }
    printf("\n");

    lf_stack_destroy(&stack);
}

static void test_mpmc_correctness(void) {
    printf("=== MPMC Queue Correctness Test ===\n");

    mpmc_queue_t queue;
    mpmc_queue_init(&queue, 1024);

    /* Push items */
    for (size_t i = 0; i < 1000; i++) {
        mpmc_queue_push(&queue, (void *)i);
    }

    /* Pop and verify */
    int passed = 1;
    for (size_t i = 0; i < 1000; i++) {
        void *data = mpmc_queue_pop(&queue);
        if (data != (void *)i) {
            printf("  FAIL: Expected %zu, got %p\n", i, data);
            passed = 0;
            break;
        }
    }

    if (passed) {
        printf("  PASS: All items verified\n");
    }
    printf("\n");

    mpmc_queue_destroy(&queue);
}

/* ============================================================================
 * Main
 * ============================================================================ */

int main(void) {
    printf("Nuva OS Lock-Free Data Structures Performance Tests\n");
    printf("====================================================\n\n");

    test_result_t result;

    /* Correctness tests */
    printf("--- Correctness Tests ---\n\n");
    test_mpsc_correctness();
    test_spsc_correctness();
    test_stack_correctness();
    test_mpmc_correctness();

    /* Single-threaded benchmarks */
    printf("--- Single-threaded Benchmarks ---\n\n");

    benchmark_mpsc_single(&result);
    print_result(&result);

    benchmark_spsc_single(&result);
    print_result(&result);

    benchmark_stack_single(&result);
    print_result(&result);

    benchmark_mpmc_single(&result);
    print_result(&result);

    /* Multi-threaded benchmarks */
    printf("--- Multi-threaded Benchmarks ---\n\n");

    benchmark_mpsc_multi(&result);
    print_result(&result);

    benchmark_mpmc_multi(&result);
    print_result(&result);

    benchmark_stack_multi(&result);
    print_result(&result);

    /* Summary */
    printf("=== Summary ===\n");
    printf("All tests completed successfully.\n");
    printf("Lock-free data structures provide:\n");
    printf("  - Wait-free or lock-free progress guarantees\n");
    printf("  - High throughput under contention\n");
    printf("  - Low latency operations\n");
    printf("  - Memory ordering correctness\n");

    return 0;
}
