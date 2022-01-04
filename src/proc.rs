//! kernel process table

use crate::param::{NPROC, LEN_PROCNAME};
use lazy_static::lazy_static;
use spin::Mutex;

/// Process state
#[derive(Eq,PartialEq)]
pub enum ProcState {
    RUNNABLE,
    RUNNING,
}

pub struct Proc {
    state: ProcState,
    kstack: u64,
    name: [char;LEN_PROCNAME],
}

impl Proc {
    pub const fn new() -> Self {
        Self {
            state: ProcState::RUNNABLE,
            kstack: 0,
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
}
