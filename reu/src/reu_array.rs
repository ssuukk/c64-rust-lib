use crate::ram_expansion_unit;
use crate::reu_allocator::ReuChunk;
use core::cell::UnsafeCell;
use core::cmp::min;
use core::mem;
use core::ops::{Index, IndexMut};

extern "C" {
    fn malloc(n: usize) -> *mut u8;
    fn free(ptr: *mut u8);
}

/// An array of copyable values stored in REU memory and paged through a RAM cache.
pub struct REUArray<T: Copy> {
    /// Scratch buffer that pages a window of `T`s between RAM and REU.
    /// `UnsafeCell` so that `Index::index` and `len`/`is_empty` can be `&self`
    /// while still allowing the cache to be loaded on demand. The buffer is
    /// logically private: we never expose `&T` references into it, so the
    /// interior-mutability aliasing is sound.
    cache: UnsafeCell<*mut T>,
    capacity: u32,
    /// REU virtual offset of the cached window, in units of `T`s.
    window_start_index: UnsafeCell<u32>,
    /// Window size in units of `T`s.
    window_size: u32,
    /// Whether the cache has unsaved changes to flush to REU.
    dirty: UnsafeCell<bool>,
    reu_chunk: ReuChunk,
    element_size: usize,
    /// Number of initialized `T`s currently in the array.
    element_count: UnsafeCell<u32>,
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
            cache: UnsafeCell::new(cache),
            capacity,
            window_start_index: UnsafeCell::new(0),
            window_size,
            dirty: UnsafeCell::new(false),
            reu_chunk,
            element_size,
            element_count: UnsafeCell::new(0),
        }
    }

    /// Number of elements currently stored in the array.
    pub fn len(&self) -> u32 {
        // SAFETY: `element_count` is only written through `&mut self` (push),
        // and this `&self` is exclusive of any concurrent write.
        unsafe { *self.element_count.get() }
    }

    /// Whether the array currently contains no elements.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Append an element. Panics if the fixed capacity is exhausted.
    pub fn push(&mut self, element: T) {
        let current_len = self.len();
        assert!(current_len < self.capacity, "REUArray capacity exceeded");
        self.ensure_in_cache(current_len, false);
        let cache_index = (current_len - self.window_start()) as usize;
        // SAFETY: cache pointer is non-null and `cache_index` is inside the window.
        unsafe { *self.cache_ptr().add(cache_index) = element };
        // SAFETY: exclusive access via `&mut self`.
        unsafe { *self.element_count.get() = current_len + 1 };
        self.mark_dirty();
    }

    /// Return a copy of an element, loading its cache window if needed.
    pub fn get(&mut self, index: u32) -> T {
        self.check_bounds(index);
        self.ensure_in_cache(index, true);
        let cache_index = (index - self.window_start()) as usize;
        // SAFETY: cache pointer is non-null and `cache_index` is inside the window.
        unsafe { *self.cache_ptr().add(cache_index) }
    }

    /// Replace an initialized element.
    pub fn set(&mut self, index: u32, value: T) {
        self.check_bounds(index);
        self.ensure_in_cache(index, true);
        let cache_index = (index - self.window_start()) as usize;
        // SAFETY: cache pointer is non-null and `cache_index` is inside the window.
        unsafe { *self.cache_ptr().add(cache_index) = value };
        self.mark_dirty();
    }

    /// Mutate an element without allowing a reference to escape the current cache window.
    pub fn with_mut<F>(&mut self, index: u32, mutate: F)
    where
        F: for<'a> FnOnce(&'a mut T),
    {
        self.check_bounds(index);
        self.ensure_in_cache(index, true);
        let cache_index = (index - self.window_start()) as usize;
        self.mark_dirty();
        // SAFETY: cache pointer is non-null and `cache_index` is inside the window.
        unsafe { mutate(&mut *self.cache_ptr().add(cache_index)) };
    }

    /// Visit each initialized element by value without exposing cache references.
    pub fn for_each<F>(&mut self, mut visit: F)
    where
        F: FnMut(u32, T),
    {
        let element_count = self.len();
        for index in 0..element_count {
            visit(index, self.get(index));
        }
    }

    /// Mutate every initialized element while keeping each cache borrow scoped to its callback.
    pub fn for_each_mut<F>(&mut self, mut mutate: F)
    where
        F: for<'a> FnMut(u32, &'a mut T),
    {
        let element_count = self.len();
        for index in 0..element_count {
            self.with_mut(index, |element| mutate(index, element));
        }
    }

    /// Iterate over initialized elements by value, yielding `T` for each.
    pub fn iter(&mut self) -> REUArrayIter<'_, T> {
        REUArrayIter { array: self, next_index: 0 }
    }

    /// Write pending cache changes back to REU memory.
    pub fn flush(&mut self) {
        if self.is_dirty() {
            self.prepare_slice();
            ram_expansion_unit::reu().push();
            // SAFETY: exclusive `&mut self` access.
            unsafe { *self.dirty.get() = false };
        }
    }

    /// Read-only access that loads the cache window for `index` if needed.
    ///
    /// Returns the element by value. There is no `&T` overload because
    /// returning a reference into the cache is unsound: the next `&self`
    /// operation on this `REUArray` may shift the cache window and invalidate
    /// the reference, but the compiler cannot reason about that.
    pub fn get_shared(&self, index: u32) -> T {
        self.check_bounds(index);
        // SAFETY: caller has `&self`; we mutate only the cache and the
        // window-tracking cells. We never expose aliased references.
        let mut_state = self as *const _ as *mut REUArray<T>;
        unsafe {
            (*mut_state).ensure_in_cache(index, true);
            let cache_index = (index - (*mut_state).window_start()) as usize;
            *(*mut_state).cache_ptr().add(cache_index)
        }
    }

    fn check_bounds(&self, index: u32) {
        assert!(index < self.len(), "REUArray index out of bounds");
    }

    fn ensure_in_cache(&mut self, index: u32, load: bool) {
        let start = self.window_start();
        let outside_window = index < start || index - start >= self.window_size;

        if outside_window {
            self.flush();
            // SAFETY: exclusive `&mut self` access.
            unsafe { *self.window_start_index.get() = index };

            if load {
                self.prepare_slice();
                ram_expansion_unit::reu().pull();
            }
        }
    }

    fn prepare_slice(&self) {
        let start = self.window_start();
        let element_count = min(self.window_size, self.len() - start);
        let byte_count = element_count * self.element_size as u32;
        ram_expansion_unit::reu().set_range(
            self.cache_ptr() as usize,
            self.reu_chunk.address() + start * self.element_size as u32,
            byte_count as usize,
        );
    }

    fn window_start(&self) -> u32 {
        // SAFETY: `window_start_index` is only written under `&mut self` or
        // inside `ensure_in_cache` (which is exclusive on this struct).
        unsafe { *self.window_start_index.get() }
    }

    fn is_dirty(&self) -> bool {
        // SAFETY: `dirty` is only written under `&mut self`.
        unsafe { *self.dirty.get() }
    }

    fn mark_dirty(&self) {
        // SAFETY: caller holds `&mut self`.
        unsafe { *self.dirty.get() = true };
    }

    fn cache_ptr(&self) -> *mut T {
        // SAFETY: `cache` is set at construction and never reassigned.
        unsafe { *self.cache.get() }
    }
}

