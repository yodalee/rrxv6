use core::panic::PanicInfo;
use crate::trap::{push_off, pop_off};
use crate::uart::UART;

pub fn println(s: &str) {
    push_off();
    {
        let mut m_uart = UART.lock();
        m_uart.puts(s);
        m_uart.putc('\n');
    }
    pop_off();
}

#[panic_handler]
fn panic(panic_info: &PanicInfo<'_>) -> ! {
    let mut m_uart = UART.lock();
    m_uart.puts(&format!("{}", panic_info));
    // Note that panic will hold the lock of UART
    // so no other process can access the UART.
    loop {}
}
