use core::fmt;
use core::str;

// Trait simples pra Writer (abstrai output: serial ou VGA)
pub trait Writer {
    fn write_byte(&mut self, byte: u8);
    fn write_string(&mut self, s: &str);
}

pub struct DummyWriter;  // Fallback vazio pra testes

impl Writer for DummyWriter {
    fn write_byte(&mut self, _byte: u8) {}
    fn write_string(&mut self, _s: &str) {}
}

// Implementa fmt::Write pra format!()
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

// Shell loop (seu código, adaptado pra Writer)
const PROMPT: &str = "tri> ";
const CMD_BUF_SIZE: usize = 128;

pub fn shell_loop(writer: &mut dyn Writer) {
    use crate::keyboard;  // Seu módulo keyboard

    let mut buffer: [u8; CMD_BUF_SIZE] = [0; CMD_BUF_SIZE];
    let mut idx: usize = 0;

    print(writer, PROMPT);

    loop {
        if let Some(byte) = keyboard::get_key() {
            match byte {
                b'\n' | b'\r' => {  // Enter
                    buffer[idx] = 0;  // Null-terminate
                    print(writer, "\n");

                    // Handle command
                    handle_command(writer, &buffer[..idx]);

                    // Reset
                    idx = 0;
                    buffer.fill(0);
                    print(writer, PROMPT);
                }
                8 | b'\x7F' => {  // Backspace ou DEL
                    if idx > 0 {
                        idx -= 1;
                        print(writer, "\x08 \x08");  // Retrocede, espaço, retrocede (apaga visual)
                    }
                }
                _ => {
                    if idx < CMD_BUF_SIZE - 1 && byte.is_ascii_graphic() || byte == b' ' {
                        buffer[idx] = byte;
                        idx += 1;
                        // Eco (converte pra str pra print)
                        let echo = str::from_utf8(&[byte]).unwrap_or("?");
                        print(writer, echo);
                    }
                }
            }
        }
        // Idle: hlt() já no get_key(), mas se quiser, x86_64::instructions::hlt();
    }
}

fn handle_command(writer: &mut dyn Writer, cmd: &[u8]) {
    let cmd_str = str::from_utf8(cmd).unwrap_or("");
    match cmd_str.trim() {
        "help" => {
            print(writer, "Comandos disponíveis:\n");
            print(writer, "  help    - mostra esta ajuda\n");
            print(writer, "  hello   - mensagem de teste\n");
            print(writer, "  tri-ratio - stats da compressão TRI\n");
            print(writer, "  halt    - para o kernel\n");
        }
        "hello" => {
            print(writer, "Olá, TRI Kernel! Bem-vindo ao mini-shell bare-metal.\n");
        }
        "tri-ratio" => {
            // Integra seu TRI (chama função do módulo)
            use crate::tri_compress;
            let original: [u8; 32] = *b"TRI Test no Shell!!!\0\0\0\0\0\0\0\0\0\0\0\0";
            let compressed = tri_compress::compress(&original);
            let orig_len = 32u32;
            let comp_len = (compressed.iter().position(|&x| x == 0).unwrap_or(64) / 2) as u32;
            let ratio = if comp_len > 0 { (orig_len * 100 / comp_len) as u8 } else { 100 };
            print(writer, &format!("TRI Ratio: {}% ({} -> {} bytes)\n", ratio, orig_len, comp_len)[..]);
        }
        "halt" => {
            print(writer, "Haltando TRI Kernel...\n");
            loop {}  // Loop infinito
        }
        "" => {}  // Enter vazio: pula
        _ => {
            print(writer, "Comando não reconhecido. Digite 'help'.\n");
        }
    }
}
