use crate::param;
use crate::context::Context;
use crate::SCHEDULER;

use core::default::Default;

type TaskStack = [u8;param::STACK_SIZE];

extern "Rust" {
    fn sys_switch(ctx1: *mut Context, ctx2: *mut Context);
}

#[derive(Debug)]
pub struct Scheduler {
    pub stack_task: [TaskStack;param::NPROC],
    pub ctx_task:   [Context;  param::NPROC],
    pub ctx_os:     Context,

    pub task_cnt: usize,
    pub current_task: usize,
}

impl Default for Scheduler {
    fn default() -> Self {
        Self {
            stack_task: [[0;param::STACK_SIZE];param::NPROC],
            ctx_task:   [Context::default();param::NPROC],
            ctx_os:     Default::default(),

            task_cnt: 0,
            current_task: 0,
        }
    }
}

pub fn os_kernel() {
    task_os();
}

pub fn task_create(f: fn()) {
    unsafe {
        let idx = SCHEDULER.task_cnt;
        let stack_top = &SCHEDULER.stack_task[idx] as *const u8 as u64 + ((param::STACK_SIZE-1) as u64);
        SCHEDULER.ctx_task[idx].sp = stack_top;
        SCHEDULER.ctx_task[idx].ra = f as u64;
        SCHEDULER.task_cnt += 1;
    }
}

pub fn task_go(i: usize) {
    unsafe {
        let mut ctx_os = &mut SCHEDULER.ctx_os;
        let mut ctx_new = &mut SCHEDULER.ctx_task[i];
        sys_switch(ctx_os as *mut Context,
                   ctx_new as *mut Context);
    }
}

pub fn task_os() {
    unsafe {
        let mut ctx_os = &mut SCHEDULER.ctx_os;
        let mut ctx_now = &mut SCHEDULER.ctx_task[SCHEDULER.current_task];
        sys_switch(ctx_now as *mut Context,
                   ctx_os as *mut Context);
    }
}

