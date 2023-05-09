//! The Console struct control the input/output to UART
//! Read line to interpreter.
//! Write character to the screen
//!
//! Special character includes:
//! * newline -- end of line
//! * control-h -- backspace
//! * control-u -- kill line
//! * control-d -- end of file
//! * control-p -- print process list

const BACKSPACE: u32 = 0x100;

use crate::param::CONSOLE_BUF_SIZE;
use crate::uart::Uart;
use lazy_static::lazy_static;
use spin::Mutex;

lazy_static! {
    pub static ref CONSOLE: Mutex<Console> = Mutex::new(Console::new());
}

pub struct Console {
    buf: [char; CONSOLE_BUF_SIZE],
    read_idx: usize,
    write_idx: usize,
    edit_idx: usize,
}

impl Console {
    pub fn new() -> Self {
        Self {
            buf: ['\0'; CONSOLE_BUF_SIZE],
            read_idx: 0,
            write_idx: 0,
            edit_idx: 0,
        }
    }

    fn console_putc(&self, c: char, uart: &mut Uart) {
        uart.putc(c);
    }

    // The console interrupt handler.
    // uart.handle_interrupt calls this for input character.
    pub fn console_interrupt(&mut self, c: char, uart: &mut Uart) {
        match c {
            '\0' => {} // Do nothing if it is a null character
            _ => {
                // echo character to user
                self.console_putc(c, uart);
            }
        }
    }
}
