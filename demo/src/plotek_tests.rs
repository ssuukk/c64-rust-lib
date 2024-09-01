use mos_hardware::cx16::COLOR_BLACK;
use mos_hardware::vic2::ScreenBank; // albo crate::vic2:ScreenBank
use mos_hardware::{c64, cbm_kernal};
use plotek::cia2;
use plotek::{C64HiresScreen, C64TextScreen, CharMatrix, PixelMatrix};

pub fn test_hires() {
    const HIRES: *const C64HiresScreen = 0x2000 as _;
    const COLORS: *const C64TextScreen = 1024 as _;

    let vic = c64::vic2();

    unsafe {
        vic.border_color.write(COLOR_BLACK);

        (*COLORS).clear(32);

        plotek::show(cia2::VicBankSelect::VIC_0000, ScreenBank::AT_0400);

        (*HIRES).clear(0);

        for i in 0..32 {
            (*HIRES).line((0, 0), (i * 10, 199));
        }
    }

    wait_for_return();

    plotek::hide();
}

fn wait_for_return() {
    unsafe {
        cbm_kernal::cbm_k_chrin();
    }
}
