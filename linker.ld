
OUTPUT_FORMAT(elf64-x86-64)

ENTRY(kmain)

/* mmu permissions for limine */
PHDRS {
    text PT_LOAD;
    rodata PT_LOAD;
    data PT_LOAD;
}

SECTIONS {
	. = 0xffffffff80000000;

	. = ALIGN(CONSTANT(MAXPAGESIZE));

	.text : {
		*(.text .text.*)
	} :text

	. = ALIGN(CONSTANT(MAXPAGESIZE));

	.rodata : {
		*(.rodata .rodata.*)
	} :rodata

	.note.gnu.build-id : {
		*(.note.gnu.build-id)
	} :rodata

	. = ALIGN(CONSTANT(MAXPAGESIZE));

	.data : {
		*(.data .data.*)

		/* place limine requests at data as we are going to initialize location inside main.rs */
        KEEP(*(.requests_start_marker))
        KEEP(*(.requests))
        KEEP(*(.requests_end_marker))
	} :data

	/* stack should be last because we do not want unnecessary 0's, unless we use init_array to use those 0's */
	.bss : {
		*(.bss .bss.*)
		*(COMMON)
	} :data

	/DISCARD/ : {
        *(.eh_frame*) /* this section does not work on all hosts */
        *(.note .note.*)
    }

}
