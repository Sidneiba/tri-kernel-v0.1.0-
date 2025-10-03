#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

mod interrupts;
mod keyboard;
mod tri_compress;
mod virtual_fs;
mod vga;  // VGA text mode
mod tri_motor;  // Motor cognitivo TRI-LIA

use x86_64::instructions;
use core::panic::PanicInfo;
use core::fmt::Write;
use bootloader::{BootInfo, entry_point};

// --- Panic Handler ---
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! { 
    loop {} 
}

// --- SerialPort (pra debug) ---
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

// --- Read Line (eco VGA + serial) ---
fn read_line(buffer: &mut [u8; 128]) -> usize {
    let mut idx = 0;
    while idx < 127 {
        if let Some(c) = keyboard::get_key() {
            // Eco VGA
            print!("{}", c as char);
            // Eco serial
            serial_putc(c as u8);
            if c == b'\n' || c == b'\r' { break; }
            buffer[idx] = c;
            idx += 1;
        } else {
            instructions::hlt();
        }
    }
    buffer[idx] = 0;
    idx
}

// --- Parse Command ---
fn parse_command(cmd: &[u8; 128]) -> (&[u8], &[u8]) {
    let mut space_pos = 0;
    while space_pos < 128 && cmd[space_pos] != b' ' && cmd[space_pos] != 0 { 
        space_pos += 1; 
    }
    let cmd_end = space_pos;
    let mut arg_start = space_pos;
    while arg_start < 128 && cmd[arg_start] == b' ' { 
        arg_start += 1; 
    }
    let mut arg_end = arg_start;
    while arg_end < 128 && cmd[arg_end] != 0 { 
        arg_end += 1; 
    }
    (&cmd[..cmd_end], &cmd[arg_start..arg_end])
}

