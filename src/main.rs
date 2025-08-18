#![no_std] // remove standard libary usage
#![no_main] // disable all Rust entry points and define ours later
#![feature(custom_test_frameworks)]
#![test_runner(carrot_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use carrot_os::println;
use core::panic::PanicInfo;
use bootloader::{entry_point, BootInfo};
// use carrot_os::task::{Task, simple_executor::SimpleExecutor, keyboard};
use carrot_os::task::{Task, executor::Executor, keyboard};

extern crate alloc; // import again to not 
use alloc::boxed::Box;

// called on panic / makes our program abort
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    carrot_os::hlt_loop();
}

// our panic handler in test mode
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    carrot_os::test_panic_handler(info);
}

entry_point!(kernel_main);

// don't mangle make _start be readable
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use carrot_os::{memory, allocator};
    use x86_64::{VirtAddr, structures::paging::Page};

    println!("hello world");

    carrot_os::init();

    // starting point / Cr3 pointer to our memory address
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };

    // reminder: we guarantee memory_map is valid
    let mut frame_allocator = unsafe {
        memory::BootInfoFrameAllocator::init(&boot_info.memory_map)
    };

    // map unused page from somewhere | worse case scenario will create level >1 pages if missing
    let page = Page::containing_address(VirtAddr::new(0));
    memory::example_mapping(page, &mut mapper, &mut frame_allocator);

    let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
    unsafe {
        page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e);
    }
    
    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap allocation failed");

    let heap_value = Box::new(40);
    println!("heap_value at {:p}", heap_value);

    let mut executor = Executor::new();
    executor.spawn(Task::new(example_task()));
    executor.spawn(Task::new(keyboard::print_keypresses()));
    executor.run();

    // stack_overflow();

    #[cfg(test)]
    test_main();
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
