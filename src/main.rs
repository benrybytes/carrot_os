#![no_std]
#![no_main]

extern crate alloc; // import again to not 
use alloc::boxed::Box;

use limine::BaseRevision;
use limine::framebuffer::Framebuffer;
use limine::request::{RequestsEndMarker, RequestsStartMarker, MemoryMapRequest, ExecutableAddressRequest, HhdmRequest};

use carrot_os::{println, print};
use core::panic::PanicInfo;
use carrot_os::task::{Task, executor::Executor, keyboard};

#[used]
// The .requests section allows limine to find the requests faster and more safely.
#[unsafe(link_section = ".requests")]
static BASE_REVISION: BaseRevision = BaseRevision::new();


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

extern "C" {
    static _binary_Cyr_a8x16_psf_start: u8;
    static _binary_Cyr_a8x16_psf_end: u8;
    static _binary_Cyr_a8x16_psf_size: u8;
}

#[cfg(not(test))]
#[unsafe(no_mangle)]
unsafe extern "C" fn kmain() -> ! {

    use carrot_os::{memory, allocator, text};
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


    if let Some(executable_address_response) = EXECUTABLE_ADDRESS_REQUEST.get_response() {
        let virtual_base = executable_address_response.virtual_base();
        let physical_base = executable_address_response.physical_base();

        println!{"physical start 0x{:x}", physical_base};
        println!{"virtual start 0x{:x}", virtual_base};
        // starting point / Cr3 pointer to our memory address
        if let Some(memory_map_response) = MEMORY_MAP_REQUEST.get_response() {
            if let Some(hddm_response) = HHDM_REQUEST.get_response() {
                let offset = hddm_response.offset();
                let mut mapper = unsafe { memory::init(VirtAddr::new(offset))};
                let mut frame_allocator = unsafe {
                    memory::BootInfoFrameAllocator::init(memory_map_response.entries())
                };

                let _ = allocator::init_heap(&mut mapper, &mut frame_allocator).unwrap();

                // unsafe {
                //     *(0xdeadbeef as *mut u8) = 42;
                // };
                let heap_value = Box::new('c');
                println!("heap_value location: {:p}", heap_value);
                println!("heap_value: {}", *heap_value);

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
    println!("panic here: {}", info);
    carrot_os::hlt_loop();
}

async fn async_number() -> u32 {
    67
}

async fn example_task() {
    let number = async_number().await;
    println!("async number :3 {}", number);
}
