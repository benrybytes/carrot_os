use core::num::NonZero;

use ez_paging::{ManagedL4PageTable, Page, PageSize};
use nodit::{
    interval::{ii, iu},
    InclusiveInterval, Interval, NoditSet,
};

use crate::{
    memory::MemoryType,
    println, serial, serial_println,
    x86_64_consts::{HIGHER_HALF_START, LOWER_HALF_END},
};

#[derive(Debug)]
pub struct VirtualMemory {
    #[allow(unused)]
    pub(super) set: NoditSet<u64, Interval<u64>>,
    #[allow(unused)]
    pub(super) l4: ManagedL4PageTable,
}

impl VirtualMemory {
    /// Returns the start page of the allocated range of pages.
    /// Pages are guaranteed not to be mapped.
    pub fn allocate_contiguous_pages(
        &mut self,
        page_size: PageSize,
        n_pages: NonZero<u64>,
        memory_type: MemoryType,
    ) -> Option<Page> {
        let search_interval = if memory_type == MemoryType::UsedByUserMode {
            iu(0x0000_0000_0000_0000) // Search User space
        } else {
            iu(HIGHER_HALF_START) // Search Kernel space
        };
        let interval = self.set.gaps_trimmed(search_interval).find_map(|gap| {
            let aligned_start = gap.start().next_multiple_of(page_size.byte_len_u64());
            let interval = ii(
                aligned_start,
                aligned_start + (n_pages.get() * page_size.byte_len_u64() - 1),
            );
            if gap.contains_interval(&interval) {
                Some(interval)
            } else {
                None
            }
        })?;

        // 3. Fix the address calculation
        // Don't OR with the kernel prefix if it's user mode!
        let addr = if memory_type == MemoryType::UsedByUserMode {
            interval.start() // Keep the lower-half address as is
        } else {
            // Ensure kernel addresses are in the higher half
            interval.start() | 0xFFFF_8000_0000_0000
        };

        self.set
            .insert_merge_touching(interval)
            .expect("no overlap");
        Some(Page::new(x86_64::addr::VirtAddr::new(addr), page_size).expect("should be aligned"))
    }

    pub fn l4_mut(&mut self) -> &mut ManagedL4PageTable {
        &mut self.l4
    }
}
