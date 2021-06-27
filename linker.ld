OUTPUT_ARCH("riscv");
ENTRY(_entry);

SECTIONS
{
  . = 0x80000000;

  .text : {
    *(.text.entry);
    *(.text .text.*);
  }

  . = ALIGN(0x1000);

  .rodata : {
    . = ALIGN(16);
    *(.srodata .srodata.*) /* do not need to distinguish this from .rodata */
    . = ALIGN(16);
    *(.rodata .rodata.*)
  }

  .data : {
    . = ALIGN(16);
    _sdata = .;
    *(.sdata .sdata.*) /* do not need to distinguish this from .data */
    . = ALIGN(16);
    *(.data .data.*)
    _edata = .;
  }

  .bss : {
    . = ALIGN(16);
    _sbss = .;
    *(.sbss .sbss.*) /* do not need to distinguish this from .bss */
    . = ALIGN(16);
    *(.bss .bss.*)
    _ebss = .;
  }
};
