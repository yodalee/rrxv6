
//! Supervisor Interrupt Register (sie)

use crate::{csrw, csrr};

/// Type of interrupt
pub enum Interrupt {
    /// Software Interrupt SSIE 1 << 1
    SoftwareInterrupt,
    /// Timer Interrupt STIE 1 << 5
    TimerInterrupt,
    /// External Interrupt STIE 1 << 9
    ExternalInterrupt,
}

/// Supervisor Interrupt Enable Register (sie)
#[derive(Clone, Copy, Debug)]
pub struct Sie {
    bits: u64
}

impl Sie {
    #[inline]
    pub fn bits(self) -> u64 {
        self.bits
    }

    #[inline]
    pub fn set_disable(&mut self, interrupt: Interrupt) {
        self.bits &= match interrupt {
            Interrupt::SoftwareInterrupt => !(1 << 1),
            Interrupt::TimerInterrupt =>    !(1 << 5),
            Interrupt::ExternalInterrupt => !(1 << 9),
        }
    }

    #[inline]
    pub fn set_enable(&mut self, interrupt: Interrupt) {
        self.bits |= match interrupt {
            Interrupt::SoftwareInterrupt => (1 << 1),
            Interrupt::TimerInterrupt =>    (1 << 5),
            Interrupt::ExternalInterrupt => (1 << 9),
        }

    }

    #[inline]
    pub fn read() -> Self {
        let bits: u64;
        csrr!("sie", bits);
        Sie { bits }
    }

    #[inline]
    pub fn write(sie: Self) {
        let bits = sie.bits();
        csrw!("sie", bits);
    }
}
