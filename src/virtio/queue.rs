use crate::kalloc::kalloc;

use super::header::VirtioHeader;
use super::Error;
use bitflags::bitflags;

use core::ptr::NonNull;

pub const MAX_QUEUE_SIZE: usize = 32768;

pub struct VirtioQueue {
    /// index of the queue
    idx: u32,

    /// size of the queue
    size: u16,

    /// address of descriptor
    desc: NonNull<Descriptor>,

    /// address of available ring
    avail: NonNull<AvailRing>,

    /// address of used ring
    used: NonNull<UsedRing>,
}

impl VirtioQueue {
    pub fn new(header: &mut VirtioHeader, idx: u32, size: u16) -> Result<Self, Error> {
        // ensure queue is not in used
        if header.queue_used(idx) {
            return Err(Error::AlreadyUsed);
        }

        // check maximum queue size
        let max = header.max_queue_size();
        if max == 0 {
            return Err(Error::NotAvailable);
        }
        if !size.is_power_of_two() || max < size as u32 {
            return Err(Error::InvalidArguments);
        }

        // allocate and zero queue memory.
        // note that kalloc will fill memory with 0 for us
        let desc = NonNull::new(kalloc() as *mut _).ok_or(Error::NoMemory)?;
        let avail = NonNull::new(kalloc() as *mut _).ok_or(Error::NoMemory)?;
        let used = NonNull::new(kalloc() as *mut _).ok_or(Error::NoMemory)?;

        // write physical address
        header.set_queue(
            idx,
            size,
            desc.addr().get() as u64,
            avail.addr().get() as u64,
            used.addr().get() as u64,
        );

        // set queue ready
        header.set_queue_ready(/*ready=*/ true);

        Ok(Self {
            idx,
            size,
            desc,
            avail,
            used,
        })
    }
}

struct Descriptor {
    /// Address
    addr: u64,
    /// Length
    len: u32,
    /// The flags
    flags: DescriptorFlag,
    /// Next field if flags & NEXT
    next: u16,
}

bitflags! {
    pub struct DescriptorFlag: u16 {
        /// Marks a buffer as continuing via the next field.
        const NEXT = 1;
        /// Marks a buffer as device write-only.
        const WRITE = 2;
        /// The buffer contains a list of buffer descriptors.
        const INDIRECT = 4;
    }
}

struct AvailRing {
    flags: AvailRingFlag,
    idx: u16,
    ring: [u16; 32],
    used_event: u16,
}

bitflags! {
    pub struct AvailRingFlag: u16 {
        const NO_INTERRUPT = 1;
    }
}

struct UsedRing {
    flags: UsedRingFlag,
    idx: u16,
    used_ring: [UsedElem; 32],
    avail_event: u16,
}

struct UsedElem {
    /// Index of start of used descriptor chain.
    id: u32,
    /// Total length of the descriptor chain which was used
    len: u32,
}

bitflags! {
    pub struct UsedRingFlag: u16 {
        const NO_NOTIFY = 1;
    }
}
