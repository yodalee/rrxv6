//! Physical memory layout

//! qemu -machine virt is set up like this,
//! based on qemu's hw/riscv/virt.c:
//! https://github.com/qemu/qemu/blob/master/hw/riscv/virt.c
//!
//! 00001000 -- boot ROM, provided by qemu
//! 02000000 -- CLINT
//! 0C000000 -- PLIC
//! 10000000 -- uart0
//! 10001000 -- virtio disk
//! 80000000 -- boot ROM jumps here in machine mode
//!             -kernel loads the kernel here
//! unused RAM after 80000000.

//! the kernel uses physical memory thus:
//! 80000000 -- entry.S, then kernel text and data
//! end -- start of kernel page allocation area
//! PHYSTOP -- end RAM used by the kernel

use crate::riscv;

// qemu virt UART registers.
pub const UART0: u64 = 0x1000_0000;
pub const UART0_IRQ: u64 = 10;

// virtio mmio interface
pub const VIRTIO0: u64 = 0x10001000;
pub const VIRTIO0_IRQ: u64 = 1;

// core local interruptor (CLINT), which contains the timer
pub const CLINT: u64 = 0x2000000;
pub const CLINT_MTIME: u64 = 0x200BFF8;
#[inline]
pub fn clint_mtimecmp(hart: u64) -> u64 {
    CLINT + 0x4000 + 8 * hart
}

// qemu puts platform-level interrupt controller (PLIC) here.
pub const PLIC_BASE: u64 = 0x0c000000;
pub const PLIC_PRIORITY: u64 = PLIC_BASE + 0x0;
pub const PLIC_PENDING: u64 = PLIC_BASE + 0x1_000;
pub const PLIC_ENABLE: u64 = PLIC_BASE + 0x2_000;
pub const PLIC_THRESHOLD: u64 = PLIC_BASE + 0x200_000;
pub const PLIC_CLAIM: u64 = PLIC_BASE + 0x200_004;

// RAM from physical address 0x8000_0000 to PHYSTOP
// 128 MB available
pub const KERNELBASE: u64 = 0x8000_0000;
pub const PHYSTOP: u64 = KERNELBASE + 128 * 1024 * 1024;

// map the trampoline page to the highest address in both user and kernel space
pub const TRAMPOLINE: u64 = riscv::MAXVA - riscv::PAGESIZE;

// map kernel stacks beneath the trampoline,
// each surrounded by invalid guard pages.
#[inline]
pub fn kstack(proc_id: u64) -> u64 {
    TRAMPOLINE - (proc_id + 1) * 2 * riscv::PAGESIZE
}

// User memory layout.
// Address zero first:
//   text
//   original data and bss
//   fixed-size stack
//   expandable heap
//   ...
//   TRAPFRAME (p->trapframe, used by the trampoline)
//   TRAMPOLINE (the same page as in the kernel)
pub const TRAPFRAME: u64 = TRAMPOLINE - riscv::PAGESIZE;
