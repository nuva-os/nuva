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

/* * Nuva OS page table management assembly code
 * Target platform: ARM64 (Kirin9020)
 *
 * Functions:
 *   1. Page table creation and mapping
 *   2. TLB management
 *   3. Address translation
 */


#include <Nuva/asm_macros.S>

/* Page table attributes */
#define PT_VALID        (1 << 0)
#define PT_TABLE        (1 << 1)
#define PT_USER         (1 << 6)
#define PT_READONLY     (1 << 7)
#define PT_ACCESSED     (1 << 10)
#define PT_NG           (1 << 11)   /* Not Global */

/* Memory attribute indices */
#define MT_DEVICE_nGnRnE    0
#define MT_DEVICE_nGnRE     1
#define MT_NORMAL_NC        2
#define MT_NORMAL           3
#define MT_NORMAL_WT        4

/* Page table levels */
#define PG_LEVEL_4K_0       0   /* Level 0: 512GB per entry */
#define PG_LEVEL_4K_1       1   /* Level 1: 1GB per entry */
#define PG_LEVEL_4K_2       2   /* Level 2: 2MB per entry */
#define PG_LEVEL_4K_3       3   /* Level 3: 4KB per entry */

/* * Create page table entry
 * x0: Physical address
 * x1: Attributes
 * Return: x0 = Page table entry
 */
ENTRY(create_pte)
    /* Clear low bits */
    and     x0, x0, #~0xFFF
    
    /* Set attributes */
    orr     x0, x0, x1
    
    /* Set valid bit */
    orr     x0, x0, #PT_VALID
    
    ret
ENDPROC(create_pte)

/* * Map single page
 * x0: Page table base address
 * x1: Virtual address
 * x2: Physical address
 * x3: Attributes
 */
