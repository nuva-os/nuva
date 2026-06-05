/*
 * Nuva OS - Lock-Free Data Structures
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
 * FFI Compatibility Layer - Not for kernel core use
 *
 * Lock-Free Data Structures (FFI wrapper)
 *
 * The kernel core path uses the Rust-native MpscQueue/SpscQueue/
 * TreiberStack/MpmcQueue implementations in lockfree.rs.
 * This C file is retained for external C/C++ code only.
 *
 * Original implemented structures:
 * - MPSC Queue (Multi-Producer Single-Consumer)
 * - SPSC Queue (Single-Producer Single-Consumer)
 * - Concurrent Stack (Treiber Stack)
 * - MPMC Queue (Multi-Producer Multi-Consumer)
 */

#include <stdint.h>
#include <stddef.h>
#include <stdlib.h>
#include <string.h>
#include <stdatomic.h>

/* ============================================================================
 * MPSC Queue - Multi-Producer Single-Consumer Queue
 * Based on Michael-Scott queue algorithm
 * ============================================================================ */

/* Queue node */
typedef struct mpsc_node {
    void *data;
    struct mpsc_node *next;
} mpsc_node_t;

/* MPSC Queue structure */
typedef struct {
    _Atomic(mpsc_node_t *) head;    /* Consumer side */
    _Atomic(mpsc_node_t *) tail;    /* Producer side */
    _Atomic(size_t) length;         /* Queue length */
    _Atomic(uint64_t) push_count;   /* Total pushes */
    _Atomic(uint64_t) pop_count;    /* Total pops */
} mpsc_queue_t;

/**
 * Initialize MPSC queue
 *
 * @param queue Pointer to queue structure
 * @return 0 on success, -1 on error
 */
int mpsc_queue_init(mpsc_queue_t *queue) {
    if (queue == NULL) {
        return -1;
    }

    /* Create sentinel node */
    mpsc_node_t *sentinel = (mpsc_node_t *)malloc(sizeof(mpsc_node_t));
    if (sentinel == NULL) {
        return -1;
    }

    sentinel->data = NULL;
    sentinel->next = NULL;

    atomic_store(&queue->head, sentinel);
    atomic_store(&queue->tail, sentinel);
    atomic_store(&queue->length, 0);
    atomic_store(&queue->push_count, 0);
    atomic_store(&queue->pop_count, 0);

    return 0;
}

/**
 * Push item to MPSC queue (producer)
 *
 * @param queue Pointer to queue structure
 * @param data Data to push
 * @return 0 on success, -1 on error
 */
int mpsc_queue_push(mpsc_queue_t *queue, void *data) {
    if (queue == NULL) {
        return -1;
    }

    /* Create new node */
    mpsc_node_t *new_node = (mpsc_node_t *)malloc(sizeof(mpsc_node_t));
    if (new_node == NULL) {
        return -1;
    }

    new_node->data = data;
    new_node->next = NULL;

    /* Add to tail using CAS loop */
    mpsc_node_t *tail;
    mpsc_node_t *next;

    while (1) {
        tail = atomic_load_explicit(&queue->tail, memory_order_acquire);
        next = tail->next;

        /* Check if tail is still the tail */
        if (tail == atomic_load_explicit(&queue->tail, memory_order_acquire)) {
            if (next == NULL) {
                /* Try to link new node */
                if (atomic_compare_exchange_weak_explicit(
                    (_Atomic(void *) *)&tail->next,
                    &next,
                    new_node,
                    memory_order_release,
                    memory_order_relaxed)) {
                    /* Successfully linked, advance tail */
                    atomic_compare_exchange_strong_explicit(
                        &queue->tail,
                        &tail,
                        new_node,
                        memory_order_release,
                        memory_order_relaxed);
                    atomic_fetch_add_explicit(&queue->length, 1, memory_order_relaxed);
                    atomic_fetch_add_explicit(&queue->push_count, 1, memory_order_relaxed);
                    return 0;
                }
            } else {
                /* Tail is lagging, advance it */
                atomic_compare_exchange_weak_explicit(
                    &queue->tail,
                    &tail,
                    next,
                    memory_order_release,
                    memory_order_relaxed);
            }
        }
    }
}

