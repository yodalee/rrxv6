#![feature(asm)]
#![no_main]
#![no_std]

mod param;
mod mstatus;
mod uart;
mod mepc;
mod start;

#[no_mangle]
pub fn main() -> ! {
    let m_uart = uart::read();
    m_uart.puts("Hello World\n");

    loop {}
}
