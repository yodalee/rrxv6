
//! The data owned by each CPU

use lazy_static::lazy_static;
use rv64::register::tp;
use spin::Mutex;

use crate::context::Context;
use crate::param::NCPU;

pub struct Cpu {
    pub proc: Option<usize>, // the process id running on this cpu
    pub context: Context,
    pub interrupt_base: Mutex<bool>,
    pub push_count: Mutex<u32>,
}

impl Cpu {
    pub const fn new() -> Self {
        Self {
            proc: None,
            context: Context::new(),
            interrupt_base: Mutex::new(false),
            push_count: Mutex::new(0),
        }
    }
}

lazy_static! {
    pub static ref CPU: [Cpu;NCPU] = {
        const INIT_CPU: Cpu = Cpu::new();
        [INIT_CPU;NCPU]
    };
}

/// Must be called with interrupts disabled,
/// to prevent race with process being moved
/// to a different CPU.
pub fn get_cpuid() -> u64 {
    tp::read()
}

pub fn get_cpu() -> &'static Cpu {
    let id = get_cpuid() as usize;
    &CPU[id]
}
