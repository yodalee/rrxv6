
//! Context of a process

use core::default::Default;

// FIXME this should put inside riscv library
type Reg = u64;

/// Saved Register for Context Switch
#[derive(Default)]
pub struct Context {
    /// return address and stack pointer
    pub ra:  Reg,
    pub sp:  Reg,

    /// Callee saved register
    pub s0:  Reg,
    pub s1:  Reg,
    pub s2:  Reg,
    pub s3:  Reg,
    pub s4:  Reg,
    pub s5:  Reg,
    pub s6:  Reg,
    pub s7:  Reg,
    pub s8:  Reg,
    pub s9:  Reg,
    pub s10: Reg,
    pub s11: Reg,
}
