use core::fmt;
use core::fmt::Write; // Mantido para compatibilidade com macros
use core::str;

// Trait simples pra Writer (abstrai output: serial ou VGA)
pub trait Writer {
    fn write_byte(&mut self, byte: u8);
    fn write_string(&mut self, s: &str);
}

pub struct DummyWriter; // Fallback vazio pra testes

impl Writer for DummyWriter {
    fn write_byte(&mut self, _byte: u8) {}
    fn write_string(&mut self, _s: &str) {}
}

// Implementa fmt::Write para suportar macros como print!
impl fmt::Write for dyn Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

// Função print genérica (usa Writer)
pub fn print(writer: &mut dyn Writer, s: &str) {
    writer.write_string(s);
}

// Shell loop
const PROMPT: &str = "tri> ";
const CMD_BUF_SIZE: usize = 128;
const HISTORY_SIZE: usize = 5; // Armazena até 5 comandos no histórico

pub fn shell_loop(writer: &mut dyn Writer) {
    use crate::keyboard; // Módulo keyboard

    let mut buffer: [u8; CMD_BUF_SIZE] = [0; CMD_BUF_SIZE];
    let mut idx: usize = 0;

    // Histórico de comandos
    static mut HISTORY: [[u8; CMD_BUF_SIZE]; HISTORY_SIZE] = [[0; CMD_BUF_SIZE]; HISTORY_SIZE];
    static mut HISTORY_COUNT: usize = 0;
    static mut HISTORY_POS: usize = 0;

    print(writer, PROMPT);

    loop {
        if let Some(byte) = keyboard::get_key() {
            match byte {
                b'\n' | b'\r' => { // Enter
                    buffer[idx] = 0; // Null-terminate
                    print(writer, "\n");

                    // Salvar comando no histórico (se não for vazio)
                    if idx > 0 {
                        unsafe {
                            HISTORY[HISTORY_POS] = buffer;
                            HISTORY_POS = (HISTORY_POS + 1) % HISTORY_SIZE;
                            if HISTORY_COUNT < HISTORY_SIZE {
                                HISTORY_COUNT += 1;
                            }
                        }
                    }

                    // Handle command
                    unsafe {
                        handle_command(writer, &buffer[..idx], &HISTORY, HISTORY_COUNT, HISTORY_POS);
                    }

                    // Reset
                    idx = 0;
                    buffer.fill(0);
                    print(writer, PROMPT);
                }
                8 | b'\x7F' => { // Backspace ou DEL
                    if idx > 0 {
                        idx -= 1;
                        print(writer, "\x08 \x08"); // Retrocede, espaço, retrocede
                    }
                }
                _ => {
                    if idx < CMD_BUF_SIZE - 1 && (byte.is_ascii_graphic() || byte == b' ') {
                        buffer[idx] = byte;
                        idx += 1;
                        // Eco (converte pra str pra print)
                        let buf = [byte]; // Correção do E0716
                        let echo = core::str::from_utf8(&buf).unwrap_or("?");
                        print(writer, echo);
                    }
                }
            }
        }
    }
}

fn handle_command(
    writer: &mut dyn Writer,
    cmd: &[u8],
    history: &[[u8; CMD_BUF_SIZE]; HISTORY_SIZE],
    history_count: usize,
    history_pos: usize,
) {
    let cmd_str = str::from_utf8(cmd).unwrap_or("");
    match cmd_str.trim() {
        "help" => {
            print(writer, "Comandos disponíveis:\n");
            print(writer, "  help    - mostra esta ajuda\n");
            print(writer, "  hello   - mensagem de teste\n");
            print(writer, "  tri-ratio - stats da compressão TRI\n");
            print(writer, "  halt    - para o kernel\n");
            print(writer, "  history - mostra os últimos comandos\n");
        }
        "hello" => {
            print(writer, "Olá, TRI Kernel! Bem-vindo ao mini-shell bare-metal.\n");
        }
        "tri-ratio" => {
            use crate::tri_compress;
            let original: [u8; 32] = *b"TRI Test no Shell!!!\0\0\0\0\0\0\0\0\0\0\0\0";
            let compressed = tri_compress::compress(&original);
            let orig_len = 32u32;
            let comp_len = (compressed.iter().position(|&x| x == 0).unwrap_or(64) / 2) as u32;
            let ratio = if comp_len > 0 { (orig_len * 100 / comp_len) as u8 } else { 100 };
            
            print(writer, "TRI Ratio: ");
            print(writer, u8_to_str(ratio));
            print(writer, "% (");
            print(writer, u32_to_str(orig_len));
            print(writer, " -> ");
            print(writer, u32_to_str(comp_len));
            print(writer, " bytes)\n");
        }
        "halt" => {
            print(writer, "Haltando TRI Kernel...\n");
            loop {
                x86_64::instructions::hlt();
            }
        }
        "history" => {
            print(writer, "Histórico de comandos:\n");
            if history_count == 0 {
                print(writer, "  (vazio)\n");
            } else {
                for i in 0..history_count {
                    let pos = (history_pos + i - history_count + HISTORY_SIZE) % HISTORY_SIZE;
                    let cmd_entry = str::from_utf8(&history[pos]).unwrap_or("?");
                    print(writer, "  ");
                    print(writer, u32_to_str((i + 1) as u32));
                    print(writer, ": ");
                    print(writer, cmd_entry);
                    print(writer, "\n");
                }
            }
        }
        "" => {} // Enter vazio
        _ => {
            print(writer, "Comando não reconhecido. Digite 'help'.\n");
        }
    }
}

// Função para formatar u8 como string
fn u8_to_str(n: u8) -> &'static str {
    static mut BUF: [u8; 3] = [0; 3]; // 2 dígitos + null

    unsafe {
        if n < 10 {
            BUF[0] = b'0' + n;
            BUF[1] = 0;
            core::str::from_utf8(&BUF[0..1]).unwrap_or("??")
        } else {
            BUF[0] = b'0' + (n / 10);
            BUF[1] = b'0' + (n % 10);
            BUF[2] = 0;
            core::str::from_utf8(&BUF[0..2]).unwrap_or("??")
        }
    }
}

// Função para formatar u32 como string
fn u32_to_str(mut n: u32) -> &'static str {
    static mut BUF: [u8; 11] = [0; 11]; // 10 dígitos + null
    static DIGITS: &[u8] = b"0123456789";

    unsafe {
        if n == 0 {
            BUF[0] = b'0';
            BUF[1] = 0;
            return core::str::from_utf8(&BUF[0..1]).unwrap_or("??");
        }

        let mut len = 0;
        let mut temp = n;
        while temp > 0 {
            len += 1;
            temp /= 10;
        }

        let mut i = len;
        while n > 0 {
            i -= 1;
            BUF[i] = DIGITS[(n % 10) as usize];
            n /= 10;
        }
        BUF[len] = 0;

        core::str::from_utf8(&BUF[0..len]).unwrap_or("??")
    }
}
