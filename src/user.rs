
//! User program

use crate::riscv::uart;
use crate::context::Context;
use crate::param;

pub static mut CTX_TASK: Context = Context {
    ra:  0,
    sp:  0,
    s0:  0,
    s1:  0,
    s2:  0,
    s3:  0,
    s4:  0,
    s5:  0,
    s6:  0,
    s7:  0,
    s8:  0,
    s9:  0,
    s10: 0,
    s11: 0,
};
pub static TASK0_STACK: [u8;param::STACK_SIZE] = [0;param::STACK_SIZE];

pub fn user_task0() {
    let m_uart = uart::read();
    m_uart.puts("Task0: Context Switch Success\n");

    loop {}
}
