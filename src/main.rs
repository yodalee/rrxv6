#![feature(asm)]
#![feature(default_free_fn)]
#![feature(alloc_error_handler)]
#![no_main]
#![no_std]

extern crate alloc;

mod context;
mod riscv;
mod param;
mod start;
mod proc;
mod scheduler;
mod util;
mod uart;
mod memorylayout;
mod memory;

use crate::riscv::register::tp;
use crate::scheduler::{Scheduler, task_go};
use crate::context::Context;
use crate::proc::user_init;
use crate::memory::HeapAllocator;

use lazy_static::lazy_static;
use spin::Mutex;

use core::default;
use core::convert::TryInto;

lazy_static! {
    static ref SCHEDULER: Mutex<Scheduler> = Mutex::new(Scheduler::default());
}

#[no_mangle]
pub fn main() -> ! {
    if tp::read() == 0 {
        let m_uart = uart::read();
        m_uart.puts("rrxv6 start\n");

        user_init();

        loop {
            m_uart.puts("OS: Activate next task\n");
            let idx = {
                let scheduler = SCHEDULER.lock();
                scheduler.current_task
            };
            task_go(idx);
            {
                let mut scheduler = SCHEDULER.lock();
                scheduler.current_task = (scheduler.current_task + 1) % scheduler.task_cnt;
            }
            m_uart.puts("OS: Back to OS\n\n");
        }
    }

    loop {}
}

#[global_allocator]
static ALLOCATOR: HeapAllocator = HeapAllocator;
