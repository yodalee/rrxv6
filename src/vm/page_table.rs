//! riscv page table

use super::page_flag::PteFlag;

// 4096 bytes / 8 bytes per entry = 512 entries
const ENTRY_COUNT: usize = 512;

#[derive(Clone,Default)]
pub struct PageTableEntry {
    entry: u64
}

impl PageTableEntry {
    // Create empty page table entry
    #[inline]
    pub const fn new() -> Self {
        Self {
            entry: 0
        }
    }

    // true if page is zero (unused)
    #[inline]
    pub const fn is_unused(&self) -> bool {
        self.entry == 0
    }

    #[inline]
    pub fn set_unused(&mut self) {
        self.entry = 0;
    }

    #[inline]
    pub fn addr(&self) -> u64 {
        (self.entry >> 10) << 12
    }

    pub fn set_addr(&mut self, addr: u64, perm: PteFlag) {
        // TODO: check aligned here
        self.entry = addr | perm.bits();
    }
}

pub struct PageTable {
    entries: [PageTableEntry;ENTRY_COUNT]
}

impl PageTable {
    /// Create empty PageTable
    #[inline]
    pub const fn new() -> Self {
        const EMPTY: PageTableEntry = PageTableEntry::new();
        Self {
            entries: [EMPTY;ENTRY_COUNT]
        }
    }
}
