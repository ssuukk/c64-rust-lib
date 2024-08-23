use crate::ram_expansion_unit;
use crate::reu_allocator::ReuChunk;
use core::cell::UnsafeCell;
use core::mem;
use core::ops::{Index, IndexMut};

extern "C" {
    fn malloc(n: usize) -> *mut u8;
    fn free(ptr: *mut u8);
}

pub struct REUArray<T> {
    cache: UnsafeCell<*mut T>, // Pointer to the heap-allocated cache wrapped in UnsafeCell
    capacity: u32,             // Total number of elements in the remote data
    window_start_index: UnsafeCell<u32>, // The starting index of the current window in the remote data
    iter_index: UnsafeCell<u32>,         // UnsafeCell to allow interior mutability
    window_size: u32,
    dirty: UnsafeCell<bool>, // Changed from bool to UnsafeCell<bool>
    reu_chunk: ReuChunk,
    element_size: usize,
    element_count: u32,
}

impl<T> REUArray<T> {
    pub fn with_capacity(capacity: u32, window_size: u32) -> Self {
        let element_size = mem::size_of::<T>();
        let cache_size = window_size as usize * element_size;

        unsafe {
            let cache_ptr = malloc(cache_size) as *mut T;
            if cache_ptr.is_null() {
                panic!("out of memory");
            }
            let reu_ptr = ram_expansion_unit::reu().alloc(capacity * element_size as u32);

            REUArray {
                cache: UnsafeCell::new(cache_ptr),
                capacity,
                window_start_index: UnsafeCell::new(0), // start with the beginning of the remote data
                iter_index: UnsafeCell::new(0),
                window_size,
                dirty: UnsafeCell::new(false), // Initialize with UnsafeCell<bool>
                reu_chunk: reu_ptr,
                element_size,
                element_count: 0,
            }
        }
    }

    pub fn push(&mut self, element: T) {
        let i = self.element_count;
        self[i] = element;
        self.element_count += 1;
    }

    fn ensure_in_cache(&self, index: u32) {
        let window_start_index = unsafe { *self.window_start_index.get() };

        if index < window_start_index || index >= window_start_index + self.window_size {
            //println!("index {:?}", self);

            // println!("Cache ptr in ensure {}", self.cache.get() as u16);
            unsafe {
                // println!("Cache missed for {}, dirty={}",index,  *self.dirty.get());
                if *self.dirty.get() {
                    self.prepare_slice();
                    ram_expansion_unit::reu().push();
                    *self.dirty.get() = false;
                }
                *self.window_start_index.get() = index;
                self.prepare_slice();
                ram_expansion_unit::reu().pull();
            }
        }
    }

    fn check_bounds(&self, index: u32) {
        assert!(
            index < self.element_count,
            "index {}/{}",
            index,
            self.element_count
        );
    }

    fn return_cached(&self, index: u32) -> &mut T {
        unsafe {
            self.ensure_in_cache(index);
            let index_in_window = (index - *self.window_start_index.get()) as isize;
            let offset_ptr = (*self.cache.get()).offset(index_in_window);
            &mut *offset_ptr // Correctly dereference the raw pointer to get a mutable reference
        }
    }

    fn prepare_slice(&self) {
        unsafe {
            let byte_count = self.element_size as u32 * self.window_size;
            ram_expansion_unit::reu().set_range(
                *self.cache.get() as usize,
                self.reu_chunk.address + *self.window_start_index.get() * self.element_size as u32,
                byte_count as u16,
            );
        }
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
        }
    }
}

impl<T: Clone> core::iter::ExactSizeIterator for REUArray<T> {
    fn len(&self) -> usize {
        self.element_count as usize
    }
}

impl<T> Index<u32> for REUArray<T> {
    type Output = T;
    fn index(&self, index: u32) -> &Self::Output {
        #[cfg(debug_assertions)]
        self.check_bounds(index);
        self.return_cached(index)
    }
}

impl<T> IndexMut<u32> for REUArray<T> {
    fn index_mut(&mut self, index: u32) -> &mut Self::Output {
        #[cfg(debug_assertions)]
        self.check_bounds(index);
        unsafe {
            *self.dirty.get() = true;
            self.return_cached(index)
        }
    }
}

impl<T> ufmt::uDebug for REUArray<T> {
    fn fmt<W: ufmt::uWrite + ?Sized>(
        &self,
        f: &mut ufmt::Formatter<'_, W>,
    ) -> Result<(), W::Error> {
        let cache_ptr = unsafe { *self.cache.get() };
        f.write_str("ReuArray of ")?;
        self.element_count.fmt(f)?;
        f.write_str("\nl:")?;
        cache_ptr.fmt(f)?;
        f.write_str("  r:")?;
        self.reu_chunk.fmt(f)?;
        f.write_str("\nwindow=")?;
        unsafe {
            let ws = *self.window_start_index.get();
            let dr = *self.dirty.get();
            ws.fmt(f)?;
            f.write_str(" dirty=")?;
            dr.fmt(f)?;
        }
        Ok(())
    }
}
