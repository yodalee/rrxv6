
use alloc::alloc::alloc;
use alloc::alloc::Layout;
use crate::memorylayout;
use crate::riscv::PAGESIZE;
use crate::ALLOCATOR;

use core::ptr::write_bytes;

pub fn init_heap() {
  extern "C" {
    // _END defined in linker.ld
    static _END: usize;
  }

  let heap_start: usize = unsafe {
    &_END as *const usize as usize
  };
  let heap_end = memorylayout::PHYSTOP as usize;
  let heap_size = heap_end - heap_start;
  unsafe {
    ALLOCATOR
      .lock()
      .init(heap_start, heap_size)
  }
}

/// Allocate one 4096-byte page of physical memory.
/// Returns a pointer that the kernel can use.
/// Returns 0 if the memory cannot be allocated.
pub fn kalloc() -> *mut u8 {
    unsafe {
        let layout = Layout::from_size_align(PAGESIZE as usize, 4096).unwrap();
        let ptr = alloc(layout);
        write_bytes(ptr, 0x0, PAGESIZE as usize);
        return ptr;
    }
}
