use crate::memory::{KernelMemoryUsageType, MemoryType};
use crate::{cpu::get_local, Stack, StackId, StackType, EXCEPTION_HANDLER_STACK_SIZE};
use conquer_once::spin::OnceCell;
use lazy_static::lazy_static;
use num_enum::IntoPrimitive;
use x86_64::instructions::segmentation::Segment;
use x86_64::registers::model_specific::Star;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;

#[derive(Debug, IntoPrimitive)]
#[repr(u8)]
pub enum IstStackIndexes {
    Exception,
}

pub struct Gdt {
    pub gdt: GlobalDescriptorTable,
    pub kernel_code_selector: SegmentSelector, // operating in kernel
    pub kernel_data_selector: SegmentSelector, // access to stack, variables, etc in ring0
    pub tss_selector: SegmentSelector,         // ring0, ring2, ring3 context switching
    pub user_code_selector: SegmentSelector,
    pub user_data_selector: SegmentSelector,
}

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

// allowing switching of ring0 and ring3, while loading TSS
pub fn init() {
    use x86_64::instructions::segmentation::{Segment, CS, SS};
    use x86_64::instructions::tables::load_tss;
    let local = get_local();
    let tss = local.tss.call_once(|| {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[u8::from(IstStackIndexes::Exception) as usize] = Stack::new(
            StackId {
                _type: StackType::ExceptionHandler,
                cpu_id: local.kernel_assigned_id,
            },
            EXCEPTION_HANDLER_STACK_SIZE,
            MemoryType::UsedByKernel(KernelMemoryUsageType::Stack),
        )
        .top();

        tss
    });
    let gdt = local.gdt.call_once(|| {
        let mut gdt = GlobalDescriptorTable::new();

        // GDT is strict on selector ordering, so TSS must go last as it is at 0x28. 0x0 - 0x20 is
        // reserved for kernel and user selectors
        let kernel_code_selector = gdt.append(Descriptor::kernel_code_segment());
        let kernel_data_selector = gdt.append(Descriptor::kernel_data_segment());

        let user_data_selector = gdt.append(Descriptor::user_data_segment());
        let user_code_selector = gdt.append(Descriptor::user_code_segment());
        let tss_selector = gdt.append(Descriptor::tss_segment(tss));
        Gdt {
            gdt,
            kernel_code_selector,
            kernel_data_selector,
            tss_selector,
            user_code_selector,
            user_data_selector,
        }
    });
    // switch to kernel mode
    gdt.gdt.load();
    unsafe {
        CS::set_reg(gdt.kernel_code_selector);
        SS::set_reg(gdt.kernel_data_selector);
        load_tss(gdt.tss_selector);
    }

    // write to ring 3 and 0 for syscall context switching of rings
    Star::write(
        gdt.user_code_selector,
        gdt.user_data_selector,
        gdt.kernel_code_selector,
        gdt.kernel_data_selector,
    )
    .unwrap();
}
