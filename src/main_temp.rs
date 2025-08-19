#![no_std] // remove standard libary usage
#![no_main] // disable all Rust entry points and define ours later
#![feature(custom_test_frameworks)]
#![test_runner(carrot_os::test_runner)]
#![reexport_test_harness_main = "test_main"]
use core::arch::asm;


use carrot_os::{println, serial_print};
use core::panic::PanicInfo;
use bootloader::BootInfo;
use carrot_os::task::{Task, executor::Executor, keyboard};

use limine::BaseRevision;
use limine::request::{FramebufferRequest, RequestsEndMarker, RequestsStartMarker};

// all limine requests are marked with #[used], otherwise they may be removed by the compiler.
#[used]
// The .requests section allows limine to find the requests faster and more safely. | requests
// communicate from exectuable to limine bootloader
#[unsafe(link_section = ".requests")]
static BASE_REVISION: BaseRevision = BaseRevision::new();

#[used]
#[unsafe(link_section = ".requests")]
static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

// define the stand and end markers for Limine requests in lds file
#[used]
#[unsafe(link_section = ".requests_start_marker")]
static _START_MARKER: RequestsStartMarker = RequestsStartMarker::new();
#[used]
#[unsafe(link_section = ".requests_end_marker")]
static _END_MARKER: RequestsEndMarker = RequestsEndMarker::new();


extern crate alloc; // import again to not 
use alloc::boxed::Box;

// called on panic / makes our program abort
// #[cfg(not(test))]
// #[panic_handler]
// fn panic(info: &PanicInfo) -> ! {
//     println!("{}", info);
//     carrot_os::hlt_loop();
// }

// our panic handler in test mode
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    carrot_os::test_panic_handler(info);
}

// entry_point!(kernel_main);

#[unsafe(no_mangle)]
unsafe extern "C" fn kmain() -> ! {
    // All limine requests must also be referenced in a called function, otherwise they may be
    // removed by the linker.
    assert_eq!(BASE_REVISION.is_supported(), false);

    if let Some(framebuffer_response) = FRAMEBUFFER_REQUEST.get_response() {
        if let Some(framebuffer) = framebuffer_response.framebuffers().next() {
            for i in 0..100_u64 {
                // Calculate the pixel offset using the framebuffer information we obtained above.
                // We skip `i` scanlines (pitch is provided in bytes) and add `i * 4` to skip `i` pixels forward.
                let pixel_offset = i * framebuffer.pitch() + i * 4;

                // Write 0xFFFFFFFF to the provided pixel offset to fill it white.
                unsafe {
                    framebuffer
                        .addr()
                        .add(pixel_offset as usize)
                        .cast::<u32>()
                        .write(0xFFFFFFFF)
                };
            }
        }
    }
    let ptr = 0xb8000 as *mut u8;
    unsafe { *ptr = b'X'; } // write to VGA if paging not enabled
    hcf();
    loop {}

}

#[panic_handler]
fn rust_panic(_info: &core::panic::PanicInfo) -> ! {
    hcf();
}

fn hcf() -> ! {
    loop {
        unsafe {
            #[cfg(target_arch = "x86_64")]
            asm!("hlt");
            #[cfg(any(target_arch = "aarch64", target_arch = "riscv64"))]
            asm!("wfi");
            #[cfg(target_arch = "loongarch64")]
            asm!("idle 0");
        }
    }
}
#[unsafe(no_mangle)]
unsafe extern "C" fn _start() -> ! {

    // make sure limine is supported
    assert!(BASE_REVISION.is_supported());

    if let Some(framebuffer_response) = FRAMEBUFFER_REQUEST.get_response() {
        if let Some(framebuffer) = framebuffer_response.framebuffers().next() {
            for i in 0..100_u64 {
                // Calculate the pixel offset using the framebuffer information we obtained above.
                // We skip `i` scanlines (pitch is provided in bytes) and add `i * 4` to skip `i` pixels forward.
                let pixel_offset = i * framebuffer.pitch() + i * 4;

                // Write 0xFFFFFFFF to the provided pixel offset to fill it white.
                unsafe {
                    framebuffer
                        .addr()
                        .add(pixel_offset as usize)
                        .cast::<u32>()
                        .write(0xFFFFFFFF)
                };
            }
        }
    }

    // temporary debug
    // let ptr = 0xb8000 as *mut u8;
    // unsafe { *ptr = b'X'; } // write to VGA if paging not enabled
    loop {}

    // use carrot_os::{memory, allocator};
    // use x86_64::{VirtAddr, structures::paging::Page};
    //
    // // serial_print!("stack_overflow::stack_overflow...\t");
    //
    // // println!("hello world");
    //
    // carrot_os::init();
    //
    // // starting point / Cr3 pointer to our memory address
    // let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    // let mut mapper = unsafe { memory::init(phys_mem_offset) };
    //
    // // reminder: we guarantee memory_map is valid
    // let mut frame_allocator = unsafe {
    //     memory::BootInfoFrameAllocator::init(&boot_info.memory_map)
    // };
    //
    // // map unused page from somewhere | worse case scenario will create level >1 pages if missing
    // let page = Page::containing_address(VirtAddr::new(0));
    // memory::example_mapping(page, &mut mapper, &mut frame_allocator);
    //
    // let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
    // unsafe {
    //     page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e);
    // }
    //
    // allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap allocation failed");
    //
    // let heap_value = Box::new(40);
    // println!("heap_value at {:p}", heap_value);
    //
    // let mut executor = Executor::new();
    // executor.spawn(Task::new(example_task()));
    // executor.spawn(Task::new(keyboard::print_keypresses()));
    // executor.run();
    //
    // // stack_overflow();
    //
    // #[cfg(test)]
    // test_main();
}

async fn async_number() -> u32 {
    67
}

async fn example_task() {
    let number = async_number().await;
    println!("async number :3 {}", number);
}

#[allow(unconditional_recursion)]
fn _stack_overflow() {
    _stack_overflow(); // for each recursion, the return address is pushed
    volatile::Volatile::new(0).read(); // prevent tail recursion optimizations
}
