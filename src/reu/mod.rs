use bitflags::bitflags;
use static_assertions::const_assert;
use volatile_register::{RO, RW};
use core::mem::size_of;

pub mod wallocator;
pub mod custom_slice;

pub const REU: *const RamExpanstionUnit = (0xDF00) as _;

pub fn reu() -> &'static RamExpanstionUnit {
    unsafe { &*REU }
}

// https://www.codebase64.org/doku.php?id=base:reu_registers
#[repr(C, packed)]
pub struct RamExpanstionUnit {
    pub status: RO<u8>,     // DF00
    pub command: RW<u8>,    // DF01
    pub c64_start: RW<u16>,     // DF02-3
    pub reu_start_l: RW<u8>, // DF04-6 reu start - 3 bajty LSB, MSB, MOST SB
    pub reu_start_m: RW<u8>,
    pub reu_start_h: RW<u8>,
    pub length: RW<u16>,        // DF07-8
    pub interrupt_mask: RW<u8>, // DF09
    pub address_control: RW<u8>, // DF0A
}

bitflags! {
    pub struct Status: u8 {
        const INTERRUPT_PENDING = 0b1000_0000;
        const END_OF_BLOCK = 0b0100_0000;
        const FAULT = 0b0010_0000;
        const SIZE = 0b0001_0000;
    }
}

bitflags! {
    pub struct Command: u8 {
        const EXECUTE = 0b1000_0000;
        const AUTOLOAD = 0b0010_0000;
        const NO_FF00_DECODE = 0b0001_0000;
        const TO_REU = 0b0000_0000;
        const FROM_REU = 0b0000_0001;
        const SWAP = 0b0000_0010;
        const VERIFY = 0b0000_0011;
    }
}

bitflags! {
    pub struct Control: u8 {
        const FIX_C64 = 0b1000_0000;
        const FIX_REU = 0b0100_0000;
        const NONE = 0b0000_0000;
    }
}

const_assert!(size_of::<RamExpanstionUnit>() == 11);

impl RamExpanstionUnit {
    #[allow(dead_code)]
    pub fn prepare(&self, c64_start: u16, reu_start: u32, length: u16) {
        unsafe {
            self.address_control.write(Control::NONE.bits());
            self.c64_start.write(c64_start);
            self.reu_start_l.write((reu_start & 0xFF) as u8);        // LSB
            self.reu_start_m.write(((reu_start >> 8) & 0xFF) as u8);  // MSB
            self.reu_start_h.write(((reu_start >> 16) & 0xFF) as u8); // MOST SB
            self.length.write(length);
        }
    }

    #[allow(dead_code)]
    pub fn swap_in(&self) {
        unsafe {
            self.command.write(Command::EXECUTE.bits() | Command::FROM_REU.bits() | Command::NO_FF00_DECODE.bits() | Command::AUTOLOAD.bits());
        }
    }

    #[allow(dead_code)]
    pub fn swap_out(&self) {
        unsafe {
            self.command.write(Command::EXECUTE.bits() | Command::TO_REU.bits() | Command::NO_FF00_DECODE.bits() | Command::AUTOLOAD.bits());
        }
    }

    #[allow(dead_code)]
    pub fn swap(&self) {
        unsafe {
            self.command.write(Command::EXECUTE.bits() | Command::SWAP.bits() | Command::NO_FF00_DECODE.bits());
        }
    }

    #[allow(dead_code)]
    pub fn fill(&self, c64_start: u16, length: u16, value: u8) {
        let temp = value;
        let temp_ptr = &temp as *const u8;
        let address_as_u16 = temp_ptr as u16;

        unsafe {
            self.prepare(address_as_u16, 0, 1);
            self.command.write(Command::EXECUTE.bits() | Command::TO_REU.bits() | Command::NO_FF00_DECODE.bits());
            self.prepare(c64_start, 0, length);
            self.address_control.write(Control::FIX_REU.bits());
            self.command.write(Command::EXECUTE.bits() | Command::FROM_REU.bits() | Command::NO_FF00_DECODE.bits());
        }
    }
}
