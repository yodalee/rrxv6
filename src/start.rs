use core::panic::PanicInfo;

use crate::param;
use crate::memorylayout;
use crate::uart::UART;

use rv64::csr::interrupt::Interrupt;
use rv64::csr::medeleg::Medeleg;
use rv64::csr::mepc::Mepc;
use rv64::csr::mhartid::Mhartid;
use rv64::csr::mideleg::Mideleg;
use rv64::csr::mie::Mie;
use rv64::csr::mscratch::Mscratch;
use rv64::csr::mstatus;
use rv64::csr::mtvec::Mtvec;
use rv64::csr::pmp::{PMPConfigMode,PMPConfigAddress,PMPAddress,PMPConfig};
use rv64::csr::satp::Satp;
use rv64::csr::sie::Sie;
use rv64::register::tp;

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
    let mhartid = Mhartid::from_read().bits();

    let mtimecmpaddr = memorylayout::CLINT_MTIMECMP + 8 * mhartid;
    unsafe {
        let val = core::ptr::read_volatile(memorylayout::CLINT_MTIME as *mut u64);
        core::ptr::write_volatile(mtimecmpaddr as *mut u64, val + INTERVAL);
    }
    unsafe {
        let arr = &mut TIMER_SCRATCH[mhartid as usize];
        arr[3] = mtimecmpaddr;
        arr[4] = INTERVAL;
        Mscratch::from_bits(arr.as_ptr() as u64).write();
    }

    // set the machine mode trap handler
    let mtvec = Mtvec::from_bits(timervec as u64);
    mtvec.write();

    // Enable machine interrupt in mstatus
    let mut mstatus = mstatus::Mstatus::from_read();
    mstatus.enable_interrupt(mstatus::Mode::MachineMode);
    mstatus.write();

    let mut mie = Mie::from_read();
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
    let mut mstatus = mstatus::Mstatus::from_read();
    mstatus.set_mpp(mstatus::Mode::SupervisedMode);
    mstatus.write();

    // Setup M exception program counter for mret
    Mepc::from_bits(main as u64).write();

    // Disable paging for now
    Satp::from_bits(0).write();

    // Delegate all interrupts and exceptions to supervisor mode
    Medeleg::from_bits(0xffff).write();
    Mideleg::from_bits(0xffff).write();

    // Enable interrupt in supervisor mode
    let mut sie = Sie::from_read();
    sie.set_supervisor_enable(Interrupt::SoftwareInterrupt);
    sie.set_supervisor_enable(Interrupt::TimerInterrupt);
    sie.set_supervisor_enable(Interrupt::ExternalInterrupt);
    sie.write();

    // Store hart id in tp register, for cpuid()
    let hartid = Mhartid::from_read().bits();
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
fn panic(panic_info: &PanicInfo<'_>) -> ! {
    let mut m_uart = UART.lock();
    m_uart.puts(&format!("{}", panic_info));
    loop {}
}
