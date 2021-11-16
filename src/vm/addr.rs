use bit_field::BitField;

use core::ops::{Add, AddAssign, Sub, SubAssign};

#[derive(Debug, Clone)]
pub struct InvalidVirtAddr;

#[derive(Clone,Copy,PartialEq,Eq,PartialOrd,Ord)]
pub struct VirtAddr(u64);

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
