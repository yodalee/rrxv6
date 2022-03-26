#![feature(asm)]
#![feature(default_free_fn)]
#![feature(alloc_error_handler)]
#![feature(const_panic)]
#![no_main]
#![no_std]

#[macro_use]
extern crate alloc;
extern crate rv64;

mod console;
mod context;
mod cpu;
mod kalloc;
mod kvm;
mod list;
mod memorylayout;
mod param;
mod plic;
mod print;
mod proc;
mod riscv;
mod scheduler;
mod start;
mod trap;
mod uart;
mod vm;

use crate::cpu::{get_cpuid, init_cpu};
use crate::kalloc::init_heap;
use crate::kvm::{init_kvm, init_page};
use crate::plic::{init_plic, init_hartplic};
use crate::print::println;
use crate::proc::init_proc;
use crate::scheduler::{init_scheduler, get_scheduler, yield_proc};
use crate::trap::init_harttrap;

use linked_list_allocator::LockedHeap;
use alloc::alloc::Layout;

#[no_mangle]
pub fn main() -> ! {
    if get_cpuid() == 0 {
        init_scheduler(); // initialize scheduler for schedule
        init_cpu();       // initialize cpu struct

        println("rrxv6 start");

        init_heap();      // initialize physical memory allocator
        init_kvm();       // initialize kernel page table
        init_page();      // initialize virtual memory
        init_proc();      // initialize process table
        init_harttrap();  // install kernel trap vector
        init_plic();      // initialize PLIC interrupt controller
        init_hartplic();  // ask PLIC for device interrupt

        println("OS started");
    }

    let scheduler = get_scheduler();
    scheduler.spawn(print_a as u64);
    scheduler.spawn(print_b as u64);
    // start scheduling, this function shall not return
    scheduler.schedule();
}

fn print_a() {
    loop {
        println("A");
        yield_proc();
    }
}

fn print_b() {
    loop {
        println("B");
        yield_proc();
    }
}

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    panic!("allocation error {:?}", layout);
}
