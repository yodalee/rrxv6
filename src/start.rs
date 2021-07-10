use core::panic::PanicInfo;

use crate::{csrr, csrw};

use crate::param;
use crate::riscv::register::mstatus;
use crate::riscv::register::mepc;
use crate::riscv::register::tp;
use crate::riscv::register::hartid;
use crate::riscv::register::delegate;
use crate::riscv::register::sie;
use crate::riscv::register::interrupt::Interrupt;
use crate::riscv::register::pmp::{PMPConfigMode,PMPConfigAddress,PMPAddress,PMPConfig};

#[no_mangle]
static STACK0: [u8;param::STACK_SIZE * param::NCPU] = [0;param::STACK_SIZE * param::NCPU];

#[no_mangle]
fn start() -> ! {
    extern "Rust" {
        fn main() -> !;
    }

    /* Set M Previous Privilege mode to SupervisedMode
     * so mret will switch to supervise mode
     */
    let mut ms = mstatus::read();
    ms.set_mpp(mstatus::Mode::SupervisedMode);
    mstatus::write(ms);

    // Setup M exception program counter for mret
    let m_mepc = mepc::Mepc::from_bits(main as u64);
    mepc::write(m_mepc);

    // Disable paging for now
    let x = 0;
    csrw!("satp", x);

    // Delegate all interrupts and exceptions to supervisor mode
    delegate::medeleg::write(0xffff);
    delegate::mideleg::write(0xffff);

    // Enable interrupt in supervisor mode
    let mut sie = sie::Sie::read();
    sie.set_supervisor_enable(Interrupt::SoftwareInterrupt);
    sie.set_supervisor_enable(Interrupt::TimerInterrupt);
    sie.set_supervisor_enable(Interrupt::ExternalInterrupt);
    sie.write();

    // Store hart id in tp register, for cpuid()
    let hartid = hartid::Mhartid::read().bits();
    tp::write(hartid);

    // Setup PMP so that supervisor mode can access memory
    PMPAddress::write(0, (!(0)) >> 10);

    let mut config = PMPConfig::from_bits(0);
    config.set_config(PMPConfigMode::Read);
    config.set_config(PMPConfigMode::Write);
    config.set_config(PMPConfigMode::Exec);
    config.set_config(PMPConfigMode::Address(PMPConfigAddress::TOR));
    PMPConfig::write(config);

    // Switch to supervisor mode and jump to main
    unsafe { asm!("mret"); }

    // mret will jump into kernel, should not execute to here
    loop {}
}

#[panic_handler]
fn panic(_panic: &PanicInfo<'_>) -> ! {
    loop {}
}
