use core::panic::PanicInfo;
use crate::uart::UART;

pub fn println(s: &str) {
    let mut m_uart = UART.lock();
    m_uart.puts(s);
    m_uart.putc('\n');
}

#[panic_handler]
fn panic(panic_info: &PanicInfo<'_>) -> ! {
    let mut m_uart = UART.lock();
    m_uart.puts(&format!("{}", panic_info));
    // Note that panic will hold the lock of UART
    // so no other process can access the UART.
    loop {}
}
