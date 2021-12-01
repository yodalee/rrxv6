use bitflags::bitflags;

// bit flag for page permission
bitflags! {
    pub struct PteFlag: u64 {
        const PTE_VALID = 0x01;
        const PTE_READ  = 0x02;
        const PTE_WRITE = 0x04;
        const PTE_EXEC  = 0x08;
        const PTE_USER  = 0x10;
        const PTE_GLOB  = 0x20;
        const PTE_ACCES = 0x40;
        const PTE_DIRTY = 0x80;
    }
}
