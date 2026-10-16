use mos_hardware::cx16::COLOR_BLACK;
use mos_hardware::vic2::ScreenBank; // albo crate::vic2:ScreenBank
use mos_hardware::{c64, cbm_kernal};
use core::ptr::{read_volatile, write_volatile};
use plotek::cia2;
use plotek::{C64HiresScreen, C64TextScreen, CharMatrix, PixelMatrix};

pub fn test_hires() {
    const HIRES: *const C64HiresScreen = 0x2000 as _;
    const COLORS: *const C64TextScreen = 1024 as _;

    let vic = c64::vic2();

    unsafe {
        run_screen_tests(HIRES, COLORS);
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

unsafe fn run_screen_tests(hires: *const C64HiresScreen, colors: *const C64TextScreen) {
    const BITMAP_START: usize = 0x2000;
    const BITMAP_SIZE: usize = 8000;
    const TEXT_START: usize = 0x0400;
    const TEXT_SIZE: usize = 1000;

    write_volatile((BITMAP_START + BITMAP_SIZE) as *mut u8, 0xa5);
    (*hires).clear(0x5a);
    assert!(read_volatile(BITMAP_START as *const u8) == 0x5a);
    assert!(read_volatile((BITMAP_START + BITMAP_SIZE - 1) as *const u8) == 0x5a);
    assert!(read_volatile((BITMAP_START + BITMAP_SIZE) as *const u8) == 0xa5);

    write_volatile((TEXT_START + TEXT_SIZE) as *mut u8, 0xa5);
    (*colors).clear(32);
    assert!(read_volatile(TEXT_START as *const u8) == 32);
    assert!(read_volatile((TEXT_START + TEXT_SIZE - 1) as *const u8) == 32);
    assert!(read_volatile((TEXT_START + TEXT_SIZE) as *const u8) == 0xa5);

    (*hires).clear(0);
    (*hires).plot(0, 0);
    (*hires).plot(7, 0);
    (*hires).plot(8, 0);
    (*hires).plot(1, 0);
    (*hires).plot(319, 199);
    assert!(read_volatile(BITMAP_START as *const u8) == 0xc1);
    assert!(read_volatile((BITMAP_START + 8) as *const u8) == 0x80);
    assert!(read_volatile((BITMAP_START + BITMAP_SIZE - 1) as *const u8) == 0x01);

    (*hires).clear(0);
    (*hires).line((2, 2), (7, 2));
    assert!(read_volatile((BITMAP_START + 2) as *const u8) == 0x3f);

    (*hires).clear(0);
    (*hires).line((3, 3), (0, 0));
    assert!(read_volatile(BITMAP_START as *const u8) == 0x80);
    assert!(read_volatile((BITMAP_START + 1) as *const u8) == 0x40);
    assert!(read_volatile((BITMAP_START + 2) as *const u8) == 0x20);
    assert!(read_volatile((BITMAP_START + 3) as *const u8) == 0x10);

    (*hires).clear(0);
    (*hires).line((0, 7), (0, 8));
    assert!(read_volatile((BITMAP_START + 7) as *const u8) == 0x80);
    assert!(read_volatile((BITMAP_START + 320) as *const u8) == 0x80);
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
