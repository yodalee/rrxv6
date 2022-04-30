
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

/// TrapFrame: per-process data for the trap handling code in trampoline.S.
///
/// The trapframe will be in its independent page. It will be placed under the
/// trampoline page in the user page table.
/// The sscratch register points to trapframe
/// uservec in trampoline.S saves user registers in the trapframe,
/// then initializes registers from the trapframe's
/// kernel_sp, kernel_hartid, kernel_satp, and jumps to kernel_trap.
///
/// usertrapret() and userret in trampoline.S will:
/// 1. Set up the kernel related registers in trapframe.
/// 2. Restore user registers from the trapframe
/// 3. Switch to the user page table
/// 4. Enter user space.
///
/// The trapframe includes callee-saved user registers like s0-s11 because the
/// return-to-user path via usertrapret() doesn't return through
/// the entire kernel call stack.
#[repr(C)]
#[derive(Debug,Default,Clone,Copy)]
pub struct TrapFrame {
  pub kernel_satp: Reg,   //   0 kernel page table
  pub kernel_sp: Reg,     //   8 top of process's kernel stack
  pub kernel_trap: Reg,   //  16 usertrap()
  pub epc: Reg,           //  24 saved user program counter
  pub kernel_hartid: Reg, //  32 saved kernel tp
  pub ra: Reg,            //  40
  pub sp: Reg,            //  48
  pub gp: Reg,            //  56
  pub tp: Reg,            //  64
  pub t0: Reg,            //  72
  pub t1: Reg,            //  80
  pub t2: Reg,            //  88
  pub s0: Reg,            //  96
  pub s1: Reg,            // 104
  pub a0: Reg,            // 112
  pub a1: Reg,            // 120
  pub a2: Reg,            // 128
  pub a3: Reg,            // 136
  pub a4: Reg,            // 144
  pub a5: Reg,            // 152
  pub a6: Reg,            // 160
  pub a7: Reg,            // 168
  pub s2: Reg,            // 176
  pub s3: Reg,            // 184
  pub s4: Reg,            // 192
  pub s5: Reg,            // 200
  pub s6: Reg,            // 208
  pub s7: Reg,            // 216
  pub s8: Reg,            // 224
  pub s9: Reg,            // 232
  pub s10: Reg,           // 240
  pub s11: Reg,           // 248
  pub t3: Reg,            // 256
  pub t4: Reg,            // 264
  pub t5: Reg,            // 272
  pub t6: Reg,            // 280
}

impl TrapFrame {
    pub const fn new() -> Self {
        Self {
            kernel_satp: 0,
            kernel_sp: 0,
            kernel_trap: 0,
            epc: 0,
            kernel_hartid: 0,
            ra: 0,
            sp: 0,
            gp: 0,
            tp: 0,
            t0: 0,
            t1: 0,
            t2: 0,
            s0: 0,
            s1: 0,
            a0: 0,
            a1: 0,
            a2: 0,
            a3: 0,
            a4: 0,
            a5: 0,
            a6: 0,
            a7: 0,
            s2: 0,
            s3: 0,
            s4: 0,
            s5: 0,
            s6: 0,
            s7: 0,
            s8: 0,
            s9: 0,
            s10: 0,
            s11: 0,
            t3: 0,
            t4: 0,
            t5: 0,
            t6: 0,
        }
    }

    pub fn reset(&mut self) {
        self.kernel_satp = 0;
        self.kernel_sp = 0;
        self.kernel_trap = 0;
        self.epc = 0;
        self.kernel_hartid = 0;
        self.ra = 0;
        self.sp = 0;
        self.gp = 0;
        self.tp = 0;
        self.t0 = 0;
        self.t1 = 0;
        self.t2 = 0;
        self.s0 = 0;
        self.s1 = 0;
        self.a0 = 0;
        self.a1 = 0;
        self.a2 = 0;
        self.a3 = 0;
        self.a4 = 0;
        self.a5 = 0;
        self.a6 = 0;
        self.a7 = 0;
        self.s2 = 0;
        self.s3 = 0;
        self.s4 = 0;
        self.s5 = 0;
        self.s6 = 0;
        self.s7 = 0;
        self.s8 = 0;
        self.s9 = 0;
        self.s10 = 0;
        self.s11 = 0;
        self.t3 = 0;
        self.t4 = 0;
        self.t5 = 0;
        self.t6 = 0;
    }
}
