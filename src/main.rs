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
//#![allow(unused_imports)] // nie pulta się
#![feature(panic_info_message)]

extern crate mos_alloc;

use core::panic::PanicInfo; // struktura zawierająca info o panic
use core::slice;
use reu::custom_slice;
use reu::wallocator::{ WAllocator, Ptr24 };
use ufmt_stdio::*; // stdio dla środowisk, które nie mają std
use mos_hardware::{c64, vic2, poke, cbm_kernal};
use mos_hardware::vic2::ScreenBank; // albo crate::vic2:ScreenBank
use plotek::{ C64TextScreen, PixelMatrix, C64HiresScreen, CharMatrix };
use cia2::VicBankSelect;


// jeśli tu nie będzie tych modów, to te pliki nie będą widoczne w całym kodzie!
mod reu;
mod ultimate_64;
mod plotek;
mod cia2;

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

macro_rules! mem_array {
    ($var_name:ident, $start_addr:expr, $size:expr) => {
        let $var_name: &mut [u8] = unsafe {
            slice::from_raw_parts_mut($start_addr as *mut u8, $size)
        };    
    }
}

macro_rules! volatile_mem_array {
    ($var_name:ident, $start_addr:expr, $size:expr) => {

        let $var_name: &mut [Volatile<u8>] = core::slice::from_raw_parts_mut($start_addr as *mut Volatile<u8>, $size);


        // let $var_name: &mut [Volatile<u8>] = unsafe {
        //     slice::from_raw_parts_mut($start_addr as *mut Volatile<u8>, $size)
        // };    
    }
}

pub fn turbo() {
    ultimate_64::get().set_speed(ultimate_64::Timings::MHZ_48.bits());
    ultimate_64::get().set_enable(ultimate_64::Turbo::ENABLE.bits());
}

fn change_border() {
    let vic = c64::vic2();
    unsafe {
        vic.border_color.write(vic2::RED);
    }
    //poke!(, 0);
}

fn reu_test() {
    reu::reu().fill(1024, 80, 65);
    // reu::reu().prepare(0x400, 0xaabbcc, 500);
    // reu::reu().swap_in();

}

fn alloc_test() {
    unsafe {
        let ptr: Ptr24 = reu::reu().alloc(70000);
        print!("{} bytes ", ptr.len);
        println!("allocated at: {}!", ptr.address);

        reu::reu().dealloc(ptr);
    }
}

fn wait_for_return() {
    unsafe {
        cbm_kernal::cbm_k_chrin();
    }
}

fn test_reu_slice() {
    // Allocate a cache that can hold 10 elements at a time
    let mut local_cache: [u32; 10] = [0; 10];
    let mut custom_slice = custom_slice::CustomSlice::new(&mut local_cache, 80000, 10);

    // Accessing elements (this will trigger the swap_in function as needed)
    println!("Setting Element 1 = 69");
    custom_slice[1]=69;
    println!("Setting Element 8 = 666");
    custom_slice[8]=666;

    println!("Getting at index 1: {}", custom_slice[1]);

    println!("Setting Element 70000 = 999");
    custom_slice[70000] = 999;
    println!("Getting at index 8: {}", custom_slice[8]);
    println!("Getting at index 70000: {}", custom_slice[70000]);
}

pub const HIRES: *const C64HiresScreen = (0xa000) as _;
//pub const SCREEN: &C64HiresScreen = unsafe { &*HIRES_SCREEN };
pub const COLORS: *const C64TextScreen = (0x8400) as _;

fn test_hires() {
    unsafe {
        (*HIRES).clear(0);
        (*COLORS).clear(33);
        //reu::reu().fill(0xa000, 8000, 0);
        plotek::show(VicBankSelect::VIC_8000, ScreenBank::AT_0400);

        for i in 0..32 {
            (*HIRES).line((0,0),(i*10,199));
        }
    
        wait_for_return();

        plotek::hide();
    }
}

#[start] // atrybut oznaczający entrypoint
fn _main(_argc: isize, _argv: *const *const u8) -> isize {

    //change_border();

    //test_hires();
    test_reu_slice();

    //box_test();
    0
}
