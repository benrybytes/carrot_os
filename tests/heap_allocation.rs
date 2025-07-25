
#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(carrot_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use carrot_os::allocator::HEAP_SIZE;
use alloc::{boxed::Box, vec, vec::Vec, rc::Rc};

entry_point!(main);

fn main(boot_info: &'static BootInfo) -> ! {
    use carrot_os::allocator;
    use carrot_os::memory::{self, BootInfoFrameAllocator};
    use x86_64::VirtAddr;

    carrot_os::init();
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe {
        BootInfoFrameAllocator::init(&boot_info.memory_map)
    };
    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("could not initialize heap");

    test_main();
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    carrot_os::test_panic_handler(info)
}

#[test_case]
fn simple_allocation() {
    let value_one = Box::new(10);
    assert_eq!(*value_one, 10);
}

#[test_case]
fn vector_checker() {
    let n = 1000;
    let mut vector_test = Vec::new();
    for i in 0..n {
        vector_test.push(i);
    }
    assert_eq!(vector_test.iter().sum::<u64>(), (n-1) * n / 2);
}

#[test_case]
fn many_boxes() {
    for i in 0..100000 {
        let x = Box::new(i);
        assert_eq!(*x, i);
    }
}
