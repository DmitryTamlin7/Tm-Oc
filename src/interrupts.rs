use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};
use x86_64::registers::control::Cr2;
use lazy_static::lazy_static;
use pic8259::ChainedPics;
use spin;
use crate::{gdt, println, print, vga_buffer};
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1, KeyCode};
use alloc::string::String;

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;
pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

pub struct Shell {
    buffer: String,
}

impl Shell {
    pub fn new() -> Self {
        Shell { buffer: String::with_capacity(256) }
    }

    pub fn handle_char(&mut self, c: char) {
        match c {
            '\n' => {
                println!();
                self.execute();
                self.buffer.clear();
                print!("> ");
            }
            '\u{0008}' => {
                if self.buffer.pop().is_some() {
                    vga_buffer::backspace();
                }
            }
            _ => {
                if self.buffer.len() < 255 {
                    self.buffer.push(c);
                    print!("{}", c);
                }
            }
        }
    }

    fn execute(&mut self) {
        let input = self.buffer.trim();
        if input.is_empty() { return; }

        let mut parts = input.splitn(2, ' ');
        let command = parts.next().unwrap_or("");
        let args = parts.next().unwrap_or("");

        match command {
            "help" => {
                println!("Available commands:");
                println!("- help: Show this list");
                println!("- echo <text>: Print text to screen");
                println!("- sum <n>: Calculate sum from 1 to n");
                println!("- uptime: System status");
                println!("- clear: Clear the screen");
                println!("Try 'color' to change theme!");
            },
            "echo" => {
                println!("{}", args);
            },
            "sum" => {
                if let Ok(n) = args.parse::<u64>() {
                    let mut total: u64 = 0;
                    for i in 1..=n {
                        total += i;
                    }
                    println!("Sum from 1 to {} is: {}", n, total);
                } else {
                    println!("Usage: sum <number>");
                }
            },
            "uptime" => {
                println!("Tm_Os is running. CPU is in HLT state when idle.");
            },
            "clear" => {
                vga_buffer::clear_screen();
            },
            "color" => {
                vga_buffer::set_color(vga_buffer::Color::Pink, vga_buffer::Color::Black);
                vga_buffer::clear_screen();
                println!("Theme updated!");
            }
            _ => println!("Unknown command: {}", command),
        }
    }
}

lazy_static! {
    static ref KEYBOARD: spin::Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> =
        spin::Mutex::new(Keyboard::new(ScancodeSet1::new(), layouts::Us104Key, HandleControl::Ignore));

    static ref SHELL: spin::Mutex<Shell> = spin::Mutex::new(Shell::new());

    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt[InterruptIndex::Timer.as_u8()].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_u8()].set_handler_fn(keyboard_interrupt_handler);
        idt
    };
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard = PIC_1_OFFSET + 1,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 { self as u8 }
}

pub fn init() {
    IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(stack_frame: InterruptStackFrame, _code: u64) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn page_fault_handler(stack_frame: InterruptStackFrame, _code: PageFaultErrorCode) {
    let addr = Cr2::read().unwrap_or(x86_64::VirtAddr::new(0));
    println!("EXCEPTION: PAGE FAULT at {:?}", addr);
    loop {}
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    unsafe { PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer.as_u8()); }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use x86_64::instructions::port::Port;
    let mut keyboard = KEYBOARD.lock();
    let mut shell = SHELL.lock();
    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };

    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(character) => shell.handle_char(character),
                DecodedKey::RawKey(KeyCode::Return) => shell.handle_char('\n'),
                DecodedKey::RawKey(KeyCode::Backspace) => shell.handle_char('\u{0008}'),
                _ => {}
            }
        }
    }
    unsafe { PICS.lock().notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8()); }
}