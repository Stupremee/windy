/*
 * Linker script for creating RISC-V qemu binary.
 * vim:ft=ld
 */

OUTPUT_ARCH(riscv64gc)

ENTRY(_boot)

MEMORY
{
  /*
   * Add 0x200000 here because OpenSBI will jump to this address since we are using fw_dynamic
   *
   * NOTE: Do not change format of `LENGTH = <x>M`, otherwise the build script will fail.
   */
  ram : ORIGIN = 0x80000000 + 0x200000, LENGTH = 128M
}

SECTIONS
{
  .text : {
    PROVIDE(__kernel_start = .);

    PROVIDE(__text_start = .);
    *(.text.init)
    *(.text)
    *(.text.*)
    . = ALIGN(0x1000);
    PROVIDE(__text_end = .);
  } >ram AT>ram :text

  PROVIDE(__global_pointer = .);

  .rodata : {
    PROVIDE(__rodata_start = .);
    *(.rodata)
    *(.rodata.*)
    . = ALIGN(0x1000);
    PROVIDE(__rodata_end = .);
  } >ram AT>ram :text

  .data : {
    PROVIDE(__data_start = .);
    *(.sdata)
    *(.sdata.*)

    *(.data)
    *(.data.*)
    . = ALIGN(0x1000);
    PROVIDE(__data_end = .);

    /* `__trap_stack` is used inside the trap handler for saving the trap 
     * frame and used as the actual stack
     */
    . += 4096;
    PROVIDE(__trap_stack = .);
  } >ram AT>ram :data

  .bss : {
    PROVIDE(__bss_start = .);
    *(.sbss)
    *(.sbss.*)
    *(.bss)
    *(.bss.*)
    . = ALIGN(8);
    PROVIDE(__bss_end = .);
  } >ram AT>ram :bss

  . += 0x80000;
  PROVIDE(__stack_end = .);

  PROVIDE(__kernel_end = ALIGN(0x1000));

  /DISCARD/ : { *(.eh_frame_hdr .eh_frame) }
}
