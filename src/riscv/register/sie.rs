
//! Supervisor Interrupt Register (sie)

use crate::{csrw, csrr};
use super::interrupt::Interrupt;

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
    pub fn set_supervisor_disable(&mut self, interrupt: Interrupt) {
        self.bits &= match interrupt {
            Interrupt::SoftwareInterrupt => !(1 << 1),
            Interrupt::TimerInterrupt =>    !(1 << 5),
            Interrupt::ExternalInterrupt => !(1 << 9),
        }
    }

    #[inline]
    pub fn set_supervisor_enable(&mut self, interrupt: Interrupt) {
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
    pub fn write(self) {
        let bits = self.bits();
        csrw!("sie", bits);
    }
}
