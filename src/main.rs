#![feature(asm)]
#![feature(default_free_fn)]
#![no_main]
#![no_std]

mod context;
mod riscv;
mod param;
mod start;
mod proc;
mod scheduler;
mod util;

use crate::riscv::register::tp;
use crate::riscv::uart;
use crate::scheduler::{Scheduler, task_go};
use crate::context::Context;
use crate::proc::user_init;

// use lazy_static::lazy_static;

use core::default;
use core::convert::TryInto;

static mut SCHEDULER: Scheduler = Scheduler {
    ctx_task:   [Context {
        ra:0, sp:0,
        s: [0;12] };param::NPROC],
    ctx_os:     Context{ ra:0, sp:0, s:[0;12] },
    task_cnt: 0,
    current_task: 0,
};

#[no_mangle]
pub fn main() -> ! {
    if tp::read() == 0 {
        let m_uart = uart::read();
        m_uart.puts("rrxv6 start\n");

        user_init();

        loop {
            m_uart.puts("OS: Activate next task\n");
            unsafe {
                task_go(SCHEDULER.current_task);
                SCHEDULER.current_task = (SCHEDULER.current_task + 1) % SCHEDULER.task_cnt;
            }
            m_uart.puts("OS: Back to OS\n");
            m_uart.putc('\n');
        }
    }

    loop {}
}
