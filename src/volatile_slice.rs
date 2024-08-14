use core::slice;

pub struct VolatileSlice {
    ptr: *mut u8,
    len: usize,
}

impl VolatileSlice {
    pub unsafe fn new(ptr: *mut u8, len: usize) -> Self {
        VolatileSlice { ptr, len }
    }

    pub fn read(&self, index: usize) -> u8 {
        assert!(index < self.len);
        unsafe { core::ptr::read_volatile(self.ptr.add(index)) }
    }

    pub fn write(&mut self, index: usize, value: u8) {
        assert!(index < self.len);
        unsafe { core::ptr::write_volatile(self.ptr.add(index), value) }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn as_ptr(&self) -> *const u8 {
        self.ptr
    }

    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.ptr
    }

    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut::new(self)
    }
}
