#![feature(abi_x86_interrupt)] // allow test to continue after interrupt

use core::sync::atomic::Ordering;

use crate::{
    cpu::get_local,
    gdt::IstStackIndexes,
    hlt_loop,
    interrupt_vector::InterruptVector,
    nmi_handler_states::{NmiHandlerState, NMI_HANDLER_STATES},
    println, serial_println,
};
use conquer_once::spin::OnceCell;
use pic8259::ChainedPics;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode}; // runtime statics

// finish in apic, rather than legacy PIC
unsafe fn end_of_interrupt() {
    get_local()
        .local_apic
        .get()
        .unwrap()
        .lock()
        .end_of_interrupt();
}

fn handle_panic_originating_on_other_cpu() -> ! {
    hlt_loop()
}

extern "x86-interrupt" fn nmi_handler(_stack_frame: InterruptStackFrame) {
    handle_panic_originating_on_other_cpu();
}

// @param stack_frame pointers to exception handlers
extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("exception: breakpoint\n {:#?}", stack_frame);
    unsafe { end_of_interrupt() };
}

// examples include: uninitalized interrupts, incorrect mapping to interrupts like 0x21 works for
// keyboard, but 0x31 does not
extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) -> ! {
    panic!(
        "EXCEPTION: DOUBLE FAULT\n{:#?} | error code: {}",
        stack_frame, error_code
    );
}

extern "x86-interrupt" fn general_protection_fault(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    serial_println! {"Stack Frame: {:#?}", stack_frame};
    serial_println! {"Error code: {}", error_code};
    panic! {"general protection fault"};
}

// handle page faults
pub extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;
    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed Address: {:?}", Cr2::read());
    println!("Error Code: {:?}", error_code);
    println!("{:#?}", stack_frame);
    hlt_loop();
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use x86_64::instructions::port::Port;

    let is_bsp = unsafe { get_local().local_apic.get().unwrap().lock().is_bsp() };
    let scancode: u8 = unsafe { Port::new(0x60).read() };
    crate::task::keyboard::add_scancode(scancode);
    unsafe { end_of_interrupt() };
}

extern "x86-interrupt" fn timer_interrupt_handler(_sf: InterruptStackFrame) {
    // We must notify the local APIC that it's the end of interrupt, otherwise we won't receive any more interrupts from it
    // Safety: We are done with an interrupt triggered by the local APIC
    unsafe { end_of_interrupt() };
}

extern "x86-interrupt" fn unexpected_irq_handler(sf: InterruptStackFrame) {
    println!("Unhandled IRQ! {:#?}", sf);
    unsafe { end_of_interrupt() };
}

pub fn init() {
    let local = get_local();
    // allow time for GDT to be initialized with its segments, else general protection fault inside
    // double fault occur
    let idt = local.idt.call_once(|| {
        let mut idt = InterruptDescriptorTable::new();

        for i in 33..48 {
            idt[i].set_handler_fn(unexpected_irq_handler);
        }
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(u8::from(IstStackIndexes::Exception).into());
            idt.page_fault
                .set_handler_fn(page_fault_handler)
                .set_stack_index(u8::from(IstStackIndexes::Exception).into());
            idt.general_protection_fault
                .set_handler_fn(general_protection_fault)
                .set_stack_index(u8::from(IstStackIndexes::Exception).into());
        }
        idt.non_maskable_interrupt.set_handler_fn(nmi_handler);
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        // should be 0x31 since we already mapped it in our ioapic and is ready for that module
        idt[0x31].set_handler_fn(keyboard_interrupt_handler);
        idt[InterruptVector::LocalApicTimer as u8].set_handler_fn(timer_interrupt_handler);

        idt
    });

    idt.load();

    // serial_println!("is_bsp: {}", is_bsp);
    // Now that we loaded the IDT, we are ready to receive NMIs to handle interrupts for each CPU
    // Let's update our state to indicate that we are ready to receive NMIs
    if NMI_HANDLER_STATES.get().unwrap()[local.kernel_assigned_id as usize]
        .compare_exchange(
            NmiHandlerState::NmiHandlerNotSet,
            NmiHandlerState::NmiHandlerSet,
            Ordering::Relaxed,
            Ordering::Relaxed,
        )
        .is_err()
    {
        // `compare_exchange` will "fail" if the value is currently not what we expected it to be.
        // In this case, the kernel already panicked and updated our state to `KernelPanicked` before we tried to indicate that we are ready to receive NMIs.
        handle_panic_originating_on_other_cpu()
    };

    // unsafe {
    //     PICS.lock().initialize();
    //     PICS.lock().write_masks(0b11111100, 0x0);
    // }
    x86_64::instructions::interrupts::enable();
}
