use crate::{print, println, vga_buffer};
use alloc::string::String;
use conquer_once::spin::OnceCell;
use crossbeam_queue::ArrayQueue;
use core::{pin::Pin, task::{Poll, Context}};
use futures_util::stream::{Stream, StreamExt};
use futures_util::task::AtomicWaker;
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1, KeyCode};

static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();
static WAKER: AtomicWaker = AtomicWaker::new();

pub(crate) fn add_scancode(scancode: u8) {
    if let Ok(queue) = SCANCODE_QUEUE.try_get() {
        if let Err(_) = queue.push(scancode) {
            // Очередь полна
        } else {
            WAKER.wake();
        }
    }
}

pub struct ScancodeStream {
    _private: (),
}

impl ScancodeStream {
    pub fn new() -> Self {
        SCANCODE_QUEUE.try_init_once(|| ArrayQueue::new(100)).expect("Init once");
        ScancodeStream { _private: () }
    }
}

impl Stream for ScancodeStream {
    type Item = u8;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<u8>> {
        let queue = SCANCODE_QUEUE.try_get().expect("not initialized");
        if let Some(scancode) = queue.pop() {
            return Poll::Ready(Some(scancode));
        }
        WAKER.register(&cx.waker());
        match queue.pop() {
            Some(scancode) => {
                WAKER.take();
                Poll::Ready(Some(scancode))
            }
            None => Poll::Pending,
        }
    }
}

pub async fn shell_task() {
    let mut scancodes = ScancodeStream::new();
    let mut keyboard = Keyboard::new(ScancodeSet1::new(), layouts::Us104Key, HandleControl::Ignore);
    let mut buffer = String::with_capacity(256);

    print!("> ");

    while let Some(scancode) = scancodes.next().await {
        if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
            if let Some(key) = keyboard.process_keyevent(key_event) {
                match key {
                    DecodedKey::Unicode(character) => match character {
                        '\n' => {
                            println!();
                            execute_command(&buffer);
                            buffer.clear();
                            print!("> ");
                        }
                        '\u{0008}' => { // Backspace
                            if buffer.pop().is_some() {
                                vga_buffer::backspace();
                            }
                        }
                        c => {
                            buffer.push(c);
                            print!("{}", c);
                        }
                    },
                    DecodedKey::RawKey(KeyCode::Return) => {
                        println!();
                        execute_command(&buffer);
                        buffer.clear();
                        print!("> ");
                    }
                    _ => {}
                }
            }
        }
    }
}

fn execute_command(input: &str) {
    let input = input.trim();
    if input.is_empty() { return; }

    let mut parts = input.splitn(2, ' ');
    let command = parts.next().unwrap();
    let args = parts.next().unwrap_or("");

    match command {
        "help" => println!("Commands: help, clear, uptime, sum <n>, echo <msg>"),
        "clear" => vga_buffer::clear_screen(),
        "uptime" => println!("Tm_Os is running in Async Mode!"),
        "echo" => println!("{}", args),
        "sum" => {
            if let Ok(n) = args.parse::<u64>() {
                let mut total: u64 = 0;
                for i in 1..=n { total += i; }
                println!("Sum (1..{}): {}", n, total);
            } else { println!("Usage: sum <number>"); }
        },
        _ => println!("Unknown command: {}", command),
    }
}