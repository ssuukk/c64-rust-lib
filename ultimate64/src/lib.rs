#![no_std] // nie ładuj biblioteki std
#![feature(panic_info_message)]

use bitflags::bitflags;
use volatile_register::RW;

// https://github.com/6510nl/48MHz

pub const ULTIMATE: *const U64TurboRegs = (0xD030) as _;

pub fn get() -> &'static U64TurboRegs {
    unsafe { &*ULTIMATE }
}

#[repr(C, packed)]
pub struct U64TurboRegs {
    pub turbo_enable: RW<u8>,     // D030
    pub turbo_control: RW<u8>,    // D031
}

// for turbo_enable
bitflags! {
    pub struct Turbo: u8 {
        const ENABLE = 0b0000_0001;
        const DISABLE = 0b0100_0000;
    }
}

// for turbo_control
bitflags! {
    pub struct Timings: u8 {
        const BADLINE_ENABLE = 0b1000_0000;
        const MHZ_1 = 0;
        const MHZ_2 = 1;
        const MHZ_48 = 15;
    }
}

impl U64TurboRegs {
    pub fn set_speed(&self, speed: u8) {
        unsafe {        
            self.turbo_control.write(speed);
        }
    }

    pub fn set_enable(&self, value: u8) {
        unsafe {
            self.turbo_enable.write(value);
        }
    }
}
