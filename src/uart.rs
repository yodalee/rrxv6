use crate::console::CONSOLE;
use crate::memorylayout;
use crate::param::UART_TX_BUF_SIZE;

use bitflags::bitflags;
use lazy_static::lazy_static;
use spin::Mutex;
use volatile_register::RW;

lazy_static! {
    pub static ref UART: Mutex<Uart> = Mutex::new(Uart::new());
}

bitflags! {
    pub struct IerFlag: u8 {
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

pub struct Uart {
    tx_buf: [char; UART_TX_BUF_SIZE],
    write_idx: usize,
    read_idx: usize,
    p: &'static mut UartRegister,
}

impl Uart {
    fn new() -> Self {
        let mut uart = Uart {
            tx_buf: ['\0'; UART_TX_BUF_SIZE],
            write_idx: 0,
            read_idx: 0,
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
            self.p.isr.write(
                (FcrFlag::FIFO_ENABLE | FcrFlag::FIFO_CLEAR_RX | FcrFlag::FIFO_CLEAR_TX).bits(),
            );
        }

        // enable transmit and receive interruit
        self.set_interrupt(IerFlag::RX_ENABLE | IerFlag::TX_ENABLE);
    }

    pub fn putc(&mut self, c: char) {
        while (self.p.lsr.read() & 0x20) == 0 {}
        unsafe {
            self.p.thr.write(c as u8);
        }
    }

    pub fn puts(&mut self, s: &str) {
        for c in s.chars() {
            self.putc(c);
        }
    }

    /// Write one character from transmit buffer if UART is idle
    /// Return if there is no characters in buffer.
    pub fn put_bufferc(&mut self) {
        if self.write_idx == self.read_idx {
            // no character in buffer
            return;
        }
        if (self.p.lsr.read() & 0x20) == 0 {
            // the UART transmit holding register is full,
            // it will interrupt us if it's ready.
            return;
        }
        let c = self.tx_buf[self.read_idx];
        self.read_idx = (self.read_idx + 1) % UART_TX_BUF_SIZE;

        unsafe {
            self.p.thr.write(c as u8);
        }
    }

    fn readc(&mut self) -> Option<char> {
        if (self.p.lsr.read() & 0x01) != 0 {
            Some(self.p.thr.read() as char)
        } else {
            None
        }
    }

    pub fn set_interrupt(&mut self, flag: IerFlag) {
        unsafe {
            self.p.ier.write(flag.bits());
        }
    }

    /// Handle an uart interrupt
    /// can be RX interrupt or TX interrupt
    pub fn handle_interrupt(&mut self) {
        // read input character
        loop {
            match self.readc() {
                Some(c) => {
                    let mut console = CONSOLE.lock();
                    console.console_interrupt(c, self);
                }
                None => break,
            }
        }

        // write output character
        self.put_bufferc();
    }
}
