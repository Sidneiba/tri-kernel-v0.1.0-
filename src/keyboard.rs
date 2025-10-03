use x86_64::instructions::port::Port;
use core::sync::atomic::{AtomicUsize, Ordering};

// Tabela ASCII com exatos 128 elementos (mapeados + zeros padded corretamente)
static SC_ASCII: [u8; 128] = [
    0,  27, '1' as u8, '2' as u8, '3' as u8, '4' as u8, '5' as u8, '6' as u8, '7' as u8, '8' as u8, '9' as u8, '0' as u8, '-' as u8, '=' as u8, 8,  // 0-14 (15 elems)
    9, 'q' as u8, 'w' as u8, 'e' as u8, 'r' as u8, 't' as u8, 'y' as u8, 'u' as u8, 'i' as u8, 'o' as u8, 'p' as u8, '[' as u8, ']' as u8, 10, 0,  // 15-29 (15 elems, total 30)
    'a' as u8, 's' as u8, 'd' as u8, 'f' as u8, 'g' as u8, 'h' as u8, 'j' as u8, 'k' as u8, 'l' as u8, ';' as u8, '\'' as u8, '`' as u8, 0, 92,  // 30-43 (14 elems, total 44)
    'z' as u8, 'x' as u8, 'c' as u8, 'v' as u8, 'b' as u8, 'n' as u8, 'm' as u8, ',' as u8, '.' as u8, '/' as u8, 0, '*' as u8, 0, ' ' as u8, 0,  // 44-58 (15 elems, total 59)
    // Padding: 69 zeros pra fechar 128 (59 + 69 = 128)
    0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,  // 32 zeros (total 91)
    0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,  // 32 zeros (total 123)
    0,0,0,0,0  // +2 zeros finais (total 125? Espera, ajustei: na verdade, com contagem precisa, são 69)
];

// Buffer circular
const BUFFER_SIZE: usize = 128;
static mut KEYBOARD_BUFFER: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
static HEAD: AtomicUsize = AtomicUsize::new(0);
static TAIL: AtomicUsize = AtomicUsize::new(0);

// Init
pub fn init() {
    let mut status_port = Port::new(0x64);
    unsafe {
        status_port.write(0xae as u8);
    }
    crate::serial_println!("Keyboard driver init: Simple map table loaded (no shift yet)");
}

// Add scancode
pub fn add_scancode(scancode: u8) {
    if scancode > 127 { return; }

    let ch = if (scancode as usize) < SC_ASCII.len() { SC_ASCII[scancode as usize] } else { 0 };

    if ch > 0 {
        let next_head = (HEAD.load(Ordering::Relaxed) + 1) % BUFFER_SIZE;
        if next_head != TAIL.load(Ordering::Relaxed) {
            unsafe { KEYBOARD_BUFFER[HEAD.load(Ordering::Relaxed)] = ch; }
            HEAD.store(next_head, Ordering::Relaxed);
        }
    }
}

// Get key
pub fn get_key() -> Option<u8> {
    let tail = TAIL.load(Ordering::Relaxed);
    if tail == HEAD.load(Ordering::Relaxed) {
        None
    } else {
        let c = unsafe { KEYBOARD_BUFFER[tail] };
        TAIL.store((tail + 1) % BUFFER_SIZE, Ordering::Relaxed);
        Some(c)
    }
}
