
//! User program

use crate::uart;
use crate::context::Context;
use crate::param;
use crate::util::delay;
use crate::scheduler::{os_kernel, task_create};

pub fn user_task0() {
    let m_uart = uart::read();
    m_uart.puts("Task0 Created\n");

    os_kernel();
    loop {
        m_uart.puts("Task0 Running\n");
        delay(1000);
        os_kernel();
    }
}

pub fn user_task1() {
    let m_uart = uart::read();
    m_uart.puts("Task1 Created\n");

    os_kernel();
    loop {
        m_uart.puts("Task1 Running\n");
        delay(1000);
        os_kernel();
    }
}

pub fn user_init() {
    task_create(user_task0);
    task_create(user_task1);
}
