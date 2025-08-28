
use x86_64::{
    structures::paging::{
        mapper::MapToError, FrameAllocator, PageTableFlags, Mapper, Page, Size4KiB
    },
    VirtAddr,
};

pub mod bump;
pub mod fixed_size_block;

use fixed_size_block::FixedSizeBlockAllocator;

pub struct Locked<A> {
    inner: spin::Mutex<A>
}

impl<A> Locked<A> {
    pub const fn new(inner: A) -> Self {
        Locked {
            inner: spin::Mutex::new(inner)
        }
    }
    
    pub fn lock(&self) -> spin::MutexGuard<'_, A> {
        self.inner.lock()
    }
}

extern "C" {
    static _heap_start: u8;
    static _heap_end: u8;
}

pub fn init_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    let heap_start = unsafe { &_heap_start as *const u8 as u64 };
    let heap_end = unsafe { &_heap_end as *const u8 as u64 };
    let heap_size = heap_end - heap_start;
    let page_range = {
        let heap_start = VirtAddr::new(heap_start);
        let heap_end = VirtAddr::new(heap_end);
        let heap_start_page: Page<Size4KiB> = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };
    for page in page_range {
        let frame = frame_allocator.allocate_frame().ok_or(MapToError::FrameAllocationFailed)?;

        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

        // Attempt to map the page to the frame
        match unsafe { mapper.map_to(page, frame, flags, frame_allocator) } {
            Ok(mapper) => {
                // Flush the TLB if mapping is successful
                mapper.flush();
            },
            Err(_) => {
                continue;
            }
        }
    };

    unsafe {
        ALLOCATOR.lock().init(heap_start as usize, heap_size as usize);
    }

    Ok(())
}

fn align_up(addr: usize, align: usize) -> usize {
    let remainder = addr % align;
    if remainder == 0 {
        addr // addr already aligned
    } else {
        addr - remainder + align
    }
}

// applies to all crates
#[global_allocator]
static ALLOCATOR: Locked<FixedSizeBlockAllocator> = Locked::new(FixedSizeBlockAllocator::new());
