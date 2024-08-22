extern crate alloc;

use core::fmt;
use crate::ram_expansion_unit::RamExpanstionUnit;

const MEMORY_POOL_START: u32 = 0x012000;
const REU_MEMORY_END: u32 = 0x1000000;
const MEMORY_SIZE: u32 = REU_MEMORY_END - MEMORY_POOL_START; // 16 MB
const BLOCK_SIZE: usize = 256; // 256 bytes per block
const BLOCK_COUNT: usize = (MEMORY_SIZE / BLOCK_SIZE as u32) as usize;
const BITMAP_ADDRESS: usize = 0x4000;
const BITMAP_REU_ADDRESS: u32 = 0x010000;

static mut BOM: *mut u8 = 0xc000 as *mut u8;

#[derive(Clone, Copy)]
pub struct Ptr24 {
    pub address: u32,
    pub len: u32,
}

impl ufmt::uDebug for Ptr24 {
    fn fmt<W: ufmt::uWrite + ?Sized>(&self, f: &mut ufmt::Formatter<'_, W>) -> Result<(), W::Error> {
        f.write_str("Ptr24:")?;
        self.address.fmt(f)?;
        Ok(())
    }
}       

fn mark_occupied(index: usize) {
    let byte_index = index / 8;
    let bit_index = index % 8;
    unsafe {
        *BOM.add(byte_index) |= 1 << bit_index;
    }
}

fn mark_free(index: usize) {
    let byte_index = index / 8;
    let bit_index = index % 8;
    unsafe {
        *BOM.add(byte_index) &= !(1 << bit_index);
    }
}

fn is_free(index: usize) -> bool {
    let byte_index = index / 8;
    let bit_index = index % 8;
    unsafe {
        (*BOM.add(byte_index) & (1 << bit_index)) == 0
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
            panic!("reu 0 alloc");
        }

        // swap in BAM, it has to be swapped out before return
        self.prepare(BITMAP_ADDRESS, BITMAP_REU_ADDRESS, 8192);
        self.swap();

        let blocks_needed = count_blocks(size);
        let mut free_blocks = 0;
        let mut start_block = 0;

        for i in 0..(BLOCK_COUNT) {
            if is_free(i) {
                if free_blocks == 0 {
                    start_block = i;
                }
                free_blocks += 1;

                if free_blocks == blocks_needed {
                    for j in start_block..start_block + blocks_needed {
                        mark_occupied(j);
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
        panic!("out of reu memory");
    }

    // Deallocation function
    unsafe fn dealloc(&self, ptr: Ptr24) {
        if ptr.address == 0 {
            return;
        }

        let offset = ((ptr.address - MEMORY_POOL_START) / BLOCK_SIZE as u32) as usize;
        let blocks_needed = count_blocks(ptr.len);

        for i in offset..offset + blocks_needed {
            mark_free(i);
        }
    }
}
