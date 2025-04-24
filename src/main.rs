#![no_std] // remove standard libary usage
#![no_main] // disable all Rust entry points and define ours later

#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

mod vga_text;
mod serial;

pub trait Testable {
    fn run(&self) -> ();
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}


// testable functions
impl<T> Testable for T
where T: Fn() {
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>()) ;
        self();
        serial_println!("[ok]");
    }
}


#[cfg(test)]
fn test_runner(tests: &[&dyn Fn()]) {
    serial_println!("running {} tests", tests.len());
    for test in tests {
        test();
    }

    exit_qemu(QemuExitCode::Success);
}


pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}


#[test_case]
fn assertion() {
    serial_print!("testing...");
    assert_eq!(1,1);
    serial_print!("[ok]");
}


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
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    loop {}
}
// don't mangle make _start be readable
#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("how do you like them apples, aka rust macros{}\n\n", "!");

    #[cfg(test)]
    test_main();

    loop {}
}
