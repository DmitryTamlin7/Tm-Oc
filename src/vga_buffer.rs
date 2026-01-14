use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);
impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            _ => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }
                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;
                let color_code = self.color_code;
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });
                self.column_position += 1;
            }
        }
    }

    pub fn backspace(&mut self) {
        if self.column_position > 0 {
            self.column_position -= 1;
            let row = BUFFER_HEIGHT - 1;
            let col = self.column_position;
            let color_code = self.color_code;
            self.buffer.chars[row][col].write(ScreenChar {
                ascii_character: b' ',
                color_code,
            });
        }
    }

    pub fn clear_screen(&mut self) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for r in 0..BUFFER_HEIGHT {
            for c in 0..BUFFER_WIDTH {
                self.buffer.chars[r][c].write(blank);
            }
        }
        self.column_position = 0;
    }

    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                _ => self.write_byte(0xfe),
            }
        }
    }

    pub fn write_byte_at(&mut self, col: usize, row: usize, byte: u8) {
        let color_code = self.color_code;
        self.buffer.chars[row][col].write(ScreenChar {
            ascii_character: byte,
            color_code,
        });
    }


    pub fn set_color(&mut self, foreground: Color, background: Color) {
        self.color_code = ColorCode::new(foreground, background);
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::LightBlue, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}

pub fn clear_screen() {
    WRITER.lock().clear_screen();
}

pub fn backspace() {
    WRITER.lock().backspace();
}

pub fn set_color(foreground: Color, background: Color) {
    WRITER.lock().set_color(foreground, background);
}

pub fn print_timer(seconds: u64) {
    let mut writer = WRITER.lock();

    let old_color = writer.color_code;
    writer.set_color(Color::Black, Color::Cyan);

    let hours = seconds / 3600;
    let mins = (seconds % 3600) / 60;
    let secs = seconds % 60;

    let col_start = 65;
    let row = 0;


    let prefix = "TIME: ";
    for (i, &byte) in prefix.as_bytes().iter().enumerate() {
        writer.write_byte_at(col_start + i, row, byte);
    }
    writer.write_byte_at(col_start + 6, row, (hours / 10) as u8 + b'0');
    writer.write_byte_at(col_start + 7, row, (hours % 10) as u8 + b'0');
    writer.write_byte_at(col_start + 8, row, b':');
    writer.write_byte_at(col_start + 9, row, (mins / 10) as u8 + b'0');
    writer.write_byte_at(col_start + 10, row, (mins % 10) as u8 + b'0');
    writer.write_byte_at(col_start + 11, row, b':');
    writer.write_byte_at(col_start + 12, row, (secs / 10) as u8 + b'0');
    writer.write_byte_at(col_start + 13, row, (secs % 10) as u8 + b'0');

    writer.color_code = old_color;
}