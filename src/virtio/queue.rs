use super::header::VirtioHeader;
use super::Error;
use bitflags::bitflags;

pub const MAX_QUEUE_SIZE: usize = 32768;

pub struct VirtioQueue {
    /// index of the queue
    idx: u32,

    /// size of the queue
    size: u16,
}

impl VirtioQueue {
    pub fn new(header: &mut VirtioHeader, idx: u32, size: u16) -> Result<Self, Error> {
        if header.queue_used(idx) {
            return Err(Error::AlreadyUsed);
        }
        let max = header.max_queue_size();
        if max == 0 {
            return Err(Error::NotAvailable);
        }
        if !size.is_power_of_two() || max < size as u32 {
            return Err(Error::InvalidArguments);
        }

        header.set_queue(idx, size);

        Ok(Self { idx, size })
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
