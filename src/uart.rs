use volatile_register::{RW};

pub struct UART {
    thr: RW<u8>,
    ier: RW<u8>,
    isr: RW<u8>,
    lcr: RW<u8>,
    mcr: RW<u8>,
    lsr: RW<u8>,
    msr: RW<u8>,
    spr: RW<u8>,
}

impl UART {
    pub fn putc(&mut self, c: char) {
        while (self.lsr.read() & 0x40) == 0 {}
        unsafe {
            self.thr.write(c as u8);
        }
    }

    pub fn puts(&mut self, s: &str) {
        for c in s.chars() {
            self.putc(c);
        }
    }
}

pub fn read() -> &'static mut UART {
    unsafe { &mut *(0x1000_0000 as *mut UART) }
}
