#![no_std] // remove standard libary usage
#![no_main] // disable all Rust entry points and define ours later
#![feature(custom_test_frameworks)]
#![test_runner(carrot_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use carrot_os::println;
use core::panic::PanicInfo;
use bootloader::{entry_point, BootInfo};

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
    use x86_64::{structures::paging::Translate, VirtAddr, structures::paging::Page};

    // println!("how do you like them apples, aka rust macros{}\n\n", "!");
    println!("hello world");

    carrot_os::init();

    // starting point / Cr3 pointer to our memory address
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    
    let addresses = [
        // the identity-mapped vga buffer page
        0xb8000,
        // some code page
        0x201008,
        // some stack page
        0x0100_0020_1a10,
        // virtual address mapped to physical address 0
        boot_info.physical_memory_offset,
    ];
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    // let mut frame_allocator = memory::EmptyFrameAllocator;
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

    // for &address in &addresses {
    //     let virt = VirtAddr::new(address);
    //     let phys = mapper.translate_addr(virt); // handles large pages
    //     println!("{:?} -> {:?}", virt, phys);
    // }

    #[cfg(test)]
    test_main();

    println!("interrupt was handled");

    carrot_os::hlt_loop();
}
