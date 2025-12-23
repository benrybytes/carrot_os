OUTPUT_FORMAT(elf64-x86-64)
ENTRY(kmain)

MEMORY
{
  rom (rx)  : ORIGIN = 0xffffffff80000000, LENGTH = 10M
  ram (rwx) : ORIGIN = 0xffffffff80500000, LENGTH = 5M
}

PHDRS {
    text PT_LOAD;
    rodata PT_LOAD;
    data PT_LOAD;
}

STACK_SIZE = 0x2000;

SECTIONS
{
    . = ORIGIN(rom);
    _kernel_start = .;

	.text : ALIGN(4K) {
	  *(.text*)
	  *(.rodata*)
	}

	. = ALIGN(CONSTANT(MAXPAGESIZE));

    .rodata : {
        *(.rodata .rodata.*)
    } :rodata

    .psf_font : {
        KEEP(*(.psf_font))
    } > rom :rodata

    .note.gnu.build-id : {
        *(.note.gnu.build-id)
    } > rom :rodata

	. = ALIGN(CONSTANT(MAXPAGESIZE));

    /* Place .data/.bss/stack/heap in RAM (high-half) */
    . = ORIGIN(ram);
    .data : {
		*(.data .data.*)

        /* place limine requests at data as we are going to initialize location inside main.rs */
        KEEP(*(.requests_start_marker))
        KEEP(*(.requests))
        KEEP(*(.requests_end_marker))
    } > ram AT > rom :data    /* AT > rom allowed because LMAs will be placed after ROM data in file */

    .bss (NOLOAD) : {
        *(.bss*)
        *(COMMON)
    } > ram :data

	.stack (NOLOAD) : {
		. = ALIGN(8);
		_stack_start = .;
		. = . + STACK_SIZE;
		_stack_end = .;
		. = ALIGN(8);
	} > ram

	.stack_low : {
		. = ALIGN(16);
		_stack_low_start = .;       /* bottom of the stack */
		. = . + 0x4000;            /* 16 KiB stack */
		_stack_end_low = .;         /* top of the stack */
	} > ram

	_kernel_end = .;
	. = ALIGN(4096);

    /* heap */
    _heap_start = .;
    .heap (NOLOAD) : {
        . = . + 1M;
    } > ram :data
	_heap_end = .;

	/* If we're on a newer compiler */
	/DISCARD/ : {
	  *(.interp)
	  *(.dynsym)
	  *(.dynstr)
	  *(.hash)
	  *(.gnu.hash)
	  *(.header)
	} : phdr
}

