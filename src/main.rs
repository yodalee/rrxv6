#![feature(default_free_fn)]
#![feature(alloc_error_handler)]
#![feature(ptr_metadata)] // from_raw_parts in kvm.rs
#![feature(try_trait_v2)]
#![feature(strict_provenance)] // NonNull.addr
#![no_main]
#![no_std]

#[macro_use]
extern crate alloc;
extern crate rv64;

mod console;
mod cpu;
mod disk;
mod kalloc;
mod kvm;
mod list;
mod memorylayout;
mod param;
mod plic;
#[macro_use]
mod print;
mod proc;
mod proc_util;
mod riscv;
mod scheduler;
mod start;
mod syscall;
mod trap;
mod uart;
mod virtio;
mod vm;

use crate::cpu::{get_cpuid, init_cpu};
use crate::disk::{init_disk, read_disk};
use crate::kalloc::init_heap;
use crate::kvm::{init_kvm, init_page};
use crate::plic::{init_hartplic, init_plic};
use crate::print::println;
use crate::proc::{init_proc, init_userproc};
use crate::scheduler::{get_scheduler, init_scheduler};
use crate::trap::{init_harttrap, intr_off, intr_on};

use alloc::alloc::Layout;
use core::sync::atomic::{AtomicBool, Ordering};
use linked_list_allocator::LockedHeap;
use rv64::asm::sync_synchronize;

#[no_mangle]
pub fn main() -> ! {
    static KERNEL_STARTED: AtomicBool = AtomicBool::new(false);
    if get_cpuid() == 0 {
        init_scheduler(); // initialize scheduler for schedule
        init_cpu(); // initialize cpu struct

        println!("rrxv6 start");

        init_heap(); // initialize physical memory allocator
        init_kvm(); // initialize kernel page table
        init_page(); // initialize virtual memory
        init_proc(); // initialize process table
        init_harttrap(); // install kernel trap vector
        init_plic(); // initialize PLIC interrupt controller
        init_hartplic(); // ask PLIC for device interrupt
        init_disk(); // emulated hard disk

        init_userproc(); // create first user process

        sync_synchronize();
        KERNEL_STARTED.swap(true, Ordering::Relaxed);
    } else {
        while !KERNEL_STARTED.load(Ordering::Relaxed) {}
        sync_synchronize();
        println!("hart {} starting", get_cpuid());
        init_page(); // initialize virtual memory
        init_harttrap(); // install kernel trap vector
        init_plic(); // initialize PLIC interrupt controller
        init_hartplic(); // ask PLIC for device interrupt
    }

    // experimentally trigger the read function with interrupt
    // uncomment to see the effect
    // intr_on();
    // read_disk();
    // intr_off();

    let scheduler = get_scheduler();
    // start scheduling, this function shall not return
    scheduler.schedule();
}

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    panic!("allocation error {:?}", layout);
}
