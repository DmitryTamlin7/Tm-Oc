#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]

extern crate alloc;

use core::panic::PanicInfo;
use bootloader::{BootInfo, entry_point};
use x86_64::VirtAddr;
use alloc::{boxed::Box, vec, vec::Vec};

mod vga_buffer;
mod interrupts;
mod gdt;
mod memory;
mod allocator;

use crate::memory::{BootInfoFrameAllocator, init_offset_page_table};

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    println!("Welcome to Tm_Os Stage 7: Interactive Mode");

    // 1. Инициализация базовых систем
    gdt::init();
    interrupts::init(); // Загружает IDT

    // 2. Инициализация памяти и кучи
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { init_offset_page_table(phys_mem_offset) };
    let mut frame_allocator = unsafe {
        BootInfoFrameAllocator::init(&boot_info.memory_map)
    };

    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("Heap initialization failed");

    // 3. Инициализация ПРЕРЫВАНИЙ ЖЕЛЕЗА
    unsafe { interrupts::PICS.lock().initialize() }; // Ремап и включение PIC
    x86_64::instructions::interrupts::enable();      // Команда процессору "слушать прерывания"

    println!("[OK] System initialized and interrupts enabled!");
    println!("Try typing on your keyboard...");

    // Тест кучи (убедимся, что прерывания не мешают памяти)
    let _x = Box::new(42);
    let mut vec = Vec::new();
    vec.push(1);

    // Бесконечный цикл ожидания прерываний
    loop {
        // hlt останавливает процессор до следующего прерывания, экономя ресурсы
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