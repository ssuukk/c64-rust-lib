// tak się buduje:
// cargo build --target mos-c64-none 
// opcjonalnie dodać --release
// 
// kopiowanie wyniku:
// docker cp determined_williamson:/workspaces/c64-rust-lib/target/mos-c64-none/release/rust-mos-hello-world D:/temp/rust
// otwarcie terminala kontenera: ctrl+shift+`
// https://github.com/mlund/mos-hardware

#![no_std] // nie ładuj biblioteki std
#![feature(start)] // używamy niestandardowego entry pointu
#![feature(panic_info_message)]

extern crate mos_alloc;

use core::panic::PanicInfo; // struktura zawierająca info o panic
// use core::alloc::GlobalAlloc;
use ufmt_stdio::*; // stdio dla środowisk, które nie mają std

mod reu_tests;
mod plotek_tests;
mod ultimate_tests;

#[panic_handler] // wymagany w programach bez std
fn panic(info: &PanicInfo) -> ! {
    // Check if there's a payload (message) in the panic info
    
    if info.message().is_some() {
        // If the message is Some, we can safely use it
        println!("PANIC: with a message");
    } else {
        // Handle the case where the message is None
        println!("PANIC: occurred!");
    }
    loop {}
}

#[start] // atrybut oznaczający entrypoint
fn _main(_argc: isize, _argv: *const *const u8) -> isize {

    //change_border();

    //test_hires();
    reu_tests::test_reu_slice();

    //box_test();
    0
}
