use mos_hardware::vic2::ScreenBank; // albo crate::vic2:ScreenBank
use plotek::{ C64TextScreen, PixelMatrix, C64HiresScreen, CharMatrix };
use plotek::cia2;
use mos_hardware::{c64, vic2, cbm_kernal};

pub const HIRES: *const C64HiresScreen = (0xa000) as _;
//pub const SCREEN: &C64HiresScreen = unsafe { &*HIRES_SCREEN };
pub const COLORS: *const C64TextScreen = (0x8400) as _;

pub fn change_border(col: u8) {
    let vic = c64::vic2();
    unsafe {
        vic.border_color.write(col);
    }
}

fn wait_for_return() {
    unsafe {
        cbm_kernal::cbm_k_chrin();
    }
}

pub fn test_hires() {
    unsafe {
        change_border(vic2::RED);
        (*HIRES).clear(0);
        (*COLORS).clear(33);
        //reu::reu().fill(0xa000, 8000, 0);
        plotek::show(cia2::VicBankSelect::VIC_8000, ScreenBank::AT_0400);

        for i in 0..32 {
            (*HIRES).line((0,0),(i*10,199));
        }
    
        wait_for_return();

        plotek::hide();
        change_border(vic2::LIGHT_BLUE);
    }
}
