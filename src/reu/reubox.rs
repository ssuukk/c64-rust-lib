use core::ptr::{self, NonNull};
use core::mem::size_of;
use core::ops::Index;

use crate::reu::wallocator::{Ptr24, WAllocator};
use crate::reu::reu;

pub struct ReuBox<T> {
    pub ptr: NonNull<T>,
    pub ptr24: Ptr24,
}

extern "C" {
    fn malloc(n: usize) -> *mut u8;
    fn free(ptr: *mut u8);
    fn __heap_bytes_free() -> usize;
    fn __heap_bytes_used() -> usize;
    fn __set_heap_limit(limit: usize);
    fn __heap_limit() -> usize;
}

impl<T> ReuBox<T> {
    pub fn new(value: T) -> ReuBox<T> {
        let size = size_of::<T>();

        let ptr = unsafe {
            let ptr: *mut T = malloc(size) as *mut T;
            if ptr.is_null() {
                panic!("Memory allocation failed");
            }
            NonNull::new_unchecked(ptr)
        };
        let ptr24 = unsafe { reu().alloc((size) as u32) };

        ReuBox { ptr, ptr24 }
    }

    // Consumes the `ReuBox`, returning the inner value.
    pub fn into_inner(self) -> T {
        unsafe {
            let value = ptr::read(self.ptr.as_ptr());
            free(self.ptr.as_ptr() as *mut u8);
            reu().dealloc(self.ptr24);
            value
        }
    }
}

// Deref trait for `ReuBox` to allow dereferencing the inner value.
impl<T> core::ops::Deref for ReuBox<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe {
            let ptr = ptr::read_volatile(&self.ptr);
            ptr.as_ref()
        }
    }
}

impl<T> core::ops::DerefMut for ReuBox<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.ptr.as_mut() }
    }
}

impl<T> Drop for ReuBox<T> {
    fn drop(&mut self) {
        unsafe {
            free(self.ptr.as_ptr() as *mut u8);
            reu().dealloc(self.ptr24);
        }
    }
}
