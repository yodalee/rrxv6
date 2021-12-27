#![feature(asm)]
#![feature(default_free_fn)]
#![feature(alloc_error_handler)]
#![feature(const_panic)]
#![no_main]
#![no_std]

#[macro_use]
extern crate alloc;
extern crate rv64;

mod cpu;
mod kalloc;
mod kvm;
mod memorylayout;
mod param;
mod plic;
mod proc;
mod riscv;
mod start;
mod trap;
mod uart;
mod vm;

use crate::cpu::get_cpuid;
use crate::kalloc::init_heap;
use crate::kvm::{init_kvm, init_page};
use crate::plic::{init_plic, init_hartplic};
use crate::proc::init_proc;
use crate::uart::UART;
use crate::trap::{init_harttrap, intr_on, intr_off};

use linked_list_allocator::LockedHeap;
use alloc::alloc::Layout;

#[no_mangle]
pub fn main() -> ! {
    if get_cpuid() == 0 {
        let mut m_uart = UART.lock();
        m_uart.puts("rrxv6 start\n");
        drop(m_uart);

        init_heap();     // initialize physical memory allocator
        init_kvm();      // initialize kernel page table
        init_page();     // initialize virtual memory
        init_proc();     // initialize process table
        init_harttrap(); // install kernel trap vector
        init_plic();     // initialize PLIC interrupt controller
        init_hartplic(); // ask PLIC for device interrupt

        let mut m_uart = UART.lock();
        m_uart.puts("OS started\n");
        drop(m_uart);
    }
    intr_on();

    loop {}
}

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    panic!("allocation error {:?}", layout);
}
