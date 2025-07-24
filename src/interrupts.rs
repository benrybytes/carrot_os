use crate::{gdt, println, hlt_loop};
use lazy_static::lazy_static; // runtime statics
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);

        // ensure stack not being used elsewhere, which is unsafe
        // unsafe {
        //     idt.double_fault
        //         .set_handler_fn(double_fault_handler)
        //         .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        // }
        idt.page_fault.set_handler_fn(page_fault_handler);

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

// handle page faults
extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) {
    panic!("exception: double fault\n {:#?}", stack_frame);
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode
) {
    use x86_64::registers::control::Cr2;
    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed Address: {:?}", Cr2::read());
    println!("Error Code: {:?}", error_code);
    println!("{:#?}", stack_frame);
    hlt_loop();
}

#[test_case]
fn test_breakpoints_exception() {
    x86_64::instructions::interrupts::int3();
}
