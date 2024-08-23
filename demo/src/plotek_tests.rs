use mos_hardware::vic2::ScreenBank; // albo crate::vic2:ScreenBank
use mos_hardware::{c64, cbm_kernal, vic2};
use plotek::cia2;
use plotek::{C64HiresScreen, C64TextScreen, CharMatrix, PixelMatrix};

pub const HIRES: *const C64HiresScreen = (0xa000) as _;
//pub const SCREEN: &C64HiresScreen = unsafe { &*HIRES_SCREEN };
pub const COLORS: *const C64TextScreen = (0x8400) as _;

pub fn test_hires() {
    unsafe {
        change_border(vic2::RED);
        (*HIRES).clear(0);
        (*COLORS).clear(vic2::GREEN << 4 & vic2::RED);
        //reu::reu().fill(0xa000, 8000, 0);
        plotek::show(cia2::VicBankSelect::VIC_8000, ScreenBank::AT_0400);

        for i in 0..32 {
            (*HIRES).line((0, 0), (i * 10, 199));
            change_border(i as u8);
        }

        wait_for_return();

        plotek::hide();
        change_border(vic2::LIGHT_BLUE);
    }
}

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
