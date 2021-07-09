
//! Physical Memory Protection (PMP) 

use crate::{csrw, csrr};

pub enum PMPConfigAddress {
    Off,
    TOR,
    NA4,
    NAPOT,
}

pub enum PMPConfigMode {
    Read,
    Write,
    Exec,
    Address(PMPConfigAddress),
    // TODO Lock
}

/// pmpaddr register
#[derive(Clone, Copy, Debug)]
pub struct PMPAddress {
}

impl PMPAddress {
    #[inline]
    pub fn write(idx: usize, addr: u64) {
        match idx {
            0  => csrw!("pmpaddr0", addr),
            1  => csrw!("pmpaddr1", addr),
            2  => csrw!("pmpaddr2", addr),
            3  => csrw!("pmpaddr3", addr),
            4  => csrw!("pmpaddr4", addr),
            5  => csrw!("pmpaddr5", addr),
            6  => csrw!("pmpaddr6", addr),
            7  => csrw!("pmpaddr7", addr),
            8  => csrw!("pmpaddr8", addr),
            9  => csrw!("pmpaddr9", addr),
            10 => csrw!("pmpaddr10", addr),
            11 => csrw!("pmpaddr11", addr),
            12 => csrw!("pmpaddr12", addr),
            13 => csrw!("pmpaddr13", addr),
            14 => csrw!("pmpaddr14", addr),
            15 => csrw!("pmpaddr15", addr),
            _ => panic!(),
        }
    }
}

/// pmpcfg register
#[derive(Clone, Copy, Debug)]
pub struct PMPConfig {
    bits: u64
}

impl PMPConfig {
    #[inline]
    pub fn from_bits(bits: u64) -> Self {
        Self { bits }
    }

    #[inline]
    pub fn set_config(&mut self, mode: PMPConfigMode) {
        match mode {
            PMPConfigMode::Read  => self.bits |= 1 << 0,
            PMPConfigMode::Write => self.bits |= 1 << 1,
            PMPConfigMode::Exec  => self.bits |= 1 << 2,
            PMPConfigMode::Address(a) => {
                self.bits &= !(3 << 3);
                self.bits |= match a {
                    PMPConfigAddress::Off   => 0 << 3,
                    PMPConfigAddress::TOR   => 1 << 3,
                    PMPConfigAddress::NA4   => 2 << 3,
                    PMPConfigAddress::NAPOT => 3 << 3,
                }
            }
        }
    }

    #[inline]
    pub fn read() -> u64 {
        let x;
        csrr!("pmpcfg0", x);
        x
    }

    #[inline]
    pub fn write(config: PMPConfig) {
        let config = config.bits;
        csrw!("pmpcfg0", config);
    }
}
