use crate::memorylayout::VIRTIO0;
use crate::virtio::block::{BlockRequest, RequestType, VirtioBlock};
use crate::virtio::header::VirtioHeader;
use crate::virtio::queue::DescriptorFlag;

use core::mem::size_of;
use rv64::asm::sync_synchronize;
use spin::Mutex;

static mut DISK: Mutex<Option<VirtioBlock>> = Mutex::new(None);

pub fn init_disk() {
    let header = VirtioHeader::new(VIRTIO0).expect("Error: Disk header initialization");
    let block = VirtioBlock::new(header).expect("Error: Disk initialization");
    unsafe {
        let mut disk = DISK.lock();
        *disk = Some(block);
    }
}

pub fn read_disk() {
    let mut disk = unsafe { DISK.lock() };

    let request = BlockRequest {
        typ: RequestType::In,
        reserved: 0,
        sector: 0,
    };

    let block = disk.as_mut().unwrap();
    let mut descriptor = unsafe { block.queue.desc.as_mut() };
    let mut available = unsafe { block.queue.avail.as_mut() };

    // setup block request
    descriptor.addr = &request as *const _ as u64;
    descriptor.len = size_of::<BlockRequest>() as u32;
    descriptor.flags = DescriptorFlag::NEXT;
    descriptor.next = 0;

    available.idx += 1;

    sync_synchronize();

    block.header.set_queue_notify(0);
}
