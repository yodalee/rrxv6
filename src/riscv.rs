// Byte per page and big offset within a page
pub const PAGESIZE : u64 = 4096;
pub const PAGESHIFT : u64 = 12;

// MAXVA marks the virtual address limitation
pub const MAXVA : u64 = 1 << (9 + 9 + 9 + 12 - 1);

// Maximum Interrupt Count
pub const MAX_INTERRUPT : u64 = 1024;
