
//! Supervisor Interrupt Register (sip)

/// Supervisor Interrupt Pending Register (sie)
#[derive(Clone, Copy, Debug)]
pub struct Sip {
    bits: u64
}
