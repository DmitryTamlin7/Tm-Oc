#![feature(abi_x86_interrupt)]
#![no_std]
#![no_main]

mod vga_buffer;
mod interrupts;
mod gdt;
mod memory;
mod allocator;
mod paging;

use core::panic::PanicInfo;
use bootloader::bootinfo::BootInfo;
use x86_64::VirtAddr;
use crate::memory::BootInfoFrameAllocator;
use crate::paging::{identity_map_range, init_offset_page_table};

#[no_mangle]
pub extern "C" fn _start(boot_info: &'static BootInfo) -> ! {
    gdt::init();
    interrupts::init();

    println!("OS Initialized!");

    let mut frame_allocator = BootInfoFrameAllocator::init(&boot_info.memory_map);

    // OffsetPageTable
    let phys_offset = VirtAddr::new(0);
    let mut mapper = unsafe { init_offset_page_table(phys_offset) };

    // Identity mapping (например, со 1 MiB, чтобы избежать конфликтов)
    identity_map_range(0x10_0000, 0x20_0000, &mut mapper, &mut frame_allocator)
        .expect("Identity mapping failed");

    println!("Paging initialized");

    for i in 0..5 {
        if let Some(frame) = frame_allocator.allocate_frame() {
            println!("Allocated frame {} at physical address {:#X}", i, frame.start_address().as_u64());
        } else {
            println!("No more frames available!");
        }
    }

    // Тест page fault
    test_page_fault();

    loop {}
}

fn test_page_fault() {
    println!("Testing page fault...");
    unsafe {
        let ptr = 0xdead_be00 as *mut u64;
        *ptr = 42;
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("PANIC: {}", info);
    loop {}
}
