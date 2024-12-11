#![no_std] // remove standard libary usage
#![no_main] // disable all Rust entry points and define ours later


use core::panic::PanicInfo;

// called on panic / makes our program abort
#[panic_handler] 
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

// converting each character to a UTF-8 byte as that is what rust uses
static HELLO: &[u8] = b"Hello World!";

// don't mangle aka encrypt function output when we want to use it again
#[no_mangle]
pub extern "C" fn _start() -> ! {
    let vga_buffer = 0xb8000 as *mut u8; // unsafe mutable pointer

    for (i, &byte) in HELLO.iter().enumerate() { // iterate and reference character slice

        // unsafe: we are changing the pointer to be of this value
        unsafe {
            *vga_buffer.offset(i as isize * 2) = byte;

            // using pointer arithmitic to set color of text
            *vga_buffer.offset(i as isize * 2 + 1) = 0xb; // add a bit next to our character for color
        }
    }

    loop {}
}
