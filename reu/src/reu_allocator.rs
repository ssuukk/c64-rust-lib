extern crate alloc;

use crate::ram_expansion_unit;
use crate::ram_expansion_unit::RamExpanstionUnit;

const MEMORY_POOL_START: u32 = 0x012000;
const REU_MEMORY_END: u32 = 0x1000000;
const MEMORY_SIZE: u32 = REU_MEMORY_END - MEMORY_POOL_START; // 16 MB
const BLOCK_SIZE: usize = 256; // 256 bytes per block
const BLOCK_COUNT: usize = (MEMORY_SIZE / BLOCK_SIZE as u32) as usize;
const BITMAP_ADDRESS: usize = 0x4000;
const BITMAP_REU_ADDRESS: u32 = 0x010000;

static mut BOM: *mut u8 = 0xc000 as *mut u8;

/// A chunk of REU memory with 24-bit addressing and length
///
/// Allocate it using ram_expansion_unit::reu()::alloc(size)
pub struct ReuChunk {
    pub address: u32,
    pub len: u32,
}

impl ReuChunk {
    /// Push C64 RAM contents into REU memory
    pub fn push(&self, reu: &RamExpanstionUnit, c64_start: usize) {
        reu.set_range(c64_start, self.address, self.len as u16);
        reu.push();
    }

    /// Pull data from REU memory into C64 RAM
    pub fn pull(&self, reu: &RamExpanstionUnit, c64_start: usize) {
        reu.set_range(c64_start, self.address, self.len as u16);
        reu.pull();
    }
}

impl Drop for ReuChunk {
    fn drop(&mut self) {
        unsafe {
            ram_expansion_unit::reu().dealloc(&self);
        }
    }
}

impl ufmt::uDebug for ReuChunk {
    fn fmt<W: ufmt::uWrite + ?Sized>(
        &self,
        f: &mut ufmt::Formatter<'_, W>,
    ) -> Result<(), W::Error> {
        f.write_str("ReuChunk:")?;
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
    unsafe { (*BOM.add(byte_index) & (1 << bit_index)) == 0 }
}

fn count_blocks(size: u32) -> usize {
    ((size + BLOCK_SIZE as u32 - 1) / BLOCK_SIZE as u32) as usize
}

impl RamExpanstionUnit {
    /// Allocate a chunk of REU memory with given size
    pub unsafe fn alloc(&self, size: u32) -> ReuChunk {
        if size == 0 {
            panic!("reu 0 alloc");
        }

        // swap in BAM, it has to be swapped out before return
        self.set_range(BITMAP_ADDRESS, BITMAP_REU_ADDRESS, 8192);
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
                    return ReuChunk {
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

    /// Deallocation of REU chunk
    unsafe fn dealloc(&self, ptr: &ReuChunk) {
        let offset = ((ptr.address - MEMORY_POOL_START) / BLOCK_SIZE as u32) as usize;
        let blocks_needed = count_blocks(ptr.len);

        for i in offset..offset + blocks_needed {
            mark_free(i);
        }
    }
}
