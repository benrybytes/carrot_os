use conquer_once::spin::OnceCell;
use lazy_static::lazy_static;
use x86_64::instructions::segmentation::Segment;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;

pub struct Selectors {
    pub code_selector: SegmentSelector, // operating in kernel
    pub data_selector: SegmentSelector, // access to stack, variables, etc in ring0
    pub tss_selector: SegmentSelector,  // ring0, ring2, ring3 context switching
    pub user_code_selector: SegmentSelector,
    pub user_data_selector: SegmentSelector,
}

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

extern "C" {
    static _stack_end_low: u8;
}

lazy_static! {
    pub static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            let stack_top = unsafe { &_stack_end_low as *const u8 as u64 };
            VirtAddr::new(stack_top)
        };
        tss
    };
}

lazy_static! {
    static ref GDT_CELL: OnceCell<(GlobalDescriptorTable, Selectors)> = OnceCell::uninit();
}

// allowing switching of ring0 and ring3, while loading TSS
pub fn init() {
    use x86_64::instructions::segmentation::{Segment, CS, SS};
    use x86_64::instructions::tables::load_tss;
    let GDT = GDT_CELL.get_or_init(|| {
        let mut gdt = GlobalDescriptorTable::new();
        let code_selector = gdt.append(Descriptor::kernel_code_segment());
        let data_selector = gdt.append(Descriptor::kernel_data_segment());
        let tss_selector = gdt.append(Descriptor::tss_segment(&TSS));

        let user_code_selector = gdt.append(Descriptor::user_code_segment());
        let user_data_selector = gdt.append(Descriptor::user_data_segment());
        (
            gdt,
            Selectors {
                code_selector,
                data_selector,
                tss_selector,
                user_code_selector,
                user_data_selector,
            },
        )
    });
    // switch to kernel mode
    GDT.0.load();
    unsafe {
        CS::set_reg(GDT.1.code_selector);
        SS::set_reg(GDT.1.data_selector);
        load_tss(GDT.1.tss_selector);
    }
}

#[no_mangle]
pub extern "C" fn jump_usermode() {
    use core::arch::asm;
    use x86_64::instructions::segmentation::SS;
    use x86_64::registers::model_specific::{Efer, EferFlags};

    let gdt = GDT_CELL.get().unwrap();
    unsafe {
        Efer::update(|flags| {
            *flags = flags.union(EferFlags::SYSTEM_CALL_EXTENSIONS);
        });
        // Set the stack segment before executing IRET
        SS::set_reg(gdt.1.tss_selector);

        asm!(
            "mov ds, {0:x}",  // Set DS to user data segment
            "mov es, {0:x}",  // Set ES to user data segment
            "mov fs, {0:x}",  // Set FS to user data segment
            "mov gs, {0:x}",  // Set GS to user data segment
            "mov rax, rsp",   // Load current stack pointer into RAX
            "push {0:x}",     // Push data segment selector
            "push rax",       // Push current stack pointer
            "pushf",          // Push flags
            "push {1:x}",     // Push code segment selector
            "iret",           // Switch to user mode
            in(reg) gdt.1.user_data_selector.0,
            in(reg) gdt.1.user_code_selector.0,
        );
    }
}
