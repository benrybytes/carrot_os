use core::arch::asm;

use x86_64::instructions::segmentation::{Segment, CS, DS, ES, FS, GS};
use x86_64::registers::rflags::RFlags;

use crate::cpu::get_local;
use crate::memory::MemoryType;
use crate::stack::{Stack, StackId, StackType, USER_NORMAL_STACK_SIZE};
use crate::{println, serial_println};

#[no_mangle]
extern "C" fn test_user_function() -> ! {
    loop {
        unsafe {
            asm!("hlt");
        }
    }
}

pub fn enter_user_mode() {
    let local = get_local();
    let gdt = local.gdt.get().unwrap();

    let user_stack = Stack::new(
        StackId {
            _type: StackType::Normal,
            cpu_id: local.kernel_assigned_id,
        },
        USER_NORMAL_STACK_SIZE,
        MemoryType::UsedByUserMode,
    );

    unsafe {
        asm!(
            "cli",
            "push {user_ds}",         // SS (Stack Segment)
            "push {user_sp}",         // RSP
            "pushfq",                 // RFLAGS to only resume program
            "pop rax",
            "or rax, 0x200",          // Enable Interrupts in RFLAGS
            "push rax",
            "push {user_cs}",         // CS (Code Segment)
            // "push {user_ip}",         // RIP
            "iretq",
            user_ds = in(reg) gdt.user_data_selector.0,
            user_sp = in(reg) user_stack.top().as_u64(),
            user_cs = in(reg) gdt.user_code_selector.0,
            // user_ip = in(reg) test_user_function as u64,
            options(noreturn)
        )
    };
}
