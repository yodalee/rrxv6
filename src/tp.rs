#[inline]
pub fn read() -> u64 {
    unsafe {
        let x: u64;
        asm!("mv {}, tp", out(reg) x);
        x
    }
}

#[inline]
pub fn write(bits: u64) {
    unsafe {
        asm!("mv tp, {}", in(reg) bits);
    }
}
