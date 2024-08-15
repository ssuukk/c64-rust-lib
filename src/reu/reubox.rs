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
    // Creates a new `ReuBox` and allocates the value on the heap.
    pub fn new(value: T, count: usize) -> ReuBox<T> {
        // Calculate size and alignment
        let size = size_of::<T>();

        // Allocate memory (use a global allocator or implement your own)
        let ptr = unsafe {
            let ptr: *mut T = malloc(size) as *mut T;
            if ptr.is_null() {
                panic!("Memory allocation failed");
            }
            //ptr::write(ptr, value);
            NonNull::new_unchecked(ptr)
        };
        let ptr24 = unsafe { reu().alloc((size*count) as u32) };

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

    pub fn swap_out(&self, idx: usize) {
        let size = size_of::<T>();
        let location = size as u32 * idx as u32;
        reu().prepare(self.ptr.as_ptr() as u16, self.ptr24.address + location, size as u16);
        reu().swap_out();
    }

    fn swap_in(&self, idx: usize) {
        let size = size_of::<T>();
        let location = size as u32 * idx as u32;
        reu().prepare(self.ptr.as_ptr() as u16, self.ptr24.address + location, size as u16);
        reu().swap_in();
    }

    pub fn put(&mut self, index: usize, value: T) {
        unsafe { 
            (*self.ptr.as_mut()) = value;
            self.swap_out(index);
        }
    }
}

// Deref trait for `ReuBox` to allow dereferencing the inner value.
impl<T> core::ops::Deref for ReuBox<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        //unsafe { self.ptr.as_ref() }
        unsafe {
            let ptr = ptr::read_volatile(&self.ptr);
            ptr.as_ref()
        }
    }
}

// DerefMut trait for `ReuBox` to allow mutable dereferencing.
impl<T> core::ops::DerefMut for ReuBox<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        //println!("Dereferencing mutable");
        unsafe { self.ptr.as_mut() }
    }
}

// Drop trait for `ReuBox` to deallocate memory when the `ReuBox` is dropped.
impl<T> Drop for ReuBox<T> {
    fn drop(&mut self) {
        unsafe {
            free(self.ptr.as_ptr() as *mut u8);
            reu().dealloc(self.ptr24);
        }
    }
}

// do pobierania wartości z tablicy
impl<T> Index<usize> for ReuBox<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        self.swap_in(index);
        unsafe { self.ptr.as_ref() }
    }
}

// // do wpisywania wartości do tablicy
// impl<T> IndexMut<usize> for ReuBox<T> {
//     fn index_mut(&mut self, index: usize) -> &mut Self::Output {
//         print!("Indexing mutable");
//         unsafe { self.ptr.as_mut() }
//     }
// }