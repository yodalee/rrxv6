use crate::virtio::header::VirtioHeader;
use crate::virtio::block::VirtioBlock;
use crate::memorylayout::VIRTIO0;

use spin::Mutex;

static mut DISK: Mutex<Option<VirtioBlock>> = Mutex::new(None);

pub fn init_disk() {
    let header = unsafe { &mut *(VIRTIO0 as *mut VirtioHeader) };
    let block = VirtioBlock::new(header);
    match block {
        Ok(block) => unsafe {
            let mut disk = DISK.lock();
            *disk = Some(block);
        },
        Err(_err) => panic!("Error: Disk initialization"),
    }
}

pub fn read_disk() {
    let disk = unsafe { DISK.lock().as_mut().unwrap() };
}
