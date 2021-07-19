
//! Context of a process

use core::default::Default;

// FIXME this should put inside riscv library
type Reg = u64;

/// Saved Register for Context Switch
#[derive(Debug,Default,Clone,Copy)]
pub struct Context {
    /// return address and stack pointer
    pub ra:  Reg,
    pub sp:  Reg,

    /// Callee saved register
    pub s: [Reg;12],
}
