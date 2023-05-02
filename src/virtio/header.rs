use bitflags::bitflags;
use volatile_register::{RO, RW, WO};

/// MMIO Device Register Layout
///
/// Ref: 4.2.2 MMIO Device Register Layout
pub struct VirtioHeader {
    // 0x00
    magic: RO<u32>,
    version: RO<u32>,
    device_id: RO<u32>,
    vendor_id: RO<u32>,
    // 0x10
    device_features: RO<u32>,
    device_features_sel: WO<u32>,
    _r0: [u32; 2],
    // 0x20
    driver_features: WO<u32>,
    driver_features_sel: WO<u32>,
    _r1: [u32; 2],
    // 0x30
    queue_sel: WO<u32>,
    queue_num_max: RO<u32>,
    queue_num: WO<u32>,
    _r2: [u32; 1],
    // 0x40
    _r3: [u32; 1],
    queue_ready: RW<u32>,
    _r4: [u32; 2],
    // 0x50
    queue_notify: WO<u32>,
    _r5: [u32; 3],
    // 0x60
    interrupt_status: RO<u32>,
    interrupt_ack: WO<u32>,
    _r6: [u32; 2],
    // 0x70
    status: RW<DeviceStatus>,
    _r7: [u32; 3],
    // 0x80
    queue_desc_low: WO<u32>,
    queue_desc_high: WO<u32>,
    _r8: [u32; 2],
    // 0x90
    queue_driver_low: WO<u32>,
    queue_driver_high: WO<u32>,
    _r9: [u32; 2],
    // 0xa0
    queue_device_low: WO<u32>,
    queue_device_high: WO<u32>,
    _r10: [u32; 21],
    // 0xfc
    config_generation: RO<u32>,
}

impl VirtioHeader {
    /// Initialize the device.
    ///
    /// Ref: virtio 3.1.1 Device Initialization
    pub fn begin_init(&mut self, negotiate_features: impl FnOnce(u64) -> u64) -> Result<(), ()> {
        unsafe {
            self.status.write(DeviceStatus::ACKNOWLEDGE);
            self.status.write(DeviceStatus::DRIVER);
            let features = self.read_device_features();
            self.write_driver_features(negotiate_features(features));
            self.status.write(DeviceStatus::FEATURES_OK);
            // check that status keep in FEATURES_OK
            if self.status.read() != DeviceStatus::FEATURES_OK {
                return Err(());
            }
        }
        Ok(())
    }

    pub fn end_init(&mut self) {
        unsafe {
            let mut status = self.status.read();
            status |= DeviceStatus::DRIVER_OK;
            self.status.write(status);
        }
    }

    fn read_device_features(&self) -> u64 {
        unsafe {
            self.device_features_sel.write(0);
            let mut device_features = self.device_features.read().into();
            self.device_features_sel.write(1);
            device_features |= (self.device_features.read() as u64) << 32;
            device_features
        }
    }

    fn write_driver_features(&mut self, driver_features: u64) {
        unsafe {
            self.driver_features_sel.write(0);
            self.driver_features.write(driver_features as u32);
            self.driver_features_sel.write(1);
            self.driver_features.write((driver_features >> 32) as u32)
        }
    }

    /// Verify header
    pub fn verify(&self) -> bool {
        self.magic.read() == 0x74726976
            && self.version.read() == 2
            && self.device_id.read() == 2
            && self.vendor_id.read() == 0x554D4551
    }

    /// Check queue status
    pub fn queue_used(&mut self, idx: u32) -> bool {
        unsafe {
            self.queue_sel.write(idx);
        }
        self.queue_ready.read() != 0
    }

    /// check queue num
    pub fn max_queue_size(&mut self) -> u32 {
        self.queue_num_max.read()
    }

    /// set queue num
    pub fn set_queue(&mut self, idx: u32, size: u16) {
        unsafe {
            self.queue_sel.write(idx);
            self.queue_num.write(size as u32);
        }
    }

    /// set queue ready
    pub fn set_queue_ready(&mut self, ready: bool) {
        unsafe {
            self.queue_ready.write(if ready { 1 } else { 0 });
        }
    }
}

bitflags! {
    pub struct DeviceStatus: u32 {
        const ACKNOWLEDGE = 1;
        const DRIVER = 2;
        const DRIVER_OK = 4;
        const FEATURES_OK = 8;
        const DEVICE_NEEDS_RESET = 64;
        const FAILED = 128;
    }
}

/// Types of virtio devices.
#[repr(u8)]
#[derive(Debug, Eq, PartialEq)]
#[allow(unused)]
pub enum DeviceType {
    Invalid = 0,
    Network = 1,
    Block = 2,
    Console = 3,
    EntropySource = 4,
    MemoryBallooning = 5,
    IoMemory = 6,
    Rpmsg = 7,
    ScsiHost = 8,
    _9pTransport = 9,
    Mac80211 = 10,
    RprocSerial = 11,
    VirtioCAIF = 12,
    MemoryBalloon = 13,
    GPU = 16,
    TimerClock = 17,
    Input = 18,
    Socket = 19,
    Crypto = 20,
    SignalDistributionModule = 21,
    Pstore = 22,
    IOMMU = 23,
    Memory = 24,
}
