use rv64::register::tp;

// Must be called with interrupts disabled,
// to prevent race with process being moved
// to a different CPU.
pub fn get_cpuid() -> u64 {
    tp::read()
}
