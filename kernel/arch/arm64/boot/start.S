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

/* * Nuva OS kernel boot assembly code
 * Target platform: ARM64 (Kirin9020)
 *
 * Functions:
 *   1. Take control from bootloader
 *   2. Set exception level (EL2 -> EL1)
 *   3. Initialize page table and MMU
 *   4. Enable caches
 *   5. Jump to Rust kernel entry point
 */


#include <Nuva/asm_macros.S>

/* Kernel load address */
#define KERNEL_BASE     0xFFFFFF8000000000
#define KERNEL_PHYS     0x00080000

/* Page table attributes */
#define PT_ATTR         0x401   /* AttrIdx=0 (Device-nGnRnE) */
#define PT_NORMAL       0x711   /* AttrIdx=1 (Normal Cacheable) */
#define PT_KERNEL       0x703   /* AttrIdx=1, AP=00 (Kernel RW) */
#define PT_USER         0x707   /* AttrIdx=1, AP=01 (User RW) */

/* Exception levels */
#define EL1             1
#define EL2             2

.section ".text.boot", "ax", @progbits
.global _start
.type _start, @function

_start:
    /* * Bootloader passed parameters:
     *   x0: Device tree pointer (FDT)
     *   x1: Reserved
     *   x2: Reserved
     *   x3: Reserved
     */




    

    /* Save device tree pointer */
    mov     x20, x0

    /* Check current exception level */
    mrs     x0, CurrentEL
    and     x0, x0, #0xC
    cmp     x0, #0x8            /* EL2 */
    b.eq    el2_setup
    cmp     x0, #0x4            /* EL1 */
    b.eq    el1_setup

    /* Unsupported exception level */
    b       .

el2_setup:
    /* ===== EL2 initialize =====

    /* Save FDT physical address */
    mov     x21, x20

    /* Configure HCR_EL2 to run EL1 */
    mov     x0, #(1 << 31)      /* RW=1: EL1 is AArch64 */
    msr     hcr_el2, x0

    /* Configure SCTLR_EL1 initial value */
    mov     x0, xzr
    msr     sctlr_el1, x0

    /* Configure exception vectors */
    ldr     x0, =exception_vectors
    msr     vbar_el1, x0

    /* Set EL1 stack pointer */
    ldr     x0, =boot_stack_top
    msr     sp_el1, x0

    /* Configure EL1 SPSR */
    mov     x0, #0x3C5          /* EL1h, DAIF masked */
    msr     spsr_el2, x0

    /* Set return address */
    ldr     x0, =el1_setup
    msr     elr_el2, x0

    /* Return to EL1 */
    eret

el1_setup:
    /* ===== EL1 initialize =====

    /* Restore FDT pointer */
    mov     x0, x20

    /* Set exception vector table (exception_vectors defined in exception.S) */
    ldr     x1, =exception_vectors
    msr     vbar_el1, x1
    isb

    /* Disable interrupts */
    msr     daifset, #0xF

    /* Set stack pointer */
    ldr     x1, =boot_stack_top
    mov     sp, x1

    /* Initialize BSS section */
    ldr     x1, =__bss_start
    ldr     x2, =__bss_end
    sub     x2, x2, x1
    cbz     x2, bss_done
bss_loop:
    str     xzr, [x1], #8
    subs    x2, x2, #8
    b.ne    bss_loop
bss_done:

    /* Initialize page tables */
    bl      setup_page_tables

    /* Enable MMU */
    bl      enable_mmu

    /* Enable caches */
    bl      enable_caches

    /* Jump to kernel main function */
    ldr     x1, =kmain
    mov     x0, x20             /* FDT pointer */
    blr     x1

    /* Should not reach here */
    b       .

setup_page_tables:
    /* ===== Page table initialization =====

    /* Get page table base address */
    ldr     x0, =boot_page_table

    /* Clear page table */
    mov     x1, xzr
    mov     x2, #0x1000
pt_clear:
    str     xzr, [x0, x1, lsl #3]
    add     x1, x1, #1
    cmp     x1, x2
    b.ne    pt_clear

    /* Set MAIR attributes */
    ldr     x1, =0x00000000_0044_FF00_BB00
    msr     mair_el1, x1

    /* Configure TCR */
    ldr     x1, =0x00000000_B518_3519
    msr     tcr_el1, x1

    /* Set TTBR0 */
    ldr     x1, =boot_page_table
    msr     ttbr0_el1, x1

    /* Set TTBR1 */
    ldr     x1, =boot_page_table
    msr     ttbr1_el1, x1

    ret

enable_mmu:
    /* ===== Enable MMU =====

    /* Read SCTLR */
    mrs     x0, sctlr_el1

    /* Set MMU enable bit */
    orr     x0, x0, #0x1        /* M=1: MMU Enable */

    /* Write back SCTLR */
    msr     sctlr_el1, x0

    /* Instruction synchronization barrier */
    isb

    ret

enable_caches:
    /* ===== Enable caches =====

    /* Read SCTLR */
    mrs     x0, sctlr_el1

    /* Enable I-Cache and D-Cache */
    orr     x0, x0, #(1 << 2)   /* C=1: D-Cache Enable */
    orr     x0, x0, #(1 << 12)  /* I=1: I-Cache Enable */

    /* Write back SCTLR */
    msr     sctlr_el1, x0

    /* Instruction synchronization barrier */
    isb

    ret

/* * Exception vector table is defined in exception.S
 * The VBAR_EL1 is already set to exception_vectors in el2_setup above.
 * start.S no longer defines a dead-loop exception_vectors stub.
 */

.size _start, . - _start

/* * Boot data section
 */
.section ".data.boot", "aw", @progbits
.balign 4096

boot_page_table:
    .space  0x1000 * 4          /* 4 page table pages */

.balign 16
boot_stack_bottom:
    .space  0x10000             /* 64KB boot stack */
boot_stack_top:
