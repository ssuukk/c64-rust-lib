// tak się buduje:
// cargo build --target mos-c64-none 
// opcjonalnie dodać --release
// 
// kopiowanie wyniku:
// docker cp vigilant_beaver:/workspaces/rust-mos-hello-world-main/target/mos-c64-none/release/rust-mos-hello-world D:/temp/rust

// https://github.com/mlund/mos-hardware

#![no_std] // nie ładuj biblioteki std
#![feature(start)] // używamy niestandardowego entry pointu
//#![allow(unused_imports)] // nie pulta się
#![feature(panic_info_message)]

extern crate mos_alloc;

use core::panic::PanicInfo; // struktura zawierająca info o panic
use core::slice;
use reu::wallocator::{ WAllocator, Ptr24 };
use reu::reubox::ReuBox;
use ufmt_stdio::*; // stdio dla środowisk, które nie mają std
use mos_hardware::{c64, vic2, poke, cbm_kernal};
use vic2::*;
use plotek::{ EightBitCanvas, Drawable };

mod reu;
mod ultimate_64;
mod plotek;

pub const HIRES_SCREEN: *const EightBitCanvas = (0x2000) as _;
pub const SCREEN: &EightBitCanvas = unsafe { &*HIRES_SCREEN };

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

pub fn text_mode(prev: u8) {
    unsafe {
        let mut mask = c64::vic2().control_y.read();
        mask.set(ControlYFlags::BITMAP_MODE, false);
        c64::vic2().control_y.write(mask);
        c64::vic2().screen_and_charset_bank.write(prev);
    }
}

pub fn fill_hires_screen_reu(value: u8) {
    reu::reu().fill(0x2000, 40*8*4, value);
}

pub fn fill_text_screen(value: u8) {
    mem_array!(chars, 1024, 1000);

    for byte in chars.iter_mut() {
        *byte = value;
    }
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

fn box_test() {
    let mut my_box = ReuBox::new(42, 40000);

    my_box.put(1,44);

    my_box.put(1, 666);
    my_box.put(3999, 124231432);
    my_box.put(40000, 69);

    println!("My box array stored at: {}",my_box.ptr24.address);
    println!("at index 1 = {}", my_box[1]);
    println!("at index 3999 {}", my_box[3999]);
    println!("at index 40000 {}", my_box[40000]);
}

fn wait_for_return() {
    unsafe {
        cbm_kernal::cbm_k_chrin();
    }
}

#[start] // atrybut oznaczający entrypoint
fn _main(_argc: isize, _argv: *const *const u8) -> isize {

    //change_border();

    unsafe {
        SCREEN.show(true);
        SCREEN.clear(1);
        for i in 0..31 {
            SCREEN.line((0,0),(i*10,199));
        }
    
        //fill_hires_screen_reu(128+4);

        wait_for_return();

        SCREEN.show(false);
    }
    //box_test();
    0
}
