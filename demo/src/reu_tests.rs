use core::ptr;
use reu::ram_expansion_unit;
use reu::REUArray;
use ufmt_stdio::*; // stdio dla środowisk, które nie mają std
use mos_hardware::c64::sid;

pub fn alloc_test() {
    sid().start_random_generator();
    let chunk_count = sid().rand8(20);

    let reu = ram_expansion_unit::reu();
    reu.init_allocator();

    for _ in 0..chunk_count {
        let size = (sid().rand16(1000)*16+1) as u32;
        let chunk = reu.alloc(size);
        println!("got reu chunk: {:?}!", chunk);
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

pub fn test_reu_array() {
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
