#![feature(asm)]
#![no_main]
#![no_std]

use core::panic::PanicInfo;

mod param;
mod mstatus;
mod uart;

#[no_mangle]
static STACK0: [u8;param::STACK_SIZE * param::NCPU] = [0;param::STACK_SIZE * param::NCPU];

#[no_mangle]
fn start() -> ! {
    let mut ms = mstatus::read();
    ms.set_mpp(mstatus::Mode::SupervisedMode);
    mstatus::write(ms);

    let m_uart = uart::read();
    m_uart.puts("Hello World\n");

    loop{}
}

#[panic_handler]
fn panic(_panic: &PanicInfo<'_>) -> ! {
    loop {}
}
