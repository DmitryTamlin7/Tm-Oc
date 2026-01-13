use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};
use lazy_static::lazy_static;
use pic8259::ChainedPics;
use spin;
use crate::{gdt, println, task::keyboard::add_scancode};
use core::sync::atomic::{AtomicU64, Ordering};

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

pub static TICKS: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard = PIC_1_OFFSET + 1,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 { self as u8 }
}

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt[InterruptIndex::Timer.as_u8()].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_u8()].set_handler_fn(keyboard_interrupt_handler);
        idt
    };
}

// Инициализация PIT (Programmable Interval Timer)
fn init_pit() {
    let divisor: u16 = 1193; // Примерно 1000 Гц (прерывание каждую 1 мс)
    unsafe {
        use x86_64::instructions::port::Port;
        let mut cmd_port = Port::new(0x43);
        let mut data_port = Port::new(0x40);
        cmd_port.write(0x36u8); // Канал 0, lobyte/hibyte, режим 3
        data_port.write((divisor & 0xFF) as u8);
        data_port.write((divisor >> 8) as u8);
    }
}

pub fn init() {
    IDT.load();
    init_pit();
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    TICKS.fetch_add(1, Ordering::Relaxed);

    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use x86_64::instructions::port::Port;
    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };
    add_scancode(scancode);
    unsafe { PICS.lock().notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8()); }
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) { println!("{:#?}", stack_frame); }
extern "x86-interrupt" fn double_fault_handler(f: InterruptStackFrame, _: u64) -> ! { panic!("{:#?}", f); }
extern "x86-interrupt" fn page_fault_handler(_: InterruptStackFrame, _: PageFaultErrorCode) { loop {} }