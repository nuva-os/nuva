# X64 linker script - Nuva OS
# Target platform: Intel Core / AMD Ryzen

OUTPUT_ARCH(i386:x86-64)
ENTRY(_start)

/* Memory layout */
MEMORY
{
    /* Boot code (1MB) */
    BOOT (rx)  : ORIGIN = 0x100000, LENGTH = 1M

    /* Kernel code (16MB) */
    KERNEL (rx) : ORIGIN = 0x200000, LENGTH = 16M

    /* Kernel data (8MB) */
    DATA (rw)  : ORIGIN = 0x1200000, LENGTH = 8M

    /* Kernel heap (256MB) */
    HEAP (rw)  : ORIGIN = 0x1A00000, LENGTH = 256M
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
    } > KERNEL

    /* Initialized data */
    .data : {
        *(.data .data.*)
    } > DATA

    /* Uninitialized data */
    .bss : {
        __bss_start = .;
        *(.bss .bss.*)
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
