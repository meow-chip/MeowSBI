OUTPUT_ARCH(riscv)
ENTRY(entry)

BASE_ADDRESS = 0x80000000;

SECTIONS
{
  . = BASE_ADDRESS;
  PROVIDE(_fw_start = .);

  .text : {
    *(.text.entry)
    .ALIGN = 0x4;
    *(.text.trap)
    *(.text .text.*)
  }

  .rodata : {
    *(.rodata .rodata.*)
    *(.rodata.payload)
  }

  .sdata : {
    *(.sdata .sdata.*)
  }

  .data : {
    *(.data .data.*)
  }

  .bss : {
    *(.bss .bss.*)
  }

  .sbss : {
    *(.sbss .sbss.*)
  }

  PROVIDE(_fw_end = .);

  .payload : {
    PROVIDE(_payload_start = .);
    *(.payload)
    PROVIDE(_payload_end = .);
  }
}
