#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

extern crate alloc; // import again to not
use crate::executor::Executor;
use alloc::boxed::Box;
use core::panic::PanicInfo;

extern "C" {
    static _binary_Cyr_a8x16_psf_start: u8;
    static _binary_Cyr_a8x16_psf_end: u8;
    static _binary_Cyr_a8x16_psf_size: u8;
    static _kernel_end: u8;
    static _kernel_start: u8;
}
use call_with_rsp::*;
use filesystem::*;
use gdt::*;
use hhdm_offset::*;
use interrupts::*;
use limine_requests::*;
use memory::*;
// use run_program_0::*;
use serial::*;
use stack::*;
// use syscall_handler::*;
use task::*;
use text::*;
use translate_addr::*;
// use usermode::*;
use x86_64_consts::*;

mod call_with_rsp;
mod filesystem;
mod gdt;
mod hhdm_offset;
mod interrupts;
mod limine_requests;
mod memory;
// mod run_program_0;
mod serial;
mod stack;
// mod syscall_handler;
mod task;
mod text;
mod translate_addr;
// mod usermode;
mod x86_64_consts;

#[cfg(not(test))]
#[unsafe(no_mangle)]
unsafe extern "C" fn kmain() -> ! {
    assert!(BASE_REVISION.is_supported());

    // starting point / Cr3 pointer to our memory address
    let memory_map_response = MEMORY_MAP_REQUEST.get_response().unwrap();

    memory::init_bsp(memory_map_response);
    Stack::new(KERNEL_NORMAL_STACK_SIZE).switch(init_bsp);
}
extern "sysv64" fn init_bsp() -> ! {
    use interrupts::InterruptIndex;
    use x86_64::structures::idt::InterruptDescriptorTable; // runtime statics
    gdt::init();

    // allow time for GDT to be initialized with its segments, else general protection fault inside
    // double fault occur
    let idt = interrupts::IDT_CELL.get_or_init(|| {
        let mut idt = InterruptDescriptorTable::new();

        for i in 33..48 {
            idt[i].set_handler_fn(interrupts::unexpected_irq_handler);
        }
        unsafe {
            idt.double_fault
                .set_handler_fn(interrupts::double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt.breakpoint
            .set_handler_fn(interrupts::breakpoint_handler);
        idt.page_fault
            .set_handler_fn(interrupts::page_fault_handler);
        idt.general_protection_fault
            .set_handler_fn(interrupts::general_protection_fault);
        idt[InterruptIndex::Keyboard as u8].set_handler_fn(interrupts::keyboard_interrupt_handler);
        idt[InterruptIndex::Timer.as_u8()].set_handler_fn(interrupts::timer_interrupt_handler);

        idt
    });

    idt.load();
    unsafe {
        interrupts::PICS.lock().initialize();
        interrupts::PICS.lock().write_masks(0b11111100, 0x0);
    }
    x86_64::instructions::interrupts::enable();

    let heap_value = Box::new('c');
    // stack_overflow();

    let mut executor = Executor::new();
    executor.spawn(Task::new(example_task()));
    executor.spawn(Task::new(keyboard::print_keypresses()));

    println! {"value: {}", heap_value};
    // run_program_0();
    executor.run();
}

// halt cpu until next interrupt arrives
pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("panic here: {}", info);
    crate::hlt_loop();
}

#[allow(stack_overflow)]
fn stack_overflow() -> ! {
    stack_overflow();
}

async fn async_number() -> u32 {
    67
}

async fn example_task() {
    let number = async_number().await;
}
