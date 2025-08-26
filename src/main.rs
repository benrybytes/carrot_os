#![no_std]
#![no_main]

extern crate alloc; // import again to not 
use alloc::boxed::Box;

use limine::BaseRevision;
use limine::request::{FramebufferRequest, RequestsEndMarker, RequestsStartMarker, MemoryMapRequest, ExecutableAddressRequest, HhdmRequest};

use carrot_os::serial_println;
use core::panic::PanicInfo;
use carrot_os::task::{Task, executor::Executor, keyboard};

#[used]
// The .requests section allows limine to find the requests faster and more safely.
#[unsafe(link_section = ".requests")]
static BASE_REVISION: BaseRevision = BaseRevision::new();

#[used]
#[unsafe(link_section = ".requests")]
static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

#[used]
#[unsafe(link_section = ".requests")]
static MEMORY_MAP_REQUEST: MemoryMapRequest = MemoryMapRequest::new();

#[used]
#[unsafe(link_section = ".requests")]
static EXECUTABLE_ADDRESS_REQUEST: ExecutableAddressRequest = ExecutableAddressRequest::new();

#[used]
#[unsafe(link_section = ".requests")]
// Request the higher-half direct mapping
static HHDM_REQUEST: HhdmRequest = HhdmRequest::new();


// Define the stand and end markers for Limine requests.
#[used]
#[unsafe(link_section = ".requests_start_marker")]
static _START_MARKER: RequestsStartMarker = RequestsStartMarker::new();
#[used]
#[unsafe(link_section = ".requests_end_marker")]
static _END_MARKER: RequestsEndMarker = RequestsEndMarker::new();

#[cfg(not(test))]
#[unsafe(no_mangle)]
unsafe extern "C" fn kmain() -> ! {

    use carrot_os::{memory, allocator};
    use x86_64::VirtAddr;
    // All limine requests must also be referenced in a called function, otherwise they may be
    // removed by the linker.
    assert!(BASE_REVISION.is_supported());
    carrot_os::init();

    // ram_storage!(tiny);
    // let mut ram = Ram::default();
    // let mut storage = RamStorage::new(&mut ram);
    //
    // // must format before first mount
    // Filesystem::format(&mut storage).unwrap();
    // // must allocate state statically before use
    // let mut allocated_filesystem = Filesystem::allocate();
    // let mut fs = Filesystem::mount(&mut allocated_filesystem, &mut storage).unwrap();
    //
    // // may use common `OpenOptions`
    // let mut buf = [0u8; 11];
    // fs.open_file_with_options_and_then(
    //     |options| options.read(true).write(true).create(true),
    //     path!("example.txt"),
    //     |file| {
    //         // file.write(b"Why is black smoke coming out?!")?;
    //         // file.seek(SeekFrom::End(-24)).unwrap();
    //         file.read(&mut buf)
    //         // assert_eq!(file.read(&mut buf)?, 11);
    //         Ok(())
    //     }
    // ).unwrap();
    // assert_eq!(&buf, b"black smoke");

    if let Some(framebuffer_response) = FRAMEBUFFER_REQUEST.get_response() {
        if let Some(framebuffer) = framebuffer_response.framebuffers().next() {
            for i in 0..500_u64 {
                // Calculate the pixel offset using the framebuffer information we obtained above.
                // We skip `i` scanlines (pitch is provided in bytes) and add `i * 4` to skip `i` pixels forward.
                let pixel_offset = i * framebuffer.pitch() + i * 4;

                // Write 0xFFFFFFFF to the provided pixel offset to fill it white.
                unsafe {
                    framebuffer
                        .addr()
                        .add(pixel_offset as usize)
                        .cast::<u32>()
                        .write(0x2596beFF)
                };
            }
        }

    }
    if let Some(executable_address_response) = EXECUTABLE_ADDRESS_REQUEST.get_response() {
        let virtual_base = executable_address_response.virtual_base();
        let physical_base = executable_address_response.physical_base();
        let offset = virtual_base + physical_base;

        serial_println!{"offset and virtual start 0x{:x}", offset};
        serial_println!{"physical start 0x{:x}", physical_base};
        serial_println!{"virtual start 0x{:x}", virtual_base};
        // starting point / Cr3 pointer to our memory address
        if let Some(memory_map_response) = MEMORY_MAP_REQUEST.get_response() {
            if let Some(hddm_response) = HHDM_REQUEST.get_response() {
                let mut mapper = unsafe { memory::init(VirtAddr::new(hddm_response.offset()))};
                let mut frame_allocator = unsafe {
                    memory::BootInfoFrameAllocator::init(memory_map_response.entries(), hddm_response.offset() as usize)
                };
                serial_println!{"about to go to mapper"};

                let _ = allocator::init_heap(&mut mapper, &mut frame_allocator).unwrap();
                serial_println!{"allocated in mapper"};

                let heap_value = Box::new('c');
                serial_println!("heap_value at {:p}", heap_value);
                let nigger_value = Box::new(444);
                serial_println!("heap_value at {:p}", nigger_value);

                let mut executor = Executor::new();
                executor.spawn(Task::new(example_task()));
                executor.spawn(Task::new(keyboard::print_keypresses()));
                executor.run();

            }
            
        }
    }



    // serial_print!("stack_overflow::stack_overflow...\t");

    // println!("hello world");


    // stack_overflow();

    // #[cfg(test)]
    // test_main();


    loop {}
}
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("panic here: {}", info);
    carrot_os::hlt_loop();
}

async fn async_number() -> u32 {
    67
}

async fn example_task() {
    let number = async_number().await;
    serial_println!("async number :3 {}", number);
}