/**
 * Pop item from MPSC queue (consumer)
 *
 * @param queue Pointer to queue structure
 * @return Data pointer, or NULL if queue is empty
 */
void *mpsc_queue_pop(mpsc_queue_t *queue) {
    if (queue == NULL) {
        return NULL;
    }

    mpsc_node_t *head;
    mpsc_node_t *tail;
    mpsc_node_t *next;
    void *data;

    while (1) {
        head = atomic_load_explicit(&queue->head, memory_order_acquire);
        tail = atomic_load_explicit(&queue->tail, memory_order_acquire);
        next = head->next;

        /* Check if head is still the head */
        if (head == atomic_load_explicit(&queue->head, memory_order_acquire)) {
            if (head == tail) {
                if (next == NULL) {
                    /* Queue is empty */
                    return NULL;
                }
                /* Tail is lagging, advance it */
                atomic_compare_exchange_weak_explicit(
                    &queue->tail,
                    &tail,
                    next,
                    memory_order_release,
                    memory_order_relaxed);
            } else {
                /* Read value before CAS */
                data = next->data;

                /* Try to advance head */
                if (atomic_compare_exchange_weak_explicit(
                    &queue->head,
                    &head,
                    next,
                    memory_order_release,
                    memory_order_relaxed)) {
                    /* Successfully popped, free old head */
                    free(head);
                    atomic_fetch_sub_explicit(&queue->length, 1, memory_order_relaxed);
                    atomic_fetch_add_explicit(&queue->pop_count, 1, memory_order_relaxed);
                    return data;
                }
            }
        }
    }
}

/**
 * Check if MPSC queue is empty
 *
 * @param queue Pointer to queue structure
 * @return 1 if empty, 0 if not, -1 on error
 */
int mpsc_queue_is_empty(mpsc_queue_t *queue) {
    if (queue == NULL) {
        return -1;
    }

    mpsc_node_t *head = atomic_load_explicit(&queue->head, memory_order_acquire);
    mpsc_node_t *tail = atomic_load_explicit(&queue->tail, memory_order_acquire);

    return (head == tail && head->next == NULL) ? 1 : 0;
}

/**
 * Get MPSC queue length
 *
 * @param queue Pointer to queue structure
 * @return Queue length
 */
size_t mpsc_queue_length(mpsc_queue_t *queue) {
    if (queue == NULL) {
        return 0;
    }
    return atomic_load_explicit(&queue->length, memory_order_relaxed);
}

/**
 * Destroy MPSC queue
 *
 * @param queue Pointer to queue structure
 */
void mpsc_queue_destroy(mpsc_queue_t *queue) {
    if (queue == NULL) {
        return;
    }

    /* Pop all remaining items */
    while (mpsc_queue_pop(queue) != NULL) {
        /* Continue */
    }

    /* Free sentinel node */
    mpsc_node_t *head = atomic_load(&queue->head);
    if (head != NULL) {
        free(head);
    }
}

/* ============================================================================
 * SPSC Queue - Single-Producer Single-Consumer Queue
 * Bounded ring buffer implementation
 * ============================================================================ */

typedef struct {
    void **buffer;              /* Ring buffer */
    size_t capacity;            /* Buffer capacity */
    size_t mask;                /* Capacity mask (capacity - 1, for power of 2) */
    _Atomic(size_t) head;       /* Consumer index */
    _Atomic(size_t) tail;       /* Producer index */
    _Atomic(uint64_t) push_count;
    _Atomic(uint64_t) pop_count;
} spsc_queue_t;

/**
 * Initialize SPSC queue
 *
 * @param queue Pointer to queue structure
 * @param capacity Queue capacity (will be rounded to power of 2)
 * @return 0 on success, -1 on error
 */
int spsc_queue_init(spsc_queue_t *queue, size_t capacity) {
    if (queue == NULL || capacity == 0) {
        return -1;
    }

    /* Round capacity to power of 2 */
    size_t cap = 1;
    while (cap < capacity) {
        cap *= 2;
    }

    queue->buffer = (void **)calloc(cap, sizeof(void *));
    if (queue->buffer == NULL) {
        return -1;
    }

    queue->capacity = cap;
    queue->mask = cap - 1;
    atomic_store(&queue->head, 0);
    atomic_store(&queue->tail, 0);
    atomic_store(&queue->push_count, 0);
    atomic_store(&queue->pop_count, 0);

    return 0;
}

