use core::num::NonZero;

use ez_paging::{ManagedL4PageTable, Page, PageSize};
use nodit::{
    interval::{ei, ii, iu},
    InclusiveInterval, Interval, NoditSet,
};

use crate::{
    memory::MemoryType,
    println, serial, serial_println,
    x86_64_consts::{KERNEL_SPACE_END, KERNEL_SPACE_START, USER_SPACE_END, USER_SPACE_START},
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
        // 3. Fix the address calculation
        // Don't OR with the kernel prefix if it's user mode!
        let search_interval = if memory_type == MemoryType::UsedByUserMode {
            ei(USER_SPACE_START, USER_SPACE_END)
        } else {
            ei(KERNEL_SPACE_START, KERNEL_SPACE_END)
        };
        let interval = self.set.gaps_trimmed(search_interval).find_map(|gap| {
            let aligned_start = gap.start().next_multiple_of(page_size.byte_len_u64());
            let size = n_pages.get() * page_size.byte_len_u64();
            let end = aligned_start + (size - 1);

            // CRITICAL: Ensure we didn't spill into the non-canonical gap
            if memory_type == MemoryType::UsedByUserMode && end > USER_SPACE_END {
                return None;
            }

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

        let addr = interval.start();

        self.set
            .insert_merge_touching(interval)
            .expect("no overlap");
        Some(Page::new(x86_64::addr::VirtAddr::new(addr), page_size).expect("should be aligned"))
    }

    pub fn l4_mut(&mut self) -> &mut ManagedL4PageTable {
        &mut self.l4
    }
}
