// credit: https://github.com/ChocolateLoverRaj/rust-os-tutorial/tree/code/kernel/src/memory

use create_page_tables::*;
use limine::response::MemoryMapResponse;
use spin::Once;
use x86_64::{
    registers::control::{Cr3, Cr3Flags},
    structures::paging::{PhysFrame, Size4KiB},
};

pub mod create_page_tables;
pub mod global_allocator;
pub mod physical_memory;
pub mod virtual_memory;

pub use physical_memory::*;
pub use virtual_memory::*;

#[non_exhaustive]
#[derive(Debug)]
pub struct Memory {
    #[allow(unused)]
    pub physical_memory: spin::Mutex<PhysicalMemory>,
    #[allow(unused)]
    pub virtual_memory: spin::Mutex<VirtualMemory>,
    pub new_kernel_cr3: PhysFrame<Size4KiB>,
    pub new_kernel_cr3_flags: Cr3Flags,
}

pub static MEMORY: Once<Memory> = Once::new();

/// Initializes global allocator, creates new page tables, and switches to new page tables.
/// This function must be called before mapping pages or running our kernel's code on APs.
///
/// # Safety
/// This function must be called exactly once, and no page tables should be modified before calling this function.
pub unsafe fn init_bsp(memory_map: &'static MemoryMapResponse) {
    // get global allocator to get us frames that are not used by limine or other processes
    let global_allocator_start = unsafe { global_allocator::init(memory_map) };
    let mut physical_memory = PhysicalMemory::new(memory_map, global_allocator_start);

    // map the physical address to virtual memory using cr3 register and frames we found
    let (new_kernel_cr3, new_kernel_cr3_flags, virtual_memory) =
        create_page_tables(memory_map, &mut physical_memory);
    // Safety: page tables are ready to be used
    unsafe { Cr3::write(new_kernel_cr3, new_kernel_cr3_flags) };
    MEMORY.call_once(|| Memory {
        physical_memory: spin::Mutex::new(physical_memory),
        virtual_memory: spin::Mutex::new(virtual_memory),
        new_kernel_cr3,
        new_kernel_cr3_flags,
    });
}

/// # Safety
/// Must be called on all APs before modifying page tables
pub unsafe fn init_ap() {
    let memory = MEMORY.get().unwrap();
    // Safety: page tables are ready to be used
    unsafe { Cr3::write(memory.new_kernel_cr3, memory.new_kernel_cr3_flags) };
}