/**
 * Push item to SPSC queue (producer)
 *
 * @param queue Pointer to queue structure
 * @param data Data to push
 * @return 0 on success, -1 if queue is full
 */
int spsc_queue_push(spsc_queue_t *queue, void *data) {
    if (queue == NULL) {
        return -1;
    }

    size_t tail = atomic_load_explicit(&queue->tail, memory_order_relaxed);
    size_t next_tail = (tail + 1) & queue->mask;

    /* Check if full */
    size_t head = atomic_load_explicit(&queue->head, memory_order_acquire);
    if (next_tail == head) {
        return -1;  /* Queue is full */
    }

    /* Store item */
    queue->buffer[tail] = data;
    atomic_store_explicit(&queue->tail, next_tail, memory_order_release);
    atomic_fetch_add_explicit(&queue->push_count, 1, memory_order_relaxed);

    return 0;
}

/**
 * Pop item from SPSC queue (consumer)
 *
 * @param queue Pointer to queue structure
 * @return Data pointer, or NULL if queue is empty
 */
void *spsc_queue_pop(spsc_queue_t *queue) {
    if (queue == NULL) {
        return NULL;
    }

    size_t head = atomic_load_explicit(&queue->head, memory_order_relaxed);

    /* Check if empty */
    size_t tail = atomic_load_explicit(&queue->tail, memory_order_acquire);
    if (head == tail) {
        return NULL;  /* Queue is empty */
    }

    /* Load item */
    void *data = queue->buffer[head];
    size_t next_head = (head + 1) & queue->mask;
    atomic_store_explicit(&queue->head, next_head, memory_order_release);
    atomic_fetch_add_explicit(&queue->pop_count, 1, memory_order_relaxed);

    return data;
}

/**
 * Check if SPSC queue is empty
 */
int spsc_queue_is_empty(spsc_queue_t *queue) {
    if (queue == NULL) {
        return -1;
    }
    size_t head = atomic_load_explicit(&queue->head, memory_order_relaxed);
    size_t tail = atomic_load_explicit(&queue->tail, memory_order_relaxed);
    return (head == tail) ? 1 : 0;
}

/**
 * Check if SPSC queue is full
 */
int spsc_queue_is_full(spsc_queue_t *queue) {
    if (queue == NULL) {
        return -1;
    }
    size_t head = atomic_load_explicit(&queue->head, memory_order_acquire);
    size_t tail = atomic_load_explicit(&queue->tail, memory_order_relaxed);
    size_t next_tail = (tail + 1) & queue->mask;
    return (next_tail == head) ? 1 : 0;
}

/**
 * Get SPSC queue length
 */
size_t spsc_queue_length(spsc_queue_t *queue) {
    if (queue == NULL) {
        return 0;
    }
    size_t head = atomic_load_explicit(&queue->head, memory_order_relaxed);
    size_t tail = atomic_load_explicit(&queue->tail, memory_order_relaxed);
    return (tail - head) & queue->mask;
}

/**
 * Destroy SPSC queue
 */
void spsc_queue_destroy(spsc_queue_t *queue) {
    if (queue == NULL) {
        return;
    }
    if (queue->buffer != NULL) {
        free(queue->buffer);
        queue->buffer = NULL;
    }
}

/* ============================================================================
 * Lock-Free Stack - Treiber Stack
 * ============================================================================ */

typedef struct stack_node {
    void *data;
    struct stack_node *next;
} stack_node_t;

typedef struct {
    _Atomic(stack_node_t *) head;
    _Atomic(size_t) length;
    _Atomic(uint64_t) push_count;
    _Atomic(uint64_t) pop_count;
} lf_stack_t;

/**
 * Initialize lock-free stack
 */
int lf_stack_init(lf_stack_t *stack) {
    if (stack == NULL) {
        return -1;
    }

    atomic_store(&stack->head, NULL);
    atomic_store(&stack->length, 0);
    atomic_store(&stack->push_count, 0);
    atomic_store(&stack->pop_count, 0);

    return 0;
}

/**
 * Push item to stack
 */
