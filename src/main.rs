#![feature(abi_x86_interrupt)]
#![no_std]
#![no_main]

mod vga_buffer;
mod interrupts;
mod gdt;
mod memory;
mod allocator;


use core::panic::PanicInfo;


#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {

    gdt::init();
    interrupts::init();

    println!("OC Initialized{}", "!");

    x86_64::instructions::interrupts::int3();

    loop{}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! { loop {} }
