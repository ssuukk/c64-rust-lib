//use std::ops::{Index, IndexMut};
use core::ops::{ Index, IndexMut };
use core::mem;
use core::cell::UnsafeCell;
use crate::reu;
use ufmt_stdio::*; // stdio dla środowisk, które nie mają std

/// A struct representing a custom slice that acts as a window over a larger dataset.
pub struct CustomSlice<'a, T> {
    cache: &'a mut [T],      // Mutable reference to the local cache
    remote_data_size: u32,   // Total number of elements in the remote data
    cache_start_index: u32,  // The starting index of the current window in the remote data
    iter_index: u32,
}

impl<'a, T> CustomSlice<'a, T> {
    /// Creates a new `CustomSlice`.
    pub fn new(
        cache: &'a mut [T],
        remote_data_size: u32,
    ) -> Self {
        CustomSlice {
            cache,
            remote_data_size,
            cache_start_index: 0, // start with the beginning of the remote data
            iter_index: 0,
        }
    }

    /// Ensure that the element at `index` is within the cached window.
    fn ensure_in_cache(&mut self, index: u32) {
        println!("Ensuring cache for index {}", index);
        let cache_size = self.cache.len() as u32;

        if index < self.cache_start_index || index >= self.cache_start_index + cache_size {
            let swap_start = index;
            let swap_end = swap_start + cache_size.min(self.remote_data_size - swap_start);

            let byte_count = ((swap_end - swap_start) as usize) * mem::size_of::<T>();
            self.swap_in(swap_start, byte_count);

            self.cache_start_index = swap_start;
        }
    }

    fn swap_in(&mut self, remote_index: u32, byte_count: usize) {
        reu::reu().prepare((self.cache.as_mut_ptr() as usize) as u16, remote_index, byte_count as u16);
        reu::reu().swap_in();
    }

    fn swap_out(&mut self, remote_index: u32, byte_count: usize) {
        reu::reu().prepare((self.cache.as_mut_ptr() as usize) as u16, remote_index, byte_count as u16);
        reu::reu().swap_out();
    }
}

impl<'a, T> Index<u32> for CustomSlice<'a, T> {
    type Output = T;

    fn index(&self, index: u32) -> &Self::Output {
        println!("index for index {} < {}", index, self.remote_data_size);
        assert!(
            index < self.remote_data_size,
            "Index out of bounds: index = {}, size = {}",
            index,
            self.remote_data_size
        );

        self.ensure_in_cache(index);

        let cache_index = index - unsafe { *self.cache_start_index.get() };
        unsafe { &(*self.cache.get())[cache_index as usize] }
    }
}

impl<'a, T> IndexMut<u32> for CustomSlice<'a, T> {
    fn index_mut(&mut self, index: u32) -> &mut Self::Output {
        println!("index_mut for index {}", index);
        assert!(
            index < self.remote_data_size,
            "Index out of bounds: index = {}, size = {}",
            index,
            self.remote_data_size
        );

        self.ensure_in_cache(index);

        let cache_index = index - self.cache_start_index;
        &mut self.cache[cache_index as usize]
    }
}

impl<'a, T> Iterator for CustomSlice<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        // Increment our count. This is why we started at zero.
        self.iter_index += 1;

        // Check to see if we've finished counting or not.
        if self.iter_index < self.remote_data_size {
            Some(self[self.iter_index])
        } else {
            None
        }
    }
}