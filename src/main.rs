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
mod kvm;
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
use crate::kvm::{init_kvm, init_page};

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
        init_kvm();
        init_page();

        m_uart.puts("OS started\n");
    }

    loop {}
}

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    panic!("allocation error {:?}", layout);
}
