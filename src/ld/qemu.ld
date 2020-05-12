OUTPUT_ARCH(riscv)
ENTRY(entry)

BASE_ADDRESS = 0x80000000;

SECTIONS
{
  . = BASE_ADDRESS;
  start = .;

  .text : {
    *(.text.entry)
    .ALIGN = 0x4;
    *(.text.trap)
    *(.text .text.*)
  }

  .rodata : {
    *(.rodata .rodata.*)
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
}
