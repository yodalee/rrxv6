use core::panic::PanicInfo;

use crate::param;
use crate::mstatus;
use crate::mepc;
use crate::uart;

#[no_mangle]
static STACK0: [u8;param::STACK_SIZE * param::NCPU] = [0;param::STACK_SIZE * param::NCPU];

#[no_mangle]
fn start() -> ! {
    extern "Rust" {
        fn main() -> !;
    }

    let mut ms = mstatus::read();
    ms.set_mpp(mstatus::Mode::SupervisedMode);
    mstatus::write(ms);

    let m_mepc = mepc::Mepc::from_bits(main as u64);
    mepc::write(m_mepc);

    unsafe { asm!("mret"); }

    // mret will jump into kernel, should not execute to here
    loop {}
}

#[panic_handler]
fn panic(_panic: &PanicInfo<'_>) -> ! {
    loop {}
}
