use crate::{gdt::IstStackIndexes, hlt_loop, println, serial_println};
use conquer_once::spin::OnceCell;
use pic8259::ChainedPics;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode}; // runtime statics

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });
pub static IDT_CELL: OnceCell<InterruptDescriptorTable> = OnceCell::uninit();

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
    // allow time for GDT to be initialized with its segments, else general protection fault inside
    // double fault occur
    let idt = IDT_CELL.get_or_init(|| {
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
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt[InterruptIndex::Keyboard as u8].set_handler_fn(keyboard_interrupt_handler);
        idt[InterruptIndex::Timer.as_u8()].set_handler_fn(timer_interrupt_handler);

        idt
    });

    idt.load();
    unsafe {
        PICS.lock().initialize();
        PICS.lock().write_masks(0b11111100, 0x0);
    }
    x86_64::instructions::interrupts::enable();
}
