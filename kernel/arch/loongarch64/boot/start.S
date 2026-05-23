#
# Nuva OS
#
# Copyright (C) 2026 Nuva OS Team
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.
#

# * Nuva OS kernel boot assembly code
# * Target platform: LoongArch64 (Loongson 3A6000)
#
# Functions:
#   1. Take control from UEFI firmware
#   2. Save boot info pointer
#   3. Configure CSR registers
#   4. Set up temporary page tables and enable MMU
#   5. Jump to Rust kernel entry point

.section .text.boot, "ax", @progbits
.global _start
.type _start, @function

_start:
    # Save UEFI boot info pointer (argument in $a0)
    move    $s0, $a0

    # Set up stack pointer
    la.abs  $sp, boot_stack_top

    # Clear BSS section
    la.abs  $t0, __bss_start
    la.abs  $t1, __bss_end
1:
    bge     $t0, $t1, 2f
    st.w    $zero, $t0, 0
    addi.d  $t0, $t0, 4
    b       1b
2:

    # Configure CRMD (Current Mode Register)
    # Set to PLV0 (most privileged), DA=1 (direct address mode initially)
    csrwr   $zero, 0x0             # CRMD: clear all, PLV0, DA=1, IE=0

    # Configure ECFG (Exception Config Register)
    # Set interrupt and exception entry
    li.d    $t0, 0
    csrwr   $t0, 0x4               # ECFG: clear all

    # Set exception entry address
    la.abs  $t0, exception_entry
    csrwr   $t0, 0xC               # EENTRY

    # Set TLB refill entry address
    la.abs  $t0, tlb_refill_entry
    csrwr   $t0, 0x88              # TLBRENTRY

    # Initialize page table directory base addresses
    la.abs  $t0, boot_page_table
    csrwr   $t0, 0x6               # PGDL (page directory base low)
    csrwr   $t0, 0x7               # PGDH (page directory base high)

    # Enable MMU: set CRMD.PG=1, CRMD.DA=0
    csrrd   $t0, 0x0               # Read CRMD
    ori     $t0, $t0, (1 << 0)     # Set PG=1 (MMU enable)
    andi    $t0, $t0, ~(1 << 3)    # Clear DA=0 (disable direct address)
    csrwr   $t0, 0x0               # Write CRMD

    # Invalidate TLB
    invtlb  0x0, $zero, $zero      # Invalidate all TLB entries

    # Jump to Rust kernel entry with boot info pointer
    move    $a0, $s0               # Pass boot info pointer as first argument
    bl      kmain

    # Should not reach here
    idle    0
    b       .

# Exception entry (stub - will be filled by exception handler setup)
.align 256
exception_entry:
    idle    0
    b       .

# TLB refill entry (stub - will be filled by page table handler)
.align 256
tlb_refill_entry:
    idle    0
    b       .

.size _start, . - _start

# Boot page table (skeleton - 3-level: PGD -> PMD -> PTE)
.section .data.boot, "aw", @progbits
.align 4096
boot_page_table:
    .space  0x3000                  # 3 pages for PGD + PMD + PTE

# Boot stack
.align 16
boot_stack_bottom:
    .space  0x10000                 # 64KB boot stack
boot_stack_top:
