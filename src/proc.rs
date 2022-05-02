//! kernel process table

use crate::memorylayout::kstack;
use crate::param::{NPROC, LEN_PROCNAME};
use crate::proc_util::{Context, TrapFrame};
use crate::scheduler::get_scheduler;
use crate::kalloc::kalloc;
use crate::riscv::PAGESIZE;

use alloc::boxed::Box;
use core::sync::atomic::{AtomicUsize, Ordering};
use core::ptr::NonNull;

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
    pub name: [u8;LEN_PROCNAME],
    pub trapframe: NonNull<TrapFrame>,
}

impl Proc {
    pub fn new(kstack: u64) -> Self {
        Self {
            state: ProcState::RUNNABLE,
            context: Context::new(),
            kstack,
            pid: 0,
            name: [0;LEN_PROCNAME],
            trapframe: NonNull::dangling(),
        }
    }

    /// Reset process to initial state
    pub fn reset(&mut self) {
        self.state = ProcState::RUNNABLE;
        self.context.reset();
        self.pid = 0;
        self.name = [0;LEN_PROCNAME];
        unsafe {
            self.trapframe.as_mut().reset();
        }
    }

    pub fn set_name(&mut self, s: &str) {
        for (dest,src) in self.name.iter_mut().zip(s.bytes()) {
            *dest = src;
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

/// setup user process
pub fn alloc_process(proc: &mut Proc) -> Result<(), &str> {
    // allocate memory for trapframe
    proc.trapframe = NonNull::new(kalloc() as *mut _)
        .ok_or("kalloc failed in alloc trapframe")?;

    Ok(())
}

/// initialize first user process
pub fn init_userproc() {
    let scheduler = get_scheduler();

    let mut proc = match scheduler.unused.pop() {
        None => panic!("init_userproc failed"),
        Some(proc) => proc,
    };

    match alloc_process(&mut proc) {
        Err(s) => {
            // cleanup process
            panic!("init_userproc: {}", s);
        }
        Ok(()) => {
            // initialize user pid
            proc.pid = get_pid();

            // Note that first user process will have its pid 0
            // we don't save additional pointer to this process
            assert!(proc.pid == 0, "User process init pid != 0");

            // initialize trapfraem
            unsafe {
                let trapframe = proc.trapframe.as_mut();
                trapframe.epc = 0;
                trapframe.sp = PAGESIZE;
            }

            // set process name
            proc.set_name("initcode");

            // set state to RUNNABLE
            proc.state = ProcState::RUNNABLE;

            let mut used_list = scheduler.used.lock();
            used_list.push(proc);
        },
    }
}
