//! Physical memory allocator for user processes,
//! kernal stacks, page-table pages and pipe buffers.
//! Allocates whole 4096-byte pages

use crate::memorylayout;
use crate::param;

use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;

pub struct HeapAllocator;

unsafe impl GlobalAlloc for HeapAllocator {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        null_mut()
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        panic!("dealloc should be never called");
    }
}

#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    panic!("allocation error {:?}", layout);
}
