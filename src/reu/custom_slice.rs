use core::ops::{Index, IndexMut};
use core::mem;
use core::cell::UnsafeCell;
use crate::reu;
use reu::wallocator::{ WAllocator, Ptr24 };
use ufmt_stdio::*; // stdio dla środowisk, które nie mają std

extern "C" {
    fn malloc(n: usize) -> *mut u8;
    fn free(ptr: *mut u8);
    fn __heap_bytes_free() -> usize;
    fn __heap_bytes_used() -> usize;
    fn __set_heap_limit(limit: usize);
    fn __heap_limit() -> usize;
}

pub struct REUArray<T> {
    cache: UnsafeCell<*mut T>,  // Pointer to the heap-allocated cache wrapped in UnsafeCell
    element_count: u32,            // Total number of elements in the remote data
    window_start_index: UnsafeCell<u32>,  // The starting index of the current window in the remote data
    iter_index: UnsafeCell<u32>,      // UnsafeCell to allow interior mutability
    window_size: u32,
    dirty: UnsafeCell<bool>, // Changed from bool to UnsafeCell<bool>
    reu_address: Ptr24,
    element_size: usize,
}

impl<T> REUArray<T> {
    pub fn new(element_count: u32, window_size: u32) -> Self {
        // Allocate memory for the cache on the heap
        let element_size = mem::size_of::<T>();
        let cache_size = window_size as usize * element_size; 
        let cache_ptr = unsafe { malloc(cache_size) as *mut T };

        if cache_ptr.is_null() {
            panic!();
        }

        unsafe {
            let reu_ptr = reu::reu().alloc(element_count*element_size as u32);
            if cache_ptr.is_null() {
                panic!();
            }
        
            REUArray {
                cache: UnsafeCell::new(cache_ptr),
                element_count,
                window_start_index: UnsafeCell::new(0), // start with the beginning of the remote data
                iter_index: UnsafeCell::new(0),
                window_size,
                dirty: UnsafeCell::new(false), // Initialize with UnsafeCell<bool>
                reu_address: reu_ptr,
                element_size,
            }
        }
    }

    fn ensure_in_cache(&self, index: u32) {
        let window_start_index = unsafe { *self.window_start_index.get() };

        if index < window_start_index || index >= window_start_index + self.window_size {
            unsafe {            
                println!("Cache missed for {}",index);    
                if *self.dirty.get() { 
                    println!("Cache was dirty, commiting");
                    self.prepare();
                    reu::reu().swap_out(); 
                    *self.dirty.get() = false;
                }
                *self.window_start_index.get() = index;
                self.prepare();
                reu::reu().swap_in();
            }
        }
    }

    fn prepare(&self) {
        unsafe {
            let byte_count = self.element_size as u32 * self.window_size;
            reu::reu().prepare(
                *self.cache.get() as u16, 
                self.reu_address.address + *self.window_start_index.get() * self.element_size as u32, 
                byte_count as u16
            );
        }
    }
}

impl<T> Index<u32> for REUArray<T> {
    type Output = T;

    fn index(&self, index: u32) -> &Self::Output {
        #[cfg(debug_assertions)]
        assert!(
            index < self.element_count,
            "Index out of bounds: index = {}, size = {}",
            index,
            self.element_count
        );

        self.ensure_in_cache(index);

        let cache_index = index - unsafe { *self.window_start_index.get() };
        unsafe { &*((*self.cache.get()).offset(cache_index as isize)) }
    }
}

impl<T> IndexMut<u32> for REUArray<T> {
    fn index_mut(&mut self, index: u32) -> &mut Self::Output {
        #[cfg(debug_assertions)]
        assert!(
            index < self.element_count,
            "Index out of bounds: index = {}, size = {}",
            index,
            self.element_count
        );

        unsafe {
            *self.dirty.get() = true;
        }
        self.ensure_in_cache(index);

        let cache_index = index - unsafe { *self.window_start_index.get() };
        unsafe { &mut *((*self.cache.get()).offset(cache_index as isize)) }
    }
}

impl<T> Iterator for REUArray<T> 
where
    T: Clone, // Ensure T can be cloned
{
    type Item = T; // Return owned values instead of references

    fn next(&mut self) -> Option<Self::Item> {
        let iter_index = unsafe { &mut *self.iter_index.get() };

        if *iter_index < self.element_count {
            self.ensure_in_cache(*iter_index);

            let item = self[*iter_index].clone(); // Clone the item
            *iter_index += 1;
            Some(item)
        } else {
            None
        }
    }
}

impl<'a, T> Drop for REUArray<T> {
    fn drop(&mut self) {
        unsafe {
            free(*self.cache.get() as *mut u8);
            reu::reu().dealloc(self.reu_address);
        }
    }
}
