use super::header::VirtioHeader;
use super::Error;

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
