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
        if i < self.capacity {
            self[i] = element;
            self.element_count += 1;
        }
    }

    fn ensure_in_cache(&self, index: u32) {
        let window_start_index = unsafe { *self.window_start_index.get() };

        if index < window_start_index || index >= window_start_index + self.window_size {
            unsafe {
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

    pub fn iter_mut(&mut self) -> REUArrayIterMut<T> {
        REUArrayIterMut {
            reu_array: self,
            index: 0,
        }
    }
}

pub struct REUArrayIterMut<'a, T> {
    reu_array: &'a mut REUArray<T>,
    index: u32,
}

impl<'a, T> Iterator for REUArrayIterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.reu_array.element_count {
            self.reu_array.ensure_in_cache(self.index);

            // Temporarily split the borrow to avoid issues with mutable references
            let window_start_index = unsafe { *self.reu_array.window_start_index.get() };
            let index_in_window = self.index - window_start_index;
            let item = unsafe {
                let cache_ptr = *self.reu_array.cache.get();
                let item_ptr = cache_ptr.add(index_in_window as usize);
                &mut *item_ptr
            };
            self.index += 1;
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
