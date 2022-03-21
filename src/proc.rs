//! kernel process table

use crate::context::Context;
use crate::memorylayout::kstack;
use crate::param::{NPROC, LEN_PROCNAME};
use crate::scheduler::get_scheduler;
use alloc::boxed::Box;
use core::sync::atomic::{AtomicUsize, Ordering};

/// Process state
#[derive(Eq,PartialEq)]
pub enum ProcState {
    RUNNABLE,
    RUNNING,
}

pub fn get_pid() -> usize {
    static PID_GENERATOR: AtomicUsize = AtomicUsize::new(0);
    let pid = PID_GENERATOR.fetch_add(1, Ordering::Relaxed);
    pid
}

pub struct Proc {
    pub state: ProcState,
    pub context: Context,
    pub kstack: u64,
    pub pid: usize,
    pub name: [char;LEN_PROCNAME],
}

impl Proc {
    pub fn new(kstack: u64) -> Self {
        Self {
            state: ProcState::RUNNABLE,
            context: Context::new(),
            kstack,
            pid: 0,
            name: ['\0';LEN_PROCNAME],
        }
    }
}

/// initialize kernel process stack
pub fn init_proc() {
    let scheduler = get_scheduler();
    for i in 0..NPROC {
        let mut proc = Box::new(Proc::new(
            kstack(i as u64)
        ));
        scheduler.unused.push(proc)
    }
}
