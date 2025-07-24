
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

## page tables

millions of pages with frames mapping to different fragments of a page, then
mapped to the physical memory

### two-level page tables

will create more levels, with level 1 pages being sub-pages of level 2 pages and so on

where, [0, 10000] bytes would be on level 1 page and so on
however, the frames are the ones that determine the page it is on

page 0 for T2 would start in frame 100 due to it serving as an offset to where
to match in physical address

## paging in x86_64

can only support 48 bits for virtual memory, and 52 bits for physical memory

the remaining bits can be used in the future for other stuff

map the virtual memory using CR3 register to where to start
mapping to level 4 table, where it maps to level 3, until level 1

level 2 - 4 frames store frames too in physical address

level 1 will map the final addresses in the frames it stores for us to use

**virtual memory**: 

the data / bits it stores in value, is where the current table is located, but
the frame is read to where it is in physical memory

remember, each index and frame is pointing to a page of the -1 level, until it reaches
level 1

{sign extension 0 * 12 bits}{level 4 0 * 9 bits}
{level 3 0 * 9 bits}{level 2 0 * 9 bits}{level 1 0 * 9 bits}
{offset for mapping to physical address 0 * 12 bits}

## translation lookaside buffer

moving 4 page translations is expensive, so TLB is used to store caches of translationsand skips translations when cached

instruction invlpg is used to check if we can skip a translation using TLB

important: TLB needs to be flushed on every page table modification, to not cause bugs

# adding mapping

each virtual memory section would point to the whole physical memory, but an offset would be placed like +10 TiB

## boot information

- memory_map loaded by BIOS or UEFI firmware in order to tell us VGA hardware and should load in bootloader so kernel can access items readily, meaning we need to place before starting the operating system
- physical_memory_offset must be >1 TiB to not cause corruption with hardware memory

```rs
entry_point();
```
