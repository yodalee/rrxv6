
//! Machine Interrupt Register (mie)

use crate::{csrw, csrr};
use super::interrupt::Interrupt;

/// Machine Interrupt Enable Register (mie)
#[derive(Clone, Copy, Debug)]
pub struct Mie {
    bits: u64
}

impl Mie {
    #[inline]
    pub fn bits(self) -> u64 {
        self.bits
    }

    #[inline]
    pub fn set_machine_disable(&mut self, interrupt: Interrupt) {
        self.bits &= match interrupt {
            Interrupt::SoftwareInterrupt => !(1 << 3),
            Interrupt::TimerInterrupt =>    !(1 << 7),
            Interrupt::ExternalInterrupt => !(1 << 11),
        }
    }

    #[inline]
    pub fn set_machine_enable(&mut self, interrupt: Interrupt) {
        self.bits |= match interrupt {
            Interrupt::SoftwareInterrupt => (1 << 3),
            Interrupt::TimerInterrupt =>    (1 << 7),
            Interrupt::ExternalInterrupt => (1 << 11),
        }
    }

    #[inline]
    pub fn read() -> Self {
        let bits: u64;
        csrr!("mie", bits);
        Mie { bits }
    }

    #[inline]
    pub fn write(self) {
        let bits = self.bits();
        csrw!("mie", bits);
    }
}
