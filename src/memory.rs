use x86_64::{
    structures::paging::{FrameAllocator, OffsetPageTable, PageTable, PhysFrame, Size4KiB},
    PhysAddr, VirtAddr,
};
// use bootloader::bootinfo::{MemoryMap, MemoryRegionType}; // boot info
use limine::memory_map::{Entry, EntryType};

pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    unsafe {
        // virtual
        let level_4_table = active_level_4_table(physical_memory_offset);

        // get physical address from level 4
        OffsetPageTable::new(level_4_table, physical_memory_offset)
    }
}

// unsafe method as we guarantee physical memory maps to virtual table 4
unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    unsafe { &mut *page_table_ptr }
}

pub unsafe fn translate_addr(addr: VirtAddr, physical_memory_offset: VirtAddr) -> Option<PhysAddr> {
    full_translate_helper(addr, physical_memory_offset)
}

// traverse
pub fn full_translate_helper(addr: VirtAddr, physical_memory_offset: VirtAddr) -> Option<PhysAddr> {
    use x86_64::registers::control::Cr3;
    use x86_64::structures::paging::page_table::FrameError;

    let (level_4_page_frame, _) = Cr3::read();

    // get each table level that help traverse to level 1
    let table_indexes = [
        addr.p4_index(),
        addr.p3_index(),
        addr.p2_index(),
        addr.p1_index(),
    ];
    let mut frame = level_4_page_frame;

    for &index in &table_indexes {
        // convert frame to table to traverse
        let virt = physical_memory_offset + frame.start_address().as_u64();
        let table_ptr: *const PageTable = virt.as_ptr();
        let table = unsafe { &*table_ptr };

        // get entry from the current table to move to the next frame until reach
        // table 1
        let entry = &table[index];
        frame = match entry.frame() {
            Ok(frame) => frame,
            Err(FrameError::FrameNotPresent) => return None,
            Err(FrameError::HugeFrame) => panic!("huge pages not supported"),
        }
    }

    Some(frame.start_address() + u64::from(addr.page_offset()))
}

extern "C" {
    static _kernel_end: u8;
    static _kernel_start: u8;
}

pub struct EmptyFrameAllocator;

// allocate pages
unsafe impl FrameAllocator<Size4KiB> for EmptyFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        None
    }
}

// usable frames by bootloader's memory map
pub struct BootInfoFrameAllocator<'a> {
    memory_map: &'a [&'a Entry],
    next: usize,
}

impl<'a> BootInfoFrameAllocator<'a> {
    pub unsafe fn init(memory_map: &'a [&'a Entry]) -> Self {
        BootInfoFrameAllocator {
            memory_map,
            next: 0,
        }
    }

    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> + '_ {
        self.memory_map
            .iter()
            .filter(move |entry| entry.entry_type == EntryType::USABLE)
            .flat_map(|r| (r.base..(r.base + r.length)).step_by(4096))
            .map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl<'a> FrameAllocator<Size4KiB> for BootInfoFrameAllocator<'a> {
    // fetch a new frame on physical, so we could use it
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}