// --- Handle Commands (VGA + serial, com tri-exec fixo) ---
fn handle_command(cmd: &[u8], arg: &[u8]) {
    let cmd_str = core::str::from_utf8(cmd).unwrap_or("");
    let arg_str = core::str::from_utf8(arg).unwrap_or("");

    println!("");  // VGA
    serial_println!("");  // Serial
    
    if cmd_str == "help" {
        println!("TRI Shell Commands:");  // VGA
        println!("  help             - Mostra esta ajuda");
        println!("  tri-compress <str> - Comprime string (NATIVO TRI)");
        println!("  tri-compress-file <file> - Comprime arquivo do FS (NATIVO TRI)");
        println!("  tri-exec <str>   - Ciclo cognitivo LIA (Motor TRI)");
        println!("  tri-ratio        - Mostra stats TRI");
        println!("  cat <file>       - Lê arquivo");
        println!("  ls               - Lista arquivos");
        println!("  halt             - Para o kernel");
        
        serial_println!("TRI Shell Commands: [VGA mostra detalhes]");  // Serial log
    } else if cmd_str == "tri-compress" && !arg_str.is_empty() {
        println!("[TRI NATIVE] Comprimindo string: '{}'", arg_str);  // VGA
        serial_println!("[TRI NATIVE] Comprimindo string: '{}'", arg_str);  // Serial
        
        let mut original_bytes = [0u8; 32];
        let len = arg_str.len().min(32);
        original_bytes[..len].copy_from_slice(&arg_str.as_bytes()[..len]);
        let compressed = tri_compress::compress(&original_bytes);
        let (orig_len, comp_len, ratio) = tri_compress::stats(&original_bytes, &compressed);
        
        println!("Original: {} bytes", orig_len);  // VGA
        println!("Comprimido: {} bytes", comp_len);
        println!("Ratio TRI: {}%", ratio);
        println!("Dados comprimidos (hex):");
        
        // Hex dump na VGA
        let comp_pos = compressed.iter().position(|&x| x == 0).unwrap_or(64);
        for (i, byte) in compressed.iter().take(comp_pos).enumerate() {
            if i % 16 == 0 { print!("\n  "); }
            print!("{:02X} ", byte);
        }
        println!("");
        
        // Testa descompressão
        let decompressed = tri_compress::decompress(&compressed);
        let decomp_ok = if decompressed[..len] == original_bytes[..len] { "SIM" } else { "NÃO" };
        println!("Descomprimido matcha? {}", decomp_ok);  // VGA
        
        serial_println!("[TRI NATIVE] Ratio: {}% ({} -> {})", ratio, orig_len, comp_len);  // Serial summary
        
    } else if cmd_str == "tri-compress-file" && !arg_str.is_empty() {
        println!("[TRI NATIVE FILE] Comprimindo arquivo: '{}'", arg_str);  // VGA
        serial_println!("[TRI NATIVE FILE] Comprimindo arquivo: '{}'", arg_str);  // Serial
        
        // Lê arquivo do FS virtual
        if let Some(content) = virtual_fs::read_file(arg_str) {
            let mut original_bytes = [0u8; 32];
            let len = content.len().min(32);
            original_bytes[..len].copy_from_slice(&content[..len]);
            let compressed = tri_compress::compress(&original_bytes);
            let (orig_len, comp_len, ratio) = tri_compress::stats(&original_bytes, &compressed);
            
            println!("Arquivo original: {} bytes", orig_len);  // VGA
            println!("Arquivo comprimido: {} bytes", comp_len);
            println!("Ratio TRI: {}%", ratio);
            println!("Dados comprimidos (hex preview - primeiros 64 bytes):");
            
            // Hex dump preview
            let comp_pos = compressed.iter().position(|&x| x == 0).unwrap_or(64);
            let preview_len = comp_pos.min(64);
            for (i, byte) in compressed.iter().take(preview_len).enumerate() {
                if i % 16 == 0 { print!("\n  "); }
                print!("{:02X} ", byte);
            }
            if comp_pos > 64 {
                println!(" ... ({} bytes total)", comp_pos);
            } else {
                println!("");
            }
            
            // Testa descompressão
            let decompressed = tri_compress::decompress(&compressed);
            let decomp_ok = if decompressed[..len] == original_bytes[..len] { "SIM!" } else { "NÃO (erro!)" };
            println!("Verificação: Descomprimido matcha original? {}", decomp_ok);
            
            // Futuro: Salva comprimido no FS
            println!("[FUTURO] Salvando como /tmp/{}.tri", arg_str.trim_end_matches('/'));
            serial_println!("[TRI NATIVE FILE] Sucesso! Ratio: {}% ({} -> {})", ratio, orig_len, comp_len);
            
        } else {
            println!("Arquivo '{}' não encontrado no FS!", arg_str);  // VGA
            serial_println!("Arquivo '{}' não encontrado!", arg_str);  // Serial
        }
        
    } else if cmd_str == "tri-exec" && !arg_str.is_empty() {
        println!("[TRI MOTOR LIA] Executando ciclo cognitivo em: '{}'", arg_str);  // VGA
        serial_println!("[TRI MOTOR LIA] Executando em: '{}'", arg_str);  // Serial
        
        let data = arg_str.as_bytes();
        let (coords, result, resonance) = tri_motor::full_cycle(data);  // Fix: 1 arg, default None
        
        println!("Coordenadas TRI (x,y,z): ({}, {}, {})", coords.0, coords.1, coords.2);  // VGA
        println!("Ressonância: {} (baixa-média-alta)", resonance);
        println!("Saída do dispatcher:");
        
        // Mostra saída
        if let Ok(s) = core::str::from_utf8(result) {
            println!("  Texto: '{}'", s);
        } else {
            println!("  Hex preview:");
            for (i, byte) in result.iter().take(16).enumerate() {
                if i % 8 == 0 { print!("\n  "); }
                print!("{:02X} ", byte);
            }
            if result.len() > 16 {
                println!(" ... ({} bytes total)", result.len());
            } else {
                println!("");
            }
        }
        
        serial_println!("[TRI MOTOR] Coords: ({}, {}, {}) | Ressonância: {} | Saída len: {}", coords.0, coords.1, coords.2, resonance, result.len());  // Serial summary
        
    } else if cmd_str == "tri-ratio" {
        let original: [u8; 32] = *b"TRI Test no Shell!!!\0\0\0\0\0\0\0\0\0\0\0\0";
        let compressed = tri_compress::compress(&original);
        let orig_len = 32u32;
        let comp_len = compressed.iter().position(|&x| x == 0).unwrap_or(0) as u32;
        let ratio = if comp_len > 0 { (orig_len * 100 / comp_len) as u8 } else { 100 };
        println!("TRI Ratio: {}% (comp: {} -> {})", ratio, orig_len, comp_len);  // VGA
        serial_println!("TRI Ratio: {}% (comp: {} -> {})", ratio, orig_len, comp_len);  // Serial
    } else if cmd_str == "cat" && !arg_str.is_empty() {
        if let Some(content) = virtual_fs::read_file(arg_str) {
            println!("Conteúdo de {}:", arg_str);  // VGA
            println!("{}", core::str::from_utf8(content).unwrap_or("Erro UTF8"));
            serial_println!("Cat {}: [VGA mostra conteúdo]", arg_str);  // Serial log
        } else {
            println!("Arquivo não encontrado: {}", arg_str);  // VGA
            serial_println!("Arquivo não encontrado: {}", arg_str);  // Serial
        }
    } else if cmd_str == "ls" {
        println!("Arquivos no FS virtual:");  // VGA
        for &file in virtual_fs::list_files() {
            println!("  {}", file);
        }
        serial_println!("LS: [VGA lista arquivos]");  // Serial log
    } else if cmd_str == "halt" || cmd_str == "exit" {
        println!("Haltando TRI Kernel...");  // VGA
        serial_println!("Haltando TRI Kernel...");  // Serial
        loop {}
    } else {
        println!("Comando desconhecido '{}'. Digite 'help'.", cmd_str);  // VGA
        serial_println!("Comando desconhecido '{}'. Digite 'help'.", cmd_str);  // Serial
    }
}

