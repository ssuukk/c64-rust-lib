// tak się buduje:
// cargo build --target mos-c64-none
// opcjonalnie dodać --release
//
// kopiowanie wyniku:
// docker cp upbeat_hawking:/workspaces/c64-rust-lib/target/mos-c64-none/release/demo D:/temp/demo
// otwarcie terminala kontenera: ctrl+shift+`
// https://github.com/mlund/mos-hardware

#![no_std] // nie ładuj biblioteki std
#![no_main]

extern crate mos_alloc;

use core::panic::PanicInfo; // struktura zawierająca info o panic
                            // use core::alloc::GlobalAlloc;
use ufmt_stdio::*; // stdio dla środowisk, które nie mają std

mod plotek_tests;
mod reu_tests;
mod ultimate_tests;

use core::fmt::{self, Write};

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

        let _ = writeln!(writer, "?{} panic", info.message());

    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn main() -> core::ffi::c_int {
    #[cfg(feature = "reu-hardware-tests")]
    {
        reu_tests::test_reu_allocator();
        reu_tests::test_reu_array();
        println!("REU tests passed");
    }

    #[cfg(not(feature = "reu-hardware-tests"))]
    plotek_tests::test_hires();
    0
}
