extern crate alloc;

use crate::ram_expansion_unit;
use crate::ram_expansion_unit::{Command, RamExpanstionUnit};
use crate::vectors::INTERRUPT_VECTORS;
use mos_hardware::{c64, cbm_kernal, vic2};
use ufmt_stdio::println; // stdio dla środowisk, które nie mają std

use mos_hardware::c64::{CpuPortFlags, CPU_PORT};

const REU_POOL_START: u32 = 0x012000;
const REU_POOL_END: u32 = 0x1000000;
const AVAILABLE_REU: u32 = REU_POOL_END - REU_POOL_START; // 16 MB
const ALLOCATION_UNIT: usize = 256; // minimal allocation unit = 256 bytes
const MAX_ALLOCATION_UNIT_COUNT: u32 = AVAILABLE_REU / ALLOCATION_UNIT as u32;
const BOM_RAM_ADDRESS: usize = 0xE000;
const BOM_REU_ADDRESS: u32 = 0x010000;
const BOM_SIZE: usize = (MAX_ALLOCATION_UNIT_COUNT / 8) as usize;

static mut BOM: *mut Bom = BOM_RAM_ADDRESS as *mut Bom;
static mut ALLOCATION_UNIT_COUNT: u32 = 0;

pub struct Bom {
    bom: [u8; BOM_SIZE],
}

/// A chunk of REU memory with 24-bit addressing and length
///
/// Allocate it using ram_expansion_unit::reu()::alloc(size)
/// use push and pull to move memory blocks between RAM and REU
pub struct ReuChunk {
    address: u32,
    len: u32,
}

impl ReuChunk {
    /// Start address of this chunk in REU memory.
    pub fn address(&self) -> u32 {
        self.address
    }

    /// Push C64 RAM contents into REU memory
    pub fn push(&self, reu: &RamExpanstionUnit, c64_start: usize) {
        assert!(self.len <= 0x1_0000, "REU chunk exceeds C64 DMA address space");
        reu.set_range(c64_start, self.address, self.len as usize);
        reu.push();
    }

    /// Pull data from REU memory into C64 RAM
    pub fn pull(&self, reu: &RamExpanstionUnit, c64_start: usize) {
        assert!(self.len <= 0x1_0000, "REU chunk exceeds C64 DMA address space");
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
    fn mark_occupied(&mut self, index: u32) {
        let byte_index = (index / 8) as usize;
        let bit_index = index % 8;
        self.bom[byte_index] |= 1u8 << bit_index;
    }

    fn mark_free(&mut self, index: u32) {
        let byte_index = (index / 8) as usize;
        let bit_index = index % 8;
        self.bom[byte_index] &= !(1u8 << bit_index);
    }

    fn is_free(&self, index: u32) -> bool {
        let byte_index = (index / 8) as usize;
        let bit_index = index % 8;
        (self.bom[byte_index] & (1u8 << bit_index)) == 0
    }
}

fn as_blocks(size: u32) -> u32 {
    let unit = ALLOCATION_UNIT as u32;
    let blocks = size / unit;
    blocks + u32::from(size % unit != 0)
}

impl RamExpanstionUnit {
    /// Initialize the allocator for the installed REU capacity in bytes.
    pub fn init_allocator(&self, reu_capacity: u32) {
        assert!(
            reu_capacity >= REU_POOL_START + ALLOCATION_UNIT as u32,
            "REU is too small for the allocator"
        );
        assert!(reu_capacity <= REU_POOL_END, "REU capacity exceeds 16 MiB");

        let block_count = (reu_capacity - REU_POOL_START) / ALLOCATION_UNIT as u32;
        unsafe {
            ALLOCATION_UNIT_COUNT = block_count;
        }
        self.fill_reu(BOM_REU_ADDRESS, BOM_SIZE, 0);
    }

    /// Allocate a chunk of REU memory with given size
    pub fn alloc(&self, size: u32) -> ReuChunk {
        if size == 0 {
            panic!("reu 0 alloc");
        }

        let blocks_needed = as_blocks(size);
        let block_count = unsafe { ALLOCATION_UNIT_COUNT };
        assert!(block_count > 0, "initialize the REU allocator first");
        assert!(blocks_needed <= block_count, "out of reu memory");

        self.swap_bom_in();

        let mut free_blocks = 0;
        let mut start_block = 0;

        'outer: for i in 0..block_count {
            unsafe {
                if (*BOM).is_free(i) {
                    if free_blocks == 0 {
                        start_block = i;
                    }
                    free_blocks += 1;

                    if free_blocks == blocks_needed {
                        for j in start_block..start_block + blocks_needed {
                            (*BOM).mark_occupied(j);
                        }
                        break 'outer;
                    }
                } else {
                    //c64::vic2().border_color.write(free_blocks as u8);
                    free_blocks = 0;
                }
            }
        }

        self.swap_bom_out();

        if free_blocks != blocks_needed {
            panic!("out of reu memory");
        } else {
            return ReuChunk {
                address: REU_POOL_START + start_block * ALLOCATION_UNIT as u32,
                len: size,
            };
        }
    }

    /// Deallocation of REU chunk
    unsafe fn dealloc(&self, ptr: &ReuChunk) {
        let offset = (ptr.address - REU_POOL_START) / ALLOCATION_UNIT as u32;
        let blocks_needed = as_blocks(ptr.len);

        self.swap_bom_in();
        for i in offset..offset + blocks_needed {
            (*BOM).mark_free(i);
        }
        self.swap_bom_out();
    }

    fn swap_bom_in(&self) {
        // step aside, Kernal!
        unsafe {
            crate::__disable_mi();
            (*CPU_PORT).write(CpuPortFlags::RAM_IO_RAM);
        }
        self.set_range(BOM_RAM_ADDRESS, BOM_REU_ADDRESS, BOM_SIZE);
        self.swap();
    }

    fn swap_bom_out(&self) {
        // write modified BOM back to REU
        self.swap();
        // restore KERNAL
        unsafe {
            (*CPU_PORT).write(CpuPortFlags::RAM_IO_KERNAL);
            crate::__enable_mi();
        }
    }
}
