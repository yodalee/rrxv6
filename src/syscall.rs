use crate::cpu::get_proc;
use crate::println;
use lazy_static::lazy_static;

const SYSCALL_NUM : usize = 1;
type SyscallEntry = fn() -> u64;
lazy_static! {
    static ref SYSCALLS: [SyscallEntry;SYSCALL_NUM] = [
        syscall_write
    ];
}

enum ArgIndex {
    A0, A1, A2, A3, A4, A5,
}

/// get u64 raw value store in trapframe->a0 to trapframe->a6
fn get_arg(n: ArgIndex) -> u64 {
    let proc = get_proc();
    let trapframe = unsafe { (*proc).trapframe.as_mut() };
    match n {
        ArgIndex::A0 => trapframe.a0,
        ArgIndex::A1 => trapframe.a1,
        ArgIndex::A2 => trapframe.a2,
        ArgIndex::A3 => trapframe.a3,
        ArgIndex::A4 => trapframe.a4,
        ArgIndex::A5 => trapframe.a5,
    }
}

fn syscall_write() -> u64 {
    let c = get_arg(ArgIndex::A1);
    let c = char::from_u32(c as u32).unwrap();
    println!("{}", c);
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
