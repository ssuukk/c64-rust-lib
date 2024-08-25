extern crate alloc;

use crate::ram_expansion_unit;
use crate::ram_expansion_unit::RamExpanstionUnit;

const REU_POOL_START: u32 = 0x012000;
const REU_POOL_END: u32 = 0x1000000;
const AVAILABLE_REU: u32 = REU_POOL_END - REU_POOL_START; // 16 MB
const ALLOCATION_UNIT: usize = 256; // minimal allocation unit = 256 bytes
const ALLOCATION_UNIT_COUNT: usize = (AVAILABLE_REU / ALLOCATION_UNIT as u32) as usize;
const BOM_RAM_ADDRESS: usize = 0x4000;
const BOM_REU_ADDRESS: u32 = 0x010000;
const BOM_SIZE: usize = ALLOCATION_UNIT_COUNT/8;

static mut BOM: *mut Bom = BOM_RAM_ADDRESS as *mut Bom;

pub struct Bom {
    bom: [u8; BOM_SIZE],
}

/// A chunk of REU memory with 24-bit addressing and length
///
/// Allocate it using ram_expansion_unit::reu()::alloc(size)
/// use push and pull to move memory blocks between RAM and REU
pub struct ReuChunk {
    pub address: u32,
    len: u32,
}

impl ReuChunk {
    /// Push C64 RAM contents into REU memory
    pub fn push(&self, reu: &RamExpanstionUnit, c64_start: usize) {
        reu.set_range(c64_start, self.address, self.len as usize);
        reu.push();
    }

    /// Pull data from REU memory into C64 RAM
    pub fn pull(&self, reu: &RamExpanstionUnit, c64_start: usize) {
        reu.set_range(c64_start, self.address, self.len as usize);
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
        self.address.fmt(f)?;
        f.write_char('(')?;
        self.len.fmt(f)?;
        f.write_char(')')?;
        Ok(())
    }
}

impl Bom {
    fn mark_occupied(&mut self, index: usize) {
        let byte_index = index / 8;
        let bit_index = index % 8;
        self.bom[byte_index] |= 1 << bit_index;
    }

    fn mark_free(&mut self, index: usize) {
        let byte_index = index / 8;
        let bit_index = index % 8;
        self.bom[byte_index] &= !(1 << bit_index);
    }

    fn is_free(&self, index: usize) -> bool {
        let byte_index = index / 8;
        let bit_index = index % 8;
        (self.bom[byte_index] & (1 << bit_index)) == 0
    }
}

fn as_blocks(size: u32) -> usize {
    ((size + ALLOCATION_UNIT as u32 - 1) / ALLOCATION_UNIT as u32) as usize
}

impl RamExpanstionUnit {
    /// Allocate a chunk of REU memory with given size
    pub unsafe fn alloc(&self, size: u32) -> ReuChunk {
        if size == 0 {
            panic!("reu 0 alloc");
        }

        // swap in BAM, it has to be swapped out before return
        self.set_range(BOM_RAM_ADDRESS, BOM_REU_ADDRESS, BOM_SIZE);
        self.swap();

        let blocks_needed = as_blocks(size);
        let mut free_blocks = 0;
        let mut start_block = 0;

        for i in 0..(ALLOCATION_UNIT_COUNT) {
            if (*BOM).is_free(i) {
                if free_blocks == 0 {
                    start_block = i;
                }
                free_blocks += 1;

                if free_blocks == blocks_needed {
                    for j in start_block..start_block + blocks_needed {
                        (*BOM).mark_occupied(j);
                    }
                    self.swap();
                    return ReuChunk {
                        address: REU_POOL_START + ((start_block * ALLOCATION_UNIT) as u32),
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
        let offset = ((ptr.address - REU_POOL_START) / ALLOCATION_UNIT as u32) as usize;
        let blocks_needed = as_blocks(ptr.len);

        for i in offset..offset + blocks_needed {
            (*BOM).mark_free(i);
        }
    }
}
