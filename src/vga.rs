use spin::Mutex;
use core::ptr::{read_volatile, write_volatile};

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

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;
const VGA_BUFFER: usize = 0xb8000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct VgaChar(u16);

impl VgaChar {
    fn new(ascii: u8, fg: Color, bg: Color) -> Self {
        VgaChar(
            (u16::from(ascii))
                | (u16::from(fg as u8 & 0xf) << 8)
                | (u16::from(bg as u8 & 0xf) << 12)
        )
    }
}

pub struct Writer {
    column_position: usize,
    color_code: u8,
    buffer: *mut u16,
}

impl Writer {
    pub fn new(fg: Color, bg: Color) -> Self {
        Writer {
            column_position: 0,
            color_code: (fg as u8 & 0xf) | ((bg as u8 & 0xf) << 4),
            buffer: VGA_BUFFER as *mut u16,
        }
    }

    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;
                let offset = (row * BUFFER_WIDTH + col) as usize;

                unsafe {
                    write_volatile(self.buffer.add(offset), VgaChar::new(byte, Color::LightCyan, Color::Black).0);
                }
                self.column_position += 1;
            }
        }
    }

    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let offset_from = (row * BUFFER_WIDTH + col) as usize;
                let offset_to = ((row - 1) * BUFFER_WIDTH + col) as usize;

                unsafe {
                    let char_from = read_volatile(self.buffer.add(offset_from));
                    write_volatile(self.buffer.add(offset_to), char_from);
                }
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    fn clear_row(&mut self, row: usize) {
        let blank = VgaChar::new(b' ', Color::LightCyan, Color::Black).0;
        for col in 0..BUFFER_WIDTH {
            let offset = (row * BUFFER_WIDTH + col) as usize;
            unsafe {
                write_volatile(self.buffer.add(offset), blank);
            }
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

    pub fn clear_screen(&mut self) {
        let blank = VgaChar::new(b' ', Color::LightCyan, Color::Black).0;
        for i in 0..(BUFFER_HEIGHT * BUFFER_WIDTH) {
            unsafe {
                write_volatile(self.buffer.add(i), blank);
            }
        }
        self.column_position = 0;
    }
}

impl core::fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

// Fix: pub static mut WRITER
pub static mut WRITER: Option<Mutex<Writer>> = None;

// Fix: init_vga fn
pub fn init_vga(fg: Color, bg: Color) {
    unsafe {
        WRITER = Some(Mutex::new(Writer::new(fg, bg)));
    }
}

// Fix: pub get_writer
pub fn get_writer() -> &'static Mutex<Writer> {
    unsafe { WRITER.as_ref().unwrap() }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    use core::fmt::Write;
    get_writer().lock().write_fmt(args).unwrap();
}
