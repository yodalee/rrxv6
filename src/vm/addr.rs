use bit_field::BitField;
use super::page_table::{PageTableIndex, PageTableLevel};

use core::ops::{Add, AddAssign, Sub, SubAssign};

#[derive(Debug, Clone)]
pub struct InvalidVirtAddr;

#[derive(Clone,Copy,PartialEq,Eq,PartialOrd,Ord)]
pub struct VirtAddr(u64);

/// A 64-bits physical memory address.
///
/// A wrapper type around `u64`
/// On riscv, only lower 56 bits can be used, top 8 bits must be zeroed.
#[derive(Clone,Copy,PartialEq,Eq,PartialOrd,Ord)]
pub struct PhysAddr(u64);

impl VirtAddr {
    #[inline]
    pub fn new(addr: u64) -> Self {
        Self::try_new(addr).expect(&format!("Virtual address in riscv should have bit 39-63 copied from bit 38 {}", addr))
    }

    /// Try to create a new virtual address.
    #[inline]
    pub fn try_new(addr: u64) -> Result<VirtAddr, InvalidVirtAddr> {
        match addr.get_bits(38..64) {
            0 | 0x3ffffff => Ok(VirtAddr(addr)),   // valid address
            1 => Ok(VirtAddr::new_truncate(addr)), // address need sign extend
            _ => Err(InvalidVirtAddr{}),
        }
    }

    /// Create a VirtAddr with signed extension
    #[inline]
    pub fn new_truncate(addr: u64) -> Self {
        Self(((addr << 25) as i64 >> 25) as u64)
    }

    #[inline]
    pub fn as_u64(self) -> u64 {
        self.0
    }

    #[inline]
    pub fn is_align(self) -> bool {
        self.0 & (4096 - 1) == 0
    }

    #[inline]
    pub fn align_down(self) -> Self {
        Self(
            align_down(self.0, 4096)
        )
    }

    #[inline]
    pub fn align_up(self) -> Self {
        Self(
            align_up(self.0, 4096)
        )
    }

    /// Return the 9 bits level 0 page table index from offset [12,20]
    #[inline]
    pub const fn p0_index(self) -> PageTableIndex {
        PageTableIndex::new_truncate((self.0 >> 12) as u16)
    }

    /// Return the 9 bits level 1 page table index from offset [21,29]
    #[inline]
    pub const fn p1_index(self) -> PageTableIndex {
        PageTableIndex::new_truncate((self.0 >> 9 >> 12) as u16)
    }

    /// Return the 9 bits level 2 page table index from offset [30,38]
    #[inline]
    pub const fn p2_index(self) -> PageTableIndex {
        PageTableIndex::new_truncate((self.0 >> 9 >> 9 >> 12) as u16)
    }

    /// Return the 9 bits level 3 page table index from offset [39,47]
    /// Only valid with sv47 mode
    #[inline]
    pub const fn p3_index(self) -> PageTableIndex {
        PageTableIndex::new_truncate((self.0 >> 9 >> 9 >> 9 >> 12) as u16)
    }

    /// Return the 9 bits page table index according to level
    #[inline]
    pub const fn get_index(self, level: PageTableLevel) -> PageTableIndex {
        match level {
            PageTableLevel::Zero  => self.p0_index(),
            PageTableLevel::One   => self.p1_index(),
            PageTableLevel::Two   => self.p2_index(),
            PageTableLevel::Three => self.p3_index(),
        }
    }
}

impl Add<u64> for VirtAddr {
    type Output = Self;

    #[inline]
    fn add(self, rhs: u64) -> Self::Output {
        VirtAddr::new(self.0 + rhs)
    }
}

impl AddAssign<u64> for VirtAddr {
    #[inline]
    fn add_assign(&mut self, rhs: u64) {
        *self = *self + rhs;
    }
}

impl Add<usize> for VirtAddr {
    type Output = Self;

    #[inline]
    fn add(self, rhs: usize) -> Self::Output {
        self + rhs as u64
    }
}

impl AddAssign<usize> for VirtAddr {
    #[inline]
    fn add_assign(&mut self, rhs: usize) {
        self.add_assign(rhs as u64)
    }
}

impl Sub<u64> for VirtAddr {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: u64) -> Self::Output {
        VirtAddr::new(self.0 - rhs)
    }
}

impl SubAssign<u64> for VirtAddr {
    #[inline]
    fn sub_assign(&mut self, rhs: u64) {
        *self = *self - rhs;
    }
}

impl Sub<usize> for VirtAddr {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: usize) -> Self::Output {
        self - rhs as u64
    }
}

impl SubAssign<usize> for VirtAddr {
    #[inline]
    fn sub_assign(&mut self, rhs: usize) {
        self.sub_assign(rhs as u64);
    }
}

#[derive(Debug)]
pub struct InvalidPhysAddr;

impl PhysAddr {
    #[inline]
    pub fn new(addr: u64) -> Self {
        Self::try_new(addr).expect(&format!("Physical address in riscv should have bit 56-63 zeroed {}", addr))
    }

    /// Try to create a new physical address.
    #[inline]
    pub fn try_new(addr: u64) -> Result<PhysAddr, InvalidPhysAddr> {
        match addr.get_bits(56..64) {
            0 => Ok(PhysAddr(addr)),   // valid address
            _ => Err(InvalidPhysAddr{}),
        }
    }

    /// Create a PhysAddr with zeroed bit 56-64
    #[inline]
    pub fn new_truncate(addr: u64) -> Self {
        Self(addr & ((1 << 56) - 1))
    }

    #[inline]
    pub fn as_u64(self) -> u64 {
        self.0
    }

    #[inline]
    pub fn as_pte(self) -> u64 {
        (self.0 >> 12) << 10
    }

    #[inline]
    pub fn align_down(self) -> Self {
        Self(
            align_down(self.0, 4096)
        )
    }

    #[inline]
    pub fn align_up(self) -> Self {
        Self(
            align_up(self.0, 4096)
        )
    }
}

impl Add<u64> for PhysAddr {
    type Output = Self;

    #[inline]
    fn add(self, rhs: u64) -> Self::Output {
        PhysAddr::new(self.0 + rhs)
    }
}

impl AddAssign<u64> for PhysAddr {
    #[inline]
    fn add_assign(&mut self, rhs: u64) {
        *self = *self + rhs;
    }
}

impl Add<usize> for PhysAddr {
    type Output = Self;

    #[inline]
    fn add(self, rhs: usize) -> Self::Output {
        self + rhs as u64
    }
}

impl AddAssign<usize> for PhysAddr {
    #[inline]
    fn add_assign(&mut self, rhs: usize) {
        self.add_assign(rhs as u64)
    }
}

impl Sub<u64> for PhysAddr {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: u64) -> Self::Output {
        PhysAddr::new(self.0 - rhs)
    }
}

impl SubAssign<u64> for PhysAddr {
    #[inline]
    fn sub_assign(&mut self, rhs: u64) {
        *self = *self - rhs;
    }
}

impl Sub<usize> for PhysAddr {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: usize) -> Self::Output {
        self - rhs as u64
    }
}

impl SubAssign<usize> for PhysAddr {
    #[inline]
    fn sub_assign(&mut self, rhs: usize) {
        self.sub_assign(rhs as u64);
    }
}

#[inline]
pub const fn align_down(addr: u64, align: u64) -> u64 {
    assert!(align.is_power_of_two());
    addr & !(align -1)
}

#[inline]
pub const fn align_up(addr: u64, align: u64) -> u64 {
    assert!(align.is_power_of_two());
    align_down(addr + align - 1, align)
}
