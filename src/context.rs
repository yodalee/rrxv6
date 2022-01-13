
//! Context of a process

use core::default::Default;

type Reg = u64;

/// Saved Register for Context Switch
#[repr(C)]
#[derive(Debug,Default,Clone,Copy)]
pub struct Context {
    /// return address
    pub ra:  Reg,
    /// stack pointer
    pub sp:  Reg,

    /// Callee saved register
    pub s: [Reg;12],
}

impl Context {
    pub const fn new() -> Self {
        Self {
            ra: 0,
            sp: 0,
            s: [0;12]
        }
    }

    pub fn reset(&mut self) {
        self.ra = 0;
        self.sp = 0;
        self.s = [0;12];
    }
}
