
# paging

allows memory protection of programs to be isolated in memory regions

x86 has **segmentation** and **paging** to deal with memory protection of programs

## segmentation

works by loading seperate GDT | known as **protected mode**

### virtual memory

with a seperate GDT, it stores an address offset used to access the physical memory address

the process to map it is used in a **translation function**

## fragmentation

problem, if physical address does not have a segement, it will copy all and squeeze
the running programs back and put the new program in

## paging solution

splitting the segmenents into **frames**, aka little chunks of memory all mapped
into the larger physical address space

**internal fragmentaion** is used by occupying a bit more space, sort of like structs in their ordering
to get more memory

101 bytes needed, 3 frames, so 50 bytes and 49 bytes extra


