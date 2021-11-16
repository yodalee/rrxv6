#![feature(asm)]
#![feature(default_free_fn)]
#![feature(alloc_error_handler)]
#![feature(const_panic)]
#![no_main]
#![no_std]

#[macro_use]
extern crate alloc;
extern crate rv64;

mod context;
mod kalloc;
mod memorylayout;
mod param;
mod proc;
mod riscv;
mod scheduler;
mod start;
mod uart;
mod util;
mod vm;

use crate::scheduler::{Scheduler, task_go};
use crate::proc::user_init;
use crate::kalloc::init_heap;

use lazy_static::lazy_static;
use spin::Mutex;
use linked_list_allocator::LockedHeap;
use alloc::boxed::Box;
use alloc::alloc::Layout;
use rv64::register::tp;

lazy_static! {
    static ref SCHEDULER: Mutex<Scheduler> = Mutex::new(Scheduler::default());
}

#[no_mangle]
pub fn main() -> ! {
    if tp::read() == 0 {
        let m_uart = uart::read();
        m_uart.puts("rrxv6 start\n");

        user_init();
        init_heap(); // physical memory allocator

        let b = Box::new(64);

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
static ALLOCATOR: LockedHeap = LockedHeap::empty();

#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    panic!("allocation error {:?}", layout);
}
