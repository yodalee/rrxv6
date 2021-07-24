use core::panic::PanicInfo;

use crate::{csrr, csrw};

use crate::param;
use crate::memorylayout;
use crate::riscv::register::mstatus;
use crate::riscv::register::mepc;
use crate::riscv::register::tp;
use crate::riscv::register::hartid;
use crate::riscv::register::delegate;
use crate::riscv::register::sie;
use crate::riscv::register::interrupt::Interrupt;
use crate::riscv::register::pmp::{PMPConfigMode,PMPConfigAddress,PMPAddress,PMPConfig};
use crate::riscv::register::mscratch;
use crate::riscv::register::mie;
use crate::riscv::register::mtvec;

#[no_mangle]
static STACK0: [u8;param::OS_STACK_SIZE * param::NCPU] = [0;param::OS_STACK_SIZE * param::NCPU];

#[no_mangle]
static mut TIMER_SCRATCH: [[u64;5];param::NCPU] = [[0u64;5];param::NCPU];

const INTERVAL : u64 = 1000000;

extern "C" {
    fn timervec();
}

// setup timer and timer interrupt
fn init_timer() {
    let hartid = hartid::Mhartid::read().bits();

    let mtimecmpaddr = memorylayout::CLINT_MTIMECMP + 8 * hartid;
    unsafe {
        let val = core::ptr::read_volatile(memorylayout::CLINT_MTIME as *mut u64);
        core::ptr::write_volatile(mtimecmpaddr as *mut u64, val + INTERVAL);
    }
    unsafe {
        let arr = &mut TIMER_SCRATCH[hartid as usize];
        arr[3] = mtimecmpaddr;
        arr[4] = INTERVAL;
        let mscratch = mscratch::Mscratch::from_bits(arr.as_ptr() as u64);
        mscratch.write();
    }

    // set the machine mode trap handler
    let mtvec = mtvec::Mtvec::from_bits(timervec as u64);
    mtvec.write();

    // Enable machine interrupt in mstatus
    let mut ms = mstatus::read();
    ms.enable_interrupt(mstatus::Mode::MachineMode);
    mstatus::write(ms);

    let mut mie = mie::Mie::read();
    mie.set_machine_enable(Interrupt::TimerInterrupt);
    mie.write();
}

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

    init_timer();

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