impl<T: Copy> Drop for REUArray<T> {
    fn drop(&mut self) {
        self.flush();
        unsafe { free(self.cache_ptr() as *mut u8) };
    }
}

impl<T: Copy> Index<u32> for REUArray<T> {
    type Output = T;

    fn index(&self, index: u32) -> &Self::Output {
        // Loads the cache window for `index` and returns a reference into
        // the cache scratch buffer. The reference is only valid until the
        // next `&self` or `&mut self` operation on this `REUArray` (which
        // may shift the cache window). Callers must not hold `&T` returned
        // from `Index` across a method call that may move the window.
        let mut_state = self as *const _ as *mut REUArray<T>;
        unsafe {
            (*mut_state).ensure_in_cache(index, true);
            let cache_index = (index - (*mut_state).window_start()) as usize;
            &*(*mut_state).cache_ptr().add(cache_index)
        }
    }
}

impl<T: Copy> IndexMut<u32> for REUArray<T> {
    fn index_mut(&mut self, index: u32) -> &mut Self::Output {
        // Loads the cache window for `index` and returns a mutable
        // reference into the cache scratch buffer. The reference borrows
        // `&mut self`, so the window cannot shift while the borrow is live.
        self.check_bounds(index);
        self.ensure_in_cache(index, true);
        // The returned `&mut T` may be written through; mark the window dirty
        // so the change is flushed to REU when the window later slides.
        self.mark_dirty();
        let cache_index = (index - self.window_start()) as usize;
        // SAFETY: cache pointer is non-null and `cache_index` is inside
        // the window. The returned `&mut T` borrows from `&mut self`.
        unsafe { &mut *self.cache_ptr().add(cache_index) }
    }
}

impl<'a, T: Copy> IntoIterator for &'a mut REUArray<T> {
    type Item = T;
    type IntoIter = REUArrayIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// Iterator that yields elements of a `REUArray<T>` by value, in stored order.
pub struct REUArrayIter<'a, T: Copy> {
    array: &'a mut REUArray<T>,
    next_index: u32,
}

impl<'a, T: Copy> Iterator for REUArrayIter<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_index < self.array.len() {
            let value = self.array.get(self.next_index);
            self.next_index += 1;
            Some(value)
        } else {
            None
        }
    }
}

impl<T: Copy> ufmt::uDebug for REUArray<T> {
    fn fmt<W: ufmt::uWrite + ?Sized>(
        &self,
        f: &mut ufmt::Formatter<'_, W>,
    ) -> Result<(), W::Error> {
        f.write_str("ReuArray of ")?;
        self.len().fmt(f)?;
        f.write_str("\nl:")?;
        self.cache_ptr().fmt(f)?;
        f.write_str("  r:")?;
        self.reu_chunk.fmt(f)?;
        f.write_str("\nwindow=")?;
        self.window_start().fmt(f)?;
        f.write_str(" dirty=")?;
        self.is_dirty().fmt(f)?;
        Ok(())
    }
}