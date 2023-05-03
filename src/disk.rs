use crate::virtio::header::VirtioHeader;
use crate::virtio::block::VirtioBlock;
use crate::memorylayout::VIRTIO0;

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
    let disk = unsafe { DISK.lock().as_mut().unwrap() };
}
