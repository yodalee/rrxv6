//! list struct that are used in scheduler

use core::ptr;
use alloc::boxed::Box;

type Link<T> = *mut Node<T>;

pub struct List<T> {
    head: Link<T>,
    tail: *mut Node<T>,
}

struct Node<T> {
    elem: T,
    next: Link<T>
}

impl<T> List<T> {
    pub fn new() -> Self {
        List {
            head: ptr::null_mut(),
            tail: ptr::null_mut(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.head.is_null()
    }

    pub fn push(&mut self, elem: T) {
        unsafe {
            let new_tail = Box::into_raw(Box::new(Node {
                elem,
                next: ptr::null_mut(),
            }));

            if self.is_empty() {
                self.head = new_tail;
            } else {
                (*self.tail).next = new_tail;
            }
            self.tail = new_tail;
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        unsafe {
            if self.is_empty() {
                None
            } else {
                let head = Box::from_raw(self.head);
                self.head = head.next;

                if self.is_empty() {
                    self.tail = ptr::null_mut();
                }

                Some(head.elem)
            }
        }
    }

    pub fn peek(&self) -> Option<&T> {
        unsafe {
            self.head.as_ref().map(|node| &node.elem)
        }
    }

    pub fn peek_mut(&mut self) -> Option<&mut T> {
        unsafe {
            self.head.as_mut().map(|node| &mut node.elem)
        }
    }
}

impl<T> Drop for List<T> {
    fn drop(&mut self) {
        while let Some(_) = self.pop() { }
    }
}
