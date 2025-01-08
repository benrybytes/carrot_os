#![no_std] // remove standard libary usage
#![no_main] // disable all Rust entry points and define ours later


use core::panic::PanicInfo;

// use vga_text::print_value;
mod vga_text;

// called on panic / makes our program abort
#[panic_handler] 
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

// converting each character to a UTF-8 byte as that is what rust uses
// static HELLO: &[u8] = b"rust is the way";

// don't mangle make _start be readable
#[no_mangle]
pub extern "C" fn _start() -> ! {
    // let vga_buffer = 0xb8000 as *mut u8; // unsafe mutable pointer
    //
    // for (i, &byte) in HELLO.iter().enumerate() { // iterate and reference character slice
    //
    //     // unsafe: we are changing the pointer value making this unsafe
    //     // the plus one works because when we iterate, offset allows us to move the pointer and
    //     // each iteration will move when offset is called
    //     unsafe {
    //         *vga_buffer.offset(i as isize * 2) = byte;
    //
    //         // using pointer arithmitic to set color of text
    //         *vga_buffer.offset(i as isize * 2 + 1) = 0xc; // add a bit next to our character for color
    //     }
    // }

    // print_value();
    println!("ferris said hi{}\n\n", "!");

    loop {}
}
