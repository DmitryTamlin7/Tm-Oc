use crate::{print, println, vga_buffer};
use alloc::string::String;
use conquer_once::spin::OnceCell;
use crossbeam_queue::ArrayQueue;
use core::{pin::Pin, task::{Poll, Context}};
use futures_util::stream::{Stream, StreamExt};
use futures_util::task::AtomicWaker;
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1, KeyCode};
use crate::task::sleep;

static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();
static WAKER: AtomicWaker = AtomicWaker::new();

pub(crate) fn add_scancode(scancode: u8) {
    if let Ok(queue) = SCANCODE_QUEUE.try_get() {
        if let Ok(_) = queue.push(scancode) {
            WAKER.wake();
        }
    }
}

pub struct ScancodeStream { _private: () }

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
                            execute_command(&buffer).await; // ТЕПЕРЬ AWAIT
                            buffer.clear();
                            print!("> ");
                        }
                        '\u{0008}' => {
                            if buffer.pop().is_some() { vga_buffer::backspace(); }
                        }
                        c => {
                            buffer.push(c);
                            print!("{}", c);
                        }
                    },
                    DecodedKey::RawKey(KeyCode::Return) => {
                        println!();
                        execute_command(&buffer).await; // ТЕПЕРЬ AWAIT
                        buffer.clear();
                        print!("> ");
                    }
                    _ => {}
                }
            }
        }
    }
}

async fn execute_command(input: &str) {
    let input = input.trim();
    if input.is_empty() { return; }

    let mut parts = input.splitn(2, ' ');
    let command = parts.next().unwrap();
    let args = parts.next().unwrap_or("");

    match command {
        "help" => println!("Commands: ls, cat <file>, help, clear, uptime, sum <n>, sleep <ms>"),
        "clear" => vga_buffer::clear_screen(),
        "ls" => crate::fs::list_files(), // ВЫЗОВ ФС
        "cat" => {
            if let Some(file) = crate::fs::get_file(args.trim()) {
                println!("{}", file.content);
            } else {
                println!("File not found: {}", args);
            }
        }
        "uptime" => println!("Ticks: {}", crate::interrupts::TICKS.load(core::sync::atomic::Ordering::Relaxed)),
        "sum" => {
            if let Ok(n) = args.parse::<u64>() {
                let mut total: u64 = 0;
                for i in 1..=n { total += i; }
                println!("Sum: {}", total);
            }
        },
        "sleep" => {
            if let Ok(ms) = args.parse::<u64>() {
                sleep(ms).await;
            }
        },
        _ => println!("Unknown command: {}", command),
    }
}