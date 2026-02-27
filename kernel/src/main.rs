#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(let_chains)]

use core::{arch::asm, panic::PanicInfo, sync::atomic::Ordering};

extern crate alloc; // import again to use allocation

use acpi::*;
use apic::*;
use call_with_rsp::*;
use cpu::*;
use filesystem::*;
use gdt::*;
use hhdm_offset::*;
use interrupt_vector::*;
use interrupts::*;
use limine_requests::*;
use local_ioapic::*;
use memory::*;
use nmi_handler_states::*;
use serial::*;
use spcr::*;
use stack::*;
use syscall_handler::*;
use task::*;
use text::*;
use translate_addr::*;
use usermode::*;
use x86_64_consts::*;

mod acpi;
mod apic;
mod call_with_rsp;
mod cpu;
mod filesystem;
mod gdt;
mod hhdm_offset;
mod interrupt_vector;
mod interrupts;
mod limine_requests;
mod local_ioapic;
mod memory;
mod nmi_handler_states;
mod serial;
mod spcr;
mod stack;
mod syscall_handler;
mod task;
mod text;
mod translate_addr;
mod usermode;
mod x86_64_consts;

#[unsafe(no_mangle)]
unsafe extern "C" fn kmain() -> ! {
    assert!(BASE_REVISION.is_supported());

    // starting point / Cr3 pointer to our memory address
    let memory_map_response = MEMORY_MAP_REQUEST.get_response().unwrap();

    unsafe {
        memory::init_bsp(memory_map_response);
        cpu::init_bsp();
    }
    crate::println!("hello from BSP :3");
    let local = get_local();
    Stack::new(
        StackId {
            _type: StackType::Normal,
            cpu_id: local.kernel_assigned_id,
        },
        KERNEL_NORMAL_STACK_SIZE,
        MemoryType::UsedByKernel(KernelMemoryUsageType::Stack),
    )
    .switch(init_bsp);
}

unsafe fn disable_pic() {
    use x86_64::instructions::port::Port;

    let mut pic1 = Port::<u8>::new(0x21);
    let mut pic2 = Port::<u8>::new(0xA1);

    pic1.write(0xFF);
    pic2.write(0xFF);
}

extern "C" fn init_bsp() -> ! {
    nmi_handler_states::init(); // initialize CPU to handle interrupts before we can init
    use crate::executor::Executor;
    use alloc::boxed::Box;
    use x86_64::structures::idt::InterruptDescriptorTable; // runtime statics

    // multiple cpu request in bsp to be used throughout all cpus
    // get ACPI to be available
    let rsdp = RSDP_REQUEST.get_response().unwrap();
    let acpi_tables = acpi::parse(rsdp);
    local_ioapic::init_ioapic(&acpi_tables);
    spcr::init(&acpi_tables);
    apic::init_bsp(&acpi_tables);
    apic::init_local_apic();

    let mp_response = MP_REQUEST.get_response().unwrap();

    gdt::init();
    interrupts::init();

    // set entry_point for each cpu
    for cpu in mp_response.cpus() {
        cpu.goto_address.write(entry_point_ap);
    }
    // Safety: Disabling PIC
    unsafe {
        disable_pic();
    }

    let mut executor = Executor::new();
    executor.spawn(Task::new(keyboard::print_keypresses()));
    // enter_user_mode();
    // let ptr = 0xFFFF_8000_0000_0000u64 as *const u64;
    // let _val = unsafe { *ptr };
    // println!("in user ring 3");
    executor.run();
    loop {}
}

// initialize cpu core
unsafe extern "C" fn entry_point_ap(cpu: &limine::mp::Cpu) -> ! {
    // Safety: we are calling this right away
    unsafe { memory::init_ap() };
    // Safety: We're actually calling the function on this CPU
    unsafe { cpu::init_ap(cpu) };

    crate::println!("hello from AP :3");
    // log::info!("Hello from AP");

    Stack::new(
        StackId {
            _type: StackType::Normal,
            cpu_id: get_local().kernel_assigned_id,
        },
        KERNEL_NORMAL_STACK_SIZE,
        MemoryType::UsedByKernel(KernelMemoryUsageType::Stack),
    )
    .switch(init_ap)
}

extern "C" fn init_ap() -> ! {
    // if get_local().kernel_assigned_id == 2 {
    //     for _ in 0..20000000 {}
    // }
    gdt::init();
    // interrupts::init();
    apic::init_local_apic();

    hlt_loop()
}

// halt cpu until next interrupt arrives
pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // Since the OS panicked, we need to tell the other CPUs to stop immediately
    // However, if we send an NMI to a CPU that didn't load its IDT yet, the system will triple fault
    if let Some(local) = try_get_local()
        && let Some(mut local_apic) = local
            .local_apic
            .get()
            .and_then(|local_apic| local_apic.try_lock())
        && let Some(nmi_handler_states) = NMI_HANDLER_STATES.get()
    {
        for (cpu_id, nmi_handler_state) in nmi_handler_states
            .iter()
            .enumerate()
            // Make sure to not send an NMI to our own CPU
            .filter(|(cpu_id, _)| *cpu_id as u32 != local.kernel_assigned_id)
        {
            if let NmiHandlerState::NmiHandlerSet =
                nmi_handler_state.swap(NmiHandlerState::KernelPanicked, Ordering::Release)
            {
                // Safety: since the kernel is panicking, we need to tell the other CPUs to hlt
                unsafe { local_apic.send_nmi(local_apic_id_of(cpu_id as u32)) };
            }
        }
    }

    println!("panic here: {}", info);
    crate::hlt_loop();
}
