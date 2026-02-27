use core::num::NonZero;

use acpi::{
    sdt::madt::{self, Madt, MadtEntry},
    AcpiTables,
};
use ez_paging::{ConfigurableFlags, Frame, PageSize};
use uart_16550::MmioSerialPort;
use x2apic::ioapic::{self, IrqFlags, RedirectionTableEntry};
use x86_64::{registers::model_specific::PatMemoryType, PhysAddr, VirtAddr};

use crate::{
    memory::{KernelMemoryUsageType, MemoryType, MEMORY},
    serial_println,
};

pub fn init_ioapic(acpi_tables: &AcpiTables<impl acpi::Handler>) {
    // Parse MADT
    let madt_table = acpi_tables.find_table::<Madt>().unwrap();
    let mut ioapic_phys = None;
    let mut ioapic_gsi_base = None;

    for entry in madt_table.get().entries() {
        match entry {
            // multiple process controller for any global interrupts
            MadtEntry::IoApic(ioapic) => {
                let _id = ioapic.io_apic_id;
                let _addr = ioapic.io_apic_address;
                let _gsi_base = ioapic.global_system_interrupt_base;

                serial_println!(
                    "IOAPIC: id={} addr={:#x} gsi_base={}",
                    _id,
                    _addr,
                    _gsi_base
                );

                ioapic_gsi_base = Some(_gsi_base);
                ioapic_phys = Some(_addr);
            }
            // if we want to override an interupt at a specific CPU core
            MadtEntry::InterruptSourceOverride(iso) => {
                let _bus = iso.bus;
                let _flags = iso.flags;
                let _irq = iso.irq;
                let _gsi = iso.global_system_interrupt;
                serial_println!(
                    "ISO: bus={} irq={} gsi={} flags={:?}",
                    _bus,
                    _irq,
                    _gsi,
                    _flags
                );
            }
            // CPU information
            MadtEntry::LocalApic(local) => {
                serial_println!(
                    "LAPIC: cpu={} apic_id={}",
                    local.processor_id,
                    local.apic_id
                );
            }
            _ => {}
        }
    }

    // setup rerouting of interrupts to our ioapic
    if let Some(ioapic_phys) = ioapic_phys
        && let Some(ioapic_gsi_base) = ioapic_gsi_base
    {
        // Map IOAPIC MMIO (always)
        // Save somewhere global
        let page_size = PageSize::_4KiB;

        let frame = Frame::new(PhysAddr::new(ioapic_phys.into()), page_size).unwrap();

        let memory = MEMORY.get().unwrap();
        let mut physical_memory = memory.physical_memory.lock();
        let mut frame_allocator = physical_memory.get_kernel_frame_allocator();
        let mut virtual_memory = memory.virtual_memory.lock();

        let page = virtual_memory
            .allocate_contiguous_pages(
                page_size,
                NonZero::new(1).unwrap(),
                MemoryType::UsedByKernel(KernelMemoryUsageType::Stack),
            )
            .unwrap();

        let flags = ConfigurableFlags {
            writable: true,
            executable: false,
            pat_memory_type: PatMemoryType::StrongUncacheable,
        };

        unsafe {
            virtual_memory
                .l4_mut()
                .map_page(page, frame, flags, &mut frame_allocator)
        }
        .unwrap();
        let mut ioapic_init = unsafe { ioapic::IoApic::new(page.start_addr().as_u64()) };
        serial_println!("ioapic init");

        // Safety:
        unsafe {
            // redirect interrupts to ioapic to get keyboard to be setup by ioapic gsi (cpu
            // interrupts)
            let mut default_entry = RedirectionTableEntry::default();
            serial_println!("enter flags");
            let mut flags = IrqFlags::empty();
            flags.set(IrqFlags::LEVEL_TRIGGERED, false);
            flags.set(IrqFlags::MASKED, false);
            flags.set(IrqFlags::LOGICAL_DEST, false);

            default_entry.set_mode(ioapic::IrqMode::Fixed);
            default_entry.set_flags(flags);
            default_entry.set_dest(0);

            // range allowed 0x10 - 0xFE
            default_entry.set_vector(0xFE); // tell CPU this is the source for the keyboard interrupt

            ioapic_init.set_table_entry(1, default_entry);
            serial_println!("init ioapic");
        };
    }
}
