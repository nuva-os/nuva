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

/* * Nuva OS ARM64 exception vector table
 * Target platform: ARM64 (Kirin9020)
 *
 * Functions:
 *   1. Define exception vector table
 *   2. Save and restore context
 *   3. Jump to exception handler
 */


#include <Nuva/asm_macros.S>

/* Exception types */
#define EXC_SYNC_EL0        0
#define EXC_IRQ_EL0         1
#define EXC_FIQ_EL0         2
#define EXC_SERROR_EL0      3
#define EXC_SYNC_EL1        4
#define EXC_IRQ_EL1         5
#define EXC_FIQ_EL1         6
#define EXC_SERROR_EL1      7

/* Save context macro */
.macro SAVE_CONTEXT
    /* Save general registers */
    stp     x0, x1, [sp, #-16]!
    stp     x2, x3, [sp, #-16]!
    stp     x4, x5, [sp, #-16]!
    stp     x6, x7, [sp, #-16]!
    stp     x8, x9, [sp, #-16]!
    stp     x10, x11, [sp, #-16]!
    stp     x12, x13, [sp, #-16]!
    stp     x14, x15, [sp, #-16]!
    stp     x16, x17, [sp, #-16]!
    stp     x18, x19, [sp, #-16]!
    stp     x20, x21, [sp, #-16]!
    stp     x22, x23, [sp, #-16]!
    stp     x24, x25, [sp, #-16]!
    stp     x26, x27, [sp, #-16]!
    stp     x28, x29, [sp, #-16]!
    
    /* Save special registers */
    mrs     x0, elr_el1
    mrs     x1, spsr_el1
    stp     x0, x1, [sp, #-16]!
    
    /* Save stack pointer */
    mov     x0, sp
    str     x0, [sp, #-8]!
.endm

/* Restore context macro */
.macro RESTORE_CONTEXT
    /* Restore stack pointer */
    ldr     x0, [sp], #8
    mov     sp, x0
    
    /* Restore special registers */
    ldp     x0, x1, [sp], #16
    msr     elr_el1, x0
    msr     spsr_el1, x1
    
    /* Restore general registers */
    ldp     x28, x29, [sp], #16
    ldp     x26, x27, [sp], #16
    ldp     x24, x25, [sp], #16
    ldp     x22, x23, [sp], #16
    ldp     x20, x21, [sp], #16
    ldp     x18, x19, [sp], #16
    ldp     x16, x17, [sp], #16
    ldp     x14, x15, [sp], #16
    ldp     x12, x13, [sp], #16
    ldp     x10, x11, [sp], #16
    ldp     x8, x9, [sp], #16
    ldp     x6, x7, [sp], #16
    ldp     x4, x5, [sp], #16
    ldp     x2, x3, [sp], #16
    ldp     x0, x1, [sp], #16
.endm

/* * Exception vector table
 * Each exception type occupies 128 bytes (32 instructions)
 */
.section ".text.exception", "ax", @progbits
.global exception_vectors
.type exception_vectors, @function
.balign 0x800

exception_vectors:
    /* ===== Current EL with SP0 ===== */
    .balign 0x80
    /* Synchronous */
    SAVE_CONTEXT
    mov     x0, #EXC_SYNC_EL1
    b       exception_common
    .balign 0x80
    /* IRQ */
    SAVE_CONTEXT
    mov     x0, #EXC_IRQ_EL1
    b       exception_common
    .balign 0x80
    /* FIQ */
    SAVE_CONTEXT
    mov     x0, #EXC_FIQ_EL1
    b       exception_common
    .balign 0x80
    /* SError */
    SAVE_CONTEXT
    mov     x0, #EXC_SERROR_EL1
    b       exception_common
    
    /* ===== Current EL with SPx ===== */
    .balign 0x80
    /* Synchronous */
    SAVE_CONTEXT
    mov     x0, #EXC_SYNC_EL1
    b       exception_common
    .balign 0x80
    /* IRQ */
    SAVE_CONTEXT
    mov     x0, #EXC_IRQ_EL1
    b       exception_common
    .balign 0x80
    /* FIQ */
    SAVE_CONTEXT
    mov     x0, #EXC_FIQ_EL1
    b       exception_common
    .balign 0x80
    /* SError */
    SAVE_CONTEXT
    mov     x0, #EXC_SERROR_EL1
    b       exception_common
    
    /* ===== Lower EL using AArch64 ===== */
    .balign 0x80
    /* Synchronous */
    SAVE_CONTEXT
    mov     x0, #EXC_SYNC_EL0
    b       exception_common
    .balign 0x80
    /* IRQ */
    SAVE_CONTEXT
    mov     x0, #EXC_IRQ_EL0
    b       exception_common
    .balign 0x80
    /* FIQ */
    SAVE_CONTEXT
    mov     x0, #EXC_FIQ_EL0
    b       exception_common
    .balign 0x80
    /* SError */
    SAVE_CONTEXT
    mov     x0, #EXC_SERROR_EL0
    b       exception_common
    
    /* ===== Lower EL using AArch32 ===== */
    .balign 0x80
    /* Synchronous */
    SAVE_CONTEXT
    mov     x0, #EXC_SYNC_EL0
    b       exception_common
    .balign 0x80
    /* IRQ */
    SAVE_CONTEXT
    mov     x0, #EXC_IRQ_EL0
    b       exception_common
    .balign 0x80
    /* FIQ */
    SAVE_CONTEXT
    mov     x0, #EXC_FIQ_EL0
    b       exception_common
    .balign 0x80
    /* SError */
    SAVE_CONTEXT
    mov     x0, #EXC_SERROR_EL0
    b       exception_common

/* * Exception common handler entry
 * x0: Exception type
 */
exception_common:
    /* Save exception type */
    mov     x1, sp
    
    /* Call Rust exception handler */
    bl      handle_exception
    
    /* Restore context and return */
    RESTORE_CONTEXT
    eret

.size exception_vectors, . - exception_vectors

/* * System call entry (SVC)
 */
.section ".text.syscall", "ax", @progbits
.global syscall_entry
.type syscall_entry, @function

syscall_entry:
    SAVE_CONTEXT
    
    /* System call number in x8, parameters in x0-x5 */
    mov     x6, sp
    bl      handle_syscall
    
    RESTORE_CONTEXT
    eret

.size syscall_entry, . - syscall_entry

/* * Interrupt return
 */
.section ".text", "ax", @progbits
.global exception_return
.type exception_return, @function

exception_return:
    RESTORE_CONTEXT
    eret

.size exception_return, . - exception_return

/* * Enable interrupt
 */
ENTRY(enable_irq)
    msr     daifclr, #2
    isb
    ret
ENDPROC(enable_irq)

/* * Disable interrupt
 */
ENTRY(disable_irq)
    msr     daifset, #2
    isb
    ret
ENDPROC(disable_irq)

/* * Enable FIQ
 */
ENTRY(enable_fiq)
    msr     daifclr, #1
    isb
    ret
ENDPROC(enable_fiq)

/* * Disable FIQ
 */
ENTRY(disable_fiq)
    msr     daifset, #1
    isb
    ret
ENDPROC(disable_fiq)

/* * Enable all exceptions
 */
ENTRY(enable_exceptions)
    msr     daifclr, #0xF
    isb
    ret
ENDPROC(enable_exceptions)

/* * Disable all exceptions
 */
ENTRY(disable_exceptions)
    msr     daifset, #0xF
    isb
    ret
ENDPROC(disable_exceptions)

/* * Get current exception level
 * Return: x0 = Exception level (0-3)
 */
ENTRY(get_current_el)
    mrs     x0, CurrentEL
    lsr     x0, x0, #2
    and     x0, x0, #3
    ret
ENDPROC(get_current_el)
