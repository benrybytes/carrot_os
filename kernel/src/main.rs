#![no_std]
#![no_main]

extern crate alloc; // import again to not
use alloc::boxed::Box;
use core::panic::PanicInfo;
use kernel::task::{executor::Executor, keyboard, Task};
use kernel::x86_64_consts::HIGHER_HALF_START;
use kernel::{print, println, serial_println};

extern "C" {
    static _binary_Cyr_a8x16_psf_start: u8;
    static _binary_Cyr_a8x16_psf_end: u8;
    static _binary_Cyr_a8x16_psf_size: u8;
    static _kernel_end: u8;
    static _kernel_start: u8;
}
use kernel::limine_requests::{BASE_REVISION, MEMORY_MAP_REQUEST};

#[cfg(not(test))]
#[unsafe(no_mangle)]
unsafe extern "C" fn kmain() -> ! {
    // removed by the linker.
    assert!(BASE_REVISION.is_supported());
    kernel::init();

    // starting point / Cr3 pointer to our memory address
    if let Some(memory_map_response) = MEMORY_MAP_REQUEST.get_response() {
        kernel::memory::init_bsp(memory_map_response);
        let heap_value = Box::new('c');

        let mut executor = Executor::new();
        executor.spawn(Task::new(example_task()));
        executor.spawn(Task::new(keyboard::print_keypresses()));

        println! {"value: {}", heap_value};
        executor.run();
    }

    kernel::hlt_loop();
}
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("panic here: {}", info);
    kernel::hlt_loop();
}

// #[stack_overflow]
fn stack_overflow() -> ! {
    stack_overflow();
}

async fn async_number() -> u32 {
    67
}

async fn example_task() {
    let number = async_number().await;
}
