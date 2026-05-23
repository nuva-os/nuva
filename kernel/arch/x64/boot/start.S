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
 * Target platform: x86_64 (Intel Core / AMD Ryzen)
 *
 * Functions:
 *   1. Take control from Multiboot2 bootloader
 *   2. Transition from 32-bit protected mode to 64-bit long mode
 *   3. Set up temporary page tables (identity mapping)
 *   4. Jump to Rust kernel entry point
 */

/* Multiboot2 header */
.section .rodata.boot, "a", @progbits
.align 8
mb2_header:
    .long 0xE85250D6                                   /* magic */
    .long 0                                              /* architecture: 0 = protected mode */
    .long mb2_header_end - mb2_header                    /* header_length */
    .long -(0xE85250D6 + 0 + (mb2_header_end - mb2_header))  /* checksum */
    /* Framebuffer tag (optional) */
    .align 8
    .word 5                                              /* tag type: framebuffer */
    .word 0                                              /* flags */
    .long 20                                             /* size */
    .long 80                                             /* width */
    .long 25                                             /* height */
    .long 16                                             /* depth */
mb2_header_end:

/* 32-bit GDT for transition to long mode */
.section .rodata.boot, "a", @progbits
.align 16
gdt32:
    .word gdt32_end - gdt32 - 1                          /* limit */
    .long gdt32                                          /* base */
    .word 0
gdt32_entries:
    .quad 0                                              /* null descriptor */
    .quad 0x00CF9A000000FFFF                             /* 32-bit code */
    .quad 0x00CF92000000FFFF                             /* 32-bit data */
gdt32_end:

/* 64-bit GDT */
.section .rodata.boot, "a", @progbits
.align 16
gdt64_ptr:
    .word gdt64_end - gdt64 - 1                          /* limit */
    .quad gdt64                                           /* base */
gdt64:
    .quad 0                                              /* null descriptor */
    .quad 0x00AF9A000000FFFF                             /* 64-bit code (kernel) */
    .quad 0x00CF92000000FFFF                             /* data (kernel) */
    .quad 0x00AFFA000000FFFF                             /* 64-bit code (user) */
    .quad 0x00CFF2000000FFFF                             /* data (user) */
gdt64_end:

/* Boot page tables (PML4, PDPT, PD) - identity mapped first 1GB */
.section .data.boot, "aw", @progbits
.align 4096
boot_pml4:
    .quad boot_pdpt + 0x03                              /* present + writable */
    .fill 511, 8, 0
boot_pdpt:
    .quad boot_pd + 0x03                                /* present + writable */
    .fill 511, 8, 0
boot_pd:
    .quad 0x00000083                                    /* 2MB page: present + writable + huge */
    .quad 0x00200083
    .quad 0x00400083
    .quad 0x00600083
    .quad 0x00800083
    .quad 0x00A00083
    .quad 0x00C00083
    .quad 0x00E00083
    .quad 0x01000083
    .quad 0x01200083
    .quad 0x01400083
    .quad 0x01600083
    .quad 0x01800083
    .quad 0x01A00083
    .quad 0x01C00083
    .quad 0x01E00083
    .fill 496, 8, 0

/* 32-bit entry point (protected mode) */
.section .text.boot, "ax", @progbits
.code32
.global _start
.type _start, @function

_start:
    /* Save Multiboot2 info pointer (ebx) */
    mov edi, ebx

    /* Load temporary 32-bit GDT */
    lgdt gdt32

    /* Set up segment registers */
    mov ax, 0x10                                         /* 32-bit data segment */
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax

    /* Disable paging before setting up new page tables */
    mov eax, cr0
    and eax, ~(1 << 31)                                  /* CR0.PG = 0 */
    mov cr0, eax

    /* Enable PAE (Physical Address Extension) */
    mov eax, cr4
    or eax, (1 << 5)                                     /* CR4.PAE = 1 */
    mov cr4, eax

    /* Load PML4 into CR3 */
    mov eax, boot_pml4
    mov cr3, eax

    /* Enable long mode via EFER MSR */
    mov ecx, 0xC0000080                                  /* IA32_EFER */
    rdmsr
    or eax, (1 << 8)                                     /* EFER.LME = 1 */
    wrmsr

    /* Enable paging */
    mov eax, cr0
    or eax, (1 << 31)                                    /* CR0.PG = 1 */
    mov cr0, eax

    /* Far jump to 64-bit code segment */
    ljmp $0x08, $start64

/* 64-bit entry point (long mode) */
.code64
start64:
    /* Load 64-bit GDT */
    lgdt gdt64_ptr

    /* Set up segment registers for 64-bit mode */
    mov ax, 0x10                                         /* data segment */
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax

    /* Set up stack */
    lea rsp, [boot_stack_top]

    /* Clear BSS section */
    lea rcx, [__bss_start]
    lea rdi, [__bss_end]
    sub rdi, rcx
    xor eax, eax
    rep stosb

    /* Call kernel main with Multiboot2 info pointer (still in edi) */
    call kmain

    /* Should not reach here */
    cli
    hlt
    jmp .

.size _start, . - _start

/* Boot stack */
.section .data.boot, "aw", @progbits
.align 16
boot_stack_bottom:
    .space 0x10000                                       /* 64KB boot stack */
boot_stack_top:
