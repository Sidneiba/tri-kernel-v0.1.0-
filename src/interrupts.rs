use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use pic8259::ChainedPics;
use x86_64::instructions::port::Port;
use lazy_static::lazy_static;
use spin::Mutex;

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard = PIC_1_OFFSET + 1,
}

static mut IDT: InterruptDescriptorTable = InterruptDescriptorTable::new();

lazy_static! {
    static ref PICS: Mutex<ChainedPics> = {
        Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) })
    };
}

pub fn init_idt() {
    unsafe {
        IDT[InterruptIndex::Timer as usize].set_handler_fn(timer_handler);
        IDT[InterruptIndex::Keyboard as usize].set_handler_fn(keyboard_handler);
        IDT.load();
    }
}

extern "x86-interrupt" fn timer_handler(_stack_frame: InterruptStackFrame) {
    // Correção: Unsafe pro notify (função unsafe)
    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer as u8);
    }
}

extern "x86-interrupt" fn keyboard_handler(_stack_frame: InterruptStackFrame) {
    let mut port = Port::<u8>::new(0x60);
    let scancode = unsafe { port.read() };

    // Debug print
    crate::serial_print("Scancode: 0x");
    print_hex(scancode);
    crate::serial_println!("");

    // Add to buffer
    crate::keyboard::add_scancode(scancode);

    // Correção: Unsafe pro notify
    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Keyboard as u8);
    }
}

fn print_hex(byte: u8) {
    let nibbles = [byte >> 4, byte & 0xF];
    for nib in nibbles {
        let c = if nib < 10 { b'0' + nib } else { b'A' + (nib - 10) };
        crate::serial_putc(c);
    }
}

// Correção: Unsafe pro initialize
pub fn init_pics() {
    unsafe {
        PICS.lock().initialize();
    }
}
