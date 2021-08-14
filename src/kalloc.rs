
use crate::memorylayout;
use crate::ALLOCATOR;

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

