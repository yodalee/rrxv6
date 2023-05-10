use rv64::csr::satp::{Satp, SatpMode};
use rv64::csr::scause::Scause;
use rv64::csr::sepc::Sepc;
use rv64::csr::sip::Sip;
use rv64::csr::sstatus::{Mode, Sstatus};
use rv64::csr::stval::Stval;
use rv64::csr::stvec::Stvec;
use rv64::register::tp;

use alloc::boxed::Box;
use lazy_static::lazy_static;
use spin::Mutex;

use crate::cpu::{get_cpu, get_cpuid, get_proc};
use crate::memorylayout::{TRAMPOLINE, TRAPFRAME, UART0_IRQ, VIRTIO0_IRQ};
use crate::plic::{Plic, PlicContext};
use crate::println;
use crate::proc::{Proc, ProcState};
use crate::riscv::{Exception, Interrupt, PAGESIZE};
use crate::scheduler::yield_proc;
use crate::syscall::syscall;
use crate::uart::UART;

lazy_static! {
    static ref TICK: Mutex<u64> = Mutex::new(0);
}

extern "C" {
    fn kernelvec();
    fn uservec();
    fn userret();
    fn trampoline();
}

// setup to take exceptions and traps in supervisor mode
pub fn init_harttrap() {
    Stvec::from_bits(kernelvec as u64).write();
}

/// enable device interrupt
pub fn intr_on() {
    let mut sstatus = Sstatus::from_read();
    sstatus.enable_interrupt(Mode::SupervisedMode);
    sstatus.write();
}

/// disable device interrupt
pub fn intr_off() {
    let mut sstatus = Sstatus::from_read();
    sstatus.disable_interrupt(Mode::SupervisedMode);
    sstatus.write();
}

/// get device interrupt status
pub fn intr_get() -> bool {
    let sstatus = Sstatus::from_read();
    sstatus.get_sie()
}

/// push_off, like intr_off but required same number of pop_off to make interrupt on.
pub fn push_off() {
    let intr = intr_get();
    let cpu = get_cpu();
    let mut push_count = cpu.push_count.lock();
    intr_off();
    if *push_count == 0 {
        let mut base = cpu.interrupt_base.lock();
        *base = intr;
    }
    *push_count = *push_count + 1;
}

/// pop_off, cancel one push_off
/// Calling pop_off without push_off will panic
pub fn pop_off() {
    let cpu = get_cpu();
    let intr = intr_get();
    let mut push_count = cpu.push_count.lock();
    let interrupt_base = cpu.interrupt_base.lock();
    if intr {
        panic!("pop_off: interruptible");
    }
    if *push_count < 1 {
        panic!("pop_off: stack empty");
    }
    *push_count = *push_count - 1;
    if *push_count == 0 && *interrupt_base {
        intr_on();
    }
}

fn handle_external_interrupt() {
    let plic = Plic::new();
    let hart = get_cpuid();
    let irq = plic.get_claim(hart, PlicContext::Supervisor);

    match irq as u64 {
        UART0_IRQ => {
            let mut uart = UART.lock();
            uart.handle_interrupt();
        }
        VIRTIO0_IRQ => {
            panic!("VIRTIO IRQ");
        }
        _ => {}
    }

    if irq != 0 {
        plic.set_complete(hart, PlicContext::Supervisor, irq);
    }
}

fn tick() {
    let mut tick = TICK.lock();
    *tick += 1;
}

fn handle_software_interrupt() {
    if get_cpuid() == 0 {
        tick();
    }

    let mut sip = Sip::from_read();
    sip.clear_pending(1);
    sip.write();
}

/// handle interrupt and return the interrupt code
fn interrupt_handler() -> Option<u64> {
    let scause = Scause::from_read();
    let code = scause.get_code();

    if scause.is_interrupt() {
        match code {
            x if x == Interrupt::SupervisorExternal as u64 => handle_external_interrupt(),
            // software interrupt from machine-mode timer interrupt
            x if x == Interrupt::SupervisorSoftware as u64 => handle_software_interrupt(),
            _ => return None,
        }
        Some(code)
    } else {
        None
    }
}

