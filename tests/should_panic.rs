
#![no_std]
#![no_main]

use core::panic::PanicInfo;

use carrot_os::{exit_qemu, serial_println, QemuExitCode};

// allows to confirm it did indeed failed to run
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_println!("[ok]");
    exit_qemu(QemuExitCode::Success);
    loop {}
}

fn fail() {
    serial_println!("should_panic::should_fail...\t");
    assert_eq!(1, 0);
}

// be able to run without making it a test case
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    fail();
    serial_println!("[test did not panic]");
    exit_qemu(QemuExitCode::Failed);
    loop {}
}
