use mos_hardware::vic2::ScreenBank; // albo crate::vic2:ScreenBank
use mos_hardware::{c64, cbm_kernal, vic2};
use plotek::cia2;
use plotek::{C64HiresScreen, C64TextScreen, CharMatrix, PixelMatrix};

const HIRES: *const C64HiresScreen = (0x2000) as _;
const COLORS: *const C64TextScreen = (1024) as _;

pub fn test_hires() {
    unsafe {
        plotek::show(cia2::VicBankSelect::VIC_0000, ScreenBank::AT_0400);
        (*HIRES).clear(0);
        (*COLORS).clear(vic2::GREEN << 4 & vic2::RED);

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
