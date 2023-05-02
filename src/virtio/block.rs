use super::header::VirtioHeader;
use super::queue::VirtioQueue;
use super::Error;

use bitflags::bitflags;

/// VirtioBlock
///
/// The virtio block device is a simple virtual block device (ie. disk).
/// Read and write requests (and other exotic requests) are placed in the queue, and serviced
/// (probably out of order) by the device except where noted.
///
/// ref: 5.2 Block Device
pub struct VirtioBlock {
    pub header: &'static mut VirtioHeader,
    // FIXME: expose method instead of public access
    pub queue: VirtioQueue,
}

impl VirtioBlock {
    pub fn new(header: &'static mut VirtioHeader) -> Result<Self, Error> {
        header.begin_init(|features| {
            let features = BlockFeatures::from_bits_truncate(features);
            let disable_features = BlockFeatures::RO
                | BlockFeatures::CONFIG_WCE
                | BlockFeatures::RING_EVENT_IDX
                | BlockFeatures::RING_INDIRECT_DESC;
            (features - disable_features).bits()
        })?;

        let queue = VirtioQueue::new(header, 0, 8)?;
        header.end_init();

        Ok(Self { header, queue })
    }
}

bitflags! {
    /// Block devices features bit
    /// ref: 5.2.3 Feature bits
    pub struct BlockFeatures: u64 {
        /// Maximum size of any single segment is in size_max.
        const SIZE_MAX = 1 << 1;
        /// Maximum number of segments in a request is in seg_max.
        const SEG_MAX = 1 << 2;
        /// Disk-style geometry specified in geometry.
        const GEOMETRY = 1 << 4;
        /// Device is read-only.
        const RO = 1 << 5;
        /// Block size of disk is in blk_size.
        const BLK_SIZE = 1 << 6;
        /// Cache flush command support.
        const FLUSH = 1 << 9;
        /// Device exports information on optimal I/O alignment.
        const TOPOLOGY = 1 << 10;
        /// Device can toggle its cache between writeback and writethrough modes.
        const CONFIG_WCE = 1 << 11;
        /// Device can support discard command, maximum discard sectors size in max_discard_sectors and maximum discard segment number in max_discard_seg.
        const DISCARD = 1 << 13;
        /// Device can support write zeroes command, maximum write zeroes sectors size in max_write_zeroes_sectors and maximum write zeroes segment number in max_write_zeroes_seg.
        const WRITE_ZEROES = 1 << 14;

        // device independent
        /// Negotiating this feature indicates that the driver can use descriptors with the VIRTQ_DESC_F_INDIRECT flag set,
        /// as described in 2.6.5.3 Indirect Descriptors and 2.7.7 Indirect Flag: Scatter-Gather Support.
        const RING_INDIRECT_DESC = 1 << 28;
        /// This feature enables the used_event and the avail_event fields as described in 2.6.7, 2.6.8 and 2.7.10.
        const RING_EVENT_IDX = 1 << 29;
        /// This indicates compliance with this specification, giving a simple way to detect legacy devices or drivers.
        const VERSION_1 = 1 << 32;
        /// This feature indicates that the device can be used on a platform where device access to data in memory is limited and/or translated.
        const ACCESS_PLATFORM = 1 << 33;
        /// This feature indicates support for the packed virtqueue layout as described in 2.7 Packed Virtqueues.
        const RING_PACKED = 1 << 34;
        /// This feature indicates that all buffers are used by the device in the same order in which they have been made available.
        const IN_ORDER = 1 << 35;
        /// This feature indicates that memory accesses by the driver and the device are ordered in a way described by the platform.
        const ORDER_PLATFORM = 1 << 36;
        /// This feature indicates that the device supports Single Root I/O Virtualization. Currently only PCI devices support this feature.
        const SR_IOV = 1 << 37;
        /// This feature indicates that the driver passes extra data (besides identifying the virtqueue) in its device notifications.
        const NOTIFICATION_DATA = 1 << 38;
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct BlockRequest {
    pub typ: RequestType,
    pub reserved: u32,
    pub sector: u64,
}

#[repr(u32)]
#[derive(Debug)]
pub enum RequestType {
    In = 0,
    Out = 1,
}
