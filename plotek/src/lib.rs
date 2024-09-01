#![no_std] // Don't use the standard library

pub mod cia2;

use cia2::{set_vic_bank, VicBankSelect};
use core::ptr::{read_volatile, write_volatile};
use mos_hardware::c64::vic2;
use mos_hardware::vic2::{ControlYFlags, ScreenBank};

pub static mut CLEAR_FUNC: fn(*mut u8, usize, u8) = default_clear;

fn default_clear(start_addr: *mut u8, size: usize, value: u8) {
    unsafe {
        for i in 0..size {
            write_volatile(start_addr.add(i), value);
        }
    }
}

pub trait CharMatrix {
    fn clear(&self, value: u8);
}

pub struct C64TextScreen;

impl C64TextScreen {
    unsafe fn buffer_ptr(&self) -> *mut u8 {
        self as *const _ as *mut u8
    }
}

impl CharMatrix for C64TextScreen {
    fn clear(&self, value: u8) {
        unsafe {
            let buffer = self.buffer_ptr();
            CLEAR_FUNC(buffer, 1000, value);
        }
    }
}

pub trait PixelMatrix {
    fn plot(&self, x: u16, y: u8);
    fn line(&self, start: (u16, u8), end: (u16, u8));
    fn clear(&self, value: u8);
}

pub struct C64HiresScreen;

impl C64HiresScreen {
    unsafe fn buffer_ptr(&self) -> *mut u8 {
        self as *const _ as *mut u8
    }
}

impl PixelMatrix for C64HiresScreen {
    fn clear(&self, value: u8) {
        unsafe {
            let buffer = self.buffer_ptr();
            CLEAR_FUNC(buffer, 8000, value);
        }
    }

    fn plot(&self, x: u16, y: u8) {
        unsafe {
            let buffer = self.buffer_ptr();
            let col = (x >> 3) << 3;
            let row = (y >> 3) as u16;
            let subrow = (y % 8) as u16;
            let byte_offset = col + 10 * (row << 5) + subrow;
            let bit_position = 7 - (x % 8);
            let byte_ptr = buffer.add(byte_offset as usize);
            let current_value = read_volatile(byte_ptr);
            write_volatile(byte_ptr, current_value | (1 << bit_position));
        }
    }

    fn line(&self, start: (u16, u8), end: (u16, u8)) {
        let mut x0 = start.0;
        let mut y0 = start.1;
        let x1 = end.0;
        let y1 = end.1;

        let dx = (x1 as i16 - x0 as i16).abs();
        let dy = (y1 as i16 - y0 as i16).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx - dy;

        loop {
            self.plot(x0, y0);

            if x0 == x1 && y0 == y1 {
                break;
            }

            let e2 = 2 * err;
            if e2 > -dy {
                err -= dy;
                x0 = (x0 as i16 + sx) as u16;
            }
            if e2 < dx {
                err += dx;
                y0 = (y0 as i16 + sy) as u8;
            }
        }
    }
}

pub fn hide() {
    unsafe {
        set_vic_bank(VicBankSelect::VIC_0000);
        vic2().screen_and_charset_bank.write(21);
        set_bitmap_mode(false);
    }
}

fn set_bitmap_mode(on: bool) {
    unsafe {
        // zrobić według tabeli na końcu: https://c64os.com/post/rethinkingthememmap
        let mut ctrl_reg_1 = vic2().control_y.read();
        ctrl_reg_1.set(ControlYFlags::BITMAP_MODE, on); // and bit 5 (BMM) must be set.
                                                        //ctrl_reg_1.set(ControlYFlags::EXTENDED_COLOR_MODE, false); // bit 6 (ECM) must be cleared
        vic2().control_y.write(ctrl_reg_1);
    }
}

pub fn show(bitmap_addr: VicBankSelect, text_addr: ScreenBank) {
    unsafe {
        set_vic_bank(bitmap_addr);
        set_bitmap_mode(true);
        //c64::vic2().control_x.write(); // bit 4 must be cleared. (MCM - multicolor)

        // 1 w bicie 3 daje hires w: start_vic + $2000
        let char_bank = text_addr.bits() | 1 << 3;
        vic2().screen_and_charset_bank.write(char_bank);
    }
}
