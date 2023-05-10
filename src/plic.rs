//! the riscv Platform Level Interrupt Controller (PLIC).

use crate::cpu::get_cpuid;
use crate::memorylayout::{
    PLIC_CLAIM, PLIC_ENABLE, PLIC_PRIORITY, PLIC_THRESHOLD, UART0_IRQ, VIRTIO0_IRQ,
};
use crate::riscv::MAX_INTERRUPT;

pub enum PlicContext {
    Machine = 0,
    Supervisor = 1,
}

pub struct Plic {}

impl Plic {
    pub fn new() -> Self {
        Plic {}
    }

    /// set id interrupt priority, zero is disabled
    pub fn set_priority(&self, id: u64, priority: u32) {
        let addr = (PLIC_PRIORITY + 4 * id) as *mut u32;
        unsafe {
            core::ptr::write_volatile(addr, priority);
        }
    }

    /// Set interrupt enable
    pub fn set_enable(&self, hart: u64, context: PlicContext, id: u64) {
        assert!(id < MAX_INTERRUPT);
        let addr = (PLIC_ENABLE + hart * 0x100 + (context as u64) * 0x80 + (id / 32)) as *mut u32;
        unsafe {
            let val = core::ptr::read_volatile(addr);
            core::ptr::write_volatile(addr, val | (1u32 << (id % 32)));
        }
    }

    /// Set interrupt enable
    pub fn set_disable(&self, hart: u64, context: PlicContext, id: u64) {
        assert!(id < MAX_INTERRUPT);
        let addr = (PLIC_ENABLE + hart * 0x100 + (context as u64) * 0x80 + (id / 32)) as *mut u32;
        unsafe {
            let val = core::ptr::read_volatile(addr);
            core::ptr::write_volatile(addr, val & !(1u32 << (id % 32)));
        }
    }

    /// Set threshold of interrupt of (hart, context)
    pub fn set_threshold(&self, hart: u64, context: PlicContext, threshold: u32) {
        let addr = (PLIC_THRESHOLD + hart * 0x2000 + (context as u64) * 0x1000) as *mut u32;
        unsafe {
            core::ptr::write_volatile(addr, threshold);
        }
    }

    /// Get PLIC current interupt id
    pub fn get_claim(&self, hart: u64, context: PlicContext) -> u32 {
        let addr = (PLIC_CLAIM + hart * 0x2000 + (context as u64) * 0x1000) as *mut u32;
        unsafe { core::ptr::read_volatile(addr) }
    }

    /// Mark irq complete
    pub fn set_complete(&self, hart: u64, context: PlicContext, id: u32) {
        assert!((id as u64) < MAX_INTERRUPT);
        let addr = (PLIC_CLAIM + hart * 0x2000 + (context as u64) * 0x1000) as *mut u32;
        unsafe {
            core::ptr::write_volatile(addr, id);
        }
    }
}

pub fn init_plic() {
    let plic = Plic::new();
    plic.set_priority(UART0_IRQ, 1);
    plic.set_priority(VIRTIO0_IRQ, 1);
}

pub fn init_hartplic() {
    let hart = get_cpuid();
    let plic = Plic::new();
    plic.set_enable(hart, PlicContext::Supervisor, UART0_IRQ);
    plic.set_enable(hart, PlicContext::Supervisor, VIRTIO0_IRQ);
    plic.set_threshold(hart, PlicContext::Supervisor, 0);
}
