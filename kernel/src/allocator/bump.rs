use super::{align_up, Locked};
use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;

pub struct _BumpAllocator {
    heap_start: usize,
    heap_end: usize,
    next: usize,
    allocations: usize,
}

impl _BumpAllocator {
    pub const fn new() -> Self {
        _BumpAllocator {
            heap_start: 0,
            heap_end: 0,
            next: 0,
            allocations: 0,
        }
    }

    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.heap_start = heap_start;
        self.heap_end = heap_start + heap_size;
        self.next = heap_start;
    }
}

// unsafe impl GlobalAlloc for BumpAllocator {
//     // returns null pointer by default as a signal error
//     unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
//         let alloc_start = self.next;
//         self.next = alloc_start + layout.size(); // shift to next region of memory
//         self.allocations += 1;
//         alloc_start as *mut u8 // return the usize as a pointer to this memory address
//     }
//
//     unsafe fn dealloc(&self, _ptr: *mut u8,_layout: Layout) {
//         panic!("cannot call dealloc");
//     }
// }

unsafe impl GlobalAlloc for Locked<_BumpAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut bump = self.lock();

        let alloc_start = align_up(bump.next, layout.align());

        // prevent large allocations beyond out page layout size
        let alloc_end = match alloc_start.checked_add(layout.size()) {
            Some(end) => end,
            None => return null_mut(),
        };

        if alloc_start > bump.heap_end {
            null_mut()
        } else {
            bump.next = alloc_end;
            bump.allocations += 1;
            alloc_start as *mut u8
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        let mut bump = self.lock();

        bump.allocations -= 1;
        if bump.allocations == 0 {
            bump.next = bump.heap_start;
        }
    }
}
