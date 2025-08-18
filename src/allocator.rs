
use alloc::alloc::{GlobalAlloc, Layout};
// use linked_list_allocator::LockedHeap;
use core::ptr::null_mut;
use x86_64::{
    structures::paging::{
        mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB,
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
    
    pub fn lock(&self) -> spin::MutexGuard<A> {
        self.inner.lock()
    }
}

// use bump::_BumpAllocator;

pub const HEAP_START: usize = 0x_4444_4444_0000;
pub const HEAP_SIZE: usize = 100 * 1024; // 100 KiB

struct _DummyAlloc;

unsafe impl GlobalAlloc for _DummyAlloc {
    // returns null pointer by default as a signal error
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        null_mut()
    }

    unsafe fn dealloc(&self, _ptr: *mut u8,_layout: Layout) {
        panic!("cannot call dealloc");
    }
    
}

pub fn init_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>
) -> Result<(), MapToError<Size4KiB>> {
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE as u64 - 1u64;
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    for page in page_range {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        // update TLB
        unsafe {
            mapper.map_to(page, frame, flags, frame_allocator)?.flush()
        };
    }

    unsafe {
        ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
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
// static ALLOCATOR: DummyAlloc = DummyAlloc;
// spinlock to prevent deadlocks | no memory, but fill later
// static ALLOCATOR: LockedHeap = LockedHeap::empty();
// static ALLOCATOR: Locked<BumpAllocator> = Locked::new(BumpAllocator::new());
static ALLOCATOR: Locked<FixedSizeBlockAllocator> = Locked::new(FixedSizeBlockAllocator::new());