/// interrupts and exceptions from kernel code go here via kernelvec,
/// on whatever the current kernel stack is.
#[no_mangle]
pub fn kerneltrap() {
    let sepc = Sepc::from_read();
    let sstatus = Sstatus::from_read();

    if sstatus.get_spp() != Mode::SupervisedMode {
        panic!("kerneltrap: not from supervised mode");
    }
    if sstatus.get_sie() {
        panic!("kerneltrap: interrupts enabled");
    }

    match interrupt_handler() {
        Some(x) if x == Interrupt::SupervisorSoftware as u64 => {
            let proc = get_proc();
            unsafe {
                if !proc.is_null() && (*proc).state == ProcState::RUNNING {
                    yield_proc();
                }
            }
        }
        None => {
            let scause = Scause::from_read().bits();
            let sepc = Sepc::from_read().bits();
            let stval = Stval::from_read().bits();
            println!("scause {:x} sepc={:x} stval={:x}", scause, sepc, stval);
            panic!("kerneltrap");
        }
        _ => (),
    }

    sepc.write();
    sstatus.write();
}

pub unsafe fn usertrapret() {
    let proc = get_proc() as *mut Box<Proc>;

    // We're about to switch the destination of traps from kerneltrap() to usertrap()
    // turn off interrupts until we're back in user space, where usertrap() is correct.
    intr_off();

    // send syscalls, interrupts, and exceptions to trampoline.S
    let mut stvec = Stvec::from_bits(0);
    stvec.set_addr(TRAMPOLINE + (uservec as u64 - trampoline as u64));
    stvec.write();

    // set up trapframe values that uservec will need when
    // the process next re-enters the kernel.
    let trapframe = (*proc).trapframe.as_mut();
    let satp = Satp::from_read();
    trapframe.kernel_satp = satp.bits(); // kernel page table
    trapframe.kernel_sp = (*proc).kstack + PAGESIZE; // process's kernel stack
    trapframe.kernel_trap = usertrap as u64;
    trapframe.kernel_hartid = tp::read(); // hartid for cpuid()

    // set up the registers that trampoline.S's sret will use
    // to get to user space.

    // set S Previous Privilege mode to User.
    let mut sstatus = Sstatus::from_read();
    sstatus.set_spp(Mode::UserMode); // clear SPP to 0 for user mode
    sstatus.set_spie(true);
    sstatus.write();

    // set S Exception Program Counter to the saved user pc.
    Sepc::from_bits(trapframe.epc).write();

    // tell trampoline.S the user page table to switch to.
    let mut satp = Satp::from_bits(0);
    let pagetable = (*proc).pagetable.as_mut();
    satp.set_mode(SatpMode::ModeSv39);
    satp.set_addr(pagetable as *const _ as u64);
    let satp = satp.bits();

    // jump to trampoline.S at the top of memory, which
    // switches to the user page table, restores user registers,
    // and switches to user mode with sret.
    let fp = (TRAMPOLINE + (userret as u64 - trampoline as u64)) as *const ();
    let code: fn(u64, u64) = core::mem::transmute(fp);
    code(TRAPFRAME, satp)
}

/// handle an interrupt, exception, or system call from user space.
/// called from trampoline.S, must not mangle its name
#[no_mangle]
pub fn usertrap() {
    let sstatus = Sstatus::from_read();
    if sstatus.get_spp() != Mode::UserMode {
        panic!("usertrap: not from user mode");
    }

    // send interrupts and exceptions to kerneltrap(),
    // since we're now in the kernel.
    Stvec::from_bits(kernelvec as u64).write();

    let proc: *mut Box<Proc> = get_proc();
    let trapframe = unsafe { (*proc).trapframe.as_mut() };

    // save user program counter.
    trapframe.epc = Sepc::from_read().bits();

    let scause = Scause::from_read();
    let code = scause.get_code();
    if scause.is_interrupt() {
        match interrupt_handler() {
            Some(x) if x == Interrupt::SupervisorSoftware as u64 => {
                yield_proc();
            }
            None => {
                // TODO just kill process, don't panic
                let pid = unsafe { (*proc).pid };
                let sepc = Sepc::from_read().bits();
                let stval = Stval::from_read().bits();
                println!("usertrap(): unexpected scause {:x}", scause.bits());
                println!("    pid = {:x} sepc={:x} stval={:x}", pid, sepc, stval);
                panic!("usertrap")
            }
            _ => (),
        }
    } else {
        match code {
            x if x == Exception::EnvironmentCallUMode as u64 => {
                // system call
                // TODO: check process is killed

                // sepc points to the ecall instruction,
                // but we want to return to the next instruction.
                trapframe.epc += 4;

                // an interrupt will change sstatus &c registers,
                // so don't enable until done with those registers.
                intr_on();

                syscall();
            }
            _ => {
                panic!("usertrap: unexpected exception");
            }
        }
    }

    unsafe {
        usertrapret();
    }
}
