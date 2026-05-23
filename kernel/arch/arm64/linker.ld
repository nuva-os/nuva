# ARM64 linker script - Nuva OS
# Target platform: Kirin9020 / Snapdragon 8 Gen 4

OUTPUT_ARCH(aarch64)
ENTRY(_start)

/* Memory layout */
MEMORY
{
    /* Boot code (1MB) */
    BOOT (rx)  : ORIGIN = 0x40080000, LENGTH = 1M

    /* Kernel code (16MB) */
    KERNEL (rx) : ORIGIN = 0x40180000, LENGTH = 16M

    /* Kernel data (8MB) */
    DATA (rw)  : ORIGIN = 0x41180000, LENGTH = 8M

    /* Kernel heap (256MB) */
    HEAP (rw)  : ORIGIN = 0x41980000, LENGTH = 256M

    /* Device memory */
    DEVICE (rw) : ORIGIN = 0x0A000000, LENGTH = 64M
}

/* Section definitions */
SECTIONS
{
    /* Boot code */
    .boot : {
        *(.boot)
        *(.boot.*)
    } > BOOT

    /* Kernel code */
    .text : {
        *(.text._start)
        *(.text .text.*)
    } > KERNEL

    /* Read-only data */
    .rodata : {
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
    } > KERNEL

    /* Exception vector table */
    .vectors : {
        *(.vectors)
    } > KERNEL

    /* Initialized data */
    .data : {
        *(.data .data.*)
        *(.sdata .sdata.*)
    } > DATA

    /* Uninitialized data */
    .bss : {
        __bss_start = .;
        *(.bss .bss.*)
        *(.sbss .sbss.*)
        *(COMMON)
        __bss_end = .;
    } > DATA

    /* Heap */
    .heap : {
        __heap_start = .;
        . = . + 256M;
        __heap_end = .;
    } > HEAP

    /* Stack */
    .stack (NOLOAD) : {
        __stack_bottom = .;
        . = . + 64K;
        __stack_top = .;
    } > DATA

    /* Discard unwanted sections */
    /DISCARD/ : {
        *(.comment)
        *(.note.*)
        *(.eh_frame*)
    }
}

/* Symbol exports */
PROVIDE(__kernel_start = ORIGIN(KERNEL));
PROVIDE(__kernel_end = ORIGIN(DATA) + LENGTH(DATA));
PROVIDE(__heap_size = LENGTH(HEAP));
PROVIDE(__stack_size = 64K);
