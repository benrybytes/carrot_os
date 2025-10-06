#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(abi_x86_interrupt)]
#![feature(never_type)]

use conquer_once::spin::OnceCell;
use core::panic::PanicInfo;
use lazy_static::lazy_static;

extern crate alloc;

pub mod filesystem;
pub mod gdt;
pub mod hhdm_offset;
pub mod interrupts;
pub mod limine_requests;
pub mod memory;
pub mod serial;
pub mod task;
pub mod text;
pub mod translate_addr;
pub mod x86_64_consts;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

// writing to ports
pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

pub trait Testable {
    fn run(&self) -> ();
}

// use trait to run method with nice print messages
impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}

pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    hlt_loop();
}

pub fn init() {
    use interrupts::InterruptIndex;
    use x86_64::structures::idt::InterruptDescriptorTable; // runtime statics
    gdt::init();

    // allow time for GDT to be initialized with its segments, else general protection fault inside
    // double fault occur
    let IDT = interrupts::IDT_CELL.get_or_init(|| {
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

    IDT.load();
    unsafe {
        interrupts::PICS.lock().initialize();
        interrupts::PICS.lock().write_masks(0b11111100, 0x0);
    }
    x86_64::instructions::interrupts::enable();
}

// halt cpu until next interrupt arrives
pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

/// Entry point for `cargo test`
#[cfg(test)]
unsafe extern "C" fn kmain() -> ! {
    // like before
    init();
    test_main();
    hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}
