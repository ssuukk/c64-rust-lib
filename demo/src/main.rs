// tak się buduje:
// cargo build --target mos-c64-none
// opcjonalnie dodać --release
//
// kopiowanie wyniku:
// docker cp upbeat_hawking:/workspaces/c64-rust-lib/target/mos-c64-none/release/demo D:/temp/demo
// otwarcie terminala kontenera: ctrl+shift+`
// https://github.com/mlund/mos-hardware

#![no_std] // nie ładuj biblioteki std
#![feature(start)] // używamy niestandardowego entry pointu
#![feature(panic_info_message)]

extern crate mos_alloc;

use core::panic::PanicInfo; // struktura zawierająca info o panic
                            // use core::alloc::GlobalAlloc;
use ufmt_stdio::*; // stdio dla środowisk, które nie mają std

mod plotek_tests;
mod reu_tests;
mod ultimate_tests;

use core::fmt::{self, Write};
use reu::ram_expansion_unit;

struct SimpleWriter;

impl Write for SimpleWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        print!("{}", s);
        Ok(())
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let mut writer = SimpleWriter;

    if let Some(message) = info.message() {
        let _ = writeln!(writer, "?{} panic", message);
    } else {
        let _ = writeln!(writer, "?panic");
    }

    loop {}
}

#[start] // atrybut oznaczający entrypoint
fn _main(_argc: isize, _argv: *const *const u8) -> isize {
    // ram_expansion_unit::reu().init_allocator();
    // reu_tests::test_reu_array();
    // reu_tests::alloc_test();
    plotek_tests::test_hires();

    0
}
