//! riscv page table

use core::ops::{Index, IndexMut};
use super::page_flag::PteFlag;

// 4096 bytes / 8 bytes per entry = 512 entries
const ENTRY_COUNT: usize = 512;

/// A 9-bits index for page table
pub struct PageTableIndex(u16);

impl PageTableIndex {
    /// Create a PageTableIndex from u16
    /// Will crash if the input > 512
    pub fn new(index: u16) -> Self {
        assert!((index as usize) < ENTRY_COUNT);
        Self (index)
    }

    /// Create a PageTableIndex from u16
    /// Truncate the input if > 512
    pub const fn new_truncate(index: u16) -> Self {
        Self(index % ENTRY_COUNT as u16)
    }
}

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

impl Index<usize> for PageTable {
    type Output = PageTableEntry;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.entries[index]
    }
}

impl IndexMut<usize> for PageTable {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.entries[index]
    }
}

impl Index<PageTableIndex> for PageTable {
    type Output = PageTableEntry;

    #[inline]
    fn index(&self, index: PageTableIndex) -> &Self::Output {
        &self.entries[usize::from(index.0)]
    }
}

impl IndexMut<PageTableIndex> for PageTable {
    #[inline]
    fn index_mut(&mut self, index: PageTableIndex) -> &mut Self::Output {
        &mut self.entries[usize::from(index.0)]
    }
}
