/* RISC-V 64-bit linker script - Nuva OS
 * Target: QEMU virt machine
 * Kernel loads at 0x80200000 (after OpenSBI at 0x80000000)
 */

OUTPUT_ARCH(riscv)
ENTRY(_start)

PHDRS
{
    text   PT_LOAD FLAGS(5);   /* Read + Execute */
    rodata PT_LOAD FLAGS(4);   /* Read only */
    data   PT_LOAD FLAGS(6);   /* Read + Write */
}

SECTIONS
{
    . = 0x80200000;

    .text : {
        *(.text.entry)
        *(.text .text.*)
    } :text

    .rodata : {
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
    } :rodata

    .data : {
        *(.data .data.*)
        *(.sdata .sdata.*)
        *(.data.boot)
    } :data

    .bss : {
        _bss_start = .;
        *(.bss .bss.*)
        *(.sbss .sbss.*)
        *(COMMON)
        _bss_end = .;
    } :data

    . = ALIGN(4096);
    _kernel_end = .;

    /DISCARD/ : {
        *(.comment)
        *(.note.*)
        *(.eh_frame*)
    }
}

PROVIDE(_kernel_start = 0x80200000);
