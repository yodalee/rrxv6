
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

// qemu virt UART registers.
pub const UART0: usize = 0x1000_0000;

// core local interruptor (CLINT), which contains the timer
pub const CLINT : u64 = 0x2000000;
pub const CLINT_MTIMECMP : u64 = CLINT + 0x4000;
pub const CLINT_MTIME : u64 = 0x200BFF8;

// RAM from physical address 0x8000_0000 to PHYSTOP
// 128 MB available
pub const KERNELBASE : u64 = 0x8000_0000;
pub const PHYSTOP : u64 = KERNELBASE + 128 * 1024 * 1024;
