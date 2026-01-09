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

    println!("Tm OC Initialized{}", "!");

    // обход безопасности и намеренное ломание ядра
    unsafe {
        *(0xdeadbeef as *mut u64) = 42;
    }

    loop{}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! { loop {} }
