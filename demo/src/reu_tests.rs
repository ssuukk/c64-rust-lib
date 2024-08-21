use reu::REUArray;
use reu::reu_allocator::{ Ptr24, WAllocator };
use reu::ram_expansion_unit;
use ufmt_stdio::*; // stdio dla środowisk, które nie mają std

pub fn reu_test() {
    ram_expansion_unit::reu().fill(1024, 80, 65);
    // reu::reu().prepare(0x400, 0xaabbcc, 500);
    // reu::reu().swap_in();
}

pub fn alloc_test() {
    unsafe {
        let ptr: Ptr24 = ram_expansion_unit::reu().alloc(70000);
        print!("{} bytes ", ptr.len);
        println!("allocated at: {}!", ptr.address);
        ram_expansion_unit::reu().dealloc(ptr);
    }
}

#[derive(Clone)]
struct GameUnit {
    number: u8,
    speed: u8,
    health: u8,
    x: u16,
    y: u16,
}

pub fn test_reu_slice() {
    // Allocate 100,000 GameUnit elements plus a cache that can hold 10 elements at a time
    let mut array = REUArray::<GameUnit>::new(100, 10);

    println!("Tablica {:?}", array);

    for i in 0..100 {
        array[i]=GameUnit { number: i as u8, speed: 1, health: 2, x: 3, y: 4 };
    }

    array[50].x = 66;

    // find all ead units
    // let dead_units = array.into_iter().filter(|unit| unit.x == 66);

    // // print coords of dead units
    // for u in dead_units {
    //     println!("{} Died at: ({},{})", u.number, u.x, u.y);
    // }
    for i in 0..100 {
        let test = &array[i];
        println!("numer:{} x,y=({},{})", test.number, test.x, test.y);
    }
}
