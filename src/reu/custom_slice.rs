use core::ops::{Index, IndexMut};
use core::mem;
use core::cell::UnsafeCell;
use crate::reu;
use ufmt_stdio::*; // stdio dla środowisk, które nie mają std

pub struct REUArray<'a, T> {
    cache: UnsafeCell<&'a mut [T]>,  // Mutable reference to the local cache wrapped in UnsafeCell
    element_count: u32,            // Total number of elements in the remote data
    window_start_index: UnsafeCell<u32>,  // The starting index of the current window in the remote data
    iter_index: UnsafeCell<u32>,      // UnsafeCell to allow interior mutability
    window_size: u32,
    dirty: bool,
}

impl<'a, T> REUArray<'a, T> {
    pub fn new(cache: &'a mut [T], element_count: u32, windows_szie: u32) -> Self {
        REUArray {
            cache: UnsafeCell::new(cache),
            element_count,
            window_start_index: UnsafeCell::new(0), // start with the beginning of the remote data
            iter_index: UnsafeCell::new(0),
            window_size: windows_szie,
            dirty: false,
        }
    }

    fn ensure_in_cache(&self, index: u32) {
        let window_start_index = unsafe { *self.window_start_index.get() };

        if index < window_start_index || index >= window_start_index + self.window_size {
            unsafe {                
                self.prepare(*self.window_start_index.get());
                if self.dirty { 
                    reu::reu().swap_out(); 
                    //self.dirty = true;
                }
                println!("Cache missed for index {}", index);
                self.prepare(index);
                reu::reu().swap_in();

                *self.window_start_index.get() = index;
            }
        }
    }

    fn prepare(&self, remote_index: u32) {
        let element_size = mem::size_of::<T>() as u32;
        let byte_count = element_size * self.window_size;
        let cache_ptr = unsafe { (*self.cache.get()).as_mut_ptr() };
        reu::reu().prepare(cache_ptr as u16, remote_index * element_size as u32, byte_count as u16);
    }
}

impl<'a, T> Index<u32> for REUArray<'a, T> {
    type Output = T;

    fn index(&self, index: u32) -> &Self::Output {
        assert!(
            index < self.element_count,
            "Index out of bounds: index = {}, size = {}",
            index,
            self.element_count
        );

        self.ensure_in_cache(index);

        let cache_index = index - unsafe { *self.window_start_index.get() };
        unsafe { &(*self.cache.get())[cache_index as usize] }
    }
}

impl<'a, T> IndexMut<u32> for REUArray<'a, T> {
    fn index_mut(&mut self, index: u32) -> &mut Self::Output {
        assert!(
            index < self.element_count,
            "Index out of bounds: index = {}, size = {}",
            index,
            self.element_count
        );

        //dirty = true;
        self.ensure_in_cache(index);

        let cache_index = index - unsafe { *self.window_start_index.get() };
        unsafe { &mut (*self.cache.get())[cache_index as usize] }
    }
}

// impl<'a, T> Iterator for REUArray<'a, T> {
//     type Item = &'a T;

//     fn next(&mut self) -> Option<Self::Item> {
//         let iter_index = unsafe { &mut *self.iter_index.get() };

//         if *iter_index < self.element_count {
//             self.ensure_in_cache(*iter_index);

//             // Return the reference directly within Some, avoiding the intermediate variable.
//             let item = &self[*iter_index];
//             *iter_index += 1;
//             Some(item)
//         } else {
//             None
//         }
//     }
// }