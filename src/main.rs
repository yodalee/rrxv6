#![feature(asm)]
#![no_main]
#![no_std]

mod asm;
mod param;
mod mstatus;
mod uart;
mod mepc;
mod start;
mod delegate;
mod supervisor_interrupt;
mod pmp;

#[no_mangle]
pub fn main() -> ! {
    let m_uart = uart::read();
    m_uart.puts("Hello World\n");

    loop {}
}
