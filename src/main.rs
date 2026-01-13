#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]

extern crate alloc;

mod vga_buffer;
mod interrupts;
mod gdt;
mod memory;
mod allocator;
mod task;

use bootloader::{BootInfo, entry_point};
use x86_64::VirtAddr;
use crate::task::{Task, simple_executor::SimpleExecutor};

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    gdt::init();
    interrupts::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init_offset_page_table(phys_mem_offset) };
    let mut frame_allocator = unsafe { memory::BootInfoFrameAllocator::init(&boot_info.memory_map) };

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("Heap fail");

    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();

    vga_buffer::clear_screen();
    println!("Tm_Os Async Edition");

    let mut executor = SimpleExecutor::new();
    executor.spawn(Task::new(task::keyboard::shell_task()));
    executor.run();

    loop { x86_64::instructions::hlt(); }
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("{}", info);
    loop { x86_64::instructions::hlt(); }
}

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("Alloc error: {:?}", layout)
}