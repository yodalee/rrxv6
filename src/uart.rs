use volatile_register::RW;
use crate::memorylayout;
use bitflags::bitflags;
use lazy_static::lazy_static;
use spin::Mutex;

lazy_static! {
    pub static ref UART: Mutex<Uart> = Mutex::new(Uart::new());
}

bitflags! {
    struct IerFlag: u8 {
        const DISABLE = 0;
        const RX_ENABLE = 1 << 0;
        const TX_ENABLE = 1 << 1;
    }

    struct LcrFlag: u8 {
        const LENGTH_5 = 0;
        const LENGTH_6 = 1;
        const LENGTH_7 = 2;
        const LENGTH_8 = 3;
        const DLAB = 1 << 7;
    }

    struct FcrFlag: u8 {
        const FIFO_ENABLE = 1 << 0;
        const FIFO_CLEAR_RX = 1 << 1;
        const FIFO_CLEAR_TX = 1 << 2;
    }
}

pub struct Uart {
    p: &'static mut UartRegister
}

#[repr(C)]
struct UartRegister {
    thr: RW<u8>,
    ier: RW<u8>,
    isr: RW<u8>,
    lcr: RW<u8>,
    mcr: RW<u8>,
    lsr: RW<u8>,
    msr: RW<u8>,
    spr: RW<u8>,
}

impl Uart {
    fn new() -> Self {
        let mut uart = Uart {
            p: unsafe { &mut *(memorylayout::UART0 as *mut UartRegister) },
        };
        uart.init();
        uart
    }

    /// Do uart initialization
    fn init(&mut self) {
        // disable interrupt
        self.set_interrupt(IerFlag::DISABLE);

        unsafe {
            // special mode to set baud rate
            self.p.lcr.write(LcrFlag::DLAB.bits());

            // set baud rate of 38.4K
            self.p.thr.write(0x03);
            self.p.ier.write(0x0);

            // set word length to 8 bits, no parity
            self.p.lcr.write(LcrFlag::LENGTH_8.bits());

            // reset and enable FIFOs
            self.p.isr.write((FcrFlag::FIFO_ENABLE | FcrFlag::FIFO_CLEAR_RX | FcrFlag::FIFO_CLEAR_TX).bits());
        }

        // enable transmit and receive interruit
        self.set_interrupt(IerFlag::RX_ENABLE | IerFlag::TX_ENABLE);
    }

    pub fn putc(&mut self, c: char) {
        while (self.p.lsr.read() & 0x40) == 0 {}
        unsafe {
            self.p.thr.write(c as u8);
        }
    }

    pub fn puts(&mut self, s: &str) {
        for c in s.chars() {
            self.putc(c);
        }
    }

    pub fn set_interrupt(&mut self, flag: IerFlag) {
        unsafe {
            self.p.ier.write(flag.bits());
        }
    }
}
