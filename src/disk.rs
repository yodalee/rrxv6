use crate::virtio::header::VirtioHeader;
use crate::virtio::block::VirtioBlock;
use crate::memorylayout::VIRTIO0;

static mut DISK: Option<VirtioBlock> = None;

pub fn init_disk() {
    let header = unsafe { &mut *(VIRTIO0 as *mut VirtioHeader) };
    let block = VirtioBlock::new(header);
    match block {
        Ok(block) => unsafe {
            DISK = Some(block);
        },
        Err(()) => panic!("Error: Disk initialization"),
    }
}

pub fn get_disk() -> &'static mut VirtioBlock {
    unsafe {
        DISK.as_mut().unwrap()
    }
}
