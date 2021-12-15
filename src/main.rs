#![feature(asm)]
#![feature(default_free_fn)]
#![feature(alloc_error_handler)]
#![feature(const_panic)]
#![no_main]
#![no_std]

#[macro_use]
extern crate alloc;
extern crate rv64;

mod kalloc;
mod kvm;
mod memorylayout;
mod param;
mod proc;
mod riscv;
mod start;
mod uart;
mod vm;

use crate::proc::init_proc;
use crate::kalloc::init_heap;
use crate::kvm::{init_kvm, init_page};

use linked_list_allocator::LockedHeap;
use alloc::alloc::Layout;
use rv64::register::tp;

#[no_mangle]
pub fn main() -> ! {
    if tp::read() == 0 {
        let m_uart = uart::read();
        m_uart.puts("rrxv6 start\n");

        init_heap(); // initialize physical memory allocator
        init_kvm();  // initialize kernel page table
        init_page(); // initialize virtual memory
        init_proc(); // initialize process table

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
