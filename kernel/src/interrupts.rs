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

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
}

impl InterruptIndex {
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    pub fn as_usize(self) -> usize {
        self as u8 as usize
    }
}

fn handle_panic_originating_on_other_cpu() -> ! {
    hlt_loop()
}

extern "x86-interrupt" fn nmi_handler(_stack_frame: InterruptStackFrame) {
    handle_panic_originating_on_other_cpu()
}

// @param stack_frame pointers to exception handlers
extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("exception: breakpoint\n {:#?}", stack_frame);
}

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

    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };
    crate::task::keyboard::add_scancode(scancode);

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}

extern "x86-interrupt" fn timer_interrupt_handler(_sf: InterruptStackFrame) {
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

extern "x86-interrupt" fn unexpected_irq_handler(sf: InterruptStackFrame) {
    println!("Unhandled IRQ! {:#?}", sf);
    unsafe {
        PICS.lock().notify_end_of_interrupt(0);
    } // still ack it
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
        idt[InterruptIndex::Keyboard as u8].set_handler_fn(keyboard_interrupt_handler);
        idt[InterruptVector::LocalApicTimer as u8].set_handler_fn(timer_interrupt_handler);

        idt
    });

    idt.load();
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

    unsafe {
        PICS.lock().initialize();
        PICS.lock().write_masks(0b11111100, 0x0);
    }
    x86_64::instructions::interrupts::enable();
}
