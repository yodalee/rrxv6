// Byte per page and big offset within a page
pub const PAGESIZE : u64 = 4096;
pub const PAGESHIFT : u64 = 12;

// MAXVA marks the virtual address limitation
pub const MAXVA : u64 = 1 << (9 + 9 + 9 + 12 - 1);

// Maximum Interrupt Count
pub const MAX_INTERRUPT : u64 = 1024;

pub enum Interrupt {
    SupervisorSoftware = 1,
    SupervisorTimer = 5,
    SupervisorExternal = 9,
}

pub enum Exception {
    InstructionAddressMisaligned = 0,
    InstructionAccessFault = 1,
    IllegalInstruction = 2,
    Breakpoint = 3,
    LoadAddressMisaligned = 4,
    LoadAccessFault = 5,
    StoreAddressMisaligned = 6,
    StoreAccessFault = 7,
    EnvironmentCallUMode = 8,
    EnvironmentCallSMode = 9,
    InstructionPageFault = 12,
    LoadPageFault = 13,
    StorePageFault = 15,
}
