#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

mod interrupts;
mod keyboard;
mod tri_compress;
mod virtual_fs;
mod vga;
mod shell;

use core::panic::PanicInfo;
use core::fmt::Write; // Adicionado para write_fmt
use bootloader::{BootInfo, entry_point};
use x86_64::instructions;

// --- Panic Handler ---
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        instructions::hlt();
    }
}

// --- SerialPort (para debug) ---
struct SerialPort;
impl core::fmt::Write for SerialPort {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for byte in s.bytes() {
            unsafe {
                let port = 0x3f8 as *mut u8;
                let status = 0x3fd as *const u8;
                while (*status & 0x20) == 0 {}
                *port = byte;
            }
        }
        Ok(())
    }
}

static mut SERIAL: SerialPort = SerialPort;

fn print_serial(args: core::fmt::Arguments) {
    unsafe { SERIAL.write_fmt(args).unwrap(); }
}

pub fn serial_print(s: &str) {
    print_serial(format_args!("{}", s));
}

pub fn serial_println(s: &str) {
    serial_print(s);
    print_serial(format_args!("\n"));
}

#[macro_export]
macro_rules! serial_println {
    ($($arg:tt)*) => {
        $crate::print_serial(format_args!($($arg)*));
        $crate::print_serial(format_args!("\n"));
    };
}

pub fn serial_putc(byte: u8) {
    unsafe {
        let port = 0x3f8 as *mut u8;
        let status = 0x3fd as *const u8;
        while (*status & 0x20) == 0 {}
        *port = byte;
    }
}

// --- Serial Init ---
fn serial_init() {
    unsafe {
        let port = 0x3f8 as *mut u8;
        *port = 0x03;
        *(port.offset(1)) = 0x00;
        *(port.offset(3)) = 0x03;
        *(port.offset(2)) = 0xc7;
        *(port.offset(4)) = 0x0b;
    }
}

// --- Entry Point ---
entry_point!(_start);

fn _start(_boot_info: &'static BootInfo) -> ! {
    serial_init();

    // Inicializar VGA
    vga::init_vga(vga::Color::LightCyan, vga::Color::Black);

    // Teste TRI
    let original: [u8; 32] = *b"TRI-Kernel Boot!!!\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
    let compressed = tri_compress::compress(&original);
    let decompressed = tri_compress::decompress(&compressed);
    let orig_len = 32u32;
    let comp_len = compressed.iter().position(|&x| x == 0).unwrap_or(0) as u32;
    let ratio = if comp_len > 0 { (orig_len * 100 / comp_len) as u8 } else { 100 };
    let decomp_ok = if decompressed.iter().zip(original.iter()).all(|(&a, &b)| a == b) { "OK" } else { "FAIL" };

    serial_println!("Boot OK! TRI Ratio: {}%. Decomp: {}", ratio, decomp_ok);
    println!("Boot OK! TRI Ratio: {}%. Decomp: {}", ratio, decomp_ok);  // VGA

    // Inicializar Interrupções
    serial_println!("Inicializando IDT e IRQs...");
    println!("Inicializando IDT e IRQs...");  // VGA
    interrupts::init_idt();
    interrupts::init_pics();
    instructions::interrupts::enable();
    serial_println!("Interrupções habilitadas (Timer + Teclado)");
    println!("Interrupções habilitadas (Timer + Teclado)");  // VGA

    // Inicializar Teclado
    keyboard::init();
    serial_println!("Keyboard init OK");
    println!("Keyboard init OK");  // VGA

    // Inicializar FS Virtual
    serial_println!("Virtual FS montado: /bin e /etc");
    println!("Virtual FS montado: /bin e /etc");  // VGA
    if let Some(config) = virtual_fs::read_file("/etc/tri-shellrc") {
        let config_str = core::str::from_utf8(config).unwrap_or("Erro UTF8");
        serial_println!("Config carregada: {}", config_str);
        println!("Config carregada: {}", config_str);  // VGA
    } else {
        serial_println!("Erro: tri-shellrc não encontrado!");
        println!("Erro: tri-shellrc não encontrado!");  // VGA
    }

    // Executar Shell
    serial_println!("Init: Executando /bin/shell (novo shell_loop)...");
    println!("Init: Executando /bin/shell (novo shell_loop)...");  // VGA
    {
        let mut writer_guard = vga::get_writer().lock();
        crate::shell::shell_loop(&mut *writer_guard);
    }

    // Caso o shell retorne (não deveria), pausar CPU
    loop {
        instructions::hlt();
    }
}
