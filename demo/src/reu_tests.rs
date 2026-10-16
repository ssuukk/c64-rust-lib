use mos_hardware::c64::sid;
use reu::ram_expansion_unit;
use reu::REUArray;
use ufmt_stdio::*; // stdio dla środowisk, które nie mają std

pub fn alloc_test() {
    sid().start_random_generator();
    let chunk_count = sid().rand8(20);

    let reu = ram_expansion_unit::reu();
    reu.init_allocator(128 * 1024);

    for _ in 0..chunk_count {
        let size = (sid().rand16(1000) * 16 + 1) as u32;
        let chunk = reu.alloc(size);
        println!("got reu chunk: {:?}!", chunk);
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
struct GameUnit {
    number: u8,
    speed: u8,
    health: u8,
    padding: u8,
    x: u16,
    y: u16,
}

pub fn test_reu_array() {
    let reu = ram_expansion_unit::reu();
    reu.init_allocator(128 * 1024);

    assert!(core::mem::size_of::<GameUnit>() * 32 == 256);

    // Exact block alignment makes a partial final window adjacent to the guard chunk.
    let mut array = REUArray::<GameUnit>::with_capacity(32, 10);
    let guard = reu.alloc(256);
    reu.fill_reu(guard.address(), 256, 0xa5);

    for i in 0..32 {
        array.push(GameUnit {
            number: i as u8,
            speed: 1,
            health: 2,
            padding: 0,
            x: 3,
            y: 4,
        });
    }

    assert!(array.len() == 32);
    assert!(!array.is_empty());

    array.with_mut(5, |unit| {
        unit.speed = 0xa;
        unit.health = 0xb;
        unit.x = 69;
        unit.y = 11;
    });
    array.with_mut(25, |unit| unit.x = 69);
    array.set(
        10,
        GameUnit {
            number: 10,
            speed: 2,
            health: 3,
            padding: 0,
            x: 4,
            y: 5,
        },
    );
    array.for_each_mut(|_, unit| unit.speed = unit.speed.wrapping_add(1));

    assert!(array.get(5).x == 69);
    assert!(array.get(25).x == 69);
    assert!(array.get(10).speed == 3);

    array.for_each(|_, unit| {
        if unit.x == 69 {
            println!("{} Unit at x=69: ({},{})", unit.number, unit.x, unit.y);
        }
    });

    drop(array);

    let mut restored = REUArray::<GameUnit>::with_capacity(32, 10);
    assert!(restored.get(5).x == 69);
    assert!(restored.get(5).speed == 0xb);
    assert!(restored.get(10).speed == 3);
    assert!(restored.get(25).x == 69);
    drop(restored);

    let mut guard_data = [0; 256];
    guard.pull(reu, guard_data.as_mut_ptr() as usize);
    assert!(guard_data.iter().all(|value| *value == 0xa5));
}

pub fn test_reu_allocator() {
    let reu = ram_expansion_unit::reu();
    reu.init_allocator(128 * 1024);

    let first = reu.alloc(1);
    let second = reu.alloc(256);
    let third = reu.alloc(257);
    assert!(second.address() == first.address() + 256);
    assert!(third.address() == second.address() + 256);

    let freed_address = second.address();
    drop(second);

    let reused = reu.alloc(1);
    assert!(reused.address() == freed_address);

    drop(reused);
    drop(first);
    drop(third);
}
