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
    use carrot_os::memory::translate_addr;
    use x86_64::VirtAddr;

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

    for &address in &addresses {
        let virt = VirtAddr::new(address);
        let phys = unsafe { translate_addr(virt, phys_mem_offset) };
        println!("{:?} -> {:?}", virt, phys);
    }
    
    // let l4_table = unsafe { active_level_4_table(phys_mem_offset) };
    //
    // for(i, entry) in l4_table.iter().enumerate() {
    //     if !entry.is_unused() {
    //         println!("L4 entry {}: {:?}", i, entry);
    //     }
    // }



    #[cfg(test)]
    test_main();

    println!("interrupt was handled");

    carrot_os::hlt_loop();
}