int lf_stack_push(lf_stack_t *stack, void *data) {
    if (stack == NULL) {
        return -1;
    }

    stack_node_t *new_node = (stack_node_t *)malloc(sizeof(stack_node_t));
    if (new_node == NULL) {
        return -1;
    }

    new_node->data = data;

    while (1) {
        stack_node_t *head = atomic_load_explicit(&stack->head, memory_order_acquire);
        new_node->next = head;

        if (atomic_compare_exchange_weak_explicit(
            &stack->head,
            &head,
            new_node,
            memory_order_release,
            memory_order_relaxed)) {
            atomic_fetch_add_explicit(&stack->length, 1, memory_order_relaxed);
            atomic_fetch_add_explicit(&stack->push_count, 1, memory_order_relaxed);
            return 0;
        }
    }
}

/**
 * Pop item from stack
 */
void *lf_stack_pop(lf_stack_t *stack) {
    if (stack == NULL) {
        return NULL;
    }

    while (1) {
        stack_node_t *head = atomic_load_explicit(&stack->head, memory_order_acquire);

        if (head == NULL) {
            return NULL;  /* Stack is empty */
        }

        stack_node_t *next = head->next;

        if (atomic_compare_exchange_weak_explicit(
            &stack->head,
            &head,
            next,
            memory_order_release,
            memory_order_relaxed)) {
            void *data = head->data;
            free(head);
            atomic_fetch_sub_explicit(&stack->length, 1, memory_order_relaxed);
            atomic_fetch_add_explicit(&stack->pop_count, 1, memory_order_relaxed);
            return data;
        }
    }
}

/**
 * Check if stack is empty
 */
int lf_stack_is_empty(lf_stack_t *stack) {
    if (stack == NULL) {
        return -1;
    }
    return atomic_load(&stack->head) == NULL ? 1 : 0;
}

/**
 * Get stack length
 */
size_t lf_stack_length(lf_stack_t *stack) {
    if (stack == NULL) {
        return 0;
    }
    return atomic_load(&stack->length);
}

/**
 * Destroy stack
 */
void lf_stack_destroy(lf_stack_t *stack) {
    if (stack == NULL) {
        return;
    }

    while (lf_stack_pop(stack) != NULL) {
        /* Continue */
    }
}

/* ============================================================================
 * MPMC Queue - Multi-Producer Multi-Consumer Queue
 * Based on Dmitry Vyukov's bounded MPMC queue
 * ============================================================================ */

typedef struct {
    void *data;
    _Atomic(size_t) sequence;
} mpmc_cell_t;

typedef struct {
    mpmc_cell_t *buffer;
    size_t capacity;
    size_t mask;
    _Atomic(size_t) enqueue_pos;
    _Atomic(size_t) dequeue_pos;
    _Atomic(uint64_t) push_count;
    _Atomic(uint64_t) pop_count;
} mpmc_queue_t;

/**
 * Initialize MPMC queue
 */
int mpmc_queue_init(mpmc_queue_t *queue, size_t capacity) {
    if (queue == NULL || capacity == 0) {
        return -1;
    }

    /* Round capacity to power of 2 */
    size_t cap = 1;
    while (cap < capacity) {
        cap *= 2;
    }

    queue->buffer = (mpmc_cell_t *)calloc(cap, sizeof(mpmc_cell_t));
    if (queue->buffer == NULL) {
        return -1;
    }

    queue->capacity = cap;
    queue->mask = cap - 1;

    for (size_t i = 0; i < cap; i++) {
        atomic_store(&queue->buffer[i].sequence, i);
    }

    atomic_store(&queue->enqueue_pos, 0);
    atomic_store(&queue->dequeue_pos, 0);
    atomic_store(&queue->push_count, 0);
    atomic_store(&queue->pop_count, 0);

    return 0;
}

/**
 * Push item to MPMC queue
 */
