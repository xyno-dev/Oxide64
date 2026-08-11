ENTRY(_start)

SECTIONS
{
	. = 2M;

	.text ALIGN(4K) :
	{
		*(.multiboot2)
		*(.text .text.*)
	}

	.rodata ALIGN(4K) :
	{
		*(.rodata .rodata.*)
	}

	.data ALIGN(4K) :
	{
		*(.data .data.*)
	}

	.bss ALIGN(4K) :
	{
		*(COMMON)
		*(.bss .bss.*)
	}

    .data.rel.ro : {
        *(.data.rel.ro.local*) *(.data.rel.ro .data.rel.ro.*)
    }
}
