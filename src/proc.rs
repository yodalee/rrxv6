//! kernel process table

use crate::context::Context;
use crate::memorylayout::kstack;
use crate::param::{NPROC, LEN_PROCNAME};
use lazy_static::lazy_static;
use spin::Mutex;

/// Process state
#[derive(Eq,PartialEq)]
pub enum ProcState {
    RUNNABLE,
    RUNNING,
}

struct PidGenerator {
    gen: Mutex<usize>
}

impl PidGenerator {
    fn next(&self) -> usize {
        let mut i = self.gen.lock();
        let pid = *i;
        *i = *i + 1;
        pid
    }
}

pub struct Proc {
    state: ProcState,
    context: Context,
    kstack: u64,
    pid: usize,
    name: [char;LEN_PROCNAME],
}

impl Proc {
    pub const fn new() -> Self {
        Self {
            state: ProcState::RUNNABLE,
            context: Context::new(),
            kstack: 0,
            pid: 0,
            name: ['\0';LEN_PROCNAME],
        }
    }
}

lazy_static! {
    pub static ref PROC: [Mutex<Proc>;NPROC] = {
        const INIT_PROC: Mutex<Proc> = Mutex::new(Proc::new());
        [INIT_PROC;NPROC]
    };
}

/// initialize kernel process stack
pub fn init_proc() {
    for i in 0..NPROC {
        let mut proc = PROC[i].lock();
        proc.kstack = kstack(i as u64);
    }
}
