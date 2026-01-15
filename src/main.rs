#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]

extern crate alloc;

use core::panic::PanicInfo;
use bootloader::{BootInfo, entry_point};
use x86_64::VirtAddr;
use crate::task::{Task, simple_executor::SimpleExecutor};

mod vga_buffer;
mod interrupts;
mod gdt;
mod memory;
mod allocator;
mod task;
mod fs;

use crate::memory::{BootInfoFrameAllocator, init_offset_page_table};

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    gdt::init();
    interrupts::init();
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init_offset_page_table(phys_mem_offset) };
    let mut frame_allocator = unsafe { memory::BootInfoFrameAllocator::init(&boot_info.memory_map) };
    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("Heap failed");
    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();
    vga_buffer::clear_screen();
    vga_buffer::draw_header();

    println!("\n\n"); 
    println!(" [BOOT]: GDT, IDT, PICS ........................ [ OK ]");
    println!(" [BOOT]: Memory Mapping & Heap (1MB) ........... [ OK ]");
    println!(" [BOOT]: RamFS (ReadOnly Filesystem) .......... [ OK ]");
    println!(" [BOOT]: Async Executor & Keyboard ............. [ OK ]");
    println!("\nWelcome to Tm_Os. Type 'help' to see available commands.");
    println!("---------------------------------------------------------");
    let mut executor = SimpleExecutor::new();
    executor.spawn(Task::new(task::keyboard::shell_task()));
    executor.run();

    loop { x86_64::instructions::hlt(); }
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