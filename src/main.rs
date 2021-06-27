#![feature(asm)]
#![no_main]
#![no_std]

use core::panic::PanicInfo;

mod param;
mod mstatus;

#[no_mangle]
static STACK0: [u8;param::STACK_SIZE * param::NCPU] = [0;param::STACK_SIZE * param::NCPU];

#[no_mangle]
fn start() -> ! {
    let mut ms = mstatus::read();
    ms.set_mpp(mstatus::Mode::SupervisedMode);
    mstatus::write(ms);

    loop{}
}

#[panic_handler]
fn panic(_panic: &PanicInfo<'_>) -> ! {
    loop {}
}
