use crate::list::List;
use crate::proc::Proc;
use alloc::boxed::Box;

static mut SCHEDULER: Option<Scheduler> = None;

pub struct Scheduler {
    pub unused: List<Box<Proc>>
}

impl Scheduler {
    fn new() -> Self {
        Self {
            unused: List::new(),
        }
    }

    pub fn spawn(&self) {
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
