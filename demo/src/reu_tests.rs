use core::ptr;
use reu::ram_expansion_unit;
use reu::ram_expansion_unit::RawAddress;
use reu::REUArray;
use ufmt_stdio::*; // stdio dla środowisk, które nie mają std

pub fn reu_test() {
    ram_expansion_unit::reu().fill(1024, 1000, 66);
    // reu::reu().set_range(0x400, 0xaabbcc, 500);
    // reu::reu().pull();
}

pub fn alloc_test() {
    unsafe {
        let reu = ram_expansion_unit::reu();
        reu.prepare_bom();
        let chunk = reu.alloc(1000);
        let chunk2 = reu.alloc(10000);
        chunk.push(reu, 1024);
        println!("got reu chunk: {:?}, {:?}!", chunk, chunk2);
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

pub fn test_memory() {
    let reu = ram_expansion_unit::reu();
    let mut t = 0xdeadbeefu32;

    let t_ptr = &t as *const u32;
    let t_addr = t.as_address();

    for i in 0..0x10 {
        let reu_addr = i * 0x100000;
        print!("reu at ${:x} = ", reu_addr);

        reu.set_range(t_addr, reu_addr, 1000);
        reu.push();
        t = 0;
        if t > 0 {}
        reu.pull();
        // Force the compiler to read `t` from memory
        let current_t = unsafe { ptr::read_volatile(t_ptr) };
        let reu_ok = current_t == 0xdeadbeefu32;
        println!("{}", reu_ok);
    }
}

pub fn test_reu_slice() {
    // Allocate 100,000 GameUnit elements plus a cache that can hold 10 elements at a time
    let mut array = REUArray::<GameUnit>::with_capacity(100, 10);

    for i in 0..100 {
        array.push(GameUnit {
            number: i as u8,
            speed: 1,
            health: 2,
            x: 3,
            y: 4,
        });
    }

    array[50].speed = 0xa;
    array[50].health = 0xb;
    array[50].x = 69;
    array[50].y = 11;

    let sixty_nine = array.iter_mut().filter(|unit| unit.x == 69);

    for u in sixty_nine {
        println!("{} Unit at x=69: ({},{})", u.number, u.x, u.y);
    }
}
