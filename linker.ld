OUTPUT_FORMAT(elf64-x86-64)

ENTRY(kmain)

/* MEMORY */
/* { */
/*     RAM : ORIGIN = 0x00000000, LENGTH = 512M */
/* } */

/* mmu permissions for limine */
PHDRS {
    text PT_LOAD;
    rodata PT_LOAD;
    data PT_LOAD;
}

SECTIONS {
    . = 0xffffffff80000000;
    PROVIDE(_kernel_start = .);

    . = ALIGN(CONSTANT(MAXPAGESIZE));

    .text : {
        *(.text .text.*)
    } :text

    . = ALIGN(CONSTANT(MAXPAGESIZE));

    .rodata : {
        *(.rodata .rodata.*)
    } :rodata

    .psf_font : {
        KEEP(*(.psf_font))
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

    /* Stack should be last because we do not want unnecessary 0's */
    .bss : {
        *(.bss*)
        *(COMMON)
    } :data

    /* Align kernel end to page boundary and export it */
    PROVIDE(_kernel_end = .);
    . = ALIGN(4096);

	PROVIDE(_heap_start = .);
	PROVIDE(_heap_size = 1M);
    .heap (NOLOAD) : {
        . = . + _heap_size;  /* 16 MiB heap */
    }
	PROVIDE(_heap_end = .);


    /DISCARD/ : {
        *(.eh_frame*) /* this section does not work on all hosts */
        *(.note .note.*)
    }
}

