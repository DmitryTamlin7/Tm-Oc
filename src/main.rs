#![no_std]
#![no_main]

mod vga_buffer;

use core::panic::PanicInfo;


#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {

    println!("Hello World from Tm_OC {}", "!");
    loop{}

}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! { loop {} }
