use rv64::csr::stvec::Stvec;
use rv64::csr::sstatus::{Sstatus, Mode};
use rv64::csr::scause::Scause;
use rv64::csr::sepc::Sepc;
use rv64::csr::sip::Sip;

use lazy_static::lazy_static;
use spin::Mutex;
use bit_field::BitField;

use crate::cpu::get_cpuid;
use crate::riscv::Interrupt;
use crate::uart::UART;
use crate::plic::{Plic, PlicContext};

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

fn handle_external_interrupt() {
    let plic = Plic::new();
    let hart = get_cpuid();
    let irq = plic.get_claim(hart, PlicContext::Supervisor);

    match irq {
        UART0_IRQ => {
            let mut uart = UART.lock();
            uart.handle_interrupt();
        }
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

fn interrupt_handler() {
    let scause = Scause::from_read();
    let code = scause.get_code();

    if scause.is_interrupt() {
        match code {
            x if x == Interrupt::SupervisorExternal as u64 => handle_external_interrupt(),
            x if x == Interrupt::SupervisorSoftware as u64 => handle_software_interrupt(),
            _ => panic!("Illegal interrupt code"),
        }
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
