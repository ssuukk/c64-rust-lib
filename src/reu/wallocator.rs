extern crate alloc;

use crate::reu::RamExpanstionUnit;
use crate::reu::reu;
use alloc::{string::String, vec::Vec};

const MEMORY_POOL_START: u32 = 0x012000;
const REU_MEMORY_END: u32 = 0x1000000;
const MEMORY_SIZE: u32 = REU_MEMORY_END - MEMORY_POOL_START; // 16 MB
const BLOCK_SIZE: usize = 256; // 256 bytes per block
//const BITMAP_SIZE: usize = (BLOCK_COUNT / 8) as usize; // 8 KB for the bitmap
const BLOCK_COUNT: usize = (MEMORY_SIZE / BLOCK_SIZE as u32) as usize;
const BITMAP_ADDRESS: usize = 0x4000;
const BITMAP_REU_ADDRESS: u32 = 0x010000;

static mut BOM: *mut u8 = 0xc000 as *mut u8;

#[derive(Clone, Copy)]
pub struct Ptr24 {
    /// Address
    pub address: u32,
    /// Length in bytes
    pub len: u32,
}

impl Ptr24 {
    pub fn null_ptr() -> Self {
        Ptr24 {
            address: 0,
            len: 0,
        }
    }
}

impl From<Ptr24> for String {
    fn from(value: Ptr24) -> Self {
        // naughty camouflage of unsafe code...
        unsafe { Self::from_utf8_unchecked(value.into()) }
    }
}

impl From<Ptr24> for Vec<u8> {
    fn from(value: Ptr24) -> Self {
        MemoryIterator::new(value.address).get_chunk(value.len as usize)
    }
}

/// Never-ending iterator to lpeek into 28-bit memory
///
/// The address is automatically pushed forward with every byte read.
///
/// # Examples
/// ~~~
/// const ADDRESS: u32 = 0x40000;
/// let mut mem = MemoryIterator::new(ADDRESS);
/// let byte: u8 = mem.next().unwrap();
/// let v: Vec<u8> = mem.get_chunk(10);
/// for byte in mem.take(4) {
///     println!("{}", byte);
/// }
/// assert_eq!(mem.address, ADDRESS + 1 + 10 + 4);
/// ~~~
///
/// # Todo
///
/// This should eventually be submitted to the `mos-hardware` crate.
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct MemoryIterator {
    /// Next address
    pub address: u32,
}

impl MemoryIterator {
    pub const fn new(address: u32) -> Self {
        Self { address }
    }

    /// Peek `n` bytes using fast Direct Memory Access (DMA) copy
    ///
    /// # Todo
    ///
    /// - Check that the DMA copy works as expected
    #[allow(clippy::uninit_vec)]
    pub fn get_chunk(&mut self, n: usize) -> Vec<u8> {
        let mut dst = Vec::<u8>::with_capacity(n);
        unsafe {
            dst.set_len(n);
            //    src           dst                                 len
            //reu().lcopy(self.address, dst.as_mut_slice().as_ptr() as u32, n);
        }
        self.address += n as u32;
        dst
    }
}

impl Iterator for MemoryIterator {
    type Item = u8;
    fn next(&mut self) -> Option<Self::Item> {
        //let value = reu().lpeek(self.address);
        let value: u8 = 0;
        self.address += 1;
        Some(value)
    }

    // #[allow(clippy::uninit_assumed_init)]
    // fn next_chunk<const N: usize>(
    //     &mut self,
    // ) -> Result<[Self::Item; N], core::array::IntoIter<Self::Item, N>>
    // where
    //     Self: Sized,
    // {
    //     let dst: [Self::Item; N] = unsafe { MaybeUninit::uninit().assume_init() };
    //     unsafe {
    //         lcopy(self.address, dst.as_ptr() as u32, N);
    //     }
    //     self.address += N as u32;
    //     Ok(dst)
    // }

    // #[cfg(version("1.69"))]
    // fn advance_by(&mut self, n: usize) -> Result<(), core::num::NonZeroUsize> {
    //     self.address += n as u32;
    //     Ok(())
    // }
    // #[cfg(not(version("1.69")))]
    // fn advance_by(&mut self, n: usize) -> Result<(), usize> {
    //     self.address += n as u32;
    //     Ok(())
    // }
}



fn set_bit(index: usize) {
    let byte_index = index / 8;
    let bit_index = index % 8;
    unsafe {
        *BOM.add(byte_index) |= 1 << bit_index;
    }
}

fn clear_bit(index: usize) {
    let byte_index = index / 8;
    let bit_index = index % 8;
    unsafe {
        *BOM.add(byte_index) &= !(1 << bit_index);
    }
}

fn is_bit_set(index: usize) -> bool {
    let byte_index = index / 8;
    let bit_index = index % 8;
    unsafe {
        (*BOM.add(byte_index) & (1 << bit_index)) != 0
    }
}

fn count_blocks(size: u32) -> usize {
    ((size + BLOCK_SIZE as u32 - 1) / BLOCK_SIZE as u32) as usize
}


pub trait WAllocator {
    unsafe fn alloc(&self, size: u32) -> Ptr24;
    unsafe fn dealloc(&self, ptr: Ptr24);
}

impl WAllocator for RamExpanstionUnit {
    unsafe fn alloc(&self, size: u32) -> Ptr24 {
        if size == 0 {
            return Ptr24::null_ptr();
        }

        // swap in BAM, it has to be swapped out before return
        self.prepare(BITMAP_ADDRESS as u16, BITMAP_REU_ADDRESS, 8192);
        self.swap();

        let blocks_needed = count_blocks(size);
        let mut free_blocks = 0;
        let mut start_block = 0;

        for i in 0..(BLOCK_COUNT) {
            if !is_bit_set(i) {
                if free_blocks == 0 {
                    start_block = i;
                }
                free_blocks += 1;

                if free_blocks == blocks_needed {
                    for j in start_block..start_block + blocks_needed {
                        set_bit(j);
                    }
                    self.swap();
                    return Ptr24 {
                        address: MEMORY_POOL_START + ((start_block * BLOCK_SIZE) as u32),
                        len: size,
                    };
                }
            } else {
                free_blocks = 0;
            }
        }

        self.swap();
        Ptr24::null_ptr()
    }

    // Deallocation function
    unsafe fn dealloc(&self, ptr: Ptr24) {
        if ptr.address == 0 {
            return;
        }

        let offset = ((ptr.address - MEMORY_POOL_START) / BLOCK_SIZE as u32) as usize;
        let blocks_needed = count_blocks(ptr.len);

        for i in offset..offset + blocks_needed {
            clear_bit(i);
        }
    }
}