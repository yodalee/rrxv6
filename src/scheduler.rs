use crate::list::List;
use crate::proc::{Proc, ProcState, get_pid};
use crate::riscv::PAGESIZE;
use alloc::boxed::Box;
use spin::Mutex;

static mut SCHEDULER: Option<Scheduler> = None;

pub struct Scheduler {
    pub used: Mutex<List<Box<Proc>>>,
    pub unused: List<Box<Proc>>
}

impl Scheduler {
    fn new() -> Self {
        Self {
            used: Mutex::new(List::new()),
            unused: List::new(),
        }
    }

    pub fn spawn(&mut self, f: u64) {
        match self.unused.pop() {
            Some(mut proc) => {
                // initialize process
                proc.pid = get_pid();
                proc.state = ProcState::RUNNABLE;
                proc.context.reset();
                proc.context.ra = f;
                proc.context.sp = proc.kstack + PAGESIZE;

                let mut used_list = self.used.lock();
                used_list.push(proc);
            },
            None => {
                panic!("No unused process left");
            }
        }
    }

    pub fn schedule(&self) -> ! {
        loop {
        }
    }
}

pub fn get_scheduler() -> &'static mut Scheduler {
    unsafe {
        SCHEDULER.as_mut().unwrap()
    }
}

pub fn init_scheduler() {
    unsafe {
        SCHEDULER = Some(Scheduler::new());
    }
}
