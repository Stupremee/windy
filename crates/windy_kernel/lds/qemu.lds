/*
 * Linker script for creating RISC-V qemu binary.
 * vim:ft=ld
 */

OUTPUT_ARCH(riscv64gc)

ENTRY(_boot)

MEMORY
{
  ram : ORIGIN = 0x80000000, LENGTH = 128M
}

SECTIONS
{
  .text : {
    PROVIDE(__text_start = .);
    *(.text.init)
    *(.text)
    *(.text.*)
    . = ALIGN(4096);
    PROVIDE(__text_end = .);
  } >ram AT>ram :text

  PROVIDE(__global_pointer = .);

  .rodata : {
    PROVIDE(__rodata_start = .);
    *(.rodata)
    *(.rodata.*)
    PROVIDE(__rodata_end = .);
  } >ram AT>ram :text

  .data : {
    . = ALIGN(4096);
    PROVIDE(__data_start = .);
    *(.sdata)
    *(.sdata.*)

    *(.data)
    *(.data.*)
    PROVIDE(__data_end = .);

    /* `__trap_stack` is used inside the trap handler for saving the trap 
     * frame and used as the actual stack
     */
    . += 4096;
    PROVIDE(__trap_stack = .);
    . = ALIGN(8);
  } >ram AT>ram :data

  .bss : {
    PROVIDE(__bss_start = .);
    *(.sbss)
    *(.sbss.*)
    *(.bss)
    *(.bss.*)
    PROVIDE(__bss_end = .);
  } >ram AT>ram :bss

  PROVIDE(__memory_start = ORIGIN(ram));
  PROVIDE(__stack_start = __bss_end);
  PROVIDE(__stack_end = __stack_start + 0x80000);
  PROVIDE(__memory_end = ORIGIN(ram) + LENGTH(ram));

  PROVIDE(__heap_start = __stack_end);
  PROVIDE(__heap_size = __memory_end - __heap_start);
}
