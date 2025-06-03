use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use lazy_static::lazy_static; // runtime statics
use crate::println;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt
    };
}

pub fn init_idt() {
    IDT.load(); // load is handled by the compiler
}

// @param stack_frame pointers to exception handlers
extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("exception: breakpoint\n {:#?}", stack_frame);
}

#[test_case]
fn test_breakpoints_exception() {
    x86_64::instructions::interrupts::int3();
}
