#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use core::panic::PanicInfo;

extern crate alloc; // import again to use allocation

use call_with_rsp::*;
use cpu::*;
use filesystem::*;
use gdt::*;
use hhdm_offset::*;
use interrupts::*;
use limine_requests::*;
use memory::*;
use serial::*;
use stack::*;
use syscall_handler::*;
use task::*;
use text::*;
use translate_addr::*;
use x86_64_consts::*;

mod call_with_rsp;
mod cpu;
mod filesystem;
mod gdt;
mod hhdm_offset;
mod interrupts;
mod limine_requests;
mod memory;
mod serial;
mod stack;
mod syscall_handler;
mod task;
mod text;
mod translate_addr;
mod x86_64_consts;

#[unsafe(no_mangle)]
unsafe extern "C" fn kmain() -> ! {
    assert!(BASE_REVISION.is_supported());

    // starting point / Cr3 pointer to our memory address
    let memory_map_response = MEMORY_MAP_REQUEST.get_response().unwrap();

    memory::init_bsp(memory_map_response);
    cpu::init_bsp();
    let local = get_local();
    Stack::new(
        StackId {
            _type: StackType::Normal,
            cpu_id: local.kernel_assigned_id,
        },
        KERNEL_NORMAL_STACK_SIZE,
    )
    .switch(init_bsp);
}

extern "C" fn init_bsp() -> ! {
    use crate::executor::Executor;
    use alloc::boxed::Box;
    use interrupts::InterruptIndex;
    use x86_64::structures::idt::InterruptDescriptorTable; // runtime statics
    gdt::init();
    interrupts::init();

    let mut executor = Executor::new();
    executor.spawn(Task::new(keyboard::print_keypresses()));

    executor.run();
}

// halt cpu until next interrupt arrives
pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("panic here: {}", info);
    crate::hlt_loop();
}
