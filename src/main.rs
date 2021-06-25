#![no_main]
#![no_std]

use core::panic::PanicInfo;

#[no_mangle]
fn start() -> ! {
    loop{}
}

#[panic_handler]
fn panic(_panic: &PanicInfo<'_>) -> ! {
    loop {}
}
