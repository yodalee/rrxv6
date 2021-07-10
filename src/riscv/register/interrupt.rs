
//! interrupt type, used in mie, mip, sie, sip, uie, uip

/// Type of interrupt
pub enum Interrupt {
    /// Software Interrupt
    SoftwareInterrupt,
    /// Timer Interrupt
    TimerInterrupt,
    /// External Interrupt
    ExternalInterrupt,
}
