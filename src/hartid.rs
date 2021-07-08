
//! Hart ID (mhartid) register

use crate::{csrw, csrr};

/// Hart ID Register
#[derive(Clone, Copy, Debug)]
pub struct Mhartid {
    bits: u64
}

impl Mhartid {
    #[inline]
    pub fn bits(self) -> u64 {
        self.bits
    }

    #[inline]
    pub fn read() -> Self {
        let bits: u64;
        csrr!("mhartid", bits);
        Self { bits }
    }
}
