use crate::param;
use crate::context::Context;
use crate::SCHEDULER;

use core::default::Default;

type TaskStack = [u8;param::STACK_SIZE];
static STACK_TASK: [TaskStack;param::NPROC] = [[0;param::STACK_SIZE];param::NPROC];

extern "Rust" {
    fn sys_switch(ctx1: *mut Context, ctx2: *mut Context);
}

#[derive(Debug)]
pub struct Scheduler {
    pub ctx_task:   [Context;  param::NPROC],
    pub ctx_os:     Context,

    pub task_cnt: usize,
    pub current_task: usize,
}

impl Default for Scheduler {
    fn default() -> Self {
        Self {
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
    let mut scheduler = SCHEDULER.lock();
    let idx = scheduler.task_cnt;
    let stack_top = &STACK_TASK[idx] as *const u8 as u64 + ((param::STACK_SIZE-1) as u64);
    scheduler.ctx_task[idx].sp = stack_top;
    scheduler.ctx_task[idx].ra = f as u64;
    scheduler.task_cnt += 1;
}

pub fn task_go(i: usize) {
    let (ctx_os,ctx_new) = {
        let mut scheduler = SCHEDULER.lock();
        (&mut scheduler.ctx_os as *mut Context,
         &mut scheduler.ctx_task[i] as *mut Context)
    };
    unsafe {
        sys_switch(ctx_os, ctx_new);
    }
}

pub fn task_os() {
    let (ctx_os,ctx_now) = {
        let mut scheduler = SCHEDULER.lock();
        let i = scheduler.current_task;
        (&mut scheduler.ctx_os as *mut Context,
         &mut scheduler.ctx_task[i] as *mut Context)
    };
    unsafe {
        sys_switch(ctx_now, ctx_os);
    }
}

