use mos_hardware::vic2::{ ScreenBank, CharsetBank, ControlYFlags };
use crate::c64::vic2;

pub trait Drawable {
    fn plot(&self, x: u16, y: u8);
    fn line(&self, start: (u16, u8), end: (u16, u8));
    fn clear(&self, value: u8);
    fn show(&self, on: bool);
}

pub struct EightBitCanvas {
}

impl Drawable for EightBitCanvas {
    fn show(&self, on: bool) {
        unsafe {
            let mut ctrl_reg_1 = vic2().control_y.read();
            ctrl_reg_1.set(ControlYFlags::BITMAP_MODE, on);
            vic2().control_y.write(ctrl_reg_1); // bit 6 (ECM) must be cleared and bit 5 (BMM) must be set.
            //c64::vic2().control_x.write(); // bit 4 must be cleared. (MCM - multicolor)
    
            let address = self as *const _ as u16;
            let char_bank = if on { CharsetBank::AT_3800 } else { CharsetBank::DEFAULT };

            let bank = ScreenBank::from_address(address).bits() | char_bank.bits();
            vic2().screen_and_charset_bank.write(bank);
        }
    }

    fn plot(&self, x: u16, y: u8) {
        let col = (x >> 3) << 3;
        let row = (y >> 3) as u16;
        let subrow = (y % 8) as u16;
        let byte_offset = col + 10 * (row << 5) + subrow;
        // Calculate the bit position within the byte
        let bit_position = 7 - (x % 8);

        // Set the pixel by setting the corresponding bit
        let start_addr = self as *const _ as *mut u8;
        unsafe {
            let byte_ptr = start_addr.add(byte_offset as usize);
            *byte_ptr |= 1 << bit_position;
        }
    }

    fn clear(&self, value: u8) {
        // Start address of the canvas memory
        let start_addr = self as *const _ as *mut u8;

        unsafe {
            for i in 0..8000 {
                *start_addr.add(i) = value;
            }
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
