use mos_hardware::cx16::COLOR_BLACK;
use mos_hardware::vic2::ScreenBank; // albo crate::vic2:ScreenBank
use mos_hardware::{c64, cbm_kernal};
use plotek::cia2;
use plotek::{C64HiresScreen, C64TextScreen, CharMatrix, PixelMatrix, CLEAR_FUNC};
use reu::ram_expansion_unit;

fn reu_clear(start_addr: *mut u8, size: usize, value: u8) {
    ram_expansion_unit::reu().fill(start_addr as usize, size, value);
}

pub fn test_hires() {
    const HIRES: *const C64HiresScreen = 0x2000 as _;
    const COLORS: *const C64TextScreen = 1024 as _;

    let vic = c64::vic2();

    unsafe {
        CLEAR_FUNC = reu_clear;
        vic.border_color.write(COLOR_BLACK);

        (*COLORS).clear(32);

        plotek::show(cia2::VicBankSelect::VIC_0000, ScreenBank::AT_0400);

        (*HIRES).clear(0);

        for i in 0..40 {
            (*HIRES).line((0, 0), (i * 8, 199));
        }
    }

    wait_for_return();

    plotek::hide();
}

pub fn raster() {
    const TRIGGER_LINE: u8 = 100;
    c64::hardware_raster_irq(TRIGGER_LINE);
}

#[no_mangle]
pub extern "C" fn called_every_frame() {
    plotek::show(cia2::VicBankSelect::VIC_0000, ScreenBank::AT_0400);
    loop {
        if c64::vic2().raster_counter.read() > 200 {
            break;
        }
    }
    plotek::hide();
}

fn wait_for_return() {
    unsafe {
        cbm_kernal::cbm_k_chrin();
    }
}
