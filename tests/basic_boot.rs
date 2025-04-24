
#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    test_main();

    loop {}
}

// built in test mode always, no need to checking if test mode
fn test_runner(tests: &[&dyn Fn()]) {
    // for test in tests.
    unimplemented!();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // println!("{}", info);
    loop {}
}
