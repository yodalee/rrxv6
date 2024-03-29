//! The data owned by each CPU

use crate::param::NCPU;
use crate::proc::Proc;
use crate::proc_util::Context;
use crate::trap::{pop_off, push_off};
use alloc::boxed::Box;
use core::ptr;
use rv64::register::tp;
use spin::Mutex;

pub struct Cpu {
    pub proc: *mut Box<Proc>, // the process id running on this cpu
    pub context: Context,
    pub interrupt_base: Mutex<bool>,
    pub push_count: Mutex<u32>,
}

impl Cpu {
    pub const fn new() -> Self {
        Self {
            proc: ptr::null_mut(),
            context: Context::new(),
            interrupt_base: Mutex::new(false),
            push_count: Mutex::new(0),
        }
    }
}

static mut CPU: [Option<Cpu>; NCPU] = {
    const INIT_CPU: Option<Cpu> = None;
    [INIT_CPU; NCPU]
};

pub fn init_cpu() {
    for i in 0..NCPU {
        unsafe {
            CPU[i] = Some(Cpu::new());
        }
    }
}

/// Must be called with interrupts disabled,
/// to prevent race with process being moved
/// to a different CPU.
pub fn get_cpuid() -> u64 {
    tp::read()
}

pub fn get_cpu() -> &'static mut Cpu {
    let id = get_cpuid() as usize;
    unsafe { CPU[id].as_mut().unwrap() }
}

pub fn get_proc() -> *mut Box<Proc> {
    push_off();
    let cpu = get_cpu();
    let proc = cpu.proc;
    pop_off();
    proc
}
