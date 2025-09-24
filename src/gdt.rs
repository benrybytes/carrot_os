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
    use x86_64::instructions::segmentation::{Segment, CS, DS, ES, SS};
    use x86_64::instructions::tables::load_tss;
    GDT_CELL.get_or_init(|| {
        let mut gdt = GlobalDescriptorTable::new();
        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        let data_selector = gdt.add_entry(Descriptor::kernel_data_segment());
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));

        (
            gdt,
            Selectors {
                code_selector,
                data_selector,
                tss_selector,
            },
        )
    });
    let GDT = GDT_CELL.get().unwrap();

    // reset cs to point to tss segment
    GDT.0.load();
    unsafe {
        CS::set_reg(GDT.1.code_selector);
        SS::set_reg(GDT.1.data_selector);
        load_tss(GDT.1.tss_selector);
    }
}
