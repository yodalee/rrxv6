use rv64::csr::stvec::Stvec;
use rv64::csr::sstatus::{Sstatus, Mode};
use rv64::csr::scause::Scause;
use rv64::csr::sepc::Sepc;
use crate::cpu::get_cpuid;
use crate::riscv::Interrupt;
use crate::uart::UART;
use crate::plic::{Plic, PlicContext};

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

pub fn interrupt_handler() {
    let plic = Plic::new();
    let scause = Scause::from_read();
    let hart = get_cpuid();

    if scause.is_interrupt() &&
        scause.get_code() == Interrupt::SupervisorExternal as u64 {
        let irq = plic.get_claim(hart, PlicContext::Supervisor);
        let mut uart = UART.lock();

        if irq != 0 {
            plic.set_complete(hart, PlicContext::Supervisor, irq);
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
