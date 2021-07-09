#![feature(asm)]
#![no_main]
#![no_std]

mod riscv;
mod param;
mod start;

use crate::riscv::register::tp;
use crate::riscv::uart;

#[no_mangle]
pub fn main() -> ! {
    if tp::read() == 0 {
        let m_uart = uart::read();
        m_uart.puts("Hello World\n");
    }

    loop {}
}
