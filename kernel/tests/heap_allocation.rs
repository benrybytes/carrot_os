#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(kernel::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use alloc::{boxed::Box, rc::Rc, vec, vec::Vec};
use core::panic::PanicInfo;
use kernel::allocator::_heap_size;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kernel::test_panic_handler(info)
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
    assert_eq!(vector_test.iter().sum::<u64>(), (n - 1) * n / 2);
}

#[test_case]
fn many_boxes() {
    for i in 0..100000 {
        let x = Box::new(i);
        assert_eq!(*x, i);
    }
}

#[test_case]
fn many_boxes_long_lived() {
    let long_lived = Box::new(1);
    unsafe {
        for i in 0.._heap_size {
            let x = Box::new(i);
            assert_eq!(*x, i);
        }
    }
    assert_eq!(*long_lived, 1); // check if lived until end of program by reusing previous memory
}
