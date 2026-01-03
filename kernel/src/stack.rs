use core::{mem, num::NonZero};

use alloc::collections::btree_map::BTreeMap;
use ez_paging::{ConfigurableFlags, Page, PageSize};
use x86_64::{
    registers::model_specific::PatMemoryType, structures::paging::PageTableFlags, VirtAddr,
};

use crate::{
    call_with_rsp,
    memory::{KernelMemoryUsageType, MemoryType, MEMORY},
    println,
    x86_64_consts::{LOWER_HALF_END, USER_SPACE_END},
};

pub const KERNEL_NORMAL_STACK_SIZE: u64 = 64 * 0x400;
pub const USER_NORMAL_STACK_SIZE: u64 = 64 * 0x400;
pub const EXCEPTION_HANDLER_STACK_SIZE: u64 = 64 * 0x400;
pub const STACK_PAGE_SIZE: PageSize = PageSize::_4KiB;
pub static STACK_GUARD_PAGES: spin::Mutex<BTreeMap<Page, StackInfo>> =
    spin::Mutex::new(BTreeMap::new());

pub struct Stack {
    top: VirtAddr,
}

#[derive(Debug, Clone, Copy)]
pub enum StackType {
    Normal,
    ExceptionHandler,
    SyscallHandler,
}

#[derive(Debug, Clone, Copy)]
pub struct StackId {
    pub _type: StackType,
    pub cpu_id: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct StackInfo {
    id: StackId,
    size: u64,
}

impl Stack {
    pub fn stack_maker(id: StackId, size: u64, memory_type: MemoryType) -> (Page, u64) {
        // get memory to create a page table for our stack
        let memory = MEMORY.get().unwrap();
        let mut physical_memory = memory.physical_memory.lock();
        let mut virtual_memory = memory.virtual_memory.lock();

        let n_mapped_pages = size.div_ceil(STACK_PAGE_SIZE.byte_len_u64());
        let n_virtual_pages = n_mapped_pages + 1;

        // gets pages ready for mapping to physical memory
        let allocated_pages = virtual_memory
            .allocate_contiguous_pages(
                STACK_PAGE_SIZE,
                NonZero::new(n_virtual_pages).unwrap(),
                memory_type,
            )
            .unwrap();

        let guard_page = Page::new(allocated_pages.start_addr(), STACK_PAGE_SIZE).unwrap();
        STACK_GUARD_PAGES
            .lock()
            .insert(guard_page, StackInfo { id, size });
        // allow offset(0) be for page fault purposes
        let (start_page, l4) = (guard_page.offset(1).unwrap(), virtual_memory.l4_mut());

        let flags = ConfigurableFlags {
            writable: true,
            executable: false,
            pat_memory_type: PatMemoryType::WriteBack,
        };
        // create frames for our pages
        for i in 0..n_mapped_pages {
            let page = start_page.offset(i).unwrap();
            let frame = physical_memory
                .allocate_frame_with_type(STACK_PAGE_SIZE, memory_type)
                .unwrap();
            let mut frame_allocator = match memory_type {
                MemoryType::UsedByUserMode => {
                    physical_memory.get_user_mode_program_frame_allocator()
                }
                _ => physical_memory.get_kernel_frame_allocator(),
            };

            match memory_type {
                // have to get the managed page table for user access
                MemoryType::UsedByUserMode => {
                    let mut l4 = l4.new_user(
                        frame_allocator
                            .allocate_4kib_frame()
                            .expect("could not allocate _4KiB frame"),
                    );
                    unsafe {
                        l4.switch_to(memory.new_kernel_cr3_flags);
                    }
                    // Safety: we would have to handle any page faults if invalid permissions occur
                    unsafe { l4.map_page(page, frame, flags, &mut frame_allocator) }
                        .expect("could not map page");
                }
                // just continue with our original page table
                _ => {
                    // Safety: we would have to handle any page faults if invalid permissions occur
                    unsafe { l4.map_page(page, frame, flags, &mut frame_allocator) }
                        .expect("could not map page");
                }
            }
        }
        (start_page, n_mapped_pages)
    }
    pub fn new(id: StackId, size: u64, memory_type: MemoryType) -> Stack {
        // remember, we want to start at the last page, and build ourselves down due to FIFO
        // principle for stack
        let (start_page, n_mapped_pages) = Stack::stack_maker(id, size, memory_type);
        Self {
            top: (start_page.start_addr() + n_mapped_pages * STACK_PAGE_SIZE.byte_len_u64()),
        }
    }

    pub fn top(&self) -> VirtAddr {
        self.top
    }

    // switch to this stack
    pub fn switch(self, f: extern "C" fn() -> !) -> ! {
        let new_rsp = self.top.as_u64();
        // Safety: The worst that can happen is a stack overflow, since we mapped a guard page
        unsafe { call_with_rsp(new_rsp, f) }
    }
}
