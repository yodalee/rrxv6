use crate::cpu::get_cpu;
use crate::list::List;
use crate::proc::{Proc, ProcState};
use crate::proc_util::Context;
use crate::trap::intr_on;
use alloc::boxed::Box;
use core::ptr::null_mut;
use rv64::asm::wfi;
use spin::Mutex;

extern "Rust" {
    // store ctx1 and load ctx2
    fn switch(ctx1: *mut Context, ctx2: *mut Context);
}

static mut SCHEDULER: Option<Scheduler> = None;

pub struct Scheduler {
    pub used: Mutex<List<Box<Proc>>>,
    pub unused: List<Box<Proc>>,
}

impl Scheduler {
    fn new() -> Self {
        Self {
            used: Mutex::new(List::new()),
            unused: List::new(),
        }
    }

    pub fn next(&self) -> Option<Box<Proc>> {
        let mut used_list = self.used.lock();
        used_list.pop()
    }

    pub fn schedule(&self) -> ! {
        loop {
            intr_on();
            match self.next() {
                Some(mut proc) => {
                    let cpu = get_cpu();
                    proc.state = ProcState::RUNNING;
                    unsafe {
                        cpu.proc = &mut proc as *mut Box<Proc>;
                        switch(
                            &mut cpu.context as *mut Context,
                            &mut proc.context as *mut Context,
                        );
                    }
                    let mut used_list = self.used.lock();
                    used_list.push(proc);
                }
                None => {
                    intr_on();
                    wfi();
                }
            }
        }
    }
}

pub fn get_scheduler() -> &'static mut Scheduler {
    unsafe { SCHEDULER.as_mut().unwrap() }
}

pub fn init_scheduler() {
    unsafe {
        SCHEDULER = Some(Scheduler::new());
    }
}

/// give up the CPU and return to scheduler
pub fn yield_proc() {
    let cpu = get_cpu();
    unsafe {
        let mut proc = &mut *cpu.proc as &mut Box<Proc>;
        cpu.proc = null_mut();
        proc.state = ProcState::RUNNABLE;

        switch(
            &mut proc.context as *mut Context,
            &mut cpu.context as *mut Context,
        );
    }
}
