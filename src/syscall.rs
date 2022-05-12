use crate::cpu::get_proc;
use lazy_static::lazy_static;

const SYSCALL_NUM : usize = 1;
type SyscallEntry = fn() -> u64;
lazy_static! {
    static ref SYSCALLS: [SyscallEntry;SYSCALL_NUM] = [
        syscall_write
    ];
}

fn syscall_write() -> u64 {
    0
}

pub fn syscall() {
    unsafe {
        let proc = get_proc();
        let trapframe = (*proc).trapframe.as_mut();
        let syscall_id : usize = trapframe.a7 as usize;

        trapframe.a0 = if syscall_id < SYSCALLS.len() {
            SYSCALLS[syscall_id]()
        } else {
            u64::MAX
        }
    }
}