int mpmc_queue_push(mpmc_queue_t *queue, void *data) {
    if (queue == NULL) {
        return -1;
    }

    mpmc_cell_t *cell;
    size_t pos = atomic_load_explicit(&queue->enqueue_pos, memory_order_relaxed);

    while (1) {
        cell = &queue->buffer[pos & queue->mask];
        size_t seq = atomic_load_explicit(&cell->sequence, memory_order_acquire);
        intptr_t diff = (intptr_t)seq - (intptr_t)pos;

        if (diff == 0) {
            if (atomic_compare_exchange_weak_explicit(
                &queue->enqueue_pos,
                &pos,
                pos + 1,
                memory_order_relaxed,
                memory_order_relaxed)) {
                break;
            }
        } else if (diff < 0) {
            return -1;  /* Queue is full */
        } else {
            pos = atomic_load_explicit(&queue->enqueue_pos, memory_order_relaxed);
        }
    }

    cell->data = data;
    atomic_store_explicit(&cell->sequence, pos + 1, memory_order_release);
    atomic_fetch_add_explicit(&queue->push_count, 1, memory_order_relaxed);

    return 0;
}

/**
 * Pop item from MPMC queue
 */
void *mpmc_queue_pop(mpmc_queue_t *queue) {
    if (queue == NULL) {
        return NULL;
    }

    mpmc_cell_t *cell;
    size_t pos = atomic_load_explicit(&queue->dequeue_pos, memory_order_relaxed);

    while (1) {
        cell = &queue->buffer[pos & queue->mask];
        size_t seq = atomic_load_explicit(&cell->sequence, memory_order_acquire);
        intptr_t diff = (intptr_t)seq - (intptr_t)(pos + 1);

        if (diff == 0) {
            if (atomic_compare_exchange_weak_explicit(
                &queue->dequeue_pos,
                &pos,
                pos + 1,
                memory_order_relaxed,
                memory_order_relaxed)) {
                break;
            }
        } else if (diff < 0) {
            return NULL;  /* Queue is empty */
        } else {
            pos = atomic_load_explicit(&queue->dequeue_pos, memory_order_relaxed);
        }
    }

    void *data = cell->data;
    atomic_store_explicit(&cell->sequence, pos + queue->mask + 1, memory_order_release);
    atomic_fetch_add_explicit(&queue->pop_count, 1, memory_order_relaxed);

    return data;
}

/**
 * Check if MPMC queue is empty
 */
int mpmc_queue_is_empty(mpmc_queue_t *queue) {
    if (queue == NULL) {
        return -1;
    }

    size_t enqueue_pos = atomic_load_explicit(&queue->enqueue_pos, memory_order_relaxed);
    size_t dequeue_pos = atomic_load_explicit(&queue->dequeue_pos, memory_order_relaxed);

    return (enqueue_pos == dequeue_pos) ? 1 : 0;
}

/**
 * Get MPMC queue length
 */
size_t mpmc_queue_length(mpmc_queue_t *queue) {
    if (queue == NULL) {
        return 0;
    }

    size_t enqueue_pos = atomic_load_explicit(&queue->enqueue_pos, memory_order_relaxed);
    size_t dequeue_pos = atomic_load_explicit(&queue->dequeue_pos, memory_order_relaxed);

    return enqueue_pos - dequeue_pos;
}

/**
 * Destroy MPMC queue
 */
void mpmc_queue_destroy(mpmc_queue_t *queue) {
    if (queue == NULL) {
        return;
    }

    if (queue->buffer != NULL) {
        free(queue->buffer);
        queue->buffer = NULL;
    }
}

/* ============================================================================
 * Utility Functions
 * ============================================================================ */

/**
 * Get statistics for any queue type
 */
typedef struct {
    uint64_t push_count;
    uint64_t pop_count;
    size_t current_length;
} queue_stats_t;

void mpsc_queue_get_stats(mpsc_queue_t *queue, queue_stats_t *stats) {
    if (queue && stats) {
        stats->push_count = atomic_load(&queue->push_count);
        stats->pop_count = atomic_load(&queue->pop_count);
        stats->current_length = atomic_load(&queue->length);
    }
}

void spsc_queue_get_stats(spsc_queue_t *queue, queue_stats_t *stats) {
    if (queue && stats) {
        stats->push_count = atomic_load(&queue->push_count);
        stats->pop_count = atomic_load(&queue->pop_count);
        stats->current_length = spsc_queue_length(queue);
    }
}

void mpmc_queue_get_stats(mpmc_queue_t *queue, queue_stats_t *stats) {
    if (queue && stats) {
        stats->push_count = atomic_load(&queue->push_count);
        stats->pop_count = atomic_load(&queue->pop_count);
        stats->current_length = mpmc_queue_length(queue);
    }
}
