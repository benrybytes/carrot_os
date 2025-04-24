#![no_std] // remove standard libary usage
#![no_main] // disable all Rust entry points and define ours later

#![feature(custom_test_frameworks)]
#![test_runner(operating_system::test_runner)]
#![reexport_test_harness_main = "test_main"]

use operating_system::println;
use core::panic::PanicInfo;

// called on panic / makes our program abort
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {

    println!("{}", info);
    loop {}
}

// our panic handler in test mode
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    operating_system::test_panic_handler(info);
}
// don't mangle make _start be readable
#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("how do you like them apples, aka rust macros{}\n\n", "!");

    #[cfg(test)]
    test_main();

    loop {}
}
