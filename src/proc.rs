//! kernel process table

include!(concat!(env!("OUT_DIR"), "/initcode.rs"));

use crate::memorylayout::kstack;
use crate::param::{NPROC, LEN_PROCNAME};
use crate::proc_util::{Context, TrapFrame};
use crate::scheduler::get_scheduler;
use crate::kalloc::{kalloc, kfree};
use crate::riscv::PAGESIZE;
use crate::vm::page_table::PageTable;
use crate::kvm::{init_user_pagetable, init_uvm};
use crate::trap::usertrapret;

use alloc::boxed::Box;
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
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
    pub memory_size: u64,
    pub name: [u8;LEN_PROCNAME],
    pub trapframe: NonNull<TrapFrame>,
    pub pagetable: NonNull<PageTable>,
}

impl Proc {
    pub fn new(kstack: u64) -> Self {
        Self {
            state: ProcState::RUNNABLE,
            context: Context::new(),
            kstack,
            pid: 0,
            memory_size: 0,
            name: [0;LEN_PROCNAME],
            trapframe: NonNull::dangling(),
            pagetable: NonNull::dangling(),
        }
    }

    /// Reset process to initial state
    pub fn reset(&mut self, free_memory: bool) {
        self.state = ProcState::RUNNABLE;
        self.context.reset();
        self.pid = 0;
        self.memory_size = 0;
        self.name = [0;LEN_PROCNAME];
        if free_memory {
            kfree(self.trapframe.as_ptr() as *mut _);
            self.trapframe = NonNull::dangling();
        } else {
            unsafe {
                self.trapframe.as_mut().reset();
            }
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
        let proc = Box::new(Proc::new(
            kstack(i as u64)
        ));
        scheduler.unused.push(proc)
    }
}

pub fn forkret() {
    static FIRST_USER_PROCESS: AtomicBool = AtomicBool::new(true);

    let is_first = FIRST_USER_PROCESS.swap(false, Ordering::Relaxed);
    if is_first {
    }

    unsafe {
        usertrapret();
    }

}

/// setup user process
pub fn alloc_process(proc: &mut Proc) -> Result<(), &str> {
    // allocate memory for trapframe
    proc.trapframe = NonNull::new(kalloc() as *mut _)
        .ok_or("kalloc failed in alloc user trapframe")?;

    // allocate memory for pagetable
    proc.pagetable = init_user_pagetable(&proc)
        .ok_or("kalloc failed in alloc user pagetable")?;

    // setup new context to start execution at forkret.
    // forkret will return to user space
    proc.context.reset();
    proc.context.ra = forkret as u64;
    proc.context.sp = proc.kstack + PAGESIZE;

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
        Err(_s) => {
            proc.reset(true);
            panic!("init_userproc: alloc_process");
        }
        Ok(()) => {
            // initialize user pid
            proc.pid = get_pid();

            // initialize memory map
            unsafe {
                let pagetable = proc.pagetable.as_mut();
                init_uvm(pagetable, &INITCODE);
            }
            proc.memory_size = PAGESIZE;

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
