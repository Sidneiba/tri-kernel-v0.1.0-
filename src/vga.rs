use core::fmt;
use spin::Mutex;
use lazy_static::lazy_static;
use volatile::Volatile;

// --- VGA Colors ---
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

// --- VGA Character ---
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct VgaChar {
    ascii: u8,
    color: u8,
}

// --- VGA Writer ---
pub struct Writer {
    row: usize,
    column: usize,
    color: u8,
    buffer: &'static mut [Volatile<VgaChar>; 25 * 80],
}

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        row: 0,
        column: 0,
        color: make_color(Color::White, Color::Black),
        buffer: unsafe {
            &mut *(0xb8000 as *mut [Volatile<VgaChar>; 25 * 80])
        },
    });
}

fn make_color(fg: Color, bg: Color) -> u8 {
    fg as u8 | (bg as u8) << 4
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            8 | b'\x7F' => {
                if self.column > 0 {
                    self.column -= 1;
                    self.buffer[self.row * 80 + self.column] = Volatile::new(VgaChar {
                        ascii: b' ',
                        color: self.color,
                    });
                }
            }
            byte => {
                if self.column >= 80 {
                    self.new_line();
                }
                let index = self.row * 80 + self.column;
                self.buffer[index] = Volatile::new(VgaChar {
                    ascii: byte,
                    color: self.color,
                });
                self.column += 1;
            }
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            self.write_byte(byte);
        }
    }

    fn new_line(&mut self) {
        if self.row < 24 {
            self.row += 1;
        } else {
            for row in 0..24 {
                for col in 0..80 {
                    let index = row * 80 + col;
                    let next_index = (row + 1) * 80 + col;
                    self.buffer[index] = Volatile::new(self.buffer[next_index].read());
                }
            }
            for col in 0..80 {
                self.buffer[24 * 80 + col] = Volatile::new(VgaChar {
                    ascii: b' ',
                    color: self.color,
                });
            }
        }
        self.column = 0;
    }

    pub fn clear_screen(&mut self) {
        for row in 0..25 {
            for col in 0..80 {
                self.buffer[row * 80 + col] = Volatile::new(VgaChar {
                    ascii: b' ',
                    color: self.color,
                });
            }
        }
        self.row = 0;
        self.column = 0;
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

// Implementa shell::Writer para vga::Writer
impl crate::shell::Writer for Writer {
    fn write_byte(&mut self, byte: u8) {
        self.write_byte(byte);
    }

    fn write_string(&mut self, s: &str) {
        self.write_string(s);
    }
}

// --- VGA Init ---
pub fn init_vga(fg: Color, bg: Color) {
    let mut writer = WRITER.lock();
    writer.color = make_color(fg, bg);
    writer.clear_screen();
}

pub fn get_writer() -> &'static Mutex<Writer> {
    &WRITER
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::vga::get_writer().lock().write_fmt(format_args!($($arg)*)).unwrap();
    };
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}