ENTRY(map_page)
    /* Save parameters */
    mov     x4, x0              /* Page table base address */
    mov     x5, x1              /* Virtual address */
    mov     x6, x2              /* Physical address */
    mov     x7, x3              /* Attributes */
    
    /* Level 0: Get PGD index */
    lsr     x0, x5, #39
    and     x0, x0, #0x1FF
    
    /* Check if PGD entry exists */
    ldr     x1, [x4, x0, lsl #3]
    cbnz    x1, 1f
    
    /* Allocate new PUD table */
    bl      alloc_page_table
    mov     x1, x0
    orr     x1, x1, #PT_TABLE | PT_VALID
    str     x1, [x4, x0, lsl #3]
    
1:
    /* Level 1: Get PUD index */
    and     x0, x5, #0x7FFFFFFF800
    lsr     x0, x0, #30
    and     x0, x0, #0x1FF
    
    /* Get PUD table address */
    and     x1, x1, #~0xFFF
    mov     x4, x1
    
    /* Check if PUD entry exists */
    ldr     x1, [x4, x0, lsl #3]
    cbnz    x1, 2f
    
    /* Allocate new PMD table */
    bl      alloc_page_table
    mov     x1, x0
    orr     x1, x1, #PT_TABLE | PT_VALID
    str     x1, [x4, x0, lsl #3]
    
2:
    /* Level 2: Get PMD index */
    and     x0, x5, #0x3FFFFFFF800
    lsr     x0, x0, #21
    and     x0, x0, #0x1FF
    
    /* Get PMD table address */
    and     x1, x1, #~0xFFF
    mov     x4, x1
    
    /* Check if PMD entry exists */
    ldr     x1, [x4, x0, lsl #3]
    cbnz    x1, 3f
    
    /* Allocate new PTE table */
    bl      alloc_page_table
    mov     x1, x0
    orr     x1, x1, #PT_TABLE | PT_VALID
    str     x1, [x4, x0, lsl #3]
    
3:
    /* Level 3: Get PTE index */
    and     x0, x5, #0x1FFFFF800
    lsr     x0, x0, #12
    and     x0, x0, #0x1FF
    
    /* Get PTE table address */
    and     x1, x1, #~0xFFF
    mov     x4, x1
    
    /* Create PTE */
    mov     x0, x6              /* Physical address */
    mov     x1, x7              /* Attributes */
    bl      create_pte
    
    /* Write PTE */
    str     x0, [x4, x0, lsl #3]
    
    ret
ENDPROC(map_page)

/* * Map large page (2MB)
 * x0: Page table base address
 * x1: Virtual address
 * x2: Physical address
 * x3: Attributes
 */
ENTRY(map_section)
    /* Save parameters */
    mov     x4, x0              /* Page table base address */
    mov     x5, x1              /* Virtual address */
    mov     x6, x2              /* Physical address */
    mov     x7, x3              /* Attributes */
    
    /* Level 0: Get PGD index */
    lsr     x0, x5, #39
    and     x0, x0, #0x1FF
    
    /* Check if PGD entry exists */
    ldr     x1, [x4, x0, lsl #3]
    cbnz    x1, 1f
    
    /* Allocate new PUD table */
    bl      alloc_page_table
    mov     x1, x0
    orr     x1, x1, #PT_TABLE | PT_VALID
    str     x1, [x4, x0, lsl #3]
    
1:
    /* Level 1: Get PUD index */
    and     x0, x5, #0x7FFFFFFF800
    lsr     x0, x0, #30
    and     x0, x0, #0x1FF
    
    /* Get PUD table address */
    and     x1, x1, #~0xFFF
    mov     x4, x1
    
    /* Check if PUD entry exists */
    ldr     x1, [x4, x0, lsl #3]
    cbnz    x1, 2f
    
    /* Allocate new PMD table */
    bl      alloc_page_table
    mov     x1, x0
    orr     x1, x1, #PT_TABLE | PT_VALID
    str     x1, [x4, x0, lsl #3]
    
2:
    /* Level 2: Get PMD index */
    and     x0, x5, #0x3FFFFFFF800
    lsr     x0, x0, #21
    and     x0, x0, #0x1FF
    
    /* Get PMD table address */
    and     x1, x1, #~0xFFF
    mov     x4, x1
    
    /* Create Section map (2MB) */
    mov     x0, x6              /* Physical address */
    mov     x1, x7              /* Attributes */
    bl      create_pte
    
    /* Write PMD */
    str     x0, [x4, x0, lsl #3]
    
    ret
ENDPROC(map_section)

/* * Flush TLB
 */
ENTRY(flush_tlb_all)
    dsb     sy
    tlbi    vmalle1is
    dsb     sy
    isb
    ret
ENDPROC(flush_tlb_all)

/* * Flush TLB for single address
 * x0: Virtual address
 */
ENTRY(flush_tlb_page)
    dsb     sy
    tlbi    vaae1is, x0
    dsb     sy
    isb
    ret
ENDPROC(flush_tlb_page)

/* * Flush instruction cache
 */
ENTRY(flush_icache_all)
    ic      ialluis
    dsb     sy
    isb
    ret
ENDPROC(flush_icache_all)

/* * Flush data cache (by address)
 * x0: Start address
 * x1: End address
 */
ENTRY(flush_dcache_range)
    dsb     sy
1:
    dc      civac, x0
    add     x0, x0, #64
    cmp     x0, x1
    b.lo    1b
    dsb     sy
    isb
    ret
ENDPROC(flush_dcache_range)

/* * Get current page table base address
 * Return: x0 = TTBR0_EL1
 */
ENTRY(get_current_pgtable)
    mrs     x0, ttbr0_el1
    and     x0, x0, #~0xFFF
    ret
ENDPROC(get_current_pgtable)

/* * Switch page table
 * x0: New page table base address
 */
ENTRY(switch_pgtable)
    dsb     sy
    msr     ttbr0_el1, x0
    isb
    tlbi    vmalle1is
    dsb     sy
    isb
    ret
ENDPROC(switch_pgtable)

/* * Allocate page table (simple implementation)
 * Return: x0 = Page table physical address
 */
ENTRY(alloc_page_table)
    /* Use statically allocated page table pool */
    ldr     x0, =page_table_pool
    ldr     x1, =page_table_pool_ptr
    ldr     x2, [x1]
    
    /* Check if exceeds pool size */
    ldr     x3, =page_table_pool_end
    cmp     x2, x3
    b.hs    1f
    
    /* Allocate one page */
    mov     x0, x2
    add     x2, x2, #0x1000
    str     x2, [x1]
    
    /* Zero page table */
    mov     x1, x0
    mov     x2, #0x1000
    mov     x3, xzr
2:
    str     xzr, [x1, x3, lsl #3]
    add     x3, x3, #1
    cmp     x3, x2, lsr #3
    b.lo    2b
    
    ret
    
1:
    /* Page table pool exhausted */
    b       .
ENDPROC(alloc_page_table)

/* * Page table pool
 */
.section ".data.page_table_pool", "aw", @progbits
.balign 4096

page_table_pool:
    .space  0x10000             /* 64KB page table pool */
page_table_pool_end:

page_table_pool_ptr:
    .quad   page_table_pool
