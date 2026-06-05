/*
 * Nuva OS
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

/* * Nuva OS Kernel Type Definition
 * Nuva OS Native Type System
 *
 * This header defines the native type system for Nuva OS kernel.
 * POSIX-compatible types are available only when NUVA_POSIX_COMPAT
 * is defined (optional compatibility module, not for kernel core use).
 */

#ifndef _Nuva_TYPES_H
#define _Nuva_TYPES_H

/* Basic Integer Types */
typedef signed char         int8_t;
typedef unsigned char       uint8_t;
typedef signed short        int16_t;
typedef unsigned short      uint16_t;
typedef signed int          int32_t;
typedef unsigned int        uint32_t;
typedef signed long long    int64_t;
typedef unsigned long long  uint64_t;

/* Pointer Size Types */
typedef uint64_t            uintptr_t;
typedef int64_t             intptr_t;
typedef int64_t             ptrdiff_t;

/* Size Types */
typedef uint64_t            size_t;
typedef int64_t             ssize_t;

/* Nuva Native Process and Thread Types */
typedef uint64_t            nuva_process_id_t;
typedef uint64_t            nuva_thread_id_t;
typedef uint64_t            nuva_capability_id_t;
typedef uint32_t            nuva_access_mode_t;

/* Nuva Native File Types */
typedef uint64_t            nuva_file_handle_t;
typedef uint64_t            nuva_file_offset_t;
typedef uint64_t            nuva_inode_id_t;

/* Time Types */
typedef int64_t             time_t;
typedef int64_t             clock_t;
typedef uint64_t            useconds_t;

/* Boolean Type */
typedef enum { false = 0, true = 1 } bool;

/* Error and Status Types */
typedef int32_t             errno_t;
typedef int32_t             status_t;

/* Physical and Virtual Address Types */
typedef uint64_t            phys_addr_t;
typedef uint64_t            virt_addr_t;

/* Interrupt Number */
typedef uint32_t            irq_num_t;

/* CPU Core ID */
typedef uint32_t            cpu_id_t;

/* Nuva Native Atomic Types (using _Atomic instead of volatile) */
typedef _Atomic int32_t     atomic_int_t;
typedef _Atomic uint32_t    atomic_uint_t;
typedef _Atomic int64_t     atomic_long_t;

/* Nuva Native Spinlock */
typedef struct {
    _Atomic uint32_t locked;
} nuva_spinlock_t;

#define NUVA_SPINLOCK_INIT { 0 }

/* Nuva Native Mutex */
typedef struct {
    _Atomic uint32_t locked;
    _Atomic uint32_t owner;
    _Atomic uint32_t count;
} nuva_mutex_t;

#define NUVA_MUTEX_INIT { 0, 0, 0 }

/* Nuva Native Read-Write Lock */
typedef struct {
    _Atomic uint32_t readers;
    _Atomic uint32_t writer;
} nuva_rwlock_t;

#define NUVA_RWLOCK_INIT { 0, 0 }

/* Nuva Native Memory Barriers (through HAL abstraction, no arch-specific hardcoding) */
#define nuva_barrier()       __asm__ __volatile__("" ::: "memory")
#define nuva_read_barrier()  __nuva_hal_read_barrier()
#define nuva_write_barrier() __nuva_hal_write_barrier()
#define nuva_full_barrier()  __nuva_hal_full_barrier()

/* HAL barrier function declarations (implemented by arch-specific HAL) */
extern void __nuva_hal_read_barrier(void);
extern void __nuva_hal_write_barrier(void);
extern void __nuva_hal_full_barrier(void);

/* NULL Definition */
#define NULL ((void *)0)

/* Common Macros */
#define likely(x)       __builtin_expect(!!(x), 1)
#define unlikely(x)     __builtin_expect(!!(x), 0)

#define ALIGN(x, a)     (((x) + (a) - 1) & ~((a) - 1))
#define ALIGN_DOWN(x, a) ((x) & ~((a) - 1))

#define MIN(a, b)       ((a) < (b) ? (a) : (b))
#define MAX(a, b)       ((a) > (b) ? (a) : (b))

#define ARRAY_SIZE(arr) (sizeof(arr) / sizeof((arr)[0]))

#define offsetof(type, member) __builtin_offsetof(type, member)
#define container_of(ptr, type, member) \
    ((type *)((char *)(ptr) - offsetof(type, member)))

/* Bit Operations */
#define BIT(n)          (1UL << (n))
#define BIT_MASK(n)     (BIT(n) - 1)

#define SET_BIT(x, n)   ((x) |= BIT(n))
#define CLEAR_BIT(x, n) ((x) &= ~BIT(n))
#define TEST_BIT(x, n)  (((x) & BIT(n)) != 0)

/* Compiler Attributes */
#define __packed        __attribute__((packed))
#define __aligned(x)    __attribute__((aligned(x)))
#define __noreturn      __attribute__((noreturn))
#define __unused        __attribute__((unused))
#define __used          __attribute__((used))
#define __weak          __attribute__((weak))
#define __section(x)    __attribute__((section(x)))
#define __init          __section(".init.text")
#define __initdata      __section(".init.data")
#define __exit          __section(".exit.text")
#define __exitdata      __section(".exit.data")

/* ========================================================================
 * POSIX Optional Compatibility Types
 * Only available when NUVA_POSIX_COMPAT is defined.
 * Not for kernel core use.
 * ======================================================================== */
#ifdef NUVA_POSIX_COMPAT

/* POSIX Process and Thread ID (compatibility aliases) */
typedef int32_t             pid_t;
typedef int32_t             tid_t;
typedef uint32_t            uid_t;
typedef uint32_t            gid_t;
typedef int32_t             mode_t;

/* POSIX File Descriptor Types (compatibility aliases) */
typedef int32_t             fd_t;
typedef int64_t             off_t;
typedef int64_t             blksize_t;
typedef int64_t             blkcnt_t;
typedef uint64_t            ino_t;
typedef uint64_t            dev_t;
typedef uint64_t            nlink_t;

/* POSIX Legacy Atomic Types (volatile-based, deprecated in favor of _Atomic) */
typedef volatile int32_t    posix_atomic_int_t;
typedef volatile uint32_t   posix_atomic_uint_t;
typedef volatile int64_t    posix_atomic_long_t;

/* POSIX Legacy Spinlock (volatile-based, maps to nuva native) */
typedef struct {
    volatile uint32_t locked;
} spinlock_t;

#define SPINLOCK_INIT { 0 }

/* POSIX Legacy Mutex (volatile-based, maps to nuva native) */
typedef struct {
    volatile uint32_t locked;
    volatile uint32_t owner;
    volatile uint32_t count;
} mutex_t;

#define MUTEX_INIT { 0, 0, 0 }

/* POSIX Legacy Read-Write Lock (volatile-based, maps to nuva native) */
typedef struct {
    volatile uint32_t readers;
    volatile uint32_t writer;
} rwlock_t;

#define RWLOCK_INIT { 0, 0 }

/* POSIX Legacy Memory Barriers (arch-specific, deprecated in favor of HAL abstraction) */
#define barrier()       __asm__ __volatile__("" ::: "memory")
#define mb()            nuva_full_barrier()
#define rmb()           nuva_read_barrier()
#define wmb()           nuva_write_barrier()

#endif /* NUVA_POSIX_COMPAT */

#endif /* _Nuva_TYPES_H */