// --- Shell (VGA clear + banner, duplo prompt) ---
fn tri_shell() {
    let mut command_buffer = [0u8; 128];
    
    // Limpa tela VGA e banner
    vga::get_writer().lock().clear_screen();  // Fix: get_writer().lock()
    println!("=== Bem-vindo ao TRI Shell v0.1 (VGA ON!) ===");  // VGA
    println!("Digite 'help' pra começar. (Teclado via IRQ!)");
    
    // Serial log
    serial_println!("=== TRI Shell iniciado (VGA + Serial) ===");
    
    loop {
        print!("tri-shell> ");  // VGA
        serial_print("tri-shell> ");  // Serial
        
        let len = read_line(&mut command_buffer);
        if len == 0 { continue; }
        let (cmd, arg) = parse_command(&command_buffer);
        handle_command(cmd, arg);
    }
}

// --- Entry Point ---
entry_point!(_start);

fn _start(_boot_info: &'static BootInfo) -> ! {
    serial_init();

    // Novo: Init VGA
    vga::init_vga(vga::Color::LightCyan, vga::Color::Black);

    // TRI Test
    let original: [u8; 32] = *b"TRI-Kernel Boot!!!\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
    let compressed = tri_compress::compress(&original);
    let decompressed = tri_compress::decompress(&compressed);
    let orig_len = 32u32;
    let comp_len = compressed.iter().position(|&x| x == 0).unwrap_or(0) as u32;
    let ratio = if comp_len > 0 { (orig_len * 100 / comp_len) as u8 } else { 100 };
    let decomp_ok = if decompressed.iter().zip(original.iter()).all(|(&a, &b)| a == b) { "OK" } else { "FAIL" };
    
    serial_println!("Boot OK! TRI Ratio: {}%. Decomp: {}", ratio, decomp_ok);
    println!("Boot OK! TRI Ratio: {}%. Decomp: {}", ratio, decomp_ok);  // Também na VGA

    // Init Interrupts
    serial_println!("Inicializando IDT e IRQs...");
    println!("Inicializando IDT e IRQs...");  // VGA
    interrupts::init_idt();
    
    interrupts::init_pics();
    
    instructions::interrupts::enable();
    serial_println!("Interrupções habilitadas (Timer + Teclado)");
    println!("Interrupções habilitadas (Timer + Teclado)");  // VGA

    // Init Keyboard
    keyboard::init();
    serial_println!("Keyboard init OK");
    println!("Keyboard init OK");  // VGA

    // Init Virtual FS
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
    
    serial_println!("Init: Executando /bin/shell...");
    println!("Init: Executando /bin/shell...");  // VGA
    tri_shell();

    loop {}
}
