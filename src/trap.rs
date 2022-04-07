use rv64::csr::stvec::Stvec;
use rv64::csr::sstatus::{Sstatus, Mode};
use rv64::csr::scause::Scause;
use rv64::csr::sepc::Sepc;
use rv64::csr::sip::Sip;

use lazy_static::lazy_static;
use spin::Mutex;

use crate::cpu::{get_cpu, get_cpuid, get_proc};
use crate::riscv::Interrupt;
use crate::uart::UART;
use crate::plic::{Plic, PlicContext};
use crate::memorylayout::UART0_IRQ;
use crate::scheduler::yield_proc;
use crate::proc::ProcState;

lazy_static! {
    static ref TICK: Mutex<u64> = Mutex::new(0);
}

extern "C" {
    fn kernelvec();
}

// setup to take exceptions and traps in supervisor mode
pub fn init_harttrap() {
    let mut stvec = Stvec::from_bits(0);
    stvec.set_addr(kernelvec as u64);
    stvec.write();
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
        },
        _ => {},
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

    let proc = get_proc();
    unsafe {
        if !proc.is_null() && (*proc).state == ProcState::RUNNING {
            yield_proc();
        }
    }
}

fn interrupt_handler() {
    let scause = Scause::from_read();
    let code = scause.get_code();

    if scause.is_interrupt() {
        match code {
            x if x == Interrupt::SupervisorExternal as u64 => handle_external_interrupt(),
            // software interrupt from machine-mode timer interrupt
            x if x == Interrupt::SupervisorSoftware as u64 => handle_software_interrupt(),
            _ => panic!("Illegal interrupt code"),
        }
    } else {
        panic!("exception");
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

    interrupt_handler();

    sepc.write();
    sstatus.write();
}
