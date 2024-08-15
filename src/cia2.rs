use bitflags::bitflags;
use core::mem::size_of;
use static_assertions::const_assert;
use volatile_register::RW;
use mos_hardware::cia::TimeOfDay;

// https://sta.c64.org/cbm64mem.html

pub const CIA2: *const MOSComplexInterfaceAdapter6526_2 = (0xdd00) as _;

/// Get reference to CIA2 chip
pub const fn cia2() -> &'static MOSComplexInterfaceAdapter6526_2 {
    unsafe { &*CIA2 }
}


bitflags! {
    #[derive(Clone, Copy)]
    pub struct VicBankSelect: u8 {
        const VIC_C000 = 0b0000_0000;
        const VIC_8000 = 0b0000_0001;
        const VIC_4000 = 0b0000_0010;
        const VIC_0000 = 0b0000_0011;
   }
}

bitflags! {
    #[derive(Clone, Copy)]
    pub struct SerialBusAccess: u8 {
        const TXD_LO   = 0b0000_0100;
        const ATNO_LO  = 0b0000_1000;
        const CLKO_LO  = 0b0001_0000;
        const DTAO_LO  = 0b0010_0000;
        const CLKO_HI  = 0b0100_0000;
        const DATI_HI  = 0b1000_0000;
    }
}


bitflags! {
    #[derive(Clone, Copy)]
    pub struct RS232Access: u8 {
    }
}

#[repr(C, packed)]
pub struct MOSComplexInterfaceAdapter6526_2 {
    pub port_a: RW<VicBankSelect>,   // 0x00
    pub port_b: RW<RS232Access>,       // 0x01
    pub data_direction_port_a: RW<u8>, // 0x02
    pub data_direction_port_b: RW<u8>, // 0x03
    pub timer_a: RW<u16>,              // 0x04
    pub timer_b: RW<u16>,              // 0x06
    pub time_of_day: TimeOfDay,        // 0x08
    pub serial_shift: RW<u8>,          // 0x0c
    pub interrupt: RW<u8>,             // 0x0d
    pub control_a: RW<u8>,             // 0x0e
    pub control_b: RW<u8>,             // 0x0f
}

const_assert!(size_of::<MOSComplexInterfaceAdapter6526_2>() == 16);

pub fn set_vic_bank(bank: VicBankSelect) {
    unsafe {
        let dir_a = cia2().data_direction_port_a.read();
        let port_a = cia2().port_a.read();
        cia2().data_direction_port_a.write(dir_a | 0b11);
        cia2().port_a.write(port_a & VicBankSelect::VIC_0000.complement() | bank);
    }
}