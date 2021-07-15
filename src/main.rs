#![feature(asm)]
#![feature(default_free_fn)]
#![no_main]
#![no_std]

mod context;
mod riscv;
mod param;
mod start;
mod spinlock;
mod user;

use crate::riscv::register::tp;
use crate::riscv::uart;
use crate::context::Context;
use core::default;
use crate::user::{CTX_TASK, TASK0_STACK, user_task0};

use core::convert::TryInto;

static mut CTX_OS: Context = Context {
    ra:  0,
    sp:  0,
    s0:  0,
    s1:  0,
    s2:  0,
    s3:  0,
    s4:  0,
    s5:  0,
    s6:  0,
    s7:  0,
    s8:  0,
    s9:  0,
    s10: 0,
    s11: 0,
};

#[no_mangle]
pub fn main() -> ! {
    extern "Rust" {
        fn sys_switch(ctx1: *mut Context, ctx2: *mut Context);
    }

    if tp::read() == 0 {
        let m_uart = uart::read();
        m_uart.puts("rrxv6 start\n");

        unsafe {
            CTX_TASK.ra = user_task0 as u64;
            CTX_TASK.sp = TASK0_STACK.as_ptr() as u64;
            sys_switch(&mut CTX_OS as *mut _,
                       &mut CTX_TASK as *mut _);
        }
    }

    loop {}
}
