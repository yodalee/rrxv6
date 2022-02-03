
static mut SCHEDULER: Option<Scheduler> = None;

pub struct Scheduler {
}

impl Scheduler {
    fn new() -> Self {
        Self {
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
