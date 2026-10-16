use crate::ram_expansion_unit;
use crate::reu_allocator::ReuChunk;
use core::cmp::min;
use core::mem;

extern "C" {
    fn malloc(n: usize) -> *mut u8;
    fn free(ptr: *mut u8);
}

/// An array of copyable values stored in REU memory and paged through a RAM cache.
pub struct REUArray<T: Copy> {
    cache: *mut T,
    capacity: u32,
    window_start_index: u32,
    window_size: u32,
    dirty: bool,
    reu_chunk: ReuChunk,
    element_size: usize,
    element_count: u32,
}

impl<T: Copy> REUArray<T> {
    /// Create a fixed-capacity REU array with a cache of `window_size` elements.
    pub fn with_capacity(capacity: u32, window_size: usize) -> Self {
        assert!(capacity > 0, "REUArray capacity must be nonzero");
        assert!(window_size > 0, "REUArray window must be nonzero");

        let element_size = mem::size_of::<T>();
        assert!(element_size > 0, "zero-sized REUArray elements are unsupported");

        let cache_size = window_size
            .checked_mul(element_size)
            .expect("REUArray cache size overflow");
        assert!(cache_size <= u16::MAX as usize, "REUArray window is too large");

        let window_size = u32::try_from(window_size).expect("REUArray window is too large");
        let element_size_u32 = u32::try_from(element_size).expect("REUArray element is too large");
        let reu_size = capacity
            .checked_mul(element_size_u32)
            .expect("REUArray size overflow");

        let reu_chunk = ram_expansion_unit::reu().alloc(reu_size);
        let cache = unsafe { malloc(cache_size) as *mut T };
        if cache.is_null() {
            panic!("out of memory");
        }

        REUArray {
            cache,
            capacity,
            window_start_index: 0,
            window_size,
            dirty: false,
            reu_chunk,
            element_size,
            element_count: 0,
        }
    }

    /// Number of elements currently stored in the array.
    pub fn len(&self) -> u32 {
        self.element_count
    }

    /// Whether the array currently contains no elements.
    pub fn is_empty(&self) -> bool {
        self.element_count == 0
    }

    /// Append an element. Panics if the fixed capacity is exhausted.
    pub fn push(&mut self, element: T) {
        assert!(self.element_count < self.capacity, "REUArray capacity exceeded");

        let index = self.element_count;
        self.ensure_in_cache(index, false);
        let cache_index = (index - self.window_start_index) as usize;
        unsafe {
            *self.cache.add(cache_index) = element;
        }
        self.element_count += 1;
        self.dirty = true;
    }

    /// Return a copy of an element, loading its cache window if needed.
    pub fn get(&mut self, index: u32) -> T {
        self.check_bounds(index);
        self.ensure_in_cache(index, true);
        let cache_index = (index - self.window_start_index) as usize;
        unsafe { *self.cache.add(cache_index) }
    }

    /// Replace an initialized element.
    pub fn set(&mut self, index: u32, value: T) {
        self.check_bounds(index);
        self.ensure_in_cache(index, true);
        let cache_index = (index - self.window_start_index) as usize;
        unsafe {
            *self.cache.add(cache_index) = value;
        }
        self.dirty = true;
    }

    /// Mutate an element without allowing a reference to escape the current cache window.
    pub fn with_mut<F>(&mut self, index: u32, mutate: F)
    where
        F: for<'a> FnOnce(&'a mut T),
    {
        self.check_bounds(index);
        self.ensure_in_cache(index, true);
        let cache_index = (index - self.window_start_index) as usize;
        self.dirty = true;
        unsafe {
            mutate(&mut *self.cache.add(cache_index));
        }
    }

    /// Visit each initialized element by value without exposing cache references.
    pub fn for_each<F>(&mut self, mut visit: F)
    where
        F: FnMut(u32, T),
    {
        let element_count = self.element_count;
        for index in 0..element_count {
            visit(index, self.get(index));
        }
    }

    /// Mutate every initialized element while keeping each cache borrow scoped to its callback.
    pub fn for_each_mut<F>(&mut self, mut mutate: F)
    where
        F: for<'a> FnMut(u32, &'a mut T),
    {
        let element_count = self.element_count;
        for index in 0..element_count {
            self.with_mut(index, |element| mutate(index, element));
        }
    }

    /// Write pending cache changes back to REU memory.
    pub fn flush(&mut self) {
        if self.dirty {
            self.prepare_slice();
            ram_expansion_unit::reu().push();
            self.dirty = false;
        }
    }

    fn check_bounds(&self, index: u32) {
        assert!(index < self.element_count, "REUArray index out of bounds");
    }

    fn ensure_in_cache(&mut self, index: u32, load: bool) {
        let outside_window = index < self.window_start_index
            || index - self.window_start_index >= self.window_size;

        if outside_window {
            self.flush();
            self.window_start_index = index;

            if load {
                self.prepare_slice();
                ram_expansion_unit::reu().pull();
            }
        }
    }

    fn prepare_slice(&self) {
        let available = self.element_count - self.window_start_index;
        let element_count = min(self.window_size, available);
        let byte_count = element_count * self.element_size as u32;
        ram_expansion_unit::reu().set_range(
            self.cache as usize,
            self.reu_chunk.address() + self.window_start_index * self.element_size as u32,
            byte_count as usize,
        );
    }
}

impl<T: Copy> Drop for REUArray<T> {
    fn drop(&mut self) {
        self.flush();
        unsafe {
            free(self.cache as *mut u8);
        }
    }
}

impl<T: Copy> ufmt::uDebug for REUArray<T> {
    fn fmt<W: ufmt::uWrite + ?Sized>(
        &self,
        f: &mut ufmt::Formatter<'_, W>,
    ) -> Result<(), W::Error> {
        f.write_str("ReuArray of ")?;
        self.element_count.fmt(f)?;
        f.write_str("\nl:")?;
        self.cache.fmt(f)?;
        f.write_str("  r:")?;
        self.reu_chunk.fmt(f)?;
        f.write_str("\nwindow=")?;
        self.window_start_index.fmt(f)?;
        f.write_str(" dirty=")?;
        self.dirty.fmt(f)?;
        Ok(())
    }
}