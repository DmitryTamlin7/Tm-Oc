#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]

extern crate alloc;

use core::panic::PanicInfo;
use bootloader::{BootInfo, entry_point};
use x86_64::VirtAddr;
use alloc::{boxed::Box, vec, vec::Vec, rc::Rc};

mod vga_buffer;
mod interrupts;
mod gdt;
mod memory; 
mod allocator;

use crate::memory::{BootInfoFrameAllocator, init_offset_page_table};

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    println!("Welcome to Tm_Os Heap Test");

    gdt::init();
    interrupts::init();
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { init_offset_page_table(phys_mem_offset) };
    let mut frame_allocator = unsafe {
        BootInfoFrameAllocator::init(&boot_info.memory_map)
    };
    
    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("Heap initialization failed");
    
    let x = Box::new(42);
    println!("Heap works! Value: {}", x);

    let mut vec = Vec::new();
    vec.push(1);
    vec.push(2);
    println!("Vector works! {:?}", vec);

    println!("It did not crash!");

    loop {
        x86_64::instructions::hlt();
    }
}

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {
        x86_64::instructions::hlt();
    }
}