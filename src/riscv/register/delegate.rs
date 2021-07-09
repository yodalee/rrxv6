
//! Machine Trap Delegation Register (medeleg and mideleg)

// TODO, give finer control with enum of interrupt and exception
// instead of wrapping value

/// medeleg
pub mod medeleg {
    use crate::{csrw, csrr};

    pub fn read() -> u64 {
        let bits: u64;
        csrr!("medeleg", bits);
        bits
    }

    pub fn write(bits: u64) {
        csrw!("medeleg", bits);
    }
}

/// mideleg
pub mod mideleg {
    use crate::{csrw, csrr};

    pub fn read() -> u64 {
        let bits: u64;
        csrr!("mideleg", bits);
        bits
    }

    pub fn write(bits: u64) {
        csrw!("mideleg", bits);
    }
}
