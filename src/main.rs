#![no_std] // remove standard libary usage
#![no_main] // disable all Rust entry points and define ours later
#![feature(custom_test_frameworks)]
#![test_runner(carrot_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use carrot_os::println;
use core::panic::PanicInfo;
use bootloader::{entry_point, BootInfo};

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
    use carrot_os::memory::active_level_4_table;
    use x86_64::VirtAddr;

    // println!("how do you like them apples, aka rust macros{}\n\n", "!");
    println!("hello world");

    carrot_os::init();

    // starting point / Cr3 pointer to our memory address
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let l4_table = unsafe { active_level_4_table(phys_mem_offset) };

    for(i, entry) in l4_table.iter().enumerate() {
        if !entry.is_unused() {
            println!("L4 entry {}: {:?}", i, entry);
        }
    }

    // // breakpoint to test if it is handled inside IDT
    // x86_64::instructions::interrupts::int3();

    // trigger a page fault
    // unsafe {
    //     *(0xdeadbeef as *mut u8) = 42;
    // };
    use x86_64::registers::control::Cr3;

    let (level_4_page_table, _) = Cr3::read();
    println!("Level 4 page table at: {:?}", level_4_page_table.start_address());


    #[cfg(test)]
    test_main();

    println!("interrupt was handled");

    carrot_os::hlt_loop();
}
