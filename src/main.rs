#![feature(abi_x86_interrupt)]
#![no_std]
#![no_main]

mod vga_buffer;
mod interrupts;
mod gdt;
mod memory;
mod allocator;


use core::panic::PanicInfo;
use bootloader::bootinfo::BootInfo;
use crate::memory::BootInfoFrameAllocator;


#[no_mangle]
pub extern "C" fn _start(boot_info: &'static BootInfo) -> ! {
    gdt::init();
    interrupts::init();

    println!("OS Initialized!");

    let mut frame_allocator = BootInfoFrameAllocator::init(&boot_info.memory_map);

    for i in 0..5 {
        if let Some(frame) = frame_allocator.allocate_frame() {
            println!("Allocated frame {} at physical address {:#X}", i, frame.start_address().as_u64());
        } else {
            println!("No more frames available!");
        }
    }

    loop {}
}
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! { loop {} }
