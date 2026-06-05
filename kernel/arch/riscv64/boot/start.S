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

/* RISC-V 64-bit S-mode kernel boot entry
 * Target: QEMU virt machine
 *
 * This code runs in S-mode after OpenSBI firmware.
 * Only Hart 0 enters the kernel; other harts spin in WFI loop.
 */

.section .text.entry, "ax", @progbits
.globl _start
.type _start, @function

_start:
    /* Select Hart 0 only; park all other harts */
    csrr    a0, mhartid
    bnez    a0, .park

    /* Set up stack pointer */
    la      sp, _boot_stack_top

    /* Disable all S-mode interrupts */
    csrw    sie, zero

    /* Set temporary trap vector to infinite loop */
    la      t0, .trap_early
    csrw    stvec, t0

    /* Clear BSS section */
    la      t0, _bss_start
    la      t1, _bss_end
.bss_clear:
    bgeu    t0, t1, .bss_done
    sd      zero, 0(t0)
    addi    t0, t0, 8
    j       .bss_clear
.bss_done:

    /* Save FDT physical address passed by SBI/firmware in a0 */
    /* a0 contains hartid from mhartid read above, FDT is in a1 from firmware */
    mv      a0, a1

    /* Jump to Rust kernel entry */
    call    rust_main

    /* Should never reach here */
    j       .

.park:
    /* Non-boot harts: wait in WFI loop */
    wfi
    j       .park

.trap_early:
    /* Early trap handler before kernel is ready */
    j       .trap_early

.size _start, . - _start

/* Boot stack section */
.section .data.boot, "aw", @progbits
.balign 16
_boot_stack_bottom:
    .space  16384           /* 16 KB boot stack */
_boot_stack_top:
