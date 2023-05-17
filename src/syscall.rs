use crate::cpu::get_proc;
use crate::kvm::copy_in_str;
use crate::println;

use alloc::string::String;
use lazy_static::lazy_static;

const SYSCALL_NUM: usize = 1;
type SyscallEntry = fn() -> u64;
lazy_static! {
    static ref SYSCALLS: [SyscallEntry; SYSCALL_NUM] = [syscall_write];
}

#[allow(dead_code)]
enum ArgIndex {
    A0,
    A1,
    A2,
    A3,
    A4,
    A5,
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

/// Fetch the nul-terminated string at addr from the current process.
/// Returns length of string, not including nul, or -1 for error.
fn get_str(n: ArgIndex, buf: &mut [u8]) -> u64 {
    let addr = get_arg(n);
    let proc = get_proc();
    let page_table = unsafe { (*proc).pagetable.as_mut() };
    match copy_in_str(page_table, addr, buf) {
        None => u64::MAX,
        Some(len) => len,
    }
}

fn syscall_write() -> u64 {
    let len = get_arg(ArgIndex::A0);
    let mut buf = vec![0; len as usize];
    get_str(ArgIndex::A1, &mut buf);
    println!("{}", String::from_utf8_lossy(&buf));
    // FIXME the real write size
    len
}

pub fn syscall() {
    unsafe {
        let proc = get_proc();
        let trapframe = (*proc).trapframe.as_mut();
        let syscall_id: usize = trapframe.a7 as usize;

        trapframe.a0 = if syscall_id < SYSCALLS.len() {
            SYSCALLS[syscall_id]()
        } else {
            u64::MAX
        }
    }
}
